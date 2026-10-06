//! Property tests (proptest) for the invariants in `docs/ARCHITECTURE.md` §4.6. Row: M1-14.
//!
//! Each case runs a random sequence of operations (create, release, refund, decline, payee
//! status changes, payout updates, time jumps, pause, allowlist and roster changes, and
//! freezing accounts) against one contract, and checks after every step:
//!
//! 1. `released + returned <= total`; terminal locks are fully settled.
//! 2. Tranches sum to `total`; every `unlock_at <= expires_at`.
//! 3. Funds go only to `lock.payout` or `lock.sender`: every tracked balance matches what
//!    the locks say it should be, and no other address ever receives anything.
//! 4. A tranche is released at most once (`released` == sum of released tranches).
//! 5. `release` succeeds exactly when the spec allows it.
//! 6. `refund` succeeds exactly when the spec allows it.
//! 7. `lock.payout` never changes after creation.
//! 8. Admin, attester, pause, and allowlist changes never block release, decline, or
//!    refund (checked by 5, 6, and `decline` succeeding exactly when the spec allows it).
//! 9. `total_locked` equals the sum of remainders of Open locks.
//! 10. A failed call changes nothing (locks, payees, config, and balances).
//!
//! Cases default to 64; set `PROPTEST_CASES` for longer runs.

use kinlock::constants::{MIN_AMOUNT, SUSPENSION_REFUND_GRACE_SECS};
use kinlock::storage;
use kinlock::types::{Category, Config, Lock, LockState, Payee, PayeeStatus, TrancheInput};
use kinlock::{Kinlock, KinlockClient};
use proptest::prelude::*;
use soroban_sdk::testutils::{Address as _, EnvTestConfig, IssuerFlags, Ledger as _};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, BytesN, Env, Vec as SVec};
use std::collections::BTreeMap;

const T0: u64 = 1_000_000;
const HOUR: u64 = 60 * 60;
const NETWORK_MAX_ENTRY_TTL: u32 = 3_110_400;
const SENDERS: usize = 3;
const PAYEES: usize = 4;
const FUNDING: i128 = 1_000 * MIN_AMOUNT;

// ----- Operations -----

#[derive(Clone, Debug)]
enum Op {
    Create {
        sender: usize,
        payee: usize,
        /// (amount in whole units, unlock offset in hours)
        tranches: Vec<(u8, u16)>,
        expiry_hours: u16,
    },
    Release {
        lock: usize,
        idx: u32,
    },
    Refund {
        lock: usize,
    },
    Decline {
        lock: usize,
    },
    SetStatus {
        payee: usize,
        status: u8,
    },
    UpdatePayout {
        payee: usize,
    },
    Advance {
        hours: u16,
    },
    /// Move the clock forward to a boundary of an existing lock:
    /// 0 = next tranche's unlock_at, 1 = expires_at - 1, 2 = expires_at,
    /// 3 = end of the payee's suspension grace - 1, 4 = end of that grace.
    JumpTo {
        lock: usize,
        point: u8,
    },
    /// Jump exactly to a boundary of a lock (or one second before it), then immediately try
    /// `action` (0 = release, 1 = refund, 2 = decline) on that lock.
    /// point: 0/1 = unlock_at - 1 / unlock_at of the next tranche, 2/3 = expires_at - 1 /
    /// expires_at, 4/5 = suspension grace end - 1 / grace end.
    /// `status` first sets the lock's payee (via the admin): 0 = leave, 1 = Suspended,
    /// 2 = Revoked, 3 = Active. Invalid transitions just fail and change nothing.
    /// `admin` then applies an admin action first: 0 = none, 1 = pause new locks,
    /// 2 = remove the token from the allowlist, 3 = remove the payee's attester.
    /// None of these may block release, refund, or decline (invariant 8).
    AtBoundary {
        lock: usize,
        status: u8,
        admin: u8,
        point: u8,
        action: u8,
    },
    /// Walk a lock's unreleased tranches in order, jumping to each `unlock_at` and releasing
    /// it, so locks actually reach Completed.
    ReleaseAll {
        lock: usize,
    },
    /// A valid schedule whose expiry is placed at the last unlock time + `delta_secs`
    /// (-1, 0, or +1), to hit the `unlock_at <= expires_at` rule exactly.
    CreateEdge {
        sender: usize,
        payee: usize,
        tranches: Vec<(u8, u16)>,
        delta_secs: i8,
    },
    Pause(bool),
    SetToken(bool),
    SetAttester {
        payee: usize,
        on: bool,
    },
    Freeze {
        party: Party,
        frozen: bool,
    },
}

#[derive(Clone, Copy, Debug)]
enum Party {
    Sender(usize),
    /// Index into every payout address ever used.
    Payout(usize),
}

fn op_strategy() -> impl Strategy<Value = Op> {
    // Mostly valid schedules (expiry after the last unlock), plus some unconstrained ones so
    // the validation failures are exercised too.
    let valid_create = (
        0..SENDERS,
        0..PAYEES,
        prop::collection::vec((1u8..=3, 0u16..=300), 1..=4),
        1u16..=400,
    )
        .prop_map(|(sender, payee, mut tranches, extra_hours)| {
            tranches.sort_by_key(|t| t.1);
            let last = tranches.last().map_or(0, |t| t.1);
            Op::Create {
                sender,
                payee,
                tranches,
                expiry_hours: last + extra_hours,
            }
        });
    let any_create = (
        0..SENDERS,
        0..PAYEES,
        prop::collection::vec((0u8..=3, 0u16..=600), 0..=13),
        0u16..=4_000,
    )
        .prop_map(|(sender, payee, tranches, expiry_hours)| Op::Create {
            sender,
            payee,
            tranches,
            expiry_hours,
        });
    prop_oneof![
        6 => valid_create,
        1 => any_create,
        8 => (0usize..8, 0u32..5).prop_map(|(lock, idx)| Op::Release { lock, idx }),
        3 => (0usize..8).prop_map(|lock| Op::Refund { lock }),
        2 => (0usize..8).prop_map(|lock| Op::Decline { lock }),
        // status: 0..9 Active, 9..17 Suspended, 17..20 Revoked (Revoked is permanent, so rare).
        2 => (0..PAYEES, 0u8..20).prop_map(|(payee, status)| Op::SetStatus { payee, status }),
        1 => (0..PAYEES).prop_map(|payee| Op::UpdatePayout { payee }),
        4 => (1u16..=72).prop_map(|hours| Op::Advance { hours }),
        1 => (1u16..=800).prop_map(|hours| Op::Advance { hours }),
        5 => (0usize..8, 0u8..5).prop_map(|(lock, point)| Op::JumpTo { lock, point }),
        8 => (0usize..8, 0u8..4, 0u8..4, 0u8..6, 0u8..3).prop_map(
            |(lock, status, admin, point, action)| Op::AtBoundary {
                lock,
                status,
                admin,
                point,
                action,
            }
        ),
        2 => (0usize..8).prop_map(|lock| Op::ReleaseAll { lock }),
        1 => (
            0..SENDERS,
            0..PAYEES,
            prop::collection::vec((1u8..=3, 2u16..=300), 1..=3),
            -1i8..=1,
        )
            .prop_map(|(sender, payee, mut tranches, delta_secs)| {
                tranches.sort_by_key(|t| t.1);
                Op::CreateEdge { sender, payee, tranches, delta_secs }
            }),
        1 => any::<bool>().prop_map(Op::Pause),
        1 => any::<bool>().prop_map(Op::SetToken),
        1 => (0..PAYEES, any::<bool>()).prop_map(|(payee, on)| Op::SetAttester { payee, on }),
        1 => (prop_oneof![
            (0..SENDERS).prop_map(Party::Sender),
            (0usize..4).prop_map(Party::Payout)
        ], any::<bool>())
            .prop_map(|(party, frozen)| Op::Freeze { party, frozen }),
    ]
}

// ----- World -----

struct World {
    env: Env,
    client: KinlockClient<'static>,
    admin: Address,
    token: Address,
    senders: Vec<Address>,
    payee_ids: Vec<BytesN<32>>,
    attesters: Vec<Address>,
    /// Every payout address ever registered, in order.
    payouts: Vec<Address>,
    lock_ids: Vec<u64>,
    /// Payout snapshot recorded when each lock was created (invariant 7).
    created_payout: BTreeMap<u64, Address>,
    /// Addresses whose token authorization is currently revoked.
    frozen: Vec<Address>,
}

/// Everything a failed call must leave untouched (invariant 10).
#[derive(Debug, PartialEq)]
struct Snapshot {
    config: Config,
    locks: Vec<Lock>,
    payees: Vec<Payee>,
    balances: Vec<i128>,
}

impl World {
    fn new() -> Self {
        let env = Env::new_with_config(EnvTestConfig {
            capture_snapshot_at_drop: false,
        });
        env.ledger().set_timestamp(T0);
        env.ledger().set_max_entry_ttl(NETWORK_MAX_ENTRY_TTL);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(Kinlock, (admin.clone(),));
        let client = KinlockClient::new(&env, &contract_id);

        let sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
        sac.issuer().set_flag(IssuerFlags::RevocableFlag);
        let token = sac.address();
        client.add_token(&token);

        let senders: Vec<Address> = (0..SENDERS).map(|_| Address::generate(&env)).collect();
        for s in &senders {
            StellarAssetClient::new(&env, &token).mint(s, &FUNDING);
        }

        let mut payee_ids = Vec::new();
        let mut attesters = Vec::new();
        let mut payouts = Vec::new();
        for i in 0..PAYEES {
            let attester = Address::generate(&env);
            client.add_attester(&attester);
            let payee_id = BytesN::from_array(&env, &[i as u8 + 1; 32]);
            let payout = Address::generate(&env);
            let meta_hash = BytesN::from_array(&env, &[0x55; 32]);
            client.register_payee(&attester, &payee_id, &payout, &Category::School, &meta_hash);
            payee_ids.push(payee_id);
            attesters.push(attester);
            payouts.push(payout);
        }

        World {
            env,
            client,
            admin,
            token,
            senders,
            payee_ids,
            attesters,
            payouts,
            lock_ids: Vec::new(),
            created_payout: BTreeMap::new(),
            frozen: Vec::new(),
        }
    }

    fn now(&self) -> u64 {
        self.env.ledger().timestamp()
    }

    fn config(&self) -> Config {
        self.env
            .as_contract(&self.client.address, || storage::read_config(&self.env))
            .unwrap()
    }

    fn balance(&self, who: &Address) -> i128 {
        TokenClient::new(&self.env, &self.token).balance(who)
    }

    /// Every address that could hold the token: senders, payouts, attesters, admin, contract.
    fn tracked(&self) -> Vec<Address> {
        let mut v = self.senders.clone();
        v.extend(self.payouts.iter().cloned());
        v.extend(self.attesters.iter().cloned());
        v.push(self.admin.clone());
        v.push(self.client.address.clone());
        v
    }

    fn locks(&self) -> Vec<Lock> {
        self.lock_ids
            .iter()
            .map(|id| self.client.get_lock(id))
            .collect()
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            config: self.config(),
            locks: self.locks(),
            payees: self
                .payee_ids
                .iter()
                .map(|id| self.client.get_payee(id))
                .collect(),
            balances: self.tracked().iter().map(|a| self.balance(a)).collect(),
        }
    }

    /// i in 0..5: an Open lock (if any); 5..7: any lock, including closed ones;
    /// 7: a lock that doesn't exist.
    fn pick_lock(&self, i: usize) -> u64 {
        if self.lock_ids.is_empty() || i == 7 {
            return 9_999;
        }
        if i < 5 {
            let open: Vec<u64> = self
                .lock_ids
                .iter()
                .copied()
                .filter(|id| self.open_lock(*id).is_some())
                .collect();
            if !open.is_empty() {
                return open[i % open.len()];
            }
        }
        self.lock_ids[i % self.lock_ids.len()]
    }

    /// A tranche index within the lock most of the time; `i == 4` is out of range.
    fn pick_tranche(&self, id: u64, i: u32) -> u32 {
        if i == 4 {
            return 99;
        }
        match self.client.try_get_lock(&id) {
            Ok(Ok(lock)) => i % lock.tranches.len(),
            _ => i,
        }
    }

    fn party(&self, p: Party) -> Address {
        match p {
            Party::Sender(i) => self.senders[i].clone(),
            Party::Payout(i) => self.payouts[i % self.payouts.len()].clone(),
        }
    }

    // ----- Spec predicates (what SHOULD happen, from current chain state) -----

    fn open_lock(&self, id: u64) -> Option<Lock> {
        self.client
            .try_get_lock(&id)
            .ok()
            .and_then(|r| r.ok())
            .filter(|l| l.state == LockState::Open)
    }

    fn release_allowed(&self, id: u64, idx: u32) -> bool {
        let Some(lock) = self.open_lock(id) else {
            return false;
        };
        let Some(t) = lock.tranches.get(idx) else {
            return false;
        };
        let payee = self.client.get_payee(&lock.payee_id);
        let now = self.now();
        payee.status == PayeeStatus::Active
            && !t.released
            && t.unlock_at <= now
            && now < lock.expires_at
            && !self.frozen.contains(&lock.payout)
    }

    fn refund_allowed(&self, id: u64) -> bool {
        let Some(lock) = self.open_lock(id) else {
            return false;
        };
        let payee = self.client.get_payee(&lock.payee_id);
        let now = self.now();
        let condition = now >= lock.expires_at
            || payee.status == PayeeStatus::Revoked
            || (payee.status == PayeeStatus::Suspended
                && now >= payee.status_changed_at + SUSPENSION_REFUND_GRACE_SECS);
        condition && !self.frozen.contains(&lock.sender)
    }

    fn decline_allowed(&self, id: u64) -> bool {
        self.open_lock(id)
            .is_some_and(|lock| !self.frozen.contains(&lock.sender))
    }

    // ----- Actions shared by the random and boundary operations -----

    fn do_release(&self, id: u64, idx: u32) -> Result<bool, TestCaseError> {
        let expected = self.release_allowed(id, idx);
        let before = self.snapshot();
        let ok = self.client.try_release(&id, &idx).is_ok();
        // 10: a refused call changes nothing.
        if !ok {
            prop_assert_eq!(&self.snapshot(), &before, "refused release changed state");
        }
        prop_assert_eq!(ok, expected, "release({}, {}) at {}", id, idx, self.now());
        Ok(ok)
    }

    fn do_refund(&self, id: u64) -> Result<bool, TestCaseError> {
        let expected = self.refund_allowed(id);
        let before = self.snapshot();
        let ok = self.client.try_refund(&id).is_ok();
        // 10: a refused call changes nothing.
        if !ok {
            prop_assert_eq!(&self.snapshot(), &before, "refused refund changed state");
        }
        prop_assert_eq!(ok, expected, "refund({}) at {}", id, self.now());
        Ok(ok)
    }

    fn do_decline(&self, id: u64) -> Result<bool, TestCaseError> {
        let expected = self.decline_allowed(id);
        let before = self.snapshot();
        let ok = self.client.try_decline(&id).is_ok();
        // 10: a refused call changes nothing.
        if !ok {
            prop_assert_eq!(&self.snapshot(), &before, "refused decline changed state");
        }
        prop_assert_eq!(ok, expected, "decline({}) at {}", id, self.now());
        Ok(ok)
    }

    fn do_create(
        &mut self,
        sender: usize,
        payee: usize,
        tranches: SVec<TrancheInput>,
        expires_at: u64,
    ) -> Result<bool, TestCaseError> {
        let r = self.client.try_create_lock(
            &self.senders[sender],
            &self.token,
            &self.payee_ids[payee],
            &tranches,
            &BytesN::from_array(&self.env, &[0xAB; 32]),
            &expires_at,
        );
        let Ok(Ok(id)) = r else { return Ok(false) };
        let lock = self.client.get_lock(&id);
        let payee_now = self.client.get_payee(&self.payee_ids[payee]);
        let config = self.config();
        // Creation only happens when the gating conditions hold.
        prop_assert!(!config.paused_new_locks);
        prop_assert_eq!(payee_now.status, PayeeStatus::Active);
        prop_assert_eq!(&lock.payout, &payee_now.payout);
        prop_assert!(lock.sender != lock.payout);
        prop_assert!(lock.expires_at >= self.now() + kinlock::constants::MIN_EXPIRY_AHEAD_SECS);
        self.lock_ids.push(id);
        self.created_payout.insert(id, lock.payout);
        Ok(true)
    }

    // ----- Apply one operation; returns whether the contract call succeeded -----

    fn apply(&mut self, op: &Op) -> Result<bool, TestCaseError> {
        let ok = match op {
            Op::Create {
                sender,
                payee,
                tranches,
                expiry_hours,
            } => {
                let now = self.now();
                let mut v = SVec::new(&self.env);
                for (units, hours) in tranches {
                    v.push_back(TrancheInput {
                        amount: i128::from(*units) * MIN_AMOUNT,
                        unlock_at: now + u64::from(*hours) * HOUR,
                    });
                }
                self.do_create(*sender, *payee, v, now + u64::from(*expiry_hours) * HOUR)?
            }
            Op::CreateEdge {
                sender,
                payee,
                tranches,
                delta_secs,
            } => {
                let now = self.now();
                let mut v = SVec::new(&self.env);
                let mut last = now;
                for (units, hours) in tranches {
                    last = now + u64::from(*hours) * HOUR;
                    v.push_back(TrancheInput {
                        amount: i128::from(*units) * MIN_AMOUNT,
                        unlock_at: last,
                    });
                }
                let expires_at = last.saturating_add_signed(i64::from(*delta_secs));
                let ok = self.do_create(*sender, *payee, v, expires_at)?;
                // The only rule a -1 s expiry breaks is `unlock_at <= expires_at`.
                if *delta_secs < 0 {
                    prop_assert!(
                        !ok,
                        "lock with a tranche unlocking after expiry was created"
                    );
                }
                ok
            }
            Op::Release { lock, idx } => {
                let id = self.pick_lock(*lock);
                let idx = self.pick_tranche(id, *idx);
                self.do_release(id, idx)?
            }
            Op::Refund { lock } => self.do_refund(self.pick_lock(*lock))?,
            Op::Decline { lock } => self.do_decline(self.pick_lock(*lock))?,
            Op::AtBoundary {
                lock,
                status,
                admin,
                point,
                action,
            } => {
                let id = self.pick_lock(*lock);
                let Ok(Ok(l)) = self.client.try_get_lock(&id) else {
                    return Ok(true);
                };
                let new_status = match status {
                    1 => Some(PayeeStatus::Suspended),
                    2 => Some(PayeeStatus::Revoked),
                    3 => Some(PayeeStatus::Active),
                    _ => None,
                };
                if let Some(st) = new_status {
                    let _ = self.client.try_set_status(&self.admin, &l.payee_id, &st);
                }
                match admin {
                    1 => {
                        let _ = self.client.try_set_paused_new_locks(&true);
                    }
                    2 => {
                        let _ = self.client.try_remove_token(&l.token);
                    }
                    3 => {
                        if let Some(i) = self.payee_ids.iter().position(|p| *p == l.payee_id) {
                            let _ = self.client.try_remove_attester(&self.attesters[i]);
                        }
                    }
                    _ => {}
                }
                // Prefer a tranche that hasn't unlocked yet, so "unlock_at - 1" is reachable;
                // otherwise the next unreleased one.
                let now = self.now();
                let future = l
                    .tranches
                    .iter()
                    .enumerate()
                    .find(|(_, t)| !t.released && t.unlock_at > now);
                let next =
                    future.or_else(|| l.tranches.iter().enumerate().find(|(_, t)| !t.released));
                let (idx, unlock_at) = next
                    .map(|(i, t)| (i as u32, t.unlock_at))
                    .unwrap_or((0, l.expires_at));
                let payee = self.client.get_payee(&l.payee_id);
                let grace_end = payee.status_changed_at + SUSPENSION_REFUND_GRACE_SECS;
                let target = match point {
                    0 => unlock_at - 1,
                    1 => unlock_at,
                    2 => l.expires_at - 1,
                    3 => l.expires_at,
                    4 => grace_end - 1,
                    _ => grace_end,
                };
                if target > self.now() {
                    self.env.ledger().set_timestamp(target);
                }
                match action {
                    0 => self.do_release(id, idx)?,
                    1 => self.do_refund(id)?,
                    _ => self.do_decline(id)?,
                };
                // Composite op: the action's own call was checked for invariant 10 above.
                true
            }
            Op::ReleaseAll { lock } => {
                let id = self.pick_lock(*lock);
                let Ok(Ok(l)) = self.client.try_get_lock(&id) else {
                    return Ok(true);
                };
                for (i, t) in l.tranches.iter().enumerate() {
                    if t.released {
                        continue;
                    }
                    if t.unlock_at > self.now() {
                        self.env.ledger().set_timestamp(t.unlock_at);
                    }
                    self.do_release(id, i as u32)?;
                }
                // Composite op: each release was checked for invariant 10 in `do_release`.
                true
            }
            Op::SetStatus { payee, status } => {
                let status = match status {
                    0..9 => PayeeStatus::Active,
                    9..17 => PayeeStatus::Suspended,
                    _ => PayeeStatus::Revoked,
                };
                self.client
                    .try_set_status(&self.admin, &self.payee_ids[*payee], &status)
                    .is_ok()
            }
            Op::UpdatePayout { payee } => {
                let new_payout = Address::generate(&self.env);
                let ok = self
                    .client
                    .try_update_payout(
                        &self.attesters[*payee],
                        &self.payee_ids[*payee],
                        &new_payout,
                    )
                    .is_ok();
                if ok {
                    self.payouts.push(new_payout);
                }
                ok
            }
            Op::Advance { hours } => {
                let t = self.now() + u64::from(*hours) * HOUR;
                self.env.ledger().set_timestamp(t);
                true
            }
            Op::JumpTo { lock, point } => {
                let id = self.pick_lock(*lock);
                let Ok(Ok(l)) = self.client.try_get_lock(&id) else {
                    return Ok(true);
                };
                let payee = self.client.get_payee(&l.payee_id);
                let grace_end = payee.status_changed_at + SUSPENSION_REFUND_GRACE_SECS;
                let target = match point {
                    0 => l
                        .tranches
                        .iter()
                        .find(|t| !t.released)
                        .map_or(l.expires_at, |t| t.unlock_at),
                    1 => l.expires_at - 1,
                    2 => l.expires_at,
                    3 => grace_end - 1,
                    _ => grace_end,
                };
                if target > self.now() {
                    self.env.ledger().set_timestamp(target);
                }
                true
            }
            Op::Pause(p) => self.client.try_set_paused_new_locks(p).is_ok(),
            Op::SetToken(on) => {
                if *on {
                    self.client.try_add_token(&self.token).is_ok()
                } else {
                    self.client.try_remove_token(&self.token).is_ok()
                }
            }
            Op::SetAttester { payee, on } => {
                let a = &self.attesters[*payee];
                if *on {
                    self.client.try_add_attester(a).is_ok()
                } else {
                    self.client.try_remove_attester(a).is_ok()
                }
            }
            Op::Freeze { party, frozen } => {
                let who = self.party(*party);
                let ok = StellarAssetClient::new(&self.env, &self.token)
                    .try_set_authorized(&who, &!frozen)
                    .is_ok();
                if ok {
                    if *frozen {
                        if !self.frozen.contains(&who) {
                            self.frozen.push(who);
                        }
                    } else {
                        self.frozen.retain(|a| *a != who);
                    }
                }
                ok
            }
        };
        Ok(ok)
    }

    // ----- Invariants checked after every step -----

    fn check_invariants(&self) -> Result<(), TestCaseError> {
        let locks = self.locks();
        let config = self.config();
        let mut open_remainders: i128 = 0;
        // (address, balance it must hold). A Vec, not a map: `Address` isn't a valid map key
        // under clippy (it carries an Env handle).
        let mut expected: Vec<(Address, i128)> = Vec::new();
        let mut add = |who: &Address, delta: i128| match expected.iter_mut().find(|(a, _)| a == who)
        {
            Some((_, b)) => *b += delta,
            None => expected.push((who.clone(), delta)),
        };
        for s in &self.senders {
            add(s, FUNDING);
        }

        for lock in &locks {
            // 1
            let settled = lock.released + lock.returned;
            prop_assert!(lock.released >= 0 && lock.returned >= 0);
            prop_assert!(settled <= lock.total, "lock {} over-settled", lock.id);
            match lock.state {
                LockState::Open => {
                    prop_assert!(settled < lock.total);
                    prop_assert_eq!(lock.returned, 0);
                    open_remainders += lock.total - settled;
                }
                LockState::Completed => {
                    prop_assert_eq!(lock.released, lock.total);
                    prop_assert_eq!(lock.returned, 0);
                }
                LockState::Refunded | LockState::Declined => {
                    prop_assert_eq!(settled, lock.total);
                }
            }
            // 2 and 4
            let mut sum = 0i128;
            let mut released_sum = 0i128;
            for t in lock.tranches.iter() {
                prop_assert!(t.unlock_at <= lock.expires_at);
                sum += t.amount;
                if t.released {
                    released_sum += t.amount;
                }
            }
            prop_assert_eq!(sum, lock.total);
            prop_assert_eq!(released_sum, lock.released);
            // 7
            prop_assert_eq!(Some(&lock.payout), self.created_payout.get(&lock.id));
            // 3: where each lock's money must be
            add(&lock.sender, lock.returned - lock.total);
            add(&lock.payout, lock.released);
        }

        // 9
        prop_assert_eq!(config.total_locked, open_remainders);
        // 3: the contract holds exactly the open remainders...
        prop_assert_eq!(self.balance(&self.client.address), open_remainders);
        // ...and every other tracked address holds exactly what the locks say, nothing more.
        for who in self.tracked() {
            if who == self.client.address {
                continue;
            }
            let want = expected
                .iter()
                .find(|(a, _)| *a == who)
                .map_or(0, |(_, b)| *b);
            prop_assert_eq!(self.balance(&who), want);
        }
        Ok(())
    }
}

fn config() -> ProptestConfig {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64);
    ProptestConfig {
        cases,
        max_shrink_iters: 2_000,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn invariants_hold_for_any_sequence(ops in prop::collection::vec(op_strategy(), 1..60)) {
        let mut w = World::new();
        // Two open locks from the start: one short (10 days) and one that outlives the
        // 14-day suspension grace (40 days), so the grace rule is reachable.
        for (sender, payee, expiry_hours) in [(0, 0, 240), (1, 1, 960)] {
            let seeded = w.apply(&Op::Create {
                sender,
                payee,
                tranches: vec![(1, 0), (2, 24), (1, 48)],
                expiry_hours,
            })?;
            prop_assert!(seeded);
        }
        w.check_invariants()?;
        for op in &ops {
            let before = w.snapshot();
            let ok = w.apply(op)?;
            // 10
            if !ok {
                prop_assert_eq!(&w.snapshot(), &before, "failed {:?} changed state", op);
            }
            w.check_invariants()?;
        }
    }
}
