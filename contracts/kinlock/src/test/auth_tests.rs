//! Explicit-auth tests: every privileged function asserts `env.auths()`, and a call signed
//! by the wrong address fails. `mock_all_auths` alone would hide a missing `require_auth`.
//! Row: M1-12.

use super::Setup;
use crate::types::{Category, PayeeStatus};
use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke,
};
use soroban_sdk::{Address, BytesN, IntoVal, Symbol, Val, Vec};

/// Asserts the last call required exactly one authorization: `who` for `fn_name(args)`.
fn assert_single_auth(s: &Setup, who: &Address, fn_name: &str, args: Vec<Val>) {
    assert_eq!(
        s.env.auths(),
        std::vec![(
            who.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.client.address.clone(),
                    Symbol::new(&s.env, fn_name),
                    args,
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
}

// ----- Each privileged function requires the right address -----

#[test]
fn admin_functions_require_admin() {
    let s = Setup::new();
    let a = Address::generate(&s.env);

    s.client.add_attester(&a);
    assert_single_auth(&s, &s.admin, "add_attester", (a.clone(),).into_val(&s.env));

    s.client.remove_attester(&a);
    assert_single_auth(
        &s,
        &s.admin,
        "remove_attester",
        (a.clone(),).into_val(&s.env),
    );

    s.client.add_token(&a);
    assert_single_auth(&s, &s.admin, "add_token", (a.clone(),).into_val(&s.env));

    s.client.remove_token(&a);
    assert_single_auth(&s, &s.admin, "remove_token", (a.clone(),).into_val(&s.env));

    s.client.set_paused_new_locks(&true);
    assert_single_auth(
        &s,
        &s.admin,
        "set_paused_new_locks",
        (true,).into_val(&s.env),
    );

    s.client.set_caps(&10, &20);
    assert_single_auth(
        &s,
        &s.admin,
        "set_caps",
        (10_i128, 20_i128).into_val(&s.env),
    );
}

#[test]
fn register_payee_requires_attester() {
    let s = Setup::new();
    let attester = s.new_attester();
    let payee_id = BytesN::from_array(&s.env, &[1; 32]);
    let payout = Address::generate(&s.env);
    let meta_hash = BytesN::from_array(&s.env, &[2; 32]);
    s.client
        .register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    assert_single_auth(
        &s,
        &attester,
        "register_payee",
        (
            attester.clone(),
            payee_id,
            payout,
            Category::School,
            meta_hash,
        )
            .into_val(&s.env),
    );
}

#[test]
fn set_status_requires_caller() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client
        .set_status(&attester, &payee_id, &PayeeStatus::Suspended);
    assert_single_auth(
        &s,
        &attester,
        "set_status",
        (attester.clone(), payee_id, PayeeStatus::Suspended).into_val(&s.env),
    );
}

#[test]
fn set_status_by_admin_requires_admin() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    s.client
        .set_status(&s.admin, &payee_id, &PayeeStatus::Revoked);
    assert_single_auth(
        &s,
        &s.admin,
        "set_status",
        (s.admin.clone(), payee_id, PayeeStatus::Revoked).into_val(&s.env),
    );
}

#[test]
fn update_payout_requires_attester() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    let new_payout = Address::generate(&s.env);
    s.client.update_payout(&attester, &payee_id, &new_payout);
    assert_single_auth(
        &s,
        &attester,
        "update_payout",
        (attester.clone(), payee_id, new_payout).into_val(&s.env),
    );
}

// ----- Wrong signer fails (no mock_all_auths) -----

#[test]
fn admin_function_signed_by_non_admin_fails() {
    let s = Setup::with_auths(false);
    let intruder = Address::generate(&s.env);
    let a = Address::generate(&s.env);
    let invoke = MockAuthInvoke {
        contract: &s.client.address,
        fn_name: "add_attester",
        args: (a.clone(),).into_val(&s.env),
        sub_invokes: &[],
    };
    let result = s
        .client
        .mock_auths(&[MockAuth {
            address: &intruder,
            invoke: &invoke,
        }])
        .try_add_attester(&a);
    assert!(matches!(result, Err(Err(_))));
    assert!(!s.is_attester(&a));
}

/// Unsigned calls fail with an auth (host) error, not a contract error, and change nothing.
/// `upgrade` is not covered here: an unknown wasm hash also fails, so the assertion couldn't
/// tell auth apart. Its auth test needs a real uploaded wasm (roadmap M1-32).
#[test]
fn admin_functions_with_no_signature_fail() {
    let s = Setup::new();
    let attester = s.new_attester();
    let token = Address::generate(&s.env);
    s.client.add_token(&token);
    let other = Address::generate(&s.env);
    // Drop the blanket mock: from here on, no call is signed.
    s.env.set_auths(&[]);

    assert!(matches!(s.client.try_add_attester(&other), Err(Err(_))));
    assert!(matches!(
        s.client.try_remove_attester(&attester),
        Err(Err(_))
    ));
    assert!(matches!(s.client.try_add_token(&other), Err(Err(_))));
    assert!(matches!(s.client.try_remove_token(&token), Err(Err(_))));
    assert!(matches!(
        s.client.try_set_paused_new_locks(&true),
        Err(Err(_))
    ));
    assert!(matches!(s.client.try_set_caps(&1, &1), Err(Err(_))));

    assert!(!s.is_attester(&other));
    assert!(s.is_attester(&attester));
    assert!(!s.is_token_allowed(&other));
    assert!(s.is_token_allowed(&token));
    let c = s.config();
    assert!(!c.paused_new_locks);
    assert_eq!(c.max_lock_amount, i128::MAX);
}

#[test]
fn register_payee_signed_by_someone_else_fails() {
    let s = Setup::with_auths(false);
    let attester = Address::generate(&s.env);
    // Roster change signed by the admin.
    let add = MockAuthInvoke {
        contract: &s.client.address,
        fn_name: "add_attester",
        args: (attester.clone(),).into_val(&s.env),
        sub_invokes: &[],
    };
    s.client
        .mock_auths(&[MockAuth {
            address: &s.admin,
            invoke: &add,
        }])
        .add_attester(&attester);

    // Impersonation: `attester` named as the caller, but `intruder` signs.
    let intruder = Address::generate(&s.env);
    let payee_id = BytesN::from_array(&s.env, &[1; 32]);
    let payout = Address::generate(&s.env);
    let meta_hash = BytesN::from_array(&s.env, &[2; 32]);
    let invoke = MockAuthInvoke {
        contract: &s.client.address,
        fn_name: "register_payee",
        args: (
            attester.clone(),
            payee_id.clone(),
            payout.clone(),
            Category::School,
            meta_hash.clone(),
        )
            .into_val(&s.env),
        sub_invokes: &[],
    };
    let result = s
        .client
        .mock_auths(&[MockAuth {
            address: &intruder,
            invoke: &invoke,
        }])
        .try_register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    assert!(matches!(result, Err(Err(_))));
    assert!(s.client.try_get_payee(&payee_id).is_err());
}

#[test]
fn set_status_naming_admin_but_signed_by_attester_fails() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, _) = s.register_payee(&attester, 1);
    let args = (s.admin.clone(), payee_id.clone(), PayeeStatus::Revoked).into_val(&s.env);
    let invoke = MockAuthInvoke {
        contract: &s.client.address,
        fn_name: "set_status",
        args,
        sub_invokes: &[],
    };
    let result = s
        .client
        .mock_auths(&[MockAuth {
            address: &attester,
            invoke: &invoke,
        }])
        .try_set_status(&s.admin, &payee_id, &PayeeStatus::Revoked);
    assert!(matches!(result, Err(Err(_))));
    assert_eq!(s.client.get_payee(&payee_id).status, PayeeStatus::Active);
}

#[test]
fn update_payout_signed_by_someone_else_fails() {
    let s = Setup::new();
    let attester = s.new_attester();
    let (payee_id, payout) = s.register_payee(&attester, 1);
    let intruder = Address::generate(&s.env);
    let args = (attester.clone(), payee_id.clone(), intruder.clone()).into_val(&s.env);
    let invoke = MockAuthInvoke {
        contract: &s.client.address,
        fn_name: "update_payout",
        args,
        sub_invokes: &[],
    };
    let result = s
        .client
        .mock_auths(&[MockAuth {
            address: &intruder,
            invoke: &invoke,
        }])
        .try_update_payout(&attester, &payee_id, &intruder);
    assert!(matches!(result, Err(Err(_))));
    assert_eq!(s.client.get_payee(&payee_id).payout, payout);
}
