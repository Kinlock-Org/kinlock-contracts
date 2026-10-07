> Synced from Kinlock-Org/.github. Do not edit here.

# 0026 — Distribute TypeScript packages as GitHub Release tarballs
Status: Accepted (supersedes ADR-0023)
Date: 2026-10-07
## Context
ADR-0023 chose public npm under `@kinlock`, which needs an npm account and org owned by the project. The owner could not create an npm account, so the SDK and the contract bindings could not be published, and the app could not install the SDK. Alternatives considered: GitHub Packages (needs a GitHub token even to install public packages, and the scope would have to be `@kinlock-org`); installing from git source (the SDK must be built after install, and the bindings live in a subfolder of another repo); JSR (scope and naming changes, and the generated bindings may not meet its type rules — unverified).
## Decision
CI attaches the packed package (`npm pack` / `pnpm pack` output, a `.tgz`) to a GitHub Release when a release tag is pushed: `bindings-vX.Y.Z` in `kinlock-contracts` for `@kinlock/contract`, `sdk-vX.Y.Z` in `kinlock-sdk` for `@kinlock/sdk`. Package names stay `@kinlock/...`. Consumers depend on the release-asset URL, for example
`"@kinlock/sdk": "https://github.com/Kinlock-Org/kinlock-sdk/releases/download/sdk-v0.1.0/kinlock-sdk-0.1.0.tgz"`.
CI uses the workflow's own `GITHUB_TOKEN`; there is no publish secret. Nobody publishes from a laptop. The same checks as before gate a release (tag matches the package version; bindings match a fresh generation; SDK lint, typecheck and tests pass). A published version is never re-uploaded: fixes ship as a new version.
## Consequences / trade-offs
No accounts or secrets to manage, and the repos are public, so anyone can install without a token. Upgrades change a URL instead of a version range, so there are no semver ranges or `npm update`. Consumers' lockfiles record each tarball's integrity hash, so a replaced asset fails installation instead of silently changing code. Moving to an npm registry later is a one-line change per dependency once an npm org exists; this ADR would then be superseded.
## Docs updated
Supersedes ADR-0023; amends DEC-03. Process: the `bindings` workflow in `kinlock-contracts` (M1-18) and the `publish` workflow in `kinlock-sdk` (M2-17).
