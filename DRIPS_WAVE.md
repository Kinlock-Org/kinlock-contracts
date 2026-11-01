# Kinlock — Drips Wave Application & Review Summary

> Program: [Stellar Wave on Drips Network](https://www.drips.network/wave/stellar) (backed by the Stellar Development Foundation)  
> Organization: [`Kinlock-Org`](https://github.com/Kinlock-Org)  
> Repository: `Kinlock-Org/kinlock-contracts` (Smart Contract Core & TypeScript Bindings)

This document provides a single-page executive and technical summary for the Drips Wave review panel evaluating Kinlock for inclusion in the Stellar Wave Program.

---

## 1. Project Overview & Real-World Utility

**Kinlock** lets anyone, anywhere lock USDC on Stellar (Soroban) for verified institutional payees (schools, landlords) or direct recipients ([ADR-0031](docs/adr/0031-universal-dual-mode-locks.md)), with scheduled milestone tranches, immutable payout addresses, and deterministic refund boundaries.

- **Real-World Problem:** Cross-border tuition payments, rent deposits, and service commitments frequently fail due to lack of escrow trust, high remittance fees, and custodial risk. Senders fear lost or diverted funds; recipients require guaranteed payout upon fulfillment.
- **Stellar / Soroban Primitives:**
  - Written in Rust (`#![no_std]`, `soroban-sdk`).
  - Circle USDC via Stellar Asset Contract (SAC).
  - Sub-invocation authorization verification (`env.auths()`, `require_auth()`).
  - State TTL management with permissionless `bump_lock` extension.
  - Zero-PII reference hashing: claim reference memos exist only in client-side URL fragments (`#r=...&s=...`), never touching the contract or server logs.

---

## 2. Live Deployed Artifacts & Verification

| Resource | Link / Reference |
|---|---|
| **Live Web App (Testnet)** | [kinlock-app.vercel.app](https://kinlock-app.vercel.app) |
| **Documentation Hub** | [kinlock-org.github.io](https://kinlock-org.github.io) |
| **Contract ID (Testnet)** | [`CCSHDQFRYFC3AHV5NE6ULQW6X2CMG5RPANBORDXJGSUD6UKECASJQBRI`](https://stellar.expert/explorer/testnet/contract/CCSHDQFRYFC3AHV5NE6ULQW6X2CMG5RPANBORDXJGSUD6UKECASJQBRI) |
| **Admin Address (2-of-3 Multisig)** | `GDFOEZ673H2FMB4UUD57BPR76PJJQJCK7U2LND25QW5377Y6VUGKCWPV` |
| **Allowlisted USDC SAC Address** | `CAQCFVLOBWSG3DRR6CE67D5Q46MVAJWJ2K7G4GB5T3D56D3N25T7P2N6` |
| **Stellar Expert Block Explorer** | [View Contract on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCSHDQFRYFC3AHV5NE6ULQW6X2CMG5RPANBORDXJGSUD6UKECASJQBRI) |
| **Product Video Demo** | [media/kinlock-brag.mp4](media/kinlock-brag.mp4) |
| **Official Release Tag** | [`v0.1.0-testnet`](https://github.com/Kinlock-Org/kinlock-contracts/releases/tag/v0.1.0-testnet) |

---

## 3. Reviewer Checklist & Repository Hygiene

- [x] **Hosted under Organization:** [`Kinlock-Org`](https://github.com/Kinlock-Org)
- [x] **Permissive License:** Apache-2.0 ([`LICENSE`](LICENSE))
- [x] **Branch Protection:** Enforced on `main` (requires peer PR approvals and passing status checks)
- [x] **Continuous Integration (CI):** Green GitHub Actions running `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace`, and `stellar contract build`
- [x] **Security Hardening:**
  - CodeQL SAST scanning (Rust, JavaScript, GitHub Actions)
  - Semgrep source security scanning
  - Dependency auditing (`cargo audit` against RustSec database)
  - OpenSSF Scorecard CI posture reporting
  - Detailed threat model & invariant mapping in [`SECURITY.md`](SECURITY.md)
- [x] **Testing Rigor:**
  - 111 unit tests covering all entry points, validations, and explicit authorization (`env.auths()`)
  - 9 budget metering tests verifying CPU and memory limits
  - 5 end-to-end integration tests in `tests/integration.rs` covering C-address callers, trustline failures, partial release remainder accounting, and mid-flight token allowlist removals
  - 10 core safety invariants verified via property-based testing (`proptest` over 512 action sequences)
- [x] **Roadmap & Progress Tracking:** Every PR tracked and enforced via [`ROADMAP.md`](ROADMAP.md)

---

## 4. Drips Wave Task Queue & Issue Board

Issues for external contributors follow a strict three-tier complexity mapping aligned with Drips Wave points:
- **Trivial (100 points):** Small, self-contained documentation or minor test fixes.
- **Medium (150 points):** Integration test scenarios, CLI tooling, and SDK boundary assertions.
- **High (200 points):** Complex multi-contract invariants and harness enhancements.

> **Task Safety Rule:** In accordance with `AGENTS.md` and `PRD.md`, security-critical contract modifications (`contracts/kinlock/src/**`, state layouts, fund exits) are restricted to maintainers. Wave tasks focus on additive test suites, fuzzing harnesses, boundary test fixtures, and documentation references.

- **Active Bounty Issues:** [Kinlock Contracts Wave Issues](https://github.com/Kinlock-Org/kinlock-contracts/issues?q=is%3Aissue+is%3Aopen+label%3Adrips-wave%2Cwave)
- **Contribution Guide:** [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`docs/wave-issue-format.md`](https://github.com/Kinlock-Org/.github/blob/main/docs/wave-issue-format.md)

---

## 5. Core Maintainers

| Name | Role | GitHub | Contact |
|---|---|---|---|
| Adenike Ajala | Core Maintainer (Smart Contracts & Protocol) | [@abrcrmb](https://github.com/abrcrmb) | [Email](mailto:adenikeajala2018@gmail.com) |
| Ayobami Yazar | Core Maintainer (Full-Stack & Integrations) | [@YazarAyobami](https://github.com/YazarAyobami) | [Email](mailto:Yazarayobami76@gmail.com) |
