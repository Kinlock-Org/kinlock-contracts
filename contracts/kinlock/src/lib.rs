#![no_std]
//! Kinlock: purpose-locked USDC transfers to verified payees.
//!
//! One contract, two modules (`registry`, `vault`) plus `admin`.
//! Design: `docs/ARCHITECTURE.md` §4. Invariants: §4.6. Hard rules: `AGENTS.md` §3.
//!
//! The contract is country-agnostic (AGENTS.md hard rule 11): it has no notion of
//! country, currency, or locale. Those live in `kinlock-registry` data only.
//!
//! SCAFFOLD: data model, errors, events, and storage keys are drafted. Entry points are
//! specified in `admin.rs`, `registry.rs`, and `vault.rs` and will be added in M1-03..M1-09.

use soroban_sdk::contract;

pub mod admin;
pub mod constants;
pub mod errors;
pub mod events;
pub mod registry;
pub mod storage;
pub mod types;
pub mod vault;

#[contract]
pub struct Kinlock;

#[cfg(test)]
mod test;
