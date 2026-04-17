pub mod chain;
pub mod config;
pub mod crrp;

use codec::Encode;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sp_core::{crypto::AccountId32, Pair};
use sp_statement_store::Statement;
use subxt::{dynamic::At, OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::{dev, Keypair};

// Matches the node-side statement store propagation limit.
const MAX_STATEMENT_STORE_ENCODED_SIZE: usize = 1024 * 1024 - 1;
const BULLETIN_MAX_UPLOAD_BYTES: usize = 8 * 1024 * 1024;

/// Resolve an sr25519 signer for the statement store from a flexible input.
pub fn resolve_statement_signer(
	input: &str,
) -> Result<sp_core::sr25519::Pair, Box<dyn std::error::Error>> {
	let uri = match input.to_lowercase().as_str() {
		"alice" => "//Alice",
		"bob" => "//Bob",
		"charlie" => "//Charlie",
		"dave" => "//Dave",
		"eve" => "//Eve",
		"ferdie" => "//Ferdie",
		_ => input,
	};

	sp_core::sr25519::Pair::from_string(uri, None)
		.map_err(|error| format!("Could not resolve statement signer {input}: {error}").into())
}

/// Resolve a Substrate tx signer from:
/// - named dev account: alice/bob/charlie/dave/eve/ferdie
/// - mnemonic phrase
/// - 0x-prefixed 32-byte secret seed
pub fn resolve_substrate_signer(input: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
	let lowered = input.to_lowercase();
	match lowered.as_str() {
		"alice" => Ok(dev::alice()),
		"bob" => Ok(dev::bob()),
		"charlie" => Ok(dev::charlie()),
		"dave" => Ok(dev::dave()),
		"eve" => Ok(dev::eve()),
		"ferdie" => Ok(dev::ferdie()),
		_ => {
			if input.contains(' ') {
				let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, input)?;
				return Ok(Keypair::from_phrase(&mnemonic, None)?);
			}

			if let Some(seed_hex) = input.strip_prefix("0x").or(input.strip_prefix("0X")) {
				let seed_bytes = hex::decode(seed_hex)?;
				if seed_bytes.len() != 32 {
					return Err("Secret seed must be 32 bytes (64 hex chars)".into());
				}
				let mut seed = [0u8; 32];
				seed.copy_from_slice(&seed_bytes);
				return Ok(Keypair::from_secret_key(seed)?);
			}

			Err(format!(
				"Unknown signer {input}. Use a dev account, mnemonic phrase, or 0x secret seed."
			)
			.into())
		},
	}
}

/// Upload bytes to Bulletin Chain via TransactionStorage.store and return extrinsic hash.
pub async fn upload_to_bulletin(
	file_bytes: &[u8],
	ws_url: &str,
	signer: &Keypair,
) -> Result<String, Box<dyn std::error::Error>> {
	if file_bytes.len() > BULLETIN_MAX_UPLOAD_BYTES {
		return Err(format!(
			"File too large ({} bytes). Bulletin max is {} bytes.",
			file_bytes.len(),
			BULLETIN_MAX_UPLOAD_BYTES
		)
		.into());
	}

	let api = OnlineClient::<PolkadotConfig>::from_url(ws_url).await?;
	ensure_bulletin_upload_authorization(&api, signer, file_bytes.len()).await?;
	let tx = subxt::dynamic::tx(
		"TransactionStorage",
		"store",
		vec![("data", subxt::dynamic::Value::from_bytes(file_bytes))],
	);
	let result = api
		.tx()
		.sign_and_submit_then_watch_default(&tx, signer)
		.await?
		.wait_for_finalized_success()
		.await?;

	Ok(format!("{}", result.extrinsic_hash()))
}

async fn ensure_bulletin_upload_authorization(
	api: &OnlineClient<PolkadotConfig>,
	signer: &Keypair,
	required_bytes: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	let account_bytes = signer.public_key().0;
	let account = AccountId32::new(account_bytes);
	let account_ss58 = account.to_string();

	let authorization = fetch_bulletin_authorization(api, account_bytes).await?;
	let Some((remaining_transactions, remaining_bytes)) = authorization else {
		return Err(format!(
			"Bulletin authorization missing for signer {account_ss58}. Authorize this account on Bulletin Paseo before uploading."
		)
		.into());
	};

	let required_bytes_u128 = required_bytes as u128;
	if remaining_transactions == 0 || remaining_bytes < required_bytes_u128 {
		return Err(format!(
			"Bulletin authorization insufficient for signer {account_ss58}: remaining transactions={remaining_transactions}, remaining bytes={remaining_bytes}, required bytes={required_bytes}. Renew or re-authorize this account on Bulletin Paseo."
		)
		.into());
	}

	Ok(())
}

async fn fetch_bulletin_authorization(
	api: &OnlineClient<PolkadotConfig>,
	account_bytes: [u8; 32],
) -> Result<Option<(u128, u128)>, Box<dyn std::error::Error>> {
	let account_variant_key = subxt::dynamic::Value::unnamed_variant(
		"Account",
		[subxt::dynamic::Value::from_bytes(account_bytes)],
	);
	match fetch_bulletin_authorization_with_key(api, account_variant_key).await {
		Ok(value) => Ok(value),
		Err(_) => {
			let raw_account_key = subxt::dynamic::Value::from_bytes(account_bytes);
			fetch_bulletin_authorization_with_key(api, raw_account_key).await
		},
	}
}

async fn fetch_bulletin_authorization_with_key(
	api: &OnlineClient<PolkadotConfig>,
	key: subxt::dynamic::Value<()>,
) -> Result<Option<(u128, u128)>, Box<dyn std::error::Error>> {
	let storage_query = subxt::dynamic::storage("TransactionStorage", "Authorizations", vec![key]);
	let maybe_auth = api.storage().at_latest().await?.fetch(&storage_query).await?;
	let Some(auth_value) = maybe_auth else {
		return Ok(None);
	};

	let auth = auth_value.to_value()?;
	let Some(extent) = decode_bulletin_authorization_extent(&auth) else {
		return Err("Could not decode TransactionStorage.Authorizations extent".into());
	};
	Ok(Some(extent))
}

fn decode_bulletin_authorization_extent<T>(
	auth: &subxt::dynamic::Value<T>,
) -> Option<(u128, u128)> {
	let extent = auth.at("extent").unwrap_or(auth);
	let transactions = extent.at("transactions")?.as_u128()?;
	let bytes = extent.at("bytes")?.as_u128()?;
	Some((transactions, bytes))
}

/// Submit file bytes to the local node's Statement Store via statement_submit RPC.
pub async fn submit_to_statement_store(
	url: &str,
	file_bytes: &[u8],
	signer: &sp_core::sr25519::Pair,
) -> Result<(), Box<dyn std::error::Error>> {
	println!("Submitting {} bytes to Statement Store...", file_bytes.len());

	let statement = build_signed_statement(file_bytes, signer);
	ensure_statement_store_size(&statement)?;

	let encoded = format!("0x{}", hex::encode(statement.encode()));
	let statement_hash = format!("0x{}", hex::encode(statement.hash()));

	rpc_call::<_, ()>(url, "statement_submit", vec![encoded]).await?;

	println!("Statement submitted to store.");
	println!("Statement hash: {statement_hash}");
	println!("Data bytes: {}", statement.data_len());

	Ok(())
}

fn build_signed_statement(file_bytes: &[u8], signer: &sp_core::sr25519::Pair) -> Statement {
	let mut statement = Statement::new();
	statement.set_plain_data(file_bytes.to_vec());
	statement.sign_sr25519_private(signer);
	statement
}

fn ensure_statement_store_size(statement: &Statement) -> Result<(), Box<dyn std::error::Error>> {
	let encoded_size = statement.encoded_size();
	if encoded_size > MAX_STATEMENT_STORE_ENCODED_SIZE {
		return Err(format!(
			"Statement is too large for node propagation ({encoded_size} encoded bytes, max {}). Choose a smaller file.",
			MAX_STATEMENT_STORE_ENCODED_SIZE
		)
		.into());
	}

	Ok(())
}

// --- Shared JSON-RPC helpers ---

pub fn rpc_url(url: &str) -> Result<Url, Box<dyn std::error::Error>> {
	let mut rpc_url = Url::parse(url)?;
	match rpc_url.scheme() {
		"ws" => rpc_url.set_scheme("http").expect("valid URL scheme conversion"),
		"wss" => rpc_url.set_scheme("https").expect("valid URL scheme conversion"),
		"http" | "https" => {},
		scheme => return Err(format!("Unsupported RPC URL scheme: {scheme}").into()),
	}
	Ok(rpc_url)
}

pub async fn rpc_call<P: Serialize, R: DeserializeOwned>(
	url: &str,
	method: &str,
	params: P,
) -> Result<R, Box<dyn std::error::Error>> {
	let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build()?;
	let response: RpcResponse = client
		.post(rpc_url(url)?)
		.json(&RpcRequest { jsonrpc: "2.0", id: 1u32, method, params })
		.send()
		.await?
		.json()
		.await?;

	match response.error {
		Some(error) => Err(error.to_string().into()),
		None => Ok(serde_json::from_value(response.result)?),
	}
}

#[derive(Serialize)]
pub struct RpcRequest<'a, P> {
	pub jsonrpc: &'static str,
	pub id: u32,
	pub method: &'a str,
	pub params: P,
}

#[derive(Deserialize)]
pub struct RpcResponse {
	#[serde(default)]
	pub result: serde_json::Value,
	#[serde(default)]
	pub error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
pub struct RpcError {
	pub code: i32,
	pub message: String,
	#[serde(default)]
	pub data: Option<serde_json::Value>,
}

impl std::fmt::Display for RpcError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.data {
			Some(data) => write!(f, "JSON-RPC error {}: {} ({data})", self.code, self.message),
			None => write!(f, "JSON-RPC error {}: {}", self.code, self.message),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::decode_bulletin_authorization_extent;
	use subxt::dynamic::Value;

	#[test]
	fn decodes_nested_bulletin_authorization_extent() {
		let auth = Value::named_composite([(
			"extent",
			Value::named_composite([
				("transactions", Value::u128(3)),
				("bytes", Value::u128(2048)),
			]),
		)]);
		let decoded = decode_bulletin_authorization_extent(&auth);
		assert_eq!(decoded, Some((3, 2048)));
	}

	#[test]
	fn decodes_flat_bulletin_authorization_extent() {
		let auth = Value::named_composite([
			("transactions", Value::u128(2)),
			("bytes", Value::u128(1024)),
		]);
		let decoded = decode_bulletin_authorization_extent(&auth);
		assert_eq!(decoded, Some((2, 1024)));
	}

	#[test]
	fn returns_none_for_invalid_bulletin_authorization_shape() {
		let auth = Value::named_composite([("extent", Value::u128(1))]);
		let decoded = decode_bulletin_authorization_extent(&auth);
		assert_eq!(decoded, None);
	}
}
