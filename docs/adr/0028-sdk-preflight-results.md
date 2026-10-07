> Synced from Kinlock-Org/.github. Do not edit here.

# 0028 — SDK preflight: chain checks block, indexer checks warn
Status: Accepted
Date: 2026-10-07
## Context
`preflight` (ARCHITECTURE.md §5.1) covers five checks. Three can be read from chain: sender balance, payee Active, and an authorized trustline with room on the payout account. Two can't: the contract stores neither when a payout last changed nor which locks use a reference, and RPC event history (about 7 days) is too short to rely on at the edge of the 7-day window. The scaffolded result type had only `ok: boolean`, which can't express "couldn't check".
## Decision
- `KinlockConfig` gains optional `indexerUrl` (ADR-0027 type). It is used only for preflight's two warnings, never for money pages.
- `preflight(config, params, now?)` returns five `PreflightResult`s in a fixed order, each with `status` (`pass` | `fail` | `unknown`), `severity` (`block` | `warn`) and a `messageKey` (`preflight.<check>.<detail>`) for the app to translate. Owner-approved.
- `block`: sender balance, payee Active, payout trustline (missing, not authorized, or no room for the total; a contract (C) payout or the issuer's own account needs none).
- `warn`: payout changed in the last 7 days (or the chain payout differs from what the indexer has seen), and a lock with the same `ref_hash` for this payee. The SDK re-checks every indexer row against the payee and reference it asked for.
- A check that can't complete (RPC or indexer unreachable, no `indexerUrl`) returns `unknown`, never `pass`.
- The list API gains `GET /locks?ref_hash=` and `GET /payees?payee_id=` for these two checks.
## Consequences / trade-offs
The app can block on hard failures and show warnings and "couldn't check" honestly. The two warnings depend on the indexer being current; a lagging indexer can miss a very recent duplicate, which the chain payout comparison partly covers for payout changes.
## Docs updated
Extends ADR-0027. Roadmap M2-05.
