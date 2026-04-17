use blake2::{Blake2b512, Digest};
use std::{
	fs,
	path::{Path, PathBuf},
	process::Command,
	time::{SystemTime, UNIX_EPOCH},
};

use super::model::{MockBundleSubmission, PreparedProposal};

pub(super) fn detect_repo_root(
	repo_override: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
	let cwd =
		if let Some(path) = repo_override { path.to_path_buf() } else { std::env::current_dir()? };

	let output = Command::new("git")
		.arg("rev-parse")
		.arg("--show-toplevel")
		.current_dir(cwd)
		.output()?;
	if !output.status.success() {
		return Err("Not inside a Git repository".into());
	}

	Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
}

pub(super) fn git_output(
	repo_root: &Path,
	args: &[&str],
) -> Result<String, Box<dyn std::error::Error>> {
	let output = Command::new("git").args(args).current_dir(repo_root).output()?;
	if !output.status.success() {
		let stderr = String::from_utf8(output.stderr)?;
		return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()).into());
	}
	Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub(super) fn prepare_proposal(
	repo_root: &Path,
	commit_override: Option<&str>,
) -> Result<PreparedProposal, Box<dyn std::error::Error>> {
	let requested = commit_override
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.unwrap_or("HEAD");
	let commit = git_output(repo_root, &["rev-parse", "--verify", requested])?;
	let base_commit = git_output(repo_root, &["rev-parse", &format!("{commit}^")]).ok();

	Ok(PreparedProposal { commit, base_commit })
}

pub(super) fn create_mock_bundle_submission(
	repo_root: &Path,
	proposal: &PreparedProposal,
) -> Result<MockBundleSubmission, Box<dyn std::error::Error>> {
	let staging_dir = repo_root.join(".crrp").join("tmp");
	fs::create_dir_all(&staging_dir)?;
	let short_commit = short_commit_id(&proposal.commit);
	let staged_bundle = staging_dir.join(format!("proposal-{short_commit}.bundle"));
	write_git_bundle(repo_root, proposal, &staged_bundle)?;

	let bundle_bytes = fs::read(&staged_bundle)?;
	let cid = mock_cid_for_bytes(&bundle_bytes);
	let bulletin_dir = mock_bulletin_dir(repo_root);
	fs::create_dir_all(&bulletin_dir)?;
	let final_bundle = bulletin_dir.join(format!("{cid}.bundle"));

	if final_bundle.exists() {
		fs::remove_file(&staged_bundle)?;
	} else {
		fs::rename(&staged_bundle, &final_bundle)?;
	}

	Ok(MockBundleSubmission { cid, bundle_path: final_bundle })
}

fn write_git_bundle(
	repo_root: &Path,
	proposal: &PreparedProposal,
	destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
	let temp_ref = format!("refs/crrp/proposals/{}", short_commit_id(&proposal.commit));
	let create_ref = Command::new("git")
		.arg("update-ref")
		.arg(&temp_ref)
		.arg(&proposal.commit)
		.current_dir(repo_root)
		.output()?;
	if !create_ref.status.success() {
		let stderr = String::from_utf8(create_ref.stderr)?;
		return Err(format!("git update-ref failed: {}", stderr.trim()).into());
	}

	let mut command = Command::new("git");
	command.arg("bundle").arg("create").arg(destination);
	// MVP invariant: proposal bundles must be self-contained snapshots.
	// Always include the full reachable history for the selected ref.
	command.arg(&temp_ref);
	let output = command.current_dir(repo_root).output()?;
	let _ = Command::new("git")
		.arg("update-ref")
		.arg("-d")
		.arg(&temp_ref)
		.current_dir(repo_root)
		.output();
	if !output.status.success() {
		let stderr = String::from_utf8(output.stderr)?;
		return Err(format!("git bundle create failed: {}", stderr.trim()).into());
	}
	Ok(())
}

fn mock_cid_for_bytes(bytes: &[u8]) -> String {
	let digest = Blake2b512::digest(bytes);
	format!("mockcid-{}", hex::encode(&digest[..16]))
}

fn mock_bulletin_dir(repo_root: &Path) -> PathBuf {
	repo_root.join(".crrp").join("bulletins")
}

pub(super) fn relative_repo_path(repo_root: &Path, path: &Path) -> String {
	path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().to_string()
}

pub(super) fn short_commit_id(commit: &str) -> &str {
	commit.get(..12).unwrap_or(commit)
}

pub(super) fn unix_timestamp_secs() -> Result<u64, Box<dyn std::error::Error>> {
	Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
