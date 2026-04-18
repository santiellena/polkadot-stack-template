mod create_repo;
mod fetch;
mod merge;
mod output;
mod proposals;
mod propose;
mod release;
mod repo;
mod review;
mod status;

use std::error::Error;

use super::args::CrrpAction;

pub(super) use create_repo::run_create_repo;
pub(super) use fetch::run_fetch;
pub(super) use merge::run_merge;
pub(super) use proposals::run_proposals;
pub(super) use propose::run_propose;
pub(super) use release::run_release;
pub(super) use repo::run_repo;
pub(super) use review::run_review;
pub(super) use status::run_status;

pub(super) type CrrpResult<T = ()> = Result<T, Box<dyn Error>>;

pub async fn run(action: CrrpAction, eth_rpc_url_override: Option<&str>) -> CrrpResult {
	match action {
		CrrpAction::CreateRepo(args) => run_create_repo(args, eth_rpc_url_override).await?,
		CrrpAction::Propose(args) => run_propose(args, eth_rpc_url_override).await?,
		CrrpAction::Fetch(args) => run_fetch(args, eth_rpc_url_override).await?,
		CrrpAction::Review(args) => run_review(args, eth_rpc_url_override).await?,
		CrrpAction::Merge(args) => run_merge(args, eth_rpc_url_override).await?,
		CrrpAction::Release(args) => run_release(args, eth_rpc_url_override).await?,
		CrrpAction::Status(args) => run_status(args, eth_rpc_url_override).await?,
		CrrpAction::Repo(args) => run_repo(args, eth_rpc_url_override).await?,
		CrrpAction::Proposals(args) => run_proposals(args, eth_rpc_url_override).await?,
	}

	Ok(())
}
