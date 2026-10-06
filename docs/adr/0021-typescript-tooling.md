> Synced from Kinlock-Org/.github. Do not edit here.

# 0021 — TypeScript tooling
Status: Accepted
Date: 2026-10-06
## Context
The TypeScript repos (`kinlock-sdk`, `kinlock-app`, and `kinlock-registry`'s scripts) had no linter, test runner, or script runner, so CI could only typecheck and build (DEC-22). Registry CI also needs a JSON Schema validator for the payee and attester schemas (DEC-23). The project rule is "minimum viable": as few tools and dependencies as do the job.
## Decision
- **Lint and format:** Biome (`@biomejs/biome`), one tool for both, configured to the existing style (2 spaces, double quotes, 100 columns).
- **Tests:** Vitest in sdk and app.
- **Running TypeScript scripts** (indexer dev server, registry scripts): tsx.
- **Registry schema validation:** Ajv with ajv-formats (the schemas are JSON Schema draft 2020-12 and use the `date` format).
- **i18n:** keep the app's small typed `t()` helper over `messages/<locale>.json` and `Intl`; adopt a library only when a second language is planned (full translations are deferred).
## Consequences / trade-offs
One dev dependency each for lint, tests, and script running; fast CI. Biome has fewer Next.js-specific rules than ESLint's Next plugin. Versions are pinned in each repo's `package.json` and lockfile.
## Docs updated
Resolves DEC-22 and DEC-23 in `ROADMAP.md`. Applied in kinlock-sdk and kinlock-app (`chore/ts-tooling`); Ajv lands with registry row M1-24.
