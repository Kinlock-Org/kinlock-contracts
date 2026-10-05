//! M1-31 and lifecycle: pause, allowlist removal, attester removal, and payout updates never
//! affect existing locks (invariants 7, 8); `total_locked` bookkeeping (invariant 9);
//! `bump_lock` (M1-09).

use super::{Setup, DAY, T0};
use crate::constants::{MIN_AMOUNT, TTL_GRACE_SECS};
use crate::errors::Error;
use crate::storage::{secs_to_ledgers, DataKey};
use crate::types::LockState;
use soroban_sdk::testutils::storage::Persistent as _;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::Address;

const EXPIRES: u64 = T0 + 10 * DAY;

#[test]
fn pause_blocks_only_new_locks() {
    let s = Setup::new();
    let release_me = s.standard_lock();
    let decline_me = s.standard_lock();
    let refund_me = s.standard_lock();
    s.client.set_paused_new_locks(&true);

    s.set_time(T0 + DAY);
    s.client.release(&release_me.id, &0);
    s.client.decline(&decline_me.id);
    s.set_time(EXPIRES);
    s.client.refund(&refund_me.id);

    assert_eq!(s.balance(&release_me.token, &release_me.payout), MIN_AMOUNT);
    assert_eq!(s.lock(decline_me.id).state, LockState::Declined);
    assert_eq!(s.lock(refund_me.id).state, LockState::Refunded);
    let sender = Address::generate(&s.env);
    let tranches = s.tranches(&[(MIN_AMOUNT, EXPIRES + DAY)]);
    assert_eq!(
        s.client.try_create_lock(
            &sender,
            &release_me.token,
            &release_me.payee_id,
            &tranches,
            &s.ref_hash(),
            &(EXPIRES + 2 * DAY)
        ),
        Err(Ok(Error::PausedNewLocks))
    );
}

#[test]
fn token_removal_blocks_only_new_locks() {
    let s = Setup::new();
    let release_me = s.standard_lock();
    let refund_me = s.standard_lock();
    s.client.remove_token(&release_me.token);
    s.client.remove_token(&refund_me.token);

    s.set_time(T0 + 2 * DAY);
    s.client.release(&release_me.id, &0);
    s.client.release(&release_me.id, &1);
    s.set_time(EXPIRES);
    s.client.refund(&refund_me.id);

    assert_eq!(s.lock(release_me.id).state, LockState::Completed);
    assert_eq!(s.lock(refund_me.id).state, LockState::Refunded);
}

#[test]
fn attester_removal_does_not_block_existing_locks() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.client.remove_attester(&f.attester);
    s.set_time(T0 + DAY);
    s.client.release(&f.id, &0);
    s.client.decline(&f.id);
    assert_eq!(s.lock(f.id).state, LockState::Declined);
}

/// Invariant 7: an existing lock keeps its payout snapshot; only new locks see the update.
#[test]
fn payout_update_affects_only_new_locks() {
    let s = Setup::new();
    let old = s.standard_lock();
    let new_payout = Address::generate(&s.env);
    s.client
        .update_payout(&old.attester, &old.payee_id, &new_payout);

    assert_eq!(s.lock(old.id).payout, old.payout);
    s.set_time(T0 + DAY);
    s.client.release(&old.id, &0);
    assert_eq!(s.balance(&old.token, &old.payout), MIN_AMOUNT);
    assert_eq!(s.balance(&old.token, &new_payout), 0);

    let sender = Address::generate(&s.env);
    s.mint(&old.token, &sender, MIN_AMOUNT);
    let new_id = s.client.create_lock(
        &sender,
        &old.token,
        &old.payee_id,
        &s.tranches(&[(MIN_AMOUNT, T0 + DAY)]),
        &s.ref_hash(),
        &EXPIRES,
    );
    assert_eq!(s.lock(new_id).payout, new_payout);
}

/// Invariant 9: `total_locked` always equals the sum of remainders of Open locks.
#[test]
fn total_locked_tracks_open_remainders() {
    let s = Setup::new();
    let a = s.standard_lock();
    let b = s.standard_lock();
    let c = s.open_lock(&[(5 * MIN_AMOUNT, T0 + DAY)], EXPIRES);
    let remainders = |s: &Setup| -> i128 {
        [a.id, b.id, c.id]
            .iter()
            .map(|id| s.lock(*id))
            .filter(|l| l.state == LockState::Open)
            .map(|l| l.total - l.released - l.returned)
            .sum()
    };
    assert_eq!(s.config().total_locked, 11 * MIN_AMOUNT);

    s.set_time(T0 + DAY);
    s.client.release(&a.id, &0);
    assert_eq!(s.config().total_locked, remainders(&s));
    s.client.decline(&b.id);
    assert_eq!(s.config().total_locked, remainders(&s));
    s.client.release(&c.id, &0);
    assert_eq!(s.config().total_locked, remainders(&s));
    s.set_time(EXPIRES);
    s.client.refund(&a.id);
    assert_eq!(s.config().total_locked, 0);
    assert_eq!(remainders(&s), 0);
}

// ----- bump_lock -----

/// The payee is registered long before the lock, so its TTL has decayed below what the lock
/// needs; `create_lock` must extend it (release and refund read the payee).
#[test]
fn create_lock_keeps_lock_and_payee_alive_past_expiry() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    let needed = secs_to_ledgers(EXPIRES + TTL_GRACE_SECS - T0);
    let ttl = |key: &DataKey| {
        s.env.as_contract(&s.client.address, || {
            s.env.storage().persistent().get_ttl(key)
        })
    };
    let payee_key = DataKey::Payee(payee_id.clone());
    let age = ttl(&payee_key) - needed / 2;
    s.env.ledger().with_mut(|l| l.sequence_number += age);
    assert!(ttl(&payee_key) < needed);

    let token = s.new_token();
    let sender = Address::generate(&s.env);
    s.mint(&token, &sender, MIN_AMOUNT);
    let id = s.client.create_lock(
        &sender,
        &token,
        &payee_id,
        &s.tranches(&[(MIN_AMOUNT, T0 + DAY)]),
        &s.ref_hash(),
        &EXPIRES,
    );

    assert!(ttl(&DataKey::Lock(id)) >= needed);
    assert!(ttl(&payee_key) >= needed);
}

#[test]
fn bump_lock_restores_ttl_without_any_signature() {
    let s = Setup::new();
    let f = s.standard_lock();
    let key = DataKey::Lock(f.id);
    let ttl = || {
        s.env.as_contract(&s.client.address, || {
            s.env.storage().persistent().get_ttl(&key)
        })
    };
    let initial = ttl();
    s.env.ledger().with_mut(|l| l.sequence_number += 100_000);
    assert_eq!(ttl(), initial - 100_000);

    s.env.set_auths(&[]);
    s.client.bump_lock(&f.id);
    assert_eq!(ttl(), initial);
    assert!(s.env.auths().is_empty());
}

#[test]
fn bump_lock_rejects_closed_or_missing_lock() {
    let s = Setup::new();
    let f = s.standard_lock();
    s.client.decline(&f.id);
    assert_eq!(s.client.try_bump_lock(&f.id), Err(Ok(Error::LockNotOpen)));
    assert_eq!(s.client.try_bump_lock(&99), Err(Ok(Error::LockNotFound)));
}
