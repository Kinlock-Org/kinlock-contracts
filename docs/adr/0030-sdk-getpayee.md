> Synced from Kinlock-Org/.github. Do not edit here.

# 0030 — SDK `getPayee` (chain read)
Status: Accepted
Date: 2026-10-07
## Context
The sender's lock page (M3-08) must show Refund only when the contract allows it (hard rule 8): after expiry, or when the payee is Revoked, or Suspended for at least the 14-day grace. The last two need the payee's on-chain status and `status_changed_at`. Money pages may not use the indexer for this (hard rule 3), and the SDK only read the payee inside `preflight`.
## Decision
Add one public function, `getPayee(config, payeeId)`: a chain read by simulation (nothing signed or sent) returning the payee (`payout`, `category`, `status`, `statusChangedAt`, `attester`, `metaHash` hex, `registeredAt`) or `null` when it isn't registered. Owner-approved over "expiry only" and "let the contract refuse". The export list grows to sixteen functions. Refund eligibility itself stays in the app, mirroring the contract rule.
## Consequences / trade-offs
Money pages can apply the full refund rule from chain state. Released as `@kinlock/sdk` 0.3.0.
## Docs updated
`AGENTS.md` §8.2 (template). Roadmap M3-08.
