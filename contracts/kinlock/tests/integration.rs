//! Integration test suite for Kinlock. Row: M1-15 (Issue #28).
//!
//! Scenarios covered:
//! (a) Classic G-address sender -> C-address payee contract (with explicit auth assertion and event schema verification).
//! (b) C-address sender (contract wallet caller, explicit auth with token transfer sub-invocation, and post-expiry refund).
//! (c) Missing / unauthorized USDC trustline at release time: call reverts, zero state moved,
//!     and a retry after trustline authorization succeeds.
//! (d) Partial release, then refund of remainder, and decline with post-release remainder.
//! (e) Allowlist removal mid-flight: existing open locks release, refund, and decline unaffected.

use kinlock::constants::{EVENT_SCHEMA_VERSION, MIN_AMOUNT};
use kinlock::errors::Error;
use kinlock::events::{Declined, Refunded, Released};
use kinlock::storage;
use kinlock::types::{Category, Lock, LockState, RefundReason, TrancheInput};
use kinlock::{Kinlock, KinlockClient};
use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, Events as _, IssuerFlags, Ledger as _,
};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, Event as _, IntoVal, Symbol, Vec as SVec,
};

const T0: u64 = 1_000_000;
const DAY: u64 = 24 * 60 * 60;
const NETWORK_MAX_ENTRY_TTL: u32 = 3_110_400;

/// Minimal mock recipient/caller contract to obtain legitimate C-addresses.
#[contract]
pub struct DummyContract;

#[contractimpl]
impl DummyContract {
    pub fn poke(_env: Env) {}
}

/// Test environment harness for integration scenarios.
struct Harness {
    env: Env,
    client: KinlockClient<'static>,
    #[allow(dead_code)]
    admin: Address,
}

impl Harness {
    fn new(mock_all: bool) -> Self {
        let env = Env::default();
        env.ledger().set_timestamp(T0);
        env.ledger().set_max_entry_ttl(NETWORK_MAX_ENTRY_TTL);
        if mock_all {
            env.mock_all_auths();
        }
        let admin = Address::generate(&env);
        let contract_id = env.register(Kinlock, (admin.clone(),));
        let client = KinlockClient::new(&env, &contract_id);
        Harness { env, client, admin }
    }

    /// Creates a fresh revocable Stellar Asset Contract token (like USDC on Stellar).
    fn new_token(&self) -> (Address, StellarAssetClient<'static>) {
        let issuer = Address::generate(&self.env);
        let sac = self.env.register_stellar_asset_contract_v2(issuer);
        sac.issuer().set_flag(IssuerFlags::RevocableFlag);
        let token = sac.address();
        self.client.add_token(&token);
        (token.clone(), StellarAssetClient::new(&self.env, &token))
    }

    fn new_attester(&self) -> Address {
        let attester = Address::generate(&self.env);
        self.client.add_attester(&attester);
        attester
    }

    fn register_payee(&self, attester: &Address, payout: &Address, seed: u8) -> BytesN<32> {
        let payee_id = BytesN::from_array(&self.env, &[seed; 32]);
        let meta_hash = BytesN::from_array(&self.env, &[seed.wrapping_add(100); 32]);
        self.client
            .register_payee(attester, &payee_id, payout, &Category::School, &meta_hash);
        payee_id
    }

    fn new_c_address(&self) -> Address {
        self.env.register(DummyContract, ())
    }

    fn balance(&self, token: &Address, who: &Address) -> i128 {
        TokenClient::new(&self.env, token).balance(who)
    }

    fn is_token_allowed(&self, token: &Address) -> bool {
        self.env.as_contract(&self.client.address, || {
            storage::is_token_allowed(&self.env, token)
        })
    }

    fn tranches(&self, spec: &[(i128, u64)]) -> SVec<TrancheInput> {
        let mut v = SVec::new(&self.env);
        for (amount, unlock_at) in spec {
            v.push_back(TrancheInput {
                amount: *amount,
                unlock_at: *unlock_at,
            });
        }
        v
    }

    fn ref_hash(&self) -> BytesN<32> {
        BytesN::from_array(&self.env, &[0xAA; 32])
    }

    fn total_locked(&self) -> i128 {
        self.env
            .as_contract(&self.client.address, || storage::read_config(&self.env))
            .unwrap()
            .total_locked
    }

    fn get_lock(&self, id: u64) -> Lock {
        self.client.get_lock(&id)
    }

    fn set_time(&self, t: u64) {
        self.env.ledger().set_timestamp(t);
    }
}

// ==============================================================================
// Scenario (a): Classic G-address sender -> C-address payee contract
// ==============================================================================

#[test]
fn test_classic_g_address_sender_to_c_address_payee_contract() {
    let h = Harness::new(true);
    let (token, sac) = h.new_token();
    let attester = h.new_attester();

    // Payee payout is a deployed smart contract (C-address, e.g. school or escrow)
    let c_payee_payout = h.new_c_address();
    let payee_id = h.register_payee(&attester, &c_payee_payout, 1);

    // Sender is a classic G-address account
    let g_sender = Address::generate(&h.env);
    let amount_1 = 100 * MIN_AMOUNT;
    let amount_2 = 200 * MIN_AMOUNT;
    let total = amount_1 + amount_2;
    sac.mint(&g_sender, &total);

    let unlock_1 = T0 + DAY;
    let unlock_2 = T0 + 2 * DAY;
    let expiry = T0 + 10 * DAY;

    let lock_id = h.client.create_lock(
        &g_sender,
        &token,
        &payee_id,
        &h.tranches(&[(amount_1, unlock_1), (amount_2, unlock_2)]),
        &h.ref_hash(),
        &expiry,
    );

    // Release tranche 0 to C-address payee
    h.set_time(unlock_1);
    h.client.release(&lock_id, &0);

    // Check explicit auth and emitted event IMMEDIATELY before querying further contract methods
    let auths = h.env.auths();
    assert_eq!(auths.len(), 1);
    let (auth_who, auth_inv) = &auths[0];
    assert_eq!(auth_who, &c_payee_payout);
    assert_eq!(
        auth_inv.function,
        AuthorizedFunction::Contract((
            h.client.address.clone(),
            Symbol::new(&h.env, "release"),
            (lock_id, 0_u32).into_val(&h.env),
        ))
    );

    let expected_event = Released {
        id: lock_id,
        schema_version: EVENT_SCHEMA_VERSION,
        idx: 0,
        amount: amount_1,
        payout: c_payee_payout.clone(),
    };
    assert_eq!(
        h.env.events().all().filter_by_contract(&h.client.address),
        [expected_event.to_xdr(&h.env, &h.client.address)]
    );

    // Check money and contract state
    assert_eq!(h.balance(&token, &g_sender), 0);
    assert_eq!(h.balance(&token, &c_payee_payout), amount_1);
    assert_eq!(h.total_locked(), total - amount_1);

    let lock = h.get_lock(lock_id);
    assert_eq!(lock.released, amount_1);
    assert_eq!(lock.returned, 0);
    assert_eq!(lock.state, LockState::Open);
    assert!(lock.tranches.get(0).unwrap().released);
    assert!(!lock.tranches.get(1).unwrap().released);
}

// ==============================================================================
// Scenario (b): C-address sender
// ==============================================================================

#[test]
fn test_c_address_sender_lifecycle() {
    let h = Harness::new(true);
    let (token, sac) = h.new_token();
    let attester = h.new_attester();

    let payee_payout = Address::generate(&h.env);
    let payee_id = h.register_payee(&attester, &payee_payout, 2);

    // Sender is a smart contract (C-address, e.g. corporate treasury / contract wallet)
    let c_sender = h.new_c_address();
    let amount_1 = 50 * MIN_AMOUNT;
    let amount_2 = 150 * MIN_AMOUNT;
    let total = amount_1 + amount_2;
    sac.mint(&c_sender, &total);

    let unlock_1 = T0 + DAY;
    let unlock_2 = T0 + 3 * DAY;
    let expiry = T0 + 10 * DAY;

    let lock_id = h.client.create_lock(
        &c_sender,
        &token,
        &payee_id,
        &h.tranches(&[(amount_1, unlock_1), (amount_2, unlock_2)]),
        &h.ref_hash(),
        &expiry,
    );

    // Verify explicit auth structure required by C-address sender
    let auths = h.env.auths();
    assert_eq!(auths.len(), 1);
    let (auth_who, auth_inv) = &auths[0];
    assert_eq!(auth_who, &c_sender);
    assert_eq!(
        auth_inv.function,
        AuthorizedFunction::Contract((
            h.client.address.clone(),
            Symbol::new(&h.env, "create_lock"),
            (
                c_sender.clone(),
                token.clone(),
                payee_id.clone(),
                h.tranches(&[(amount_1, unlock_1), (amount_2, unlock_2)]),
                h.ref_hash(),
                expiry,
            )
                .into_val(&h.env),
        ))
    );
    // Must include the SAC transfer sub-invocation
    assert_eq!(auth_inv.sub_invocations.len(), 1);
    assert_eq!(
        auth_inv.sub_invocations[0].function,
        AuthorizedFunction::Contract((
            token.clone(),
            Symbol::new(&h.env, "transfer"),
            (c_sender.clone(), h.client.address.clone(), total).into_val(&h.env),
        ))
    );

    assert_eq!(h.balance(&token, &c_sender), 0);
    assert_eq!(h.total_locked(), total);

    // Release tranche 0
    h.set_time(unlock_1);
    h.client.release(&lock_id, &0);
    assert_eq!(h.balance(&token, &payee_payout), amount_1);
    assert_eq!(h.total_locked(), total - amount_1);

    // Time passes until expiry; C-address sender refunds the remainder
    h.set_time(expiry);
    h.client.refund(&lock_id);

    // Verify refund explicit auth and event IMMEDIATELY after refund call
    let refund_auths = h.env.auths();
    assert_eq!(refund_auths.len(), 1);
    let (who, inv) = &refund_auths[0];
    assert_eq!(who, &c_sender);
    assert_eq!(
        inv.function,
        AuthorizedFunction::Contract((
            h.client.address.clone(),
            Symbol::new(&h.env, "refund"),
            (lock_id,).into_val(&h.env),
        ))
    );

    let expected_refund = Refunded {
        id: lock_id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: amount_2,
        reason: RefundReason::Expired,
    };
    assert_eq!(
        h.env.events().all().filter_by_contract(&h.client.address),
        [expected_refund.to_xdr(&h.env, &h.client.address)]
    );

    // C-address sender successfully receives post-release remainder back
    assert_eq!(h.balance(&token, &c_sender), amount_2);
    assert_eq!(h.total_locked(), 0);

    let lock = h.get_lock(lock_id);
    assert_eq!(lock.released, amount_1);
    assert_eq!(lock.returned, amount_2);
    assert_eq!(lock.state, LockState::Refunded);
}

// ==============================================================================
// Scenario (c): Missing / unauthorized USDC trustline reverts, then succeeds
// ==============================================================================

#[test]
fn test_release_payout_lacks_trustline_reverts_then_succeeds_when_authorized() {
    let h = Harness::new(true);
    let (token, sac) = h.new_token();
    let attester = h.new_attester();

    let payout = Address::generate(&h.env);
    let payee_id = h.register_payee(&attester, &payout, 3);

    let sender = Address::generate(&h.env);
    let amount = 100 * MIN_AMOUNT;
    sac.mint(&sender, &amount);

    let unlock_at = T0 + DAY;
    let expiry = T0 + 10 * DAY;

    let lock_id = h.client.create_lock(
        &sender,
        &token,
        &payee_id,
        &h.tranches(&[(amount, unlock_at)]),
        &h.ref_hash(),
        &expiry,
    );

    assert_eq!(h.total_locked(), amount);
    let contract_bal_before = h.balance(&token, &h.client.address);
    assert_eq!(contract_bal_before, amount);

    // Simulate missing or revoked/frozen USDC trustline for payout
    sac.set_authorized(&payout, &false);

    h.set_time(unlock_at);

    // Step 1: Release MUST revert due to trustline transfer failure
    let try_res = h.client.try_release(&lock_id, &0);
    assert!(
        try_res.is_err(),
        "release must revert when payout lacks trustline authorization"
    );

    // Hard Rule 10: State must NOT half-move. Assert all invariants and balances are intact.
    assert_eq!(
        h.balance(&token, &h.client.address),
        contract_bal_before,
        "contract balance must not change"
    );
    assert_eq!(
        h.balance(&token, &payout),
        0,
        "payout must not have received funds"
    );
    assert_eq!(
        h.total_locked(),
        amount,
        "total_locked must remain unchanged"
    );

    let lock_after_fail = h.get_lock(lock_id);
    assert_eq!(lock_after_fail.released, 0, "released must still be 0");
    assert_eq!(lock_after_fail.returned, 0, "returned must still be 0");
    assert_eq!(
        lock_after_fail.state,
        LockState::Open,
        "state must stay Open"
    );
    assert!(
        !lock_after_fail.tranches.get(0).unwrap().released,
        "tranche must stay unreleased"
    );

    // Step 2: Payee establishes / authorizes the trustline
    sac.set_authorized(&payout, &true);

    // Step 3: Retry release: MUST now succeed completely
    let res = h.client.try_release(&lock_id, &0);
    assert!(
        res.is_ok(),
        "release must succeed once trustline is authorized"
    );

    // Verify settled state
    assert_eq!(h.balance(&token, &payout), amount);
    assert_eq!(h.balance(&token, &h.client.address), 0);
    assert_eq!(h.total_locked(), 0);

    let lock_settled = h.get_lock(lock_id);
    assert_eq!(lock_settled.released, amount);
    assert_eq!(lock_settled.state, LockState::Completed);
    assert!(lock_settled.tranches.get(0).unwrap().released);
}

// ==============================================================================
// Scenario (d): Partial release then refund of remainder, and decline
// ==============================================================================

#[test]
fn test_partial_release_then_refund_of_remainder_and_decline_remainder() {
    let h = Harness::new(true);
    let (token, sac) = h.new_token();
    let attester = h.new_attester();

    // --- Sub-scenario 1: Partial release then refund ---
    let payout_1 = Address::generate(&h.env);
    let payee_1 = h.register_payee(&attester, &payout_1, 10);
    let sender_1 = Address::generate(&h.env);

    let t0_amt = 100 * MIN_AMOUNT;
    let t1_amt = 200 * MIN_AMOUNT;
    let t2_amt = 300 * MIN_AMOUNT;
    let total_1 = t0_amt + t1_amt + t2_amt; // 600
    sac.mint(&sender_1, &total_1);

    let unlock_0 = T0 + DAY;
    let unlock_1 = T0 + 2 * DAY;
    let unlock_2 = T0 + 3 * DAY;
    let expiry_1 = T0 + 10 * DAY;

    let id_1 = h.client.create_lock(
        &sender_1,
        &token,
        &payee_1,
        &h.tranches(&[(t0_amt, unlock_0), (t1_amt, unlock_1), (t2_amt, unlock_2)]),
        &h.ref_hash(),
        &expiry_1,
    );

    // Release tranche 0 only (100)
    h.set_time(unlock_0);
    h.client.release(&id_1, &0);
    assert_eq!(h.balance(&token, &payout_1), t0_amt);
    assert_eq!(h.total_locked(), total_1 - t0_amt);

    // Advance to expiry; refund lock 1
    h.set_time(expiry_1);
    h.client.refund(&id_1);

    // Edge case assertion: Returned amount is EXACTLY the post-release remainder (500), NOT original total (600)
    let remainder_1 = t1_amt + t2_amt;
    assert_eq!(h.balance(&token, &sender_1), remainder_1);
    assert_eq!(h.balance(&token, &sender_1), total_1 - t0_amt);

    let lock_1 = h.get_lock(id_1);
    assert_eq!(lock_1.released, t0_amt);
    assert_eq!(lock_1.returned, remainder_1);
    assert_eq!(lock_1.state, LockState::Refunded);

    // Cannot refund again or release further tranches
    assert_eq!(h.client.try_refund(&id_1), Err(Ok(Error::LockNotOpen)));
    assert_eq!(h.client.try_release(&id_1, &1), Err(Ok(Error::LockNotOpen)));

    // --- Sub-scenario 2: Partial release then decline ---
    let now_2 = h.env.ledger().timestamp();
    let payout_2 = Address::generate(&h.env);
    let payee_2 = h.register_payee(&attester, &payout_2, 20);
    let sender_2 = Address::generate(&h.env);

    let d0_amt = 150 * MIN_AMOUNT;
    let d1_amt = 250 * MIN_AMOUNT;
    let total_2 = d0_amt + d1_amt; // 400
    sac.mint(&sender_2, &total_2);

    let d_unlock_0 = now_2 + DAY;
    let d_unlock_1 = now_2 + 2 * DAY;
    let d_expiry = now_2 + 10 * DAY;

    let id_2 = h.client.create_lock(
        &sender_2,
        &token,
        &payee_2,
        &h.tranches(&[(d0_amt, d_unlock_0), (d1_amt, d_unlock_1)]),
        &h.ref_hash(),
        &d_expiry,
    );

    // Release first tranche (150)
    h.set_time(d_unlock_0);
    h.client.release(&id_2, &0);
    assert_eq!(h.balance(&token, &payout_2), d0_amt);

    // Payee declines lock 2 before tranche 1
    h.client.decline(&id_2);

    // Verify Declined event immediately after decline call
    let expected_decline = Declined {
        id: id_2,
        schema_version: EVENT_SCHEMA_VERSION,
        amount: d1_amt,
    };
    assert_eq!(
        h.env.events().all().filter_by_contract(&h.client.address),
        [expected_decline.to_xdr(&h.env, &h.client.address)]
    );

    // Sender 2 receives the post-release remainder (250)
    assert_eq!(h.balance(&token, &sender_2), d1_amt);

    let lock_2 = h.get_lock(id_2);
    assert_eq!(lock_2.released, d0_amt);
    assert_eq!(lock_2.returned, d1_amt);
    assert_eq!(lock_2.state, LockState::Declined);

    // --- Sub-scenario 3: Zero release direct decline ---
    let now_3 = h.env.ledger().timestamp();
    let payout_3 = Address::generate(&h.env);
    let payee_3 = h.register_payee(&attester, &payout_3, 30);
    let sender_3 = Address::generate(&h.env);
    let total_3 = 500 * MIN_AMOUNT;
    sac.mint(&sender_3, &total_3);

    let d3_unlock = now_3 + DAY;
    let d3_expiry = now_3 + 10 * DAY;

    let id_3 = h.client.create_lock(
        &sender_3,
        &token,
        &payee_3,
        &h.tranches(&[(total_3, d3_unlock)]),
        &h.ref_hash(),
        &d3_expiry,
    );

    // Direct decline with zero releases
    h.client.decline(&id_3);
    assert_eq!(h.balance(&token, &sender_3), total_3);

    let lock_3 = h.get_lock(id_3);
    assert_eq!(lock_3.released, 0);
    assert_eq!(lock_3.returned, total_3);
    assert_eq!(lock_3.state, LockState::Declined);
}

// ==============================================================================
// Scenario (e): Allowlist removal mid-flight - existing locks unaffected
// ==============================================================================

#[test]
fn test_allowlist_removal_midflight_existing_locks_unaffected() {
    let h = Harness::new(true);
    let (token, sac) = h.new_token();
    let attester = h.new_attester();

    let payout_1 = Address::generate(&h.env);
    let payee_1 = h.register_payee(&attester, &payout_1, 50);
    let sender_1 = Address::generate(&h.env);

    let amt_release = 200 * MIN_AMOUNT;
    let amt_refund = 300 * MIN_AMOUNT;
    let total_1 = amt_release + amt_refund;
    sac.mint(&sender_1, &total_1);

    let unlock_at = T0 + DAY;
    let expiry = T0 + 10 * DAY;

    // Open lock 1 while token is allowlisted
    let id_1 = h.client.create_lock(
        &sender_1,
        &token,
        &payee_1,
        &h.tranches(&[(amt_release, unlock_at), (amt_refund, unlock_at + DAY)]),
        &h.ref_hash(),
        &expiry,
    );

    // Open lock 2 for decline test
    let payout_2 = Address::generate(&h.env);
    let payee_2 = h.register_payee(&attester, &payout_2, 51);
    let sender_2 = Address::generate(&h.env);
    let total_2 = 100 * MIN_AMOUNT;
    sac.mint(&sender_2, &total_2);
    let id_2 = h.client.create_lock(
        &sender_2,
        &token,
        &payee_2,
        &h.tranches(&[(total_2, unlock_at)]),
        &h.ref_hash(),
        &expiry,
    );

    assert_eq!(h.total_locked(), total_1 + total_2);

    // ADMIN REMOVES TOKEN FROM ALLOWLIST MID-FLIGHT
    h.client.remove_token(&token);
    assert!(!h.is_token_allowed(&token));

    // Invariant 8 / Gate check: New lock with removed token MUST fail
    let sender_new = Address::generate(&h.env);
    sac.mint(&sender_new, &MIN_AMOUNT);
    let new_res = h.client.try_create_lock(
        &sender_new,
        &token,
        &payee_1,
        &h.tranches(&[(MIN_AMOUNT, unlock_at)]),
        &h.ref_hash(),
        &expiry,
    );
    assert_eq!(new_res, Err(Ok(Error::TokenNotAllowed)));

    // Invariant 8 / Existing lock operations MUST NOT be blocked:
    // 1. Release on existing lock 1 succeeds
    h.set_time(unlock_at);
    let rel_res = h.client.try_release(&id_1, &0);
    assert!(
        rel_res.is_ok(),
        "existing lock release must not be blocked by token allowlist removal"
    );
    assert_eq!(h.balance(&token, &payout_1), amt_release);

    // 2. Decline on existing lock 2 succeeds
    let dec_res = h.client.try_decline(&id_2);
    assert!(
        dec_res.is_ok(),
        "existing lock decline must not be blocked by token allowlist removal"
    );
    assert_eq!(h.balance(&token, &sender_2), total_2);

    // 3. Refund on remainder of existing lock 1 succeeds
    h.set_time(expiry);
    let ref_res = h.client.try_refund(&id_1);
    assert!(
        ref_res.is_ok(),
        "existing lock refund must not be blocked by token allowlist removal"
    );
    assert_eq!(h.balance(&token, &sender_1), amt_refund);

    // Both locks completely settled and total_locked cleanly decremented
    assert_eq!(h.total_locked(), 0);
    assert_eq!(h.get_lock(id_1).state, LockState::Refunded);
    assert_eq!(h.get_lock(id_2).state, LockState::Declined);
}
