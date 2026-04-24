# Deployment

This guide covers the remaining deployable Aperio surfaces: contracts and frontend.

Current public deployment used for the presentation:

```text
https://aperio.dot.li/
```

## Contracts

Target: Paseo (`420420417`).

```bash
cd contracts/evm && npx hardhat vars set PRIVATE_KEY
cd ../pvm && npx hardhat vars set PRIVATE_KEY
cd ../..
./scripts/deploy-paseo.sh
```

Equivalent Makefile target:

```bash
make deploy-paseo
```

The deploy scripts write the resulting addresses to:

- `deployments.json`
- `web/src/config/deployments.ts`
- `cli/aperio/deployments.json`

## Frontend

The frontend is a static Vite app with hash routing and relative assets, so it can run from IPFS gateways and static hosts.

Build locally:

```bash
cd web
npm install
npm run build
```

Deploy locally through Bulletin/IPFS:

```bash
./scripts/deploy-frontend.sh --domain aperio.dot
```

Equivalent Makefile target:

```bash
make deploy-frontend DOMAIN=aperio.dot
```

Deploy through GitHub Actions (recommended path):

1. Open `Actions`.
2. Run `Deploy Frontend to DotNS`.
3. Provide the DotNS basename. For `https://aperio.dot.li/`, the basename is `aperio`.

The GitHub Action builds `web/`, exports an IPFS CAR, uploads it to Bulletin, then registers/updates the DotNS content hash.

## Required Secrets

- `PRIVATE_KEY` in Hardhat vars for contract deployment.
- `MNEMONIC` locally, or a Hardhat `MNEMONIC` var, for frontend deployment when you do not want deploy-tool defaults.
- `DOTNS_MNEMONIC` in GitHub Actions if you do not want the workflow fallback account.
- `BULLETIN_MNEMONIC` in GitHub Actions if you do not want the workflow fallback account.

## Frontend Environment

Set these when building hosted releases:

- `VITE_APERIO_REGISTRY_KIND`
- `VITE_APERIO_REGISTRY_ADDRESS`
- `VITE_WS_URL`
- `VITE_ETH_RPC_URL`

The checked-in defaults target the current test deployment.

## Pre-Submit Checks

Run these before a final deployment:

```bash
cd web && npm run build && npm run lint && npm run fmt:check
cd contracts/evm && npm test && npm run fmt:check
cd contracts/pvm && npm test && npm run fmt:check
```

After deploying, open the DotNS URL from a fresh browser profile or private window and verify that repository reads, wallet connection, and bundle download links use the intended Paseo endpoints.
