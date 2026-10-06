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
  2: {message:"NotInitialized"},
  3: {message:"NotAttester"},
  4: {message:"NotVouchingAttester"},
  5: {message:"InvalidCap"},
  10: {message:"PayeeAlreadyExists"},
  11: {message:"PayeeNotFound"},
  12: {message:"PayeeNotActive"},
  13: {message:"InvalidStatusTransition"},
  14: {message:"PayoutUnchanged"},
  15: {message:"PayeeRevoked"},
  16: {message:"InvalidPayout"},
  20: {message:"PausedNewLocks"},
  21: {message:"TokenNotAllowed"},
  22: {message:"AmountBelowMinimum"},
  23: {message:"AmountAboveLockCap"},
  24: {message:"GlobalCapExceeded"},
  25: {message:"InvalidTrancheCount"},
  26: {message:"InvalidTrancheAmount"},
  28: {message:"UnlockAfterExpiry"},
  29: {message:"UnlockOutOfOrder"},
  30: {message:"ExpiryTooSoon"},
  31: {message:"ExpiryTooFar"},
  32: {message:"SenderIsPayout"},
  33: {message:"LockTtlTooLong"},
  40: {message:"LockNotFound"},
  41: {message:"LockNotOpen"},
  42: {message:"TrancheIndexOutOfRange"},
  43: {message:"TrancheAlreadyReleased"},
  44: {message:"TrancheNotUnlocked"},
  45: {message:"LockExpired"},
  46: {message:"RefundNotAllowed"},
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
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAAIAAAAAAAAAAOTm90SW5pdGlhbGl6ZWQAAAAAAAIAAAAAAAAAC05vdEF0dGVzdGVyAAAAAAMAAAAAAAAAE05vdFZvdWNoaW5nQXR0ZXN0ZXIAAAAABAAAAAAAAAAKSW52YWxpZENhcAAAAAAABQAAAAAAAAASUGF5ZWVBbHJlYWR5RXhpc3RzAAAAAAAKAAAAAAAAAA1QYXllZU5vdEZvdW5kAAAAAAAACwAAAAAAAAAOUGF5ZWVOb3RBY3RpdmUAAAAAAAwAAAAAAAAAF0ludmFsaWRTdGF0dXNUcmFuc2l0aW9uAAAAAA0AAAAAAAAAD1BheW91dFVuY2hhbmdlZAAAAAAOAAAAAAAAAAxQYXllZVJldm9rZWQAAAAPAAAAAAAAAA1JbnZhbGlkUGF5b3V0AAAAAAAAEAAAAAAAAAAOUGF1c2VkTmV3TG9ja3MAAAAAABQAAAAAAAAAD1Rva2VuTm90QWxsb3dlZAAAAAAVAAAAAAAAABJBbW91bnRCZWxvd01pbmltdW0AAAAAABYAAAAAAAAAEkFtb3VudEFib3ZlTG9ja0NhcAAAAAAAFwAAAAAAAAARR2xvYmFsQ2FwRXhjZWVkZWQAAAAAAAAYAAAAAAAAABNJbnZhbGlkVHJhbmNoZUNvdW50AAAAABkAAAAAAAAAFEludmFsaWRUcmFuY2hlQW1vdW50AAAAGgAAAAAAAAARVW5sb2NrQWZ0ZXJFeHBpcnkAAAAAAAAcAAAAAAAAABBVbmxvY2tPdXRPZk9yZGVyAAAAHQAAAAAAAAANRXhwaXJ5VG9vU29vbgAAAAAAAB4AAAAAAAAADEV4cGlyeVRvb0ZhcgAAAB8AAAAAAAAADlNlbmRlcklzUGF5b3V0AAAAAAAgAAAAAAAAAA5Mb2NrVHRsVG9vTG9uZwAAAAAAIQAAAAAAAAAMTG9ja05vdEZvdW5kAAAAKAAAAAAAAAALTG9ja05vdE9wZW4AAAAAKQAAAAAAAAAWVHJhbmNoZUluZGV4T3V0T2ZSYW5nZQAAAAAAKgAAAAAAAAAWVHJhbmNoZUFscmVhZHlSZWxlYXNlZAAAAAAAKwAAAAAAAAASVHJhbmNoZU5vdFVubG9ja2VkAAAAAAAsAAAAAAAAAAtMb2NrRXhwaXJlZAAAAAAtAAAAAAAAABBSZWZ1bmROb3RBbGxvd2VkAAAALgAAAAAAAAAIT3ZlcmZsb3cAAAAy",
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