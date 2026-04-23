# Scripts

Only deployment helpers remain in this directory. The old local runtime, Zombienet, and Polkadot SDK downloader scripts were removed with the unused template chain.

## Contract Deployment

```bash
./scripts/deploy-paseo.sh
```

Deploys the EVM and PVM Aperio contracts to Polkadot TestNet. Set the deployer key first:

```bash
cd contracts/evm && npx hardhat vars set PRIVATE_KEY
cd ../pvm && npx hardhat vars set PRIVATE_KEY
```

## Frontend Deployment

```bash
./scripts/deploy-frontend.sh --domain aperio00.dot
```

Builds `web/` and deploys the static output through `bulletin-deploy`.

Requirements:

- Node.js 22
- `bulletin-deploy`
- IPFS Kubo
