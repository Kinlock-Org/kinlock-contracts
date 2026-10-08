---
name: Wave task
about: A scoped, self-contained task for an external or bounty-program contributor
labels: wave
---

<!--
Kinlock's one issue format. Keep every section; write "None" if one does not apply.
Delete nothing: a missing section is returned, not merged.
Before filing: the task must (a) map to at least one ROADMAP.md row, (b) avoid every
security-sensitive path in docs/project_structure.md §8, and (c) avoid anything on the
AGENTS.md §4 deferred list. Tier rules and triage: docs/wave-issue-format.md.
-->

**Seeded:** YYYY-MM-DD

## Task

One to three sentences: what to build or fix, and what the world looks like when it is done.

## Context

Where this sits today, with `path:line` evidence for each claim. Name the `docs/ARCHITECTURE.md`
sections the task touches, because the mandatory review below points at them.

-

## Before you start: mandatory context review

Kinlock moves other people's money, so the docs are **binding constraints, not background reading**.
They record decisions this task must not silently reopen. Read them in this order and tick each box:

- [ ] `docs/ARCHITECTURE_ESSENTIALS.md` — start here, it is short (`AGENTS.md` §1)
- [ ] `docs/ARCHITECTURE.md` — only the sections named in **Context** above
- [ ] `docs/PRD.md` — if this task touches user-facing behaviour or scope
- [ ] `AGENTS.md` §3 (hard rules), §4 (deferred list), §8 for this repo, §12 (roadmap rule)
- [ ] `ROADMAP.md` — the row(s) listed under **Related roadmap rows**, including their **Depends on**

In your first PR comment, name the hard-rule numbers or doc sections that constrain this task. That
line is how we know the review happened.

Then:
- If code and docs disagree, or two docs disagree: **stop and say so on this issue.** Do not pick one
  silently, and do not "fix" either side without approval (`AGENTS.md` §1).
- If the task looks like it needs anything from the deferred list (§4), a change to a hard rule (§3),
  or a **new dependency** (§2): raise it here first. Those need a human; deciding them is not your job.
- If a row you are told to finish is `BLOCKED` or depends on a row that is not `DONE`, say so here
  instead of starting.

## Why we need this

Two to four bullets. Say which commitment this serves: a hard rule (§3), a gate, a pilot need, or a
row's **Done when**. If the honest answer is "nicer", the issue is not ready to be filed.

-

## Scope

**In scope**
-

**Do not touch** (security-sensitive or generated; changing these needs a human first — `AGENTS.md` §2, `docs/project_structure.md` §8)
-

## Suggested implementation

Numbered steps, with the read-first documents in step 1. Say explicitly that an alternative approach
is welcome and should be posted on the issue before building.

1.

## Done means

Binary and checkable, one line each. Anything a reviewer cannot verify by looking is not a criterion.

- [ ]

## Tests

What to add, what not to weaken, and which existing cases must stay untouched. State the rule: no
skipped tests, no relaxed assertions, no deleting a fixture to make a case fit.

## Documentation

Which doc or ADR changes with this task, or "None" — and if the behaviour changes, "None" is wrong.

## Verify before you open the PR

Every command, in the order that works, so a contributor on a clean machine is not guessing.

```
```

Run every command and paste the real output in the PR. We do not accept "should pass" —
`AGENTS.md` §6: *"You actually ran the commands."*

## Opening the pull request

- Branch `feat/…`, `fix/…`, `docs/…`, `test/…` or `chore/…`; Conventional Commit subject.
- Update `ROADMAP.md` in the same PR (`AGENTS.md` §12): row status, a Changelog entry, `Last updated`.
  CI fails a PR without it. Only rows owned by this repo.
- Follow `.github/PULL_REQUEST_TEMPLATE.md` (`AGENTS.md` §7); in "Hard rules / invariants touched",
  give numbers, not prose.
- One logical change. Do not reformat or touch files outside **Scope**.
- Do not add the `security-sensitive` label — that is the maintainer's flag for contract-logic PRs.
- Never commit to `main`; open a PR. Never force-push.

## Tier and effort

`tier/trivial` | `tier/medium` | `tier/high` · roughly \<hours or days\>. Definitions and the Wave
point mapping: `docs/wave-issue-format.md` in `Kinlock-Org/.github`.

## Related roadmap rows

`ID` — row description (status). New rows created for a batch of issues are listed with the PR that
added them.

## Questions

Answer time for this repo, and the claim protocol ("I'll take this"), plus the instruction to post a
draft PR when stuck rather than going quiet.
