use alloy::primitives::FixedBytes;
use std::{
	fs,
	path::{Path, PathBuf},
};

use super::{
	args::ProposalStateFilter,
	model::{MockProposalStatus, MockRepoState, MockState},
};

pub(super) fn repo_key(repo_id: FixedBytes<32>) -> String {
	format!("{:#x}", repo_id)
}

fn mock_state_path(repo_root: &Path) -> PathBuf {
	repo_root.join(".crrp").join("mock-state.json")
}

pub(super) fn load_mock_state(repo_root: &Path) -> Result<MockState, Box<dyn std::error::Error>> {
	let path = mock_state_path(repo_root);
	if !path.exists() {
		return Ok(MockState::default());
	}

	let raw = fs::read_to_string(path)?;
	Ok(serde_json::from_str(&raw)?)
}

pub(super) fn save_mock_state(
	repo_root: &Path,
	state: &MockState,
) -> Result<(), Box<dyn std::error::Error>> {
	let dir = repo_root.join(".crrp");
	fs::create_dir_all(&dir)?;
	let path = mock_state_path(repo_root);
	fs::write(path, serde_json::to_string_pretty(state)? + "\n")?;
	Ok(())
}

pub(super) fn mock_repo_state_mut(
	state: &mut MockState,
	repo_id: FixedBytes<32>,
) -> &mut MockRepoState {
	state.repos.entry(repo_key(repo_id)).or_default()
}

pub(super) fn resolve_fetch_destination(into: &Path, proposal_id: u64, commit: &str) -> PathBuf {
	if looks_like_bundle_file(into) {
		return into.to_path_buf();
	}

	into.join(format!("proposal-{proposal_id}-{}.bundle", super::git::short_commit_id(commit)))
}

fn looks_like_bundle_file(path: &Path) -> bool {
	path.extension()
		.and_then(|value| value.to_str())
		.map(|value| value.eq_ignore_ascii_case("bundle"))
		.unwrap_or(false)
}

pub(super) fn proposal_matches_filter(
	state: MockProposalStatus,
	filter: Option<ProposalStateFilter>,
) -> bool {
	match filter {
		None => true,
		Some(ProposalStateFilter::Open) => state == MockProposalStatus::Open,
		Some(ProposalStateFilter::Rejected) => state == MockProposalStatus::Rejected,
		Some(ProposalStateFilter::Merged) => state == MockProposalStatus::Merged,
	}
}

pub(super) fn mock_proposal_status_label(state: MockProposalStatus) -> &'static str {
	match state {
		MockProposalStatus::Open => "open",
		MockProposalStatus::Rejected => "rejected",
		MockProposalStatus::Merged => "merged",
	}
}
