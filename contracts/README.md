# Contracts

This directory contains the Aperio Solidity contracts for two execution targets:

| Project | Path | Toolchain | Target |
| --- | --- | --- | --- |
| EVM | `evm/` | Hardhat + solc + viem | Ethereum-compatible bytecode |
| PVM | `pvm/` | Hardhat + `@parity/resolc` + viem | PolkaVM bytecode |

Both projects deploy the same Aperio contract set:

- `AperioRepositoryRegistry.sol` - repositories, proposals, reviews, merges, HEAD, releases, and roles.
- `AperioIncentivesTreasury.sol` - pull-based contribution and review rewards.

## Test

```bash
cd contracts/evm
npm install
npm run compile
npm test

cd ../pvm
npm install
npm run compile
npm test
```

## Deploy To Polkadot TestNet

```bash
cd contracts/evm && npx hardhat vars set PRIVATE_KEY
cd ../pvm && npx hardhat vars set PRIVATE_KEY
cd ../..
./scripts/deploy-paseo.sh
```

Deploy scripts update:

- `deployments.json`
- `web/src/config/deployments.ts`
- `cli/aperio/deployments.json`

## CLI Flow

After deployment, use the CLI to create and operate repositories:

```bash
cd cli/aperio
npm install
node ./bin/aperio.mjs import "//Alice"
node ./bin/aperio.mjs map
node ./bin/aperio.mjs create-repo acme my-repo --bundle /tmp/repo.bundle --repo . --permissionless
```
