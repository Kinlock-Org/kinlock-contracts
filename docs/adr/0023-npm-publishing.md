> Synced from Kinlock-Org/.github. Do not edit here.

# 0023 — Publish TypeScript packages to npm under @kinlock
Status: Accepted
Date: 2026-10-06
## Context
The SDK and app need the contract's generated TypeScript bindings, and integrators need the SDK (DEC-03, roadmap F-11, M1-18). Options were public npm, GitHub Packages (which needs a token even to install public packages), or not publishing yet.
## Decision
Publish public packages to npm under the `@kinlock` scope (for example `@kinlock/contract` for the bindings and `@kinlock/sdk`). Packages are published only by CI, on a release tag, using an npm automation token stored as the repo secret `NPM_TOKEN`. Nobody publishes from a laptop.
## Consequences / trade-offs
Anyone can install without a token. An owner must create the npm org `kinlock` and manage the token; until then, packages are built but unpublished. If the npm org name is unavailable, this ADR is superseded with the chosen name.
## Docs updated
Resolves DEC-03. Process: `kinlock-contracts` publish workflow (M1-18).
