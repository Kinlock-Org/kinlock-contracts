# kinlock-contracts

The `kinlock` Soroban contract (registry + vault modules), tests, deploy scripts, and generated TypeScript bindings.

> **Status: scaffold.** Structure and data models are drafted; features are not built. Testnet only.

## Quick start
```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
stellar contract build
scripts/localnet.sh        # local network
```

## Read first
- `docs/ARCHITECTURE_ESSENTIALS.md` (short; read at the start of every task)
- `AGENTS.md` (rules for humans and agents) and `CLAUDE.md`
- `ROADMAP.md`: **every PR updates it**

Docs in `docs/` are read-only copies synced from [Kinlock-Org/.github](https://github.com/Kinlock-Org/.github).
