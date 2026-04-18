use blake2::{Blake2b512, Digest};
use std::{
	path::Path,
	time::{SystemTime, UNIX_EPOCH},
};

use super::{
	super::model::{CrrpContext, WalletSession},
	session::{load_wallet_session, save_wallet_session},
};

pub(super) async fn ensure_papp_session(
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
