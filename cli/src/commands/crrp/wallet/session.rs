use blake2::{Blake2b512, Digest};
use std::{
	fs,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use super::super::model::WalletSession;

pub(crate) fn wallet_session_path(repo_root: &Path) -> PathBuf {
	repo_root.join(".crrp").join("wallet-session.json")
}

pub(crate) fn load_wallet_session(
	repo_root: &Path,
) -> Result<Option<WalletSession>, Box<dyn std::error::Error>> {
	let path = wallet_session_path(repo_root);
	if !path.exists() {
		return Ok(None);
	}
	let raw = fs::read_to_string(path)?;
	Ok(Some(serde_json::from_str(&raw)?))
}

pub(super) fn save_wallet_session(
	repo_root: &Path,
	session: &WalletSession,
) -> Result<(), Box<dyn std::error::Error>> {
	let dir = repo_root.join(".crrp");
	fs::create_dir_all(&dir)?;
	let path = wallet_session_path(repo_root);
	fs::write(path, serde_json::to_string_pretty(session)? + "\n")?;
	Ok(())
}

pub(super) fn ensure_mock_wallet_session(
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
		local_account_id_hex: None,
		remote_account_id_hex: None,
		shared_secret_hex: None,
		local_secret_hex: None,
		local_entropy_hex: None,
		metadata_url: None,
	})
}

fn session_uri(session: &WalletSession) -> String {
	format!("crrp://wallet-connect?session={}&wallet={}", session.session_id, session.wallet_label)
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
