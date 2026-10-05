> Synced from Kinlock-Org/.github. Do not edit here.

# 0016 — Strict expiry boundary
Status: Accepted
Date: 2026-10-05
## Context
No overlap window.
## Decision
Strict boundary: release if `now < expires_at`, refund if `now ≥ expires_at`.
## Consequences / trade-offs
Payee must act before expiry.
## Docs updated
Recorded from `ARCHITECTURE.md` §12 (v0.2 starting set). Full reasoning: `ARCHITECTURE.md` §13.
