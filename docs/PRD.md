> Synced from Kinlock-Org/.github. Do not edit here.

# Kinlock — Product Requirements Document

> Purpose-locked transfers on Stellar. Money that can only be released to the institution it was meant for.

| | |
|---|---|
| **Status** | Draft v0.3 (worldwide scope) |
| **Date** | 2026-10-05 |
| **Network** | Stellar (Soroban), testnet first |
| **Primary asset** | USDC on Stellar (SEP-41 via Stellar Asset Contract) |
| **Markets** | **Worldwide by design:** any sender → verified schools and landlords in any **supported country**. Launched market by market; pilot markets chosen at M0 |
| **Companion docs** | `ARCHITECTURE.md`, `ARCHITECTURE_ESSENTIALS.md` |

### What changed in v0.3 (worldwide scope)
- **Product is no longer "abroad → Nigeria".** Any sender, anywhere, can lock USDC for a verified school or landlord anywhere in a **supported country**. Domestic payments (sender and payee in the same country) are in scope.
- **Worldwide by design, launched market by market.** The core stays country-agnostic: no country, currency, or anchor is hard-coded in the contract, SDK, indexer, or app. Country and local currency are **registry data** (ISO 3166-1 and ISO 4217).
- **Supported-countries list** (REG-9), enforced in registry CI and app policy, set with counsel. It is **not** enforced on-chain (§9.1 B19).
- **Naira-specific items generalized:** indicative **local-currency** equivalent (SND-4), per-market cash-out routes (PAY-5), per-market legal review.
- **New requirements:** attester-country scope (REG-10), locale-aware formatting and externalized UI strings (SND-10), sender-side restrictions per counsel (SND-11).
- **New risks and edge cases:** B19, B20, E32 to E39.
- **Pilot must prove it:** at least two markets in different regions and currencies, running the same code with only registry-data differences (§8.1).

### What changed from v0.1 to v0.2
- **Scope cut hard.** MVP categories are **School and Rent only**. Health and Utility are deferred (public-ledger privacy, see §9.2).
- **Claim softened.** Kinlock proves *payment reached a verified payee*, not that the service was delivered. "Proof of use" is renamed (§1, §9.1).
- **SenderApproval release mode removed.** It gave payees no assurance and contradicted the core promise.
- **Mutual cancel replaced by payee `decline`.** One signer, same safety.
- **New P0 features found in review:** payee-generated payment-request links, pre-flight checks, USD/local-currency rate disclosure (originally written for NGN).
- **Validation before build.** M0 now includes customer validation with explicit pivot triggers (§8.2).
- **Deferred:** protocol fee, passkey wallets, fee sponsorship, in-app ramp, PDF/QR receipts, notifications.

---

## 1. Summary

**Kinlock lets anyone, anywhere lock USDC on Stellar (Soroban) for a verified school or landlord anywhere in the world.**

In more detail: a sender locks USDC to a **verified payee** (a school or landlord) for a declared purpose. The funds can only be released to that payee, or returned to the sender if unclaimed or if the payee is revoked. Each release yields a **receipt anyone can verify on-chain**.

Kinlock is non-custodial: a Soroban contract holds locked balances and no Kinlock server holds keys or funds.

**Worldwide by design, launched market by market.** The product has no built-in geography. What differs per country is *registry data and operations*: which attesters can verify payees there, which payout and cash-out routes exist, which local currency to show, and what the law allows. A country becomes a **supported country** only after legal review, at least one willing attester, and a practical cash-out route (or payees able to hold USDC). Until then it is not listed. "Worldwide" therefore means *no technical or product barrier to any country*, not *live everywhere on day one*.

**What Kinlock proves:** a specific amount of USDC reached a payee that a named attester verified, at a specific time, for a specific reference.
**What Kinlock does not prove:** that the school credited the student, or that the landlord applied the rent. Optional payee acknowledgment narrows this gap (LCK-10) but cannot close it.

**MVP in one sentence:** a sender locks USDC to a registered school or landlord, the payee releases it in tranches, and anyone can verify the payment on-chain.

## 2. Problem

People who fund essential costs for someone else, from another city, country, or continent, hit these failure modes:

1. **Diversion.** Money sent "for school fees" to a relative is spent on something else. There's no proof either way.
2. **No trustworthy proof of payment.** Screenshots and fake alerts are easy to forge.
3. **Reconciliation pain for payees.** Institutions need to know which student or unit a payment is for.
4. **Cost and delay** on cross-border payment routes (and in some countries on domestic ones).

**Honest competitive framing.** The diversion problem can already be avoided today by paying the institution's bank account directly through an existing bank or payment app. Kinlock must therefore win on something else: **verifiable receipts, installment-style locks the payee can rely on, and lower all-in cost**. If it can't, it has no reason to exist (§8.2).

## 3. Goals and non-goals

### Goals
- G1. A sender can lock funds to a verified payee in under 2 minutes without understanding Stellar internals.
- G2. A payee can see exactly what a payment is for (reference, amount, schedule) and release it without blockchain expertise.
- G3. Every release and refund yields a public receipt verifiable from chain data.
- G4. **No operator or admin can redirect locked funds.** Once a lock exists, funds can only go to the payout address recorded in that lock or back to the sender. The payee's claim is assured *only* against sender interference before expiry, not against the sender's refund after expiry.
- G5. Payees can convert USDC to their **local currency** through a licensed route available in their country (guidance in MVP, in-app later), or keep USDC where that is practical.
- G6. Contributor-friendly repos with scoped issues for Drips Wave sprints.
- G7. **Country-agnostic core.** No country-, currency-, or anchor-specific logic in the contract, SDK, indexer, or app. Market specifics come from registry data and operations.

### Non-goals (MVP)
- Fiat custody, a Kinlock-run exchange, or a Kinlock token.
- Launching in every country at once. Markets open one at a time (§5 J8).
- On-chain country enforcement or geofencing. Country is registry and app policy, not contract logic.
- Lending, credit, yield.
- General marketplace or open-ended escrow.
- Health and Utility categories (privacy, see §9.2).
- Protocol fees (revenue model is an open question, §13).
- Passkey wallets, sponsored fees, in-app on/off-ramp.
- On-chain disputes or juror voting.
- Native mobile apps.
- Mainnet before an external audit and legal review.

## 4. Users and personas

| Persona | Who | Needs |
|---|---|---|
| **Sender** ("Ada") | Anyone paying an institution in another city, country, or continent: a migrant worker, remote parent, employer, or a sponsor or NGO funding a student. Domestic senders too | Confidence the money reaches the institution; proof; low all-in cost; works on a phone |
| **Payee** ("Mr. Okeke", bursar) | School or landlord/property manager in any supported country | Certainty funds are real; clear reference; a simple way to receive and cash out in their own currency |
| **Beneficiary** ("Mama Tunde") | Family member the payment is *for* | Read-only visibility via the share link. **Has no account**; does not hold or move funds |
| **Attester** | School association, NGO, community org, or Kinlock ops, **with local standing in the countries they vouch for** | A lightweight way to verify a payee and publish that verification, plus a checklist to get the payee operational |
| **Contributor** | Open-source developer in a Wave | Small well-scoped issues, clear repo boundaries |

## 5. Core journeys

### J1. Payee onboarding (assisted)
1. Attester verifies the institution off-chain (evidence stays with the attester; Kinlock stores none). The attester may only vouch for payees in countries they are authorized for (REG-10), and the payee's country must be on the supported list (REG-9).
2. Attester completes the **operational checklist**: payee has a Stellar account, USDC trustline established and authorized, small XLM float for fees, and has been walked through receiving and cashing out.
3. Attester adds the payee's public JSON (including `country` and `local_currency`) to `kinlock-registry` (PR), then registers the payee on-chain with the file hash.
4. Payee shows as **Verified by [attester name]**.

### J2. Payee creates a payment request
1. Payee enters amount, reference (student ID, unit, invoice), and schedule (e.g. three termly tranches).
2. App produces a **payment-request link** pre-filling the sender's form.
3. Payee sends the link to the sender (WhatsApp, email). This avoids wrong amounts, wrong payee, and missing references.

### J3. Sender creates a lock
1. Sender opens the request link (or picks a verified payee manually).
2. App runs **pre-flight checks** (§6.4): balance, payee status, payee payout address changed recently, duplicate reference.
3. App shows the USD amount, an **indicative equivalent in the payee's local currency** (when a reliable rate exists, otherwise USD only), and a rate-risk note.
4. Sender signs and funds the lock. The app produces a **claim link** that contains the reference (so the server never sees it) and tells the sender to save it.

### J4. Payee releases
1. Payee opens the claim link or dashboard. The app verifies the reference against the on-chain hash.
2. Payee releases each tranche once its unlock time has passed and before expiry.
3. USDC arrives at the payee's payout address; the receipt page is available immediately.

### J5. Refund, decline, revocation
- **Expiry:** after expiry, the sender refunds any unreleased remainder.
- **Payee declines:** the payee can return the unreleased remainder to the sender at any time (covers overpayment, wrong payee, duplicate lock).
- **Payee revoked:** the sender can refund immediately.
- **Payee suspended:** the sender can refund after a grace period.

### J6. Cash-out
MVP: payee cashes out through their own wallet's anchor integration (SEP-24 where supported). Kinlock provides guidance and checks. In-app tracked cash-out is gated by the M0 anchor spike.

### J7. Receipt verification
Anyone opens `/r/{tx_hash}/{event_index}`. The page rebuilds the receipt from chain data and shows **Valid / Not valid**, independent of Kinlock's database. For old transactions beyond RPC retention, verification depends on the archive approach chosen before mainnet.

### J8. Opening a new market (operations playbook)
A country joins the supported list only when all of these are done and recorded in `ROADMAP.md`:
1. Counsel's written outcome for that country (stablecoin acceptance by payees, money transmission, sanctions, data protection).
2. At least one willing, named attester with local standing, added to the attester-country scope.
3. A practical cash-out route for the payee's currency, or confirmation that payees can hold USDC.
4. A reliable rate source for the local currency, or a decision to show USD only.
5. Registry directory and supported-countries entry merged; pilot payees onboarded using the attester checklist.

## 6. Functional requirements

Priority: **P0** MVP (testnet pilot) · **P1** before mainnet · **P2** later.

### 6.1 Payee registry
| ID | Requirement | Pri |
|---|---|---|
| REG-1 | Attesters register a payee (category School or Rent, payout address, metadata hash). Country and local currency live in the registry JSON, not on-chain | P0 |
| REG-2 | Payee status Active, Suspended, Revoked; set by the vouching attester or admin | P0 |
| REG-3 | Attester can update a payee's payout address. **Applies to new locks only.** Event emitted; UI warns senders if changed in the last 7 days | P0 |
| REG-4 | Public registry repo: one JSON per payee, schema-validated; CI checks the hash matches on-chain | P0 |
| REG-5 | Attester roster managed by admin multisig | P0 |
| REG-6 | Attester onboarding checklist and script: trustline present and authorized, XLM float, test payment | P0 |
| REG-7 | New-payee release delay (e.g. 48h after lock creation while payee is young) | P1 |
| REG-8 | Payee self-service portal | P2 |
| REG-9 | **Supported-countries list** (`supported-countries.json`, ISO 3166-1 alpha-2). Registry CI rejects payees whose `country` is not listed | P0 |
| REG-10 | **Attester-country scope** (`attesters/<handle>.json` lists allowed countries). Registry CI rejects a payee vouched by an attester not authorized for its country | P0 |
| REG-11 | Payee JSON requires `country` (ISO 3166-1) and `local_currency` (ISO 4217) | P0 |

### 6.2 Locks
| ID | Requirement | Pri |
|---|---|---|
| LCK-1 | Create a lock: allowlisted token; Active payee; min and max amount; ≤ 12 tranches; every `unlock_at` ≤ `expires_at`; expiry within max duration; sender ≠ payout address | P0 |
| LCK-2 | Release a tranche: payee only, after `unlock_at` and **before** `expires_at`, payee Active | P0 |
| LCK-3 | Sender refund of unreleased remainder: after expiry, or payee Revoked, or payee Suspended beyond grace | P0 |
| LCK-4 | Payee `decline`: returns unreleased remainder to sender any time while Open | P0 |
| LCK-5 | Payout address is **snapshotted at creation** and cannot change for that lock | P0 |
| LCK-6 | Repeat a previous lock in one tap (UI only; useful for monthly rent) | P1 |
| LCK-7 | `AttesterApproval` release mode | P2 |
| LCK-8 | Top-up; dispute flag and mediator split | P2 |
| LCK-9 | Local-currency-indexed tranches via oracle | P2 (likely never) |
| LCK-10 | Payee acknowledges "applied to [reference]" (event only) | P1 |

Removed: `SenderApproval` mode, mutual cancel, protocol fee.

### 6.3 Receipts
| ID | Requirement | Pri |
|---|---|---|
| RCP-1 | Each release, refund, and decline produces a receipt page at `/r/{tx_hash}/{event_index}` | P0 |
| RCP-2 | Verification rebuilds the receipt from chain data (live RPC window; archive path before mainnet) | P0 |
| RCP-3 | Wording: "Payment to verified payee". Never "proof of use" or "service delivered" | P0 |
| RCP-4 | QR and PDF receipts | P2 |

### 6.4 Sender experience
| ID | Requirement | Pri |
|---|---|---|
| SND-1 | Connect an existing Stellar wallet | P0 |
| SND-2 | **Pre-flight checks**: sufficient USDC, payee Active, payee payout recently changed, duplicate reference, payout account has authorized USDC trustline | P0 |
| SND-3 | Create from a payee request link or manually | P0 |
| SND-4 | Show USD amount, **indicative equivalent in the payee's `local_currency`** from a rate provider (USD only, with a note, if no reliable rate), and rate-risk disclosure | P0 |
| SND-5 | Dashboard with live status read from chain for the detail page | P0 |
| SND-6 | Save and re-share the claim link (kept in the browser; export option) | P0 |
| SND-7 | Instructions for getting USDC (external anchors/wallets) | P0 |
| SND-8 | Email notifications | P1 |
| SND-9 | In-app on-ramp; passkey wallet | P2 |
| SND-10 | **Locale-aware formatting** (numbers, currencies, dates via `Intl`) and **all UI strings externalized** from the start. Full translations are later | P0 |
| SND-11 | Sender-side restrictions (for example sanctioned jurisdictions) applied in the app as counsel directs | P1 |

### 6.5 Payee experience
| ID | Requirement | Pri |
|---|---|---|
| PAY-1 | Claim page from link; reference verified client-side against the on-chain hash | P0 |
| PAY-2 | Release available tranches; receipt shown | P0 |
| PAY-3 | Dashboard and **payment-request creator** (amount, reference, schedule → link) | P0 |
| PAY-4 | Decline a lock | P0 |
| PAY-5 | Cash-out guidance **per supported country** (wallet + anchor, or holding USDC), minimums and fees noted | P0 |
| PAY-6 | CSV export for reconciliation | P1 |
| PAY-7 | In-app tracked cash-out via anchor | P2 (gated by M0) |

### 6.6 Platform
| ID | Requirement | Pri |
|---|---|---|
| PLT-1 | Admin can pause **new** locks only. Release, refund, decline can never be paused | P0 |
| PLT-2 | TypeScript SDK | P0 |
| PLT-3 | Public read API | P0 |
| PLT-4 | Per-lock and global value caps | P1 (mainnet gate) |
| PLT-5 | Upgrade via admin multisig; timelock added before mainnet | P1 |
| PLT-6 | Permissionless TTL bump for open locks | P0 |

## 7. Non-functional requirements

- **Security:** no operator, admin, or attester function can move locked funds. Attesters are a trusted role (they decide which payees exist), and this is stated plainly. External audit before mainnet.
- **Non-custody:** no Kinlock service holds user keys or funds.
- **Privacy:** no personal data on-chain or in the public registry. The sender-to-payee link is public on a public ledger (§9.2).
- **Reliability:** claim and refund pages read lock state from chain, not from the database. The indexer may lag without breaking correctness.
- **Cost awareness:** entry-point resource budgets enforced in CI.
- **Reach:** mobile-first, low-bandwidth tolerant, plain language. English first, but multi-language ready (strings externalized, locale-aware formatting). Dates shown in UTC and local time.
- **Compliance posture:** fiat conversion only through licensed anchors. Counsel review **per market** before any real-value launch there. This document isn't legal advice.

## 8. Success metrics and pivot triggers

### 8.1 Metrics (testnet pilot targets, to revisit with data)

| Metric | Target |
|---|---|
| Verified payees onboarded and *actually able to receive* | 10 |
| **Pilot markets running end to end** (different regions and currencies, same code, only registry data differs) | ≥ 2 |
| Locks created | 100 |
| Median time from lock to first release | < 48h |
| Locks ending in release (vs refund/decline) | > 80% |
| Receipt verification success | 100% |
| Sender all-in cost vs paying the institution directly through an incumbent app | Measured, with a target set at M0 |
| Sender repeat usage (≥ 2 locks) | > 40% |

### 8.2 Pivot triggers (evaluated at the M0 gate)

| Trigger | Response |
|---|---|
| Fewer than 3 of 5 interviewed payees will operate a wallet even with assisted onboarding | Evaluate anchor-managed receiving for payees (payee gets local currency, never touches crypto). Re-scope trust model |
| Sender all-in cost is clearly worse than incumbents and senders don't value receipts or installments | Reposition as a B2B proof-and-installments layer for existing remittance apps instead of consumer-direct |
| No anchor or wallet route offers a workable local-currency payout in a candidate market | Narrow to payees who can hold USDC in that market, or choose other pilot markets |
| No candidate market clears legal review | Hold real-value launch; continue testnet-only work and revisit market selection |
| Fewer than 2 attesters willing to put their name on payee verification | Stop. The trust model has no supply side |

## 9. Hard questions review (product)

Technical items are in `ARCHITECTURE.md` §13. IDs (B, E, O) are shared across files.

### 9.1 What would break?

| ID | What breaks | Likelihood / Impact | Decision |
|---|---|---|---|
| B1 | **Payees can't or won't operate a wallet** (account, reserves, USDC trustline, key safety). The biggest adoption risk | High / High | Assisted onboarding with checklist; validate with 5 real payees in M0; pivot trigger in §8.2 |
| B2 | **Senders compare us to paying the school's bank account directly.** Fiat → USDC → lock → USDC → local currency may cost more than the incumbent | High / High | Cost model and sender interviews in M0; differentiate on receipts and installments; pivot trigger |
| B3 | **No viable local-currency off-ramp in a given market** for USDC at acceptable minimums and fees. Varies per country | High / High | M0 anchor spike per candidate market; MVP doesn't depend on a Kinlock-run ramp; payees may hold USDC |
| B4 | **Three-sided cold start**: no attesters → no payees → no senders. Attesters also hold real fraud power over *future* locks | Medium-High / High | Pilot with one school or landlord association; show attester name on every payee; caps and revocation; P1 new-payee delay |
| B11 | **"Proof of use" overclaim.** Chain proves payment to payee, not delivery | Certain / Medium | Reword (RCP-3); optional payee acknowledgment (LCK-10) |
| B14 | **FX mismatch.** Invoice in a local currency, lock in USDC; the currency moves across tranches (worse in volatile or high-inflation currencies). Payee may be short or senders confused | High / Medium | Show indicative rate and disclose risk (SND-4); payee prices in USD in the request; defer indexed tranches |
| B15 | **Legal, multiplied by every market.** Whether payees in each country can accept stablecoins for domestic services; money-transmission, sanctions, data-protection (including where senders live), and illicit-use exposure | Unknown / Severe | Counsel before any real-value launch; testnet-only pilot until then |
| B16 | **Wave contributors alter security-critical contract code** | Medium / High | Contract changes are maintainer-led and reviewed; open issues focus on UI, docs, tests, and registry |
| B19 | **"Worldwide" multiplies work per market** (legal, attesters, cash-out, rate source). Trying to be everywhere launches nowhere. Also: the contract can't enforce geography, since anyone can call it directly | High / High | Market-by-market playbook (J8); supported-countries list; ≥ 2 pilot markets only; country gating is registry and app policy, stated as a trust assumption |
| B20 | **Country-specific assumptions leak into code** (hard-coded currency, anchor, locale) and the second market needs a rewrite | Medium / High | Country-agnostic core rule (G7); CI guard against hard-coded country/currency literals; second pilot market is a deliberate test |

### 9.2 What edge cases are we missing?

Product-level cases. Technical cases (trustlines, TTL, timing) are in `ARCHITECTURE.md` §13.2.

| ID | Edge case | Decision |
|---|---|---|
| E27 | **Rent is monthly; tranches require full prefunding.** Few senders will lock 12 months upfront | Add one-tap **repeat lock** (LCK-6); treat Rent as recurring single locks, School as multi-tranche |
| E28 | **Payee-initiated flow is better.** Senders mis-enter amounts and references | Payee creates payment-request links (PAY-3) |
| E19 | **Public ledger privacy.** Sender wallet ↔ payee is visible to anyone; with Health this reveals medical payments | Health category deferred; advise a dedicated sender wallet; no extra purpose data on-chain |
| E6 | Duplicate or overpaid locks for the same reference | UI duplicate warning; payee `decline` returns remainder |
| E7 | Wrong payee chosen (two similar schools) | Payee page shows attester, city, address; payee `decline`; expiry refund |
| E21 | Payee lost the claim link, so no reference | Dashboard still shows the lock; reference comes from sender's saved link |
| E29 | Several senders pay one obligation (siblings splitting fees) | Separate locks with the same reference; payee reconciles. No group-funding in MVP |
| E23 | Anchor withdrawal minimum exceeds a small tranche | Show minimums in cash-out guidance; payee can accumulate |
| E30 | Low literacy, shared phones, intermittent connectivity | Plain language, short flows, no account creation for beneficiaries, claim link works without login |
| E25 | Illicit use (fake payee, stolen-card on-ramp then lock) | Attester screening, caps, counsel; Kinlock cannot reverse on-chain releases |
| E32 | Payee is in an unsupported or sanctioned country | Registry CI rejects it; app blocks selection; contract can't enforce (trust assumption) |
| E33 | No reliable rate for a payee's currency | Show USD only with a note; never invent or hard-code a rate |
| E34 | Currencies differ in decimals, symbols, and number formats | Use `Intl` and ISO 4217 data for **display only**; on-chain amounts are USDC only |
| E35 | Volatile or high-inflation currencies | Stronger disclosure; payee prices in USD in the request; encourage shorter schedules |
| E36 | Capital controls or rules restricting payees from receiving stablecoins or foreign currency | Counsel per market; attester checklist confirms the payee is legally able to receive |
| E37 | Non-Latin names and scripts in payee data | `display_name` is UTF-8; slug stays ASCII; disambiguate with city, attester, and address |
| E38 | Sender and payee in the same country, or sender in a restricted country | Domestic is allowed; sender restrictions only as counsel directs (SND-11) |
| E39 | School calendars, rent conventions, and time zones differ by country | Tranche schedule is free-form; times shown in UTC and viewer's local time |

### 9.3 What is overengineered?

| ID | Item | Verdict |
|---|---|---|
| O1 | Two contracts (registry + vault) | **Cut** → one contract, modular code |
| O2 | Three release modes | **Cut** `SenderApproval`; **defer** `AttesterApproval` to P2 |
| O3 | Protocol fee, fee recipient, fee-per-lock | **Cut** from MVP |
| O4 | Payout-change timelock + approval flow | **Cut** → snapshot payout at lock creation |
| O5 | Mutual two-signer cancel | **Cut** → payee `decline` |
| O6 | Separate Purpose enum | **Cut** → purpose = payee category |
| O7 | Health, Utility, Other categories | **Cut** → School and Rent |
| O8 | Receipt table, random slugs, PDF and QR | **Cut/defer** → deterministic tx-based URL |
| O9 | Encrypted private reference store, users and notifications tables, attestation workflow and evidence storage | **Cut/defer** |
| O12 | Separate `kinlock-ramp` service at MVP | **Defer** until the M0 spike passes |
| O13 | SEP-12, SEP-38, SEP-45 | **Defer** |
| O14 | Passkey wallets, fee sponsorship, relayers | **Defer** → fund payee accounts with a small XLM float at onboarding |
| O18 | Full multi-language translations, CSV export, analytics | **Defer.** String externalization and locale-aware formatting are **not** deferred (SND-10) |
| O19 | On-chain country enforcement, geofencing in the contract, per-country compliance engines | **Cut.** Country is registry data and app policy |
| O20 | Multi-token support beyond USDC (for example EURC) | **Defer.** The allowlist already permits it later without contract changes |

**Keep (don't cut):** property tests for fund invariants, budget assertions, the registry repo with hash binding, expiry refund, revocation refund, chain-first reads, caps for mainnet.

## 10. Assumptions to validate in M0

1. Payees will accept USDC (or an equivalent route) for school fees or rent.
2. At least two credible attesters exist per pilot community.
3. Senders already can or will acquire USDC, and the all-in cost is acceptable.
4. In each pilot market, at least one anchor or wallet route supports practical local-currency cash-out, or payees can hold USDC.
5. Counsel agrees a testnet pilot with no real value is fine, and outlines mainnet constraints **per market**.
6. Two or more pilot markets can run on the same code with only registry-data and operational differences.

## 11. Milestones

| Milestone | Scope | Exit criteria |
|---|---|---|
| **M0 Validate and spike** (3 wks) | **Choose pilot market(s) and supported-countries policy**; 15 sender interviews, 5 payee interviews, 2 attester conversations across candidate markets; cost model vs incumbents; anchor/wallet cash-out spike per candidate market; counsel intro call | Written findings; **go / pivot / stop** decision against §8.2 |
| **M1 Contract** | Single `kinlock` contract with tests, property tests, budget assertions; testnet deploy; TS bindings | All P0 LCK/REG requirements pass |
| **M2 Indexer + SDK** | Event ingestion, read API, deterministic receipts, SDK | Receipt verification works from chain data |
| **M3 App** | Payee request flow, sender flow, claim flow, attester checklist tooling | End-to-end pilot on testnet with real institutions' *test* accounts |
| **M4 Hardening** | External audit, legal review **per launch market**, caps, monitoring, TTL and archive decisions | Go/no-go for capped mainnet beta |
| **M5 Ramp** (conditional) | Create `kinlock-ramp` only if M0 found a viable anchor | Tracked cash-out end-to-end |

**Delivery tracking:** detailed task status, dependencies, and gates live in `ROADMAP.md`. It must be updated in **every** contribution (see `AGENTS.md` §12), so this milestone table stays a summary and the roadmap stays the live record.

## 12. Repository map

| Repo | Responsibility | When |
|---|---|---|
| `kinlock-contracts` | Single Soroban contract (registry + vault modules), tests, deploy scripts, bindings | M1 |
| `kinlock-sdk` | TypeScript SDK and the event indexer + read API | M2 |
| `kinlock-app` | Next.js web app: payee, sender, attester views, receipt pages | M3 |
| `kinlock-registry` | Public payee JSON, schema, hash-check CI | M1 |
| `kinlock-ramp` | Anchor integration service | M5, conditional |

## 13. Open questions

1. Which attesters can we recruit, and what are they accountable for if a verified payee defrauds a sender?
2. Revenue model without a protocol fee: grants, anchor referral, B2B SDK licensing?
3. In each pilot market, which anchor or wallet route will payees actually use to reach local currency?
4. What are the legal entity and jurisdiction for the app and registry?
5. What does a sender's all-in cost need to be to beat a direct bank transfer?
6. ~~Should the maximum lock duration be 90 or 180 days?~~ Resolved: 149 days, the most the network's storage limit allows with a 30-day refund grace (ADR-0020).
7. Is a testnet pilot with institutions' test accounts enough signal, or do we need a very small real-value pilot after legal review?
8. **Which country (or countries) are the pilot markets?** Candidate: Nigeria (team is in Lagos) plus at least one market in a different region and currency, to prove the core is market-agnostic.
9. Supported-countries policy: who decides, on what criteria, and how are sanctions handled?
10. Enforce attester-country scope in registry CI only (current plan), or also on-chain?
11. Which rate source supplies indicative local-currency equivalents across many currencies?
