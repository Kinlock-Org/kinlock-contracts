//! Unit tests. Explicit-auth tests live in `auth_tests.rs` (not only `mock_all_auths`).

mod auth_tests;
mod create_lock_tests;
mod refund_decline_tests;
mod registry_tests;
mod release_tests;

use crate::Kinlock;
use soroban_sdk::Env;

#[test]
fn contract_registers() {
    let env = Env::default();
    env.register(Kinlock, ());
}
