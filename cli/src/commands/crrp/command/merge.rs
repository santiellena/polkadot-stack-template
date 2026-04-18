use super::{
	super::{
		args::MergeArgs,
		git::git_output,
		mock::{load_mock_state, mock_repo_state_mut, save_mock_state},
		model::{Backend, MockProposalStatus},
		preflight::preflight,
		wallet::ensure_wallet_session,
	},
	output::{kv, line, steps},
	CrrpResult,
};

pub(crate) async fn run_merge(args: MergeArgs, eth_rpc_url_override: Option<&str>) -> CrrpResult {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	if !args.dry_run {
		ensure_wallet_session(&ctx, "proposal merge").await?;
	}
	let head = git_output(&ctx.repo_root, &["rev-parse", "HEAD"])?;

	line(format!("Merging proposal {}...", args.proposal_id));
	kv("Repository", ctx.repo_root.display());
	kv("Repo ID", format!("{:#x}", ctx.repo_id));
	kv("Current local HEAD", head);
	steps(
		"Next steps (skeleton):",
		&[
			"Fetch proposal bundle",
			"Merge locally with Git and resolve conflicts",
			"Create final bundle and upload for CID",
			"Request wallet signature",
			"Submit merge transaction (update canonical HEAD)",
		],
	);
	if args.dry_run {
		line("Dry-run enabled: no upload/signature/transaction executed.");
	} else if ctx.backend == Backend::Mock {
		let mut state = load_mock_state(&ctx.repo_root)?;
		let repo_state = mock_repo_state_mut(&mut state, ctx.repo_id);
		let proposal = repo_state.proposals.get_mut(&args.proposal_id).ok_or_else(|| {
			format!("Mock backend: proposal {} not found for this repo.", args.proposal_id)
		})?;
		if proposal.state != MockProposalStatus::Open {
			return Err(format!(
				"Mock backend: proposal {} is not open for merge.",
				args.proposal_id
			)
			.into());
		}

		proposal.state = MockProposalStatus::Merged;
		let merged_cid = proposal.cid.clone();
		repo_state.head_cid = merged_cid.clone();
		save_mock_state(&ctx.repo_root, &state)?;
		line(format!(
			"Mock backend: proposal {} marked merged locally. HEAD CID set to {}.",
			args.proposal_id, merged_cid
		));
	}

	Ok(())
}
