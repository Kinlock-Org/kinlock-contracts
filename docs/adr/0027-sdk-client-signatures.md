> Synced from Kinlock-Org/.github. Do not edit here.

# 0027 — SDK client: config and signer parameters
Status: Accepted
Date: 2026-10-07
## Context
The SDK's five contract functions were scaffolded without a network or a signer (`release(lockId, idx)`), which can't work: each call needs the RPC, network passphrase and contract, and every write needs the right account's signature. The SDK must stay non-custodial (hard rule 4), and the app's wallets go through Stellar Wallets Kit (`lib/wallet`).
## Decision
The five functions keep their names and gain a config first argument and, for writes, a signer last (owner-approved over a client factory):
- `createLock(config, params, signer)`, `release(config, { lockId, trancheIndex }, signer)`, `refund(config, { lockId }, signer)`, `decline(config, { lockId }, signer)`, `getLock(config, lockId)`.
- New public types: `KinlockConfig` (`rpcUrl`, `networkPassphrase`, `contractId`, optional `allowHttp`) and `Signer` (`address`, `signTransaction` with the Stellar Wallets Kit / `@stellar/stellar-sdk` `SignTransaction` shape). The SDK never sees a key.
- New `KinlockError` codes: `INVALID_INPUT`, `CONTRACT_ERROR` (with `contractError`, the contract's error name such as `RefundNotAllowed`), and `TX_FAILED` (signing declined, submission or confirmation failed).
- Writes run build → simulate → sign → submit → wait. A contract error in simulation stops before the wallet is asked to sign. `getLock` returns `null` for an unknown lock and reads chain state only.
## Consequences / trade-offs
No new functions, so the export list stays at fifteen; the app keeps one `KinlockConfig` in `lib/sdk.ts`. Callers pass the config on every call. These signatures are now a public contract: changing them needs approval and a version bump.
## Docs updated
Amends ADR-0025's signatures (names unchanged). Roadmap M2-02.
