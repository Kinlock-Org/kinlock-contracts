//! `create_lock` success and one failing test per validation. Rows: M1-05, M1-11.

use super::{Setup, DAY, T0};
use crate::constants::{
    EVENT_SCHEMA_VERSION, MAX_LOCK_DURATION_SECS, MAX_TRANCHES, MIN_AMOUNT, MIN_EXPIRY_AHEAD_SECS,
};
use crate::errors::Error;
use crate::events::LockCreated;
use crate::types::{LockState, PayeeStatus, TrancheInput};
use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{Address, BytesN, Event as _, Vec};

/// Everything needed to call `create_lock`, with a funded sender and an Active payee.
struct Ctx {
    s: Setup,
    sender: Address,
    token: Address,
    payee_id: BytesN<32>,
    payout: Address,
    attester: Address,
}

fn ctx() -> Ctx {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, payout) = s.register_payee(&attester, 1);
    let token = s.new_token();
    let sender = Address::generate(&s.env);
    s.mint(&token, &sender, 1_000 * MIN_AMOUNT);
    Ctx {
        s,
        sender,
        token,
        payee_id,
        payout,
        attester,
    }
}

impl Ctx {
    fn try_create(&self, tranches: &Vec<TrancheInput>, expires_at: u64) -> Result<u64, Error> {
        match self.s.client.try_create_lock(
            &self.sender,
            &self.token,
            &self.payee_id,
            tranches,
            &self.s.ref_hash(),
            &expires_at,
        ) {
            Ok(Ok(id)) => Ok(id),
            Err(Ok(e)) => Err(e),
            other => panic!("unexpected host result: {other:?}"),
        }
    }

    fn one(&self, amount: i128) -> Vec<TrancheInput> {
        self.s.tranches(&[(amount, T0 + DAY)])
    }
}

const EXPIRES: u64 = T0 + 10 * DAY;

// ----- success -----

#[test]
fn create_lock_stores_lock_and_pulls_funds() {
    let c = ctx();
    let before = c.s.balance(&c.token, &c.sender);
    let tranches =
        c.s.tranches(&[(MIN_AMOUNT, T0 + DAY), (2 * MIN_AMOUNT, T0 + 2 * DAY)]);

    let id = c.try_create(&tranches, EXPIRES).unwrap();

    assert_eq!(id, 1);
    let lock = c.s.lock(id);
    assert_eq!(lock.sender, c.sender);
    assert_eq!(lock.payee_id, c.payee_id);
    assert_eq!(lock.payout, c.payout);
    assert_eq!(lock.token, c.token);
    assert_eq!(lock.total, 3 * MIN_AMOUNT);
    assert_eq!(lock.released, 0);
    assert_eq!(lock.returned, 0);
    assert_eq!(lock.ref_hash, c.s.ref_hash());
    assert_eq!(lock.expires_at, EXPIRES);
    assert_eq!(lock.state, LockState::Open);
    assert_eq!(lock.created_at, T0);
    assert_eq!(lock.tranches.len(), 2);
    for t in lock.tranches.iter() {
        assert!(!t.released);
    }

    assert_eq!(c.s.balance(&c.token, &c.sender), before - 3 * MIN_AMOUNT);
    assert_eq!(c.s.balance(&c.token, &c.s.client.address), 3 * MIN_AMOUNT);
    let config = c.s.config();
    assert_eq!(config.total_locked, 3 * MIN_AMOUNT);
    assert_eq!(config.next_lock_id, 2);
}

#[test]
fn lock_ids_increment() {
    let c = ctx();
    assert_eq!(c.try_create(&c.one(MIN_AMOUNT), EXPIRES), Ok(1));
    assert_eq!(c.try_create(&c.one(MIN_AMOUNT), EXPIRES), Ok(2));
    assert_eq!(c.s.config().total_locked, 2 * MIN_AMOUNT);
}

#[test]
fn create_lock_emits_event() {
    let c = ctx();
    let tranches = c.s.tranches(&[(MIN_AMOUNT, T0), (MIN_AMOUNT, T0 + DAY)]);
    let id = c.try_create(&tranches, EXPIRES).unwrap();
    let expected = LockCreated {
        id,
        schema_version: EVENT_SCHEMA_VERSION,
        sender: c.sender.clone(),
        payee_id: c.payee_id.clone(),
        payout: c.payout.clone(),
        token: c.token.clone(),
        total: 2 * MIN_AMOUNT,
        ref_hash: c.s.ref_hash(),
        expires_at: EXPIRES,
        tranche_count: 2,
    };
    assert_eq!(
        c.s.env
            .events()
            .all()
            .filter_by_contract(&c.s.client.address),
        [expected.to_xdr(&c.s.env, &c.s.client.address)]
    );
}

#[test]
fn accepts_boundary_values() {
    let c = ctx();
    // Expiry exactly MIN_EXPIRY_AHEAD and exactly MAX_LOCK_DURATION ahead.
    assert!(c
        .try_create(
            &c.s.tranches(&[(MIN_AMOUNT, T0)]),
            T0 + MIN_EXPIRY_AHEAD_SECS
        )
        .is_ok());
    assert!(c
        .try_create(&c.one(MIN_AMOUNT), T0 + MAX_LOCK_DURATION_SECS)
        .is_ok());
    // unlock_at == expires_at, equal unlock times, MAX_TRANCHES tranches, total == MIN_AMOUNT.
    let tranches =
        c.s.tranches(&[(MIN_AMOUNT / 2, EXPIRES), (MIN_AMOUNT / 2, EXPIRES)]);
    assert!(c.try_create(&tranches, EXPIRES).is_ok());
    let spec: std::vec::Vec<(i128, u64)> = (0..MAX_TRANCHES)
        .map(|i| (MIN_AMOUNT, T0 + u64::from(i)))
        .collect();
    assert!(c.try_create(&c.s.tranches(&spec), EXPIRES).is_ok());
}

// ----- one failing test per validation -----

#[test]
fn rejects_when_paused() {
    let c = ctx();
    c.s.client.set_paused_new_locks(&true);
    assert_eq!(
        c.try_create(&c.one(MIN_AMOUNT), EXPIRES),
        Err(Error::PausedNewLocks)
    );
}

#[test]
fn rejects_token_not_allowlisted() {
    let c = ctx();
    c.s.client.remove_token(&c.token);
    assert_eq!(
        c.try_create(&c.one(MIN_AMOUNT), EXPIRES),
        Err(Error::TokenNotAllowed)
    );
}

#[test]
fn rejects_unknown_payee() {
    let mut c = ctx();
    c.payee_id = BytesN::from_array(&c.s.env, &[99; 32]);
    assert_eq!(
        c.try_create(&c.one(MIN_AMOUNT), EXPIRES),
        Err(Error::PayeeNotFound)
    );
}

#[test]
fn rejects_inactive_payee() {
    for status in [PayeeStatus::Suspended, PayeeStatus::Revoked] {
        let c = ctx();
        c.s.client.set_status(&c.attester, &c.payee_id, &status);
        assert_eq!(
            c.try_create(&c.one(MIN_AMOUNT), EXPIRES),
            Err(Error::PayeeNotActive),
            "{status:?}"
        );
    }
}

#[test]
fn rejects_bad_tranche_count() {
    let c = ctx();
    assert_eq!(
        c.try_create(&c.s.tranches(&[]), EXPIRES),
        Err(Error::InvalidTrancheCount)
    );
    let spec: std::vec::Vec<(i128, u64)> = (0..=MAX_TRANCHES)
        .map(|i| (MIN_AMOUNT, T0 + u64::from(i)))
        .collect();
    assert_eq!(
        c.try_create(&c.s.tranches(&spec), EXPIRES),
        Err(Error::InvalidTrancheCount)
    );
}

#[test]
fn rejects_zero_or_negative_tranche() {
    let c = ctx();
    for bad in [0, -1, i128::MIN] {
        let tranches = c.s.tranches(&[(MIN_AMOUNT, T0), (bad, T0 + DAY)]);
        assert_eq!(
            c.try_create(&tranches, EXPIRES),
            Err(Error::InvalidTrancheAmount),
            "{bad}"
        );
    }
}

#[test]
fn rejects_unlock_out_of_order() {
    let c = ctx();
    let tranches =
        c.s.tranches(&[(MIN_AMOUNT, T0 + 2 * DAY), (MIN_AMOUNT, T0 + DAY)]);
    assert_eq!(
        c.try_create(&tranches, EXPIRES),
        Err(Error::UnlockOutOfOrder)
    );
}

#[test]
fn rejects_unlock_after_expiry() {
    let c = ctx();
    let tranches = c.s.tranches(&[(MIN_AMOUNT, EXPIRES + 1)]);
    assert_eq!(
        c.try_create(&tranches, EXPIRES),
        Err(Error::UnlockAfterExpiry)
    );
}

#[test]
fn rejects_expiry_too_soon() {
    let c = ctx();
    assert_eq!(
        c.try_create(
            &c.s.tranches(&[(MIN_AMOUNT, T0)]),
            T0 + MIN_EXPIRY_AHEAD_SECS - 1
        ),
        Err(Error::ExpiryTooSoon)
    );
}

#[test]
fn rejects_expiry_too_far() {
    let c = ctx();
    assert_eq!(
        c.try_create(&c.one(MIN_AMOUNT), T0 + MAX_LOCK_DURATION_SECS + 1),
        Err(Error::ExpiryTooFar)
    );
}

#[test]
fn rejects_total_below_minimum() {
    let c = ctx();
    let tranches =
        c.s.tranches(&[(MIN_AMOUNT / 2, T0), (MIN_AMOUNT / 2 - 1, T0)]);
    assert_eq!(
        c.try_create(&tranches, EXPIRES),
        Err(Error::AmountBelowMinimum)
    );
}

#[test]
fn rejects_total_above_lock_cap() {
    let c = ctx();
    c.s.client.set_caps(&(2 * MIN_AMOUNT), &(100 * MIN_AMOUNT));
    assert!(c.try_create(&c.one(2 * MIN_AMOUNT), EXPIRES).is_ok());
    assert_eq!(
        c.try_create(&c.one(2 * MIN_AMOUNT + 1), EXPIRES),
        Err(Error::AmountAboveLockCap)
    );
}

#[test]
fn rejects_global_cap_breach() {
    let c = ctx();
    c.s.client.set_caps(&(3 * MIN_AMOUNT), &(5 * MIN_AMOUNT));
    assert!(c.try_create(&c.one(3 * MIN_AMOUNT), EXPIRES).is_ok());
    assert!(c.try_create(&c.one(2 * MIN_AMOUNT), EXPIRES).is_ok());
    assert_eq!(
        c.try_create(&c.one(1), EXPIRES),
        Err(Error::AmountBelowMinimum)
    );
    assert_eq!(
        c.try_create(&c.one(MIN_AMOUNT), EXPIRES),
        Err(Error::GlobalCapExceeded)
    );
}

#[test]
fn rejects_sender_equal_to_payout() {
    let mut c = ctx();
    c.sender = c.payout.clone();
    c.s.mint(&c.token, &c.sender, 10 * MIN_AMOUNT);
    assert_eq!(
        c.try_create(&c.one(MIN_AMOUNT), EXPIRES),
        Err(Error::SenderIsPayout)
    );
}

#[test]
fn rejects_tranche_sum_overflow() {
    let c = ctx();
    let tranches = c.s.tranches(&[(i128::MAX, T0), (1, T0)]);
    assert_eq!(c.try_create(&tranches, EXPIRES), Err(Error::Overflow));
}

#[test]
fn rejects_lock_the_network_cant_keep_alive() {
    let c = ctx();
    // Shrink the network's max TTL below what a 10-day lock + 30-day grace needs.
    c.s.env.ledger().set_max_entry_ttl(10 * 17_280);
    assert_eq!(
        c.try_create(&c.one(MIN_AMOUNT), EXPIRES),
        Err(Error::LockTtlTooLong)
    );
    assert_eq!(c.s.config().next_lock_id, 1);
}

// ----- failed validation and failed transfer change nothing -----

#[test]
fn rejected_lock_changes_nothing() {
    let c = ctx();
    let before = c.s.balance(&c.token, &c.sender);
    let _ = c.try_create(&c.s.tranches(&[(MIN_AMOUNT, EXPIRES + 1)]), EXPIRES);
    let config = c.s.config();
    assert_eq!(config.next_lock_id, 1);
    assert_eq!(config.total_locked, 0);
    assert_eq!(c.s.balance(&c.token, &c.sender), before);
    assert_eq!(c.s.client.try_get_lock(&1), Err(Ok(Error::LockNotFound)));
}

#[test]
fn insufficient_balance_reverts_everything() {
    let c = ctx();
    let poor = Address::generate(&c.s.env);
    c.s.mint(&c.token, &poor, MIN_AMOUNT - 1);
    let result = c.s.client.try_create_lock(
        &poor,
        &c.token,
        &c.payee_id,
        &c.one(MIN_AMOUNT),
        &c.s.ref_hash(),
        &EXPIRES,
    );
    // The token contract's own error code surfaces here and collides with Kinlock's codes
    // (the SAC's balance error 10 decodes as `PayeeAlreadyExists`), so only assert failure.
    assert!(result.is_err());
    let config = c.s.config();
    assert_eq!(config.next_lock_id, 1);
    assert_eq!(config.total_locked, 0);
    assert_eq!(c.s.balance(&c.token, &poor), MIN_AMOUNT - 1);
    assert_eq!(c.s.client.try_get_lock(&1), Err(Ok(Error::LockNotFound)));
}

/// The longest lock allowed fits the network's storage limit, including the grace period,
/// and the documented-but-unworkable old maximum doesn't.
#[test]
fn max_duration_fits_the_network_ttl() {
    use crate::constants::TTL_GRACE_SECS;
    use crate::storage::secs_to_ledgers;
    let c = ctx();
    let max_ttl =
        c.s.env
            .as_contract(&c.s.client.address, || c.s.env.storage().max_ttl());
    // Same formula as `storage::extend_persistent_until`.
    let needed = |secs: u64| secs_to_ledgers(secs).saturating_add(1);
    assert!(needed(MAX_LOCK_DURATION_SECS + TTL_GRACE_SECS) <= max_ttl);
    assert!(c
        .try_create(&c.one(MIN_AMOUNT), T0 + MAX_LOCK_DURATION_SECS)
        .is_ok());
    assert!(needed(150 * DAY + TTL_GRACE_SECS) > max_ttl);
}
