# kinlock-contracts

The `kinlock` Soroban contract (registry + vault modules), tests, deploy scripts, and generated TypeScript bindings.

> **Status: scaffold.** Structure and data models are drafted; features are not built. Testnet only.

## Quick start
```
cargo fmt --all -- --check
stellar contract build     # first: tests load the built WASM (needs stellar-cli 25.2+)
cargo clippy --all-targets -- -D warnings
cargo test --workspace
scripts/localnet.sh        # local network
```

## Deploy (testnet)
```
STELLAR_ACCOUNT=<keystore identity name> KINLOCK_ADMIN=<admin address> scripts/deploy.sh --dry-run
STELLAR_ACCOUNT=<keystore identity name> KINLOCK_ADMIN=<admin address> scripts/deploy.sh
```
Defaults to testnet; `STELLAR_NETWORK=local` targets `scripts/localnet.sh`. Mainnet needs `KINLOCK_ALLOW_MAINNET=1` plus a typed confirmation. Each deploy writes `deployments/<network>.json` and regenerates `DEPLOYMENTS.md` (never edit either by hand). Current deployments: [`DEPLOYMENTS.md`](DEPLOYMENTS.md).

## Read first
- `docs/ARCHITECTURE_ESSENTIALS.md` (short; read at the start of every task)
- `AGENTS.md` (rules for humans and agents) and `CLAUDE.md`
- `ROADMAP.md`: **every PR updates it**

Docs in `docs/` are read-only copies synced from [Kinlock-Org/.github](https://github.com/Kinlock-Org/.github).
