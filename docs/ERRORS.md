> Synced from Kinlock-Org/.github. Do not edit here.

# Kinlock Contract Errors Reference

This document is the authoritative specification for all error codes emitted by the `kinlock` Soroban smart contract.

Error codes are non-contiguous numerical values returned via Soroban's `#[contracterror]` attribute (`Error` enum in `contracts/kinlock/src/errors.rs`). Numerical codes form a permanent part of the public interface: **they are never renumbered or reused**.

## Retired Error Codes

The following error codes were retired prior to public deployment and are never returned by the contract. They are reserved to prevent accidental reuse:
- **`1` (`AlreadyInitialized`)**: Retired in ADR-0018 when contract initialization moved into `__constructor`.
- **`27` (`TrancheSumMismatch`)**: Retired when the lock's `total` amount was derived directly from the sum of its tranches rather than accepted as a separate input parameter.

---

## Active Error Codes Catalog

Total active error variants: **32**.

| Code | Variant Name | Category | Firing Condition | Caller Guidance | Terminal for Lock |
| :---: | :--- | :--- | :--- | :--- | :---: |
| **2** | `NotInitialized` | Setup & Admin | Configuration has not been written (`config` missing in instance storage). | Contract deployer must initialize the contract via `__constructor` before invoking entry points. | N/A (no lock exists) |
| **3** | `NotAttester` | Setup & Admin | Caller address is not present on the authorized attester roster. | Ensure the caller keypair is authorized by the contract admin via `add_attester`. | N/A (no lock exists) |
| **4** | `NotVouchingAttester` | Setup & Admin | Caller is on the attester roster but is neither the original vouching attester that registered this specific payee nor the contract admin. | The operation (`update_payout`, `suspend_payee`, `reactivate_payee`) must be signed by the specific attester who vouched for the payee, or by the admin. | N/A (no lock exists) |
| **5** | `InvalidCap` | Setup & Admin | Admin called `set_caps` with invalid numeric parameters (`min_amount <= 0`, `max_per_lock < min_amount`, or `max_global < max_per_lock`). | Pass valid non-zero amounts satisfying: `0 < min_amount <= max_per_lock <= max_global`. | N/A (no lock exists) |
| **10** | `PayeeAlreadyExists` | Registry | `register_payee` called with a `payee_id` that is already registered in contract storage. | Verify the payee ID. If the payee is already registered, use update functions (`update_payout`, `set_status`) instead. | N/A (no lock exists) |
| **11** | `PayeeNotFound` | Registry | Target `payee_id` does not exist in registry storage during registry management or `create_lock`. | Verify the 32-byte `payee_id` against the official registry directory before invoking. | No (lock creation reverted) |
| **12** | `PayeeNotActive` | Registry / Vault | Payee status is `Suspended` or `Revoked` during `create_lock`, or not `Active` during `release`. | On `create_lock`: cannot lock funds to an inactive payee; wait for reactivation. On `release`: payout is halted; if suspended, wait for reactivation; if revoked or grace expires, sender can refund. | No (temporary while suspended; sender can refund if revoked or grace expires) |
| **13** | `InvalidStatusTransition` | Registry | Status change is disallowed: attempting to reactivate a `Revoked` payee, or transitioning to the status the payee already holds. | `Revoked` is permanent and terminal. `Active` and `Suspended` payees may transition between each other. | N/A (registry level) |
| **14** | `PayoutUnchanged` | Registry | `update_payout` called with a `new_payout` identical to the current payout address. | No update required; supply a different payout address if a rotation is intended. | N/A (registry level) |
| **15** | `PayeeRevoked` | Registry | `update_payout` attempted on a payee whose status is `Revoked`. | Revoked payees are permanently frozen; their payout destination cannot be updated. | N/A (registry level) |
| **16** | `InvalidPayout` | Registry | `update_payout` or `register_payee` provides the Kinlock contract address itself as the payee payout address. | Provide a valid external account or recipient contract address; funds cannot be routed to Kinlock itself. | N/A (registry level) |
| **20** | `PausedNewLocks` | Lock Creation | `create_lock` called while `paused_new_locks` is active in contract configuration. | Wait until admin clears the pause flag. Existing open locks remain fully operational (release, refund, decline unaffected). | No (lock not created) |
| **21** | `TokenNotAllowed` | Lock Creation | `create_lock` called with a token address not present on the admin allowlist. | Use an approved asset (e.g. allowlisted Circle USDC SAC address). | No (lock not created) |
| **22** | `AmountBelowMinimum` | Lock Creation | Sum of all tranche amounts in `create_lock` is less than `config.min_amount`. | Increase the total lock amount to meet or exceed the minimum threshold (e.g. 1 USDC = 10,000,000 stroops). | No (lock not created) |
| **23** | `AmountAboveLockCap` | Lock Creation | Sum of all tranche amounts in `create_lock` exceeds `config.max_per_lock`. | Reduce the lock amount or split the transfer into multiple distinct locks under the cap. | No (lock not created) |
| **24** | `GlobalCapExceeded` | Lock Creation | Total lock amount would cause aggregate `total_locked` across all open locks to exceed `config.max_global`. | Wait for existing locks to complete (release/refund/decline) or for admin to increase the global cap. | No (lock not created) |
| **25** | `InvalidTrancheCount` | Lock Creation | `tranches` vector in `create_lock` is empty (`len == 0`) or exceeds `MAX_TRANCHES` (12). | Supply between 1 and 12 tranche specifications. | No (lock not created) |
| **26** | `InvalidTrancheAmount` | Lock Creation | At least one tranche in `create_lock` specifies an `amount <= 0`. | Ensure every tranche amount is a strictly positive integer in base units. | No (lock not created) |
| **28** | `UnlockAfterExpiry` | Lock Creation | A tranche specifies an `unlock_time > expires_at`. | Set all tranche unlock timestamps to be less than or equal to the overall lock `expires_at`. | No (lock not created) |
| **29** | `UnlockOutOfOrder` | Lock Creation | Tranche unlock timestamps decrease (`tranches[i].unlock_time < tranches[i-1].unlock_time`). | Order tranches monotonically non-decreasing by `unlock_time`. | No (lock not created) |
| **30** | `ExpiryTooSoon` | Lock Creation | Lock expiry timestamp is less than `now + MIN_LOCK_DURATION_SECS` (1 hour / 3,600s). | Set `expires_at` to at least 1 hour into the future from the current ledger timestamp. | No (lock not created) |
| **31** | `ExpiryTooFar` | Lock Creation | Lock expiry timestamp exceeds `now + MAX_LOCK_DURATION_SECS` (149 days / 12,873,600s, ADR-0020). | Set `expires_at` within 149 days of the current ledger timestamp. | No (lock not created) |
| **32** | `SenderIsPayout` | Lock Creation | Sender address in `create_lock` matches the payee's current payout address. | Senders cannot lock funds to themselves; verify the target payee identity. | No (lock not created) |
| **33** | `LockTtlTooLong` | Lock Creation | Lock duration plus TTL grace period exceeds the network maximum entry TTL. | Shorten the lock duration so storage rent can be sustained within Soroban network limits. | No (lock not created) |
| **40** | `LockNotFound` | Lock Actions | Looked up `lock_id` does not exist in storage during `release`, `refund`, `decline`, `bump_lock`, or `get_lock`. | Verify the `lock_id` (u64); check indexer or explorer to ensure transaction was confirmed. | Call failed (lock absent) |
| **41** | `LockNotOpen` | Lock Actions | Action attempted on a lock whose status is not `Open` (already `Completed`, `Refunded`, or `Declined`). | The lock has reached a final state; inspect receipt or lock history. | Yes (already terminal) |
| **42** | `TrancheIndexOutOfRange` | Lock Actions | `release` called with `tranche_idx >= lock.tranches.len()`. | Pass a valid 0-based tranche index in the range `[0, tranche_count - 1]`. | No (retryable with correct index) |
| **43** | `TrancheAlreadyReleased` | Lock Actions | `release` called on a tranche where `released == true`. | This tranche has already been paid out to the payee; check remaining unreleased tranches. | No (other tranches unaffected) |
| **44** | `TrancheNotUnlocked` | Lock Actions | `release` called before the current ledger timestamp reaches `tranche.unlock_time`. | Wait until `ledger.timestamp >= tranche.unlock_time` before calling `release`. | No (retryable once unlocked) |
| **45** | `LockExpired` | Lock Actions | `release` called when `ledger.timestamp >= lock.expires_at`. | The lock has expired and can no longer be released to the payee; the sender should call `refund` to recover remaining funds. | Yes for release; lock transitions to refund path |
| **46** | `RefundNotAllowed` | Lock Actions | `refund` called when the lock has not expired and the payee is neither `Revoked` nor `Suspended` past the 14-day grace period (`SUSPENSION_REFUND_GRACE_SECS`). | Wait until the lock reaches `expires_at`, or until the payee is revoked, or until 14 days have elapsed since payee suspension. | No (retryable once conditions are met) |
| **50** | `Overflow` | Arithmetic | Mathematical overflow detected during arithmetic computation (e.g. tranche summation, balance addition/subtraction). | Check integer amounts; ensure values fit within standard positive `i128` scale without overflow. | No (safeguard revert) |
