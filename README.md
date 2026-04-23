# Aperio

Aperio is a censorship-resistant repository registry. Git keeps code and history off-chain, Bulletin stores Git bundle artifacts, and the smart contract records canonical repository decisions: HEAD, proposals, reviews, merges, and releases.

## Project Scope

- `web/` - React frontend for repository discovery, proposals, maintainer actions, rewards, and wallet configuration.
- `contracts/` - Solidity contracts for the Aperio repository registry and incentives treasury, compiled for EVM and PVM.
- `cli/aperio/` - Node.js CLI for creating repositories, proposing bundles, reviewing, merging, and downloading canonical bundles.
- `scripts/` - Deployment helpers for contracts and the frontend.
- `.github/workflows/` - CI for web/contracts plus DotNS frontend deployment.

The old Polkadot runtime/pallet template has been removed. Aperio targets existing Polkadot Hub / Asset Hub infrastructure through `pallet-revive`, the Ethereum RPC endpoint, PAPI descriptors, and the Bulletin chain.

## Quick Start

Install and build the frontend:

```bash
cd web
npm install
npm run build
```

Run the web app locally:

```bash
cd web
npm run dev
```

Install the CLI:

```bash
cd cli/aperio
npm install
node ./bin/aperio.mjs --help
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

Deploy contracts to Polkadot TestNet:

```bash
cd contracts/evm && npx hardhat vars set PRIVATE_KEY
cd ../pvm && npx hardhat vars set PRIVATE_KEY
cd ../..
./scripts/deploy-paseo.sh
```

Deploy the frontend via the GitHub Actions DotNS workflow or locally with:

```bash
./scripts/deploy-frontend.sh --domain aperio00.dot
```

## Documentation

- [docs/PROJECT.md](docs/PROJECT.md) - Aperio architecture and protocol model.
- [docs/CLI.md](docs/CLI.md) - CLI developer experience specification.
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) - Contract and frontend deployment notes.
- [contracts/README.md](contracts/README.md) - Contract project commands.
- [cli/aperio/README.md](cli/aperio/README.md) - CLI command reference.

## Invariants

- Git builds state off-chain.
- Bulletin stores bundle bytes addressed by CID.
- The contract selects canonical truth.
- Releases point only to accepted commits.
- The CLI orchestrates actions and must not store private keys for production use.
