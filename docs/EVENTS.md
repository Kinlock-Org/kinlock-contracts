> Synced from Kinlock-Org/.github. Do not edit here.

# Kinlock Contract Events Reference

This document is the authoritative specification for all events emitted by the `kinlock` Soroban smart contract.

Events are consumed by the Kinlock indexer (`kinlock-sdk/services/indexer`) to maintain materialized Postgres state for exploratory list queries, and are verifiable directly on-chain via archive nodes.

## Event Invariants and Standards

1. **Schema Versioning**: Every event struct carries `pub schema_version: u32`. When emitted, this field equals `constants::EVENT_SCHEMA_VERSION` (currently `1`). Any structural change to an event payload requires incrementing `EVENT_SCHEMA_VERSION` and upgrading the indexer schema.
2. **Numeric Representation & Scale**: All token amounts are represented as signed 128-bit integers (`i128`) in atomic base units (stroops). For USDC on Stellar (via the Stellar Asset Contract / SEP-41), 1 USDC equals $10^7$ base units (`10_000_000`). **Consumers must parse amounts as BigInt or fixed-point strings, never as IEEE-754 floating-point numbers**.
3. **Payout Snapshotting (Hard Rule 2)**: The `LockCreated` event records the payee's payout address (`payout: Address`) at the exact block height when the lock was instantiated. On-chain, `lock.payout` is immutable. If the payee later rotates their address in the registry via `update_payout`, existing open locks continue to release only to their snapshotted address.

---

## Registry Events (`registry.rs`)

### 1. `PayeeRegistered`
Emitted when an authorized attester registers a new payee in the registry (`registry.rs:48`).

- **Topic**: `payee_id: BytesN<32>` (unique 32-byte hash identifying the payee)
- **Data Payload**:
  - `schema_version`: `u32` — Event schema version (`1`).
  - `payout`: `Address` — Stellar address (G... or C...) designated to receive released lock funds.
  - `category`: `Category` — Enum representation of the payee category (`School = 0`, `Healthcare = 1`, `Contractor = 2`, `Supplier = 3`, `Other = 4`).
  - `attester`: `Address` — Stellar address of the vouching attester on the active roster.
  - `meta_hash`: `BytesN<32>` — SHA-256 hash of the canonical off-chain payee metadata record.

```json
{
  "topic": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "data": {
    "schema_version": 1,
    "payout": "GBBD7U7HUR7P7WD47Y6XAUDFB4D36R6P6P6K4L5P7N7N3M2J1K0L9M8N",
    "category": 0,
    "attester": "GAATT3ST3RROST3R7Y6XAUDFB4D36R6P6P6K4L5P7N7N3M2J1K0L9M8N",
    "meta_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
  }
}
```

---

### 2. `PayeeStatusChanged`
Emitted when an authorized attester or admin updates the operating status of a payee (`registry.rs:100`).

- **Topic**: `payee_id: BytesN<32>` — Payee identifier.
- **Data Payload**:
  - `schema_version`: `u32` — Event schema version (`1`).
  - `status`: `PayeeStatus` — Updated status enum (`Active = 0`, `Suspended = 1`, `Revoked = 2`).
  - `changed_by`: `Address` — Stellar address of the attester or admin who executed the status change.

```json
{
  "topic": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "data": {
    "schema_version": 1,
    "status": 1,
    "changed_by": "GAATT3ST3RROST3R7Y6XAUDFB4D36R6P6P6K4L5P7N7N3M2J1K0L9M8N"
  }
}
```

---

### 3. `PayoutUpdated`
Emitted when the vouching attester updates the destination payout address for future locks created for a payee (`registry.rs:137`).

- **Topic**: `payee_id: BytesN<32>` — Payee identifier.
- **Data Payload**:
  - `schema_version`: `u32` — Event schema version (`1`).
  - `new_payout`: `Address` — Newly configured payout destination address for subsequent locks.

*Note: Existing locks previously created under this payee retain their original snapshotted payout address.*

```json
{
  "topic": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "data": {
    "schema_version": 1,
    "new_payout": "GCNEWPAYOUT47Y6XAUDFB4D36R6P6P6K4L5P7N7N3M2J1K0L9M8N1234"
  }
}
```

---

## Vault Events (`vault.rs`)

### 4. `LockCreated`
Emitted when a sender successfully instantiates an escrow lock in the vault (`vault.rs:147`).

- **Topic**: `id: u64` — Monotonically increasing lock ID assigned by the contract.
- **Data Payload**:
  - `schema_version`: `u32` — Event schema version (`1`).
  - `sender`: `Address` — Stellar account or contract that deposited the funds.
  - `payee_id`: `BytesN<32>` — Target verified payee in the registry.
  - `payout`: `Address` — **Immutable payout snapshot**. Destination for all subsequent releases for this lock.
  - `token`: `Address` — Allowlisted token address (e.g. SAC USDC).
  - `total`: `i128` — Total locked amount in base units (stroops). For 100 USDC, `total = 1_000_000_000`.
  - `ref_hash`: `BytesN<32>` — Double-hashed claim reference (`sha256(reference || salt)`).
  - `expires_at`: `u64` — Unix timestamp (seconds) when lock expires and becomes refundable.
  - `tranche_count`: `u32` — Total number of scheduled tranches in this lock (1 to 12).

```json
{
  "topic": 11,
  "data": {
    "schema_version": 1,
    "sender": "GBSENDER47Y6XAUDFB4D36R6P6P6K4L5P7N7N3M2J1K0L9M8N0001",
    "payee_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "payout": "GBBD7U7HUR7P7WD47Y6XAUDFB4D36R6P6P6K4L5P7N7N3M2J1K0L9M8N",
    "token": "CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75",
    "total": 1000000000,
    "ref_hash": "0xdeadbeef1234567890abcdef1234567890abcdef1234567890abcdef12345678",
    "expires_at": 1792400000,
    "tranche_count": 2
  }
}
```

---

### 5. `Released`
Emitted when an authorized payout caller triggers release of a specific tranche (`vault.rs:211`).

- **Topic**: `id: u64` — Lock identifier.
- **Data Payload**:
  - `schema_version`: `u32` — Event schema version (`1`).
  - `idx`: `u32` — 0-based tranche index that was released.
  - `amount`: `i128` — Released tranche amount transferred to payout address in base units.
  - `payout`: `Address` — Destination address where released funds were sent.

```json
{
  "topic": 11,
  "data": {
    "schema_version": 1,
    "idx": 0,
    "amount": 500000000,
    "payout": "GBBD7U7HUR7P7WD47Y6XAUDFB4D36R6P6P6K4L5P7N7N3M2J1K0L9M8N"
  }
}
```

---

### 6. `Refunded`
Emitted when unreleased escrow funds are refunded back to the sender (`vault.rs:272`).

- **Topic**: `id: u64` — Lock identifier.
- **Data Payload**:
  - `schema_version`: `u32` — Event schema version (`1`).
  - `amount`: `i128` — Total refunded amount returned to sender in base units (remainder of unreleased tranches).
  - `reason`: `RefundReason` — Reason for refund (`Expired = 0`, `Revoked = 1`, `SuspendedTimeout = 2`).

```json
{
  "topic": 11,
  "data": {
    "schema_version": 1,
    "amount": 500000000,
    "reason": 0
  }
}
```

---

### 7. `Declined`
Emitted when the payee's payout address explicitly declines an open lock, returning all remaining unreleased funds to the sender (`vault.rs:291`).

- **Topic**: `id: u64` — Lock identifier.
- **Data Payload**:
  - `schema_version`: `u32` — Event schema version (`1`).
  - `amount`: `i128` — Total unreleased amount returned to sender in base units upon payee decline.

```json
{
  "topic": 11,
  "data": {
    "schema_version": 1,
    "amount": 500000000
  }
}
```
