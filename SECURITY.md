# Security

Please report vulnerabilities privately. Policy: [Kinlock-Org/.github SECURITY.md](https://github.com/Kinlock-Org/.github/blob/main/SECURITY.md).

## Threat model

This maps each of the 10 property-tested invariants to where it's actually checked, and each
threat in the security model to its control and the invariant(s) that enforce it. Everything
below is a pointer into real, running tests, not a design aspiration: run `cargo test --workspace`
and `cargo test --test properties` to verify any row yourself.

### Invariants → tests

The full statements are `docs/ARCHITECTURE.md` §4.6. Every invariant is checked after **every**
step of a randomized action sequence in the property-test suite
(`contracts/kinlock/tests/properties.rs::check_invariants`, `proptest` row `M1-14`), not just by
fixed unit tests. The unit tests below are the clearest single example of each rule; the property
suite is what proves it holds under arbitrary combinations of create/release/refund/decline,
status changes, admin actions, and time jumps.

| # | Invariant | Checked in `properties.rs` | Representative unit test |
|---|---|---|---|
| 1 | `released + returned ≤ total`; equality when terminal | `check_invariants`, per-state match | `create_lock_tests::accepts_boundary_values` |
| 2 | Tranches sum to `total`; `unlock_at ≤ expires_at` | `check_invariants`, tranche loop | `create_lock_tests::rejects_unlock_after_expiry`, `rejects_tranche_sum_overflow` |
| 3 | Funds exit only to `lock.payout` or `lock.sender` | `check_invariants`: every tracked address's balance must equal exactly what the locks say it should hold, nothing more | `release_tests::release_at_unlock_time_pays_the_payout`, `refund_decline_tests::refund_at_expiry_returns_everything` |
| 4 | A tranche releases at most once | `check_invariants`, `released_sum` vs tranche flags | `release_tests::double_release_fails` |
| 5 | Release only inside its time window, by `lock.payout` | `release_allowed` oracle, asserted equal to the real call's outcome in `do_release` | `release_tests::release_before_unlock_fails`, `release_at_expiry_fails`, `release_blocked_while_payee_not_active`; `auth_tests::release_and_decline_require_the_lock_payout` |
| 6 | Refund only on expiry, Revoked, or Suspended-past-grace | `refund_allowed` oracle, asserted equal in `do_refund` | `refund_decline_tests::refund_not_allowed_before_expiry_while_active`, `refund_immediately_when_payee_revoked`, `refund_after_suspension_grace_only`; `auth_tests::refund_requires_the_lock_sender` |
| 7 | `lock.payout` is immutable after creation | `check_invariants`, compared against `created_payout` snapshot taken at creation | `existing_locks_tests::payout_update_affects_only_new_locks` |
| 8 | Admin/attester/pause/allowlist actions never block release, decline, or refund on existing locks | `Op::AtBoundary`: applies an admin action (pause, remove token, remove attester), then tries the action at an exact time boundary, still checked against the unchanged `*_allowed` oracles | `existing_locks_tests::pause_blocks_only_new_locks`, `token_removal_blocks_only_new_locks`, `attester_removal_does_not_block_existing_locks` |
| 9 | `total_locked` equals the sum of Open-lock remainders | `check_invariants`, `config.total_locked` vs `open_remainders` | `existing_locks_tests::total_locked_tracks_open_remainders` |
| 10 | A failed token transfer reverts the whole call | Every `do_release`/`do_refund`/`do_decline` snapshots state before the call and asserts no change on failure | `create_lock_tests::insufficient_balance_reverts_everything`, `release_tests::frozen_payout_reverts_and_can_retry` |

### Threats → controls

Base table: `docs/ARCHITECTURE.md` §8. Enriched here with the invariant(s) and tests that make
each control verifiable rather than asserted.

| Threat | Control | Invariant(s) | Verified by |
|---|---|---|---|
| Admin or operator steals locked funds | No code path to any destination except `lock.payout` or `lock.sender` | 3 | Property suite, every step; `auth_tests::admin_functions_require_admin` |
| Attester registers a fraudulent payee | Accepted trust assumption: named attester shown on every payee; revocation triggers immediate refund; affects *future* locks only | 6, 7 | `refund_decline_tests::refund_immediately_when_payee_revoked` |
| Attester or payee key compromise redirecting *existing* locks | Not possible: payout is snapshotted per lock | 7 | `existing_locks_tests::payout_update_affects_only_new_locks` |
| Replay / double release | Per-tranche `released` flag set before transfer | 1, 4, 10 | `release_tests::double_release_fails` |
| Admin pauses, removes a token, or removes an attester mid-flight | None of these block release, decline, or refund on locks that already exist | 8 | `existing_locks_tests::pause_blocks_only_new_locks`, `token_removal_blocks_only_new_locks`, `attester_removal_does_not_block_existing_locks` |
| Upgrade abuse | Multisig required; state must survive the upgrade; intruder-signed upgrade fails | n/a | `upgrade_tests::admin_upgrade_requires_admin_and_keeps_state`, `upgrade_signed_by_non_admin_fails` |
| Missing `require_auth` on a privileged call | Explicit-auth tests assert the actual signer, not just `mock_all_auths` | n/a | `auth_tests::*` (14 tests, every entry point) |
| Failed token transfer leaves partial state | State written before transfer; failure reverts the whole call | 10 | `create_lock_tests::insufficient_balance_reverts_everything` |
| Frozen / unauthorized trustline on release or refund | Call fails cleanly and is retryable once the trustline is fixed | 10 | `release_tests::frozen_payout_reverts_and_can_retry`, `refund_decline_tests::frozen_sender_reverts_refund_and_decline` |

The remaining rows in `docs/ARCHITECTURE.md` §8 (look-alike assets, reference leakage, indexer
lies, issuer-level controls, spam locks, country/sanctions policy) are product- or
off-chain-level controls rather than contract invariants, so they aren't repeated here.

### Independent review evidence

- Property suite: 512 random action sequences per run, catching 13-14 of 14 deliberately injected
  bugs in a 64-case run (`test/property-invariants`).
- `feat/vault`: 19 mutation-testing checks caught before merge.
- Every admin/registry/vault PR has gone through an independent fresh-context review against
  these invariants before merge (`AGENTS.md` §6, CLAUDE.md "Subagents").

### What this doesn't cover yet

- `M1-15` (integration tests: C-address sender/payee, missing trustline, allowlist removal with
  open locks) and `M1-16` (budget tests) are still open; this document will be updated when they
  land.
- `M1-22` (this checklist run end to end with evidence) is the next step after this document
  exists.
- No external audit has happened yet. Everything above is internal verification, not independent
  third-party review.
