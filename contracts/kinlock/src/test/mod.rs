//! Unit tests. Explicit-auth tests live in `auth_tests.rs` (not only `mock_all_auths`).

mod admin_tests;
mod auth_tests;
mod create_lock_tests;
mod existing_locks_tests;
mod refund_decline_tests;
mod registry_tests;
mod release_tests;
mod ttl_tests;

use crate::constants::MIN_AMOUNT;
use crate::types::{Category, Config, Lock, TrancheInput};
use crate::{storage, Kinlock, KinlockClient};
use core::cell::Cell;
use soroban_sdk::testutils::{Address as _, IssuerFlags, Ledger as _};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, BytesN, Env, Vec};

/// Ledger time at the start of every test.
pub(crate) const T0: u64 = 1_000_000;
pub(crate) const DAY: u64 = 24 * 60 * 60;

/// Everything about one funded lock created by `Setup::open_lock`.
pub(crate) struct LockFixture {
    pub id: u64,
    pub sender: Address,
    pub payout: Address,
    pub payee_id: BytesN<32>,
    pub attester: Address,
    pub token: Address,
}

/// A deployed contract with its admin. `mock_all_auths` is on unless `with_auths` is false.
pub(crate) struct Setup {
    pub env: Env,
    pub client: KinlockClient<'static>,
    pub admin: Address,
    seed: Cell<u8>,
}

impl Setup {
    pub fn new() -> Self {
        Self::with_auths(true)
    }

    pub fn with_auths(mock_all: bool) -> Self {
        let env = Env::default();
        env.ledger().set_timestamp(T0);
        if mock_all {
            env.mock_all_auths();
        }
        let admin = Address::generate(&env);
        let contract_id = env.register(Kinlock, (admin.clone(),));
        let client = KinlockClient::new(&env, &contract_id);
        Setup {
            env,
            client,
            admin,
            seed: Cell::new(200),
        }
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

impl Setup {
    /// A fresh Stellar Asset Contract token (issuer can revoke authorization), allowlisted.
    pub fn new_token(&self) -> Address {
        let issuer = Address::generate(&self.env);
        let sac = self.env.register_stellar_asset_contract_v2(issuer);
        // Like USDC: the issuer can freeze accounts, so trustline failures can be tested.
        sac.issuer().set_flag(IssuerFlags::RevocableFlag);
        let token = sac.address();
        self.client.add_token(&token);
        token
    }

    pub fn mint(&self, token: &Address, to: &Address, amount: i128) {
        StellarAssetClient::new(&self.env, token).mint(to, &amount);
    }

    pub fn balance(&self, token: &Address, who: &Address) -> i128 {
        TokenClient::new(&self.env, token).balance(who)
    }

    pub fn set_authorized(&self, token: &Address, who: &Address, authorized: bool) {
        StellarAssetClient::new(&self.env, token).set_authorized(who, &authorized);
    }

    pub fn tranches(&self, spec: &[(i128, u64)]) -> Vec<TrancheInput> {
        let mut v = Vec::new(&self.env);
        for (amount, unlock_at) in spec {
            v.push_back(TrancheInput {
                amount: *amount,
                unlock_at: *unlock_at,
            });
        }
        v
    }

    pub fn ref_hash(&self) -> BytesN<32> {
        BytesN::from_array(&self.env, &[0xAB; 32])
    }

    pub fn lock(&self, id: u64) -> Lock {
        self.client.get_lock(&id)
    }

    pub fn set_time(&self, t: u64) {
        self.env.ledger().set_timestamp(t);
    }

    /// Registers a new attester + payee + token, funds a sender with exactly the total,
    /// and creates a lock with tranches `spec` = [(amount, unlock_at)].
    pub fn open_lock(&self, spec: &[(i128, u64)], expires_at: u64) -> LockFixture {
        let attester = self.new_attester();
        let seed = self.seed.get();
        self.seed.set(seed.wrapping_add(1));
        let (payee_id, payout) = self.register_payee(&attester, seed);
        let token = self.new_token();
        let sender = Address::generate(&self.env);
        let total: i128 = spec.iter().map(|(a, _)| *a).sum();
        self.mint(&token, &sender, total);
        let id = self.client.create_lock(
            &sender,
            &token,
            &payee_id,
            &self.tranches(spec),
            &self.ref_hash(),
            &expires_at,
        );
        LockFixture {
            id,
            sender,
            payout,
            payee_id,
            attester,
            token,
        }
    }

    /// Two tranches (1 and 2 units), unlocking at T0+1 day and T0+2 days; expires T0+10 days.
    pub fn standard_lock(&self) -> LockFixture {
        self.open_lock(
            &[(MIN_AMOUNT, T0 + DAY), (2 * MIN_AMOUNT, T0 + 2 * DAY)],
            T0 + 10 * DAY,
        )
    }
}

#[test]
fn contract_registers() {
    let s = Setup::new();
    assert_eq!(s.config().admin, s.admin);
}
