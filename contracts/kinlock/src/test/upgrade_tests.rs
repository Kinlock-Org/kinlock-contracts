//! `upgrade` with a real compiled contract: admin-only, explicit auth, and state survives.
//! Rows: M1-32, M1-12.
//!
//! Needs the release WASM: run `stellar contract build` (or
//! `cargo build --release --target wasm32v1-none -p kinlock`) before `cargo test`.
//! CI builds it first.

use super::{Setup, DAY, T0};
use crate::constants::{MIN_AMOUNT, STORAGE_VERSION};
use crate::storage;
use crate::types::LockState;
use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke,
};
use soroban_sdk::{Address, BytesN, IntoVal, Symbol};

const KINLOCK_WASM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../target/wasm32v1-none/release/kinlock.wasm"
));

fn upload(s: &Setup) -> BytesN<32> {
    s.env.deployer().upload_contract_wasm(KINLOCK_WASM)
}

#[test]
fn admin_upgrade_requires_admin_and_keeps_state() {
    let s = Setup::new();
    let f = s.standard_lock();
    let config_before = s.config();
    let payee_before = s.client.get_payee(&f.payee_id);
    let lock_before = s.lock(f.id);
    let hash = upload(&s);

    s.client.upgrade(&hash);

    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.client.address.clone(),
                    Symbol::new(&s.env, "upgrade"),
                    (hash.clone(),).into_val(&s.env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );

    // The upgraded (WASM) contract serves the same state and keeps working.
    assert_eq!(s.config(), config_before);
    assert_eq!(s.client.get_payee(&f.payee_id), payee_before);
    assert_eq!(s.lock(f.id), lock_before);
    let version = s
        .env
        .as_contract(&s.client.address, || storage::read_storage_version(&s.env));
    assert_eq!(version, Some(STORAGE_VERSION));

    s.set_time(T0 + DAY);
    s.client.release(&f.id, &0);
    assert_eq!(s.balance(&f.token, &f.payout), MIN_AMOUNT);
    assert_eq!(s.lock(f.id).state, LockState::Open);
}

#[test]
fn upgrade_signed_by_non_admin_fails() {
    let s = Setup::new();
    let f = s.standard_lock();
    let hash = upload(&s);
    let intruder = Address::generate(&s.env);
    let invoke = MockAuthInvoke {
        contract: &s.client.address,
        fn_name: "upgrade",
        args: (hash.clone(),).into_val(&s.env),
        sub_invokes: &[],
    };

    let result = s
        .client
        .mock_auths(&[MockAuth {
            address: &intruder,
            invoke: &invoke,
        }])
        .try_upgrade(&hash);

    assert!(matches!(result, Err(Err(_))));
    // Unchanged and still working.
    assert_eq!(s.lock(f.id).released, 0);
    s.set_time(T0 + DAY);
    s.client.release(&f.id, &0);
}
