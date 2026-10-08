# kinlock-contracts

[![CI](https://github.com/Kinlock-Org/kinlock-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Kinlock-Org/kinlock-contracts/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

The `kinlock` Soroban contract (registry + vault modules), tests, deploy scripts, and generated TypeScript bindings.

> **Status: active development, testnet only.** Vault and registry entry points (`create_lock`, `release`, `refund`, `decline`, `bump_lock`) are implemented with property, auth, and TTL tests; budget and full integration suites are still open. See `ROADMAP.md`.

**Try it live (testnet):** [kinlock-app.vercel.app](https://kinlock-app.vercel.app) · **Docs:** [kinlock-org.github.io](https://kinlock-org.github.io)

## Quick start
```
cargo fmt --all -- --check
stellar contract build     # first: tests load the built WASM (needs stellar-cli 25.2+)
cargo clippy --all-targets -- -D warnings
cargo test --workspace
scripts/localnet.sh        # local network
scripts/localnet-setup.sh  # fresh local Kinlock: identities, local USDC, contract, one payee
```

## Deploy (testnet)
```
STELLAR_ACCOUNT=<keystore identity name> KINLOCK_ADMIN=<admin address> scripts/deploy.sh --dry-run
STELLAR_ACCOUNT=<keystore identity name> KINLOCK_ADMIN=<admin address> scripts/deploy.sh
```
Defaults to testnet; `STELLAR_NETWORK=local` targets `scripts/localnet.sh`. Mainnet needs `KINLOCK_ALLOW_MAINNET=1` plus a typed confirmation. Each deploy writes `deployments/<network>.json` and regenerates `DEPLOYMENTS.md` (never edit either by hand). Current deployments: [`DEPLOYMENTS.md`](DEPLOYMENTS.md); testnet admin multisig and configuration: [`deployments/testnet-setup.md`](deployments/testnet-setup.md). Admin calls on testnet: `scripts/multisig-invoke.sh`.

## TypeScript bindings
`bindings/typescript` holds the generated client for the contract, released as `@kinlock/contract` (a `.tgz` attached to each `bindings-vX.Y.Z` GitHub Release, ADR-0026). Never edit it by hand: change the contract, run `scripts/gen-bindings.sh`, and commit the result (CI fails on stale bindings). To release, bump the version with `scripts/gen-bindings.sh <x.y.z>`, merge, then push the tag `bindings-v<x.y.z>`; the `bindings` workflow publishes it (needs the `NPM_TOKEN` secret).

## Read first
- `docs/ARCHITECTURE_ESSENTIALS.md` (short; read at the start of every task)
- `AGENTS.md` (rules for humans and agents) and `CLAUDE.md`
- `ROADMAP.md`: **every PR updates it**

Docs in `docs/` are read-only copies synced from [Kinlock-Org/.github](https://github.com/Kinlock-Org/.github).
