use blake2::{Blake2b512, Digest};
use std::{
	fs,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use super::{
	args::WalletBackend,
	model::{CrrpContext, WalletSession},
};

fn wallet_session_path(repo_root: &Path) -> PathBuf {
	repo_root.join(".crrp").join("wallet-session.json")
}

pub(super) fn load_wallet_session(
	repo_root: &Path,
) -> Result<Option<WalletSession>, Box<dyn std::error::Error>> {
	let path = wallet_session_path(repo_root);
	if !path.exists() {
		return Ok(None);
	}
	let raw = fs::read_to_string(path)?;
	Ok(Some(serde_json::from_str(&raw)?))
}

fn save_wallet_session(
	repo_root: &Path,
	session: &WalletSession,
) -> Result<(), Box<dyn std::error::Error>> {
	let dir = repo_root.join(".crrp");
	fs::create_dir_all(&dir)?;
	let path = wallet_session_path(repo_root);
	fs::write(path, serde_json::to_string_pretty(session)? + "\n")?;
	Ok(())
}

pub(super) async fn ensure_wallet_session(
	ctx: &CrrpContext,
	action_label: &str,
) -> Result<WalletSession, Box<dyn std::error::Error>> {
	match ctx.wallet_backend {
		WalletBackend::Mock => ensure_mock_wallet_session(&ctx.repo_root, action_label),
		WalletBackend::Papp => ensure_papp_session(ctx, action_label).await,
	}
}

fn ensure_mock_wallet_session(
	repo_root: &Path,
	action_label: &str,
) -> Result<WalletSession, Box<dyn std::error::Error>> {
	if let Some(session) = load_wallet_session(repo_root)? {
		println!(
			"Wallet session active ({}). Continuing with {}.",
			session.session_id, action_label
		);
		return Ok(session);
	}

	println!("Wallet sign-in required for {}.", action_label);
	println!("Scan this QR with your phone wallet to sign in:");
	let session = create_mock_wallet_session(repo_root)?;
	let uri = session_uri(&session);
	print_mock_qr(&uri);
	println!("Sign-in URI: {uri}");
	save_wallet_session(repo_root, &session)?;
	println!("Wallet connected (mock session {}).", session.session_id);
	Ok(session)
}

async fn ensure_papp_session(
	ctx: &CrrpContext,
	action_label: &str,
) -> Result<WalletSession, Box<dyn std::error::Error>> {
	if let Some(session) = load_wallet_session(&ctx.repo_root)? {
		println!(
			"Wallet session active ({} via {}). Continuing with {}.",
			session.session_id, session.wallet_label, action_label
		);
		return Ok(session);
	}

	let metadata = ctx
		.papp_term_metadata
		.as_deref()
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.unwrap_or(papp_term::DEFAULT_METADATA);

	let endpoint_values: Vec<String> = ctx
		.papp_term_endpoint
		.as_deref()
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.map(|value| vec![value.to_string()])
		.unwrap_or_default();
	let endpoints = papp_term::resolve_endpoints(&endpoint_values);

	println!("Wallet sign-in required for {}.", action_label);
	println!("Launching papp-terminal TUI...");
	papp_term::tui::run_tui(metadata, &endpoints).await.map_err(|error| {
		format!(
			"papp-terminal library flow failed while requesting wallet sign-in for {action_label}: {error}"
		)
	})?;

	let session = create_papp_wallet_session(&ctx.repo_root, ctx.papp_term_endpoint.as_deref())?;
	save_wallet_session(&ctx.repo_root, &session)?;
	println!("Wallet connected via papp-terminal (session {}).", session.session_id);
	Ok(session)
}

fn create_mock_wallet_session(
	repo_root: &Path,
) -> Result<WalletSession, Box<dyn std::error::Error>> {
	let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
	let mut hasher = Blake2b512::new();
	hasher.update(repo_root.display().to_string().as_bytes());
	hasher.update(now.to_le_bytes());
	let digest = hasher.finalize();
	let session_id = hex::encode(&digest[..8]);

	Ok(WalletSession {
		backend: "mock".to_string(),
		session_id,
		created_at_unix_secs: now,
		wallet_label: "mock-wallet".to_string(),
		chain: None,
		accounts: Vec::new(),
	})
}

fn session_uri(session: &WalletSession) -> String {
	format!("crrp://wallet-connect?session={}&wallet={}", session.session_id, session.wallet_label)
}

fn create_papp_wallet_session(
	repo_root: &Path,
	endpoint: Option<&str>,
) -> Result<WalletSession, Box<dyn std::error::Error>> {
	let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
	let mut hasher = Blake2b512::new();
	hasher.update(repo_root.display().to_string().as_bytes());
	hasher.update(now.to_le_bytes());
	if let Some(endpoint) = endpoint {
		hasher.update(endpoint.trim().as_bytes());
	}
	let digest = hasher.finalize();
	let session_id = hex::encode(&digest[..8]);

	Ok(WalletSession {
		backend: "papp".to_string(),
		session_id,
		created_at_unix_secs: now,
		wallet_label: "papp-terminal".to_string(),
		chain: endpoint.map(|value| value.to_string()),
		accounts: Vec::new(),
	})
}

fn print_mock_qr(payload: &str) {
	let size = 25usize;
	let mut bits = Vec::with_capacity(size * size);
	let mut counter = 0u64;

	while bits.len() < size * size {
		let mut hasher = Blake2b512::new();
		hasher.update(payload.as_bytes());
		hasher.update(counter.to_le_bytes());
		let digest = hasher.finalize();
		for byte in digest {
			for bit in 0..8 {
				bits.push(((byte >> bit) & 1) == 1);
				if bits.len() == size * size {
					break;
				}
			}
			if bits.len() == size * size {
				break;
			}
		}
		counter += 1;
	}

	println!("Mock QR:");
	for y in 0..(size + 4) {
		let mut line = String::with_capacity((size + 4) * 2);
		for x in 0..(size + 4) {
			let dark = if x < 2 || y < 2 || x >= size + 2 || y >= size + 2 {
				true
			} else {
				bits[(y - 2) * size + (x - 2)]
			};
			line.push_str(if dark { "██" } else { "  " });
		}
		println!("{line}");
	}
}
