//! Boundary timestamp tests for exact `unlock_at` and `expires_at` ledger times.
//! Issue #58; Rows: M1-06, M1-07, M1-13.
//!
//! Hard rule 8 states boundary semantics are fixed:
//! - release succeeds only if `unlock_at <= now < expires_at`
//! - refund succeeds only if `now >= expires_at` (or payee Revoked/Suspended past grace)
//!
//! These tests prove that no off-by-one second discrepancy exists on lock release
//! or refund windows at exact seconds:
//! - `now == unlock_at - 1` -> release fails with TrancheNotUnlocked
//! - `now == unlock_at` -> release succeeds
//! - `now == expires_at - 1` -> release succeeds, refund fails with RefundNotAllowed
//! - `now == expires_at` -> release fails with LockExpired, refund succeeds with Expired

use super::{Setup, DAY, T0};
use crate::constants::{EVENT_SCHEMA_VERSION, MIN_AMOUNT, MIN_EXPIRY_AHEAD_SECS};
use crate::errors::Error;
use crate::events::Refunded;
use crate::types::{LockState, RefundReason};
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Address, Event as _};

#[test]
fn exact_boundary_unlock_at_second_transitions() {
    let s = Setup::new();
    let unlock_at = T0 + 2 * MIN_EXPIRY_AHEAD_SECS;
    let expires_at = T0 + 10 * DAY;

    // Single tranche lock unlocking at `unlock_at` and expiring at `expires_at`.
    let f = s.open_lock(&[(MIN_AMOUNT, unlock_at)], expires_at);

    // 1 second before unlock_at: release must fail with TrancheNotUnlocked.
    s.set_time(unlock_at - 1);
    assert_eq!(
        s.client.try_release(&f.id, &0),
        Err(Ok(Error::TrancheNotUnlocked))
    );
    assert_eq!(s.balance(&f.token, &f.payout), 0);

    // Exactly at unlock_at: release must succeed.
    s.set_time(unlock_at);
    s.client.release(&f.id, &0);
    assert_eq!(s.balance(&f.token, &f.payout), MIN_AMOUNT);

    let lock = s.lock(f.id);
    assert_eq!(lock.released, MIN_AMOUNT);
    assert_eq!(lock.state, LockState::Completed);
}

#[test]
fn exact_boundary_expires_at_second_transitions_payee_lock() {
    let s = Setup::new();
    let unlock_at = T0 + MIN_EXPIRY_AHEAD_SECS;
    let expires_at = T0 + 5 * DAY;

    // Create both locks at initial time T0.
    let f1 = s.open_lock(&[(2 * MIN_AMOUNT, unlock_at)], expires_at);
    let f2 = s.open_lock(&[(3 * MIN_AMOUNT, unlock_at)], expires_at);

    // At expires_at - 1:
    // 1) release must succeed.
    // 2) refund must fail with RefundNotAllowed (since now < expires_at and payee is Active).
    s.set_time(expires_at - 1);
    assert_eq!(
        s.client.try_refund(&f1.id),
        Err(Ok(Error::RefundNotAllowed))
    );

    // Release executes cleanly at expires_at - 1.
    s.client.release(&f1.id, &0);
    assert_eq!(s.balance(&f1.token, &f1.payout), 2 * MIN_AMOUNT);
    assert_eq!(s.lock(f1.id).state, LockState::Completed);

    // Advance to exactly `expires_at`.
    s.set_time(expires_at);

    // At now == expires_at:
    // 1) release must fail with LockExpired.
    // 2) refund must succeed with RefundReason::Expired.
    assert_eq!(
        s.client.try_release(&f2.id, &0),
        Err(Ok(Error::LockExpired))
    );
    assert_eq!(s.balance(&f2.token, &f2.payout), 0);

    s.client.refund(&f2.id);

    let expected = Refunded {
        id: f2.id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: 3 * MIN_AMOUNT,
        reason: RefundReason::Expired,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );

    let lock2 = s.lock(f2.id);
    assert_eq!(lock2.state, LockState::Refunded);
    assert_eq!(lock2.returned, 3 * MIN_AMOUNT);
    assert_eq!(s.balance(&f2.token, &f2.sender), 3 * MIN_AMOUNT);
}

#[test]
fn exact_boundary_two_tranches_last_second_release_and_expiry() {
    let s = Setup::new();
    let unlock_0 = T0 + DAY;
    let unlock_1 = T0 + 2 * DAY;
    let expires_at = T0 + 3 * DAY;

    // Create both locks at initial time T0.
    let f1 = s.open_lock(
        &[(MIN_AMOUNT, unlock_0), (2 * MIN_AMOUNT, unlock_1)],
        expires_at,
    );
    let f2 = s.open_lock(
        &[(MIN_AMOUNT, unlock_0), (2 * MIN_AMOUNT, unlock_1)],
        expires_at,
    );

    // Tranche 0 released on both locks at unlock_0.
    s.set_time(unlock_0);
    s.client.release(&f1.id, &0);
    s.client.release(&f2.id, &0);
    assert_eq!(s.balance(&f1.token, &f1.payout), MIN_AMOUNT);
    assert_eq!(s.balance(&f2.token, &f2.payout), MIN_AMOUNT);

    // Advance to exactly expires_at - 1.
    s.set_time(expires_at - 1);
    // Refund is still forbidden on f1.
    assert_eq!(
        s.client.try_refund(&f1.id),
        Err(Ok(Error::RefundNotAllowed))
    );

    // Tranche 1 release succeeds at the last available second.
    s.client.release(&f1.id, &1);
    assert_eq!(s.balance(&f1.token, &f1.payout), 3 * MIN_AMOUNT);
    assert_eq!(s.lock(f1.id).state, LockState::Completed);

    // Advance to exactly expires_at.
    s.set_time(expires_at);

    // Tranche 1 release on f2 now fails.
    assert_eq!(
        s.client.try_release(&f2.id, &1),
        Err(Ok(Error::LockExpired))
    );

    // Refund on f2 now returns only the unreleased remainder (2 * MIN_AMOUNT) to the sender.
    s.client.refund(&f2.id);
    let lock2 = s.lock(f2.id);
    assert_eq!(lock2.state, LockState::Refunded);
    assert_eq!(lock2.released, MIN_AMOUNT);
    assert_eq!(lock2.returned, 2 * MIN_AMOUNT);
    assert_eq!(s.balance(&f2.token, &f2.sender), 2 * MIN_AMOUNT);
}

#[test]
fn exact_boundary_direct_lock_release_and_expiry() {
    let s = Setup::new();
    let unlock_at = T0 + 2 * DAY;
    let expires_at = T0 + 4 * DAY;
    let sender = Address::generate(&s.env);
    let recipient = Address::generate(&s.env);
    let token = s.new_token();
    s.mint(&token, &sender, MIN_AMOUNT);

    let id = s.client.create_direct_lock(
        &sender,
        &token,
        &recipient,
        &s.tranches(&[(MIN_AMOUNT, unlock_at)]),
        &s.ref_hash(),
        &expires_at,
    );

    // 1 second before unlock: fails.
    s.set_time(unlock_at - 1);
    assert_eq!(
        s.client.try_release(&id, &0),
        Err(Ok(Error::TrancheNotUnlocked))
    );

    // Exactly at expires_at - 1: release succeeds.
    s.set_time(expires_at - 1);
    s.client.release(&id, &0);
    assert_eq!(s.balance(&token, &recipient), MIN_AMOUNT);
    assert_eq!(s.lock(id).state, LockState::Completed);
}
