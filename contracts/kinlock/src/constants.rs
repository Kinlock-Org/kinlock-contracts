//! Contract constants. Single source of truth: `docs/ARCHITECTURE.md` §4.1.
//! Changing any value here is ask-first, needs an ADR, and is `security-sensitive`.

/// Maximum tranches per lock. Termly fees = 3, monthly rent = up to 12.
pub const MAX_TRANCHES: u32 = 12;

/// Dust guard: 1 whole token in base units, assuming the 7-decimal SAC convention used by USDC.
pub const MIN_AMOUNT: i128 = 10_000_000;

/// `expires_at` must be at least this far ahead of `now` (seconds). Prevents instant-expiry locks.
pub const MIN_EXPIRY_AHEAD_SECS: u64 = 60 * 60;

/// Maximum lock lifetime (seconds). Pending DEC-07 / M0-08: must stay below the network's
/// max entry TTL. 180 days until verified.
pub const MAX_LOCK_DURATION_SECS: u64 = 180 * 24 * 60 * 60;

/// After a payee is Suspended, the sender may refund once this much time has passed (seconds).
pub const SUSPENSION_REFUND_GRACE_SECS: u64 = 14 * 24 * 60 * 60;

/// Extra TTL kept beyond `expires_at` so senders can still refund after expiry (seconds).
pub const TTL_GRACE_SECS: u64 = 30 * 24 * 60 * 60;

/// Approximate ledger close time used to convert seconds to TTL ledgers.
/// Unverified: confirm against current network settings in M0-08.
pub const APPROX_LEDGER_CLOSE_SECS: u64 = 5;

/// Long-lived entries (config, attesters, tokens, payees) are extended to the network's
/// max TTL, but only once their remaining TTL has dropped this far below it. Bounds the
/// cost of keeping them alive to roughly one extension per day per entry.
pub const TTL_REFRESH_WINDOW_SECS: u64 = 24 * 60 * 60;

/// Version stamped on every event. Bump when any event's shape changes; the indexer must
/// handle every version ever emitted.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Layout version of contract storage, written at deploy. An upgrade that changes the stored
/// layout must bump this and migrate (ARCHITECTURE.md §4.7).
pub const STORAGE_VERSION: u32 = 1;
