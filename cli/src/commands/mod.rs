pub mod chain;
pub mod config;
pub mod crrp;

use codec::Encode;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sp_core::Pair;
use sp_statement_store::Statement;

// Matches the node-side statement store propagation limit.
const MAX_STATEMENT_STORE_ENCODED_SIZE: usize = 1024 * 1024 - 1;

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
