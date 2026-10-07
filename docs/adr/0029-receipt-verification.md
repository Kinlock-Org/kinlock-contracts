> Synced from Kinlock-Org/.github. Do not edit here.

# 0029 — Receipt verification: tiers, results, and the indexer lookup
Status: Accepted
Date: 2026-10-07
## Context
Receipts live at `/r/{txHash}/{eventIndex}` and must be verified from chain data, never the database (ARCHITECTURE.md §7.3). Tier 1 (RPC) only reaches about 7 days back. Tier 2 (`get_lock`) needs to know which lock and tranche a receipt refers to, which a transaction hash alone doesn't say once RPC has forgotten the transaction. The scaffolded `verifyReceipt(ref)` had no network config and a result that couldn't explain a failure.
## Decision
- `verifyReceipt(config, ref)` (config as in ADR-0027) returns `{ valid, tier, kind, reason, receipt }`; owner-approved.
- **Tier 1 `live_rpc`:** the transaction is in RPC retention; the receipt is valid only if that exact event was emitted by the configured contract in a successful call, is `Released`, `Refunded` or `Declined`, and has a known `schema_version`. All details shown come from the event.
- **Tier 2 `lock_state`:** the indexer's new `GET /events/{txHash}/{eventIndex}` says which lock and tranche (a lookup aid only); `get_lock` must confirm it — the tranche released with that amount, or the lock in that end state with that returned amount. Payment is confirmed; transaction details are not.
- `reason`: `verified`, `not_found` (no Kinlock receipt event there: a tampered or wrong link), `mismatch` (the chain contradicts the lookup), `unverifiable` (too old for RPC and not confirmable: no indexer, indexer down, or the lock entry has expired — tier 3, M2-16). Only `verified` is valid; errors never throw except for malformed input.
## Consequences / trade-offs
The receipt page can say "Valid", "Not valid" or "Can't verify yet" honestly. Tier 2 trusts the indexer only to point at a lock; the chain decides. Receipts whose lock entry has expired stay unverifiable until an archive source is chosen (DEC-09).
## Docs updated
Extends ADR-0027. Roadmap M2-06.
