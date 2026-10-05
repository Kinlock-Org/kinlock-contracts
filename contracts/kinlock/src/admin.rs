//! Admin entry points (admin multisig). Spec: `docs/ARCHITECTURE.md` §4.3. Row: M1-03.
//!
//! Planned (not yet implemented):
//! - `init(admin)`: one-time setup.
//! - `add_attester(attester)` / `remove_attester(attester)`: roster management.
//! - `add_token(token)` / `remove_token(token)`: allowlist; removal only blocks NEW locks.
//! - `set_paused_new_locks(paused)`: blocks `create_lock` only.
//! - `set_caps(max_lock, max_total)`: P1 mainnet gating.
//! - `upgrade(wasm_hash)`: multisig now; timelock before mainnet (X-07).
//!
//! Hard rule: nothing here may move, freeze, or redirect funds in existing locks.
