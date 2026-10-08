> Synced from Kinlock-Org/.github. Do not edit here.

# Kinlock — Architecture Essentials (v0.3)

> Quick-reference outline. Full detail: `ARCHITECTURE.md`. Product intent: `PRD.md`.
> If code and this file disagree, stop and flag it. Don't silently pick one.

## 1. What this is
- **Anyone, anywhere can lock USDC on Stellar/Soroban for a verified payee anywhere in the world.** Worldwide by design, launched market by market.
- A sender locks funds to a **verified payee** (School or Rent) in a **supported country**.
- Funds go only to the payee's **snapshotted payout address** or back to the sender.
- Receipts prove **payment to a verified payee**. They do **not** prove service delivery.

## 2. Non-negotiable principles
- **Chain is truth.** Postgres is for lists. **Money-moving pages read chain state**, not the DB.
- **Non-custodial.** No service holds keys or funds.
- **No PII on-chain** or in the public registry.
- **No privileged path to locked funds.** Admin, attesters, pause: none can move or redirect existing locks.
- **Country-agnostic core.** Country (ISO 3166-1) and local currency (ISO 4217) are **registry data**. Never hard-code a country, currency, anchor, or locale in contract, SDK, indexer, or app.
- **Build the minimum.** If it doesn't serve the M0–M3 pilot, defer it.
- **Roadmap is always current.** **Every PR updates `ROADMAP.md`** (row statuses, new rows, Changelog entry). No exceptions. CI enforces it.

## 3. Repos
| Repo | Owns |
|---|---|
| `kinlock-contracts` | One contract `kinlock` (modules: registry, vault), bindings, deploy |
| `kinlock-sdk` | `packages/sdk` + `services/indexer` (events → Postgres → list API) |
| `kinlock-app` | Next.js: payee request, sender, claim, attester checklist, receipts |
| `kinlock-registry` | Public payee JSON, schema, hash-check CI |
| `kinlock-ramp` | **Conditional.** Create only if the M0 anchor spike passes |

Flow: contracts → bindings → sdk → app. Registry is data only.

## 4. Stack (pin current stable versions at start)
- Rust + `soroban-sdk`; `cargo test` + `proptest`; `soroban-budget-assert` in CI
- USDC via SAC (SEP-41), allowlisted; 7 decimals; `i128`
- `@stellar/stellar-sdk` + generated TS bindings
- Next.js (App Router) + TypeScript + Tailwind
- Node + Fastify + Zod; PostgreSQL + **Drizzle**; in-process scheduling
- Logs (pino) + `/health` with indexer lag
- Local: Stellar quickstart + docker compose. Two RPC providers in deployed envs
- **Deferred:** Redis/queue, OpenTelemetry, SEP-12/38/45, passkeys, fee-bump relayer, object storage

## 5. Contract essentials

### Data
- `Payee`: payout, category (School|Rent), status (Active|Suspended|Revoked), status_changed_at, attester, meta_hash, registered_at
- `Lock`: sender, payee_id, **payout (snapshot)**, token, total, released, returned, ref_hash, tranches (≤12), expires_at, state (Open|Completed|Refunded|Declined)
- `Tranche`: amount, unlock_at, released
- Enums are append-only. No purpose field (purpose = category)

### Entry points
- Admin: `init`, `add/remove_attester`, `add/remove_token`, `set_paused_new_locks`, `set_caps`, `upgrade`
- Registry: `register_payee`, `set_status`, `update_payout` (new locks only)
- Vault: `create_lock`, `release`, `refund`, `decline`, `bump_lock` (permissionless), reads

### Rules
- `release`: caller is `lock.payout`; payee Active; `unlock_at ≤ now < expires_at`
- `refund` (sender): `now ≥ expires_at`, **or** payee Revoked, **or** payee Suspended past 14-day grace
- `decline` (payee): any time while Open; returns remainder to sender
- `create_lock` rejects: unallowlisted token, inactive payee, dust, >12 tranches, `unlock_at > expires_at`, expiry beyond max duration, `sender == payout`, cap breaches
- **No fee. No SenderApproval. No mutual cancel. No payout timelock**

### Invariants (property-test all)
1. `released + returned ≤ total`; terminal ⇒ equality
2. Tranches sum to `total`; `unlock_at ≤ expires_at`
3. Funds exit only to `lock.payout` or `lock.sender`
4. Tranche released at most once
5. Release only inside its time window by `lock.payout`
6. Refund only on expiry, Revoked, or Suspended+grace
7. `lock.payout` immutable
8. Admin/attester/pause/allowlist never block release, decline, refund
9. `total_locked` = sum of remainders of Open locks
10. Failed transfer reverts all state

### Implementation rules
- State before token transfer
- `require_auth` everywhere; support C-addresses
- No unbounded storage; no per-user on-chain lists
- Extend TTL on create to `expires_at + 30d`; keep `bump_lock` public
- Max lock duration must stay **below network max entry TTL** (verify)
- Events carry `schema_version`; test upgrades on a testnet snapshot

## 6. Off-chain essentials
- **Indexer:** `getEvents` from persisted cursor; RPC retention is limited, so run continuously and alert on lag; idempotent on `(tx_hash, event_index)`
- **List API:** `/locks`, `/locks/:id`, `/payees`, `/health`
- **Claim link:** `/claim/{id}#r={reference}&s={salt}`. Fragment never reaches a server. Payee client checks `sha256(reference || salt) == ref_hash`
- **Request link:** `/send?payee=…&amount=…&ref=…&schedule=…` pre-fills the form; carries no authority
- **Preflight:** balance, payee Active, recent payout change, duplicate reference, authorized USDC trustline
- **Receipts:** `/r/{txHash}/{eventIndex}`; no table, no slugs. Copy says "Payment to verified payee"
- **Verification tiers:** (1) live RPC, (2) `get_lock` state, (3) archive provider, decided before mainnet
- **Cash-out (MVP):** guidance per supported country via payee's wallet/anchor, or hold USDC. In-app ramp only after M0
- **Local-currency display:** indicative equivalent in payee's `local_currency` via a swappable `RateProvider`; USD only if no reliable rate. Display only, never in money logic
- **i18n-ready:** UI strings externalized (`messages/`); `Intl` for numbers, currencies, dates. Full translations deferred

## 7. Tables (indexer-owned)
`chain_events`, `indexer_cursor`, `locks`, `tranches`, `payees` (with `country`, `local_currency` for filter/display only).
Deferred: `subscriptions`, ramp tables. **No** receipts, private-reference, attestation, or user tables.

## 8. Registry rules
- One public JSON per payee under `payees/<country>/`; no PII, no evidence
- Required fields include `country` (ISO 3166-1 alpha-2) and `local_currency` (ISO 4217)
- CI enforces: country ∈ `supported-countries.json`; attester authorized for that country in `attesters/<handle>.json`; directory matches country; slug globally unique
- `meta_hash` = SHA-256 of sorted-key compact JSON; CI verifies match on-chain
- Attester keeps identity evidence privately and completes the onboarding checklist (trustline, XLM float, test release)

## 9. Key decisions (ADR index)
1. Non-custodial; contract holds funds
2. **One contract**, modular code
3. No on-chain enumeration
4. Tiered receipt verification
5. Reference + salt in URL fragment
6. **Payout snapshotted per lock**
7. Attester updates payout for new locks only
8. Pause affects only new locks
9. **No protocol fee in MVP**
10. Upgradeable via multisig; timelock before mainnet
11. Postgres for lists; chain reads for actions
12. **Payee `decline`** replaces mutual cancel
13. Registry bound by `meta_hash`
14. Deterministic receipt URL
15. Refund on Revoked / Suspended+grace
16. Strict expiry boundary (release `<`, refund `≥`)

## 10. Trust assumptions (state them, don't hide them)
- **Attesters decide which payees exist.** A fraudulent attester can divert *future* locks. Existing locks are safe from redirect
- Admin multisig can upgrade the contract (timelock before mainnet)
- USDC issuer controls apply (freeze/authorization)
- Public ledger: sender↔payee linkage is visible to anyone
- **Geography is not enforceable on-chain.** Supported-countries and attester-country rules live in registry CI and app policy. Anyone can call the contract directly

## 11. Top failure risks (in order)
1. **B1** Payees can't operate a wallet → assisted onboarding, M0 validation
2. **B2** Sender cost vs paying the school's bank directly → M0 cost model
3. **B3** No local-currency off-ramp in a market → M0 spike per market; ramp deferred; payees may hold USDC
4. **B4** Cold start + attester trust → one-community pilot
5. **B15** Legal, per market → counsel per market; testnet only until reviewed
6. **B19** "Worldwide" multiplies per-market work → market-by-market playbook; ≥ 2 pilot markets only
7. **B20** Country assumptions leak into code → principle + CI guard; second pilot market is the test

None is a contract bug. Don't over-invest in the contract before M0 answers these.

## 12. Edge cases that must be tested
- Missing / full / unauthorized trustline on release (reverts, retryable)
- `now == expires_at` boundary
- `unlock_at > expires_at` rejected
- Payee Suspended → refund after grace; Revoked → immediate refund
- Token removed from allowlist with open locks
- C-address sender and payee
- Partial release then refund/decline returns only remainder
- Duplicate reference for same payee (UI warns; contract allows)
- Event schema change after upgrade
- Payee country not in supported list → registry CI fails
- Attester vouching outside authorized countries → registry CI fails
- Currency with no rate → USD only, no invented rate
- Zero-decimal and 3-decimal currencies, non-Latin names, RTL/LTR text render correctly (display only)

## 13. Don'ts
- Don't add admin functions that move, freeze, or redirect locked funds
- Don't let payout change on an existing lock
- Don't make the DB the proof for receipts or the source for money pages
- Don't store references, names, or evidence on-chain, in the registry, or on servers
- Don't hard-code any country, currency, anchor, or locale; read them from registry data
- Don't put country logic or geofencing in the contract
- Don't inline user-visible strings; use the message files
- Don't add fees, release modes, categories, or services without a documented need
- Don't use "proof of use" or "service delivered" wording
- Don't ship mainnet without audit and legal review
- Don't open, finish, or merge a PR without updating `ROADMAP.md`
- Don't mark a roadmap row `DONE` without running its verification, and never delete rows

## 14. Open questions
- Archive source for verification and indexer backfill
- ~~Network max entry TTL~~ Resolved: `MAX_LOCK_DURATION` = 149 days (ADR-0020); re-check mainnet before launch
- Optional `refund_to` address?
- Pilot markets and the supported-countries policy (M0)
- Which wallet/anchor route gives payees local-currency cash-out, per pilot market (M0)
- Rate provider for indicative multi-currency display
- Enforce attester-country scope on-chain too, or registry CI only (current plan)?
- New-payee release delay parameters (P1)
