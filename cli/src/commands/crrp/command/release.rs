use super::{
	super::{
		args::ReleaseArgs,
		mock::{load_mock_state, mock_repo_state_mut, save_mock_state},
		model::Backend,
		preflight::preflight,
		wallet::ensure_wallet_session,
	},
	output::{kv, line},
	CrrpResult,
};

pub(crate) async fn run_release(
	args: ReleaseArgs,
	eth_rpc_url_override: Option<&str>,
) -> CrrpResult {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	if !args.dry_run {
		ensure_wallet_session(&ctx, "release creation").await?;
	}
	line(format!("Creating release {}...", args.version));
	kv("Repository", ctx.repo_root.display());
	kv("Repo ID", format!("{:#x}", ctx.repo_id));
	line("Skeleton: read canonical HEAD -> request wallet signature -> submit release.");
	if args.dry_run {
		line("Dry-run enabled: no signature/transaction executed.");
	} else if ctx.backend == Backend::Mock {
		let mut state = load_mock_state(&ctx.repo_root)?;
		let repo_state = mock_repo_state_mut(&mut state, ctx.repo_id);
		let release_id = repo_state.release_count;
		repo_state.release_count += 1;
		save_mock_state(&ctx.repo_root, &state)?;
		line(format!(
			"Mock backend: release {} recorded locally as #{}.",
			args.version, release_id
		));
	}

	Ok(())
}
