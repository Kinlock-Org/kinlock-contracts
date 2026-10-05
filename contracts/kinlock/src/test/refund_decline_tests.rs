//! Refund (expired, Revoked, Suspended+grace) and decline, incl. after partial release.
//! Rows: M1-07, M1-08, M1-13.

use super::{Setup, DAY, T0};
use crate::constants::{EVENT_SCHEMA_VERSION, MIN_AMOUNT, SUSPENSION_REFUND_GRACE_SECS};
use crate::errors::Error;
use crate::events::{Declined, Refunded};
use crate::types::{LockState, PayeeStatus, RefundReason};
use soroban_sdk::testutils::Events as _;
use soroban_sdk::Event as _;

const EXPIRES: u64 = T0 + 10 * DAY;

// ----- refund -----

#[test]
fn refund_not_allowed_before_expiry_while_active() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(EXPIRES - 1);
    assert_eq!(s.client.try_refund(&f.id), Err(Ok(Error::RefundNotAllowed)));
}

#[test]
fn refund_at_expiry_returns_everything() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(EXPIRES);
    s.client.refund(&f.id);
    let expected = Refunded {
        id: f.id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: 3 * MIN_AMOUNT,
        reason: RefundReason::Expired,
    };
    // Events are those of the last call, so check them before any further call.
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
    let lock = s.lock(f.id);
    assert_eq!(lock.state, LockState::Refunded);
    assert_eq!(lock.returned, 3 * MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &f.sender), 3 * MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &s.client.address), 0);
    assert_eq!(s.config().total_locked, 0);
}

#[test]
fn refund_immediately_when_payee_revoked() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Revoked);
    s.client.refund(&f.id);
    let expected = Refunded {
        id: f.id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: 3 * MIN_AMOUNT,
        reason: RefundReason::Revoked,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
    assert_eq!(s.lock(f.id).state, LockState::Refunded);
    assert_eq!(s.balance(&f.token, &f.sender), 3 * MIN_AMOUNT);
}

/// The 14-day grace needs a lock that outlives it, so this lock expires at T0 + 60 days.
#[test]
fn refund_after_suspension_grace_only() {
    let s = Setup::new();
    let f = s.open_lock(&[(MIN_AMOUNT, T0 + DAY)], T0 + 60 * DAY);
    let suspended_at = T0 + DAY;
    s.set_time(suspended_at);
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Suspended);

    s.set_time(suspended_at + SUSPENSION_REFUND_GRACE_SECS - 1);
    assert_eq!(s.client.try_refund(&f.id), Err(Ok(Error::RefundNotAllowed)));

    s.set_time(suspended_at + SUSPENSION_REFUND_GRACE_SECS);
    s.client.refund(&f.id);
    let expected = Refunded {
        id: f.id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: MIN_AMOUNT,
        reason: RefundReason::SuspendedTimeout,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
    assert_eq!(s.lock(f.id).state, LockState::Refunded);
    assert_eq!(s.balance(&f.token, &f.sender), MIN_AMOUNT);
}

#[test]
fn reactivation_restarts_the_suspension_clock() {
    let s = Setup::new();
    let f = s.open_lock(&[(MIN_AMOUNT, T0 + DAY)], T0 + 60 * DAY);
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Suspended);
    s.set_time(T0 + 10 * DAY);
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Active);
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Suspended);
    s.set_time(T0 + SUSPENSION_REFUND_GRACE_SECS);
    assert_eq!(s.client.try_refund(&f.id), Err(Ok(Error::RefundNotAllowed)));
}

#[test]
fn partial_release_then_refund_returns_only_remainder() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + DAY);
    s.client.release(&f.id, &0);
    s.set_time(EXPIRES);
    s.client.refund(&f.id);
    let lock = s.lock(f.id);
    assert_eq!(lock.released, MIN_AMOUNT);
    assert_eq!(lock.returned, 2 * MIN_AMOUNT);
    assert_eq!(lock.released + lock.returned, lock.total);
    assert_eq!(s.balance(&f.token, &f.payout), MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &f.sender), 2 * MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &s.client.address), 0);
    assert_eq!(s.config().total_locked, 0);
}

#[test]
fn refund_on_closed_or_missing_lock_fails() {
    let s = Setup::new();
    let f = s.standard_lock();
    let done = s.standard_lock();
    s.set_time(EXPIRES);
    s.client.refund(&f.id);
    assert_eq!(s.client.try_refund(&f.id), Err(Ok(Error::LockNotOpen)));
    assert_eq!(s.client.try_refund(&99), Err(Ok(Error::LockNotFound)));

    s.set_time(EXPIRES - 1);
    s.client.release(&done.id, &0);
    s.client.release(&done.id, &1);
    s.set_time(EXPIRES);
    assert_eq!(s.client.try_refund(&done.id), Err(Ok(Error::LockNotOpen)));
}

// ----- decline -----

#[test]
fn decline_returns_everything_to_sender() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.client.decline(&f.id);
    let expected = Declined {
        id: f.id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: 3 * MIN_AMOUNT,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
    let lock = s.lock(f.id);
    assert_eq!(lock.state, LockState::Declined);
    assert_eq!(lock.returned, 3 * MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &f.sender), 3 * MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &f.payout), 0);
    assert_eq!(s.config().total_locked, 0);
}

#[test]
fn partial_release_then_decline_returns_only_remainder() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + 2 * DAY);
    s.client.release(&f.id, &1);
    s.client.decline(&f.id);
    let lock = s.lock(f.id);
    assert_eq!(lock.released, 2 * MIN_AMOUNT);
    assert_eq!(lock.returned, MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &f.sender), MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &f.payout), 2 * MIN_AMOUNT);
    assert_eq!(s.config().total_locked, 0);
}

#[test]
fn decline_allowed_any_time_while_open() {
    // After expiry, and with the payee revoked: decline only ever returns funds to the sender.
    let s = Setup::new();
    let expired = s.standard_lock();
    let revoked = s.standard_lock();
    s.client
        .set_status(&revoked.attester, &revoked.payee_id, &PayeeStatus::Revoked);
    s.set_time(EXPIRES + DAY);
    s.client.decline(&expired.id);
    s.client.decline(&revoked.id);
    assert_eq!(s.lock(expired.id).state, LockState::Declined);
    assert_eq!(s.lock(revoked.id).state, LockState::Declined);
}

#[test]
fn decline_on_closed_lock_fails() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.client.decline(&f.id);
    assert_eq!(s.client.try_decline(&f.id), Err(Ok(Error::LockNotOpen)));
    assert_eq!(s.client.try_decline(&99), Err(Ok(Error::LockNotFound)));
}

#[test]
fn revoked_after_suspended_refunds_immediately() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Suspended);
    s.set_time(T0 + DAY);
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Revoked);
    s.client.refund(&f.id);
    let expected = Refunded {
        id: f.id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: 3 * MIN_AMOUNT,
        reason: RefundReason::Revoked,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
}

/// Invariant 10 on the refund and decline paths: if the sender's account is frozen the
/// transfer fails, nothing changes, and the call can be retried once it's unfrozen.
#[test]
fn frozen_sender_reverts_refund_and_decline() {
    let s = Setup::new();
    let refund_me = s.standard_lock();
    let decline_me = s.standard_lock();
    s.set_authorized(&refund_me.token, &refund_me.sender, false);
    s.set_authorized(&decline_me.token, &decline_me.sender, false);
    s.set_time(EXPIRES);

    assert!(s.client.try_refund(&refund_me.id).is_err());
    assert!(s.client.try_decline(&decline_me.id).is_err());
    for f in [&refund_me, &decline_me] {
        let lock = s.lock(f.id);
        assert_eq!(lock.state, LockState::Open);
        assert_eq!(lock.returned, 0);
        assert_eq!(s.balance(&f.token, &s.client.address), 3 * MIN_AMOUNT);
    }
    assert_eq!(s.config().total_locked, 6 * MIN_AMOUNT);

    s.set_authorized(&refund_me.token, &refund_me.sender, true);
    s.set_authorized(&decline_me.token, &decline_me.sender, true);
    s.client.refund(&refund_me.id);
    s.client.decline(&decline_me.id);
    assert_eq!(s.config().total_locked, 0);
}
