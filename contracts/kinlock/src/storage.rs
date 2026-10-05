//! Storage keys and typed accessors. Source: `docs/ARCHITECTURE.md` §4.2, §4.7.
//!
//! Rules: typed keys only; no unbounded collections; no per-user lists (enumeration comes
//! from events). TTL for a lock is extended to `expires_at + TTL_GRACE_SECS` on create.

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

pub fn read_config(env: &Env) -> Option<Config> {
    env.storage().instance().get(&DataKey::Config)
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

pub fn is_token_allowed(env: &Env, token: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Token(token.clone()))
        .unwrap_or(false)
}

pub fn read_payee(env: &Env, payee_id: &BytesN<32>) -> Option<Payee> {
    env.storage()
        .persistent()
        .get(&DataKey::Payee(payee_id.clone()))
}

pub fn write_payee(env: &Env, payee_id: &BytesN<32>, payee: &Payee) {
    env.storage()
        .persistent()
        .set(&DataKey::Payee(payee_id.clone()), payee);
}

pub fn read_lock(env: &Env, lock_id: u64) -> Option<Lock> {
    env.storage().persistent().get(&DataKey::Lock(lock_id))
}

pub fn write_lock(env: &Env, lock: &Lock) {
    env.storage()
        .persistent()
        .set(&DataKey::Lock(lock.id), lock);
}
