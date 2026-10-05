> Synced from Kinlock-Org/.github. Do not edit here.

# 0017 — Country-agnostic core
Status: Proposed
Date: 2026-10-05
## Context
v0.3 widened Kinlock from "abroad to Nigeria" to any sender paying a verified payee in any supported country, opened market by market. Country-specific assumptions in code would force a rewrite for every new market (risk B20), and the contract cannot enforce geography because anyone can call it directly (B19).
## Decision
Country (ISO 3166-1 alpha-2) and local currency (ISO 4217) are registry data only. Nothing in the contract, SDK, indexer, or app hard-codes a country, currency, anchor, or locale. Supported countries and attester-country scope are enforced in registry CI and app policy, not on-chain. Local-currency amounts are display-only, through a swappable `RateProvider`.
## Consequences / trade-offs
A second market needs only registry data and operations. Geography is a stated trust assumption, not a protocol guarantee. Every market still needs its own legal review, attester, cash-out route, and rate source (PRD J8).
## Docs updated
Codifies v0.3 principle #7 (`ARCHITECTURE.md` §1), `AGENTS.md` hard rule 11, `PRD.md` G7. Needs acceptance by a maintainer, then a row in the ADR indexes (`ARCHITECTURE.md` §12, `ARCHITECTURE_ESSENTIALS.md` §9).
