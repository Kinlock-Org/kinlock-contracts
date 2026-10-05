//! Storage keys and typed accessors. Source: `docs/ARCHITECTURE.md` §4.2, §4.7.
//!
//! Rules: typed keys only; no unbounded collections; no per-user lists (enumeration comes
//! from events). TTL for a lock is extended to `expires_at + TTL_GRACE_SECS` on create.
//!
//! Long-lived entries (config, attesters, tokens, payees) are extended to the network's max
//! TTL. Extensions are always capped at `max_ttl()`: behavior above it is not relied on.

use crate::constants::{APPROX_LEDGER_CLOSE_SECS, TTL_REFRESH_WINDOW_SECS};
use crate::errors::Error;
use crate::types::{Config, Lock, Payee};
use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Instance storage -> Config.
    Config,
    /// Persistent -> bool. Attester roster.
    Attester(Address),
    /// Persistent -> bool. Token allowlist; gates creation only.
    Token(Address),
    /// Persistent: payee_id -> Payee.
    Payee(BytesN<32>),
    /// Persistent: lock_id -> Lock.
    Lock(u64),
}

/// Approximate number of ledgers in `secs`, saturating at `u32::MAX`.
pub fn secs_to_ledgers(secs: u64) -> u32 {
    u32::try_from(secs / APPROX_LEDGER_CLOSE_SECS).unwrap_or(u32::MAX)
}

/// `(threshold, extend_to)` that keeps an entry near the network's max TTL.
fn max_ttl_window(env: &Env) -> (u32, u32) {
    let max = env.storage().max_ttl();
    let threshold = max.saturating_sub(secs_to_ledgers(TTL_REFRESH_WINDOW_SECS));
    (threshold, max)
}

pub fn extend_instance_ttl(env: &Env) {
    let (threshold, extend_to) = max_ttl_window(env);
    env.storage().instance().extend_ttl(threshold, extend_to);
}

fn extend_persistent_to_max(env: &Env, key: &DataKey) {
    let (threshold, extend_to) = max_ttl_window(env);
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}

pub fn read_config(env: &Env) -> Result<Config, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)
}

pub fn write_config(env: &Env, config: &Config) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn is_attester(env: &Env, who: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Attester(who.clone()))
        .unwrap_or(false)
}

/// Keeps an attester's roster entry alive while it is being used, not only when written.
pub fn extend_attester_ttl(env: &Env, who: &Address) {
    extend_persistent_to_max(env, &DataKey::Attester(who.clone()));
}

pub fn set_attester(env: &Env, who: &Address, enabled: bool) {
    let key = DataKey::Attester(who.clone());
    if enabled {
        env.storage().persistent().set(&key, &true);
        extend_persistent_to_max(env, &key);
    } else {
        env.storage().persistent().remove(&key);
    }
}

pub fn is_token_allowed(env: &Env, token: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Token(token.clone()))
        .unwrap_or(false)
}

pub fn set_token_allowed(env: &Env, token: &Address, allowed: bool) {
    let key = DataKey::Token(token.clone());
    if allowed {
        env.storage().persistent().set(&key, &true);
        extend_persistent_to_max(env, &key);
    } else {
        env.storage().persistent().remove(&key);
    }
}

pub fn read_payee(env: &Env, payee_id: &BytesN<32>) -> Option<Payee> {
    env.storage()
        .persistent()
        .get(&DataKey::Payee(payee_id.clone()))
}

pub fn write_payee(env: &Env, payee_id: &BytesN<32>, payee: &Payee) {
    let key = DataKey::Payee(payee_id.clone());
    env.storage().persistent().set(&key, payee);
    extend_persistent_to_max(env, &key);
}

pub fn read_lock(env: &Env, lock_id: u64) -> Option<Lock> {
    env.storage().persistent().get(&DataKey::Lock(lock_id))
}

pub fn write_lock(env: &Env, lock: &Lock) {
    env.storage()
        .persistent()
        .set(&DataKey::Lock(lock.id), lock);
}
