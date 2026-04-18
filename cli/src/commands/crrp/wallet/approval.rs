use blake2::{Blake2b512, Digest};
use std::{
	path::Path,
	time::{SystemTime, UNIX_EPOCH},
};

use super::{
	super::{args::WalletBackend, model::CrrpContext},
	papp::ensure_papp_session,
	session::load_wallet_session,
};

pub(crate) struct WalletTxApprovalReceipt {
	pub approval_id: String,
	pub approved_at_unix_secs: u64,
}

pub(crate) async fn request_wallet_tx_approval(
	ctx: &CrrpContext,
	action_label: &str,
	payload_summary: &str,
) -> Result<WalletTxApprovalReceipt, Box<dyn std::error::Error>> {
	let approved_at_unix_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
	let approval_id =
		build_approval_id(&ctx.repo_root, action_label, payload_summary, approved_at_unix_secs);

	match ctx.wallet_backend {
		WalletBackend::Mock => {
			println!("Mock wallet approval required for {action_label}.");
			println!("Payload summary: {payload_summary}");
			println!("Mock wallet approved transaction request {approval_id}.");
		},
		WalletBackend::Papp => {
			// Reuse the active papp session to avoid a duplicate QR/pairing prompt
			// within the same command execution. Actual tx signing is still pending.
			let session = if let Some(session) = load_wallet_session(&ctx.repo_root)? {
				session
			} else {
				ensure_papp_session(ctx, action_label).await?
			};

			println!("Wallet approval required for {action_label}.");
			println!("Payload summary: {payload_summary}");
			println!(
				"Using active papp session {} ({}).",
				session.session_id, session.wallet_label
			);
			println!("Wallet approval recorded locally ({approval_id}).");
		},
	}

	Ok(WalletTxApprovalReceipt { approval_id, approved_at_unix_secs })
}

fn build_approval_id(
	repo_root: &Path,
	action_label: &str,
	payload_summary: &str,
	approved_at_unix_secs: u64,
) -> String {
	let mut hasher = Blake2b512::new();
	hasher.update(repo_root.display().to_string().as_bytes());
	hasher.update(action_label.as_bytes());
	hasher.update(payload_summary.as_bytes());
	hasher.update(approved_at_unix_secs.to_le_bytes());
	let digest = hasher.finalize();
	hex::encode(&digest[..8])
}

#[cfg(test)]
mod tests {
	use super::build_approval_id;
	use std::path::Path;

	#[test]
	fn approval_id_is_deterministic_for_same_input() {
		let id_a = build_approval_id(Path::new("/tmp/repo"), "bulletin upload", "bytes=42", 123);
		let id_b = build_approval_id(Path::new("/tmp/repo"), "bulletin upload", "bytes=42", 123);
		assert_eq!(id_a, id_b);
	}

	#[test]
	fn approval_id_changes_when_payload_changes() {
		let id_a = build_approval_id(Path::new("/tmp/repo"), "bulletin upload", "bytes=42", 123);
		let id_b = build_approval_id(Path::new("/tmp/repo"), "bulletin upload", "bytes=43", 123);
		assert_ne!(id_a, id_b);
	}
}
