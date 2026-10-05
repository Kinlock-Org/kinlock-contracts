//! Unit tests. Explicit-auth tests live in `auth_tests.rs` (not only `mock_all_auths`).

mod admin_tests;
mod auth_tests;
mod create_lock_tests;
mod refund_decline_tests;
mod registry_tests;
mod release_tests;
mod ttl_tests;

use crate::types::{Category, Config};
use crate::{storage, Kinlock, KinlockClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

/// A deployed contract with its admin. `mock_all_auths` is on unless `with_auths` is false.
pub(crate) struct Setup {
    pub env: Env,
    pub client: KinlockClient<'static>,
    pub admin: Address,
}

impl Setup {
    pub fn new() -> Self {
        Self::with_auths(true)
    }

    pub fn with_auths(mock_all: bool) -> Self {
        let env = Env::default();
        if mock_all {
            env.mock_all_auths();
        }
        let admin = Address::generate(&env);
        let contract_id = env.register(Kinlock, (admin.clone(),));
        let client = KinlockClient::new(&env, &contract_id);
        Setup { env, client, admin }
    }

    pub fn config(&self) -> Config {
        self.env
            .as_contract(&self.client.address, || storage::read_config(&self.env))
            .unwrap()
    }

    pub fn is_attester(&self, who: &Address) -> bool {
        self.env.as_contract(&self.client.address, || {
            storage::is_attester(&self.env, who)
        })
    }

    pub fn is_token_allowed(&self, token: &Address) -> bool {
        self.env.as_contract(&self.client.address, || {
            storage::is_token_allowed(&self.env, token)
        })
    }

    pub fn new_attester(&self) -> Address {
        let attester = Address::generate(&self.env);
        self.client.add_attester(&attester);
        attester
    }

    /// Registers a School payee vouched for by `attester`; returns (payee_id, payout).
    pub fn register_payee(&self, attester: &Address, seed: u8) -> (BytesN<32>, Address) {
        let payee_id = BytesN::from_array(&self.env, &[seed; 32]);
        let payout = Address::generate(&self.env);
        let meta_hash = BytesN::from_array(&self.env, &[seed.wrapping_add(100); 32]);
        self.client
            .register_payee(attester, &payee_id, &payout, &Category::School, &meta_hash);
        (payee_id, payout)
    }
}

#[test]
fn contract_registers() {
    let s = Setup::new();
    assert_eq!(s.config().admin, s.admin);
}
