# kinlock-contracts

[![CI](https://github.com/Kinlock-Org/kinlock-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Kinlock-Org/kinlock-contracts/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

The `kinlock` Soroban contract — payee registry and lock vault in one contract, modular code — plus its tests, deploy tooling, and generated TypeScript bindings.

> **Status: functionally complete, hardening open.** All 20 entry points are implemented with 107 unit tests and property tests over invariants 1–10, deployed to testnet behind a 2-of-3 multisig. Budget tests (`M1-16`), the integration suite (`M1-15`), the internal review (`M1-22`), and the external audit (`H-04`) are still open — which is exactly the gap between "testnet pilot" and "mainnet". See `ROADMAP.md`.

**Try it live (testnet):** [kinlock-app.vercel.app](https://kinlock-app.vercel.app) · **Docs:** [kinlock-org.github.io](https://kinlock-org.github.io) · **Deployment:** [`DEPLOYMENTS.md`](DEPLOYMENTS.md)

## The contract

One contract, `kinlock`. `#![no_std]`, no floating point, `i128` amounts with checked arithmetic, USDC via the Stellar Asset Contract from an admin allowlist.

| Caller | Entry points |
|---|---|
| Deploy-time | `__constructor(admin)` |
| Admin (multisig) | `add_attester`, `remove_attester`, `add_token`, `remove_token`, `set_paused_new_locks`, `set_caps`, `upgrade` |
| Attester | `register_payee`; `update_payout` (vouching attester only); `set_status` (vouching attester or admin) |
| Sender | `create_lock`, `refund` |
| Payee payout address | `release`, `decline` |
| Anyone | `bump_lock`, `get_lock`, `get_payee` |

Rules that the code cannot bend:

- **Funds leave only to `lock.payout` or `lock.sender`.** No admin, attester, pause, or allowlist change opens another path.
- **`lock.payout` is snapshotted at creation and immutable.** A payout update affects new locks only.
- **Release** requires the payout address's auth, an Active payee, and `unlock_at ≤ now < expires_at`. **Refund** requires `now ≥ expires_at`, or a Revoked payee, or Suspended past a 14-day grace. **Decline** is the payee's, any time while Open, and returns only the remainder.
- **Pause blocks new locks only.** Never release, decline, or refund.
- State is written **before** the token transfer, so a failed transfer reverts everything.

Constants live in `contracts/kinlock/src/constants.rs`: `MAX_TRANCHES` 12, `MIN_AMOUNT` 1 USDC, `MIN_EXPIRY_AHEAD_SECS` 1 hour, `MAX_LOCK_DURATION_SECS` **149 days** (lock + 30-day TTL grace must fit the network's max entry TTL — ADR-0020), `SUSPENSION_REFUND_GRACE_SECS` 14 days, `TTL_GRACE_SECS` 30 days, `EVENT_SCHEMA_VERSION` 1, `STORAGE_VERSION` 1. Every stored enum is append-only. All seven events (`PayeeRegistered`, `PayeeStatusChanged`, `PayoutUpdated`, `LockCreated`, `Released`, `Refunded`, `Declined`) carry `schema_version`.

## Quick start

```
cargo fmt --all -- --check
stellar contract build     # first: tests load the built WASM (stellar-cli 27.0.0, as CI pins)
cargo clippy --all-targets -- -D warnings
cargo test --workspace
scripts/localnet.sh        # local Stellar quickstart
scripts/localnet-setup.sh  # fresh local Kinlock: identities, local USDC, contract, one payee
```

Toolchain is pinned in `rust-toolchain.toml` (Rust 1.96.1, `wasm32v1-none`); `soroban-sdk` 28.

## Tests

- **107 unit tests** in `contracts/kinlock/src/test/`: every entry point's success path and every failure path, each `create_lock` validation, explicit-auth assertions (`env.auths()`, not only `mock_all_auths`), boundary timestamps, TTL behavior, and an upgrade test using the real compiled WASM.
- **Property tests** in `contracts/kinlock/tests/properties.rs`: random sequences of create, release, refund, decline, status and payout changes, time jumps, pause, allowlist and roster changes, and frozen accounts, with all ten invariants re-checked after every step.
- `tests/budget.rs` and `tests/integration.rs` are **placeholders** (`M1-16`, `M1-15`); `soroban-budget-assert` isn't on crates.io, so `M1-30` has to source it first.

CI runs fmt → build → a bindings-freshness diff → clippy `-D warnings` → `cargo test --workspace`.

## Deploy

```
STELLAR_ACCOUNT=<keystore identity name> KINLOCK_ADMIN=<admin address> scripts/deploy.sh --dry-run
STELLAR_ACCOUNT=<keystore identity name> KINLOCK_ADMIN=<admin address> scripts/deploy.sh
```

Defaults to testnet; `STELLAR_NETWORK=local` targets `scripts/localnet.sh`. Mainnet requires `KINLOCK_ALLOW_MAINNET=1` **plus** a typed confirmation, and is out of scope until audit and legal review are done. Each deploy writes `deployments/<network>.json` and regenerates `DEPLOYMENTS.md` — never edit either by hand. Admin calls on testnet go through `scripts/multisig-invoke.sh`, which refuses to run against mainnet.

Current testnet: `CCSHDQFRYFC3AHV5NE6ULQW6X2CMG5RPANBORDXJGSUD6UKECASJQBRI`, 2-of-3 multisig admin, Circle testnet USDC allowlisted, one test attester, no production payees. Setup detail: [`deployments/testnet-setup.md`](deployments/testnet-setup.md).

## TypeScript bindings

`bindings/typescript` holds the generated `@kinlock/contract` client (currently 0.1.0). Never hand-edit it: change the contract, run `scripts/gen-bindings.sh`, commit the result — CI fails on a stale copy. To release, bump with `scripts/gen-bindings.sh <x.y.z>`, merge, then push the tag `bindings-v<x.y.z>`; the `bindings` workflow attaches a `.tgz` to that GitHub Release (workflow `GITHUB_TOKEN`, no publish secret — ADR-0026). `kinlock-sdk` consumes it from there.

## Read first

- `docs/ARCHITECTURE_ESSENTIALS.md` (short; read at the start of every task)
- `AGENTS.md` (rules for humans and agents) and `CLAUDE.md`
- `ROADMAP.md`: **every PR updates it**
- [`SECURITY.md`](SECURITY.md): threat model, mapped to the invariants and the tests that check each one

Contract changes are **ask-first** and labeled `security-sensitive`: entry points, storage layout, events, invariants, constants, errors, auth, or where funds can go.

Docs in `docs/` are read-only copies synced from [Kinlock-Org/.github](https://github.com/Kinlock-Org/.github).

Found a documentation gap (missing, unclear, or outdated docs)? File it at [Kinlock-Org.github.io](https://github.com/Kinlock-Org/Kinlock-Org.github.io/issues/new/choose) with `area:contract`, the org's documentation hub, not here.
