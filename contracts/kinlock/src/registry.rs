//! Registry entry points. Spec: `docs/ARCHITECTURE.md` §4.3. Row: M1-04.
//!
//! Planned (not yet implemented):
//! - `register_payee(payee_id, payout, category, meta_hash)`, auth: attester. Creates Active payee.
//! - `set_status(payee_id, status)`, auth: vouching attester or admin.
//!   Active ⇄ Suspended; Revoked is terminal. Updates `status_changed_at`.
//! - `update_payout(payee_id, new_payout)`, auth: vouching attester. NEW locks only. Emits event.
//!
//! Country and attester-country scope are NOT checked here: they are registry-CI policy.
