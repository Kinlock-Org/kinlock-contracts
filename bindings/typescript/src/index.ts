import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}





/**
 * A funded lock.
 */
export interface Lock {
  created_at: u64;
  expires_at: u64;
  id: u64;
  payee_id: Buffer;
  /**
 * SNAPSHOT of `payee.payout` at creation. Immutable (invariant 7).
 */
payout: string;
  /**
 * sha256(reference || salt). The reference itself never touches the chain.
 */
ref_hash: Buffer;
  released: i128;
  /**
 * Refunded or declined.
 */
returned: i128;
  sender: string;
  state: LockState;
  token: string;
  total: i128;
  /**
 * 1..=MAX_TRANCHES, non-decreasing `unlock_at`, each `unlock_at <= expires_at`.
 */
tranches: Array<Tranche>;
}


/**
 * A verified payee. Keyed by `payee_id = sha256(registry slug)`.
 */
export interface Payee {
  /**
 * The vouching attester.
 */
attester: string;
  category: Category;
  /**
 * SHA-256 of the canonical registry JSON.
 */
meta_hash: Buffer;
  /**
 * G or C address. Updating it affects NEW locks only.
 */
payout: string;
  registered_at: u64;
  status: PayeeStatus;
  status_changed_at: u64;
}


/**
 * A scheduled portion of a lock.
 */
export interface Tranche {
  amount: i128;
  released: boolean;
  unlock_at: u64;
}

/**
 * Payee category. Purpose of a lock = its payee's category. APPEND-ONLY.
 */
export type Category = {tag: "School", values: void} | {tag: "Rent", values: void};

/**
 * APPEND-ONLY. Open → Completed | Refunded | Declined.
 */
export type LockState = {tag: "Open", values: void} | {tag: "Completed", values: void} | {tag: "Refunded", values: void} | {tag: "Declined", values: void};

/**
 * APPEND-ONLY. Active ⇄ Suspended; Revoked is terminal.
 */
export type PayeeStatus = {tag: "Active", values: void} | {tag: "Suspended", values: void} | {tag: "Revoked", values: void};

/**
 * Why a refund was allowed. Carried in the `Refunded` event. APPEND-ONLY.
 */
export type RefundReason = {tag: "Expired", values: void} | {tag: "Revoked", values: void} | {tag: "SuspendedTimeout", values: void};


/**
 * Caller-supplied tranche schedule for `create_lock` (no `released` flag to forge).
 * Scaffold addition: `ARCHITECTURE.md` §4.3 only says "tranches"; confirm in M1-05 review.
 */
export interface TrancheInput {
  amount: i128;
  unlock_at: u64;
}

export const Errors = {
  /**
   * NotInitialized: the contract has no configuration (it was not initialized).
   */
  2: {message:"NotInitialized"},
  /**
   * NotAttester: the caller is not on the attester roster.
   */
  3: {message:"NotAttester"},
  /**
   * NotVouchingAttester: only the payee's vouching attester (still on the roster), or the admin where allowed, can do this.
   */
  4: {message:"NotVouchingAttester"},
  /**
   * InvalidCap: caps must be at least the minimum lock amount, and the per-lock cap can't exceed the global cap.
   */
  5: {message:"InvalidCap"},
  /**
   * PayeeAlreadyExists: a payee with this ID is already registered.
   */
  10: {message:"PayeeAlreadyExists"},
  /**
   * PayeeNotFound: no payee is registered with this ID.
   */
  11: {message:"PayeeNotFound"},
  /**
   * PayeeNotActive: the payee is suspended or revoked.
   */
  12: {message:"PayeeNotActive"},
  /**
   * InvalidStatusTransition: that status change isn't allowed (Revoked is final; the status must change).
   */
  13: {message:"InvalidStatusTransition"},
  /**
   * PayoutUnchanged: the new payout address is the same as the current one.
   */
  14: {message:"PayoutUnchanged"},
  /**
   * PayeeRevoked: the payee is revoked, so its payout can't be updated.
   */
  15: {message:"PayeeRevoked"},
  /**
   * InvalidPayout: the payout address can't be the Kinlock contract itself.
   */
  16: {message:"InvalidPayout"},
  /**
   * PausedNewLocks: new locks are paused; existing locks still work.
   */
  20: {message:"PausedNewLocks"},
  /**
   * TokenNotAllowed: this token isn't on the allowlist.
   */
  21: {message:"TokenNotAllowed"},
  /**
   * AmountBelowMinimum: the total is below the minimum lock amount.
   */
  22: {message:"AmountBelowMinimum"},
  /**
   * AmountAboveLockCap: the total is above the per-lock cap.
   */
  23: {message:"AmountAboveLockCap"},
  /**
   * GlobalCapExceeded: this lock would take the total locked above the global cap.
   */
  24: {message:"GlobalCapExceeded"},
  /**
   * InvalidTrancheCount: a lock needs between 1 and 12 tranches.
   */
  25: {message:"InvalidTrancheCount"},
  /**
   * InvalidTrancheAmount: every tranche amount must be greater than zero.
   */
  26: {message:"InvalidTrancheAmount"},
  /**
   * UnlockAfterExpiry: a tranche can't unlock after the lock expires.
   */
  28: {message:"UnlockAfterExpiry"},
  /**
   * UnlockOutOfOrder: tranche unlock times must not go backwards.
   */
  29: {message:"UnlockOutOfOrder"},
  /**
   * ExpiryTooSoon: the lock must expire at least one hour from now.
   */
  30: {message:"ExpiryTooSoon"},
  /**
   * ExpiryTooFar: the lock must expire within the maximum lock duration (149 days).
   */
  31: {message:"ExpiryTooFar"},
  /**
   * SenderIsPayout: the sender can't be the payee's payout address.
   */
  32: {message:"SenderIsPayout"},
  /**
   * LockTtlTooLong: the network can't keep this lock in storage until expiry plus the refund grace.
   */
  33: {message:"LockTtlTooLong"},
  /**
   * LockNotFound: no lock exists with this ID.
   */
  40: {message:"LockNotFound"},
  /**
   * LockNotOpen: the lock is already completed, refunded, or declined.
   */
  41: {message:"LockNotOpen"},
  /**
   * TrancheIndexOutOfRange: this lock has no tranche at that index.
   */
  42: {message:"TrancheIndexOutOfRange"},
  /**
   * TrancheAlreadyReleased: this tranche has already been released.
   */
  43: {message:"TrancheAlreadyReleased"},
  /**
   * TrancheNotUnlocked: this tranche isn't unlocked yet.
   */
  44: {message:"TrancheNotUnlocked"},
  /**
   * LockExpired: the lock has expired, so it can't be released; the sender can refund it.
   */
  45: {message:"LockExpired"},
  /**
   * RefundNotAllowed: a refund is allowed only after expiry, or if the payee is revoked or suspended past the grace period.
   */
  46: {message:"RefundNotAllowed"},
  /**
   * Overflow: an amount calculation overflowed.
   */
  50: {message:"Overflow"}
}








export interface Client {
  /**
   * Construct and simulate a refund transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refund: ({lock_id}: {lock_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a decline transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  decline: ({lock_id}: {lock_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a release transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  release: ({lock_id, idx}: {lock_id: u64, idx: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_lock transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_lock: ({lock_id}: {lock_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Lock>>>

  /**
   * Construct and simulate a set_caps transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_caps: ({max_lock_amount, max_total_locked}: {max_lock_amount: i128, max_total_locked: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_token: ({token}: {token: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a bump_lock transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  bump_lock: ({lock_id}: {lock_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_payee transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_payee: ({payee_id}: {payee_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Payee>>>

  /**
   * Construct and simulate a set_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_status: ({caller, payee_id, status}: {caller: string, payee_id: Buffer, status: PayeeStatus}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a create_lock transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  create_lock: ({sender, token, payee_id, tranches, ref_hash, expires_at}: {sender: string, token: string, payee_id: Buffer, tranches: Array<TrancheInput>, ref_hash: Buffer, expires_at: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a add_attester transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_attester: ({attester}: {attester: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_token: ({token}: {token: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_payout transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_payout: ({attester, payee_id, new_payout}: {attester: string, payee_id: Buffer, new_payout: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a register_payee transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  register_payee: ({attester, payee_id, payout, category, meta_hash}: {attester: string, payee_id: Buffer, payout: string, category: Category, meta_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_attester transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_attester: ({attester}: {attester: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_paused_new_locks transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_paused_new_locks: ({paused}: {paused: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin}: {admin: string},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAAGcmVmdW5kAAAAAAABAAAAAAAAAAdsb2NrX2lkAAAAAAYAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAAHZGVjbGluZQAAAAABAAAAAAAAAAdsb2NrX2lkAAAAAAYAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAAHcmVsZWFzZQAAAAACAAAAAAAAAAdsb2NrX2lkAAAAAAYAAAAAAAAAA2lkeAAAAAAEAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAAIZ2V0X2xvY2sAAAABAAAAAAAAAAdsb2NrX2lkAAAAAAYAAAABAAAD6QAAB9AAAAAETG9jawAAAAM=",
        "AAAAAAAAAAAAAAAIc2V0X2NhcHMAAAACAAAAAAAAAA9tYXhfbG9ja19hbW91bnQAAAAACwAAAAAAAAAQbWF4X3RvdGFsX2xvY2tlZAAAAAsAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAAJYWRkX3Rva2VuAAAAAAAAAQAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAJYnVtcF9sb2NrAAAAAAAAAQAAAAAAAAAHbG9ja19pZAAAAAAGAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAJZ2V0X3BheWVlAAAAAAAAAQAAAAAAAAAIcGF5ZWVfaWQAAAPuAAAAIAAAAAEAAAPpAAAH0AAAAAVQYXllZQAAAAAAAAM=",
        "AAAAAAAAAAAAAAAKc2V0X3N0YXR1cwAAAAAAAwAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAhwYXllZV9pZAAAA+4AAAAgAAAAAAAAAAZzdGF0dXMAAAAAB9AAAAALUGF5ZWVTdGF0dXMAAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAALY3JlYXRlX2xvY2sAAAAABgAAAAAAAAAGc2VuZGVyAAAAAAATAAAAAAAAAAV0b2tlbgAAAAAAABMAAAAAAAAACHBheWVlX2lkAAAD7gAAACAAAAAAAAAACHRyYW5jaGVzAAAD6gAAB9AAAAAMVHJhbmNoZUlucHV0AAAAAAAAAAhyZWZfaGFzaAAAA+4AAAAgAAAAAAAAAApleHBpcmVzX2F0AAAAAAAGAAAAAQAAA+kAAAAGAAAAAw==",
        "AAAAAAAAAAAAAAAMYWRkX2F0dGVzdGVyAAAAAQAAAAAAAAAIYXR0ZXN0ZXIAAAATAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAMcmVtb3ZlX3Rva2VuAAAAAQAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAADpSdW5zIG9uY2UgYXQgZGVwbG95LCBzbyBpbml0aWFsaXphdGlvbiBjYW4ndCBiZSBmcm9udC1ydW4uAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAANdXBkYXRlX3BheW91dAAAAAAAAAMAAAAAAAAACGF0dGVzdGVyAAAAEwAAAAAAAAAIcGF5ZWVfaWQAAAPuAAAAIAAAAAAAAAAKbmV3X3BheW91dAAAAAAAEwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAOcmVnaXN0ZXJfcGF5ZWUAAAAAAAUAAAAAAAAACGF0dGVzdGVyAAAAEwAAAAAAAAAIcGF5ZWVfaWQAAAPuAAAAIAAAAAAAAAAGcGF5b3V0AAAAAAATAAAAAAAAAAhjYXRlZ29yeQAAB9AAAAAIQ2F0ZWdvcnkAAAAAAAAACW1ldGFfaGFzaAAAAAAAA+4AAAAgAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAPcmVtb3ZlX2F0dGVzdGVyAAAAAAEAAAAAAAAACGF0dGVzdGVyAAAAEwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAUc2V0X3BhdXNlZF9uZXdfbG9ja3MAAAABAAAAAAAAAAZwYXVzZWQAAAAAAAEAAAABAAAD6QAAAAIAAAAD",
        "AAAAAQAAAA5BIGZ1bmRlZCBsb2NrLgAAAAAAAAAAAARMb2NrAAAADQAAAAAAAAAKY3JlYXRlZF9hdAAAAAAABgAAAAAAAAAKZXhwaXJlc19hdAAAAAAABgAAAAAAAAACaWQAAAAAAAYAAAAAAAAACHBheWVlX2lkAAAD7gAAACAAAABAU05BUFNIT1Qgb2YgYHBheWVlLnBheW91dGAgYXQgY3JlYXRpb24uIEltbXV0YWJsZSAoaW52YXJpYW50IDcpLgAAAAZwYXlvdXQAAAAAABMAAABIc2hhMjU2KHJlZmVyZW5jZSB8fCBzYWx0KS4gVGhlIHJlZmVyZW5jZSBpdHNlbGYgbmV2ZXIgdG91Y2hlcyB0aGUgY2hhaW4uAAAACHJlZl9oYXNoAAAD7gAAACAAAAAAAAAACHJlbGVhc2VkAAAACwAAABVSZWZ1bmRlZCBvciBkZWNsaW5lZC4AAAAAAAAIcmV0dXJuZWQAAAALAAAAAAAAAAZzZW5kZXIAAAAAABMAAAAAAAAABXN0YXRlAAAAAAAH0AAAAAlMb2NrU3RhdGUAAAAAAAAAAAAABXRva2VuAAAAAAAAEwAAAAAAAAAFdG90YWwAAAAAAAALAAAATTEuLj1NQVhfVFJBTkNIRVMsIG5vbi1kZWNyZWFzaW5nIGB1bmxvY2tfYXRgLCBlYWNoIGB1bmxvY2tfYXQgPD0gZXhwaXJlc19hdGAuAAAAAAAACHRyYW5jaGVzAAAD6gAAB9AAAAAHVHJhbmNoZQA=",
        "AAAAAQAAAD5BIHZlcmlmaWVkIHBheWVlLiBLZXllZCBieSBgcGF5ZWVfaWQgPSBzaGEyNTYocmVnaXN0cnkgc2x1ZylgLgAAAAAAAAAAAAVQYXllZQAAAAAAAAcAAAAWVGhlIHZvdWNoaW5nIGF0dGVzdGVyLgAAAAAACGF0dGVzdGVyAAAAEwAAAAAAAAAIY2F0ZWdvcnkAAAfQAAAACENhdGVnb3J5AAAAJ1NIQS0yNTYgb2YgdGhlIGNhbm9uaWNhbCByZWdpc3RyeSBKU09OLgAAAAAJbWV0YV9oYXNoAAAAAAAD7gAAACAAAAAzRyBvciBDIGFkZHJlc3MuIFVwZGF0aW5nIGl0IGFmZmVjdHMgTkVXIGxvY2tzIG9ubHkuAAAAAAZwYXlvdXQAAAAAABMAAAAAAAAADXJlZ2lzdGVyZWRfYXQAAAAAAAAGAAAAAAAAAAZzdGF0dXMAAAAAB9AAAAALUGF5ZWVTdGF0dXMAAAAAAAAAABFzdGF0dXNfY2hhbmdlZF9hdAAAAAAAAAY=",
        "AAAAAQAAAB5BIHNjaGVkdWxlZCBwb3J0aW9uIG9mIGEgbG9jay4AAAAAAAAAAAAHVHJhbmNoZQAAAAADAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlbGVhc2VkAAAAAQAAAAAAAAAJdW5sb2NrX2F0AAAAAAAABg==",
        "AAAAAgAAAEZQYXllZSBjYXRlZ29yeS4gUHVycG9zZSBvZiBhIGxvY2sgPSBpdHMgcGF5ZWUncyBjYXRlZ29yeS4gQVBQRU5ELU9OTFkuAAAAAAAAAAAACENhdGVnb3J5AAAAAgAAAAAAAAAAAAAABlNjaG9vbAAAAAAAAAAAAAAAAAAEUmVudA==",
        "AAAAAgAAADZBUFBFTkQtT05MWS4gT3BlbiDihpIgQ29tcGxldGVkIHwgUmVmdW5kZWQgfCBEZWNsaW5lZC4AAAAAAAAAAAAJTG9ja1N0YXRlAAAAAAAABAAAAAAAAAAAAAAABE9wZW4AAAAAAAAAAAAAAAlDb21wbGV0ZWQAAAAAAAAAAAAAAAAAAAhSZWZ1bmRlZAAAAAAAAAAAAAAACERlY2xpbmVk",
        "AAAAAgAAADdBUFBFTkQtT05MWS4gQWN0aXZlIOKHhCBTdXNwZW5kZWQ7IFJldm9rZWQgaXMgdGVybWluYWwuAAAAAAAAAAALUGF5ZWVTdGF0dXMAAAAAAwAAAAAAAAAAAAAABkFjdGl2ZQAAAAAAAAAAAAAAAAAJU3VzcGVuZGVkAAAAAAAAAAAAAAAAAAAHUmV2b2tlZAA=",
        "AAAAAgAAAEdXaHkgYSByZWZ1bmQgd2FzIGFsbG93ZWQuIENhcnJpZWQgaW4gdGhlIGBSZWZ1bmRlZGAgZXZlbnQuIEFQUEVORC1PTkxZLgAAAAAAAAAADFJlZnVuZFJlYXNvbgAAAAMAAAAAAAAAAAAAAAdFeHBpcmVkAAAAAAAAAAAAAAAAB1Jldm9rZWQAAAAAAAAAAAAAAAAQU3VzcGVuZGVkVGltZW91dA==",
        "AAAAAQAAAKtDYWxsZXItc3VwcGxpZWQgdHJhbmNoZSBzY2hlZHVsZSBmb3IgYGNyZWF0ZV9sb2NrYCAobm8gYHJlbGVhc2VkYCBmbGFnIHRvIGZvcmdlKS4KU2NhZmZvbGQgYWRkaXRpb246IGBBUkNISVRFQ1RVUkUubWRgIMKnNC4zIG9ubHkgc2F5cyAidHJhbmNoZXMiOyBjb25maXJtIGluIE0xLTA1IHJldmlldy4AAAAAAAAAAAxUcmFuY2hlSW5wdXQAAAACAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACXVubG9ja19hdAAAAAAAAAY=",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAAIAAAAEtOb3RJbml0aWFsaXplZDogdGhlIGNvbnRyYWN0IGhhcyBubyBjb25maWd1cmF0aW9uIChpdCB3YXMgbm90IGluaXRpYWxpemVkKS4AAAAADk5vdEluaXRpYWxpemVkAAAAAAACAAAANk5vdEF0dGVzdGVyOiB0aGUgY2FsbGVyIGlzIG5vdCBvbiB0aGUgYXR0ZXN0ZXIgcm9zdGVyLgAAAAAAC05vdEF0dGVzdGVyAAAAAAMAAAB3Tm90Vm91Y2hpbmdBdHRlc3Rlcjogb25seSB0aGUgcGF5ZWUncyB2b3VjaGluZyBhdHRlc3RlciAoc3RpbGwgb24gdGhlIHJvc3RlciksIG9yIHRoZSBhZG1pbiB3aGVyZSBhbGxvd2VkLCBjYW4gZG8gdGhpcy4AAAAAE05vdFZvdWNoaW5nQXR0ZXN0ZXIAAAAABAAAAGxJbnZhbGlkQ2FwOiBjYXBzIG11c3QgYmUgYXQgbGVhc3QgdGhlIG1pbmltdW0gbG9jayBhbW91bnQsIGFuZCB0aGUgcGVyLWxvY2sgY2FwIGNhbid0IGV4Y2VlZCB0aGUgZ2xvYmFsIGNhcC4AAAAKSW52YWxpZENhcAAAAAAABQAAAD9QYXllZUFscmVhZHlFeGlzdHM6IGEgcGF5ZWUgd2l0aCB0aGlzIElEIGlzIGFscmVhZHkgcmVnaXN0ZXJlZC4AAAAAElBheWVlQWxyZWFkeUV4aXN0cwAAAAAACgAAADNQYXllZU5vdEZvdW5kOiBubyBwYXllZSBpcyByZWdpc3RlcmVkIHdpdGggdGhpcyBJRC4AAAAADVBheWVlTm90Rm91bmQAAAAAAAALAAAAMlBheWVlTm90QWN0aXZlOiB0aGUgcGF5ZWUgaXMgc3VzcGVuZGVkIG9yIHJldm9rZWQuAAAAAAAOUGF5ZWVOb3RBY3RpdmUAAAAAAAwAAABlSW52YWxpZFN0YXR1c1RyYW5zaXRpb246IHRoYXQgc3RhdHVzIGNoYW5nZSBpc24ndCBhbGxvd2VkIChSZXZva2VkIGlzIGZpbmFsOyB0aGUgc3RhdHVzIG11c3QgY2hhbmdlKS4AAAAAAAAXSW52YWxpZFN0YXR1c1RyYW5zaXRpb24AAAAADQAAAEdQYXlvdXRVbmNoYW5nZWQ6IHRoZSBuZXcgcGF5b3V0IGFkZHJlc3MgaXMgdGhlIHNhbWUgYXMgdGhlIGN1cnJlbnQgb25lLgAAAAAPUGF5b3V0VW5jaGFuZ2VkAAAAAA4AAABDUGF5ZWVSZXZva2VkOiB0aGUgcGF5ZWUgaXMgcmV2b2tlZCwgc28gaXRzIHBheW91dCBjYW4ndCBiZSB1cGRhdGVkLgAAAAAMUGF5ZWVSZXZva2VkAAAADwAAAEdJbnZhbGlkUGF5b3V0OiB0aGUgcGF5b3V0IGFkZHJlc3MgY2FuJ3QgYmUgdGhlIEtpbmxvY2sgY29udHJhY3QgaXRzZWxmLgAAAAANSW52YWxpZFBheW91dAAAAAAAABAAAABAUGF1c2VkTmV3TG9ja3M6IG5ldyBsb2NrcyBhcmUgcGF1c2VkOyBleGlzdGluZyBsb2NrcyBzdGlsbCB3b3JrLgAAAA5QYXVzZWROZXdMb2NrcwAAAAAAFAAAADNUb2tlbk5vdEFsbG93ZWQ6IHRoaXMgdG9rZW4gaXNuJ3Qgb24gdGhlIGFsbG93bGlzdC4AAAAAD1Rva2VuTm90QWxsb3dlZAAAAAAVAAAAP0Ftb3VudEJlbG93TWluaW11bTogdGhlIHRvdGFsIGlzIGJlbG93IHRoZSBtaW5pbXVtIGxvY2sgYW1vdW50LgAAAAASQW1vdW50QmVsb3dNaW5pbXVtAAAAAAAWAAAAOEFtb3VudEFib3ZlTG9ja0NhcDogdGhlIHRvdGFsIGlzIGFib3ZlIHRoZSBwZXItbG9jayBjYXAuAAAAEkFtb3VudEFib3ZlTG9ja0NhcAAAAAAAFwAAAE5HbG9iYWxDYXBFeGNlZWRlZDogdGhpcyBsb2NrIHdvdWxkIHRha2UgdGhlIHRvdGFsIGxvY2tlZCBhYm92ZSB0aGUgZ2xvYmFsIGNhcC4AAAAAABFHbG9iYWxDYXBFeGNlZWRlZAAAAAAAABgAAAA8SW52YWxpZFRyYW5jaGVDb3VudDogYSBsb2NrIG5lZWRzIGJldHdlZW4gMSBhbmQgMTIgdHJhbmNoZXMuAAAAE0ludmFsaWRUcmFuY2hlQ291bnQAAAAAGQAAAEVJbnZhbGlkVHJhbmNoZUFtb3VudDogZXZlcnkgdHJhbmNoZSBhbW91bnQgbXVzdCBiZSBncmVhdGVyIHRoYW4gemVyby4AAAAAAAAUSW52YWxpZFRyYW5jaGVBbW91bnQAAAAaAAAAQVVubG9ja0FmdGVyRXhwaXJ5OiBhIHRyYW5jaGUgY2FuJ3QgdW5sb2NrIGFmdGVyIHRoZSBsb2NrIGV4cGlyZXMuAAAAAAAAEVVubG9ja0FmdGVyRXhwaXJ5AAAAAAAAHAAAAD1VbmxvY2tPdXRPZk9yZGVyOiB0cmFuY2hlIHVubG9jayB0aW1lcyBtdXN0IG5vdCBnbyBiYWNrd2FyZHMuAAAAAAAAEFVubG9ja091dE9mT3JkZXIAAAAdAAAAP0V4cGlyeVRvb1Nvb246IHRoZSBsb2NrIG11c3QgZXhwaXJlIGF0IGxlYXN0IG9uZSBob3VyIGZyb20gbm93LgAAAAANRXhwaXJ5VG9vU29vbgAAAAAAAB4AAABPRXhwaXJ5VG9vRmFyOiB0aGUgbG9jayBtdXN0IGV4cGlyZSB3aXRoaW4gdGhlIG1heGltdW0gbG9jayBkdXJhdGlvbiAoMTQ5IGRheXMpLgAAAAAMRXhwaXJ5VG9vRmFyAAAAHwAAAD9TZW5kZXJJc1BheW91dDogdGhlIHNlbmRlciBjYW4ndCBiZSB0aGUgcGF5ZWUncyBwYXlvdXQgYWRkcmVzcy4AAAAADlNlbmRlcklzUGF5b3V0AAAAAAAgAAAAX0xvY2tUdGxUb29Mb25nOiB0aGUgbmV0d29yayBjYW4ndCBrZWVwIHRoaXMgbG9jayBpbiBzdG9yYWdlIHVudGlsIGV4cGlyeSBwbHVzIHRoZSByZWZ1bmQgZ3JhY2UuAAAAAA5Mb2NrVHRsVG9vTG9uZwAAAAAAIQAAACpMb2NrTm90Rm91bmQ6IG5vIGxvY2sgZXhpc3RzIHdpdGggdGhpcyBJRC4AAAAAAAxMb2NrTm90Rm91bmQAAAAoAAAAQkxvY2tOb3RPcGVuOiB0aGUgbG9jayBpcyBhbHJlYWR5IGNvbXBsZXRlZCwgcmVmdW5kZWQsIG9yIGRlY2xpbmVkLgAAAAAAC0xvY2tOb3RPcGVuAAAAACkAAAA/VHJhbmNoZUluZGV4T3V0T2ZSYW5nZTogdGhpcyBsb2NrIGhhcyBubyB0cmFuY2hlIGF0IHRoYXQgaW5kZXguAAAAABZUcmFuY2hlSW5kZXhPdXRPZlJhbmdlAAAAAAAqAAAAP1RyYW5jaGVBbHJlYWR5UmVsZWFzZWQ6IHRoaXMgdHJhbmNoZSBoYXMgYWxyZWFkeSBiZWVuIHJlbGVhc2VkLgAAAAAWVHJhbmNoZUFscmVhZHlSZWxlYXNlZAAAAAAAKwAAADRUcmFuY2hlTm90VW5sb2NrZWQ6IHRoaXMgdHJhbmNoZSBpc24ndCB1bmxvY2tlZCB5ZXQuAAAAElRyYW5jaGVOb3RVbmxvY2tlZAAAAAAALAAAAFVMb2NrRXhwaXJlZDogdGhlIGxvY2sgaGFzIGV4cGlyZWQsIHNvIGl0IGNhbid0IGJlIHJlbGVhc2VkOyB0aGUgc2VuZGVyIGNhbiByZWZ1bmQgaXQuAAAAAAAAC0xvY2tFeHBpcmVkAAAAAC0AAAB3UmVmdW5kTm90QWxsb3dlZDogYSByZWZ1bmQgaXMgYWxsb3dlZCBvbmx5IGFmdGVyIGV4cGlyeSwgb3IgaWYgdGhlIHBheWVlIGlzIHJldm9rZWQgb3Igc3VzcGVuZGVkIHBhc3QgdGhlIGdyYWNlIHBlcmlvZC4AAAAAEFJlZnVuZE5vdEFsbG93ZWQAAAAuAAAAK092ZXJmbG93OiBhbiBhbW91bnQgY2FsY3VsYXRpb24gb3ZlcmZsb3dlZC4AAAAACE92ZXJmbG93AAAAMg==",
        "AAAABQAAAAAAAAAAAAAACERlY2xpbmVkAAAAAQAAAAhkZWNsaW5lZAAAAAMAAAAAAAAAAmlkAAAAAAAGAAAAAQAAAAAAAAAOc2NoZW1hX3ZlcnNpb24AAAAAAAQAAAAAAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAACFJlZnVuZGVkAAAAAQAAAAhyZWZ1bmRlZAAAAAQAAAAAAAAAAmlkAAAAAAAGAAAAAQAAAAAAAAAOc2NoZW1hX3ZlcnNpb24AAAAAAAQAAAAAAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAAAAAAZyZWFzb24AAAAAB9AAAAAMUmVmdW5kUmVhc29uAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAACFJlbGVhc2VkAAAAAQAAAAhyZWxlYXNlZAAAAAUAAAAAAAAAAmlkAAAAAAAGAAAAAQAAAAAAAAAOc2NoZW1hX3ZlcnNpb24AAAAAAAQAAAAAAAAAAAAAAANpZHgAAAAABAAAAAAAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAAAAAABnBheW91dAAAAAAAEwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAC0xvY2tDcmVhdGVkAAAAAAEAAAAMbG9ja19jcmVhdGVkAAAACgAAAAAAAAACaWQAAAAAAAYAAAABAAAAAAAAAA5zY2hlbWFfdmVyc2lvbgAAAAAABAAAAAAAAAAAAAAABnNlbmRlcgAAAAAAEwAAAAAAAAAAAAAACHBheWVlX2lkAAAD7gAAACAAAAAAAAAAAAAAAAZwYXlvdXQAAAAAABMAAAAAAAAAAAAAAAV0b2tlbgAAAAAAABMAAAAAAAAAAAAAAAV0b3RhbAAAAAAAAAsAAAAAAAAAAAAAAAhyZWZfaGFzaAAAA+4AAAAgAAAAAAAAAAAAAAAKZXhwaXJlc19hdAAAAAAABgAAAAAAAAAAAAAADXRyYW5jaGVfY291bnQAAAAAAAAEAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADVBheW91dFVwZGF0ZWQAAAAAAAABAAAADnBheW91dF91cGRhdGVkAAAAAAADAAAAAAAAAAhwYXllZV9pZAAAA+4AAAAgAAAAAQAAAAAAAAAOc2NoZW1hX3ZlcnNpb24AAAAAAAQAAAAAAAAAAAAAAApuZXdfcGF5b3V0AAAAAAATAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAD1BheWVlUmVnaXN0ZXJlZAAAAAABAAAAEHBheWVlX3JlZ2lzdGVyZWQAAAAGAAAAAAAAAAhwYXllZV9pZAAAA+4AAAAgAAAAAQAAAAAAAAAOc2NoZW1hX3ZlcnNpb24AAAAAAAQAAAAAAAAAAAAAAAZwYXlvdXQAAAAAABMAAAAAAAAAAAAAAAhjYXRlZ29yeQAAB9AAAAAIQ2F0ZWdvcnkAAAAAAAAAAAAAAAhhdHRlc3RlcgAAABMAAAAAAAAAAAAAAAltZXRhX2hhc2gAAAAAAAPuAAAAIAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAElBheWVlU3RhdHVzQ2hhbmdlZAAAAAAAAQAAABRwYXllZV9zdGF0dXNfY2hhbmdlZAAAAAQAAAAAAAAACHBheWVlX2lkAAAD7gAAACAAAAABAAAAAAAAAA5zY2hlbWFfdmVyc2lvbgAAAAAABAAAAAAAAAAAAAAABnN0YXR1cwAAAAAH0AAAAAtQYXllZVN0YXR1cwAAAAAAAAAAAAAAAApjaGFuZ2VkX2J5AAAAAAATAAAAAAAAAAI=" ]),
      options
    )
  }
  public readonly fromJSON = {
    refund: this.txFromJSON<Result<void>>,
        decline: this.txFromJSON<Result<void>>,
        release: this.txFromJSON<Result<void>>,
        upgrade: this.txFromJSON<Result<void>>,
        get_lock: this.txFromJSON<Result<Lock>>,
        set_caps: this.txFromJSON<Result<void>>,
        add_token: this.txFromJSON<Result<void>>,
        bump_lock: this.txFromJSON<Result<void>>,
        get_payee: this.txFromJSON<Result<Payee>>,
        set_status: this.txFromJSON<Result<void>>,
        create_lock: this.txFromJSON<Result<u64>>,
        add_attester: this.txFromJSON<Result<void>>,
        remove_token: this.txFromJSON<Result<void>>,
        update_payout: this.txFromJSON<Result<void>>,
        register_payee: this.txFromJSON<Result<void>>,
        remove_attester: this.txFromJSON<Result<void>>,
        set_paused_new_locks: this.txFromJSON<Result<void>>
  }
}