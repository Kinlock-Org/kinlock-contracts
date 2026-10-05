//! Vault entry points. Spec: `docs/ARCHITECTURE.md` §4.3, §4.5, §4.6. Rows: M1-05..M1-10.
//!
//! Funds leave the contract only to `lock.payout` (the snapshot) or `lock.sender`
//! (invariant 3). State is always written before any token transfer, so a failed transfer
//! reverts the whole call (invariant 10). Pause and the token allowlist gate `create_lock`
//! only (invariant 8).

use crate::constants::{
    EVENT_SCHEMA_VERSION, MAX_LOCK_DURATION_SECS, MAX_TRANCHES, MIN_AMOUNT, MIN_EXPIRY_AHEAD_SECS,
    SUSPENSION_REFUND_GRACE_SECS, TTL_GRACE_SECS,
};
use crate::errors::Error;
use crate::events::{Declined, LockCreated, Refunded, Released};
use crate::storage;
use crate::types::{Lock, LockState, PayeeStatus, RefundReason, Tranche, TrancheInput};
use soroban_sdk::{token, Address, BytesN, Env, MuxedAddress, Vec};

fn transfer_out(env: &Env, token: &Address, to: &Address, amount: i128) {
    token::Client::new(env, token).transfer(
        &env.current_contract_address(),
        MuxedAddress::from(to.clone()),
        &amount,
    );
}

/// Ledger time until which a lock (and its payee) must stay in storage.
fn keep_until(lock_expires_at: u64) -> Result<u64, Error> {
    lock_expires_at
        .checked_add(TTL_GRACE_SECS)
        .ok_or(Error::Overflow)
}

pub fn create_lock(
    env: &Env,
    sender: &Address,
    token: &Address,
    payee_id: &BytesN<32>,
    tranches: &Vec<TrancheInput>,
    ref_hash: &BytesN<32>,
    expires_at: u64,
) -> Result<u64, Error> {
    sender.require_auth();
    storage::extend_instance_ttl(env);
    let mut config = storage::read_config(env)?;

    if config.paused_new_locks {
        return Err(Error::PausedNewLocks);
    }
    if !storage::is_token_allowed(env, token) {
        return Err(Error::TokenNotAllowed);
    }
    let payee = storage::read_payee(env, payee_id).ok_or(Error::PayeeNotFound)?;
    if payee.status != PayeeStatus::Active {
        return Err(Error::PayeeNotActive);
    }

    let count = tranches.len();
    if count == 0 || count > MAX_TRANCHES {
        return Err(Error::InvalidTrancheCount);
    }
    let now = env.ledger().timestamp();
    let earliest = now
        .checked_add(MIN_EXPIRY_AHEAD_SECS)
        .ok_or(Error::Overflow)?;
    let latest = now
        .checked_add(MAX_LOCK_DURATION_SECS)
        .ok_or(Error::Overflow)?;
    if expires_at < earliest {
        return Err(Error::ExpiryTooSoon);
    }
    if expires_at > latest {
        return Err(Error::ExpiryTooFar);
    }

    let mut total: i128 = 0;
    let mut previous_unlock: u64 = 0;
    let mut stored = Vec::new(env);
    for t in tranches.iter() {
        if t.amount <= 0 {
            return Err(Error::InvalidTrancheAmount);
        }
        if t.unlock_at < previous_unlock {
            return Err(Error::UnlockOutOfOrder);
        }
        if t.unlock_at > expires_at {
            return Err(Error::UnlockAfterExpiry);
        }
        total = total.checked_add(t.amount).ok_or(Error::Overflow)?;
        previous_unlock = t.unlock_at;
        stored.push_back(Tranche {
            amount: t.amount,
            unlock_at: t.unlock_at,
            released: false,
        });
    }
    if total < MIN_AMOUNT {
        return Err(Error::AmountBelowMinimum);
    }
    if total > config.max_lock_amount {
        return Err(Error::AmountAboveLockCap);
    }
    if *sender == payee.payout {
        return Err(Error::SenderIsPayout);
    }
    let new_total_locked = config
        .total_locked
        .checked_add(total)
        .ok_or(Error::Overflow)?;
    if new_total_locked > config.max_total_locked {
        return Err(Error::GlobalCapExceeded);
    }

    let id = config.next_lock_id;
    config.next_lock_id = id.checked_add(1).ok_or(Error::Overflow)?;
    config.total_locked = new_total_locked;
    let lock = Lock {
        id,
        sender: sender.clone(),
        payee_id: payee_id.clone(),
        payout: payee.payout.clone(),
        token: token.clone(),
        total,
        released: 0,
        returned: 0,
        ref_hash: ref_hash.clone(),
        tranches: stored,
        expires_at,
        state: LockState::Open,
        created_at: now,
    };

    // State first, then the transfer: if the transfer fails, everything reverts.
    storage::write_lock(env, &lock);
    storage::write_config(env, &config);
    let until = keep_until(expires_at)?;
    storage::extend_lock_ttl_until(env, id, until)?;
    storage::extend_payee_ttl_until(env, payee_id, until)?;

    token::Client::new(env, token).transfer(
        sender,
        MuxedAddress::from(env.current_contract_address()),
        &total,
    );

    LockCreated {
        id,
        schema_version: EVENT_SCHEMA_VERSION,
        sender: sender.clone(),
        payee_id: payee_id.clone(),
        payout: lock.payout,
        token: token.clone(),
        total,
        ref_hash: ref_hash.clone(),
        expires_at,
        tranche_count: count,
    }
    .publish(env);
    Ok(id)
}

pub fn release(env: &Env, lock_id: u64, idx: u32) -> Result<(), Error> {
    storage::extend_instance_ttl(env);
    let mut lock = storage::read_lock(env, lock_id).ok_or(Error::LockNotFound)?;
    lock.payout.require_auth();
    if lock.state != LockState::Open {
        return Err(Error::LockNotOpen);
    }
    let payee = storage::read_payee(env, &lock.payee_id).ok_or(Error::PayeeNotFound)?;
    if payee.status != PayeeStatus::Active {
        return Err(Error::PayeeNotActive);
    }
    let mut tranche = lock
        .tranches
        .get(idx)
        .ok_or(Error::TrancheIndexOutOfRange)?;
    if tranche.released {
        return Err(Error::TrancheAlreadyReleased);
    }
    let now = env.ledger().timestamp();
    if now < tranche.unlock_at {
        return Err(Error::TrancheNotUnlocked);
    }
    if now >= lock.expires_at {
        return Err(Error::LockExpired);
    }

    let amount = tranche.amount;
    tranche.released = true;
    lock.tranches.set(idx, tranche);
    lock.released = lock.released.checked_add(amount).ok_or(Error::Overflow)?;
    let settled = lock
        .released
        .checked_add(lock.returned)
        .ok_or(Error::Overflow)?;
    if settled == lock.total {
        lock.state = LockState::Completed;
    }
    let mut config = storage::read_config(env)?;
    config.total_locked = config
        .total_locked
        .checked_sub(amount)
        .ok_or(Error::Overflow)?;

    // State first, then the transfer.
    storage::write_lock(env, &lock);
    storage::write_config(env, &config);
    transfer_out(env, &lock.token, &lock.payout, amount);

    Released {
        id: lock_id,
        schema_version: EVENT_SCHEMA_VERSION,
        idx,
        amount,
        payout: lock.payout,
    }
    .publish(env);
    Ok(())
}

/// Moves an Open lock to a terminal `state`, returning the unreleased remainder to the
/// sender. Returns the amount returned.
fn close_to_sender(env: &Env, lock: &mut Lock, state: LockState) -> Result<i128, Error> {
    let remainder = lock
        .total
        .checked_sub(lock.released)
        .and_then(|r| r.checked_sub(lock.returned))
        .ok_or(Error::Overflow)?;
    lock.returned = lock
        .returned
        .checked_add(remainder)
        .ok_or(Error::Overflow)?;
    lock.state = state;
    let mut config = storage::read_config(env)?;
    config.total_locked = config
        .total_locked
        .checked_sub(remainder)
        .ok_or(Error::Overflow)?;

    // State first, then the transfer.
    storage::write_lock(env, lock);
    storage::write_config(env, &config);
    transfer_out(env, &lock.token, &lock.sender, remainder);
    Ok(remainder)
}

pub fn refund(env: &Env, lock_id: u64) -> Result<(), Error> {
    storage::extend_instance_ttl(env);
    let mut lock = storage::read_lock(env, lock_id).ok_or(Error::LockNotFound)?;
    lock.sender.require_auth();
    if lock.state != LockState::Open {
        return Err(Error::LockNotOpen);
    }

    let now = env.ledger().timestamp();
    let reason = if now >= lock.expires_at {
        RefundReason::Expired
    } else {
        let payee = storage::read_payee(env, &lock.payee_id).ok_or(Error::PayeeNotFound)?;
        let grace_ends = payee
            .status_changed_at
            .saturating_add(SUSPENSION_REFUND_GRACE_SECS);
        match payee.status {
            PayeeStatus::Revoked => RefundReason::Revoked,
            PayeeStatus::Suspended if now >= grace_ends => RefundReason::SuspendedTimeout,
            _ => return Err(Error::RefundNotAllowed),
        }
    };

    let amount = close_to_sender(env, &mut lock, LockState::Refunded)?;
    Refunded {
        id: lock_id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount,
        reason,
    }
    .publish(env);
    Ok(())
}

pub fn decline(env: &Env, lock_id: u64) -> Result<(), Error> {
    storage::extend_instance_ttl(env);
    let mut lock = storage::read_lock(env, lock_id).ok_or(Error::LockNotFound)?;
    lock.payout.require_auth();
    if lock.state != LockState::Open {
        return Err(Error::LockNotOpen);
    }

    let amount = close_to_sender(env, &mut lock, LockState::Declined)?;
    Declined {
        id: lock_id,
        schema_version: EVENT_SCHEMA_VERSION,
        amount,
    }
    .publish(env);
    Ok(())
}

/// Permissionless: anyone may keep an Open lock and its payee alive in storage.
pub fn bump_lock(env: &Env, lock_id: u64) -> Result<(), Error> {
    storage::extend_instance_ttl(env);
    let lock = storage::read_lock(env, lock_id).ok_or(Error::LockNotFound)?;
    if lock.state != LockState::Open {
        return Err(Error::LockNotOpen);
    }
    let until = keep_until(lock.expires_at)?;
    storage::extend_lock_ttl_until(env, lock_id, until)?;
    storage::extend_payee_ttl_until(env, &lock.payee_id, until)?;
    Ok(())
}

pub fn get_lock(env: &Env, lock_id: u64) -> Result<Lock, Error> {
    storage::read_lock(env, lock_id).ok_or(Error::LockNotFound)
}
