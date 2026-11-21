> Synced from Kinlock-Org/.github. Do not edit here.

# 0031 — Universal Dual-Mode Locks (Direct and Verified)
Status: Accepted
Date: 2026-10-28
## Context
Kinlock originally restricted all payments to verified institutional payees (School and Rent) registered on-chain via attesters. To support universal payments by anyone to any Stellar recipient (classic `G...` or contract `C...` address) while preserving institutional protections for vetted payees, the contract, SDK, and application needed to support dual payment modes.
## Decision
Implement a Dual-Mode Architecture (Option 1 from the approved pivot plan):
1. **Contract (`kinlock-contracts`):**
   - Retain `create_lock` for verified institutional payees (validating `read_payee` is Active, snapshotting payee payout address).
   - Add `create_direct_lock(env, sender, token, recipient, tranches, ref_hash, expires_at) -> u64`, delegating to shared validation logic with `payee_id = None` and `payout = recipient`.
   - Update `Lock` struct: `payee_id: Option<BytesN<32>>`.
   - Update `release`: if `payee_id` is None, bypass `read_payee` Active check (direct locks do not require an active registry entry to release).
   - Update `refund`: if `payee_id` is None, early refunds for Revoked/Suspended payee status are skipped; refunds are permitted strictly when `now >= expires_at`.
   - Fix M1-35 tranche boundary: enforce `unlock_at < expires_at` (rejecting `unlock_at >= expires_at` with `UnlockAfterExpiry`), ensuring all tranches can be claimed before expiry.
2. **SDK (`kinlock-sdk`):**
   - Update `Lock.payeeId` to `Hash32 | null`.
   - Update `createLock` to accept either `payeeId` or `recipient: Address`. When `recipient` is provided, invoke `create_direct_lock`.
   - Update `preflight`: for direct locks, skip `payee_active` and `recent_payout_change` checks while verifying `payout_trustline_authorized`, `sender_balance`, and `duplicate_ref_hash`.
3. **Application (`kinlock-app`):**
   - Add mode toggle in `/send`: "Verified payee" vs "Any Stellar address".
   - Render recipient address input for direct mode.
   - Direct lock receipts and views display "Payment to recipient" rather than "Payment to verified payee" (adhering to Rule 7).
## Consequences / trade-offs
- Anyone anywhere can lock funds to any Stellar address with scheduled tranches and privacy-preserving reference hashes.
- Verified payees retain attester guarantees and institutional directories without breaking changes to existing flows.
- Receipts honestly reflect the payment tier: verified payee vs direct recipient.
## Docs updated
`PRD.md`, `ARCHITECTURE.md`, `ROADMAP.md`.
