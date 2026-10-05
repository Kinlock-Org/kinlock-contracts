//! Contract events. Source: `docs/ARCHITECTURE.md` §4.4.
//! Every event carries `schema_version` (= `constants::EVENT_SCHEMA_VERSION` when emitted).
//! Changing an event's shape is ask-first and requires indexer handler updates.

use crate::types::{Category, PayeeStatus, RefundReason};
use soroban_sdk::{contractevent, Address, BytesN};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayeeRegistered {
    #[topic]
    pub payee_id: BytesN<32>,
    pub schema_version: u32,
    pub payout: Address,
    pub category: Category,
    pub attester: Address,
    pub meta_hash: BytesN<32>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayeeStatusChanged {
    #[topic]
    pub payee_id: BytesN<32>,
    pub schema_version: u32,
    pub status: PayeeStatus,
    pub changed_by: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayoutUpdated {
    #[topic]
    pub payee_id: BytesN<32>,
    pub schema_version: u32,
    pub new_payout: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockCreated {
    #[topic]
    pub id: u64,
    pub schema_version: u32,
    pub sender: Address,
    pub payee_id: BytesN<32>,
    pub payout: Address,
    pub token: Address,
    pub total: i128,
    pub ref_hash: BytesN<32>,
    pub expires_at: u64,
    pub tranche_count: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Released {
    #[topic]
    pub id: u64,
    pub schema_version: u32,
    pub idx: u32,
    pub amount: i128,
    pub payout: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refunded {
    #[topic]
    pub id: u64,
    pub schema_version: u32,
    pub amount: i128,
    pub reason: RefundReason,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declined {
    #[topic]
    pub id: u64,
    pub schema_version: u32,
    pub amount: i128,
}
