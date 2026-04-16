# CRRP CLI (Current Usage)

This folder contains the Rust CLI package `stack-cli`, which builds the `crrp` binary.

The current command surface is CRRP-focused:

- `config` (`init`, `show`)
- `propose`
- `fetch`
- `review`
- `merge`
- `release`
- `status`
- `repo`
- `proposals`
- `chain` (`info`, `blocks`, `statement-submit`, `statement-dump`)

Important: CRRP commands are still marked as `skeleton` in output. The workflow and hooks are wired, but full transaction execution is still being implemented.

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
  - can also write `.crrp/repo-id`
- `config show`
  - prints current repo config and repo id

Examples:

```bash
# Interactive setup
cargo run -p stack-cli -- config init --interactive

# Non-interactive setup
cargo run -p stack-cli -- config init \
  --repo /path/to/repo \
  --repo-id 0x1111111111111111111111111111111111111111111111111111111111111111 \
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

- Repo ID: `--repo-id` -> `.crrp/repo-id`
- Registry: `--registry` -> `.crrp/config.json` -> `CRRP_REGISTRY_ADDRESS` -> `deployments.json`
- Wallet backend: `--wallet-backend` -> `.crrp/config.json` -> default `papp`
- papp-term metadata: `--papp-term-metadata` -> `.crrp/config.json`
- papp-term endpoint: `--papp-term-endpoint` -> `.crrp/config.json`
- ETH RPC URL: `--eth-rpc-url` -> `.crrp/config.json` (`ethRpcHttp`) -> `http://127.0.0.1:8545`

## Shared CRRP Flags

All CRRP commands (`propose`, `fetch`, `review`, `merge`, `release`, `status`, `repo`, `proposals`) share:

- `--repo <REPO>`
  - Local Git repo path (see section above).
- `--repo-id <REPO_ID>`
  - `0x` bytes32 override for repo id.
  - If omitted, CLI reads `.crrp/repo-id` at repo root.
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
- `--allow-non-main`
  - Allow CRRP execution outside `main` branch for testing.
  - Env: `CRRP_ALLOW_NON_MAIN`

Extra per-command flags:

- `propose --dry-run`
- `merge --dry-run`
- `release --dry-run`
- `fetch --into <DIR>`
- `review --decision <approve|reject>`
- `proposals --state <open|rejected|merged> --limit <N>`

## Command Reference

### `propose`

Prepare/submit a proposal flow.

```bash
cargo run -p stack-cli -- propose --repo /path/to/repo
```

### `fetch <proposal_id>`

Fetch proposal artifact flow.

```bash
cargo run -p stack-cli -- fetch 7 --repo /path/to/repo --into /tmp/proposal-7
```

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
- repo id available via `.crrp/repo-id` or `--repo-id`

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

CRRP links to `papp-terminal` as a Rust library and opens the TUI in-process for wallet sign-in.

Run command:

```bash
cargo run -p stack-cli -- propose --repo /path/to/repo --wallet-backend papp
```

CRRP launches `papp-term tui` for sign-in when a signature-requiring command runs.
