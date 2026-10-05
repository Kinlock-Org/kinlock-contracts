> Synced from Kinlock-Org/.github. Do not edit here.

# Kinlock — Architecture

| | |
|---|---|
| **Status** | Draft v0.3 (worldwide scope) |
| **Date** | 2026-10-05 |
| **Reads with** | `PRD.md` (what/why), `ARCHITECTURE_ESSENTIALS.md` (quick reference) |

Library and tool versions are intentionally not pinned. Pin current stable releases at project start and record them in each repo.

### What changed in v0.3 (worldwide scope)

Kinlock is no longer designed around "abroad → Nigeria". The **contract is unchanged**: it never knew about countries. The changes are in the registry, app, indexer, and process:

- **Country-agnostic core** is now a principle (§1, #7). No country, currency, or anchor is hard-coded in contract, SDK, indexer, or app.
- **Registry** gains `country` (ISO 3166-1), `local_currency` (ISO 4217), per-country directories, a `supported-countries.json` list, and an attester-country scope file, all enforced in **registry CI** (§5.3). Not enforced on-chain.
- **App** shows an indicative **local-currency** equivalent through a swappable `RateProvider` (USD only if no reliable rate), uses `Intl` for formatting, and externalizes all UI strings (§5.2).
- **Indexer** stores `country` and `local_currency` for filtering and display only (§6).
- **Security model** states plainly that geography is not enforceable on-chain (§8).
- **Hard questions:** B19, B20, E32 to E40, O19 to O21 (§13).

### What changed from v0.1 to v0.2

**Design bugs found and fixed**
- **Hollow timelock.** v0.1 timelocked payout-address changes "so senders could react", but senders had no way to exit during the window. Now the payout address is **snapshotted per lock** and the timelock machinery is removed.
- **Reference delivery gap.** v0.1 hashed the reference on-chain but had no way to hand the plaintext to the payee without a server holding it. Now the reference and salt travel in the **claim-link URL fragment**; no private reference store.
- **Random receipt slugs were security theater.** The chain is public, so slugs hid nothing. Receipts are now at a deterministic URL derived from the transaction.
- **"Verifiable from chain only" overclaimed.** RPC history is limited. Verification now has defined tiers (§7.3).
- **Missing field.** ADR-9 said fees were fixed per lock but the struct had no field. Resolved by cutting fees from MVP.
- **Contradiction.** `SenderApproval` mode let a sender block a payee forever, which undercut the core promise. Removed.

**Simplifications**
- One contract instead of two. Three release modes → one. Mutual cancel → payee `decline`. No fee logic, no purpose enum, two categories. No Redis, no OpenTelemetry, no ramp service at MVP, no receipts table, no private-data tables.

Full reasoning is in §13.

---

## 1. Principles

1. **The chain is the source of truth.** Postgres is a read-optimized mirror for lists. Pages that drive money actions read from chain.
2. **Non-custodial.** No service holds user keys or funds.
3. **Minimal on-chain surface.** Addresses, amounts, hashes, enums only. No personal data.
4. **No privileged path to locked funds.** Admin, attesters, and pause switches cannot move or redirect existing locks.
5. **Say only what we can prove.** Receipts prove payment to a verified payee, nothing more.
6. **Build the least that tests the hypothesis.** Anything that doesn't serve the M0–M3 pilot is deferred.
7. **Country-agnostic core.** Country and local currency are **registry data** (ISO 3166-1, ISO 4217), never code. Nothing in the contract, SDK, indexer, or app may hard-code a country, currency, anchor, or locale. A second market must need only registry data and operations, not a rewrite.

## 2. System overview

```
                      ┌──────────────────────────┐
                      │       kinlock-app        │
                      │ Next.js: payee, sender,  │
                      │ attester views, receipts │
                      └───┬─────────────┬────────┘
   wallet-signed txs;     │             │ REST (lists only)
   chain reads for        │             ▼
   money-moving pages     │    ┌─────────────────────┐
                          │    │ kinlock-sdk          │
                          │    │ packages/sdk         │
                          │    │ services/indexer+API │
                          │    └──────────┬──────────┘
                          ▼               │ getEvents
        ┌─────────────────────────────────▼────────┐
        │              Stellar network              │
        │  Soroban RPC · kinlock contract · USDC SAC │
        └───────────────────────────────────────────┘
                          ▲
                          │ meta_hash must match
                 ┌────────┴─────────┐
                 │ kinlock-registry │  public payee JSON, PR-reviewed
                 └──────────────────┘

 Deferred until the M0 anchor spike passes:  kinlock-ramp  (anchor sessions, path payments)
```

Dependency direction: `kinlock-contracts` → generated TS bindings → `kinlock-sdk` → `kinlock-app`. `kinlock-registry` is data only.

## 3. Tech stack

| Layer | Choice | Notes |
|---|---|---|
| Contract | Rust, `soroban-sdk`, Stellar CLI | One contract, modular code (`registry`, `vault` modules) |
| Contract testing | `cargo test`, `proptest` | Property tests required for fund invariants |
| Cost assertions | `soroban-budget-assert` in CI | Plus periodic testnet simulation to compare estimates |
| Token | USDC via SAC (SEP-41), allowlisted | 7 decimals; amounts `i128` |
| Client libs | `@stellar/stellar-sdk`; generated bindings via `stellar contract bindings typescript` | Published from `kinlock-contracts` |
| Frontend | Next.js (App Router), TypeScript, Tailwind | Mobile-first. **i18n-ready:** all UI strings externalized from the start; `Intl` APIs for numbers, currencies, dates. Full translations deferred |
| Display FX | `RateProvider` interface in the app; provider decided in M0 | **Display only.** Falls back to USD-only when no reliable rate exists. Never used by the contract or in any money logic |
| Wallets | Freighter and others via a wallet-kit abstraction | Passkey smart accounts deferred |
| Indexer/API | Node.js, TypeScript, Fastify, Zod | Long-running process (not serverless) |
| Database | PostgreSQL with **Drizzle** | Decided; stop deliberating |
| Scheduling | In-process interval loops; revisit only if needed | **No Redis/BullMQ** |
| Observability | Structured logs (pino) + `/health` exposing indexer lag | No OpenTelemetry at MVP |
| CI/CD | GitHub Actions | |
| Local dev | Stellar quickstart container + `docker compose` (Postgres) | One-command bootstrap |
| Hosting | App on edge/static host; indexer as one container; managed Postgres | |
| RPC | At least two providers configured, with failover | |

**Deferred stack items:** Redis/queue, OpenTelemetry, SEP-12/38/45, passkey accounts, fee-bump relayer, object storage, ramp service.

## 4. On-chain design

One contract, `kinlock`, with `registry` and `vault` modules in separate files for clean audit and testing.

### 4.1 Constants

| Constant | Suggested value | Note |
|---|---|---|
| `MAX_TRANCHES` | 12 | Termly fees = 3, monthly rent = up to 12 |
| `MIN_AMOUNT` | 1 USDC in base units | Dust guard |
| `MIN_EXPIRY_AHEAD` | 1 hour | Prevents instant-expiry locks |
| `MAX_LOCK_DURATION` | 180 days | **Must stay below the network's max entry TTL.** Verify current network settings |
| `SUSPENSION_REFUND_GRACE` | 14 days | Sender early-refund after payee suspension |
| `TTL_GRACE` | 30 days | Added beyond expiry when extending TTL |

### 4.2 Data model

```rust
// Instance storage
struct Config {
    admin: Address,              // multisig
    next_lock_id: u64,
    paused_new_locks: bool,
    max_lock_amount: i128,       // P1: mainnet caps
    max_total_locked: i128,      // P1
    total_locked: i128,          // sum of remainders of Open locks
}

// Persistent storage
enum DataKey {
    Attester(Address),           // -> bool
    Token(Address),              // -> bool (allowlist; gates creation only)
    Payee(BytesN<32>),           // payee_id -> Payee
    Lock(u64),                   // lock_id  -> Lock
}

// Append-only: add new variants at the end only
enum Category { School, Rent }
enum PayeeStatus { Active, Suspended, Revoked }
enum LockState { Open, Completed, Refunded, Declined }

struct Payee {
    payout: Address,             // G or C address; applies to NEW locks only
    category: Category,
    status: PayeeStatus,
    status_changed_at: u64,
    attester: Address,           // vouching attester
    meta_hash: BytesN<32>,       // hash of canonical registry JSON
    registered_at: u64,
}

struct Tranche { amount: i128, unlock_at: u64, released: bool }

struct Lock {
    id: u64,
    sender: Address,
    payee_id: BytesN<32>,
    payout: Address,             // SNAPSHOT of payee.payout at creation, immutable
    token: Address,
    total: i128,
    released: i128,
    returned: i128,              // refunded or declined
    ref_hash: BytesN<32>,        // sha256(reference || salt)
    tranches: Vec<Tranche>,      // 1..=MAX_TRANCHES, non-decreasing unlock_at
    expires_at: u64,
    state: LockState,
    created_at: u64,
}
```

`payee_id` is a SHA-256 of the registry slug. Purpose is the payee's category; there is no separate purpose field.

### 4.3 Entry points

**Admin (multisig)**

| Function | Behavior |
|---|---|
| `init(admin)` | One-time setup |
| `add_attester` / `remove_attester` | Roster management |
| `add_token` / `remove_token` | Allowlist; removal only blocks *new* locks |
| `set_paused_new_locks(bool)` | Blocks `create_lock` only |
| `set_caps(max_lock, max_total)` | Mainnet gating (P1) |
| `upgrade(wasm_hash)` | Multisig now; timelock before mainnet |

**Registry module**

| Function | Auth | Behavior |
|---|---|---|
| `register_payee(payee_id, payout, category, meta_hash)` | attester | Creates Active payee |
| `set_status(payee_id, status)` | vouching attester or admin | Active ⇄ Suspended; Revoked is terminal. Updates `status_changed_at` |
| `update_payout(payee_id, new_payout)` | vouching attester | **Affects new locks only.** Emits event |

**Vault module**

| Function | Auth | Behavior |
|---|---|---|
| `create_lock(token, payee_id, tranches, ref_hash, expires_at)` | sender | Validates (see below), snapshots payout, pulls `total` from sender, stores lock, extends TTL, emits event |
| `release(lock_id, idx)` | `lock.payout` | Requires state Open, payee Active, `unlock_at ≤ now < expires_at`, tranche not released. Sets flags **before** transferring |
| `refund(lock_id)` | sender | Requires Open and (`now ≥ expires_at` or payee Revoked or (payee Suspended and `now ≥ status_changed_at + GRACE`)). Returns remainder |
| `decline(lock_id)` | `lock.payout` | Any time while Open. Returns remainder to sender |
| `bump_lock(lock_id)` | none | Permissionless TTL extension for Open locks |
| `get_lock`, `get_payee` | none | Reads |

**`create_lock` validations:** contract not paused for new locks; token allowlisted; payee Active; `total ≥ MIN_AMOUNT` and ≤ `max_lock_amount`; tranche count 1..=12; tranche amounts sum to `total`; each `unlock_at ≤ expires_at` and non-decreasing; `expires_at` between `now + MIN_EXPIRY_AHEAD` and `now + MAX_LOCK_DURATION`; `sender != payout`; `total_locked + total ≤ max_total_locked`.

### 4.4 Events

All include `schema_version`. The indexer must handle multiple versions after upgrades.

`PayeeRegistered`, `PayeeStatusChanged`, `PayoutUpdated`, `LockCreated {id, sender, payee_id, payout, token, total, ref_hash, expires_at, tranche_count}`, `Released {id, idx, amount, payout}`, `Refunded {id, amount, reason: Expired | Revoked | SuspendedTimeout}`, `Declined {id, amount}`.

### 4.5 State machine

```
                create_lock
                     │
                     ▼
                 ┌──────┐  release (last tranche)  ┌───────────┐
                 │ Open │─────────────────────────►│ Completed │
                 └──┬───┘                          └───────────┘
   refund           │ decline (payee)
 (expired/revoked/  ├────────────────────────────►  Declined
  suspended+grace)  │
                    └────────────────────────────►  Refunded
```

Partial releases keep the lock Open. Refund or decline after partial release returns only the remainder.

### 4.6 Invariants (property-tested)

1. `released + returned ≤ total`; when terminal, `released + returned == total`.
2. Tranche amounts sum to `total`; every `unlock_at ≤ expires_at`.
3. Funds leave the contract **only** to `lock.payout` (snapshot) or `lock.sender`. No fee path exists.
4. Each tranche is released at most once.
5. Release succeeds only when `unlock_at ≤ now < expires_at`, payee Active, and caller is `lock.payout`.
6. Refund succeeds only for: expiry, payee Revoked, or payee Suspended beyond grace.
7. `lock.payout` never changes after creation.
8. Admin, attester, pause, and allowlist changes never block release, decline, or refund on existing locks.
9. `total_locked` equals the sum of remainders of Open locks.
10. A tranche transfer that fails reverts the whole call, leaving state unchanged.

### 4.7 Implementation rules

- Write state before token transfers.
- `require_auth` on every privileged action; support C-address accounts for senders and payees (test it).
- No unbounded storage and no per-user on-chain lists. Enumeration comes from events.
- Extend persistent TTL on create to `expires_at + TTL_GRACE`; extend completed locks for a receipt-retention period if the network allows; keep `bump_lock` permissionless.
- Time uses ledger timestamp; treat it as approximate to the ledger close interval.
- Contract storage is versioned. Upgrades require a migration test against a snapshot of testnet state.
- Budgets per entry point enforced with `soroban-budget-assert`.

## 5. Off-chain design

### 5.1 `kinlock-sdk` (package + indexer)

**Package:** typed client over generated bindings: `createLock`, `release`, `refund`, `decline`, `getLock`, `verifyReceipt`, `buildClaimLink`, `parseClaimLink`, `preflight`.

**Preflight checks (shared by app and third parties):** sender balance; payee Active; payee `PayoutUpdated` in the last 7 days; existing locks with the same `ref_hash` for the same payee; payout account exists with an **authorized USDC trustline** and sufficient limit.

**Indexer service:**
- Polls Soroban RPC `getEvents` from a persisted cursor.
- RPC history is limited, so the indexer runs continuously, checkpoints each batch, and alerts on lag. Gap recovery needs an archive/backfill source (open question).
- Idempotent on `(tx_hash, event_index)`; stores `schema_version`.
- Read API is **for lists and dashboards only**. Money-moving pages read the chain directly.

| Endpoint | Description |
|---|---|
| `GET /locks?sender=&payee_id=&state=` | Lists |
| `GET /locks/:id` | Detail (display convenience; the app re-reads chain before actions) |
| `GET /payees?category=&country=&q=` | Verified payees, joined with registry repo data; filterable by country |
| `GET /health` | Includes indexer lag |

### 5.2 `kinlock-app`

- Routes: `/` landing, `/request` payee creates a payment request, `/send` sender flow, `/locks/[id]`, `/claim/[id]` (payee), `/payee` dashboard, `/attester` checklist tooling, `/r/[txHash]/[eventIndex]` receipt, `/verify`.
- Transaction flow: build with SDK → simulate → wallet signs → submit → poll → optimistic UI until the indexer catches up.
- **Claim link:** `/claim/{id}#r={reference}&s={salt}`. The fragment is never sent to a server. The payee's browser recomputes `sha256(reference || salt)` and compares with the on-chain `ref_hash`. The salt prevents outsiders from guessing low-entropy references (e.g. student IDs) from the public hash.
- **Payment-request link:** `/send?payee={payee_id}&amount=…&ref=…&schedule=…` pre-fills the sender form. It carries no authority; the contract still validates everything.
- **Local-currency display:** the payee's `local_currency` (from registry data) selects the indicative equivalent via `RateProvider`. Labeled **indicative**; if no reliable rate exists, show USD only with a note. Formatting uses `Intl` with the viewer's locale. Amounts on-chain are always USDC.
- **Strings:** every user-visible string lives in message files (for example `messages/en.json`), never inline. Adding a language later means adding a file, not editing components.
- Server surface is minimal: static pages plus read calls to the indexer. No user database in MVP.
- Claim links are saved in the sender's browser storage with an export option. If lost, the sender must re-share from their records (see E21).

### 5.3 `kinlock-registry`

```
supported-countries.json  # ISO 3166-1 alpha-2 list; changed only with counsel's sign-off
attesters/<handle>.json   # attester identity (public) and the countries they may vouch for
payees/<country>/<slug>.json   # public profile, grouped by country (e.g. payees/ng/…, payees/ph/…)
schemas/payee.schema.json
scripts/hash.sh|ts        # canonical hash
.github/workflows/validate.yml
```

Public fields only: `slug`, `display_name`, `category`, `country` (ISO 3166-1 alpha-2), `local_currency` (ISO 4217), `city`, `payout_address`, `attester`, `verified_at`. Canonicalization is simple and documented (sorted keys, compact, UTF-8, SHA-256), not a full RFC 8785 implementation. CI checks schema validity and, post-registration, that the on-chain `meta_hash` matches the file. It also enforces, **off-chain**:
- the payee's `country` appears in `supported-countries.json`;
- the vouching `attester` is authorized for that `country` in `attesters/<handle>.json`;
- `country` matches the directory the file lives in, and `local_currency` is a valid ISO 4217 code;
- `slug` is globally unique (recommended prefix: the country code).

These are registry and app **policy**, not contract rules. See §8 for the trust implication.

**Attester onboarding checklist** (scripted where possible): payout account exists; USDC trustline present and authorized; small XLM float for fees; test receive and a test release on testnet; payee walked through cash-out options. Evidence of identity stays with the attester, not in any Kinlock system.

### 5.4 `kinlock-ramp` (deferred)

Created only if the M0 spike finds a viable anchor. If created: SEP-1 discovery and `/info` checks, SEP-10 auth, SEP-24 interactive withdrawal, status polling, and a path-payment builder for anchors that don't accept USDC directly (some local-currency anchors list only their own token; one NGN anchor was recently observed doing so). SEP-24 handles KYC in the anchor's own interface, so SEP-12 is not needed. SEP-45 only if contract-account payees are supported.

Until then, cash-out is guidance: some Stellar wallets include SEP-24 anchor access, so payees can use their own wallet, and in some markets payees can simply hold USDC. Anchors and wallet routes differ **per country and currency**, so the M0 spike runs per candidate market, and any ramp adapter is per anchor, never per country assumption.

## 6. Off-chain data model (PostgreSQL)

Owned by the indexer. Amounts use `NUMERIC(39,0)` to hold `i128`.

```sql
CREATE TABLE chain_events (
  id             BIGSERIAL PRIMARY KEY,
  event_type     TEXT NOT NULL,
  schema_version INT  NOT NULL,
  ledger         BIGINT NOT NULL,
  ledger_time    TIMESTAMPTZ NOT NULL,
  tx_hash        TEXT NOT NULL,
  event_index    INT  NOT NULL,
  payload        JSONB NOT NULL,
  UNIQUE (tx_hash, event_index)
);

CREATE TABLE indexer_cursor (
  id             INT PRIMARY KEY DEFAULT 1,
  last_ledger    BIGINT NOT NULL,
  last_event_id  TEXT,
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE locks (
  id          BIGINT PRIMARY KEY,
  sender      TEXT NOT NULL,
  payee_id    TEXT NOT NULL,
  payout      TEXT NOT NULL,                 -- snapshot
  token       TEXT NOT NULL,
  total       NUMERIC(39,0) NOT NULL,
  released    NUMERIC(39,0) NOT NULL DEFAULT 0,
  returned    NUMERIC(39,0) NOT NULL DEFAULT 0,
  ref_hash    TEXT NOT NULL,
  state       TEXT NOT NULL,
  end_reason  TEXT,                          -- Expired | Revoked | SuspendedTimeout | Declined
  expires_at  TIMESTAMPTZ NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL,
  created_tx  TEXT NOT NULL
);
CREATE INDEX ON locks (sender);
CREATE INDEX ON locks (payee_id, state);
CREATE INDEX ON locks (payee_id, ref_hash);   -- duplicate-reference preflight

CREATE TABLE tranches (
  lock_id    BIGINT REFERENCES locks(id),
  idx        INT NOT NULL,
  amount     NUMERIC(39,0) NOT NULL,
  unlock_at  TIMESTAMPTZ NOT NULL,
  released   BOOLEAN NOT NULL DEFAULT false,
  release_tx TEXT,
  PRIMARY KEY (lock_id, idx)
);

CREATE TABLE payees (
  payee_id           TEXT PRIMARY KEY,
  slug               TEXT UNIQUE NOT NULL,
  category           TEXT NOT NULL,
  status             TEXT NOT NULL,
  status_changed_at  TIMESTAMPTZ NOT NULL,
  payout             TEXT NOT NULL,
  payout_updated_at  TIMESTAMPTZ,
  attester           TEXT NOT NULL,
  meta_hash          TEXT NOT NULL,
  display_name       TEXT,                   -- from registry repo
  country            TEXT,                   -- ISO 3166-1 alpha-2, from registry repo; filter/display only
  local_currency     TEXT,                   -- ISO 4217, from registry repo; display only
  city               TEXT,
  registered_at      TIMESTAMPTZ NOT NULL
);
```

**Deferred tables:** `subscriptions` (P1 notifications), anchors and ramp sessions (with `kinlock-ramp`). There is no receipts table, no private reference store, no attestation or evidence tables, and no user table in MVP.

## 7. Key flows

### 7.1 Payee prepares a request
1. Attester has completed the onboarding checklist and registered the payee.
2. Payee enters amount, reference, schedule on `/request`; the app generates a payment-request link.

### 7.2 Create and release
1. Sender opens the request link. App loads the payee, runs `preflight`, shows USD amount, indicative equivalent in the payee's local currency (or USD only if no reliable rate), and rate-risk note.
2. App generates a random `salt`, computes `ref_hash`, builds `create_lock`, simulates, wallet signs, submits.
3. Contract validates, snapshots payout, pulls funds, emits `LockCreated`. App builds the claim link with reference and salt in the fragment and prompts the sender to save it.
4. Payee opens the claim link. App **reads the lock from chain**, verifies the reference hash, and shows tranches.
5. Payee signs `release(lock_id, idx)`. On success the receipt page is available immediately.
6. If release fails because of a missing or unauthorized trustline, the whole call reverts, funds stay locked, and the payee fixes the account and retries. If never fixed, the sender refunds after expiry.

### 7.3 Receipt verification (tiers)
- **Tier 1, live:** fetch the transaction and events through RPC while within the RPC retention window; rebuild the receipt; compare.
- **Tier 2, state:** while the lock entry is live, read `get_lock` to confirm the tranche's `released` flag and amounts. Confirms payment happened but not the transaction details.
- **Tier 3, archive:** older transactions need a history/archive provider. **Decision required before mainnet** (open question §14).
- The DB is a lookup convenience, never proof. Receipt copy says "Payment to verified payee", never "service delivered".

### 7.4 Registration and revocation
1. Attester verifies the institution and completes the checklist.
2. PR to `kinlock-registry`; CI validates; merge.
3. Attester calls `register_payee` with the file hash; registry CI confirms the match.
4. Revocation: attester or admin sets status Revoked. Open locks to that payee become immediately refundable by senders.

## 8. Security model

| Threat | Control |
|---|---|
| Admin or operator steals locked funds | No code path to any destination except `lock.payout` or `lock.sender`; invariants property-tested; audit |
| **Attester registers a fraudulent payee** | **Accepted trust assumption.** Mitigations: named attester shown on every payee; revocation → immediate sender refund; P1 caps; P1 new-payee release delay; monitoring for anomalies. This risk affects *future* locks, not existing ones |
| Attester or payee key compromise redirecting existing locks | Not possible: payout is snapshotted per lock. A compromised *payee* key can claim that payee's own locks, which is the payee's responsibility |
| Payee loses key | Locks unclaimable until expiry, then sender refunds and re-locks to the updated payout |
| Look-alike assets | Token allowlist in contract; UI shows code and issuer |
| Replay / double release | Per-tranche `released` flag set before transfer |
| Reference leakage | Only salted hash on-chain; reference and salt in URL fragment; anyone holding the link learns the reference but cannot move funds |
| Indexer lies or lags | Money-moving pages read chain directly |
| Upgrade abuse | Multisig; timelock before mainnet; upgrade events monitored; trust assumption documented |
| Issuer-level controls (freeze, authorization changes) on USDC | Documented dependency on the issuer; pause new locks if the asset becomes unsafe; existing locks unaffected by Kinlock admin |
| Spam locks to a payee | Minimum amount; dashboard filters |
| **Payee in an unsupported or sanctioned country, or attester vouching outside their country** | Registry CI rejects both; app blocks selection; sender-side restrictions per counsel. **Not enforceable on-chain:** the contract has no notion of country, and anyone can call it directly. This is an accepted trust assumption: country policy governs what Kinlock *lists and verifies*, not what the protocol permits |

Mainnet gating: external audit → legal review → capped beta (per-lock and global caps) → raise caps gradually.

## 9. Testing strategy

- **Unit tests** for every entry point: success and each failure path (including each `create_lock` validation).
- **Property tests (`proptest`)**: random sequences of create, release, refund, decline, status changes, and time advances against the §4.6 invariants.
- **Budget assertions** with `soroban-budget-assert`; compare local estimates against testnet simulation.
- **Integration tests** on the local quickstart network through the SDK, including C-address sender and payee, missing trustline, and boundary timestamps (`now == expires_at`).
- **Indexer tests:** replay recorded event fixtures; idempotency; cursor recovery; mixed `schema_version`.
- **Upgrade test:** migrate a snapshot of testnet state to a new WASM and re-run invariants.
- **One E2E** (Playwright): request → send → claim → verify on testnet.
- **Deferred:** anchor drift checks, broader E2E suites.

## 10. Environments and deployment

| Env | Network | Purpose |
|---|---|---|
| local | Stellar quickstart | Dev and CI integration |
| testnet | Stellar testnet | Public pilot with test assets |
| mainnet | Stellar mainnet | After audit and legal review; capped |

- Deployments recorded in `DEPLOYMENTS.md` (contract ID, WASM hash, admin multisig, config).
- Indexer is one stateless container; Postgres backed up daily.
- Feature flags only where needed (e.g. ramp availability once built).
- Delivery status is tracked in `ROADMAP.md` at each repo's root. **Every PR updates it** (statuses, new rows, Changelog entry); CI enforces this with `roadmap-check`. See `AGENTS.md` §12.

## 11. Repository structure

```
kinlock-contracts/
  contracts/kinlock/src/{lib.rs, types.rs, registry.rs, vault.rs, events.rs}
  tests/ (integration, proptest)   scripts/ (deploy, bindings)
  DEPLOYMENTS.md   SECURITY.md

kinlock-sdk/
  packages/sdk/    services/indexer/

kinlock-app/
  app/   components/   lib/ (sdk, wallet, claim-links, rates, i18n)   messages/   e2e/

kinlock-registry/
  payees/   schemas/   scripts/   .github/workflows/

kinlock-ramp/            # conditional, created only after M0 passes
```

## 12. Architecture Decision Records

| ADR | Decision | Reason | Trade-off |
|---|---|---|---|
| 1 | Non-custodial; contract holds funds | Core trust promise | Payees need an address; onboarding burden |
| 2 | **One contract**, modular code | Single upgrade authority makes a split pointless; fewer cross-contract calls and smaller audit | Coarser upgrade granularity |
| 3 | No on-chain enumeration; events + indexer | Bounded storage | Needs a reliable indexer |
| 4 | Receipts verified from chain data in tiers | Database cannot forge proof | Retention limits; archive decision pending |
| 5 | Salted reference hash; reference and salt in URL fragment | Keeps references off-chain and off servers | Lost link = no reference shown |
| 6 | **Payout snapshotted per lock** | Eliminates redirect attacks and the timelock machinery | Payee key loss strands locks until expiry |
| 7 | Attester can update payout for **new** locks only | Allows rotation without touching existing locks | Attester is trusted for future locks |
| 8 | Pause affects only new locks | Admin cannot trap funds | No emergency freeze of claims; rely on caps and audit |
| 9 | **No protocol fee in MVP** | Smaller audit surface; revenue model unproven | Revenue deferred to a later contract version |
| 10 | Upgradeable via multisig; timelock before mainnet | Fixes in beta | Trust in multisig |
| 11 | Postgres mirror for lists; chain reads for actions | Fast UI without trusting the DB | Two read paths to maintain |
| 12 | **Payee `decline`** instead of mutual cancel | One signer, always safe (only returns to sender) | Sender must ask the payee |
| 13 | Public registry bound by `meta_hash` | Transparent, contributor-friendly | Public data must exclude personal information |
| 14 | Deterministic receipt URL; no receipts table | Chain is public; slugs added nothing | None meaningful |
| 15 | Refund on payee Revoked, and after Suspended + grace | Prevents stuck funds | Attester could revoke maliciously (they are trusted) |
| 16 | Strict boundary: release if `now < expires_at`, refund if `now ≥ expires_at` | No overlap window | Payee must act before expiry |

## 13. Hard questions review

IDs are shared with `PRD.md` §9. B = what breaks, E = missing edge cases, O = overengineered.

### 13.1 What would break?

| ID | Failure | L / I | Resolution |
|---|---|---|---|
| B1 | Payees can't operate a wallet (account, reserve, trustline, key safety) | High / High | Assisted onboarding checklist (REG-6); M0 validation; pivot trigger. Not a technical fix |
| B2 | Sender all-in cost and friction lose to paying the institution's bank account directly | High / High | M0 cost model; differentiate on receipts and installments; pivot trigger |
| B3 | No viable local-currency off-ramp **in a given market** (varies per country) | High / High | M0 spike per candidate market; ramp deferred; payee cashes out via own wallet or holds USDC |
| B4 | Cold start; attester fraud power | Med-High / High | Single-community pilot; named attesters; revocation → refund; P1 delay and caps |
| B5 | `release` fails: missing, full, or unauthorized trustline; unfunded account; issuer controls | High / Medium | Preflight and onboarding checks; failure reverts atomically; retry; refund at expiry |
| B6 | **Hollow timelock (v0.1 bug)** | Certain / High | **Fixed:** payout snapshot (ADR-6) |
| B7 | **Reference delivery gap (v0.1 bug)** | Certain / High | **Fixed:** fragment-carried reference and salt |
| B8 | **Verification beyond RPC retention (v0.1 overclaim)** | Certain / Medium | Tiered verification (§7.3); archive decision before mainnet |
| B9 | Indexer lag or outage shows stale state at claim time | Medium / Medium | Money pages read chain; lag alert; two RPC providers |
| B10 | State archival: entry TTL ends before lock expiry, or desired duration exceeds network max TTL | Medium / High | Duration cap below max TTL; extend on create; permissionless `bump_lock`; documented restore procedure |
| B11 | Overclaim of "proof of use" | Certain / Medium | Reword; optional payee acknowledgment (P1) |
| B12 | **`SenderApproval` let senders block payees (v0.1 contradiction)** | Certain / High | **Removed** |
| B13 | Suspended payee strands funds until a long expiry | Medium / Medium | Suspended + grace refund; max duration |
| B14 | FX mismatch between local-currency invoices and USDC locks (worse for volatile currencies) | High / Medium | Disclosure; USD pricing in the request; indexed tranches deferred |
| B15 | Legal and regulatory exposure, **per market** | Unknown / Severe | Counsel per market; testnet pilot first |
| B17 | Upgrade mid-lock breaks stored data or indexer events | Medium / High | Storage versioning, `schema_version`, migration test on testnet snapshot |
| B18 | Failed sponsored transactions still cost fees; sponsor account drained | Low (deferred) | Sponsorship deferred; if added, simulate first and allowlist operations |
| B19 | "Worldwide" multiplies work per market; geography can't be enforced on-chain | High / High | Market-by-market playbook (`PRD.md` J8); supported-countries list in registry CI; ≥ 2 pilot markets; stated trust assumption (§8) |
| B20 | Country-specific assumptions leak into code, so the second market needs a rewrite | Medium / High | Principle #7; CI guard against hard-coded country/currency literals outside registry data and tests; second pilot market is a deliberate test |

**Pre-mortem: the five most likely causes of failure, in order.** B1 payee wallet friction, B2 sender economics, B3 no off-ramp in a market, B4 attester supply and trust, B15 legal per market. B19 (worldwide scope creep) is the sixth to watch. None is a smart-contract bug. The contract is the easiest part, so don't over-invest there before M0 answers these.

### 13.2 What edge cases are we missing?

**Funds and tokens**

| ID | Case | Decision |
|---|---|---|
| E1 | Payee trustline missing, limit reached, or unauthorized | Preflight; atomic revert; retry |
| E2 | Issuer freezes or revokes authorization or claws back | Documented issuer dependency; pause new locks if unsafe |
| E3 | Token removed from allowlist while locks are open | Allowlist gates creation only; release and refund still work |
| E4 | Dust and rounding when splitting into tranches | Min amount; UI puts remainder in the last tranche |
| E5 | Sender can't receive refund (lost key, no trustline) | Funds stay locked; consider optional `refund_to` address later (P2) |
| E6 | Duplicate or overpaid lock | Duplicate warning; payee `decline` |
| E7 | Wrong payee selected | Show attester, city, address; `decline`; expiry refund |
| E18 | Sender equals payout (self-dealing) | Rejected at creation |

**Time**

| ID | Case | Decision |
|---|---|---|
| E8 | `unlock_at` after `expires_at` | Rejected at creation |
| E9 | Action exactly at `expires_at` | Release only if `now < expires_at`; refund only if `now ≥ expires_at` |
| E10 | Ledger timestamp is approximate | No minute-level guarantees; UI shows UTC and local time |
| E11 | TTL or archival before expiry | See B10 |
| E12 | Very short or very long expiry | Min 1 hour ahead; max 180 days |

**Roles and keys**

| ID | Case | Decision |
|---|---|---|
| E13 | Payee loses key | Locks strand until expiry; sender refunds; attester updates payout for *new* locks |
| E14 | Payee suspended or revoked mid-lock | Release blocked; refund path per ADR-15 |
| E15 | Attester key compromised or attester leaves | Admin removes attester; runbook to re-attest or suspend their payees |
| E16 | Admin multisig keys lost | Existing locks keep working; can't add attesters or tokens or upgrade. Acceptable and documented |
| E17 | C-address sender or payee | Supported through `require_auth`; integration-tested |

**Privacy and links**

| ID | Case | Decision |
|---|---|---|
| E19 | Sender↔payee linkage is public | Health deferred; advise a dedicated sender wallet; minimal on-chain data |
| E20 | Claim link forwarded | Holder learns the reference but cannot move funds; warn in UI |
| E21 | Claim link lost | Payee dashboard still lists the lock; sender re-shares from saved link; no server copy by design |

**Operations**

| ID | Case | Decision |
|---|---|---|
| E22 | Indexer gap beyond RPC retention, duplicate events, event schema change | Idempotent writes, `schema_version`, backfill source decision |
| E23 | Anchor minimum exceeds a small tranche | Surface minimums in guidance |
| E25 | Illicit use or stolen-card funding of locks | Attester screening, caps, counsel; on-chain releases can't be reversed |
| E31 | Single RPC provider outage | Two providers with failover |

**Worldwide scope**

| ID | Case | Decision |
|---|---|---|
| E32 | Payee in an unsupported or sanctioned country | Registry CI rejects; app blocks selection; contract can't enforce (accepted) |
| E33 | No reliable rate for a payee's currency | USD only with a note; never hard-code or invent a rate |
| E34 | Currencies differ in decimals, symbols, number formats | `Intl` and ISO 4217 for **display only**; on-chain amounts are USDC only |
| E35 | Volatile or high-inflation currency | Stronger disclosure; USD pricing in request; shorter schedules encouraged |
| E36 | Capital controls restrict payees receiving stablecoins or foreign currency | Counsel per market; attester checklist confirms legal ability to receive |
| E37 | Non-Latin payee names | UTF-8 `display_name`; ASCII slug; disambiguate by city, attester, address |
| E38 | Sender and payee in the same country, or sender in a restricted country | Domestic allowed; sender restrictions only as counsel directs |
| E39 | School calendars, rent conventions, time zones differ | Free-form tranche schedule; times in UTC and viewer's local time |
| E40 | Attester vouches for a payee outside their authorized countries | Registry CI rejects; named attester shown on every payee |

### 13.3 What is overengineered?

| ID | Item | Verdict | Why |
|---|---|---|---|
| O1 | Two contracts | **Cut** → one | One upgrade authority; extra cross-contract calls and audit scope |
| O2 | Three release modes | **Cut** `SenderApproval`; `AttesterApproval` → P2 | Contradicts promise / no demand evidence |
| O3 | Fee machinery | **Cut** | Revenue model unproven; adds audit surface |
| O4 | Payout timelock and approvals | **Cut** | Replaced by snapshot; the timelock was hollow |
| O5 | Mutual cancel | **Cut** → `decline` | Two-signer coordination is poor UX |
| O6 | Purpose enum | **Cut** | Derivable from category |
| O7 | Health, Utility, Other | **Cut** | Privacy and scope |
| O8 | Receipts table, slugs, payload hash, PDF/QR | **Cut/defer** | Deterministic URL suffices |
| O9 | Encrypted private store, users, notifications, attestations, object storage | **Cut/defer** | Fragment-carried reference; attesters keep evidence |
| O10 | Redis + BullMQ | **Cut** | Postgres and in-process loops suffice |
| O11 | OpenTelemetry | **Cut** | Logs and a lag metric suffice |
| O12 | Separate ramp service at MVP | **Defer** | Unvalidated; gated by M0 |
| O13 | SEP-12, SEP-38, SEP-45 | **Defer** | SEP-24 handles KYC in the anchor's UI |
| O14 | Passkey accounts, relayer, fee-bump | **Defer** | Fund payees with a small XLM float at onboarding |
| O15 | RFC 8785 canonicalization | **Simplify** | Documented sorted-key compact JSON is enough |
| O16 | Upgrade timelock on testnet | **Defer** | Add at mainnet gate |
| O17 | Large E2E suite, anchor drift CI | **Defer** | One happy-path E2E for now |
| O19 | On-chain country enforcement, geofencing in the contract, per-country compliance logic | **Cut** | Country is registry data and app policy; the contract stays country-agnostic |
| O20 | Multi-token support beyond USDC | **Defer** | Allowlist already permits it later without contract changes |
| O21 | Full translation framework and many languages at MVP | **Defer** | Externalized strings and `Intl` now; translations later |

**Keep, despite temptation to cut:** property tests (funds), budget assertions (cheap, already built by the team), registry repo with hash binding (gives contributors safe work), tranches (School termly fees need them), refund on revocation, chain-first reads, caps for mainnet.

## 14. Open technical questions

1. Archive source for receipt verification beyond RPC retention (Tier 3), and for indexer backfill.
2. Network's current max entry TTL: confirms `MAX_LOCK_DURATION`.
3. Whether to ship optional `refund_to` (E5).
4. Which wallets and anchors give payees a workable local-currency cash-out in each pilot market (M0 spike, per market).
5. Whether `LCK-10` payee acknowledgment is an event only or also surfaced on receipts.
6. New-payee release delay parameters (P1): delay length, payee age threshold.
7. Which pilot markets, and the supported-countries policy (who decides, criteria, sanctions handling).
8. Rate provider for indicative local-currency display across many currencies, and its fallback behavior.
9. Whether attester-country scope should also be enforced on-chain, or stay registry-CI only (current plan).
