//! Release window and boundaries: `now == unlock_at`, `expires_at - 1`, `expires_at`.
//! Rows: M1-06, M1-13.

use super::{Setup, DAY, T0};
use crate::constants::{EVENT_SCHEMA_VERSION, MIN_AMOUNT};
use crate::errors::Error;
use crate::events::Released;
use crate::types::{LockState, PayeeStatus};
use soroban_sdk::testutils::Events as _;
use soroban_sdk::Event as _;

const EXPIRES: u64 = T0 + 10 * DAY;

#[test]
fn release_at_unlock_time_pays_the_payout() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + DAY);

    s.client.release(&f.id, &0);

    assert_eq!(s.balance(&f.token, &f.payout), MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &s.client.address), 2 * MIN_AMOUNT);
    let lock = s.lock(f.id);
    assert_eq!(lock.released, MIN_AMOUNT);
    assert_eq!(lock.state, LockState::Open);
    assert!(lock.tranches.get(0).unwrap().released);
    assert!(!lock.tranches.get(1).unwrap().released);
    assert_eq!(s.config().total_locked, 2 * MIN_AMOUNT);
}

#[test]
fn release_emits_event() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + DAY);
    s.client.release(&f.id, &0);
    let expected = Released {
        id: f.id,
        schema_version: EVENT_SCHEMA_VERSION,
        idx: 0,
        amount: MIN_AMOUNT,
        payout: f.payout.clone(),
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
}

#[test]
fn release_before_unlock_fails() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + DAY - 1);
    assert_eq!(
        s.client.try_release(&f.id, &0),
        Err(Ok(Error::TrancheNotUnlocked))
    );
}

#[test]
fn release_one_second_before_expiry_succeeds() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(EXPIRES - 1);
    s.client.release(&f.id, &1);
    assert_eq!(s.balance(&f.token, &f.payout), 2 * MIN_AMOUNT);
}

#[test]
fn release_at_expiry_fails() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(EXPIRES);
    assert_eq!(s.client.try_release(&f.id, &0), Err(Ok(Error::LockExpired)));
    assert_eq!(s.balance(&f.token, &f.payout), 0);
}

#[test]
fn releasing_every_tranche_completes_the_lock() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + 2 * DAY);
    s.client.release(&f.id, &1);
    s.client.release(&f.id, &0);
    let lock = s.lock(f.id);
    assert_eq!(lock.state, LockState::Completed);
    assert_eq!(lock.released, lock.total);
    assert_eq!(s.balance(&f.token, &f.payout), 3 * MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &s.client.address), 0);
    assert_eq!(s.config().total_locked, 0);
    assert_eq!(s.client.try_release(&f.id, &0), Err(Ok(Error::LockNotOpen)));
}

#[test]
fn double_release_fails() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + DAY);
    s.client.release(&f.id, &0);
    assert_eq!(
        s.client.try_release(&f.id, &0),
        Err(Ok(Error::TrancheAlreadyReleased))
    );
    assert_eq!(s.balance(&f.token, &f.payout), MIN_AMOUNT);
}

#[test]
fn release_bad_index_or_lock() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + 2 * DAY);
    assert_eq!(
        s.client.try_release(&f.id, &2),
        Err(Ok(Error::TrancheIndexOutOfRange))
    );
    assert_eq!(s.client.try_release(&99, &0), Err(Ok(Error::LockNotFound)));
}

#[test]
fn release_blocked_while_payee_not_active() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + DAY);
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Suspended);
    assert_eq!(
        s.client.try_release(&f.id, &0),
        Err(Ok(Error::PayeeNotActive))
    );
    s.client
        .set_status(&f.attester, &f.payee_id, &PayeeStatus::Active);
    s.client.release(&f.id, &0);
    assert_eq!(s.balance(&f.token, &f.payout), MIN_AMOUNT);
}

/// Invariant 10: a failed transfer (payout account frozen by the issuer) reverts everything,
/// and the payee can retry once the account is fixed.
#[test]
fn frozen_payout_reverts_and_can_retry() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.set_time(T0 + DAY);
    s.set_authorized(&f.token, &f.payout, false);

    assert!(s.client.try_release(&f.id, &0).is_err());
    let lock = s.lock(f.id);
    assert!(!lock.tranches.get(0).unwrap().released);
    assert_eq!(lock.released, 0);
    assert_eq!(s.config().total_locked, 3 * MIN_AMOUNT);
    assert_eq!(s.balance(&f.token, &s.client.address), 3 * MIN_AMOUNT);

    s.set_authorized(&f.token, &f.payout, true);
    s.client.release(&f.id, &0);
    assert_eq!(s.balance(&f.token, &f.payout), MIN_AMOUNT);
}
