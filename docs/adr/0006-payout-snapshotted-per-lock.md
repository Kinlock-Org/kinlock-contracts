> Synced from Kinlock-Org/.github. Do not edit here.

# 0006 — Payout snapshotted per lock
Status: Accepted
Date: 2026-10-05
## Context
Eliminates redirect attacks and the timelock machinery.
## Decision
Payout snapshotted per lock.
## Consequences / trade-offs
Payee key loss strands locks until expiry.
## Docs updated
Recorded from `ARCHITECTURE.md` §12 (v0.2 starting set). Full reasoning: `ARCHITECTURE.md` §13.
