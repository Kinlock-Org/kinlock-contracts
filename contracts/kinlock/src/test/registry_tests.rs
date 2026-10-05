//! `register_payee`, `set_status` transitions, `update_payout`, events. Row: M1-04.

use super::Setup;
use crate::constants::EVENT_SCHEMA_VERSION;
use crate::errors::Error;
use crate::events::{PayeeRegistered, PayeeStatusChanged, PayoutUpdated};
use crate::types::{Category, PayeeStatus};
use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{Address, BytesN, Event as _};

// ----- register_payee -----

#[test]
fn register_payee_stores_active_payee() {
    let s = Setup::new();
    s.env.ledger().set_timestamp(1_000);
    let attester = s.new_attester();
    let payee_id = BytesN::from_array(&s.env, &[7; 32]);
    let payout = Address::generate(&s.env);
    let meta_hash = BytesN::from_array(&s.env, &[9; 32]);

    s.client
        .register_payee(&attester, &payee_id, &payout, &Category::Rent, &meta_hash);

    let p = s.client.get_payee(&payee_id);
    assert_eq!(p.payout, payout);
    assert_eq!(p.category, Category::Rent);
    assert_eq!(p.status, PayeeStatus::Active);
    assert_eq!(p.status_changed_at, 1_000);
    assert_eq!(p.attester, attester);
    assert_eq!(p.meta_hash, meta_hash);
    assert_eq!(p.registered_at, 1_000);
}

#[test]
fn register_payee_emits_event() {
    let s = Setup::new();
    let attester = s.new_attester();
    let payee_id = BytesN::from_array(&s.env, &[7; 32]);
    let payout = Address::generate(&s.env);
    let meta_hash = BytesN::from_array(&s.env, &[9; 32]);

    s.client
        .register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);

    let expected = PayeeRegistered {
        payee_id,
        schema_version: EVENT_SCHEMA_VERSION,
        payout,
        category: Category::School,
        attester,
        meta_hash,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
}

#[test]
fn register_payee_rejects_non_attester() {
    let s = Setup::new();
    let not_attester = Address::generate(&s.env);
    let payee_id = BytesN::from_array(&s.env, &[1; 32]);
    let payout = Address::generate(&s.env);
    let meta_hash = BytesN::from_array(&s.env, &[2; 32]);
    assert_eq!(
        s.client.try_register_payee(
            &not_attester,
            &payee_id,
            &payout,
            &Category::School,
            &meta_hash
        ),
        Err(Ok(Error::NotAttester))
    );
}

#[test]
fn register_payee_rejects_removed_attester() {
    let s = Setup::new();
    let attester = s.new_attester();
    s.client.remove_attester(&attester);
    let payee_id = BytesN::from_array(&s.env, &[1; 32]);
    let payout = Address::generate(&s.env);
    let meta_hash = BytesN::from_array(&s.env, &[2; 32]);
    assert_eq!(
        s.client
            .try_register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash),
        Err(Ok(Error::NotAttester))
    );
}

#[test]
fn register_payee_rejects_duplicate_id() {
    let s = Setup::new();
    let attester = s.new_attester();
    let other_attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    let payout = Address::generate(&s.env);
    let meta_hash = BytesN::from_array(&s.env, &[3; 32]);
    assert_eq!(
        s.client.try_register_payee(
            &other_attester,
            &payee_id,
            &payout,
            &Category::Rent,
            &meta_hash
        ),
        Err(Ok(Error::PayeeAlreadyExists))
    );
    assert_eq!(s.client.get_payee(&payee_id).attester, attester);
}

#[test]
fn get_payee_unknown_id() {
    let s = Setup::new();
    let payee_id = BytesN::from_array(&s.env, &[42; 32]);
    assert_eq!(
        s.client.try_get_payee(&payee_id),
        Err(Ok(Error::PayeeNotFound))
    );
}

// ----- set_status -----

#[test]
fn set_status_allowed_transitions() {
    let allowed = [
        (PayeeStatus::Active, PayeeStatus::Suspended),
        (PayeeStatus::Suspended, PayeeStatus::Active),
        (PayeeStatus::Active, PayeeStatus::Revoked),
        (PayeeStatus::Suspended, PayeeStatus::Revoked),
    ];
    for (from, to) in allowed {
        let s = Setup::new();
        let attester = s.new_attester();
        let (payee_id, _) = s.register_payee(&attester, 1);
        if from == PayeeStatus::Suspended {
            s.client
                .set_status(&attester, &payee_id, &PayeeStatus::Suspended);
        }
        s.env.ledger().set_timestamp(5_000);
        s.client.set_status(&attester, &payee_id, &to);
        let p = s.client.get_payee(&payee_id);
        assert_eq!(p.status, to, "{from:?} -> {to:?}");
        assert_eq!(p.status_changed_at, 5_000);
    }
}

#[test]
fn set_status_rejected_transitions() {
    let rejected = [
        (PayeeStatus::Active, PayeeStatus::Active),
        (PayeeStatus::Suspended, PayeeStatus::Suspended),
        (PayeeStatus::Revoked, PayeeStatus::Active),
        (PayeeStatus::Revoked, PayeeStatus::Suspended),
        (PayeeStatus::Revoked, PayeeStatus::Revoked),
    ];
    for (from, to) in rejected {
        let s = Setup::new();
        let attester = s.new_attester();
        let (payee_id, _) = s.register_payee(&attester, 1);
        if from != PayeeStatus::Active {
            s.client.set_status(&attester, &payee_id, &from);
        }
        let before = s.client.get_payee(&payee_id);
        assert_eq!(
            s.client.try_set_status(&attester, &payee_id, &to),
            Err(Ok(Error::InvalidStatusTransition)),
            "{from:?} -> {to:?}"
        );
        assert_eq!(s.client.get_payee(&payee_id), before);
    }
}

#[test]
fn set_status_by_admin() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client
        .set_status(&s.admin, &payee_id, &PayeeStatus::Suspended);
    assert_eq!(s.client.get_payee(&payee_id).status, PayeeStatus::Suspended);
}

#[test]
fn set_status_rejects_other_attester() {
    let s = Setup::new();
    let attester = s.new_attester();
    let other_attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    assert_eq!(
        s.client
            .try_set_status(&other_attester, &payee_id, &PayeeStatus::Revoked),
        Err(Ok(Error::NotVouchingAttester))
    );
}

#[test]
fn removed_attester_loses_status_power_but_admin_keeps_it() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client.remove_attester(&attester);
    assert_eq!(
        s.client
            .try_set_status(&attester, &payee_id, &PayeeStatus::Suspended),
        Err(Ok(Error::NotVouchingAttester))
    );
    s.client
        .set_status(&s.admin, &payee_id, &PayeeStatus::Suspended);
    assert_eq!(s.client.get_payee(&payee_id).status, PayeeStatus::Suspended);
}

#[test]
fn set_status_unknown_payee() {
    let s = Setup::new();
    let payee_id = BytesN::from_array(&s.env, &[42; 32]);
    assert_eq!(
        s.client
            .try_set_status(&s.admin, &payee_id, &PayeeStatus::Suspended),
        Err(Ok(Error::PayeeNotFound))
    );
}

#[test]
fn set_status_emits_event() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client
        .set_status(&attester, &payee_id, &PayeeStatus::Suspended);
    let expected = PayeeStatusChanged {
        payee_id,
        schema_version: EVENT_SCHEMA_VERSION,
        status: PayeeStatus::Suspended,
        changed_by: attester,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
}

// ----- update_payout -----

#[test]
fn update_payout_changes_only_payout() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    let before = s.client.get_payee(&payee_id);
    let new_payout = Address::generate(&s.env);
    s.env.ledger().set_timestamp(9_000);

    s.client.update_payout(&attester, &payee_id, &new_payout);

    let after = s.client.get_payee(&payee_id);
    assert_eq!(after.payout, new_payout);
    assert_eq!(after.category, before.category);
    assert_eq!(after.status, before.status);
    assert_eq!(after.status_changed_at, before.status_changed_at);
    assert_eq!(after.attester, before.attester);
    assert_eq!(after.meta_hash, before.meta_hash);
    assert_eq!(after.registered_at, before.registered_at);
}

#[test]
fn update_payout_allowed_while_suspended() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client
        .set_status(&attester, &payee_id, &PayeeStatus::Suspended);
    let new_payout = Address::generate(&s.env);
    s.client.update_payout(&attester, &payee_id, &new_payout);
    assert_eq!(s.client.get_payee(&payee_id).payout, new_payout);
}

#[test]
fn update_payout_rejects_revoked_payee() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client
        .set_status(&attester, &payee_id, &PayeeStatus::Revoked);
    let new_payout = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_update_payout(&attester, &payee_id, &new_payout),
        Err(Ok(Error::PayeeRevoked))
    );
}

#[test]
fn update_payout_rejects_same_address() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, payout) = s.register_payee(&attester, 1);
    assert_eq!(
        s.client.try_update_payout(&attester, &payee_id, &payout),
        Err(Ok(Error::PayoutUnchanged))
    );
}

#[test]
fn update_payout_rejects_other_attester_and_admin() {
    let s = Setup::new();
    let attester = s.new_attester();
    let other_attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    let new_payout = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_update_payout(&other_attester, &payee_id, &new_payout),
        Err(Ok(Error::NotVouchingAttester))
    );
    assert_eq!(
        s.client.try_update_payout(&s.admin, &payee_id, &new_payout),
        Err(Ok(Error::NotVouchingAttester))
    );
}

#[test]
fn update_payout_rejects_removed_attester() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client.remove_attester(&attester);
    let new_payout = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_update_payout(&attester, &payee_id, &new_payout),
        Err(Ok(Error::NotVouchingAttester))
    );
}

#[test]
fn update_payout_unknown_payee() {
    let s = Setup::new();
    let attester = s.new_attester();
    let payee_id = BytesN::from_array(&s.env, &[42; 32]);
    let new_payout = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_update_payout(&attester, &payee_id, &new_payout),
        Err(Ok(Error::PayeeNotFound))
    );
}

#[test]
fn update_payout_emits_event() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    let new_payout = Address::generate(&s.env);
    s.client.update_payout(&attester, &payee_id, &new_payout);
    let expected = PayoutUpdated {
        payee_id,
        schema_version: EVENT_SCHEMA_VERSION,
        new_payout,
    };
    assert_eq!(
        s.env.events().all().filter_by_contract(&s.client.address),
        [expected.to_xdr(&s.env, &s.client.address)]
    );
}

// ----- payout can't be the contract itself -----

#[test]
fn register_payee_rejects_contract_as_payout() {
    let s = Setup::new();
    let attester = s.new_attester();
    let payee_id = BytesN::from_array(&s.env, &[1; 32]);
    let meta_hash = BytesN::from_array(&s.env, &[2; 32]);
    assert_eq!(
        s.client.try_register_payee(
            &attester,
            &payee_id,
            &s.client.address,
            &Category::School,
            &meta_hash
        ),
        Err(Ok(Error::InvalidPayout))
    );
}

#[test]
fn update_payout_rejects_contract_as_payout() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, payout) = s.register_payee(&attester, 1);
    assert_eq!(
        s.client
            .try_update_payout(&attester, &payee_id, &s.client.address),
        Err(Ok(Error::InvalidPayout))
    );
    assert_eq!(s.client.get_payee(&payee_id).payout, payout);
}
