import { Link, useParams } from "react-router-dom";
import { keccak256 } from "viem";
import { useSubstrateSession } from "../features/auth/useSubstrateSession";
import { useWalletSession } from "../features/auth/useWalletSession";
import { useRepoOverview } from "../features/repo/useRepoOverview";
import { buildBundleUrl, formatGitCommitHash } from "../lib/aperio";

export default function RepoTreeRoute() {
	const { organization, repository } = useParams();
	const { account } = useWalletSession();
	const { browserAccounts, selectedBrowserAccountIndex } = useSubstrateSession();
	const substrateAccount = browserAccounts[selectedBrowserAccountIndex] ?? null;
	const substrateH160 = substrateAccount
		? (`0x${keccak256(substrateAccount.polkadotSigner.publicKey).slice(-40)}` as `0x${string}`)
		: null;
	const effectiveAccount = substrateH160 ?? account;
	const {
		repo,
		loading: repoLoading,
		error: repoError,
	} = useRepoOverview(organization, repository, effectiveAccount);

	if (repoLoading) {
		return <div className="card animate-pulse h-40" />;
	}

	if (repoError || !repo) {
		return (
			<div className="card">
				<h1 className="section-title">Tree Unavailable</h1>
				<p className="mt-3 text-sm text-accent-red">
					{repoError || "Repository not found"}
				</p>
			</div>
		);
	}

	const repoLink = `/repo/${encodeURIComponent(repo.organization)}/${encodeURIComponent(repo.repository)}`;
	const bundleUrl = buildBundleUrl(repo.latestCid);

	return (
		<div className="space-y-4">
			<section className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
				<div>
					<h1 className="page-title">Repository Tree</h1>
					<p className="mt-1 break-all text-text-secondary">
						{repo.organization}/{repo.repository}
					</p>
					<p className="mt-1 font-mono text-xs text-text-tertiary">
						HEAD {formatGitCommitHash(repo.latestCommitHash)}
					</p>
				</div>
				<Link to={repoLink} className="btn-secondary">
					Back To Overview
				</Link>
			</section>

			<div className="card space-y-4">
				<div>
					<h2 className="section-title">Bundle Browser Unavailable</h2>
					<p className="mt-2 text-sm text-text-secondary">
						The in-browser repository tree is temporarily disabled in this build. You
						can still inspect the canonical commit and download the canonical bundle
						directly.
					</p>
				</div>

				<div className="grid gap-3 md:grid-cols-2">
					<ValuePanel
						label="Canonical Commit"
						value={formatGitCommitHash(repo.latestCommitHash)}
						mono
					/>
					<ValuePanel
						label="Canonical CID"
						value={repo.latestCid || "Not recorded"}
						mono
					/>
				</div>

				{bundleUrl ? (
					<div className="space-y-3">
						<a
							href={bundleUrl}
							target="_blank"
							rel="noreferrer"
							className="btn-primary inline-flex w-fit"
						>
							Download Canonical Bundle
						</a>
						<div className="rounded-lg border border-white/[0.06] bg-white/[0.03] p-3">
							<div className="panel-label">Resolved Bundle URL</div>
							<div className="mt-2 break-all font-mono text-xs text-text-secondary">
								{bundleUrl}
							</div>
						</div>
					</div>
				) : (
					<div className="rounded-lg border border-white/[0.06] bg-white/[0.03] p-3 text-sm text-text-secondary">
						No canonical CID has been recorded yet, so there is no bundle to inspect.
					</div>
				)}
			</div>
		</div>
	);
}

function ValuePanel({
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
			<div className="panel-label">{label}</div>
			<div className={`mt-2 break-all text-sm text-text-primary ${mono ? "font-mono" : ""}`}>
				{value}
			</div>
		</div>
	);
}
