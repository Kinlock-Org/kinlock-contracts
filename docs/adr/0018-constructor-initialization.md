> Synced from Kinlock-Org/.github. Do not edit here.

# 0018 — Initialize the contract in its constructor
Status: Accepted
Date: 2026-10-05
## Context
`ARCHITECTURE.md` §4.3 specifies a one-time `init(admin)` entry point. A separate `init` call can be front-run: between deployment and the `init` transaction, anyone could call `init` with their own address and become admin. Soroban supports a `__constructor` that runs atomically as part of deployment.
## Decision
`__constructor(admin)` sets the initial config (admin, not paused, caps unlimited, `total_locked = 0`, `next_lock_id = 1`). There is no `init` entry point. Registry entry points that need a caller's signature take the caller explicitly (`register_payee(attester, …)`, `set_status(caller, …)`, `update_payout(attester, …)`), so the contract knows whose authorization to require.
## Consequences / trade-offs
Initialization can't be front-run. Deploy scripts must pass the admin as a constructor argument. The SDK and bindings must pass the caller address on those three calls.
## Docs updated
Pending roadmap F-19: `ARCHITECTURE.md` §4.3 and `ARCHITECTURE_ESSENTIALS.md` §5 still list `init` and the old signatures.
