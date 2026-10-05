//! Admin entry points: constructor, roster, allowlist, pause, caps. Row: M1-03.

use super::Setup;
use crate::errors::Error;
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
    s.client.set_caps(&1_000, &5_000);
    let c = s.config();
    assert_eq!(c.max_lock_amount, 1_000);
    assert_eq!(c.max_total_locked, 5_000);
    assert_eq!(c.total_locked, 0);
    assert!(!c.paused_new_locks);
}

#[test]
fn set_caps_rejects_zero_and_negative() {
    let s = Setup::new();
    assert_eq!(
        s.client.try_set_caps(&0, &5_000),
        Err(Ok(Error::InvalidCap))
    );
    assert_eq!(
        s.client.try_set_caps(&1_000, &0),
        Err(Ok(Error::InvalidCap))
    );
    assert_eq!(
        s.client.try_set_caps(&-1, &5_000),
        Err(Ok(Error::InvalidCap))
    );
    assert_eq!(
        s.client.try_set_caps(&1_000, &-1),
        Err(Ok(Error::InvalidCap))
    );
    assert_eq!(s.config().max_lock_amount, i128::MAX);
}
