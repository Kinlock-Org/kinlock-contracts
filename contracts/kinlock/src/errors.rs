//! Contract errors. DRAFT: variant set to be confirmed in M1-02 review.
//! Codes are part of the public interface: never renumber; add new codes at the end.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Setup and admin
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAttester = 3,
    NotVouchingAttester = 4,
    InvalidCap = 5,
    // Registry
    PayeeAlreadyExists = 10,
    PayeeNotFound = 11,
    PayeeNotActive = 12,
    InvalidStatusTransition = 13,
    PayoutUnchanged = 14,
    PayeeRevoked = 15,
    InvalidPayout = 16,
    // create_lock validation
    PausedNewLocks = 20,
    TokenNotAllowed = 21,
    AmountBelowMinimum = 22,
    AmountAboveLockCap = 23,
    GlobalCapExceeded = 24,
    InvalidTrancheCount = 25,
    InvalidTrancheAmount = 26,
    TrancheSumMismatch = 27,
    UnlockAfterExpiry = 28,
    UnlockOutOfOrder = 29,
    ExpiryTooSoon = 30,
    ExpiryTooFar = 31,
    SenderIsPayout = 32,
    LockTtlTooLong = 33,
    // Lock actions
    LockNotFound = 40,
    LockNotOpen = 41,
    TrancheIndexOutOfRange = 42,
    TrancheAlreadyReleased = 43,
    TrancheNotUnlocked = 44,
    LockExpired = 45,
    RefundNotAllowed = 46,
    // Arithmetic
    Overflow = 50,
}
