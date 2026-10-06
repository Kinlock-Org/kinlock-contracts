//! Admin entry points: constructor, roster, allowlist, pause, caps. Row: M1-03.

use super::Setup;
use crate::constants::{MIN_AMOUNT, STORAGE_VERSION};
use crate::errors::Error;
use crate::storage;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

#[test]
fn constructor_sets_initial_config() {
    let s = Setup::new();
    let c = s.config();
    assert_eq!(c.admin, s.admin);
    assert_eq!(c.next_lock_id, 1);
    assert!(!c.paused_new_locks);
    assert_eq!(c.max_lock_amount, i128::MAX);
    assert_eq!(c.max_total_locked, i128::MAX);
    assert_eq!(c.total_locked, 0);
}

#[test]
fn add_and_remove_attester() {
    let s = Setup::new();
    let attester = Address::generate(&s.env);
    assert!(!s.is_attester(&attester));
    s.client.add_attester(&attester);
    assert!(s.is_attester(&attester));
    s.client.remove_attester(&attester);
    assert!(!s.is_attester(&attester));
}

#[test]
fn remove_unknown_attester_is_a_no_op() {
    let s = Setup::new();
    let attester = Address::generate(&s.env);
    s.client.remove_attester(&attester);
    assert!(!s.is_attester(&attester));
}

#[test]
fn add_and_remove_token() {
    let s = Setup::new();
    let token = Address::generate(&s.env);
    assert!(!s.is_token_allowed(&token));
    s.client.add_token(&token);
    assert!(s.is_token_allowed(&token));
    s.client.remove_token(&token);
    assert!(!s.is_token_allowed(&token));
}

#[test]
fn pause_and_unpause_new_locks() {
    let s = Setup::new();
    s.client.set_paused_new_locks(&true);
    assert!(s.config().paused_new_locks);
    s.client.set_paused_new_locks(&false);
    assert!(!s.config().paused_new_locks);
}

#[test]
fn set_caps_updates_config_only() {
    let s = Setup::new();
    s.client.set_caps(&(5 * MIN_AMOUNT), &(20 * MIN_AMOUNT));
    let c = s.config();
    assert_eq!(c.max_lock_amount, 5 * MIN_AMOUNT);
    assert_eq!(c.max_total_locked, 20 * MIN_AMOUNT);
    assert_eq!(c.total_locked, 0);
    assert!(!c.paused_new_locks);
}

#[test]
fn set_caps_accepts_boundaries() {
    let s = Setup::new();
    s.client.set_caps(&MIN_AMOUNT, &MIN_AMOUNT);
    assert_eq!(s.config().max_lock_amount, MIN_AMOUNT);
}

#[test]
fn set_caps_rejects_invalid_values() {
    let s = Setup::new();
    let cases = [
        (0, 5 * MIN_AMOUNT),
        (-1, 5 * MIN_AMOUNT),
        (MIN_AMOUNT - 1, 5 * MIN_AMOUNT),
        (MIN_AMOUNT, MIN_AMOUNT - 1),
        (5 * MIN_AMOUNT, 4 * MIN_AMOUNT),
        (MIN_AMOUNT, -1),
    ];
    for (max_lock, max_total) in cases {
        assert_eq!(
            s.client.try_set_caps(&max_lock, &max_total),
            Err(Ok(Error::InvalidCap)),
            "{max_lock}, {max_total}"
        );
    }
    assert_eq!(s.config().max_lock_amount, i128::MAX);
}

#[test]
fn constructor_writes_storage_version() {
    let s = Setup::new();
    let v = s
        .env
        .as_contract(&s.client.address, || storage::read_storage_version(&s.env));
    assert_eq!(v, Some(STORAGE_VERSION));
}

/// Every entry point that needs the config fails cleanly if it's missing.
#[test]
fn missing_config_fails_with_not_initialized() {
    let s = Setup::new();
    s.env.as_contract(&s.client.address, || {
        s.env.storage().instance().remove(&storage::DataKey::Config)
    });
    let a = Address::generate(&s.env);
    assert_eq!(
        s.client.try_add_attester(&a),
        Err(Ok(Error::NotInitialized))
    );
}
