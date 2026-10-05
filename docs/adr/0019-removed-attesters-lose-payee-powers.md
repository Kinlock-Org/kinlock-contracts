> Synced from Kinlock-Org/.github. Do not edit here.

# 0019 — Removed attesters lose powers over their payees
Status: Accepted
Date: 2026-10-05
## Context
`set_status` and `update_payout` are restricted to the payee's vouching attester (and, for `set_status`, the admin). The docs don't say what happens when that attester is removed from the roster, for example after a key compromise (`ARCHITECTURE.md` E15).
## Decision
An attester must still be on the roster to act on payees it vouched for. After removal, only the admin can change those payees' status. Nobody can update their payout until a new attester process is defined (runbook H-18).
## Consequences / trade-offs
A compromised or departed attester can't keep changing payees once removed. The admin must handle orphaned payees (suspend, revoke, or re-attest). Existing locks are unaffected either way, because each lock keeps its own payout snapshot.
## Docs updated
Pending roadmap F-19: add to `ARCHITECTURE.md` §4.3 and §13.2 (E15).
