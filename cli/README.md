# CRRP CLI (Current Usage)

This folder contains the Rust CLI package `stack-cli`, which builds the `crrp` binary.

The current command surface is CRRP-focused:

- `config` (`init`, `show`)
- `create-repo`
- `propose`
- `fetch`
- `review`
- `merge`
- `release`
- `status`
- `repo`
- `proposals`
- `chain` (`info`, `blocks`, `statement-submit`, `statement-dump`)

Important: most CRRP commands are still `skeleton` flows. `create-repo` is implemented for both RPC and mock backends, and `propose` in `--mock` mode creates a real git bundle, derives a mock CID, and records a local proposal submission.

## Run

From the repository root:

```bash
cargo run -p stack-cli -- --help
```

## What Is `--repo /path/to/repo`?

`/path/to/repo` means the local Git repository you want CRRP to operate on.

- It can be the repo root, or any subdirectory inside that repo.
- The CLI runs `git rev-parse --show-toplevel` and resolves the real repo root.
- If omitted, CLI uses your current directory.

Examples:

```bash
# Use current directory
cargo run -p stack-cli -- status

# Explicit repo root
cargo run -p stack-cli -- status --repo /home/user/my-project

# Explicit subfolder inside a repo (also valid)
cargo run -p stack-cli -- status --repo /home/user/my-project/src
```

## Global Flags

These are top-level flags for all commands:

- `--url <URL>`
  - Substrate WS endpoint.
  - Env: `SUBSTRATE_RPC_WS`
  - Default: `ws://127.0.0.1:9944`
  - Used by `chain` commands.
- `--eth-rpc-url <URL>`
  - Ethereum JSON-RPC endpoint used for contract reads/writes.
  - Env: `ETH_RPC_HTTP`
  - Used by CRRP commands when provided.

## `config` Command

Initialize/show repository-local CRRP configuration in `.crrp/`.

- `config init`
  - writes `.crrp/config.json`
  - can also write `.crrp/repo-slug.json`
- `config show`
  - prints current repo config, repository slug, and derived repo id

Examples:

```bash
# Interactive setup
cargo run -p stack-cli -- config init --interactive

# Non-interactive setup
cargo run -p stack-cli -- config init \
  --repo /path/to/repo \
  --organization acme \
  --repository crrp \
  --registry 0x0000000000000000000000000000000000000001 \
  --eth-rpc-http http://127.0.0.1:8545 \
  --substrate-rpc-ws ws://127.0.0.1:9944 \
  --wallet-backend papp \
  --papp-term-metadata https://example.com/metadata.json \
  --papp-term-endpoint wss://pop3-testnet.parity-lab.parity.io/people \
  --allow-non-main true

# Inspect current config
cargo run -p stack-cli -- config show --repo /path/to/repo
```

CRRP command resolution order:

- Repository slug: `--organization` + `--repository` -> `.crrp/repo-slug.json`
- Repo ID: derived from `keccak256("organization/repository")` unless `--repo-id` override is used
- Registry: `--registry` -> `.crrp/config.json` -> `CRRP_REGISTRY_ADDRESS` -> `deployments.json`
- Wallet backend: `--wallet-backend` -> `.crrp/config.json` -> default `papp`
- papp-term metadata: `--papp-term-metadata` -> `.crrp/config.json`
- papp-term endpoint: `--papp-term-endpoint` -> `.crrp/config.json`
- ETH RPC URL: `--eth-rpc-url` -> `.crrp/config.json` (`ethRpcHttp`) -> `http://127.0.0.1:8545`

## Shared CRRP Flags

All CRRP commands (`create-repo`, `propose`, `fetch`, `review`, `merge`, `release`, `status`, `repo`, `proposals`) share:

- `--repo <REPO>`
  - Local Git repo path (see section above).
- `--organization <ORG>`
  - Repository organization override.
- `--repository <NAME>`
  - Repository name override.
- `--repo-id <REPO_ID>`
  - `0x` bytes32 override for advanced/debug flows.
  - If omitted, CLI derives the repo id from the repository slug.
- `--registry <REGISTRY>`
  - Registry contract address override.
  - If omitted in RPC mode, CLI tries env/file resolution.
- `--mock`
  - Use local mock backend instead of contract RPC reads/writes.
  - Env: `CRRP_MOCK`
- `--wallet-backend <mock|papp>`
  - Wallet sign-in backend for signature-requiring commands.
  - Default: `papp`
- `--papp-term-metadata <URL>`
  - Metadata URL passed to `papp-term tui`.
  - Env: `CRRP_PAPP_TERM_METADATA`
- `--papp-term-endpoint <WSS_URL>`
  - Statement-store endpoint passed to `papp-term tui`.
  - Env: `CRRP_PAPP_TERM_ENDPOINT`
- `--bulletin-signer <SIGNER>`
  - Substrate signer for Bulletin upload in non-mock `propose` (required there).
  - Supports dev account name (`alice`), mnemonic phrase, or `0x` 32-byte secret seed.
  - Env: `CRRP_BULLETIN_SIGNER`
- `--allow-non-main`
  - Allow CRRP execution outside `main` branch for testing.
  - Env: `CRRP_ALLOW_NON_MAIN`

Extra per-command flags:

- `create-repo --initial-commit <REV> --initial-cid <CID> --signer <SIGNER> [--contributor <ADDR>] [--reviewer <ADDR>] [--skip-role-grants]`
- `propose --commit <REV> --dry-run`
- `merge --dry-run`
- `release --dry-run`
- `fetch --into <DIR>`
- `review --decision <approve|reject>`
- `proposals --state <open|rejected|merged> --limit <N>`

## Command Reference

### `create-repo`

Create/register a repository in the on-chain CRRP registry.

```bash
cargo run -p stack-cli -- create-repo \
  --repo /path/to/repo \
  --organization acme \
  --repository crrp \
  --registry 0xYourRegistryAddress \
  --signer alice \
  --initial-cid mock://init \
  --allow-non-main
```

Behavior:

- resolves initial commit from `HEAD` (or `--initial-commit`)
- derives `repoId = keccak256("organization/repository")`
- submits `createRepo(organization, repository, initialHeadCommit, initialHeadCid)`
- by default, also grants contributor/reviewer roles:
  - contributor defaults to signer address (or `--contributor`)
  - reviewer defaults to contributor (or `--reviewer`)
- writes `.crrp/repo-slug.json` and `.crrp/repo-id`

For local testing without RPC:

```bash
cargo run -p stack-cli -- create-repo --repo /path/to/repo --mock --initial-cid mock://init
```

### `propose`

Prepare/submit a proposal flow.

```bash
cargo run -p stack-cli -- propose --repo /path/to/repo --bulletin-signer alice
```

Use a specific commit instead of `HEAD`:

```bash
cargo run -p stack-cli -- propose --repo /path/to/repo --commit HEAD~1 --bulletin-signer alice
```

In `--mock` mode this now:

- creates a git bundle for the selected commit
- stores the bundle under `.crrp/bulletins/`
- derives a mock CID
- records the proposal in `.crrp/mock-state.json`

In non-mock mode, `propose` now also:

- requires `--bulletin-signer` / `CRRP_BULLETIN_SIGNER`
- requests a pwallet SignRequest popup before broadcast (`--wallet-backend papp`)
- checks `TransactionStorage.Authorizations` first and fails early if authorization is missing/insufficient
- uploads the bundle bytes with a Bulletin extrinsic (`TransactionStorage.store`)
- prints finalized extrinsic hash
- keeps using a local CID placeholder until chain-derived CID wiring is completed

Current limitation:

- The Bulletin extrinsic is still signed/submitted by `--bulletin-signer` in the CLI runtime.
- The pwallet step now captures a real wallet signature via phone approval and stores the signature artifact locally. Bulletin submission still uses `--bulletin-signer` until direct signer wiring is completed.

Testing note:

- For local/dev demonstrations, use `--bulletin-signer alice` (or `bob`/`charlie`).
- For real user flows, pass a user-controlled mnemonic or `0x` secret seed instead.

### `fetch <proposal_id>`

Fetch proposal artifact flow.

```bash
cargo run -p stack-cli -- fetch 7 --repo /path/to/repo --into /tmp/proposal-7
```

In `--mock` mode, `fetch` copies the saved bundle from the local bulletin store into the target directory (or into the exact `.bundle` file path you pass).

### `review <proposal_id> --decision <approve|reject>`

Submit review decision flow.

```bash
cargo run -p stack-cli -- review 7 --decision approve --repo /path/to/repo
```

### `merge <proposal_id>`

Merge proposal and update canonical HEAD flow.

```bash
cargo run -p stack-cli -- merge 7 --repo /path/to/repo
```

### `release <version>`

Create release flow.

```bash
cargo run -p stack-cli -- release v1.2.0 --repo /path/to/repo
```

### `status`

Show repo + chain status summary.

```bash
cargo run -p stack-cli -- status --repo /path/to/repo
```

### `repo`

Show repository metadata from registry backend.

```bash
cargo run -p stack-cli -- repo --repo /path/to/repo
```

### `proposals`

List proposals summary.

```bash
cargo run -p stack-cli -- proposals --repo /path/to/repo --limit 50
```

### `chain`

Node/statement-store utility commands:

```bash
cargo run -p stack-cli -- chain info
cargo run -p stack-cli -- chain blocks
cargo run -p stack-cli -- chain statement-submit --file ./README.md --signer alice
cargo run -p stack-cli -- chain statement-dump
```

`chain statement-submit` flags:

- `--file <FILE>` (required)
- `--signer <SIGNER>` (default `alice`)
- `--unsigned` (test runtime rejection path)

## Required Repo Conditions for CRRP Commands

CRRP commands require:

- valid Git repository
- current branch must be `main` (unless `--allow-non-main` or `.crrp/config.json` has `allowNonMain: true`)
- repository slug available via `.crrp/repo-slug.json` or `--organization` + `--repository`

## Wallet Sign-In Behavior

Signature-requiring commands call a shared wallet-session hook:

- `propose` (unless `--dry-run`)
- `review`
- `merge` (unless `--dry-run`)
- `release` (unless `--dry-run`)

Session files:

- `.crrp/wallet-session.json` (CLI session summary)

### `mock` wallet backend

No external wallet required. CLI prints a mock QR and persists a mock session.

```bash
cargo run -p stack-cli -- propose --repo /path/to/repo --mock --wallet-backend mock
```

### `papp` backend (`papp-terminal`)

CRRP links to `papp-terminal` as a Rust library for pairing protocol compatibility and uses a QR pairing flow in CLI output.

Run command:

```bash
cargo run -p stack-cli -- propose --repo /path/to/repo --wallet-backend papp --bulletin-signer alice
```

CRRP performs QR pairing if needed, then sends a real pwallet sign request for each signature gate.

Bridge dependency setup (one-time):

```bash
cd cli/wallet-bridge
npm install
```
