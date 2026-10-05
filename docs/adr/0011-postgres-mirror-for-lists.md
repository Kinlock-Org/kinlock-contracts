> Synced from Kinlock-Org/.github. Do not edit here.

# 0011 — Postgres mirror for lists
Status: Accepted
Date: 2026-10-05
## Context
Fast UI without trusting the DB.
## Decision
Postgres mirror for lists; chain reads for actions.
## Consequences / trade-offs
Two read paths to maintain.
## Docs updated
Recorded from `ARCHITECTURE.md` §12 (v0.2 starting set). Full reasoning: `ARCHITECTURE.md` §13.
