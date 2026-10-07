> Synced from Kinlock-Org/.github. Do not edit here.

# 0025 — SDK public API additions for the app
Status: Accepted
Date: 2026-10-06
## Context
`AGENTS.md` §8.2 fixed the SDK's public API at nine functions. Building the app (fast-track step 3) needs three more capabilities that already exist inside the SDK: converting what senders type into token base units, creating and checking the salted reference hash, and the payee's payment-request link (PRD J2, PAY-3). The rules also say amount conversion and link formats live only in the SDK, so the app can't reimplement them.
## Decision
Add six functions to the public API:
- `toBaseUnits`, `fromBaseUnits` (and the `USDC_DECIMALS` constant): exact amount conversion, never rounding.
- `generateSalt`, `computeRefHash`: the format in ADR-0024.
- `buildRequestLink`, `parseRequestLink`: the request-link format.

A fixture-payee helper was considered and not added: the app shows payees from the indexer (fast-track step 5), or a static list until then.
## Consequences / trade-offs
The app uses one implementation of each rule. These signatures are now a public contract: changing them needs approval and a version bump. The SDK's test pins the export list to exactly these fifteen functions.
## Docs updated
`AGENTS.md` §8.2.
