//! Registry entry points. Spec: `docs/ARCHITECTURE.md` §4.3. Row: M1-04.
//!
//! Country and attester-country scope are NOT checked here: they are registry-CI policy.
//! Attesters removed from the roster lose their powers; the admin manages their payees (E15).

use crate::constants::EVENT_SCHEMA_VERSION;
use crate::errors::Error;
use crate::events::{PayeeRegistered, PayeeStatusChanged, PayoutUpdated};
use crate::storage;
use crate::types::{Category, Payee, PayeeStatus};
use soroban_sdk::{Address, BytesN, Env};

pub fn register_payee(
    env: &Env,
    attester: &Address,
    payee_id: &BytesN<32>,
    payout: &Address,
    category: Category,
    meta_hash: &BytesN<32>,
) -> Result<(), Error> {
    attester.require_auth();
    storage::extend_instance_ttl(env);
    if !storage::is_attester(env, attester) {
        return Err(Error::NotAttester);
    }
    storage::extend_attester_ttl(env, attester);
    if storage::read_payee(env, payee_id).is_some() {
        return Err(Error::PayeeAlreadyExists);
    }
    if *payout == env.current_contract_address() {
        return Err(Error::InvalidPayout);
    }

    let now = env.ledger().timestamp();
    let payee = Payee {
        payout: payout.clone(),
        category,
        status: PayeeStatus::Active,
        status_changed_at: now,
        attester: attester.clone(),
        meta_hash: meta_hash.clone(),
        registered_at: now,
    };
    storage::write_payee(env, payee_id, &payee);

    PayeeRegistered {
        payee_id: payee_id.clone(),
        schema_version: EVENT_SCHEMA_VERSION,
        payout: payout.clone(),
        category,
        attester: attester.clone(),
        meta_hash: meta_hash.clone(),
    }
    .publish(env);
    Ok(())
}

/// Active ⇄ Suspended; Active or Suspended → Revoked. Revoked is terminal.
/// Setting the current status again is rejected.
fn transition_allowed(from: PayeeStatus, to: PayeeStatus) -> bool {
    matches!(
        (from, to),
        (PayeeStatus::Active, PayeeStatus::Suspended)
            | (PayeeStatus::Suspended, PayeeStatus::Active)
            | (PayeeStatus::Active, PayeeStatus::Revoked)
            | (PayeeStatus::Suspended, PayeeStatus::Revoked)
    )
}

/// Caller must be the admin, or the payee's vouching attester while still on the roster.
pub fn set_status(
    env: &Env,
    caller: &Address,
    payee_id: &BytesN<32>,
    status: PayeeStatus,
) -> Result<(), Error> {
    caller.require_auth();
    storage::extend_instance_ttl(env);
    let config = storage::read_config(env)?;
    let mut payee = storage::read_payee(env, payee_id).ok_or(Error::PayeeNotFound)?;

    let is_admin = *caller == config.admin;
    let is_vouching_attester = *caller == payee.attester && storage::is_attester(env, caller);
    if !is_admin && !is_vouching_attester {
        return Err(Error::NotVouchingAttester);
    }
    if is_vouching_attester {
        storage::extend_attester_ttl(env, caller);
    }
    if !transition_allowed(payee.status, status) {
        return Err(Error::InvalidStatusTransition);
    }

    payee.status = status;
    payee.status_changed_at = env.ledger().timestamp();
    storage::write_payee(env, payee_id, &payee);

    PayeeStatusChanged {
        payee_id: payee_id.clone(),
        schema_version: EVENT_SCHEMA_VERSION,
        status,
        changed_by: caller.clone(),
    }
    .publish(env);
    Ok(())
}

/// Affects NEW locks only: existing locks keep their payout snapshot (invariant 7).
pub fn update_payout(
    env: &Env,
    attester: &Address,
    payee_id: &BytesN<32>,
    new_payout: &Address,
) -> Result<(), Error> {
    attester.require_auth();
    storage::extend_instance_ttl(env);
    let mut payee = storage::read_payee(env, payee_id).ok_or(Error::PayeeNotFound)?;
    if *attester != payee.attester || !storage::is_attester(env, attester) {
        return Err(Error::NotVouchingAttester);
    }
    storage::extend_attester_ttl(env, attester);
    if payee.status == PayeeStatus::Revoked {
        return Err(Error::PayeeRevoked);
    }
    if *new_payout == payee.payout {
        return Err(Error::PayoutUnchanged);
    }
    if *new_payout == env.current_contract_address() {
        return Err(Error::InvalidPayout);
    }

    payee.payout = new_payout.clone();
    storage::write_payee(env, payee_id, &payee);

    PayoutUpdated {
        payee_id: payee_id.clone(),
        schema_version: EVENT_SCHEMA_VERSION,
        new_payout: new_payout.clone(),
    }
    .publish(env);
    Ok(())
}

pub fn get_payee(env: &Env, payee_id: &BytesN<32>) -> Result<Payee, Error> {
    storage::read_payee(env, payee_id).ok_or(Error::PayeeNotFound)
}
