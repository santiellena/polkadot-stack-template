use clap::{Args, ValueEnum};

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

#[derive(Clone, Args)]
pub struct CrrpCommonArgs {
	/// Optional repository path (defaults to current directory).
	#[arg(long)]
	pub repo: Option<std::path::PathBuf>,
	/// Optional repo ID override (0x-prefixed bytes32). Defaults to .crrp/repo-id.
	#[arg(long)]
	pub repo_id: Option<String>,
	/// Optional registry contract address override.
	#[arg(long)]
	pub registry: Option<String>,
	/// Use local mock backend instead of eth-rpc contract reads/writes.
	#[arg(long, env = "CRRP_MOCK", default_value_t = false)]
	pub mock: bool,
	/// Allow running CRRP commands outside main branch (testing only).
	#[arg(long, env = "CRRP_ALLOW_NON_MAIN", default_value_t = false)]
	pub allow_non_main: bool,
	/// Wallet backend for signature-requiring CRRP commands.
	#[arg(long, value_enum)]
	pub wallet_backend: Option<WalletBackend>,
	/// Optional metadata URL passed to papp-term auth handshake.
	#[arg(long, env = "CRRP_PAPP_TERM_METADATA")]
	pub papp_term_metadata: Option<String>,
	/// Optional statement-store endpoint passed to papp-term.
	#[arg(long, env = "CRRP_PAPP_TERM_ENDPOINT")]
	pub papp_term_endpoint: Option<String>,
}

#[derive(Args)]
pub struct ProposeArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
	/// Commit to submit (defaults to HEAD).
	#[arg(long)]
	pub commit: Option<String>,
	/// Dry-run: print planned steps without network submission.
	#[arg(long)]
	pub dry_run: bool,
}

#[derive(Args)]
pub struct FetchArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
	/// Proposal ID to fetch.
	/// On-chain this is `uint256`; CLI currently accepts the `u64` subset.
	pub proposal_id: u64,
	/// Optional destination directory for fetched bundle.
	#[arg(long)]
	pub into: Option<std::path::PathBuf>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ReviewDecision {
	Approve,
	Reject,
}

#[derive(Args)]
pub struct ReviewArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
	/// Proposal ID to review.
	/// On-chain this is `uint256`; CLI currently accepts the `u64` subset.
	pub proposal_id: u64,
	/// Review decision.
	#[arg(long, value_enum)]
	pub decision: ReviewDecision,
}

#[derive(Args)]
pub struct MergeArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
	/// Proposal ID to merge.
	/// On-chain this is `uint256`; CLI currently accepts the `u64` subset.
	pub proposal_id: u64,
	/// Dry-run: print planned steps without network submission.
	#[arg(long)]
	pub dry_run: bool,
}

#[derive(Args)]
pub struct ReleaseArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
	/// Version string (e.g. v1.2.0).
	pub version: String,
	/// Dry-run: print planned steps without network submission.
	#[arg(long)]
	pub dry_run: bool,
}

#[derive(Args)]
pub struct StatusArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
}

#[derive(Args)]
pub struct RepoArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProposalStateFilter {
	Open,
	Rejected,
	Merged,
}

#[derive(Args)]
pub struct ProposalsArgs {
	#[command(flatten)]
	pub common: CrrpCommonArgs,
	/// Optional state filter.
	#[arg(long, value_enum)]
	pub state: Option<ProposalStateFilter>,
	/// Max number of proposals to list.
	#[arg(long, default_value_t = 20)]
	pub limit: u16,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum WalletBackend {
	Mock,
	#[value(alias = "pwallet")]
	Papp,
}
