//! On-chain data model. Source: `docs/ARCHITECTURE.md` §4.2.
//!
//! Stored enums are APPEND-ONLY: add new variants at the end, never reorder or remove.
//! No personal data and no country/currency fields, ever.

use soroban_sdk::{contracttype, Address, BytesN, Vec};

/// Instance storage: global configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// Admin multisig.
    pub admin: Address,
    pub next_lock_id: u64,
    /// Blocks `create_lock` only. Never blocks release, decline, or refund.
    pub paused_new_locks: bool,
    /// P1 mainnet cap per lock.
    pub max_lock_amount: i128,
    /// P1 mainnet cap on `total_locked`.
    pub max_total_locked: i128,
    /// Sum of remainders (`total - released - returned`) of Open locks. Invariant 9.
    pub total_locked: i128,
}

/// Payee category. Purpose of a lock = its payee's category. APPEND-ONLY.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Category {
    School,
    Rent,
}

/// APPEND-ONLY. Active ⇄ Suspended; Revoked is terminal.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayeeStatus {
    Active,
    Suspended,
    Revoked,
}

/// APPEND-ONLY. Open → Completed | Refunded | Declined.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockState {
    Open,
    Completed,
    Refunded,
    Declined,
}

/// Why a refund was allowed. Carried in the `Refunded` event. APPEND-ONLY.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefundReason {
    Expired,
    Revoked,
    SuspendedTimeout,
}

/// A verified payee. Keyed by `payee_id = sha256(registry slug)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payee {
    /// G or C address. Updating it affects NEW locks only.
    pub payout: Address,
    pub category: Category,
    pub status: PayeeStatus,
    pub status_changed_at: u64,
    /// The vouching attester.
    pub attester: Address,
    /// SHA-256 of the canonical registry JSON.
    pub meta_hash: BytesN<32>,
    pub registered_at: u64,
}

/// A scheduled portion of a lock.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tranche {
    pub amount: i128,
    pub unlock_at: u64,
    pub released: bool,
}

/// Caller-supplied tranche schedule for `create_lock` (no `released` flag to forge).
/// Scaffold addition: `ARCHITECTURE.md` §4.3 only says "tranches"; confirm in M1-05 review.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrancheInput {
    pub amount: i128,
    pub unlock_at: u64,
}

/// A funded lock.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lock {
    pub id: u64,
    pub sender: Address,
    pub payee_id: BytesN<32>,
    /// SNAPSHOT of `payee.payout` at creation. Immutable (invariant 7).
    pub payout: Address,
    pub token: Address,
    pub total: i128,
    pub released: i128,
    /// Refunded or declined.
    pub returned: i128,
    /// sha256(reference || salt). The reference itself never touches the chain.
    pub ref_hash: BytesN<32>,
    /// 1..=MAX_TRANCHES, non-decreasing `unlock_at`, each `unlock_at <= expires_at`.
    pub tranches: Vec<Tranche>,
    pub expires_at: u64,
    pub state: LockState,
    pub created_at: u64,
}
