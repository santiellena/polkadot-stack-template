# 🖥️ Aperio CLI - Developer Experience Specification

## Overview

The CLI is the **primary interface** of the Censorship-Resistant Repository Protocol (Aperio).

Its goal is to provide a **Git-native experience**, abstracting away:
- artifact creation
- storage (Bulletin Chain)
- smart contract interaction
- wallet signing

The user should feel like they are performing a normal Git workflow, with minimal additional friction.

---

## 🧠 Core UX Principle

> The blockchain is a **confirmation layer**, not the primary interaction.

The CLI must ensure:
- no unnecessary manual inputs  
- no exposure to low-level protocol details  
- a single, clear confirmation step (wallet signature)  
- full compatibility with existing Git workflows  

---

## 🎯 UX Goal

The ideal experience should feel like:

```
git commit
git push polkadot main
```

With minimal feedback:

```
Preparing proposal...
Uploading bundle...
Waiting for signature...
Proposal submitted ✔
```

---

## 🔄 High-Level Flow

### Contributor

```
git commit
git push polkadot main
```

CLI internally performs:

1. Detect repository state
2. Extract:
   - HEAD commit
   - base commit
3. Generate Git bundle (artifact)
4. Upload artifact → receive CID
5. Prepare smart contract transaction
6. Request wallet signature
7. Submit proposal
8. Return proposal ID

---

### Reviewer

```
aperio fetch <proposalId>
aperio review <proposalId> --approve
```

Flow:

1. Fetch bundle via CID
2. Load into local Git
3. Inspect diff:
   git diff base..head
4. Submit review decision on-chain

---

### Maintainer

```
aperio merge <proposalId>
```

Flow:

1. Fetch proposal bundle
2. Merge locally using Git
3. Resolve conflicts if needed
4. Generate final bundle
5. Upload bundle → get CID
6. Request wallet signature
7. Submit merge transaction
8. Update canonical HEAD

---

### Release

```
aperio release v1.2.0
```

Flow:

1. Read current HEAD
2. Associate version with commit + CID
3. Submit release transaction
4. Mark as stable

---

## 🔐 Wallet Interaction

### Principle

> The CLI never holds private keys.

Instead:
- it constructs the transaction
- requests a signature from the user wallet
- submits the signed transaction

---

### UX Behavior

When a signature is required:

```
Waiting for signature...
```

User receives wallet prompt:
- "Approve code proposal"
- "Approve merge"
- "Create release"

After approval:

```
Transaction submitted ✔
```

---

## 🧩 CLI Command Set

### Contributor

- `git push polkadot`

---

### Reviewer

- `aperio fetch <proposalId>`
- `aperio review <proposalId> --approve`
- `aperio review <proposalId> --reject`

---

### Maintainer

- `aperio merge <proposalId>`
- `aperio release <version>`

---

### Utility

- `aperio status`
- `aperio repo`
- `aperio proposals`

---

## ⚙️ CLI Responsibilities

### Git Integration
- detect repo root
- read HEAD and base commits
- generate bundles

### Artifact Handling
- package repo into bundle
- upload to Bulletin Chain
- retrieve CID

### Contract Interaction
- build transaction payloads
- encode parameters
- submit to network

### Wallet Communication
- request signature
- wait for approval
- broadcast signed transaction

---

## 🧠 UX Design Rules

### No Protocol Leakage
Avoid exposing:
- raw transaction hashes
- low-level encoding
- CID internals unless requested

---

### Minimal Commands
Prefer:
```
git push polkadot
```

Avoid:
```
aperio propose --cid ...
```

---

### Human Language
Use:
- "Proposal submitted"
- "Merge completed"
- "Release created"

Avoid:
- "Transaction executed"

---

## 🧠 Final Mental Model

- Git → manages code  
- CLI → manages interaction  
- Wallet → confirms intent  
- Contract → records truth  
- Bulletin → stores data  

---

## 🎯 One-Line Summary

> The CLI makes a decentralized repository protocol feel like a normal Git workflow, with a single explicit confirmation step replacing centralized trust.
