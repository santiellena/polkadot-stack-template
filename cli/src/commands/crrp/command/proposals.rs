use super::{
	super::{
		args::ProposalsArgs,
		mock::{load_mock_state, mock_proposal_status_label, proposal_matches_filter, repo_key},
		model::Backend,
		preflight::preflight,
	},
	output::{backend_label, kv, line},
	CrrpResult,
};

pub(crate) async fn run_proposals(
	args: ProposalsArgs,
	eth_rpc_url_override: Option<&str>,
) -> CrrpResult {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;

	if ctx.backend == Backend::Mock {
		let state = load_mock_state(&ctx.repo_root)?;
		let repo_state = state.repos.get(&repo_key(ctx.repo_id)).cloned().unwrap_or_default();
		let mut printed = 0u16;

		line("CRRP Proposals (mock)");
		kv("Repository", ctx.repo_root.display());
		kv("Repo ID", format!("{:#x}", ctx.repo_id));
		kv("State filter", format!("{:?}", args.state));
		kv("Limit", args.limit);

		for (proposal_id, proposal) in &repo_state.proposals {
			if printed >= args.limit {
				break;
			}
			if !proposal_matches_filter(proposal.state, args.state) {
				continue;
			}

			line(format!(
				"[{}] state={} commit={} cid={}",
				proposal_id,
				mock_proposal_status_label(proposal.state),
				proposal.commit,
				proposal.cid
			));
			printed += 1;
		}

		if printed == 0 {
			line("No proposals matched the current filter.");
		}
		return Ok(());
	}

	line("CRRP Proposals (skeleton)");
	kv("Backend", backend_label(ctx.backend));
	kv("Repository", ctx.repo_root.display());
	kv("Repo ID", format!("{:#x}", ctx.repo_id));
	kv("Registry", ctx.registry);
	kv("State filter", format!("{:?}", args.state));
	kv("Limit", args.limit);
	line(format!(
		"On-chain proposal count: {} (detailed listing will be added in next iteration).",
		ctx.proposal_count
	));

	Ok(())
}
