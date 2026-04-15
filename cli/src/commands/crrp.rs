use clap::{Args, ValueEnum};
use std::{
	path::{Path, PathBuf},
	process::Command,
};

#[derive(clap::Subcommand)]
pub enum CrrpAction {
	/// Prepare and submit a repository proposal (skeleton).
	Propose(ProposeArgs),
	/// Fetch a proposal bundle (skeleton).
	Fetch(FetchArgs),
	/// Submit an approve/reject review for a proposal (skeleton).
	Review(ReviewArgs),
	/// Merge a proposal and submit canonical HEAD (skeleton).
	Merge(MergeArgs),
	/// Create a release from canonical HEAD (skeleton).
	Release(ReleaseArgs),
	/// Show local repo status relevant to CRRP (skeleton).
	Status(StatusArgs),
	/// Show repo-level CRRP metadata (skeleton).
	Repo(RepoArgs),
	/// List proposals for the current repository (skeleton).
	Proposals(ProposalsArgs),
}

#[derive(Args)]
pub struct ProposeArgs {
	/// Optional repository path (defaults to current directory).
	#[arg(long)]
	pub repo: Option<PathBuf>,
	/// Dry-run: print planned steps without network submission.
	#[arg(long)]
	pub dry_run: bool,
}

#[derive(Args)]
pub struct FetchArgs {
	/// Proposal ID to fetch.
	pub proposal_id: u64,
	/// Optional destination directory for fetched bundle.
	#[arg(long)]
	pub into: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ReviewDecision {
	Approve,
	Reject,
}

#[derive(Args)]
pub struct ReviewArgs {
	/// Proposal ID to review.
	pub proposal_id: u64,
	/// Review decision.
	#[arg(long, value_enum)]
	pub decision: ReviewDecision,
}

#[derive(Args)]
pub struct MergeArgs {
	/// Proposal ID to merge.
	pub proposal_id: u64,
	/// Optional repository path (defaults to current directory).
	#[arg(long)]
	pub repo: Option<PathBuf>,
	/// Dry-run: print planned steps without network submission.
	#[arg(long)]
	pub dry_run: bool,
}

#[derive(Args)]
pub struct ReleaseArgs {
	/// Version string (e.g. v1.2.0).
	pub version: String,
	/// Dry-run: print planned steps without network submission.
	#[arg(long)]
	pub dry_run: bool,
}

#[derive(Args)]
pub struct StatusArgs {
	/// Optional repository path (defaults to current directory).
	#[arg(long)]
	pub repo: Option<PathBuf>,
}

#[derive(Args)]
pub struct RepoArgs {
	/// Optional repository path (defaults to current directory).
	#[arg(long)]
	pub repo: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProposalStateFilter {
	Open,
	Rejected,
	Merged,
}

#[derive(Args)]
pub struct ProposalsArgs {
	/// Optional state filter.
	#[arg(long, value_enum)]
	pub state: Option<ProposalStateFilter>,
	/// Max number of proposals to list.
	#[arg(long, default_value_t = 20)]
	pub limit: u16,
}

pub async fn run(
	action: CrrpAction,
	_ws_url: &str,
	_eth_rpc_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
	match action {
		CrrpAction::Propose(args) => run_propose(args)?,
		CrrpAction::Fetch(args) => run_fetch(args)?,
		CrrpAction::Review(args) => run_review(args)?,
		CrrpAction::Merge(args) => run_merge(args)?,
		CrrpAction::Release(args) => run_release(args)?,
		CrrpAction::Status(args) => run_status(args)?,
		CrrpAction::Repo(args) => run_repo(args)?,
		CrrpAction::Proposals(args) => run_proposals(args)?,
	}

	Ok(())
}

fn run_propose(args: ProposeArgs) -> Result<(), Box<dyn std::error::Error>> {
	let repo_root = detect_repo_root(args.repo.as_deref())?;
	let branch = git_output(&repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
	if branch != "main" {
		return Err(format!("CRRP only supports main branch. Current branch: {branch}").into());
	}

	let head = git_output(&repo_root, &["rev-parse", "HEAD"])?;
	let base = git_output(&repo_root, &["rev-parse", "HEAD~1"]).unwrap_or_else(|_| head.clone());

	println!("Preparing proposal...");
	println!("Repository: {}", repo_root.display());
	println!("Base commit: {base}");
	println!("HEAD commit: {head}");
	println!("Next steps (skeleton):");
	println!("1. Create Git bundle artifact");
	println!("2. Upload bundle and obtain CID");
	println!("3. Request wallet signature");
	println!("4. Submit proposal transaction");
	if args.dry_run {
		println!("Dry-run enabled: no upload/signature/transaction executed.");
	}

	Ok(())
}

fn run_fetch(args: FetchArgs) -> Result<(), Box<dyn std::error::Error>> {
	let into = args
		.into
		.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
	println!("Fetching proposal {}...", args.proposal_id);
	println!("Target directory: {}", into.display());
	println!("Skeleton: resolve proposal CID -> download bundle -> import into local Git.");
	Ok(())
}

fn run_review(args: ReviewArgs) -> Result<(), Box<dyn std::error::Error>> {
	println!("Reviewing proposal {}...", args.proposal_id);
	println!("Decision: {:?}", args.decision);
	println!("Skeleton: request wallet signature -> submit on-chain review.");
	Ok(())
}

fn run_merge(args: MergeArgs) -> Result<(), Box<dyn std::error::Error>> {
	let repo_root = detect_repo_root(args.repo.as_deref())?;
	let head = git_output(&repo_root, &["rev-parse", "HEAD"])?;

	println!("Merging proposal {}...", args.proposal_id);
	println!("Repository: {}", repo_root.display());
	println!("Current local HEAD: {head}");
	println!("Next steps (skeleton):");
	println!("1. Fetch proposal bundle");
	println!("2. Merge locally with Git and resolve conflicts");
	println!("3. Create final bundle and upload for CID");
	println!("4. Request wallet signature");
	println!("5. Submit merge transaction (update canonical HEAD)");
	if args.dry_run {
		println!("Dry-run enabled: no upload/signature/transaction executed.");
	}

	Ok(())
}

fn run_release(args: ReleaseArgs) -> Result<(), Box<dyn std::error::Error>> {
	println!("Creating release {}...", args.version);
	println!("Skeleton: read canonical HEAD -> request wallet signature -> submit release.");
	if args.dry_run {
		println!("Dry-run enabled: no signature/transaction executed.");
	}

	Ok(())
}

fn run_status(args: StatusArgs) -> Result<(), Box<dyn std::error::Error>> {
	let repo_root = detect_repo_root(args.repo.as_deref())?;
	let branch = git_output(&repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
	let head = git_output(&repo_root, &["rev-parse", "HEAD"])?;

	println!("CRRP Status (skeleton)");
	println!("Repository: {}", repo_root.display());
	println!("Branch: {branch}");
	println!("HEAD: {head}");
	println!("Note: canonical on-chain status lookup will be added in next iteration.");

	Ok(())
}

fn run_repo(args: RepoArgs) -> Result<(), Box<dyn std::error::Error>> {
	let repo_root = detect_repo_root(args.repo.as_deref())?;
	println!("CRRP Repo (skeleton)");
	println!("Repository: {}", repo_root.display());
	println!("Note: on-chain repo metadata lookup will be added in next iteration.");

	Ok(())
}

fn run_proposals(args: ProposalsArgs) -> Result<(), Box<dyn std::error::Error>> {
	println!("CRRP Proposals (skeleton)");
	println!("State filter: {:?}", args.state);
	println!("Limit: {}", args.limit);
	println!("Note: on-chain proposal listing will be added in next iteration.");

	Ok(())
}

fn detect_repo_root(repo_override: Option<&Path>) -> Result<PathBuf, Box<dyn std::error::Error>> {
	let cwd = if let Some(path) = repo_override {
		path.to_path_buf()
	} else {
		std::env::current_dir()?
	};

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

fn git_output(repo_root: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
	let output = Command::new("git").args(args).current_dir(repo_root).output()?;
	if !output.status.success() {
		let stderr = String::from_utf8(output.stderr)?;
		return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()).into());
	}
	Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
