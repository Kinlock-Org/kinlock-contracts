//! Long-lived entries stay alive: written entries get the max TTL, and attesters are
//! re-extended when used, not only when written.

use super::Setup;
use crate::constants::TTL_REFRESH_WINDOW_SECS;
use crate::storage::{secs_to_ledgers, DataKey};
use crate::types::PayeeStatus;
use soroban_sdk::testutils::storage::{Instance as _, Persistent as _};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::Address;

fn persistent_ttl(s: &Setup, key: &DataKey) -> u32 {
    s.env.as_contract(&s.client.address, || {
        s.env.storage().persistent().get_ttl(key)
    })
}

fn instance_ttl(s: &Setup) -> u32 {
    s.env
        .as_contract(&s.client.address, || s.env.storage().instance().get_ttl())
}

fn max_ttl(s: &Setup) -> u32 {
    s.env
        .as_contract(&s.client.address, || s.env.storage().max_ttl())
}

/// Moves the ledger forward far enough that max-TTL entries fall below the refresh threshold.
fn age_ledger(s: &Setup) {
    let step = 2 * secs_to_ledgers(TTL_REFRESH_WINDOW_SECS);
    s.env.ledger().with_mut(|l| l.sequence_number += step);
}

#[test]
fn written_entries_get_max_ttl() {
    let s = Setup::new();
    let token = Address::generate(&s.env);
    s.client.add_token(&token);
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);

    let max = max_ttl(&s);
    assert_eq!(instance_ttl(&s), max);
    assert_eq!(persistent_ttl(&s, &DataKey::Token(token)), max);
    assert_eq!(persistent_ttl(&s, &DataKey::Attester(attester)), max);
    assert_eq!(persistent_ttl(&s, &DataKey::Payee(payee_id)), max);
}

#[test]
fn attester_is_re_extended_when_used() {
    let s = Setup::new();
    let attester = s.new_attester();
    let key = DataKey::Attester(attester.clone());

    age_ledger(&s);
    assert!(persistent_ttl(&s, &key) < max_ttl(&s));
    s.register_payee(&attester, 1);
    assert_eq!(persistent_ttl(&s, &key), max_ttl(&s));

    age_ledger(&s);
    let (payee_id, _) = s.register_payee(&attester, 2);
    age_ledger(&s);
    s.client
        .set_status(&attester, &payee_id, &PayeeStatus::Suspended);
    assert_eq!(persistent_ttl(&s, &key), max_ttl(&s));

    age_ledger(&s);
    s.client
        .update_payout(&attester, &payee_id, &Address::generate(&s.env));
    assert_eq!(persistent_ttl(&s, &key), max_ttl(&s));
}

#[test]
fn instance_is_re_extended_by_admin_calls() {
    let s = Setup::new();
    age_ledger(&s);
    assert!(instance_ttl(&s) < max_ttl(&s));
    s.client.set_paused_new_locks(&true);
    assert_eq!(instance_ttl(&s), max_ttl(&s));
}
