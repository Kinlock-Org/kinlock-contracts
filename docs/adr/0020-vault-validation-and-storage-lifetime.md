> Synced from Kinlock-Org/.github. Do not edit here.

# 0020 — Vault validation and storage-lifetime rules
Status: Accepted
Date: 2026-10-05
## Context
Implementing the vault (`kinlock-contracts` `feat/vault`, roadmap M1-05..M1-10) required decisions the docs don't cover: how long a lock must stay in storage, what payout addresses are valid, what caps are sane, and how future upgrades find the storage layout.
## Decision
- `create_lock(sender, token, payee_id, tranches, ref_hash, expires_at)` takes the sender explicitly (as in ADR-0018). Tranches are passed as `{amount, unlock_at}`; `total` is their checked sum.
- A lock and its payee must stay in storage until `expires_at + TTL_GRACE`. If the network's max entry TTL can't cover that, `create_lock` fails with `LockTtlTooLong` (code 33) instead of creating a lock that could be archived while holding funds.
- A payee's payout can't be the contract's own address (`InvalidPayout`, code 16), in `register_payee` and `update_payout`.
- `set_caps` requires `MIN_AMOUNT <= max_lock_amount <= max_total_locked`, so a tiny cap can't act as a silent pause.
- The constructor writes a storage layout version (`STORAGE_VERSION = 1`); an upgrade that changes the layout must bump it and migrate.
## Consequences / trade-offs
- On testnet (`max_entry_ttl` = 3,110,400 ledgers ≈ 180 days at a 5 s target close, read with `stellar network settings` on 2026-10-05) a contract can extend an entry to at most one ledger less. 150 + 30 days needs exactly 3,110,400 ledgers, one too many, so `MAX_LOCK_DURATION` is **149 days** (resolves DEC-07). Contract tests run with the real network limit.
- TTL is counted in ledgers and converted from seconds at an assumed 5 s close; faster ledgers shorten real-world coverage. The 30-day grace absorbs small drift.
## Docs updated
`ARCHITECTURE.md` §4.1 and E12 (149 days), `PRD.md` §13 open question 6. Still pending under F-19: `ARCHITECTURE.md` §4.2, §4.3 and `ARCHITECTURE_ESSENTIALS.md` §5.
