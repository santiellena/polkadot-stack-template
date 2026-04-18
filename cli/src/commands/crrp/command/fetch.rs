use std::{fs, path::PathBuf};

use super::{
	super::{
		args::FetchArgs,
		mock::{load_mock_state, repo_key, resolve_fetch_destination},
		model::Backend,
		preflight::preflight,
	},
	output::{kv, line},
	CrrpResult,
};

pub(crate) async fn run_fetch(args: FetchArgs, eth_rpc_url_override: Option<&str>) -> CrrpResult {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	let into = args
		.into
		.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

	if ctx.backend == Backend::Mock {
		let state = load_mock_state(&ctx.repo_root)?;
		let repo_state = state
			.repos
			.get(&repo_key(ctx.repo_id))
			.ok_or_else(|| format!("Mock backend: repo {:#x} has no proposals.", ctx.repo_id))?;
		let proposal = repo_state.proposals.get(&args.proposal_id).ok_or_else(|| {
			format!("Mock backend: proposal {} not found for this repo.", args.proposal_id)
		})?;
		let source = ctx.repo_root.join(&proposal.bundle_path);
		if !source.exists() {
			return Err(format!(
				"Mock backend: stored bundle for proposal {} is missing at {}.",
				args.proposal_id,
				source.display()
			)
			.into());
		}

		let destination = resolve_fetch_destination(&into, args.proposal_id, &proposal.commit);
		let destination_dir = destination
			.parent()
			.ok_or_else(|| format!("Invalid fetch destination: {}", destination.display()))?;
		fs::create_dir_all(destination_dir)?;
		fs::copy(&source, &destination)?;

		line(format!("Fetched proposal {}.", args.proposal_id));
		kv("Repository", ctx.repo_root.display());
		kv("Repo ID", format!("{:#x}", ctx.repo_id));
		kv("Mock CID", &proposal.cid);
		kv("Source bundle", source.display());
		kv("Copied to", destination.display());
		return Ok(());
	}

	line(format!("Fetching proposal {}...", args.proposal_id));
	kv("Repository", ctx.repo_root.display());
	kv("Repo ID", format!("{:#x}", ctx.repo_id));
	kv("Target directory", into.display());
	line("Skeleton: resolve proposal CID -> download bundle -> import into local Git.");
	Ok(())
}
