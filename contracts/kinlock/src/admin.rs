//! Admin entry points (admin multisig). Spec: `docs/ARCHITECTURE.md` §4.3. Row: M1-03.
//!
//! Hard rule: nothing here may move, freeze, or redirect funds in existing locks.
//! Pause and allowlist changes gate `create_lock` only.

use crate::constants::MIN_AMOUNT;
use crate::errors::Error;
use crate::storage;
use crate::types::Config;
use soroban_sdk::{Address, BytesN, ContractExecutable, Env};

/// One-time setup, run by the contract constructor at deploy time so no one can front-run
/// initialization. Caps start unlimited; mainnet values are set with `set_caps` (P1).
pub fn initialize(env: &Env, admin: &Address) {
    let config = Config {
        admin: admin.clone(),
        next_lock_id: 1,
        paused_new_locks: false,
        max_lock_amount: i128::MAX,
        max_total_locked: i128::MAX,
        total_locked: 0,
    };
    storage::write_config(env, &config);
    storage::write_storage_version(env);
    storage::extend_instance_ttl(env);
}

/// Loads the config and requires the admin's authorization.
fn require_admin(env: &Env) -> Result<Config, Error> {
    let config = storage::read_config(env)?;
    config.admin.require_auth();
    storage::extend_instance_ttl(env);
    Ok(config)
}

pub fn add_attester(env: &Env, attester: &Address) -> Result<(), Error> {
    require_admin(env)?;
    storage::set_attester(env, attester, true);
    Ok(())
}

pub fn remove_attester(env: &Env, attester: &Address) -> Result<(), Error> {
    require_admin(env)?;
    storage::set_attester(env, attester, false);
    Ok(())
}

pub fn add_token(env: &Env, token: &Address) -> Result<(), Error> {
    require_admin(env)?;
    storage::set_token_allowed(env, token, true);
    Ok(())
}

/// Blocks NEW locks in this token only. Existing locks keep working (invariant 8).
pub fn remove_token(env: &Env, token: &Address) -> Result<(), Error> {
    require_admin(env)?;
    storage::set_token_allowed(env, token, false);
    Ok(())
}

/// Blocks `create_lock` only. Never blocks release, decline, or refund (hard rule 9).
pub fn set_paused_new_locks(env: &Env, paused: bool) -> Result<(), Error> {
    let mut config = require_admin(env)?;
    config.paused_new_locks = paused;
    storage::write_config(env, &config);
    Ok(())
}

/// Both caps must be at least `MIN_AMOUNT` (a smaller cap would act as a silent pause), and
/// the per-lock cap can't exceed the global cap. A global cap below the current
/// `total_locked` only blocks new locks.
pub fn set_caps(env: &Env, max_lock_amount: i128, max_total_locked: i128) -> Result<(), Error> {
    let mut config = require_admin(env)?;
    if max_lock_amount < MIN_AMOUNT || max_lock_amount > max_total_locked {
        return Err(Error::InvalidCap);
    }
    config.max_lock_amount = max_lock_amount;
    config.max_total_locked = max_total_locked;
    storage::write_config(env, &config);
    Ok(())
}

/// Multisig now; timelock before mainnet (X-07).
pub fn upgrade(env: &Env, new_wasm_hash: &BytesN<32>) -> Result<(), Error> {
    require_admin(env)?;
    env.deployer()
        .update_current_contract(ContractExecutable::Wasm(new_wasm_hash.clone()));
    Ok(())
}
