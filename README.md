# Aperio

Aperio is a censorship-resistant repository platform. Git keeps code and history off-chain, Bulletin stores Git bundle artifacts, and the smart contract records canonical repository decisions: HEAD, proposals, reviews, merges, and releases.

Live deployment: https://aperio.dot.li/

The core flow is:

```text
Git -> Bundle -> Upload -> CID -> Contract -> HEAD
```

## Path Picked

- Backend: Solidity smart contracts on Polkadot Hub through `pallet-revive`.
- Frontend: static React web app deployed through Bulletin Chain and DotNS.
- Supporting tool: Node.js CLI for the same repository workflow from a terminal.

## Project Scope

- `web/` - React frontend for repository discovery, proposals, maintainer actions, rewards, and wallet configuration.
- `contracts/` - Solidity contracts for the Aperio repository registry and incentives treasury, compiled for EVM and PVM.
- `cli/aperio/` - Node.js CLI for creating repositories, proposing bundles, reviewing, merging, and downloading canonical bundles.
- `scripts/` - Deployment helpers for contracts and the frontend.
- `.github/workflows/` - CI for web/contracts plus DotNS frontend deployment.

The old Polkadot runtime/pallet template has been removed. Aperio targets existing Polkadot Hub / Asset Hub infrastructure through `pallet-revive`, the Ethereum RPC endpoint, PAPI descriptors, and the Bulletin chain.

## Quick Start

Prerequisites:

- Node.js 22
- npm 10+
- git

Install and build the frontend:

```bash
cd web
npm install
npm run build
```

Run the web app locally:

```bash
cd web
npm run dev:paseo
# Then open this link: https://dot.li/localhost:5173
```

Install the CLI:

```bash
cd cli/aperio
npm install
npm link
aperio --help
```

Run contract tests:

```bash
cd contracts/evm
npm install
npm test

cd ../pvm
npm install
npm test
```

## Deployment

Deploy contracts to Polkadot Testnet (Paseo):

```bash
# For Paseo you can set Alice private key: 
# 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133
cd contracts/evm && npx hardhat vars set PRIVATE_KEY
cd ../pvm && npx hardhat vars set PRIVATE_KEY
cd ../..
./scripts/deploy-paseo.sh
```

Deploy the frontend via the GitHub Actions workflow `Deploy Frontend to DotNS`.
The current presentation deployment is:

```text
https://aperio.dot.li/
```

## Documentation

- [docs/PROJECT.md](docs/PROJECT.md) - Aperio architecture and protocol model.
- [docs/CLI.md](docs/CLI.md) - Current CLI behavior and target developer experience.
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) - Contract and frontend deployment notes.
- [contracts/README.md](contracts/README.md) - Contract project commands.
- [cli/aperio/README.md](cli/aperio/README.md) - CLI command reference.
- [web/README.md](web/README.md) - Frontend development notes.
- [scripts/README.md](scripts/README.md) - Deployment helper scripts.

## Current MVP Status

- Contracts implement repository creation, proposals, reviews, maintainer merges, canonical HEAD tracking, releases, roles, and pull-based rewards.
- The CLI implements create, propose, review, merge, role management, download, and info commands.
- The frontend supports repository discovery and operation flows, and reads canonical history.
- Release creation exists in the contract layer; a dedicated CLI/web release command is not part of the current MVP surface yet.

## What Works

- A maintainer can create a repository with an initial Git bundle CID and canonical HEAD commit.
- Contributors can submit proposal bundles.
- Reviewers can approve or reject proposals after inspecting bundle contents off-chain.
- Maintainers can merge approved proposals by recording the final commit and CID.
- The frontend reads repositories, proposals, canonical history, releases, roles, treasury balances, and reward leaderboards from the registry.
- The CLI can create, propose, review, merge, manage roles, inspect repo state, and download the canonical bundle.
- Contracts are tested for create, propose, review, merge, release, role, and reward behavior on both EVM and PVM projects.

## What Does Not Work Yet

- CLI/web release creation is not exposed yet, although the registry contract supports releases.
- The target `git push polkadot main` flow is not implemented; users still run explicit Aperio commands.
- The CLI does not yet generate bundles automatically for every command. It accepts a `.bundle` path and can infer the commit from a local Git repo.
- The CLI/web currently trust the user-supplied commit and bundle to match. Reviewers are expected to verify bundle contents off-chain before approving.
- Production wallet signing for the CLI is not complete. The CLI uses an environment SURI or plaintext testnet import helper.

## Known Limitations

- Git branch support is intentionally limited to `main`.
- Large repositories are limited by the practical Bulletin transaction storage size used by this MVP.
- Bulletin uploads currently rely on a pre-authorized testnet signer path.
- Demo role separation is relaxed so one test account can exercise the full flow. A production deployment should prevent contributors from reviewing their own proposals and decide whether maintainers can review.
- The maintainer is a single address in the MVP. The intended production direction is DAO or governance-controlled maintainership.
- Statement Store is not used yet. It would fit proposal discussion, review notes, and maintainer/contributor coordination.
- A proposal spam deposit, reviewer slashing, richer release downloads, and organization ownership are future work.

## Verification

These checks passed locally during the final documentation pass:

```bash
cd web && npm run build && npm run lint && npm run fmt:check
cd contracts/evm && npm test && npm run fmt:check
cd contracts/pvm && npm test && npm run fmt:check
cd docs/slides && npm run export
cd cli/aperio && npm start -- --help
```

`npm run build` for the web app currently emits a Vite chunk-size warning for the main bundle. The build succeeds; code-splitting PAPI/metadata-heavy chunks is a polish task.

## Invariants

- Git builds state off-chain.
- Bulletin stores bundle bytes addressed by CID.
- The contract selects canonical truth.
- Releases point only to accepted commits.
- Production signing should be wallet or environment-driven. The current CLI includes a plaintext SURI import helper for testnet/dev accounts only.
