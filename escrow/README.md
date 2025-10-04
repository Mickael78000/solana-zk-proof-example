# Anchor Escrow 2025

## 🆕 Updated for Solana Kit, Kite, and Codama

## 🆕 Now includes Rust/LiteSVM tests - check out `programs/escrow/src/tests.rs`

[![CI Badge](https://github.com/mikemaccana/anchor-escrow-2025/actions/workflows/tests.yaml/badge.svg)](https://github.com/mikemaccana/anchor-escrow-2025/actions)

**Start here for your first real Solana program / smart contract** (Solana generally uses the word 'program', older blockchains use 'smart contract'). As the saying goes, "All Solana programs are variations of an escrow."

This makes **Anchor Escrow 2025** the perfect starting point for anyone diving into Solana program development with a practical, real-world application.

**Anchor Escrow 2025** provides:

- Full compatibility with the latest Rust, Agave CLI, Node.js, Anchor, and Solana Kit.
- Clean builds with zero warnings or errors.
- Testing via npm and Node.js, avoiding third-party package managers or test runners.

## Animated walk through

Check out the **full animated explanation** of the Escrow program (also called Swap) from this video from QuickNode:

[![Full animated explanation of the Escrow program](https://img.youtube.com/vi/B5eBWWQfQuM/maxresdefault.jpg)](https://www.youtube.com/watch?v=B5eBWWQfQuM)

This repository is [designed for teaching and learning](CHANGELOG.md).

## Introduction

This Solana program implements an **escrow**, enabling secure token swaps between users. For example, Alice can offer 10 USDC in exchange for 100 WIF.

Without an escrow, users face significant risks:

- **Traditional finance** charges 1-6% in fees, eating into your funds.
- **Manual swaps** are prone to fraud. If Bob takes Alice's 10 USDC but doesn't send the 100 WIF, or if Alice fails to deliver after receiving Bob's tokens, someone gets burned.

The **Anchor Escrow 2025** program acts as a trusted intermediary, releasing tokens only when both parties meet the agreed terms. This ensures Alice and Bob each receive 100% of their desired tokens, securely and without middleman fees.

## Versions

Verify your local environment with:

```bash
bash show-versions.sh
```

This repository was tested with:

```
OS:
  MacOS 15.4.1
Solana CLI:
  solana-cli 2.1.21 (src:8a085eeb; feat:1416569292, client:Agave)
Anchor:
  anchor-cli 0.31.1
Node:
  v22.14.0
Rust:
  rustc 1.86.0 (05f9846f8 2025-03-31)
build-sbf version:
  solana-cargo-build-sbf 2.1.21
```

Using different versions may cause compatibility issues.

## Usage

1. Clone the repository:

   ```bash
   git clone https://github.com/mikemaccana/anchor-escrow-2025.git
   cd anchor-escrow-2025
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

3. Run tests:

   ```bash
   # RUSTUP_TOOLCHAIN is needed for consistent builds per
   # https://solana.stackexchange.com/questions/21664/why-is-the-same-commit-of-an-anchor-repo-giving-different-results-when-run-at-di
   # TODO: remove when no longer necessary
   RUSTUP_TOOLCHAIN=nightly-2025-04-16 anchor test
   ```

4. Deploy the program:
   ```bash
   anchor deploy
   ```

## Changelog and Credits

See the [CHANGELOG](CHANGELOG.md) for updates and contributor credits.
