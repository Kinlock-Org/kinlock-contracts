//! Per-entry-point budget tests (M1-16).
//!
//! `soroban-budget-assert` doesn't exist anywhere we could find it: not on crates.io
//! (confirmed, it isn't published there), and no GitHub repo or docs page under that name
//! turned up in a live search either. M1-30 resolved by using the SDK's own first-party
//! metering API instead, `Env::cost_estimate()` (soroban-sdk 28.0.0, `testutils` feature;
//! verified against the real docs.rs pages for this exact pinned version, not assumed) --
//! no new dependency needed. `cost_estimate()` reports the resources metered during the
//! last top-level contract invocation, including CPU instructions and memory via `.budget()`,
//! and resource-limit enforcement against real Mainnet limits (`InvocationResourceLimits::mainnet()`)
//! is already on by default, so a call that's actually too expensive to run on Mainnet fails
//! the test on its own. The explicit ceilings below are a *tighter* early-warning budget set
//! from real measured values with headroom, so a regression that grows cost without yet
//! hitting the hard Mainnet limit still gets caught here instead of in production.
//!
//! This is a separate integration-test crate (`tests/`), so it only uses the contract's
//! public interface -- no access to the unit tests' private `Setup`/`LockFixture` helpers
//! in `src/test/mod.rs`.
//!
//! "Compare local estimates to testnet simulation": done against a fresh testnet deploy
//! (owner's go-ahead; `DEPLOYMENTS.md`), by simulating each call with the real `stellar`
//! CLI (`contract invoke --send=no --very-verbose`) and decoding the returned
//! `SorobanTransactionData` XDR for its real `instructions`/`write_bytes`. Local native-Rust
//! `cpu_instruction_cost()` consistently undercounts the real WASM-VM `instructions` figure
//! (the docs.rs page for `Budget` says native runs "may underestimate" this; now measured,
//! not just noted): roughly 4-5x for the fund-moving calls, 8-12x for the lighter
//! registry/read calls (smaller absolute numbers amplify the ratio). Local `memory_bytes_cost()`
//! and the network's `write_bytes` measure different things entirely (VM heap vs. ledger
//! storage footprint) and aren't directly comparable.
//!
//! | Entry point      | local cpu | testnet instructions | ratio | testnet write_bytes |
//! |-------------------|----------:|----------------------:|------:|---------------------:|
//! | create_lock       |   483,305 |             2,244,689 |  4.6x |                1,620 |
//! | release           |   483,958 |             2,120,124 |  4.4x |                1,628 |
//! | refund            |   424,011 |             2,111,536 |  5.0x |                1,624 |
//! | decline           |   426,104 |             1,997,922 |  4.7x |                1,624 |
//! | bump_lock         |   132,728 |               756,966 |  5.7x |                    0 |
//! | register_payee    |   145,715 |             1,742,500 | 12.0x |                  556 |
//! | update_payout     |   169,263 |             1,771,597 | 10.5x |                  556 |
//! | set_status        |   187,258 |             1,823,953 |  9.7x |                  560 |
//! | get_lock          |    90,975 |               732,050 |  8.0x |                    0 |
//!
//! Every real figure is tiny next to Mainnet's actual per-transaction cap of 100,000,000
//! instructions (confirmed against Stellar's own resource-limits docs, not assumed): the
//! worst case here, `create_lock`, uses about 2.2% of it. The divergence is consistent and
//! well understood (WASM execution vs. native Rust), not a red flag, and doesn't call for
//! raising any ceiling below. `M1-16` is `DONE`.

use soroban_sdk::testutils::cost_estimate::CostEstimate;
use soroban_sdk::testutils::{Address as _, IssuerFlags, Ledger as _};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{Address, BytesN, Env, Vec};

use kinlock::types::{Category, PayeeStatus, TrancheInput};
use kinlock::{Kinlock, KinlockClient};

const T0: u64 = 1_000_000;
const DAY: u64 = 24 * 60 * 60;
/// Testnet's `max_entry_ttl` (ledgers) as of 2026-10-05 (`src/test/mod.rs`'s own constant).
const NETWORK_MAX_ENTRY_TTL: u32 = 3_110_400;
const MIN_AMOUNT: i128 = 10_000_000;

struct Fixture<'a> {
    env: Env,
    client: KinlockClient<'a>,
}

fn setup() -> Fixture<'static> {
    let env = Env::default();
    env.ledger().set_timestamp(T0);
    env.ledger().set_max_entry_ttl(NETWORK_MAX_ENTRY_TTL);
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(Kinlock, (admin,));
    let client = KinlockClient::new(&env, &contract_id);
    Fixture { env, client }
}

fn new_token(env: &Env, client: &KinlockClient) -> Address {
    let issuer = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    let token = sac.address();
    client.add_token(&token);
    token
}

fn tranches(env: &Env, spec: &[(i128, u64)]) -> Vec<TrancheInput> {
    let mut v = Vec::new(env);
    for (amount, unlock_at) in spec {
        v.push_back(TrancheInput {
            amount: *amount,
            unlock_at: *unlock_at,
        });
    }
    v
}

/// Reports the CPU/memory budget of the call just made, and asserts it's within a
/// documented ceiling. Each ceiling below is roughly 2x the real measured baseline
/// (see the per-test values in the Changelog entry for this row) -- generous enough
/// not to flake on minor cost-model changes, tight enough to catch an actual regression.
fn assert_budget(env: &Env, label: &str, ceiling_cpu: u64, ceiling_mem: u64) {
    let est: CostEstimate = env.cost_estimate();
    let budget = est.budget();
    let cpu = budget.cpu_instruction_cost();
    let mem = budget.memory_bytes_cost();
    std::println!(
        "budget[{label}]: cpu={cpu} mem={mem} (ceiling cpu={ceiling_cpu} mem={ceiling_mem})"
    );
    assert!(
        cpu <= ceiling_cpu,
        "{label}: cpu_instruction_cost {cpu} exceeded ceiling {ceiling_cpu} -- investigate before raising the ceiling"
    );
    assert!(
        mem <= ceiling_mem,
        "{label}: memory_bytes_cost {mem} exceeded ceiling {ceiling_mem} -- investigate before raising the ceiling"
    );
}

#[test]
fn budget_create_lock() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[1; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[101; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    let token = new_token(&env, &client);
    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &(3 * MIN_AMOUNT));
    let ref_hash = BytesN::from_array(&env, &[0xAB; 32]);

    client.create_lock(
        &sender,
        &token,
        &payee_id,
        &tranches(
            &env,
            &[(MIN_AMOUNT, T0 + DAY), (2 * MIN_AMOUNT, T0 + 2 * DAY)],
        ),
        &ref_hash,
        &(T0 + 10 * DAY),
    );

    assert_budget(&env, "create_lock", 1_000_000, 400_000);
}

#[test]
fn budget_release() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[2; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[102; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    let token = new_token(&env, &client);
    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &MIN_AMOUNT);
    let ref_hash = BytesN::from_array(&env, &[0xAB; 32]);
    let lock_id = client.create_lock(
        &sender,
        &token,
        &payee_id,
        &tranches(&env, &[(MIN_AMOUNT, T0 + DAY)]),
        &ref_hash,
        &(T0 + 10 * DAY),
    );
    env.ledger().set_timestamp(T0 + DAY);

    client.release(&lock_id, &0);

    assert_budget(&env, "release", 1_000_000, 400_000);
}

#[test]
fn budget_refund() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[3; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[103; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    let token = new_token(&env, &client);
    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &MIN_AMOUNT);
    let ref_hash = BytesN::from_array(&env, &[0xAB; 32]);
    let lock_id = client.create_lock(
        &sender,
        &token,
        &payee_id,
        &tranches(&env, &[(MIN_AMOUNT, T0 + DAY)]),
        &ref_hash,
        &(T0 + 10 * DAY),
    );
    env.ledger().set_timestamp(T0 + 11 * DAY);

    client.refund(&lock_id);

    assert_budget(&env, "refund", 900_000, 350_000);
}

#[test]
fn budget_decline() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[4; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[104; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    let token = new_token(&env, &client);
    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &MIN_AMOUNT);
    let ref_hash = BytesN::from_array(&env, &[0xAB; 32]);
    let lock_id = client.create_lock(
        &sender,
        &token,
        &payee_id,
        &tranches(&env, &[(MIN_AMOUNT, T0 + DAY)]),
        &ref_hash,
        &(T0 + 10 * DAY),
    );

    client.decline(&lock_id);

    assert_budget(&env, "decline", 900_000, 350_000);
}

#[test]
fn budget_bump_lock() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[5; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[105; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    let token = new_token(&env, &client);
    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &MIN_AMOUNT);
    let ref_hash = BytesN::from_array(&env, &[0xAB; 32]);
    let lock_id = client.create_lock(
        &sender,
        &token,
        &payee_id,
        &tranches(&env, &[(MIN_AMOUNT, T0 + DAY)]),
        &ref_hash,
        &(T0 + 10 * DAY),
    );

    client.bump_lock(&lock_id);

    assert_budget(&env, "bump_lock", 300_000, 200_000);
}

#[test]
fn budget_register_payee() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[6; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[106; 32]);

    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);

    assert_budget(&env, "register_payee", 300_000, 150_000);
}

#[test]
fn budget_update_payout() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[7; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[107; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    let new_payout = Address::generate(&env);

    client.update_payout(&attester, &payee_id, &new_payout);

    assert_budget(&env, "update_payout", 350_000, 150_000);
}

#[test]
fn budget_set_status() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[8; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[108; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);

    client.set_status(&attester, &payee_id, &PayeeStatus::Suspended);

    assert_budget(&env, "set_status", 400_000, 150_000);
}

#[test]
fn budget_get_lock() {
    let Fixture { env, client } = setup();
    let attester = Address::generate(&env);
    client.add_attester(&attester);
    let payee_id = BytesN::from_array(&env, &[9; 32]);
    let payout = Address::generate(&env);
    let meta_hash = BytesN::from_array(&env, &[109; 32]);
    client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
    let token = new_token(&env, &client);
    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &MIN_AMOUNT);
    let ref_hash = BytesN::from_array(&env, &[0xAB; 32]);
    let lock_id = client.create_lock(
        &sender,
        &token,
        &payee_id,
        &tranches(&env, &[(MIN_AMOUNT, T0 + DAY)]),
        &ref_hash,
        &(T0 + 10 * DAY),
    );

    client.get_lock(&lock_id);

    assert_budget(&env, "get_lock", 200_000, 150_000);
}
