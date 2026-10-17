# Testing Kinlock Contracts

This document explains the testing layout, requirements, and execution commands for the Kinlock smart contracts suite.

## Toolchain Requirements

- **Rust toolchain:** Pinned in `rust-toolchain.toml` (Rust 1.96.1, `wasm32v1-none` target).
- **Soroban SDK:** `soroban-sdk = "28.0.0"`.
- **Stellar CLI:** Installed locally for contract compilation (`stellar contract build`).

## Test Layout

1. **Unit Tests (`contracts/kinlock/src/test/`):**
   - 107 unit tests verifying individual functions, entry points, access controls (`admin_tests.rs`, `auth_tests.rs`), lock lifecycle (`create_lock_tests.rs`, `release_tests.rs`, `refund_decline_tests.rs`), TTL extensions (`ttl_tests.rs`), and contract upgrades (`upgrade_tests.rs`).
2. **Property Tests (`contracts/kinlock/tests/properties.rs`):**
   - Proptest harness testing 64 random sequences of operations against invariant guarantees defined in `ARCHITECTURE.md` §4.6.
3. **Integration Test Suite (`contracts/kinlock/tests/integration.rs`):**
   - End-to-end integration scenarios verifying real token topology, contract callers, trustline failures, and lifecycle interactions:
     - **Scenario (a):** Classic G-address sender to C-address payee contract (verifying explicit invocation auths and token delivery).
     - **Scenario (b):** C-address sender (contract wallet caller, requiring sub-invocation auth for token transfer, partial release, and remaining refund custody).
     - **Scenario (c):** Missing or unauthorized USDC trustline at release time (asserts call reverts without partial state change, followed by successful release once trustline authorization is established).
     - **Scenario (d):** Partial release followed by refund of remainder, and decline with post-release remainder (verifying that refunded/declined amounts equal post-release remainder, never the original total).
     - **Scenario (e):** Allowlist removal mid-flight (verifying new lock creation is rejected with `TokenNotAllowed`, while existing open locks release, refund, and decline without disruption).

## Running Tests

### 1. Build Contract WASM

The unit test suite includes an upgrade test that inspects the compiled wasm bytecode:

```bash
stellar contract build
```

This compiles the optimized wasm to `target/wasm32v1-none/release/kinlock.wasm`.

### 2. Run Integration Tests

Run the integration suite directly:

```bash
cargo test --test integration
```

### 3. Run Property Tests

Run the property test suite:

```bash
cargo test --test properties
```

### 4. Run Entire Workspace Suite

Run all unit, integration, and property tests across the workspace:

```bash
cargo test --workspace
```

### 5. Formatting and Lint Verification

Before submitting PRs, verify zero warnings and clean formatting:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```
