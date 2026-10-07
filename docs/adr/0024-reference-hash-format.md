> Synced from Kinlock-Org/.github. Do not edit here.

# 0024 — Reference hash and claim-link format
Status: Accepted
Date: 2026-10-06
## Context
ADR-0005 keeps a payment's reference (student ID, unit, invoice) off-chain: only a salted hash is stored, and the reference and salt travel in the claim link's URL fragment. "sha256(reference ‖ salt)" left the exact bytes open, and every client (app, SDK, third-party verifiers) must compute the same hash or a payee's check fails.
## Decision
- **Salt:** 16 bytes from a cryptographically secure RNG, written in links as unpadded base64url (22 characters).
- **Reference:** Unicode-normalized to NFC and trimmed of surrounding whitespace; 1 to 200 characters after that.
- **ref_hash:** SHA-256 over the reference's UTF-8 bytes followed by the 16 raw salt bytes, stored as 32 bytes (lowercase hex off-chain). The fixed-length salt makes the concatenation unambiguous.
- **Claim link:** `{origin}/claim/{lockId}#r={reference}&s={salt}`, with the fragment URL-encoded. Origins are https (http only for localhost). Nothing derived from the fragment is ever logged, stored server-side, or sent to a third party.
- Implemented once, in `kinlock-sdk` `hash.ts` and `links.ts`.
## Consequences / trade-offs
Two devices typing the same reference get the same hash despite spacing or composition differences. Case is not normalized ("stu-1" and "STU-1" differ), because institutions' references can be case-sensitive. Changing any of this later would break every existing claim link, so it needs a new link version.
## Docs updated
Refines ADR-0005; `ARCHITECTURE.md` §5.2 describes the link at a high level.
