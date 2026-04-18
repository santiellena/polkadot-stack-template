use super::{
	super::{
		args::ReviewArgs, model::Backend, preflight::preflight, wallet::ensure_wallet_session,
	},
	output::{kv, line},
	CrrpResult,
};

pub(crate) async fn run_review(args: ReviewArgs, eth_rpc_url_override: Option<&str>) -> CrrpResult {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	ensure_wallet_session(&ctx, "review submission").await?;
	line(format!("Reviewing proposal {}...", args.proposal_id));
	kv("Repository", ctx.repo_root.display());
	kv("Repo ID", format!("{:#x}", ctx.repo_id));
	kv("Decision", format!("{:?}", args.decision));
	line("Skeleton: request wallet signature -> submit on-chain review.");
	if ctx.backend == Backend::Mock {
		line("Mock backend: review accepted locally (no transaction submitted).");
	}
	Ok(())
}
