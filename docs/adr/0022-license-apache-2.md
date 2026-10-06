> Synced from Kinlock-Org/.github. Do not edit here.

# 0022 — License: Apache-2.0
Status: Accepted
Date: 2026-10-06
## Context
The repos had no license (DEC-02, roadmap F-10), so nobody could legally reuse or contribute to the code. Kinlock is open source and plans contributor sprints (Drips Wave). Stellar's own tooling (stellar-cli, soroban-sdk) is Apache-2.0.
## Decision
Every Kinlock repo is licensed under Apache-2.0: the unmodified license text in `LICENSE` at each repo root, and `Apache-2.0` in every `package.json` and `Cargo.toml`. The template lives in `.github/templates/LICENSE`.
## Consequences / trade-offs
Permissive, with an explicit patent grant from contributors, which matters for financial software. Anyone may build closed products on the code, including competitors. The copyright holder is not named until the legal entity is decided (DEC-13).
## Docs updated
Resolves DEC-02. `LICENSE` added to all repos.
