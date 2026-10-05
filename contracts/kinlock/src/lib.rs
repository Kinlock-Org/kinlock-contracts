#![no_std]
//! Kinlock: purpose-locked USDC transfers to verified payees.
//!
//! One contract, two modules (`registry`, `vault`) plus `admin`.
//! Design: `docs/ARCHITECTURE.md` §4. Invariants: §4.6. Hard rules: `AGENTS.md` §3.
//!
//! The contract is country-agnostic (AGENTS.md hard rule 11): it has no notion of
//! country, currency, or locale. Those live in `kinlock-registry` data only.
//!
//! This file is the only `#[contractimpl]`: entry points delegate to their modules.

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

pub mod admin;
pub mod constants;
pub mod errors;
pub mod events;
pub mod registry;
pub mod storage;
pub mod types;
pub mod vault;

use errors::Error;
use types::{Category, Lock, Payee, PayeeStatus, TrancheInput};

#[contract]
pub struct Kinlock;

#[contractimpl]
impl Kinlock {
    // ----- Admin -----

    /// Runs once at deploy, so initialization can't be front-run.
    pub fn __constructor(env: Env, admin: Address) {
        admin::initialize(&env, &admin);
    }

    pub fn add_attester(env: Env, attester: Address) -> Result<(), Error> {
        admin::add_attester(&env, &attester)
    }

    pub fn remove_attester(env: Env, attester: Address) -> Result<(), Error> {
        admin::remove_attester(&env, &attester)
    }

    pub fn add_token(env: Env, token: Address) -> Result<(), Error> {
        admin::add_token(&env, &token)
    }

    pub fn remove_token(env: Env, token: Address) -> Result<(), Error> {
        admin::remove_token(&env, &token)
    }

    pub fn set_paused_new_locks(env: Env, paused: bool) -> Result<(), Error> {
        admin::set_paused_new_locks(&env, paused)
    }

    pub fn set_caps(env: Env, max_lock_amount: i128, max_total_locked: i128) -> Result<(), Error> {
        admin::set_caps(&env, max_lock_amount, max_total_locked)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        admin::upgrade(&env, &new_wasm_hash)
    }

    // ----- Registry -----

    pub fn register_payee(
        env: Env,
        attester: Address,
        payee_id: BytesN<32>,
        payout: Address,
        category: Category,
        meta_hash: BytesN<32>,
    ) -> Result<(), Error> {
        registry::register_payee(&env, &attester, &payee_id, &payout, category, &meta_hash)
    }

    pub fn set_status(
        env: Env,
        caller: Address,
        payee_id: BytesN<32>,
        status: PayeeStatus,
    ) -> Result<(), Error> {
        registry::set_status(&env, &caller, &payee_id, status)
    }

    pub fn update_payout(
        env: Env,
        attester: Address,
        payee_id: BytesN<32>,
        new_payout: Address,
    ) -> Result<(), Error> {
        registry::update_payout(&env, &attester, &payee_id, &new_payout)
    }

    pub fn get_payee(env: Env, payee_id: BytesN<32>) -> Result<Payee, Error> {
        registry::get_payee(&env, &payee_id)
    }

    // ----- Vault -----

    pub fn create_lock(
        env: Env,
        sender: Address,
        token: Address,
        payee_id: BytesN<32>,
        tranches: Vec<TrancheInput>,
        ref_hash: BytesN<32>,
        expires_at: u64,
    ) -> Result<u64, Error> {
        vault::create_lock(
            &env, &sender, &token, &payee_id, &tranches, &ref_hash, expires_at,
        )
    }

    pub fn release(env: Env, lock_id: u64, idx: u32) -> Result<(), Error> {
        vault::release(&env, lock_id, idx)
    }

    pub fn refund(env: Env, lock_id: u64) -> Result<(), Error> {
        vault::refund(&env, lock_id)
    }

    pub fn decline(env: Env, lock_id: u64) -> Result<(), Error> {
        vault::decline(&env, lock_id)
    }

    pub fn bump_lock(env: Env, lock_id: u64) -> Result<(), Error> {
        vault::bump_lock(&env, lock_id)
    }

    pub fn get_lock(env: Env, lock_id: u64) -> Result<Lock, Error> {
        vault::get_lock(&env, lock_id)
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
