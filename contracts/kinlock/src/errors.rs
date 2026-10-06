//! Contract errors. Codes are part of the public interface: never renumber or reuse a code;
//! add new codes at the end of their group.
//!
//! Each variant's doc comment is published in the contract spec and becomes the error message
//! TypeScript clients see, so keep it in the form `Name: plain-language explanation`.
//!
//! Retired before any deployment (never returned, so never published):
//! 1 = AlreadyInitialized (initialization moved into the constructor, ADR-0018) and
//! 27 = TrancheSumMismatch (`total` is now derived from the tranches). Don't reuse 1 or 27.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Setup and admin
    /// NotInitialized: the contract has no configuration (it was not initialized).
    NotInitialized = 2,
    /// NotAttester: the caller is not on the attester roster.
    NotAttester = 3,
    /// NotVouchingAttester: only the payee's vouching attester (still on the roster), or the admin where allowed, can do this.
    NotVouchingAttester = 4,
    /// InvalidCap: caps must be at least the minimum lock amount, and the per-lock cap can't exceed the global cap.
    InvalidCap = 5,
    // Registry
    /// PayeeAlreadyExists: a payee with this ID is already registered.
    PayeeAlreadyExists = 10,
    /// PayeeNotFound: no payee is registered with this ID.
    PayeeNotFound = 11,
    /// PayeeNotActive: the payee is suspended or revoked.
    PayeeNotActive = 12,
    /// InvalidStatusTransition: that status change isn't allowed (Revoked is final; the status must change).
    InvalidStatusTransition = 13,
    /// PayoutUnchanged: the new payout address is the same as the current one.
    PayoutUnchanged = 14,
    /// PayeeRevoked: the payee is revoked, so its payout can't be updated.
    PayeeRevoked = 15,
    /// InvalidPayout: the payout address can't be the Kinlock contract itself.
    InvalidPayout = 16,
    // create_lock validation
    /// PausedNewLocks: new locks are paused; existing locks still work.
    PausedNewLocks = 20,
    /// TokenNotAllowed: this token isn't on the allowlist.
    TokenNotAllowed = 21,
    /// AmountBelowMinimum: the total is below the minimum lock amount.
    AmountBelowMinimum = 22,
    /// AmountAboveLockCap: the total is above the per-lock cap.
    AmountAboveLockCap = 23,
    /// GlobalCapExceeded: this lock would take the total locked above the global cap.
    GlobalCapExceeded = 24,
    /// InvalidTrancheCount: a lock needs between 1 and 12 tranches.
    InvalidTrancheCount = 25,
    /// InvalidTrancheAmount: every tranche amount must be greater than zero.
    InvalidTrancheAmount = 26,
    /// UnlockAfterExpiry: a tranche can't unlock after the lock expires.
    UnlockAfterExpiry = 28,
    /// UnlockOutOfOrder: tranche unlock times must not go backwards.
    UnlockOutOfOrder = 29,
    /// ExpiryTooSoon: the lock must expire at least one hour from now.
    ExpiryTooSoon = 30,
    /// ExpiryTooFar: the lock must expire within the maximum lock duration (149 days).
    ExpiryTooFar = 31,
    /// SenderIsPayout: the sender can't be the payee's payout address.
    SenderIsPayout = 32,
    /// LockTtlTooLong: the network can't keep this lock in storage until expiry plus the refund grace.
    LockTtlTooLong = 33,
    // Lock actions
    /// LockNotFound: no lock exists with this ID.
    LockNotFound = 40,
    /// LockNotOpen: the lock is already completed, refunded, or declined.
    LockNotOpen = 41,
    /// TrancheIndexOutOfRange: this lock has no tranche at that index.
    TrancheIndexOutOfRange = 42,
    /// TrancheAlreadyReleased: this tranche has already been released.
    TrancheAlreadyReleased = 43,
    /// TrancheNotUnlocked: this tranche isn't unlocked yet.
    TrancheNotUnlocked = 44,
    /// LockExpired: the lock has expired, so it can't be released; the sender can refund it.
    LockExpired = 45,
    /// RefundNotAllowed: a refund is allowed only after expiry, or if the payee is revoked or suspended past the grace period.
    RefundNotAllowed = 46,
    // Arithmetic
    /// Overflow: an amount calculation overflowed.
    Overflow = 50,
}
