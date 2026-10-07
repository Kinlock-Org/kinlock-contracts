# AGENTS.md — Kinlock

Instructions for AI coding agents working in any Kinlock repository. Humans may read it too.

Kinlock lets **anyone, anywhere lock USDC on Stellar (Soroban) for a verified school or landlord anywhere in the world.** A sender locks funds to a verified payee (School or Rent), and funds can only go to that payee's payout address or back to the sender. The product is **worldwide by design and launched market by market**: nothing in the code is tied to one country. It handles other people's money, so **correctness and restraint matter more than speed or cleverness.**

Docs version this file was written against: **v0.3** (`PRD.md`, `ARCHITECTURE.md`, `ARCHITECTURE_ESSENTIALS.md`).

---

## 1. Read this first

**Source-of-truth order** (earlier wins):
1. `docs/ARCHITECTURE_ESSENTIALS.md`: read this at the start of every task. It is short.
2. `docs/ARCHITECTURE.md`: read only the sections relevant to your task.
3. `docs/PRD.md`: read when a task touches user-facing behavior or scope.
4. The code.

**Status, not design:** `ROADMAP.md` (repo root) tracks what is done, in progress, and left. Find the rows your task maps to before you start (see §12).

If code and docs disagree, or two docs disagree: **stop, say so, and ask.** Do not silently pick one. Do not "fix" the docs to match code (or code to match docs) without approval.

If a task would build something on the **Deferred list** (§4) or contradict a **Don't** (§3), refuse that part and explain why.

Docs are expected at `docs/` in each repo. If they're missing, say so and ask where they live.

## 2. How much autonomy you have

### Do freely
- Write and improve tests, docs, comments, types.
- UI components, copy fixes, accessibility, styling in `kinlock-app`.
- Registry tooling and schema validation in `kinlock-registry`.
- Refactors inside one module **with tests passing before and after**.
- Bug fixes outside the security-critical paths listed below.

### Ask a human first (and wait)
- Any change to contract **entry points, storage layout, events, invariants, constants, or errors**.
- Any change to who can call what (auth), or to where funds can go.
- New dependencies (state why, size, maintenance status, alternatives considered).
- Database migrations that alter or drop existing columns.
- CI/CD, deploy scripts, release config, `DEPLOYMENTS.md`.
- Anything touching keys, secrets, signing, or fee sponsorship.
- Starting any work in a repo or feature marked **conditional/deferred**.
- Changing a P0 row's priority, moving or redefining a gate, editing "What 100% readiness means," changing estimates, or dropping a P0/P1 row in `ROADMAP.md`.

### Never
- Commit secrets, secret keys, seed phrases, `.env` files, or Stellar identity files.
- Run anything against **mainnet**. Testnet and local only, unless a human explicitly says otherwise in this session.
- Force-push, rewrite shared history, or delete branches you didn't create.
- Disable, skip, or weaken a test to make it pass. If a test is wrong, explain why and ask.
- Hand-edit generated files (TS bindings, lockfiles, migrations' generated metadata).
- Invent facts about Stellar/Soroban behavior. If unsure, check the official docs or ask. Don't guess.

## 3. Hard rules (the non-negotiables)

These come from the architecture. Violating one is a bug even if tests pass.

1. **No privileged path to locked funds.** Funds leave the contract only to `lock.payout` (snapshotted at creation) or `lock.sender`. Admin, attesters, pause switches, and upgrades-by-convenience must not create another path.
2. **`lock.payout` is immutable.** Payout-address updates affect **new** locks only.
3. **Chain is truth.** Money-moving pages (claim, release, refund, decline) read state from chain, **never** from Postgres. Postgres is for lists and dashboards.
4. **Non-custodial.** No service holds user keys or funds. No server-side signing for users.
5. **No personal data** on-chain, in the public registry, in logs, or on servers. The claim-link fragment (`#r=<reference>&s=<salt>`) must never reach a server, log, analytics tool, error tracker, or third-party script.
6. **State before transfer.** In contract code, write state first, then move tokens. A failed transfer must revert everything.
7. **Honest wording.** Receipts say **"Payment to verified payee."** Never write "proof of use," "service delivered," or similar claims.
8. **Boundary semantics are fixed:** release only if `unlock_at ≤ now < expires_at`; refund only if `now ≥ expires_at`, payee Revoked, or payee Suspended past grace.
9. **Pause only blocks new locks.** It never blocks release, decline, or refund.
10. **Minimum viable.** Don't add features, config flags, abstractions, or services that the task doesn't need.
11. **Country-agnostic core.** Never hard-code a country, currency, anchor, language, or locale in the contract, SDK, indexer, or app. Country (ISO 3166-1 alpha-2) and local currency (ISO 4217) come from **registry data**. Don't put country logic or geofencing in the contract. Geography is registry-CI and app policy only. Don't assume Nigeria, naira, or any single market in code, tests, fixtures, copy, or examples (use neutral or multiple-country fixtures).

## 4. Deferred (do not build without explicit approval)

- Protocol fees or fee logic
- `SenderApproval` or any other release mode besides `PayeeClaim`
- Mutual cancel (use payee `decline`)
- Payout-change timelocks
- Health, Utility, or other categories (only `School` and `Rent`)
- `kinlock-ramp` and anything anchor/SEP-24/SEP-10 related (conditional on the M0 spike)
- SEP-12, SEP-38, SEP-45
- Passkey wallets, fee-bump relayers, fee sponsorship
- Redis, queues, OpenTelemetry
- Receipt tables, receipt slugs, PDF/QR receipts
- Private reference storage, user accounts, notification systems
- Full multi-language translations, CSV export (until P1), analytics. **Not deferred:** externalizing UI strings and locale-aware formatting (required from the start)
- Tokens beyond USDC (the allowlist permits them later without contract changes)

## 5. Repositories and where you are

| Repo | What it is |
|---|---|
| `kinlock-contracts` | One Soroban contract `kinlock` (modules `registry`, `vault`), tests, deploy scripts, generated bindings |
| `kinlock-sdk` | `packages/sdk` (TS client) and `services/indexer` (events → Postgres → list API) |
| `kinlock-app` | Next.js web app |
| `kinlock-registry` | Public payee JSON, schema, hash-check CI |
| `kinlock-ramp` | **Conditional.** Does not exist until the M0 spike passes |

Dependency flow: `contracts` → bindings → `sdk` → `app`. `registry` is data only.

Identify which repo you're in and apply its section in §8. Ignore sections for other repos.

## 6. Workflow

### Before coding
1. State the task in your own words and list your assumptions.
2. Read `ARCHITECTURE_ESSENTIALS.md` and the relevant `ARCHITECTURE.md` sections.
3. For anything non-trivial, write a short plan (files to touch, tests to add, risks) **before** editing. For anything in the "Ask first" list, stop after the plan and wait for approval.
4. Prefer the smallest change that solves the problem.

### While coding
- Edit existing files rather than creating parallel ones. Follow existing patterns.
- Keep each change to one logical purpose. Don't mix refactors with behavior changes.
- No dead code, commented-out code, or unresolved TODOs. If something is deferred, open an issue note in the PR description instead.
- Put design decisions in an ADR (`docs/adr/NNNN-title.md`) when you change or add an architectural decision. Don't bury decisions in code comments.

### Definition of done
A task is done only when **all** are true:
- [ ] Code compiles/builds; linters, formatters, and type checks pass.
- [ ] Tests added or updated for the change, and the full relevant suite passes.
- [ ] You **actually ran** the commands. Don't claim success from reading code.
- [ ] Docs updated if behavior, API, events, or decisions changed.
- [ ] **`ROADMAP.md` updated in this same PR** (row statuses, new rows, Changelog entry, date, counts). See §12.
- [ ] No hard rule (§3) is violated and nothing from §4 was introduced.
- [ ] PR description follows §7.

If you couldn't run something (missing toolchain, no network), say exactly what you didn't run.

### Git
- Branches: `feat/…`, `fix/…`, `docs/…`, `chore/…`, `test/…`.
- Commits: Conventional Commits (`feat(vault): …`, `fix(indexer): …`). Small and focused.
- Never commit to `main`. Open a PR.
- Mark PRs touching contract logic with the label **`security-sensitive`**.

## 7. Pull request format

```
## What and why
<1–3 sentences>

## Changes
<bullets>

## Hard rules / invariants touched
<list invariant numbers from ARCHITECTURE.md §4.6, or "none">

## Tests
<what you added; commands you ran and their result>

## Docs / ADR
<updated files, or "none needed">

## Roadmap
<row IDs touched and their new status; Changelog entry added: yes/no. If no rows changed, say "no row changes" and still add the Changelog entry>

## Assumptions and open questions
<anything a reviewer should double-check>
```

## 8. Repo-specific rules

### 8.1 `kinlock-contracts` (Rust / Soroban)

**Commands** (intended conventions; if a command doesn't exist yet, add it as a script rather than inventing a different one):
```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
stellar contract build
cargo test --test budget          # soroban-budget-assert cost tests
```
Bindings: generate with `stellar contract bindings typescript` into the committed bindings output. Never hand-edit.
Read `rust-toolchain.toml` for the pinned toolchain.

**Rules**
- `#![no_std]`. No floating point. No `std` collections.
- **No `unwrap`, `expect`, or bare `panic!`** in contract code. Use `#[contracterror]` enums and `panic_with_error!` / `Result`.
- **Checked arithmetic** on all `i128` amounts (`checked_add`, `checked_sub`). No `as` casts on amounts.
- `require_auth` on every privileged action. Test both classic (G) and contract (C) account callers.
- Storage: use typed `DataKey` enums. No unbounded collections. No per-user on-chain lists.
- TTL: extend on create to `expires_at + TTL_GRACE`; keep `bump_lock` permissionless. Max lock duration must stay below the network's max entry TTL.
- Time comes from `env.ledger().timestamp()`. Treat as approximate.
- Enums that are stored are **append-only**. Never reorder or remove variants.
- Every event carries `schema_version`.
- Constants live in one place with comments. Don't scatter magic numbers.

**Tests**
- Unit test every entry point, success and each failure path, including each `create_lock` validation.
- Property tests (`proptest`) for the invariants in `ARCHITECTURE.md` §4.6. Any change to fund logic must keep them green, and new logic needs new properties.
- `env.mock_all_auths()` is fine for convenience tests, but **auth behavior must also be tested with explicit auths** (assert `env.auths()`), since mock-all hides missing `require_auth`.
- Include boundary tests: `now == unlock_at`, `now == expires_at`, `now == expires_at - 1`.
- Include: missing/unauthorized trustline on release (reverts, retryable), token removed from allowlist with open locks, partial release then refund/decline, C-address sender and payee.
- Budget tests must not regress without an explanation in the PR.

**Deploys**
- Scripts default to **testnet**. Mainnet requires an explicit flag **and** human confirmation in-session.
- Don't edit `DEPLOYMENTS.md` by hand; update it through the deploy script output.

### 8.2 `kinlock-sdk` (TypeScript: SDK + indexer)

**Commands** (intended):
```
pnpm install
pnpm lint && pnpm typecheck && pnpm test
pnpm build
pnpm --filter indexer db:migrate     # Drizzle
pnpm --filter indexer dev
```

**Rules**
- TypeScript `strict`. No `any`. Validate all external input with **Zod** at boundaries (HTTP, RPC responses, env).
- **Amounts are `bigint` in code, decimal strings in JSON, `NUMERIC(39,0)` in Postgres.** Never use `number` for money. Decimal formatting (7 decimals for USDC) lives only in the SDK's formatting helpers.
- Validate Stellar addresses with the SDK's StrKey utilities.
- SDK exposes: `createLock`, `release`, `refund`, `decline`, `getLock`, `verifyReceipt`, `buildClaimLink`, `parseClaimLink`, `preflight`, plus (ADR-0025) `toBaseUnits`, `fromBaseUnits`, `generateSalt`, `computeRefHash`, `buildRequestLink`, `parseRequestLink`. Don't add public API without approval.
- `getLock` for action pages reads **chain state**. Indexer-backed reads are for lists only and must be labeled as such in the API.
- `buildClaimLink` puts reference and salt in the **URL fragment**. Never log, persist, or transmit them.
- Preflight covers: sender balance, payee Active, recent payout change (≤ 7 days), duplicate `ref_hash` for the payee, authorized USDC trustline on the payout account.

**Indexer**
- Idempotent writes: unique on `(tx_hash, event_index)`. Safe to replay.
- Persist the cursor after each batch. Never skip a gap silently: stop and alert.
- `chain_events` is append-only. Never update or delete rows.
- Handle multiple `schema_version` values per event type.
- RPC retention is limited: treat missing history as a gap, not as "no events".
- Migrations are forward-only and reviewed. Never edit an applied migration.
- Logs: structured (pino). No addresses paired with personal details, no claim-link data.

### 8.3 `kinlock-app` (Next.js)

**Commands** (intended):
```
pnpm install
pnpm lint && pnpm typecheck && pnpm test
pnpm dev
pnpm e2e        # one happy-path Playwright test: request → send → claim → verify
```

**Rules**
- Use only the SDK to talk to the contract. No ad hoc contract calls in components.
- **Claim, release, refund, decline, and approve pages read chain state**, not the indexer.
- `/claim/*` pages: **no third-party scripts, no analytics, no error-tracker payloads that include the URL fragment**, strict CSP, `Referrer-Policy: no-referrer`.
- Save claim links only in the sender's browser storage, with an export option. Never send them to a server.
- Wallet access goes through the wallet-kit abstraction so wallets stay swappable.
- Always show asset **code and issuer**. Never display an asset by code alone.
- Show dates in **UTC and local time**.
- Show USD amount plus an **indicative** equivalent in the payee's `local_currency` (from registry data) with a rate-risk note. If there is no reliable rate, show USD only with a note. **Never invent, hard-code, or present a local-currency number as guaranteed.** Rates come from the `RateProvider` interface and are display-only.
- **Externalize every user-visible string** into the message files (for example `messages/en.json`). No inline UI strings.
- Format numbers, currencies, and dates with `Intl` and the viewer's locale. Handle zero- and three-decimal currencies, non-Latin names, and right-to-left text without breaking layout.
- Don't hard-code any country, currency, or anchor in components. Read them from registry data.
- Copy: plain language, short flows, mobile-first, usable on slow connections. Receipt copy follows hard rule 7.
- Optimistic UI is allowed only until the indexer catches up, and must reconcile against chain.
- Accessibility: keyboard navigation, labels, contrast. Treat as required, not polish.

### 8.4 `kinlock-registry` (public data)

**Rules**
- One JSON file per payee in `payees/<country>/`, validated by `schemas/payee.schema.json`.
- **Public fields only**: `slug`, `display_name`, `category` (`School`|`Rent`), `country` (ISO 3166-1 alpha-2), `local_currency` (ISO 4217), `city`, `payout_address`, `attester`, `verified_at`.
- `supported-countries.json` lists the countries Kinlock currently supports. **Changing it needs a human and counsel's sign-off. Never edit it on your own initiative.**
- `attesters/<handle>.json` lists the countries each attester may vouch for. CI must fail a payee whose `country` is not supported, whose attester is not authorized for it, whose directory doesn't match its `country`, or whose `slug` is not globally unique.
- Use ISO codes everywhere. Never free-text country or currency names.
- **No personal data, no identity documents, no phone numbers, no evidence.** If in doubt, leave it out and ask.
- `meta_hash` = SHA-256 of sorted-key compact UTF-8 JSON via the repo's hash script. Don't compute hashes by hand.
- CI must validate schema, canonical formatting, and (post-registration) that the on-chain hash matches.
- Never add a payee on your own initiative. Payee additions come from a named attester through a reviewed PR.

### 8.5 `kinlock-ramp`
Conditional. If this repo is being created or modified and the M0 anchor spike hasn't passed, stop and ask.

## 9. Domain glossary

| Term | Meaning |
|---|---|
| **Lock** | A funded record: total, tranches, expiry, payee, payout snapshot, `ref_hash` |
| **Tranche** | A scheduled portion of a lock with its own `unlock_at` (max 12) |
| **Payee** | A verified institution (School or Rent) that can receive locks |
| **Attester** | A trusted party that verifies payees and vouches for them on-chain |
| **Payout** | The payee's receiving address, snapshotted into each lock at creation |
| **Release** | Payee claims a tranche |
| **Decline** | Payee returns the unreleased remainder to the sender |
| **Refund** | Sender reclaims the remainder after expiry, or on payee Revoked / Suspended + grace |
| **`ref_hash`** | `sha256(reference ‖ salt)`; the reference itself lives only in the claim link fragment |
| **Claim link** | `/claim/{id}#r=…&s=…`. The fragment never reaches servers |
| **Request link** | Payee-generated link that pre-fills the sender form; carries no authority |
| **Supported country** | A country on `supported-countries.json`: legal review done, attester(s) authorized, cash-out route or USDC holding practical. Registry/app policy, **not** enforced on-chain |
| **`local_currency`** | The payee's ISO 4217 currency from registry data; used for display-only indicative equivalents |
| **SAC** | Stellar Asset Contract; how USDC is used from Soroban |
| **TTL / archival** | Soroban state expiry; locks must have TTL ≥ expiry + grace |

## 10. When you're unsure

- Ambiguous requirement → ask **one** focused question, or state your assumption and proceed if it's low-risk and easy to reverse.
- Possible security implication → stop and ask. Don't "just try it."
- Stellar/Soroban behavior you can't verify → say it's unverified and point to what should be checked in the official docs. Don't present guesses as fact.
- Task seems to require breaking a hard rule → explain the conflict and propose an alternative.

## 11. Keeping docs honest

If your change alters behavior, an event, a constant, an API, or an architectural decision, update the matching doc section in the same PR and add or amend an ADR. The three docs must stay consistent with each other and with the code. `ROADMAP.md` must stay consistent with reality (§12).

## 12. ROADMAP.md is mandatory on every contribution

`ROADMAP.md` at each repo's root is the live record of everything left to reach 100% readiness. It is only useful if it is always current, so:

**Every pull request, from any human or agent, in any repo, must update `ROADMAP.md` in the same PR.** There are no "too small to update" exceptions. If no rows change, add a Changelog entry that says "no row changes."

### At the start of a task
- Open `ROADMAP.md` and find the row(s) your task belongs to (IDs like `M1-06`, `M3-14`).
- If no row covers your work, you'll add one when you finish (see below).
- Check the row's **Depends on** and **Status**. If it is `BLOCKED` or depends on rows that aren't `DONE`, stop and say so.
- Set the row to `IN PROGRESS` when you open your branch or PR.

### Before you finish (every task, every PR)
1. **Update row statuses** your change affects.
2. **Add new rows** for any work you discovered or created. Use the next unused ID at the end of the correct phase table. Never reuse or renumber IDs.
3. **Add a Changelog entry** (newest first): date, PR or issue reference, repo, rows touched, one-line summary.
4. **Refresh "Last updated"** and the progress counts (run `scripts/roadmap-progress` if it exists; otherwise update by hand and say so in the PR).
5. **Resolve decisions**: if your PR settles an item in the Pending Decisions table, mark it resolved and link the ADR.

### Status rules
- `DONE` only when your PR (or `main`) fully meets the row's **Done when** criteria **and you actually ran the verification commands.** Never mark `DONE` from reading code.
- Partially finished? Leave `IN PROGRESS` and add a new row for the remainder.
- `BLOCKED` must name what blocks it. `DEFERRED` and `DROPPED` need a reason. **Never delete rows.**
- Only edit rows owned by **your repo** (Repo column). Cross-cutting rows (`org`, `all`) go in the PR that completes them.
- On merge conflicts, rebase and keep **both** sides' row changes. Never overwrite another repo's rows.

### What needs a human first
Changing a P0 row's priority, gate definitions, §1 "What 100% means," estimates, or dropping a P0/P1 row (see §2).

### Enforcement
CI job `roadmap-check` fails any PR where `ROADMAP.md` is unchanged or the diff lacks a new Changelog entry. Reviewers must reject PRs without a roadmap update. Don't try to bypass the check.

