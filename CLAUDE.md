# CLAUDE.md — Kinlock

@AGENTS.md

The file above is the full rulebook. Everything below is **additional guidance specific to Claude Code**. If anything here conflicts with `AGENTS.md`, `AGENTS.md` wins. Flag the conflict instead of choosing.

## Start of every session

1. Read `docs/ARCHITECTURE_ESSENTIALS.md`. It's short. Don't load `ARCHITECTURE.md` or `PRD.md` in full unless the task needs it; read just the relevant sections.
2. Work out which repo you're in (`AGENTS.md` §5) and apply only that repo's rules.
3. Open `ROADMAP.md` and find the row(s) your task maps to. Check **Status** and **Depends on**. If the row is `BLOCKED` or depends on rows that aren't `DONE`, stop and say so. Set the row to `IN PROGRESS` once you start.
4. Check git status and the current branch before editing anything.

## How to work

### Plan first, then wait when required
- For anything beyond a small fix, **use plan mode**: list files to change, tests to add, invariants touched, and risks.
- For contract changes, auth changes, storage/event changes, dependency additions, migrations, CI/deploy changes, or anything in `AGENTS.md` §2 "Ask a human first": **present the plan and stop for approval.** Don't start editing.

### Verify, don't assume
- Run the commands for the repo (`AGENTS.md` §8) before saying something works.
- Report results honestly: which commands ran, what passed, what failed, and what you couldn't run. Never say "tests pass" without running them.
- If a failing test points at a real bug, fix the bug. Don't edit the test to match.
- After a fix, run the **full** relevant suite, not just the one test.

### Stay in scope
- Do what was asked. Don't add features, config options, or "while I'm here" refactors.
- If you notice something unrelated that's broken or risky, **mention it at the end** instead of fixing it silently.
- Prefer editing existing files. Don't create new files, abstractions, or docs unless the task needs them.
- If the task touches the **Deferred list** (`AGENTS.md` §4), say so and don't build that part.

## Finish every task by updating `ROADMAP.md` (mandatory)

**Every PR you produce must update `ROADMAP.md`. No exceptions, including docs-only, test-only, and one-line fixes.** This is part of the work, not an afterthought. Full rules: `AGENTS.md` §12.

Before you say a task is done:
1. Update the status of every row your change affects. Use `DONE` **only** if the row's "Done when" is met and you ran the verification.
2. Add new rows (next unused ID, end of the right phase table) for work you discovered or created. Never reuse or renumber IDs. Never delete rows.
3. Add a Changelog entry (newest first): date, PR/ref, repo, rows touched, one-line summary. If no rows changed, write "no row changes."
4. Refresh "Last updated" and the progress counts. Run `scripts/roadmap-progress` if it exists; otherwise update by hand and say so.
5. Resolve any Pending Decision your change settles, with an ADR link.
6. Include the **Roadmap** section in the PR description.

Edit only rows owned by the repo you're in. On conflicts, keep both sides' row changes. Changing a P0 priority, a gate, §1 "What 100% means," or estimates needs human approval first.

In your final summary to the human, state which roadmap rows you changed.

## Using Claude Code features

### Subagents
Use subagents for **independent review**, not to bypass approval rules:
- After any change to contract fund logic, have a fresh-context review pass check the change against the invariants in `ARCHITECTURE.md` §4.6 and the hard rules in `AGENTS.md` §3. The reviewer should try to *break* it: boundary times, missing trustlines, auth gaps, arithmetic overflow, state-before-transfer ordering.
- Use read-only exploration subagents when you need to survey a large area of code without filling your own context.

### Context hygiene
- Keep context lean. Summarize findings rather than pasting large files back.
- Don't paste secrets, keys, or `.env` contents into the conversation, ever.
- When context gets long, compact and keep: the task, the plan, decisions made, files touched, and open questions.

### Optional project setup (recommended, not required)
Ask the human before creating any of these:
- `.claude/settings.json` permission rules that **deny reading** `.env*` and Stellar identity/key files, and **deny** mainnet-targeting commands.
- A hook that runs `cargo fmt` / `cargo clippy` (contracts) or lint/typecheck (TypeScript) after edits.
- A reusable `/review-contract` command and a `contract-reviewer` subagent that apply the checklist below.

Verify exact configuration syntax against the current Claude Code documentation before writing these files.

## Contract change review checklist

Run this before you call any contract change finished. Report each item as pass/fail with evidence.

- [ ] No country, currency, or geography logic in the contract (it stays country-agnostic)
- [ ] No new path moves funds anywhere except `lock.payout` or `lock.sender`
- [ ] `lock.payout` is never modified after creation
- [ ] State is written **before** every token transfer
- [ ] Every privileged function calls `require_auth` on the right address, with an **explicit-auth test** (not only `mock_all_auths`)
- [ ] All amount arithmetic is checked; no `as` casts on amounts; no `unwrap`/`expect`/`panic!`
- [ ] Boundaries tested: `now == unlock_at`, `now == expires_at - 1`, `now == expires_at`
- [ ] Failed token transfer reverts all state (tested with a missing/unauthorized trustline)
- [ ] Pause, allowlist removal, attester removal, and status changes don't block release/decline/refund on existing locks
- [ ] Partial release followed by refund or decline returns only the remainder
- [ ] `total_locked` bookkeeping stays equal to the sum of Open-lock remainders
- [ ] Property tests updated for any new logic and green
- [ ] Budget tests green; any increase explained
- [ ] Events include `schema_version`; indexer handles the change
- [ ] Stored enums changed append-only; migration/upgrade test added if layout changed
- [ ] `ARCHITECTURE.md` and `ARCHITECTURE_ESSENTIALS.md` updated; ADR added if a decision changed
- [ ] PR labeled `security-sensitive`
- [ ] `ROADMAP.md` updated (rows, Changelog entry, date, counts)

## Frontend change review checklist

- [ ] Money-moving pages read from chain, not the indexer
- [ ] Nothing derived from the claim-link fragment is logged, stored server-side, or sent to third parties
- [ ] No new third-party script on `/claim/*`
- [ ] Asset shown as code **and** issuer
- [ ] Local-currency amounts labeled **indicative**, USD-only fallback when no reliable rate, never hard-coded
- [ ] No hard-coded country, currency, anchor, or locale; values come from registry data
- [ ] Every user-visible string is in the message files; formatting uses `Intl`
- [ ] Dates shown in UTC and local time
- [ ] Receipt copy says "Payment to verified payee," nothing stronger
- [ ] Works on a narrow mobile viewport and with keyboard only
- [ ] `ROADMAP.md` updated (rows, Changelog entry, date, counts)

## Communication style

- Be concise and direct. Lead with the result, then the evidence.
- Separate **what you did**, **what you verified**, and **what you assumed or couldn't check**.
- Always list the **`ROADMAP.md` rows you changed** and their new statuses.
- Ask at most one focused question at a time when blocked. If you can proceed safely on a stated assumption, do that instead.
- If you made a mistake, say so plainly, fix it, and move on.
- Don't invent Stellar/Soroban behavior. If you're not sure, say it's unverified and name the doc or test that would settle it.

## Quick reference: the rules most likely to be broken

1. Funds only go to `lock.payout` (snapshot) or the sender.
2. Action pages read chain, not Postgres.
3. Claim-link fragment never leaves the browser.
4. State before transfer; failed transfer reverts everything.
5. No `unwrap`/`expect`; checked `i128` math.
6. Testnet only. Never mainnet.
7. Don't build anything on the Deferred list.
8. Contract, auth, storage, event, dependency, migration, CI, and deploy changes: **plan, then wait for approval.**
9. **Every PR updates `ROADMAP.md`** (statuses, new rows, Changelog entry). No exceptions.
10. **Country-agnostic core.** No hard-coded country, currency, anchor, or locale anywhere. Don't assume Nigeria or naira in code, tests, fixtures, or copy. Never edit `supported-countries.json` on your own.
