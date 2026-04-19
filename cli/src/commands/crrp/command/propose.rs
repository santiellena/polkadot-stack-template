use std::fs;

use crate::commands::{
	ensure_bulletin_upload_capability, resolve_substrate_signer, upload_to_bulletin,
};

use super::{
	super::{
		args::ProposeArgs,
		git::{
			create_mock_bundle_submission, prepare_proposal, relative_repo_path,
			unix_timestamp_secs,
		},
		mock::{load_mock_state, mock_repo_state_mut, save_mock_state},
		model::{Backend, MockProposalState, MockProposalStatus},
		preflight::preflight,
		wallet::{ensure_wallet_session, request_wallet_tx_approval},
	},
	output::{kv, line, steps},
	CrrpResult,
};

pub(crate) async fn run_propose(
	args: ProposeArgs,
	eth_rpc_url_override: Option<&str>,
) -> CrrpResult {
	let ctx = preflight(&args.common, eth_rpc_url_override).await?;
	let proposal = prepare_proposal(&ctx.repo_root, args.commit.as_deref())?;

	line("Preparing proposal...");
	kv("Repository", ctx.repo_root.display());
	kv("Repo ID", format!("{:#x}", ctx.repo_id));
	kv("Registry", ctx.registry);
	kv("Selected commit", &proposal.commit);
	match proposal.base_commit.as_deref() {
		Some(base_commit) => kv("Bundle base commit", base_commit),
		None => kv("Bundle base commit", "<root commit>"),
	}
	steps(
		"Next steps:",
		&[
			"Create Git bundle artifact for selected commit",
			"Submit bundle to Bulletin abstraction and obtain mock CID",
			"Reuse or establish wallet session",
			"Record proposal submission in selected backend",
		],
	);

	if args.dry_run {
		line("Dry-run enabled: no bundle/upload/signature/transaction executed.");
		return Ok(());
	}

	if ctx.backend == Backend::Mock {
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
		line("Mock backend: proposal submitted successfully.");
		kv("Proposal ID", proposal_id);
		kv("Bundle path", bundle_submission.bundle_path.display());
		kv("Mock CID", bundle_submission.cid);
		kv("Wallet session", wallet_session.session_id);
		return Ok(());
	}

	let signer_input = args.common.bulletin_signer.as_deref().ok_or(
		"Missing --bulletin-signer for non-mock propose. Provide a dev account, mnemonic phrase, or 0x secret seed for Bulletin upload.",
	)?;
	let signer = resolve_substrate_signer(signer_input)?;
	ensure_bulletin_upload_capability(&ctx.substrate_rpc_ws).await?;
	let bundle_submission = create_mock_bundle_submission(&ctx.repo_root, &proposal)?;
	let bundle_bytes = fs::read(&bundle_submission.bundle_path).map_err(|error| {
		format!(
			"Failed to read proposal bundle at {}: {error}",
			bundle_submission.bundle_path.display()
		)
	})?;
	let approval = request_wallet_tx_approval(
		&ctx,
		"Bulletin upload signature request",
		&format!(
			"repo_id={:#x}, commit={}, bundle_bytes={}",
			ctx.repo_id,
			proposal.commit,
			bundle_bytes.len()
		),
	)
	.await?;
	let extrinsic_hash = upload_to_bulletin(&bundle_bytes, &ctx.substrate_rpc_ws, &signer).await?;

	line("Bulletin upload submitted.");
	kv("Bulletin RPC", ctx.substrate_rpc_ws);
	kv("Bulletin extrinsic hash", extrinsic_hash);
	kv("Local bundle path", bundle_submission.bundle_path.display());
	kv("Local CID placeholder", bundle_submission.cid);
	if let Some(session_id) = approval.session_id.as_deref() {
		kv("Wallet session", session_id);
	}
	kv("Wallet approval id", approval.approval_id);
	kv("Wallet approval timestamp", approval.approved_at_unix_secs);
	kv("Wallet payload digest", approval.payload_digest_hex);
	kv("Wallet signature", approval.signature_hex);
	kv("Wallet signature origin", approval.signature_origin.as_str());
	kv("Wallet signature receipt", approval.receipt_path.display());
	line("Note: extrinsic signing still uses --bulletin-signer; pwallet signing submission is pending.");
	line("Contract submission step still pending: use proposal commit + Bulletin-backed artifact reference.");

	Ok(())
}
