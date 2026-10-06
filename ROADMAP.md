# Kinlock — Roadmap

> **Mandatory:** every contribution (human or AI agent, any repo) must update this file in the same PR. See [§2](#2-how-this-file-is-maintained-mandatory).

| | |
|---|---|
| **Last updated** | 2026-10-06 |
| **Docs baseline** | v0.3, worldwide scope (`PRD.md`, `ARCHITECTURE.md`, `ARCHITECTURE_ESSENTIALS.md`, `AGENTS.md`, `CLAUDE.md`, `project_structure.md`) |
| **Current phase** | Phase 0 (Foundations) → starting Phase 1 (M0 Validate) |
| **Readiness** | See [§3](#3-progress-snapshot) |

---

## 1. What "100% readiness" means

Kinlock is **100% ready** when **every non-deferred row below is `DONE`**, which means all of the following are true:

1. All **P0 and P1** requirements in `PRD.md` are implemented, tested, and documented.
2. The M0 validation produced a recorded **go** decision (or the roadmap was revised after a pivot).
3. The testnet pilot met its success metrics, or a documented decision explains why we proceed anyway.
4. An **external audit** is complete, and all critical and high findings are fixed and re-verified.
5. A **legal review** has produced a written outcome **for each launch market** and the product's terms and disclosures reflect it.
6. Operations are ready: multisig and key ceremony done, monitoring and alerting live, runbooks written and rehearsed.
7. The contract is deployed to mainnet with **caps**, and a small capped beta is live and stable.
8. Docs are consistent with each other and with the code, and the Wave-ready issue backlog exists.
9. The core is **market-agnostic**: at least two pilot markets (different regions and currencies) ran on the same code, with only registry data and operations differing. Kinlock is worldwide by design and launched market by market.

Deferred and conditional work (Phase 9 and the parking lot in Phase 11) is **not** required for 100%.

> This roadmap is conditional on M0. If the M0 gate says **pivot** or **stop**, revise this file before doing anything else (row `M0-12`).

### Gates

| Gate | Name | Passes when |
|---|---|---|
| **G0** | Foundations | All Phase 0 rows `DONE` |
| **G1** | M0 decision | `M0-11` and `M0-12` `DONE`; decision recorded as go, pivot, or stop |
| **G2** | Contract complete | All Phase 2 rows `DONE`; contract on testnet |
| **G3** | Platform complete | All Phase 3 and Phase 4 rows `DONE` |
| **G4** | Pilot passed | All Phase 5 rows `DONE` |
| **G5** | Pre-mainnet features and freeze | All Phase 6 rows `DONE`; contract code freeze declared |
| **G6** | Hardening complete | All Phase 7 rows `DONE`; formal go/no-go signed (`H-24`) |
| **G7** | Launch | Launch rows `L-01` to `L-05` `DONE`; capped beta live |

Critical path: `G0 → G1 → G2 → G3 → G4 → G5 → G6 → G7`. Phases 3 and 4 can overlap once bindings and the SDK client exist. Book the auditor (`H-03`) **during the pilot**; audit lead time is the most likely schedule risk.

### Rough estimates (assumes 2–3 core engineers plus contributors; revisit after M0)

| Phase | Estimate |
|---|---|
| 0 Foundations | 1 week |
| 1 M0 Validate | 3 weeks |
| 2 Contract + registry | 6–8 weeks |
| 3 SDK + indexer | 4–5 weeks (overlaps) |
| 4 App | 6–8 weeks (overlaps) |
| 5 Testnet pilot | 4–6 weeks |
| 6 Pre-mainnet features | 3–4 weeks |
| 7 Hardening and audit | 8–12 weeks (audit queue dependent) |
| 8 Launch | 2–4 weeks |
| **Total** | **≈ 8–12 months** |

---

## 2. How this file is maintained (mandatory)

### The rule

**Every pull request, in every Kinlock repo, from any human or agent, must update `ROADMAP.md`.** No exceptions for "small" changes. If the PR changes no rows, it still adds a Changelog entry saying so.

### What to update in every PR

1. **Row statuses** affected by the change (see transitions below).
2. **New rows** for any work you discovered or created. Use the next unused ID in the right phase.
3. **Changelog entry** (§17): date, PR or issue reference, repo, rows touched, one-line summary. Newest first.
4. **Last updated** date at the top.
5. **Progress snapshot** counts in §3 (run `scripts/roadmap-progress` *(intended)*, or update by hand and say so).
6. **Decisions** (§10): move a decision to resolved and link the ADR if your PR settles it.
7. If the PR changes behavior, scope, or a gate, update the relevant docs too (`AGENTS.md` §11).

### Status values and transitions

| Status | Meaning | Rule |
|---|---|---|
| `TODO` | Not started | Default |
| `IN PROGRESS` | Branch or PR open | Set when you start; note the PR in the Changelog |
| `DONE` | Merged **and** the "Done when" criteria are met **and verified by running the commands** | Never set `DONE` from reading code or a hunch |
| `BLOCKED` | Can't proceed | Must name the blocking row or decision in the "Depends on" cell |
| `DEFERRED` | Intentionally postponed | Needs a reason; link an ADR if it changes a decision |
| `DROPPED` | No longer needed | Needs a reason. **Never delete rows** |

### Rules for editing rows

- **IDs are permanent.** Never reuse or renumber an ID. Add new rows at the end of their phase table.
- **Agents may:** update statuses, tick off their own work, add new rows, and add Changelog entries.
- **Agents must ask a human first** before: changing a row's priority from P0, moving or redefining a gate, editing §1 ("What 100% means"), changing estimates, marking someone else's `BLOCKED` row unblocked, or dropping a P0/P1 row.
- Only mark a row `DONE` if **your PR** completes it, or if you have verified it is complete on `main`.
- If a PR only partially completes a row, leave it `IN PROGRESS` and split off the remainder as a new row.

### Multiple repos, one roadmap

- Each repo has a copy of `ROADMAP.md` **at its root**. This is the one doc that is **not** vendored read-only.
- Each row has an **owning repo** (the Repo column). A PR edits only rows owned by its own repo, plus its own Changelog line and the top-of-file date. Cross-cutting rows (`all`, `org`) are edited in whichever repo's PR completes them.
- Because rows have unique IDs and owners, copies merge cleanly. A maintainer merges them into the canonical copy in the org `.github` repo (`docs/ROADMAP.md`) with `scripts/roadmap-merge` *(intended)*, at least weekly and at every gate.
- On conflict: rebase and keep **both** sides' row changes. Never overwrite another repo's rows.

### Enforcement (intended)

- CI job `roadmap-check` (every repo) fails the PR if `ROADMAP.md` is unchanged, or if the diff lacks a new Changelog entry that references the PR.
- The PR template includes a **Roadmap** section (`AGENTS.md` §7).
- Reviewers reject PRs without a roadmap update.

### Copy-paste checklist for PR authors

```
- [ ] Row statuses updated (and only DONE if verified)
- [ ] New rows added for new work
- [ ] Changelog entry added (date, PR, repo, rows, summary)
- [ ] Last-updated date and progress counts refreshed
- [ ] Decisions section updated if a decision was made
```

---

## 3. Progress snapshot

Readiness % = `DONE ÷ (all rows − DEFERRED − DROPPED)`. Conditional rows (Phase 9) are `DEFERRED` until their gate opens, then they join the denominator.

| Phase | Total | DONE | IN PROGRESS | TODO | BLOCKED | DEFERRED | DROPPED | Readiness |
|---|---|---|---|---|---|---|---|---|
| 0 Foundations | 19 | 12 | 2 | 5 | 0 | 0 | 0 | 63% |
| 1 M0 Validate | 17 | 0 | 1 | 16 | 0 | 0 | 0 | 0% |
| 2 Contract + registry | 36 | 16 | 5 | 15 | 0 | 0 | 0 | 44% |
| 3 SDK + indexer | 19 | 1 | 1 | 17 | 0 | 0 | 0 | 5% |
| 4 App | 24 | 1 | 1 | 22 | 0 | 0 | 0 | 4% |
| 5 Testnet pilot | 7 | 0 | 0 | 7 | 0 | 0 | 0 | 0% |
| 6 Pre-mainnet features | 10 | 0 | 0 | 10 | 0 | 0 | 0 | 0% |
| 7 Hardening | 26 | 0 | 0 | 26 | 0 | 0 | 0 | 0% |
| 8 Launch | 8 | 0 | 0 | 8 | 0 | 0 | 0 | 0% |
| 9 Conditional ramp | 9 | 0 | 0 | 0 | 0 | 9 | 0 | n/a |
| 10 Wave + community | 8 | 1 | 0 | 7 | 0 | 0 | 0 | 13% |
| 11 Deferred parking lot | 16 | 0 | 0 | 0 | 0 | 16 | 0 | n/a |
| **All** | **199** | **31** | **10** | **133** | **0** | **25** | **0** | **18%** |

---

## 4. Phase 0 — Foundations

**Goal:** the org, docs, and rules exist so everything after is consistent.
**Exit:** Gate G0.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| F-01 | Write `PRD.md` v0.2 | org | P0 | DONE | — | Reviewed; hard-questions section present |
| F-02 | Write `ARCHITECTURE.md` v0.2 | org | P0 | DONE | F-01 | Includes data models, stack, ADRs, hard-questions review |
| F-03 | Write `ARCHITECTURE_ESSENTIALS.md` v0.2 | org | P0 | DONE | F-02 | Outline fits one screen of key decisions |
| F-04 | Write `AGENTS.md` | org | P0 | DONE | F-02 | Autonomy levels, hard rules, per-repo rules |
| F-05 | Write `CLAUDE.md` | org | P0 | DONE | F-04 | Imports `AGENTS.md`; checklists present |
| F-06 | Write `project_structure.md` | org | P0 | DONE | F-02 | Repo layouts and ownership documented |
| F-07 | Write `ROADMAP.md` and the update rule | org | P0 | DONE | F-04 | This file; rule added to instruction files and docs |
| F-08 | Confirm org name availability and create the GitHub org (`kinlock` or backup) | org | P0 | DONE | — | Org exists; owners set |
| F-09 | Create org `.github` repo: profile README, CoC, CONTRIBUTING, SECURITY, SUPPORT, canonical `docs/`, `templates/` | org | P0 | IN PROGRESS | F-08 | Repo public; docs and templates committed |
| F-10 | Decide license (org-wide) | org | P0 | DONE | F-08 | ADR written; `LICENSE` template ready |
| F-11 | Decide npm scope and publish rights | org | P0 | IN PROGRESS | F-08 | Scope reserved; publish tokens/process documented |
| F-12 | Create GitHub teams: maintainers, contract-reviewers, attesters | org | P0 | TODO | F-08 | Teams exist; handles match `CODEOWNERS` |
| F-13 | Create label set across repos (`security-sensitive`, `good first issue`, `wave`, `area:*`, `blocked:m0`, `deferred`) | org | P0 | TODO | F-08 | Labels applied via script |
| F-14 | Write `scripts/sync-docs` and the `docs-in-sync` CI job | org | P0 | DONE | F-09 | CI fails when a vendored copy drifts |
| F-15 | Write `roadmap-check` CI job and `scripts/roadmap-progress` and `scripts/roadmap-merge` | org | P0 | DONE | F-09 | CI fails PRs without roadmap update; scripts produce correct counts and merges |
| F-16 | Define branch protection and repo settings (required reviews, required checks, no force-push to `main`) | org | P0 | TODO | F-12 | Applied to all repos; documented |
| F-17 | Create PR template and issue templates (bug, feature, wave-task) with Roadmap section | org | P0 | DONE | F-09 | Templates live in org `.github` and each repo |
| F-18 | Cross-doc consistency review after any M0-driven change | org | P1 | TODO | M0-12 | All docs agree; ADR index current |
| F-19 | Reconcile docs with scaffold findings: Next.js 16 renamed `middleware.ts` to `proxy.ts`; contract `tests/` must live in `contracts/kinlock/tests/` (a virtual Cargo workspace root can't hold integration tests); GitHub org is `Kinlock-Org`, not `kinlock`; request-link helpers are not in the SDK public API list; `TrancheInput` type added for `create_lock`; accept ADR-0017 and add it to the ADR indexes | org | P1 | TODO | — | Docs and scaffold agree; ADR-0017 accepted or rejected |

---

## 5. Phase 1 — M0: Validate and spike

**Goal:** find out whether the product should be built, and how, before building it.
**Exit:** Gate G1 (written go / pivot / stop against `PRD.md` §8.2).
**Why it matters:** the five likeliest failure causes (payee wallets, sender economics, off-ramp, attester trust, legal) are all answered here.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| M0-01 | Interview 15 senders across at least 2 candidate markets (script + note template) | org | P0 | TODO | F-08 | Notes stored; themes summarized; willingness to use USDC recorded |
| M0-02 | Interview 5 payees (schools, landlords) in candidate markets, incl. willingness to operate a wallet | org | P0 | TODO | F-08 | At least 3 of 5 answers recorded against the §8.2 trigger |
| M0-03 | Talk to at least 2 attesters (associations, NGOs) about accountability and process | org | P0 | TODO | F-08 | Named willing attesters or a documented "no" |
| M0-04 | Build sender all-in cost model vs incumbent routes | org | P0 | TODO | M0-01 | Spreadsheet with fees at each hop; target cost set (`M0-13`) |
| M0-05 | Off-ramp spike **per candidate market**: identify local-currency anchors and wallet routes (or whether payees can simply hold USDC); test SEP-1/10/24 on testnet; record assets, minimums, fees | org | P0 | TODO | — | Table of viable routes or a documented "none" |
| M0-06 | Payee usability test: onboard 3–5 payees on testnet using the attester checklist | org | P0 | TODO | M0-02 | Task completion rates and pain points recorded |
| M0-07 | Counsel intro call; written scoping of legal questions **per candidate market** (stablecoin acceptance by domestic payees, money transmission, sanctions and restricted jurisdictions, data protection including where senders live, terms) | org | P0 | TODO | — | Counsel engaged; question list and timeline agreed |
| M0-08 | Verify network facts: max entry TTL, RPC event retention, SAC/trustline failure behavior, USDC issuer flags (freeze, authorization, clawback) | contracts | P0 | IN PROGRESS | — | Findings written; `MAX_LOCK_DURATION` confirmed or changed via ADR |
| M0-09 | Research archive/backfill options for receipt verification (tier 3) and indexer gap recovery | sdk | P0 | TODO | M0-08 | Option chosen and costed; ADR drafted |
| M0-10 | Review Drips Wave rules: application limits, KYC, issue-sizing guidelines | org | P1 | TODO | — | Short note; Wave plan (`W-01`) adjusted |
| M0-11 | Write M0 findings report and go / pivot / stop decision against `PRD.md` §8.2 | org | P0 | TODO | M0-01..M0-09 | Report committed; decision recorded in §10 |
| M0-12 | Revise PRD, architecture, essentials, and this roadmap per findings | org | P0 | TODO | M0-11 | Docs updated; changed rows noted in Changelog |
| M0-13 | Set sender all-in cost target and pilot metric thresholds | org | P0 | TODO | M0-04 | Numbers recorded in PRD §8.1 |
| M0-14 | Competitive note: BarakahPay and other purpose-bound remittance projects | org | P1 | TODO | — | One-page comparison; differentiation confirmed or revised |
| M0-15 | Decide a sustainability/revenue approach (grants, referral, B2B SDK, fee later) | org | P1 | TODO | M0-11 | ADR or PRD §13 answer recorded |
| M0-16 | Choose pilot market(s) and write market-selection criteria (legal feasibility, attester supply, cash-out route, rate source, sender demand). Candidate: Nigeria plus at least one market in a different region and currency | org | P0 | TODO | M0-01, M0-02, M0-03 | Markets chosen and recorded (`DEC-19`) |
| M0-17 | Define the supported-countries policy and restricted-jurisdiction approach with counsel (who decides, criteria, how the list changes) | org | P0 | TODO | M0-07 | Policy written; `DEC-20` resolved |

---

## 6. Phase 2 — M1: Contract and registry

**Goal:** a complete, tested single contract on testnet, and the public registry.
**Exit:** Gate G2.
**Rule:** contract changes are **ask-first** and `security-sensitive` (see `AGENTS.md` §2).

### 6.1 `kinlock-contracts`

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| M1-01 | Scaffold workspace: Cargo workspace, `rust-toolchain.toml`, empty modules, CI (fmt, clippy, test, build), `localnet.sh` | contracts | P0 | IN PROGRESS | G1 | CI green on empty skeleton; one-command local network |
| M1-02 | Implement `constants.rs`, `types.rs`, `errors.rs`, `events.rs`, `storage.rs` | contracts | P0 | IN PROGRESS | M1-01 | Match `ARCHITECTURE.md` §4; enums append-only; events carry `schema_version` |
| M1-03 | Admin functions: `init`, attester add/remove, token add/remove, `set_paused_new_locks`, `set_caps`, `upgrade` | contracts | P0 | DONE | M1-02 | Auth tested; pause blocks only `create_lock` |
| M1-04 | Registry: `register_payee`, `set_status`, `update_payout` | contracts | P0 | DONE | M1-02 | Status transitions enforced; payout update affects new locks only |
| M1-05 | Vault: `create_lock` with all validations (token, payee Active, min/max, ≤12 tranches, `unlock_at ≤ expires_at`, max duration, `sender ≠ payout`, caps) | contracts | P0 | DONE | M1-04 | Each validation has a failing test |
| M1-06 | Vault: `release` (payee only; `unlock_at ≤ now < expires_at`; state before transfer) | contracts | P0 | DONE | M1-05 | Boundary tests pass |
| M1-07 | Vault: `refund` (expired, Revoked, Suspended past grace) | contracts | P0 | DONE | M1-05 | All three paths and rejections tested |
| M1-08 | Vault: `decline` | contracts | P0 | DONE | M1-05 | Returns only remainder; payee-only |
| M1-09 | `bump_lock` and TTL policy on create and on terminal states | contracts | P0 | IN PROGRESS | M1-05, M0-08 | TTL ≥ `expires_at + TTL_GRACE`; permissionless bump tested |
| M1-10 | `total_locked` bookkeeping and cap enforcement | contracts | P1 | DONE | M1-05 | Invariant 9 holds under property tests |
| M1-11 | Unit tests for every entry point (success and each failure) | contracts | P0 | DONE | M1-03..M1-09 | Coverage of every error variant |
| M1-12 | Explicit-auth tests (not only `mock_all_auths`) | contracts | P0 | DONE | M1-11 | Every privileged function asserts `env.auths()` |
| M1-13 | Boundary tests: `now == unlock_at`, `expires_at - 1`, `expires_at` | contracts | P0 | DONE | M1-06, M1-07 | Pass |
| M1-14 | Property tests for invariants 1–10 | contracts | P0 | DONE | M1-11 | `proptest` sequences green over many runs |
| M1-15 | Integration tests: C-address sender and payee, missing/unauthorized trustline, partial release then refund/decline, allowlist removal with open locks | contracts | P0 | TODO | M1-11 | Pass on local network |
| M1-16 | Budget tests with `soroban-budget-assert`; compare local estimates to testnet simulation | contracts | P0 | TODO | M1-11 | Budgets recorded; divergence measured and documented |
| M1-17 | `deploy.sh` (testnet default, mainnet guarded), `deployments/testnet.json`, generated `DEPLOYMENTS.md` | contracts | P0 | DONE | M1-11 | Deploys to testnet; mainnet requires flag and confirmation |
| M1-18 | Generate TS bindings and publish pipeline | contracts | P0 | TODO | M1-17, F-11 | Package published on tag; SDK can consume |
| M1-19 | Deploy to testnet with a test multisig admin; add test attesters and testnet USDC | contracts | P0 | DONE | M1-17 | Contract ID recorded in `DEPLOYMENTS.md` |
| M1-20 | Upgrade/migration test against a snapshot of testnet state | contracts | P1 | TODO | M1-19 | Invariants hold after upgrade |
| M1-21 | Contract `README`, `SECURITY.md`, and threat-model doc linked to invariants | contracts | P1 | TODO | M1-14 | Reviewer can map each threat to a control |
| M1-22 | Internal review using the `CLAUDE.md` contract checklist; fix findings | contracts | P0 | TODO | M1-16 | Checklist completed with evidence |
| M1-23 | Decide optional `refund_to` (ADR) | contracts | P2 | TODO | M1-05 | ADR accepted or declined |
| M1-30 | Source `soroban-budget-assert` (not published on crates.io) or choose an alternative for budget tests | contracts | P0 | TODO | — | Crate usable from CI; `M1-16` can start |
| M1-31 | Finish M1-03/M1-04 "Done when" once locks exist: tests that pause and token removal block only `create_lock`, and that `update_payout` leaves existing locks' payout unchanged | contracts | P0 | DONE | M1-05 | Tests pass; M1-03 and M1-04 can be marked `DONE` |
| M1-32 | Upgrade auth and success test with a real uploaded WASM (an unknown hash also fails, so today's test can't isolate auth) | contracts | P0 | DONE | M1-17 | Upgrade signed by admin succeeds and asserts `env.auths()`; intruder-signed upgrade fails |
| M1-33 | Reconcile `MAX_LOCK_DURATION` (180 days) with the network max entry TTL: testnet is 3,110,400 ledgers (~180 days at 5 s), so lock + 30-day grace caps expiry at ~150 days today. Set tests to the real max TTL | contracts | P0 | DONE | DEC-07 | Constant and tests match the network; ADR updated |
| M1-34 | Decide `bump_lock` behavior when the full TTL no longer fits (clamp vs error) and after `expires_at + TTL_GRACE`; decide whether to convert seconds to ledgers more conservatively than 5 s | contracts | P1 | TODO | M1-33 | Decision recorded; tests cover it |
| M1-35 | Spec decisions from the vault review: tranches with `unlock_at == expires_at` can never be released; suspend/re-activate toggling restarts the sender's refund grace | contracts | P1 | TODO | — | ADR accepted; code and tests follow it |
| M1-36 | Failed token transfers surface the token's error code, which collides with Kinlock codes (SAC code 10 decodes as `PayeeAlreadyExists`); decide on a dedicated error or document it for the SDK | contracts | P1 | TODO | — | Decision recorded; SDK handles it |

### 6.2 `kinlock-registry`

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| M1-24 | Scaffold registry: schema, `hash`, `validate` scripts, CI | registry | P0 | IN PROGRESS | G1 | CI validates schema and canonical formatting |
| M1-25 | `check-onchain` script and `onchain-match` workflow | registry | P0 | TODO | M1-19, M1-24 | Detects hash mismatch against testnet |
| M1-26 | Write `ATTESTER_CHECKLIST.md` and a script for trustline, XLM float, test release | registry | P0 | TODO | M0-06 | Checklist runs end to end on testnet |
| M1-27 | Add testnet fixture payees | registry | P0 | TODO | M1-24 | At least 3 examples across at least 2 countries, no personal data |
| M1-28 | `CODEOWNERS` for attesters | registry | P0 | TODO | F-12 | Attester review enforced |
| M1-29 | Registry: add `country` (ISO 3166-1), `local_currency` (ISO 4217), per-country directories, `supported-countries.json`, `attesters/<handle>.json`, and CI rules (supported country, attester scope, directory match, unique slug) | registry | P0 | IN PROGRESS | M0-16, M1-24 | Each rule has a failing-case test; fixtures cover at least 2 countries |

---

## 7. Phase 3 — M2: Indexer and SDK

**Goal:** typed client and a reliable event mirror for lists.
**Exit:** part of Gate G3.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| M2-01 | Scaffold `kinlock-sdk`: pnpm workspace, tsconfig, lint, CI, Postgres `docker-compose` | sdk | P0 | DONE | G1 | CI green on skeleton |
| M2-02 | SDK `client`: `createLock`, `release`, `refund`, `decline`, `getLock` (chain reads) | sdk | P0 | TODO | M1-18 | Works against local and testnet |
| M2-03 | SDK `hash` and `links`: `ref_hash`, `buildClaimLink`, `parseClaimLink`, request links | sdk | P0 | TODO | M2-02 | Fragment never leaves the client; round-trip tests |
| M2-04 | SDK `format`: bigint ↔ decimal strings; 7-decimal USDC formatting | sdk | P0 | TODO | M2-01 | No `number` used for money; edge-case tests |
| M2-05 | SDK `preflight`: balance, payee Active, recent payout change, duplicate `ref_hash`, authorized trustline | sdk | P0 | TODO | M2-02 | Each check has a test |
| M2-06 | SDK `receipts`: `verifyReceipt` tiers 1 and 2 | sdk | P0 | TODO | M2-02 | Valid and tampered receipts distinguished |
| M2-07 | SDK tests, including a check that nothing derived from the fragment is logged | sdk | P0 | TODO | M2-03 | Test fails if fragment data reaches logger |
| M2-08 | Indexer scaffold: Zod config, RPC client with multi-provider failover | sdk | P0 | TODO | M2-01 | Failover tested |
| M2-09 | Indexer ingest: poller, persisted cursor, gap detection | sdk | P0 | TODO | M2-08 | Gaps stop ingestion and alert; never skipped silently |
| M2-10 | Indexer handlers per event type and `schema_version` | sdk | P0 | TODO | M2-09, M1-02 | Mixed-version fixtures handled |
| M2-11 | DB schema and forward-only Drizzle migrations | sdk | P0 | IN PROGRESS | M2-01 | `chain_events`, `indexer_cursor`, `locks`, `tranches`, `payees` created |
| M2-12 | List API: `/locks`, `/locks/:id`, `/payees`, `/health` with lag | sdk | P0 | TODO | M2-10, M2-11 | OpenAPI or typed docs; lag exposed |
| M2-13 | Join payee display data from the registry repo | sdk | P0 | TODO | M1-25 | Payees show name, city, attester |
| M2-14 | Indexer tests: fixture replay, idempotency, cursor recovery, mixed versions | sdk | P0 | TODO | M2-10 | Replay twice yields identical DB |
| M2-15 | Lag alerting and health metric | sdk | P1 | TODO | M2-12 | Alert fires in a simulated lag test |
| M2-16 | Implement tier-3 verification and backfill per `M0-09` decision | sdk | P1 | TODO | M0-09, M2-06 | Old receipts verify; backfill fills a deliberate gap |
| M2-17 | SDK docs and API reference; publish testnet package versions | sdk | P0 | TODO | M2-07 | Published; docs match exports |
| M2-18 | Indexer container, testnet deployment, managed Postgres, daily backups | sdk | P0 | TODO | M2-14 | Running on testnet; restore tested once |
| M2-19 | Indexer and list API: store and filter payees by `country` and `local_currency` (display and filter only) | sdk | P0 | TODO | M2-13, M1-29 | `/payees?country=` works; no country logic elsewhere |

---

## 8. Phase 4 — M3: App

**Goal:** the full payee, sender, attester, and receipt experience on testnet.
**Exit:** completes Gate G3.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| M3-01 | Scaffold `kinlock-app`: Next.js, Tailwind, lint, CI | app | P0 | DONE | G1 | CI green |
| M3-02 | Wallet abstraction (Freighter and others behind one interface) | app | P0 | TODO | M3-01 | Wallets swappable; signing flow tested |
| M3-03 | `lib/sdk.ts` and Zod-validated public config | app | P0 | TODO | M2-02 | Single SDK instance; env validation |
| M3-04 | Landing page and plain-language explainer | app | P1 | TODO | M3-01 | Content reviewed against wording rules |
| M3-05 | `/request`: payee creates a payment-request link | app | P0 | TODO | M2-03 | Link pre-fills the sender form; carries no authority |
| M3-06 | `/send`: sender flow with preflight results, USD amount, indicative local-currency equivalent (USD-only fallback), rate-risk note | app | P0 | TODO | M2-05, M3-16 | Creates a lock on testnet end to end |
| M3-07 | Claim-link generation, browser-only storage, export option | app | P0 | TODO | M2-03 | Nothing stored or sent server-side |
| M3-08 | `/locks/[id]`: sender detail from chain and refund action | app | P0 | TODO | M2-02 | Refund shown only when allowed |
| M3-09 | `/claim/[id]`: payee claim page; verify reference against `ref_hash`; release | app | P0 | TODO | M2-03 | Release works; wrong reference flagged |
| M3-10 | Payee `decline` action | app | P0 | TODO | M3-09 | Returns remainder; confirmation step |
| M3-11 | `/payee` dashboard (lists via indexer) | app | P0 | TODO | M2-12 | Filter by status and reference |
| M3-12 | `/attester` tooling: read-only account checks (trustline, authorization, XLM float) | app | P1 | TODO | M1-26 | Checks match the checklist script |
| M3-13 | `/r/[txHash]/[eventIndex]` receipt page and `/verify` | app | P0 | TODO | M2-06 | Shows "Payment to verified payee"; Valid or Not valid |
| M3-14 | Security headers, strict CSP, `Referrer-Policy`, no third-party scripts on `/claim/*` | app | P0 | TODO | M3-09 | Automated test confirms fragment never appears in any network request |
| M3-15 | `AssetLabel` (code and issuer), `AmountDisplay`, UTC+local time components | app | P0 | TODO | M3-01 | Used everywhere money or time is shown |
| M3-16 | Choose and implement the `RateProvider` for indicative local-currency equivalents across currencies, with USD-only fallback and disclosure | app | P0 | TODO | M0-05 | Source documented; labeled indicative in UI |
| M3-17 | Accessibility pass (keyboard, labels, contrast) | app | P1 | TODO | M3-13 | Audit checklist passed |
| M3-18 | Mobile and low-bandwidth performance pass | app | P1 | TODO | M3-13 | Dashboard loads in under 2s on a mid-range phone profile |
| M3-19 | Playwright happy-path: request → send → claim → verify | app | P0 | TODO | M3-13 | Green in CI against testnet |
| M3-20 | Copy and wording review (plain language, no "proof of use", honest disclaimers) | app | P0 | TODO | M3-13 | Reviewer sign-off |
| M3-21 | Error and empty states: trustline failure, expired, revoked, suspended, indexer lag | app | P0 | TODO | M3-09 | Each state has clear guidance |
| M3-22 | Testnet hosting and environment config | app | P0 | TODO | M3-19 | Public testnet URL live |
| M3-23 | Externalize all UI strings (`messages/en.json`) and add `Intl`-based locale-aware formatting; handle zero- and three-decimal currencies, non-Latin text, and RTL-safe layout | app | P0 | IN PROGRESS | M3-01 | Lint or test fails on inline strings; formatting tests across at least 3 locales and currencies |
| M3-24 | CI guard: fail on hard-coded country, currency, anchor, or locale literals outside registry data, tests, and fixtures | all | P1 | TODO | M3-01, M2-01 | Guard runs in app and sdk CI; passing and failing examples tested |

---

## 9. Phase 5 — Testnet pilot

**Goal:** real institutions use the flow with **test assets only** and we measure it.
**Exit:** Gate G4.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| P-01 | Pilot plan and participant agreement (testnet, no real value) | org | P0 | TODO | G3 | Signed by participants |
| P-02 | Onboard at least 2 attesters and 10 payees using the checklist | org | P0 | TODO | P-01, M1-26 | 10 payees able to receive and release a test lock |
| P-03 | Run at least 100 locks across senders and payees | org | P0 | TODO | P-02 | Count reached; data exported |
| P-04 | Weekly feedback sessions; triage issues into the roadmap | org | P0 | TODO | P-02 | New rows created for findings |
| P-05 | Measure against `PRD.md` §8.1 and write the pilot report | org | P0 | TODO | P-03 | Report committed with metrics vs targets |
| P-06 | Decide: proceed to mainnet capped beta, extend pilot, or pivot | org | P0 | TODO | P-05 | Decision recorded in §10 |
| P-07 | Pilot covers at least 2 markets in different regions and currencies; confirm the same code runs with only registry-data differences; log any country-specific code found | org | P0 | TODO | P-02, M1-29 | Both markets complete end-to-end locks; no code changes needed for market 2 (or defects logged and fixed) |

---

## 10. Pending decisions

Resolve each with an ADR and link it here. Move resolved rows to the bottom with the date.

| ID | Decision | Needed by | Status |
|---|---|---|---|
| DEC-01 | Org name final (`kinlock` or backup) | F-08 | Open |
| DEC-02 | License | F-10 | Resolved 2026-10-06: Apache-2.0 (ADR-0022) |
| DEC-03 | npm scope and publish rights | F-11 | Resolved 2026-10-06: public npm, scope @kinlock, CI publishes on tag (ADR-0023) |
| DEC-04 | Canonical docs location (org `.github`, recommended) | F-09 | Open |
| DEC-05 | Commit Soroban `test_snapshots/` (default: ignore) | M1-01 | Open |
| DEC-06 | Package manager (default: pnpm) | M2-01 | Open |
| DEC-07 | Max lock duration (90 or 180 days) given network max TTL | M0-08 | Resolved 2026-10-05: 149 days (ADR-0020) |
| DEC-08 | Optional `refund_to` | M1-23 | Open |
| DEC-09 | Tier-3 archive and backfill source | M0-09 | Open |
| DEC-10 | Rate source for indicative local-currency display across currencies | M3-16 | Open |
| DEC-11 | Which anchor or wallet route gives payees local-currency cash-out, per pilot market | M0-05 | Open |
| DEC-12 | Revenue / sustainability approach | M0-15 | Open |
| DEC-13 | Legal entity and jurisdiction | M0-07 | Open |
| DEC-14 | New-payee release delay parameters | X-03 | Open |
| DEC-15 | Whether the indexer becomes its own repo | M2-18 | Open |
| DEC-16 | Bug bounty scope and budget | H-10 | Open |
| DEC-17 | **M0 outcome: go / pivot / stop** | M0-11 | Open |
| DEC-18 | Pilot outcome: proceed / extend / pivot | P-06 | Open |
| DEC-19 | Pilot market(s) | M0-16 | Open |
| DEC-20 | Supported-countries policy (who decides, criteria, restricted jurisdictions) | M0-17 | Open |
| DEC-21 | Enforce attester-country scope on-chain too, or registry CI only (default: CI only) | M1-29 | Open |
| DEC-22 | TypeScript tooling: linter, test runner, dev/script runner, and i18n library | M2-01, M3-01 | Resolved 2026-10-06: Biome, Vitest, tsx; built-in i18n helper (ADR-0021) |
| DEC-23 | JSON Schema validator for registry CI | M1-24 | Resolved 2026-10-06: Ajv + ajv-formats (ADR-0021) |

---

## 11. Phase 6 — Pre-mainnet features (P1) and code freeze

**Goal:** finish everything the PRD marks P1 **before** the audit, so the audit sees final contract code.
**Exit:** Gate G5.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| X-01 | `LCK-6` Repeat a previous lock in one tap (UI only) | app | P1 | TODO | M3-08 | Works for rent-style monthly locks |
| X-02 | `LCK-10` Payee "applied" acknowledgment (contract event) and UI | contracts | P1 | TODO | P-05 | Event emitted and shown on receipt; ADR if contract surface changes |
| X-03 | `REG-7` New-payee release delay in contract; set parameters via ADR | contracts | P1 | TODO | DEC-14 | Property tests updated; docs updated |
| X-04 | `SND-8` Email notifications (opt-in, provider-agnostic, `subscriptions` table) | sdk | P1 | TODO | P-05 | Opt-in and unsubscribe work; no personal data in logs |
| X-05 | `PAY-6` Payee CSV export | app | P1 | TODO | M3-11 | Export matches dashboard |
| X-06 | `PLT-4` Per-lock and global caps: set values and test enforcement | contracts | P1 | TODO | M1-10 | Cap breach rejected in tests; values documented |
| X-07 | `PLT-5` Upgrade timelock before mainnet | contracts | P1 | TODO | M1-20 | Timelock enforced; upgrade runbook updated |
| X-08 | Re-run property, budget, and integration suites after P1 contract changes | contracts | P1 | TODO | X-02, X-03, X-06, X-07 | All green; budgets recorded |
| X-09 | **Declare contract code freeze** and tag the audit candidate | contracts | P1 | TODO | X-08 | Tag created; no further contract changes without re-review |
| X-10 | Update all docs for P1 changes (PRD, architecture, essentials, ADRs) | org | P1 | TODO | X-09 | Docs consistent with frozen code |

---

## 12. Phase 7 — M4: Hardening

**Goal:** audit, legal, operations, and supply-chain readiness.
**Exit:** Gate G6.

### 12.1 Security

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| H-01 | Pre-audit pack: invariants, threat model, architecture, test results, deploy notes | contracts | P1 | TODO | X-09 | Pack delivered to auditor |
| H-02 | Internal adversarial review (fresh-context reviewer tries to break the contract) | contracts | P1 | TODO | X-09 | Findings triaged; fixes merged |
| H-03 | Select and book auditor; agree scope and schedule (start during pilot) | org | P1 | TODO | P-02 | Contract signed; slot confirmed |
| H-04 | External audit executed | contracts | P1 | TODO | H-01, H-03 | Draft report received |
| H-05 | Fix all critical and high findings; auditor verifies fixes | contracts | P1 | TODO | H-04 | Re-verification letter or updated report |
| H-06 | Publish the audit report and remediation summary | org | P1 | TODO | H-05 | Public link in docs |
| H-07 | Dependency and supply-chain checks (`cargo audit`, npm audit, lockfile review, SBOM) | all | P1 | TODO | X-09 | Clean or accepted risks documented |
| H-08 | CI and secrets hardening (least privilege, branch protection, required reviews) | all | P1 | TODO | F-16 | Verified by checklist |
| H-09 | App and indexer security test; verify the claim-link fragment never appears in any request or log | app | P1 | TODO | M3-14 | Test and manual review recorded |
| H-10 | Responsible disclosure policy live; decide on bug bounty | org | P1 | TODO | DEC-16 | `SECURITY.md` published; bounty decision recorded |

### 12.2 Legal and compliance

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| H-11 | Legal review with written outcome **per launch market** (stablecoin acceptance, money transmission, sanctions, data protection) | org | P1 | TODO | M0-07 | Written opinion on file; constraints listed |
| H-12 | Terms of service, privacy policy, disclosures (FX risk, issuer risk, public ledger privacy, attester trust) | app | P1 | TODO | H-11 | Published in app; reviewed by counsel |
| H-13 | Attester agreement and accountability policy | org | P1 | TODO | H-11, M0-03 | Signed by launch attesters |
| H-14 | Payee screening and sanctions process for attesters | org | P1 | TODO | H-11 | Documented procedure in attester checklist |

### 12.3 Operations

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| H-15 | Mainnet admin multisig: signer selection, hardware keys, key ceremony, recovery plan | org | P1 | TODO | X-07 | Ceremony documented and witnessed; recovery tested |
| H-16 | Monitoring and alerting: indexer lag, RPC health, contract anomalies (large locks, bursts to new payees, status changes), TTL watch | sdk | P1 | TODO | M2-15 | Alerts routed to on-call; test alerts received |
| H-17 | TTL bump job for open locks | sdk | P1 | TODO | M1-09 | Job runs; locks near expiry stay alive in a staging test |
| H-18 | Incident response runbook (compromised attester, payee fraud, issuer freeze, RPC outage, indexer gap, critical bug) | org | P1 | TODO | H-16 | Runbook reviewed; tabletop exercise done |
| H-19 | Operational runbooks: deploy, upgrade with timelock, restore archived entries, Postgres backup and restore, RPC rotation | org | P1 | TODO | X-07 | Each runbook rehearsed once |
| H-20 | Disaster recovery test: rebuild indexer from scratch via backfill | sdk | P1 | TODO | M2-16 | DB reconstructed and matches chain |
| H-21 | Load and performance test (indexer and app) | sdk | P1 | TODO | M3-18 | Meets targets in `PRD.md` §7 |
| H-22 | Reproducible build check: mainnet WASM hash matches the audited tag | contracts | P1 | TODO | H-05 | Hash match documented |

### 12.4 Readiness review

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| H-23 | Final docs consistency pass (PRD, architecture, essentials, agents, structure, roadmap) | org | P1 | TODO | H-12 | No contradictions; ADR index complete |
| H-24 | Mainnet go/no-go review and sign-off checklist | org | P1 | TODO | H-01..H-23 | Signed by maintainers; gate G6 passed |
| H-25 | Implement supported-country checks and sender-side restrictions in the app as counsel directs | app | P1 | TODO | H-11, M0-17 | Behavior matches counsel's written outcome; tested |
| H-26 | Record legal outcome, attesters, cash-out route, and rate source for **each** launch market (J8 playbook) | org | P1 | TODO | H-11 | One completed checklist per launch market |

---

## 13. Phase 8 — Launch (capped mainnet beta)

**Goal:** a small, monitored, capped mainnet beta.
**Exit:** Gate G7. After `L-05` the product counts as launched; `L-06` and `L-07` are ongoing.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| L-01 | Deploy to mainnet with caps; verify WASM hash and config | contracts | P1 | TODO | H-24 | `DEPLOYMENTS.md` updated by script; hash verified |
| L-02 | Configure token allowlist with the verified mainnet USDC issuer | contracts | P1 | TODO | L-01 | Issuer verified by two people |
| L-03 | Register launch attesters and first payees on mainnet | registry | P1 | TODO | L-01, H-13 | Registry and on-chain hashes match |
| L-04 | Small real-value pilot with explicit limits; monitor closely | org | P1 | TODO | L-03, H-16 | First locks released and verified; no incidents |
| L-05 | Public docs, status page, support channel | org | P1 | TODO | L-04 | Live and linked from the app |
| L-06 | Cap-raise review after a defined period of clean operation | org | P2 | TODO | L-05 | Criteria written; first raise decided |
| L-07 | Post-launch retrospective and roadmap v2 | org | P2 | TODO | L-05 | Retro notes; new roadmap section |
| L-08 | Publish and use the new-market launch playbook (`PRD.md` J8): checklist, templates, ownership | org | P2 | TODO | H-26 | Playbook used for the second launch market |

---

## 14. Phase 9 — Conditional: `kinlock-ramp`

**Condition:** only if the M0 anchor spike (`M0-05`) found a viable route **and** the decision allows it. Until then these rows are `DEFERRED` and excluded from readiness.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| C-01 | Gate decision: open the ramp work? | org | P2 | DEFERRED | M0-11 | Decision recorded; rows below re-statused |
| C-02 | Create `kinlock-ramp` repo with the standard files | ramp | P2 | DEFERRED | C-01 | Scaffold merged |
| C-03 | SEP-1 discovery and `/info` checks | ramp | P2 | DEFERRED | C-02 | Anchor table populated and re-checked on schedule |
| C-04 | SEP-10 auth and SEP-24 interactive withdrawal | ramp | P2 | DEFERRED | C-03 | Testnet withdrawal started from a payee wallet |
| C-05 | Status polling jobs and `ramp_sessions` table | ramp | P2 | DEFERRED | C-04 | Status reflects anchor state |
| C-06 | Path-payment builder (USDC to the anchor's accepted asset) | ramp | P2 | DEFERRED | C-04 | Unsigned transaction built and signed by payee in test |
| C-07 | App integration for in-app tracked cash-out (`PAY-7`) | app | P2 | DEFERRED | C-05 | End-to-end cash-out tracked in the UI |
| C-08 | Anchor drift checks and fixtures | ramp | P2 | DEFERRED | C-03 | CI detects `/info` changes |
| C-09 | Tests and docs; update architecture and structure docs | ramp | P2 | DEFERRED | C-07 | Docs consistent |

---

## 15. Phase 10 — Wave and community readiness

**Goal:** contributors can find, scope, and safely complete work.

| ID | Task | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| W-01 | Prepare Wave applications per repo (description, README, issues), within program limits | org | P1 | TODO | M0-10 | Applications submitted for the four active repos |
| W-02 | Seed issues: tests and docs in contracts (security-aware), registry tooling, app UI, SDK, indexer fixtures | all | P1 | TODO | M1-01 | Backlog exists; security-sensitive issues excluded from open contribution |
| W-03 | Org and repo `CONTRIBUTING.md` including the roadmap rule | org | P0 | DONE | F-09 | Rule stated prominently |
| W-04 | Contributor quick-start tested on a clean machine for each repo | all | P1 | TODO | M3-01 | New contributor runs tests in under 15 minutes |
| W-05 | Issue sizing guide aligned with current program guidelines | org | P1 | TODO | M0-10 | Guide published |
| W-06 | PR review SLAs and triage routine | org | P1 | TODO | F-12 | Documented; rota exists |
| W-07 | Maintainer rota for `security-sensitive` PRs | org | P1 | TODO | F-12 | Named reviewers per week |
| W-08 | Track application limits and KYC requirements per cycle | org | P2 | TODO | W-01 | Calendar and owner set |

---

## 16. Phase 11 — Deferred parking lot (not required for 100%)

Revisit only with a written reason and an ADR. See `AGENTS.md` §4.

| ID | Item | Repo | Pri | Status | Depends on | Done when |
|---|---|---|---|---|---|---|
| D-01 | `AttesterApproval` release mode (`LCK-7`) | contracts | P2 | DEFERRED | P-05 | Demand shown in pilot; ADR |
| D-02 | Top-up and dispute flows (`LCK-8`) | contracts | P2 | DEFERRED | P-05 | ADR |
| D-03 | Local-currency-indexed tranches via oracle (`LCK-9`) | contracts | P2 | DEFERRED | P-05 | ADR; likely never |
| D-04 | Health and Utility categories | contracts | P2 | DEFERRED | H-12 | Privacy analysis; ADR |
| D-05 | Protocol fee | contracts | P2 | DEFERRED | M0-15 | Revenue decision; vault v2 plan |
| D-06 | Passkey smart-account wallets | app | P2 | DEFERRED | L-05 | Library evaluated; SEP-45 compatibility checked |
| D-07 | Fee sponsorship / relayer | sdk | P2 | DEFERRED | L-05 | Abuse model written |
| D-08 | In-app on-ramp | app | P2 | DEFERRED | C-01 | Anchor route chosen |
| D-09 | PDF and QR receipts (`RCP-4`) | app | P2 | DEFERRED | L-05 | Design approved |
| D-10 | Payee self-service portal (`REG-8`) | app | P2 | DEFERRED | L-06 | Evidence handling designed |
| D-11 | Attester reputation and multi-attester verification | contracts | P2 | DEFERRED | L-06 | ADR |
| D-12 | Redis/queue, OpenTelemetry | sdk | P2 | DEFERRED | H-21 | Load test shows need |
| D-13 | SEP-12, SEP-38, SEP-45 support | ramp | P2 | DEFERRED | C-01 | Specific need documented |
| D-14 | Additional languages and full translations (string externalization is already required: `M3-23`) | app | P2 | DEFERRED | L-05 | Target languages chosen |
| D-15 | Group funding for one obligation | contracts | P2 | DEFERRED | P-05 | Demand shown |
| D-16 | Additional stablecoins via the token allowlist (for example EURC) | contracts | P2 | DEFERRED | L-06 | Demand shown; ADR; no contract change expected |

---

## 17. Changelog

Newest first. One entry per PR. Required for every contribution (see §2).

| Date | PR / ref | Repo | Rows touched | Summary |
|---|---|---|---|---|
| 2026-10-06 | `chore/license-and-publishing` | org | DONE: F-10. IN PROGRESS: F-11. DEC-02, DEC-03 resolved | Apache-2.0 for every repo (ADR-0022, `LICENSE` + template); TypeScript packages publish to npm under `@kinlock` from CI on tag (ADR-0023). F-11 waits on an owner creating the npm org and the `NPM_TOKEN` secret. Canonical roadmap re-merged from all repos (picks up M2-01, M3-01) |
| 2026-10-06 | `chore/ts-tooling` | sdk | DONE: M2-01. DEC-22, DEC-23 resolved (ADR-0021) | Biome (lint + format), Vitest, and tsx added; CI now runs lint, typecheck, test, and build. Tests pin the SDK's public API to the approved list and the contract's enum order, and check the indexer refuses bad config |
| 2026-10-06 | `chore/roadmap-sync` (lockfile) | sdk | no row changes (M2-01 stays IN PROGRESS: lint and test tooling pending DEC-22) | Commit `pnpm-lock.yaml` for the dependency versions approved by the owner on 2026-10-06, so CI's frozen install works; typecheck and build pass |
| 2026-10-06 | `chore/ts-tooling` | app | DONE: M3-01. DEC-22, DEC-23 resolved (ADR-0021) | Biome (lint + format) and Vitest added; CI now runs lint, typecheck, test, and build. Tests keep the receipt wording exactly "Payment to verified payee", ban overclaiming wording, and check every message key resolves. M3-23 stays IN PROGRESS (inline-string lint and multi-locale formatting tests) |
| 2026-10-06 | `chore/roadmap-sync` (lockfile) | app | no row changes (M3-01 stays IN PROGRESS: lint and test tooling pending DEC-22) | Commit `pnpm-lock.yaml` for the dependency versions approved by the owner on 2026-10-06, so CI's frozen install works; typecheck and build pass |
| 2026-10-06 | `docs/adr-ts-tooling` | org | DEC-22, DEC-23 resolved (no row changes) | ADR-0021: Biome, Vitest, tsx, Ajv + ajv-formats; the app keeps its built-in i18n helper |
| 2026-10-06 | `chore/roadmap-merge` | org | DONE: F-08, F-14, F-15, F-17, W-03 | Added `scripts/roadmap-merge` (three-way merge against the last merged copy; conflicts reported, never guessed) and merged every repo's ROADMAP.md into the canonical copy, then synced it to all repos. Verified: org exists with an owner; `docs-in-sync` and `roadmap-check` fail and pass correctly; templates and the CONTRIBUTING rule are in every repo. F-09 stays IN PROGRESS (code of conduct is still a draft) |
| 2026-10-06 | `chore/testnet-multisig-deploy` | contracts | DONE: M1-19 | Testnet contract `CCSHDQFRYFC3AHV5NE6ULQW6X2CMG5RPANBORDXJGSUD6UKECASJQBRI` with a 2-of-3 multisig admin (verified: one signature rejected `TxBadAuth`, two accepted), test attester added, Circle testnet USDC allowlisted. Adds `scripts/multisig-invoke.sh` and `deployments/testnet-setup.md`; vendored docs resynced (ADR-0020). Supersedes the single-key contract `CDIPDHSA…TLYL` |
| 2026-10-06 | `feat/deploy-script` | contracts | DONE: M1-17 | `scripts/deploy.sh` (testnet/local, guarded mainnet, dry run, uploaded-hash check) and `scripts/gen-deployments-md.sh`. Deployed to testnet: `CDIPDHSAKP2MNANLYV6VRWVTWFJH3PQRHBRDBVMYNMRYKSR66JVPTLYL`, single-key admin `kinlock-testnet-deployer` (multisig admin, attesters, and testnet USDC remain M1-19) |
| 2026-10-06 | `test/close-m1-11-m1-12` | contracts | DONE: M1-11, M1-12, M1-32 | Removed never-returned errors `AlreadyInitialized` (1) and `TrancheSumMismatch` (27), codes retired; every remaining error variant is now covered by a test. Upgrade tested with the real compiled WASM (admin auth asserted, state survives, intruder fails). CI installs stellar-cli 27.0.0 (checksum-verified) and builds the WASM before clippy and tests |
| 2026-10-06 | `test/property-invariants` | contracts | DONE: M1-14, M1-10 | Property tests for invariants 1–10: random sequences (create, release, refund, decline, status and payout changes, time jumps and exact boundaries, pause, allowlist and roster changes, frozen accounts) check every invariant after every step, and that release/refund/decline succeed exactly when the spec allows. Green over 512 cases; catches 13–14 of 14 injected vault bugs per 64-case run. Started before M1-11 was DONE, at the owner's request |
| 2026-10-05 | `fix/max-lock-duration` | contracts | DONE: M1-33. DEC-07 resolved | `MAX_LOCK_DURATION` = 149 days: testnet `max_entry_ttl` is 3,110,400 ledgers and a contract can extend to one ledger less, so 150 + 30-day grace doesn't fit. Tests now run with the real network limit |
| 2026-10-05 | `feat/vault` | contracts | DONE: M1-03, M1-04, M1-05, M1-06, M1-07, M1-08, M1-13, M1-31. IN PROGRESS: M1-09, M1-10, M1-11, M0-08. Added: M1-33, M1-34, M1-35, M1-36 | Vault: `create_lock`, `release`, `refund`, `decline`, `bump_lock`, `get_lock`; lock and payee TTL kept to `expires_at + TTL_GRACE` (`LockTtlTooLong` otherwise); `InvalidPayout`; `set_caps` sanity; storage version. 103 tests; 19 mutation checks caught; independent review: no critical/high. Testnet `max_entry_ttl` read as 3,110,400 ledgers (M0-08, M1-33). Vendored `docs/adr/` resynced with ADR-0018 and ADR-0019 |
| 2026-10-05 | `feat/admin-registry` | contracts | IN PROGRESS: M1-03, M1-04, M1-12 (M1-02 stays IN PROGRESS). Added: M1-31, M1-32 | Admin functions (constructor replaces `init`), payee registry (`register_payee`, `set_status`, `update_payout`, `get_payee`), TTL extension for long-lived entries, new errors `InvalidCap`, `PayoutUnchanged`, `PayeeRevoked`; 42 unit, auth, event, and TTL tests; independent review findings addressed. Started ahead of G1 at the owner's request |
| 2026-10-05 | `docs/adr-vault` | org | DEC-07 resolved (no row changes) | ADR-0020: vault validation and storage-lifetime rules (LockTtlTooLong, InvalidPayout, set_caps sanity, storage version); `MAX_LOCK_DURATION` = 149 days in ARCHITECTURE, ESSENTIALS, and PRD |
| 2026-10-05 | `docs/adr-admin-registry` | org | no row changes | ADR-0018 (constructor initialization, explicit caller parameters) and ADR-0019 (removed attesters lose payee powers), recording decisions approved for `kinlock-contracts` `feat/admin-registry`. Doc text updates remain under F-19 |
| 2026-10-05 | scaffold (requested by owner; branch `chore/scaffold` in each repo, uncommitted) | all | IN PROGRESS: F-09, F-14, F-15, F-17, W-03, M1-01, M1-02, M1-24, M1-29, M2-01, M2-11, M3-01, M3-23. Added: F-19, M1-30, DEC-22, DEC-23 | Scaffolded org `.github` (docs, 16 ADRs + proposed ADR-0017, templates, `sync-docs.sh`, `roadmap-progress`) and the contracts, registry, sdk, and app repos with data models and stubs. Done ahead of G1 at the owner's request. `kinlock-ramp` left untouched (conditional). Counts refreshed with `scripts/roadmap-progress` |
| 2026-10-05 | scope change (requested by owner) | org | Edited: M0-01, M0-02, M0-05, M0-07, M1-27, M3-06, M3-16, H-11, D-03, D-14, DEC-10, DEC-11, §1 items 5 and 9. Added: M0-16, M0-17, M1-29, M2-19, M3-23, M3-24, H-25, H-26, P-07, L-08, D-16, DEC-19..21 | Product generalized from "abroad to Nigeria" to worldwide (market-by-market launch). Docs bumped to v0.3; country-agnostic-core rule added to `AGENTS.md`, `CLAUDE.md`, `ARCHITECTURE_ESSENTIALS.md`. §1 edited with the owner's approval |
| 2026-10-05 | initial | org | F-01..F-07 | Roadmap created; docs v0.2 baseline recorded; update rule added to `AGENTS.md`, `CLAUDE.md`, `project_structure.md`, `PRD.md`, `ARCHITECTURE.md`, `ARCHITECTURE_ESSENTIALS.md` |
