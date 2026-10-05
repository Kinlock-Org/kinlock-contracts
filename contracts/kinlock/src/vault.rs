//! Vault entry points. Spec: `docs/ARCHITECTURE.md` §4.3, §4.5, §4.6. Rows: M1-05..M1-09.
//!
//! Planned (not yet implemented):
//! - `create_lock(sender, token, payee_id, tranches, ref_hash, expires_at) -> u64`, auth: sender.
//!   Validates, snapshots payout, pulls `total`, stores lock, extends TTL, emits `LockCreated`.
//! - `release(lock_id, idx)`, auth: `lock.payout`. Open, payee Active,
//!   `unlock_at <= now < expires_at`, tranche not released. State BEFORE transfer.
//! - `refund(lock_id)`, auth: sender. Open and (expired | payee Revoked |
//!   payee Suspended and `now >= status_changed_at + SUSPENSION_REFUND_GRACE_SECS`).
//! - `decline(lock_id)`, auth: `lock.payout`. Any time while Open; remainder to sender.
//! - `bump_lock(lock_id)`, permissionless TTL extension for Open locks.
//! - `get_lock(lock_id)`, `get_payee(payee_id)`: reads.
//!
//! Funds may leave only to `lock.payout` or `lock.sender` (invariant 3).
