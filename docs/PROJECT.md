# Aperio Project Model

Aperio is a censorship-resistant repository registry. It does not replace Git and it does not store code on-chain. Git remains the tool that creates repository state; Aperio records which state a project accepts as canonical.

```text
Git -> Bundle -> Bulletin CID -> Registry Contract -> Canonical HEAD
```

## Architecture

- Git manages source code, history, branches, and local merges off-chain.
- Git bundles are the artifact format. A bundle must reconstruct the repository state being proposed or accepted.
- Bulletin Chain stores bundle bytes and returns content-addressed storage through a CID.
- The registry contract stores repository decisions: initial HEAD, proposals, reviews, accepted merge results, releases, and role assignments.
- The frontend and CLI orchestrate the flow, but neither should decide canonical truth without a contract transaction.

## Roles

- Contributor: prepares a Git commit, creates/uploads a bundle, and submits a proposal.
- Reviewer: downloads the proposal bundle, inspects it locally, and only records approve/reject on-chain.
- Maintainer: performs the Git merge locally, resolves conflicts if needed, uploads the final bundle, and records the final canonical commit/CID.

## Proposal Flow

1. Contributor creates a commit on `main`.
2. Contributor creates a Git bundle containing that commit.
3. Bundle bytes are uploaded to Bulletin.
4. The resulting CID and proposed commit are submitted to the registry.
5. Reviewers inspect the bundle off-chain and record decisions.
6. A maintainer records the accepted final commit and CID.
7. The registry updates canonical `HEAD`.

## Merge Model

If there is no conflict, the final commit can be the proposed commit. If there is a conflict or maintainer adjustment, the maintainer resolves it locally in Git and records the new final commit and CID. The contract never performs a merge.

## Release Model

A release is `(version, commit, CID)`. Releases can only point to canonical commits that were already accepted by the registry. Creating a release does not change `HEAD`.

## Current MVP Compromises

- The CLI and frontend expose explicit commands/forms instead of a native `git push polkadot main` workflow.
- Release creation exists in the contract but not in the CLI or frontend write surface.
- The CLI and frontend do not yet prove that the provided commit exists inside the provided bundle. Review is expected to catch mismatches.
- The CLI includes plaintext SURI import for testnet/dev usage. Production signing should move to external wallet signing.
- Demo role separation is relaxed to make a one-account walkthrough possible.
