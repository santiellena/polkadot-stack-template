use std::{fs, path::PathBuf};

use crate::commands::{resolve_substrate_signer, upload_to_bulletin};

use super::{
	args::{
		CrrpAction, FetchArgs, MergeArgs, ProposalsArgs, ProposeArgs, ReleaseArgs, RepoArgs,
		ReviewArgs, StatusArgs,
	},
	git::{
		create_mock_bundle_submission, git_output, prepare_proposal, relative_repo_path,
		unix_timestamp_secs,
	},
	mock::{
		load_mock_state, mock_proposal_status_label, mock_repo_state_mut, proposal_matches_filter,
		repo_key, resolve_fetch_destination, save_mock_state,
	},
	model::{Backend, MockProposalState, MockProposalStatus},
	preflight::preflight,
	wallet::ensure_wallet_session,
};

pub async fn run(
	action: CrrpAction,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	match action {
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

pub(super) async fn run_propose(
	args: ProposeArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	let proposal = prepare_proposal(&ctx.repo_root, args.commit.as_deref())?;

	println!("Preparing proposal...");
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Registry: {}", ctx.registry);
	println!("Selected commit: {}", proposal.commit);
	match proposal.base_commit.as_deref() {
		Some(base_commit) => println!("Bundle base commit: {base_commit}"),
		None => println!("Bundle base commit: <root commit>"),
	}
	println!("Next steps:");
	println!("1. Create Git bundle artifact for selected commit");
	println!("2. Submit bundle to Bulletin abstraction and obtain mock CID");
	println!("3. Reuse or establish wallet session");
	println!("4. Record proposal submission in selected backend");
	if args.dry_run {
		println!("Dry-run enabled: no bundle/upload/signature/transaction executed.");
	} else if ctx.backend == Backend::Mock {
		let bundle_submission = create_mock_bundle_submission(&ctx.repo_root, &proposal)?;
		let wallet_session = ensure_wallet_session(&ctx, "proposal submission").await?;
		let mut state = load_mock_state(&ctx.repo_root)?;
		let repo_state = mock_repo_state_mut(&mut state, ctx.repo_id);
		let proposal_id = repo_state.proposal_count;
		repo_state.proposal_count += 1;
		repo_state.proposals.insert(
			proposal_id,
			MockProposalState {
				commit: proposal.commit.clone(),
				base_commit: proposal.base_commit.clone(),
				cid: bundle_submission.cid.clone(),
				bundle_path: relative_repo_path(&ctx.repo_root, &bundle_submission.bundle_path),
				state: MockProposalStatus::Open,
				submitted_at_unix_secs: unix_timestamp_secs()?,
			},
		);
		save_mock_state(&ctx.repo_root, &state)?;
		println!("Mock backend: proposal submitted successfully.");
		println!("Proposal ID: {proposal_id}");
		println!("Bundle path: {}", bundle_submission.bundle_path.display());
		println!("Mock CID: {}", bundle_submission.cid);
		println!("Wallet session: {}", wallet_session.session_id);
	} else {
		let bundle_submission = create_mock_bundle_submission(&ctx.repo_root, &proposal)?;
		let wallet_session = ensure_wallet_session(&ctx, "proposal submission").await?;
		let signer_input = args.common.bulletin_signer.as_deref().ok_or(
			"Missing --bulletin-signer for non-mock propose. Provide a dev account, mnemonic phrase, or 0x secret seed for Bulletin upload.",
		)?;
		let signer = resolve_substrate_signer(signer_input)?;
		let bundle_bytes = fs::read(&bundle_submission.bundle_path)?;
		let extrinsic_hash =
			upload_to_bulletin(&bundle_bytes, &ctx.substrate_rpc_ws, &signer).await?;

		println!("Bulletin upload submitted.");
		println!("Bulletin RPC: {}", ctx.substrate_rpc_ws);
		println!("Bulletin extrinsic hash: {extrinsic_hash}");
		println!("Local bundle path: {}", bundle_submission.bundle_path.display());
		println!("Local CID placeholder: {}", bundle_submission.cid);
		println!("Wallet session: {}", wallet_session.session_id);
		println!(
			"Contract submission step still pending: use proposal commit + Bulletin-backed artifact reference."
		);
	}

	Ok(())
}

pub(super) async fn run_fetch(
	args: FetchArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
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

		println!("Fetched proposal {}.", args.proposal_id);
		println!("Repository: {}", ctx.repo_root.display());
		println!("Repo ID: {:#x}", ctx.repo_id);
		println!("Mock CID: {}", proposal.cid);
		println!("Source bundle: {}", source.display());
		println!("Copied to: {}", destination.display());
		return Ok(());
	}

	println!("Fetching proposal {}...", args.proposal_id);
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Target directory: {}", into.display());
	println!("Skeleton: resolve proposal CID -> download bundle -> import into local Git.");
	Ok(())
}

pub(super) async fn run_review(
	args: ReviewArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	ensure_wallet_session(&ctx, "review submission").await?;
	println!("Reviewing proposal {}...", args.proposal_id);
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Decision: {:?}", args.decision);
	println!("Skeleton: request wallet signature -> submit on-chain review.");
	if ctx.backend == Backend::Mock {
		println!("Mock backend: review accepted locally (no transaction submitted).");
	}
	Ok(())
}

pub(super) async fn run_merge(
	args: MergeArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	if !args.dry_run {
		ensure_wallet_session(&ctx, "proposal merge").await?;
	}
	let head = git_output(&ctx.repo_root, &["rev-parse", "HEAD"])?;

	println!("Merging proposal {}...", args.proposal_id);
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Current local HEAD: {head}");
	println!("Next steps (skeleton):");
	println!("1. Fetch proposal bundle");
	println!("2. Merge locally with Git and resolve conflicts");
	println!("3. Create final bundle and upload for CID");
	println!("4. Request wallet signature");
	println!("5. Submit merge transaction (update canonical HEAD)");
	if args.dry_run {
		println!("Dry-run enabled: no upload/signature/transaction executed.");
	} else if ctx.backend == Backend::Mock {
		let mut state = load_mock_state(&ctx.repo_root)?;
		let repo_state = mock_repo_state_mut(&mut state, ctx.repo_id);
		let proposal = repo_state.proposals.get_mut(&args.proposal_id).ok_or_else(|| {
			format!("Mock backend: proposal {} not found for this repo.", args.proposal_id)
		})?;
		if proposal.state != MockProposalStatus::Open {
			return Err(format!(
				"Mock backend: proposal {} is not open for merge.",
				args.proposal_id,
			)
			.into());
		}

		proposal.state = MockProposalStatus::Merged;
		let merged_cid = proposal.cid.clone();
		repo_state.head_cid = merged_cid.clone();
		save_mock_state(&ctx.repo_root, &state)?;
		println!(
			"Mock backend: proposal {} marked merged locally. HEAD CID set to {}.",
			args.proposal_id, merged_cid
		);
	}

	Ok(())
}

pub(super) async fn run_release(
	args: ReleaseArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	if !args.dry_run {
		ensure_wallet_session(&ctx, "release creation").await?;
	}
	println!("Creating release {}...", args.version);
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Skeleton: read canonical HEAD -> request wallet signature -> submit release.");
	if args.dry_run {
		println!("Dry-run enabled: no signature/transaction executed.");
	} else if ctx.backend == Backend::Mock {
		let mut state = load_mock_state(&ctx.repo_root)?;
		let repo_state = mock_repo_state_mut(&mut state, ctx.repo_id);
		let release_id = repo_state.release_count;
		repo_state.release_count += 1;
		save_mock_state(&ctx.repo_root, &state)?;
		println!("Mock backend: release {} recorded locally as #{}.", args.version, release_id);
	}

	Ok(())
}

pub(super) async fn run_status(
	args: StatusArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	let branch = git_output(&ctx.repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
	let local_head = git_output(&ctx.repo_root, &["rev-parse", "HEAD"])?;

	println!("CRRP Status (skeleton)");
	println!("Backend: {}", if ctx.backend == Backend::Mock { "mock" } else { "rpc" });
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Registry: {}", ctx.registry);
	println!("Branch: {branch}");
	println!("Local HEAD: {local_head}");
	println!("On-chain HEAD: {:#x}", ctx.head_commit);
	println!("On-chain HEAD CID: {}", ctx.head_cid);
	println!("On-chain proposals: {}", ctx.proposal_count);
	println!("On-chain releases: {}", ctx.release_count);

	Ok(())
}

pub(super) async fn run_repo(
	args: RepoArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	println!("CRRP Repo (skeleton)");
	println!("Backend: {}", if ctx.backend == Backend::Mock { "mock" } else { "rpc" });
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Registry: {}", ctx.registry);
	println!("Maintainer: {}", ctx.maintainer);
	println!("On-chain HEAD: {:#x}", ctx.head_commit);
	println!("On-chain HEAD CID: {}", ctx.head_cid);
	println!("Proposals: {}", ctx.proposal_count);
	println!("Releases: {}", ctx.release_count);

	Ok(())
}

pub(super) async fn run_proposals(
	args: ProposalsArgs,
	eth_rpc_url_override: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;

	if ctx.backend == Backend::Mock {
		let state = load_mock_state(&ctx.repo_root)?;
		let repo_state = state.repos.get(&repo_key(ctx.repo_id)).cloned().unwrap_or_default();
		let mut printed = 0u16;

		println!("CRRP Proposals (mock)");
		println!("Repository: {}", ctx.repo_root.display());
		println!("Repo ID: {:#x}", ctx.repo_id);
		println!("State filter: {:?}", args.state);
		println!("Limit: {}", args.limit);

		for (proposal_id, proposal) in &repo_state.proposals {
			if printed >= args.limit {
				break;
			}
			if !proposal_matches_filter(proposal.state, args.state) {
				continue;
			}

			println!(
				"[{}] state={} commit={} cid={}",
				proposal_id,
				mock_proposal_status_label(proposal.state),
				proposal.commit,
				proposal.cid
			);
			printed += 1;
		}

		if printed == 0 {
			println!("No proposals matched the current filter.");
		}
		return Ok(());
	}

	println!("CRRP Proposals (skeleton)");
	println!("Backend: {}", if ctx.backend == Backend::Mock { "mock" } else { "rpc" });
	println!("Repository: {}", ctx.repo_root.display());
	println!("Repo ID: {:#x}", ctx.repo_id);
	println!("Registry: {}", ctx.registry);
	println!("State filter: {:?}", args.state);
	println!("Limit: {}", args.limit);
	println!(
		"On-chain proposal count: {} (detailed listing will be added in next iteration).",
		ctx.proposal_count
	);

	Ok(())
}
