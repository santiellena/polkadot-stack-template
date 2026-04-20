import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { type Address, type Hex } from "viem";
import { getPublicClient } from "../config/evm";
import { getStoredEthRpcUrl } from "../config/network";
import { useWalletSession } from "../features/auth/useWalletSession";
import { useRepoOverview } from "../features/repo/useRepoOverview";
import {
	buildBundleUrl,
	crrpRegistryAbi,
	formatGitCommitHash,
	getRegistryAddress,
	shortenAddress,
} from "../lib/crrp";

type ProposalReadResult = readonly [Address, Hex, string, bigint, bigint, number, Hex, string];
type ReviewReadResult = readonly [boolean, boolean];

type ProposalEntry = {
	id: number;
	contributor: Address;
	proposedCommit: Hex;
	proposedCid: string;
	approvals: bigint;
	rejections: bigint;
	status: number;
	mergedCommit: Hex;
	mergedCid: string;
	reviewerVote: { exists: boolean; approved: boolean } | null;
};

const STATUS_LABEL: Record<number, string> = { 1: "Open", 2: "Rejected", 3: "Merged" };
const STATUS_CLASS: Record<number, string> = {
	1: "border-blue-500/30 bg-blue-500/10 text-blue-300",
	2: "border-red-500/30 bg-red-500/10 text-red-300",
	3: "border-emerald-500/30 bg-emerald-500/10 text-emerald-300",
};

export default function RepoProposalsRoute() {
	const { organization: rawOrg, repository: rawRepo } = useParams();
	const { account, getWalletClientForWrite } = useWalletSession();
	const { repo, loading: repoLoading, refresh: refreshRepo } = useRepoOverview(
		rawOrg,
		rawRepo,
		account,
	);

	const [proposals, setProposals] = useState<ProposalEntry[]>([]);
	const [proposalsLoading, setProposalsLoading] = useState(false);
	const [refreshKey, setRefreshKey] = useState(0);
	const [submittingId, setSubmittingId] = useState<number | null>(null);
	const [txStatus, setTxStatus] = useState<Record<number, string>>({});

	const refresh = useCallback(() => {
		setRefreshKey((k) => k + 1);
		void refreshRepo();
	}, [refreshRepo]);

	useEffect(() => {
		if (!repo) {
			setProposals([]);
			return;
		}
		if (repo.proposalCount === 0n) {
			setProposals([]);
			setProposalsLoading(false);
			return;
		}

		let cancelled = false;
		setProposalsLoading(true);

		const fetchProposals = async () => {
			const client = getPublicClient(getStoredEthRpcUrl());
			const registryAddress = getRegistryAddress();
			const count = Number(repo.proposalCount);
			const ids = Array.from({ length: count }, (_, i) => i);

			const [proposalResults, reviewResults] = await Promise.all([
				Promise.all(
					ids.map((i) =>
						client.readContract({
							address: registryAddress,
							abi: crrpRegistryAbi,
							functionName: "getProposal",
							args: [repo.repoId, BigInt(i)],
						}) as Promise<ProposalReadResult>,
					),
				),
				account && repo.roles.isReviewer
					? Promise.all(
							ids.map((i) =>
								client.readContract({
									address: registryAddress,
									abi: crrpRegistryAbi,
									functionName: "getReview",
									args: [repo.repoId, BigInt(i), account],
								}) as Promise<ReviewReadResult>,
							),
						)
					: Promise.resolve(null),
			]);

			if (cancelled) return;

			const entries: ProposalEntry[] = proposalResults.map((data, i) => ({
				id: i,
				contributor: data[0],
				proposedCommit: data[1],
				proposedCid: data[2],
				approvals: data[3],
				rejections: data[4],
				status: data[5],
				mergedCommit: data[6],
				mergedCid: data[7],
				reviewerVote: reviewResults
					? { exists: reviewResults[i][0], approved: reviewResults[i][1] }
					: null,
			}));

			setProposals(entries.reverse());
			setProposalsLoading(false);
		};

		void fetchProposals().catch(() => setProposalsLoading(false));
		return () => {
			cancelled = true;
		};
	}, [repo, account, refreshKey]);

	const submitReview = async (proposalId: number, approved: boolean) => {
		if (!repo) return;

		setSubmittingId(proposalId);
		setTxStatus((prev) => ({
			...prev,
			[proposalId]: approved ? "Submitting approval..." : "Submitting rejection...",
		}));

		try {
			const walletClient = await getWalletClientForWrite();
			if (!walletClient.account) throw new Error("No EVM signer available");

			const publicClient = getPublicClient(getStoredEthRpcUrl());
			const hash = await walletClient.writeContract({
				address: getRegistryAddress(),
				abi: crrpRegistryAbi,
				functionName: "reviewProposal",
				args: [repo.repoId, BigInt(proposalId), approved],
				account: walletClient.account,
				chain: walletClient.chain,
			});
			await publicClient.waitForTransactionReceipt({ hash });

			setTxStatus((prev) => ({
				...prev,
				[proposalId]: approved ? "Approved." : "Rejected.",
			}));

			setProposals((prev) =>
				prev.map((p) =>
					p.id === proposalId
						? {
								...p,
								approvals: approved ? p.approvals + 1n : p.approvals,
								rejections: approved ? p.rejections : p.rejections + 1n,
								status: approved ? p.status : 2,
								reviewerVote: { exists: true, approved },
							}
						: p,
				),
			);
		} catch (cause) {
			setTxStatus((prev) => ({
				...prev,
				[proposalId]: cause instanceof Error ? cause.message : "Review failed",
			}));
		} finally {
			setSubmittingId(null);
		}
	};

	if (repoLoading) {
		return <div className="card animate-pulse h-40" />;
	}

	if (!repo) {
		return (
			<div className="card">
				<h1 className="section-title">Repository Unavailable</h1>
				<p className="mt-3 text-sm text-accent-red">Repository not found</p>
			</div>
		);
	}

	const repoLink = `/repo/${encodeURIComponent(repo.organization)}/${encodeURIComponent(repo.repository)}`;
	const canPropose = repo.permissionlessContributions || repo.roles.isContributor;

	return (
		<div className="space-y-6">
			<section className="space-y-3">
				<div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
					<div>
						<h1 className="page-title">Proposals</h1>
						<p className="mt-2 text-text-secondary break-all">
							{repo.organization}/{repo.repository}
						</p>
						<p className="mt-1 text-text-tertiary font-mono text-sm">
							{proposals.length} of {repo.proposalCount.toString()} proposals
						</p>
					</div>
					<div className="flex flex-wrap gap-2">
						<button onClick={refresh} className="btn-secondary">
							Refresh
						</button>
						{canPropose ? (
							<Link to={`${repoLink}/propose`} className="btn-primary">
								Submit Proposal
							</Link>
						) : null}
						<Link to={repoLink} className="btn-secondary">
							Back To Repository
						</Link>
					</div>
				</div>
				{repo.roles.isReviewer && !repo.roles.isMaintainer ? (
					<div className="rounded-lg border border-blue-500/20 bg-blue-500/10 p-3 text-sm text-text-secondary">
						You have the reviewer role. Download a proposal bundle, inspect the changes,
						then submit your approval or rejection.
					</div>
				) : null}
			</section>

			{proposalsLoading ? (
				<div className="space-y-3">
					{[0, 1, 2].map((n) => (
						<div key={n} className="card animate-pulse h-32" />
					))}
				</div>
			) : proposals.length === 0 ? (
				<div className="card py-10 text-center text-sm text-text-secondary">
					No proposals have been submitted yet.
				</div>
			) : (
				<div className="space-y-3">
					{proposals.map((proposal) => {
						const isOwnProposal =
							account?.toLowerCase() === proposal.contributor.toLowerCase();
						const canReview =
							repo.roles.isReviewer &&
							!repo.roles.isMaintainer &&
							!isOwnProposal &&
							proposal.status === 1 &&
							!proposal.reviewerVote?.exists;
						const bundleUrl = buildBundleUrl(proposal.proposedCid);
						const mergedBundleUrl =
							proposal.mergedCid ? buildBundleUrl(proposal.mergedCid) : null;
						const statusClass =
							STATUS_CLASS[proposal.status] ??
							"border-white/[0.06] bg-white/[0.03] text-text-secondary";
						const statusLabel = STATUS_LABEL[proposal.status] ?? "Unknown";
						const proposalTxStatus = txStatus[proposal.id];
						const isSubmitting = submittingId === proposal.id;

						return (
							<div key={proposal.id} className="card space-y-4">
								<div className="flex flex-wrap items-start justify-between gap-3">
									<div className="flex flex-wrap items-center gap-2">
										<span className="rounded-md border border-white/[0.08] bg-white/[0.04] px-2 py-0.5 font-mono text-xs text-text-muted">
											#{proposal.id}
										</span>
										<span
											className={`rounded-md border px-2 py-0.5 text-xs font-medium ${statusClass}`}
										>
											{statusLabel}
										</span>
										{isOwnProposal ? (
											<span className="rounded-md border border-amber-500/30 bg-amber-500/10 px-2 py-0.5 text-xs text-amber-300">
												Your submission
											</span>
										) : null}
									</div>
									<div className="flex items-center gap-2 text-xs text-text-tertiary">
										<span className="text-emerald-400">
											↑ {proposal.approvals.toString()}
										</span>
										<span>·</span>
										<span className="text-red-400">
											↓ {proposal.rejections.toString()}
										</span>
									</div>
								</div>

								<div className="grid gap-3 md:grid-cols-2">
									<InfoRow
										label="Contributor"
										value={shortenAddress(proposal.contributor)}
										mono
									/>
									<InfoRow
										label="Proposed Commit"
										value={formatGitCommitHash(proposal.proposedCommit)}
										mono
									/>
									<InfoRow
										label="Proposed CID"
										value={proposal.proposedCid}
										mono
									/>
									{proposal.status === 3 ? (
										<>
											<InfoRow
												label="Merged Commit"
												value={formatGitCommitHash(proposal.mergedCommit)}
												mono
											/>
											<InfoRow
												label="Merged CID"
												value={proposal.mergedCid}
												mono
											/>
										</>
									) : null}
								</div>

								<div className="flex flex-wrap gap-2">
									{bundleUrl ? (
										<a
											href={bundleUrl}
											target="_blank"
											rel="noreferrer"
											className="btn-secondary text-sm"
										>
											Download Proposed Bundle
										</a>
									) : null}
									{mergedBundleUrl && proposal.status === 3 ? (
										<a
											href={mergedBundleUrl}
											target="_blank"
											rel="noreferrer"
											className="btn-secondary text-sm"
										>
											Download Merged Bundle
										</a>
									) : null}
								</div>

								{repo.roles.isReviewer && !repo.roles.isMaintainer ? (
									<div className="border-t border-white/[0.06] pt-3">
										{proposal.reviewerVote?.exists ? (
											<span
												className={`text-sm ${proposal.reviewerVote.approved ? "text-emerald-400" : "text-red-400"}`}
											>
												You{" "}
												{proposal.reviewerVote.approved
													? "approved"
													: "rejected"}{" "}
												this proposal.
												{proposalTxStatus ? ` ${proposalTxStatus}` : ""}
											</span>
										) : isOwnProposal ? (
											<span className="text-sm text-text-tertiary">
												You cannot review your own proposal.
											</span>
										) : proposal.status !== 1 ? (
											<span className="text-sm text-text-tertiary">
												This proposal is no longer open for review.
											</span>
										) : canReview ? (
											<div className="flex flex-wrap items-center gap-3">
												<span className="text-sm text-text-secondary">
													Submit your review:
												</span>
												<div className="flex gap-2">
													<button
														onClick={() => void submitReview(proposal.id, true)}
														disabled={isSubmitting}
														className="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-1.5 text-sm text-emerald-300 transition-colors hover:bg-emerald-500/20 disabled:opacity-50"
													>
														{isSubmitting ? "..." : "Approve"}
													</button>
													<button
														onClick={() =>
															void submitReview(proposal.id, false)
														}
														disabled={isSubmitting}
														className="rounded-md border border-red-500/30 bg-red-500/10 px-3 py-1.5 text-sm text-red-300 transition-colors hover:bg-red-500/20 disabled:opacity-50"
													>
														{isSubmitting ? "..." : "Reject"}
													</button>
												</div>
												{proposalTxStatus ? (
													<span className="break-all text-sm text-text-secondary">
														{proposalTxStatus}
													</span>
												) : null}
											</div>
										) : null}
									</div>
								) : null}
							</div>
						);
					})}
				</div>
			)}
		</div>
	);
}

function InfoRow({
	label,
	value,
	mono = false,
}: {
	label: string;
	value: string;
	mono?: boolean;
}) {
	return (
		<div className="rounded-lg border border-white/[0.06] bg-white/[0.03] p-3">
			<div className="text-[11px] uppercase tracking-[0.18em] text-text-muted">{label}</div>
			<div
				className={`mt-1 break-all text-sm text-text-primary ${mono ? "font-mono" : ""}`}
			>
				{value || "—"}
			</div>
		</div>
	);
}
