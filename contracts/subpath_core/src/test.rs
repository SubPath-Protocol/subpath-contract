#![cfg(test)]

use crate::types::SubscriptionStatus;
use crate::{SubPathContract, SubPathContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String,
};

fn setup<'a>(
    env: &'a Env,
) -> (
    SubPathContractClient<'a>,
    token::StellarAssetClient<'a>,
    Address,
    Address,
    Address,
    Address,
) {
    env.mock_all_auths();

    let contract_id = env.register(SubPathContract, ());
    let client = SubPathContractClient::new(env, &contract_id);

    let merchant = Address::generate(env);
    let subscriber = Address::generate(env);
    let executor = Address::generate(env);
    let admin = Address::generate(env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token = token::StellarAssetClient::new(env, &token_contract.address());

    (client, token, merchant, subscriber, executor, admin)
}

fn default_meta(env: &Env) -> String {
    String::from_str(
        env,
        "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    )
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let (client, _, _, _, _, _) = setup(&env);

    // First init
    client.initialize();

    // Repeat init should fail
    let res = client.try_initialize();
    assert!(res.is_err());
}

#[test]
fn test_plan_create_success() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    assert_eq!(plan_id, 1);

    let plan = client.get_plan(&1).unwrap();
    assert_eq!(plan.merchant, merchant);
    assert_eq!(plan.amount, 1000);
    assert_eq!(plan.cycle_seconds, 2592000);
    assert_eq!(plan.metadata, meta);
}

#[test]
fn test_plan_create_invalid_amount() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);

    let res = client.try_create_plan(&merchant, &token.address, &0, &2592000, &meta);
    assert!(res.is_err());

    let res2 = client.try_create_plan(&merchant, &token.address, &-100, &2592000, &meta);
    assert!(res2.is_err());
}

#[test]
fn test_plan_create_invalid_cycle() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);

    let res = client.try_create_plan(&merchant, &token.address, &1000, &0, &meta);
    assert!(res.is_err());
}

#[test]
fn test_subscription_success() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);

    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);

    let initial_timestamp = env.ledger().timestamp();

    client.subscribe(&subscriber, &plan_id);

    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub.subscriber, subscriber);
    assert_eq!(sub.plan_id, plan_id);
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert_eq!(sub.next_billing_time, initial_timestamp + 2592000);

    assert_eq!(token.balance(&subscriber), 9000);
    assert_eq!(token.balance(&merchant), 1000);
}

#[test]
fn test_subscription_duplicate() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);

    client.subscribe(&subscriber, &plan_id);
    let res = client.try_subscribe(&subscriber, &plan_id);
    assert!(res.is_err());
}

#[test]
fn test_subscription_cancel() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);

    client.cancel_subscription(&subscriber, &plan_id);
    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Canceled);
}

#[test]
fn test_billing_success() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);

    let sub1 = client.get_subscription(&subscriber, &plan_id).unwrap();

    env.ledger()
        .with_mut(|l| l.timestamp = sub1.next_billing_time);

    client.execute_billing(&subscriber, &plan_id);

    assert_eq!(token.balance(&subscriber), 8000);
    assert_eq!(token.balance(&merchant), 2000);

    let sub2 = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub2.next_billing_time, sub1.next_billing_time + 2592000);
}

#[test]
fn test_billing_too_early() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);

    let res = client.try_execute_billing(&subscriber, &plan_id);
    assert!(res.is_err());
}

#[test]
fn test_billing_canceled() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);
    client.cancel_subscription(&subscriber, &plan_id);

    let sub1 = client.get_subscription(&subscriber, &plan_id).unwrap();
    env.ledger()
        .with_mut(|l| l.timestamp = sub1.next_billing_time);

    let res = client.try_execute_billing(&subscriber, &plan_id);
    assert!(res.is_err());
}

#[test]
fn test_billing_late() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);

    let sub1 = client.get_subscription(&subscriber, &plan_id).unwrap();

    env.ledger()
        .with_mut(|l| l.timestamp = sub1.next_billing_time + 3000000);

    client.execute_billing(&subscriber, &plan_id);

    assert_eq!(token.balance(&subscriber), 8000);

    let sub2 = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub2.next_billing_time, sub1.next_billing_time + 2592000);
}

#[test]
fn test_pause_and_resume() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);

    client.pause_subscription(&subscriber, &plan_id);
    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Paused);

    let sub_time = sub.next_billing_time;
    env.ledger().with_mut(|l| l.timestamp = sub_time);

    let res = client.try_execute_billing(&subscriber, &plan_id);
    assert!(res.is_err());

    client.resume_subscription(&subscriber, &plan_id);
    let sub_resumed = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_resumed.status, SubscriptionStatus::Active);

    client.execute_billing(&subscriber, &plan_id);
    assert_eq!(token.balance(&subscriber), 8000);
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut z = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.state = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn range_u64(&mut self, min: u64, max_inclusive: u64) -> u64 {
        let span = max_inclusive - min + 1;
        min + (self.next_u64() % span)
    }
}

#[test]
fn test_property_rollover_monotonic_and_integer_multiples() {
    let mut rng = DeterministicRng::new(0x50B9_A740_C0DE_0003);

    for _case in 0..32 {
        let env = Env::default();
        let (client, token, merchant, subscriber, _, _) = setup(&env);
        client.initialize();
        let meta = default_meta(&env);

        let initial_ts = rng.range_u64(0, 10_000_000_000);
        let cycle_seconds = rng.range_u64(1, 31_536_000);
        let steps = rng.range_u64(1, 10);
        let amount: i128 = i128::from(rng.range_u64(1, 10_000));

        let total_needed = amount * i128::from(steps + 2);
        token.mint(&subscriber, &total_needed);
        token.approve(&subscriber, &client.address, &total_needed, &2_000_000);

        env.ledger().with_mut(|l| l.timestamp = initial_ts);

        let plan_id = client.create_plan(&merchant, &token.address, &amount, &cycle_seconds, &meta);
        client.subscribe(&subscriber, &plan_id);

        let sub_init = client.get_subscription(&subscriber, &plan_id).unwrap();
        let anchor_next_billing = initial_ts + cycle_seconds;
        assert_eq!(sub_init.next_billing_time, anchor_next_billing);

        let mut prev_next_billing = sub_init.next_billing_time;

        for step in 1..=steps {
            // 1 second before scheduled billing time must always be rejected
            let early_ts = prev_next_billing - 1;
            env.ledger().with_mut(|l| l.timestamp = early_ts);
            assert!(client.try_execute_billing(&subscriber, &plan_id).is_err());

            // Execute billing with arbitrary late jitter (from 0 up to 3x cycle_seconds)
            let jitter = rng.range_u64(0, cycle_seconds.saturating_mul(3));
            let exec_ts = prev_next_billing.saturating_add(jitter);
            env.ledger().with_mut(|l| l.timestamp = exec_ts);

            client.execute_billing(&subscriber, &plan_id);

            let sub_now = client.get_subscription(&subscriber, &plan_id).unwrap();

            // Invariant 1: Strict monotonicity
            assert!(sub_now.next_billing_time > prev_next_billing);

            // Invariant 2: Exact step increment of cycle_seconds
            assert_eq!(sub_now.next_billing_time - prev_next_billing, cycle_seconds);

            // Invariant 3: Cumulative delta from initial anchor is an exact integer multiple
            let cumulative_delta = sub_now.next_billing_time - anchor_next_billing;
            assert_eq!(cumulative_delta, step * cycle_seconds);
            assert_eq!((sub_now.next_billing_time - initial_ts) % cycle_seconds, 0);

            prev_next_billing = sub_now.next_billing_time;
        }

        // Conservation of funds across initial subscription + `steps` rollovers
        let expected_paid = amount * i128::from(steps + 1);
        assert_eq!(token.balance(&merchant), expected_paid);
        assert_eq!(token.balance(&subscriber), total_needed - expected_paid);
    }
}

#[test]
fn test_property_multi_cycle_lapse_catchup_and_window_rollover() {
    let mut rng = DeterministicRng::new(0xCA7C_4090_0000_0003);

    for _case in 0..16 {
        let env = Env::default();
        let (client, token, merchant, subscriber, _, _) = setup(&env);
        client.initialize();
        let meta = default_meta(&env);

        let initial_ts = rng.range_u64(1_000, 1_000_000_000);
        let cycle_seconds = rng.range_u64(60, 86_400);
        let elapsed_cycles = rng.range_u64(2, 12);
        let partial_offset = rng.range_u64(0, cycle_seconds - 1);
        let amount: i128 = 500;

        let total_mint = amount * i128::from(elapsed_cycles + 5);
        token.mint(&subscriber, &total_mint);
        token.approve(&subscriber, &client.address, &total_mint, &2_000_000);

        env.ledger().with_mut(|l| l.timestamp = initial_ts);

        let plan_id = client.create_plan(&merchant, &token.address, &amount, &cycle_seconds, &meta);
        client.subscribe(&subscriber, &plan_id);

        let sub0 = client.get_subscription(&subscriber, &plan_id).unwrap();
        let scheduled_first = sub0.next_billing_time;

        // Jump `now` many cycles past the scheduled `next_billing_time`
        let late_now = scheduled_first + elapsed_cycles * cycle_seconds + partial_offset;
        env.ledger().with_mut(|l| l.timestamp = late_now);

        // Exactly `elapsed_cycles + 1` catch-up executions are due at `late_now`
        for catchup_idx in 1..=(elapsed_cycles + 1) {
            client.execute_billing(&subscriber, &plan_id);
            let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
            assert_eq!(
                sub.next_billing_time,
                scheduled_first + catchup_idx * cycle_seconds
            );
        }

        // After catching up to the current window, `next_billing_time > late_now`
        let sub_caught_up = client.get_subscription(&subscriber, &plan_id).unwrap();
        assert!(sub_caught_up.next_billing_time > late_now);
        assert!(
            sub_caught_up.next_billing_time - late_now <= cycle_seconds,
            "next_billing_time must land within one cycle after catching up"
        );

        // Any further billing attempt at the same `late_now` must fail with BillingTooEarly
        assert!(client.try_execute_billing(&subscriber, &plan_id).is_err());
    }
}

#[test]
fn test_edge_minimum_cycle_one_second() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);

    token.mint(&subscriber, &10_000);
    token.approve(&subscriber, &client.address, &10_000, &2_000_000);

    let start_ts = 1_700_000_000u64;
    env.ledger().with_mut(|l| l.timestamp = start_ts);

    let plan_id = client.create_plan(&merchant, &token.address, &100, &1, &meta);
    client.subscribe(&subscriber, &plan_id);

    for step in 1..=5u64 {
        let due_ts = start_ts + step;
        env.ledger().with_mut(|l| l.timestamp = due_ts - 1);
        assert!(client.try_execute_billing(&subscriber, &plan_id).is_err());

        env.ledger().with_mut(|l| l.timestamp = due_ts);
        client.execute_billing(&subscriber, &plan_id);

        let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
        assert_eq!(sub.next_billing_time, start_ts + step + 1);
    }
}

#[test]
fn test_edge_u64_max_cycle_and_timestamp_saturation_safety() {
    // Case 1: cycle_seconds == u64::MAX starting at timestamp 0
    {
        let env = Env::default();
        let (client, token, merchant, subscriber, _, _) = setup(&env);
        client.initialize();
        let meta = default_meta(&env);

        token.mint(&subscriber, &10_000);
        token.approve(&subscriber, &client.address, &10_000, &2_000_000);

        env.ledger().with_mut(|l| l.timestamp = 0);
        let plan_id = client.create_plan(&merchant, &token.address, &1_000, &u64::MAX, &meta);
        client.subscribe(&subscriber, &plan_id);

        let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
        assert_eq!(sub.next_billing_time, u64::MAX);

        env.ledger().with_mut(|l| l.timestamp = u64::MAX - 1);
        assert!(client.try_execute_billing(&subscriber, &plan_id).is_err());

        env.ledger().with_mut(|l| l.timestamp = u64::MAX);
        client.execute_billing(&subscriber, &plan_id);

        let sub_after = client.get_subscription(&subscriber, &plan_id).unwrap();
        assert_eq!(sub_after.next_billing_time, u64::MAX);
    }

    // Case 2: timestamp near u64::MAX with standard cycle_seconds saturates without overflow
    let boundary_cases: &[(u64, u64)] = &[
        (u64::MAX - 2_592_000, 2_592_000),
        (u64::MAX - 2_592_000 + 1, 2_592_000),
        (u64::MAX - 1, 1),
        (u64::MAX - 1, u64::MAX),
        (u64::MAX, 1),
        (u64::MAX, u64::MAX),
    ];

    for &(initial_ts, cycle_seconds) in boundary_cases {
        let env = Env::default();
        let (client, token, merchant, subscriber, _, _) = setup(&env);
        client.initialize();
        let meta = default_meta(&env);

        token.mint(&subscriber, &10_000);
        token.approve(&subscriber, &client.address, &10_000, &2_000_000);

        env.ledger().with_mut(|l| l.timestamp = initial_ts);
        let plan_id = client.create_plan(&merchant, &token.address, &1_000, &cycle_seconds, &meta);
        client.subscribe(&subscriber, &plan_id);

        let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
        let expected_first = initial_ts.saturating_add(cycle_seconds);
        assert_eq!(sub.next_billing_time, expected_first);

        env.ledger().with_mut(|l| l.timestamp = u64::MAX);
        client.execute_billing(&subscriber, &plan_id);

        let sub_after = client.get_subscription(&subscriber, &plan_id).unwrap();
        assert_eq!(
            sub_after.next_billing_time,
            expected_first.saturating_add(cycle_seconds)
        );
        assert_eq!(sub_after.next_billing_time, u64::MAX);
    }
}

#[test]
fn test_plan_create_before_initialization_fails() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    let meta = default_meta(&env);
    // Intentionally do NOT call client.initialize()

    let res = client.try_create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    assert_eq!(res, Err(Ok(crate::errors::Error::NotInitialized)));
}

#[test]
fn test_plan_counter_monotonic_and_non_overwriting() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();

    let meta1 = String::from_str(&env, "Starter Tier - Basic");
    let meta2 = String::from_str(&env, "Pro Tier - Standard");
    let meta3 = String::from_str(&env, "Enterprise Tier - Unlimited");

    let plan1 = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta1);
    assert_eq!(plan1, 1);

    let plan2 = client.create_plan(&merchant, &token.address, &2000, &86400, &meta2);
    assert_eq!(plan2, 2);

    let plan3 = client.create_plan(&merchant, &token.address, &5000, &604800, &meta3);
    assert_eq!(plan3, 3);

    // Verify each plan remains distinct and intact
    let p1 = client.get_plan(&1).unwrap();
    assert_eq!(p1.amount, 1000);
    assert_eq!(p1.metadata, meta1);
    let p2 = client.get_plan(&2).unwrap();
    assert_eq!(p2.amount, 2000);
    assert_eq!(p2.metadata, meta2);
    let p3 = client.get_plan(&3).unwrap();
    assert_eq!(p3.amount, 5000);
    assert_eq!(p3.metadata, meta3);

    // Re-initialization attempt must fail and preserve counter
    let re_init = client.try_initialize();
    assert!(re_init.is_err());
    assert_eq!(client.next_plan_id(), 4);
}

#[test]
fn test_subscribe_while_paused_rejected() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);

    client.pause_subscription(&subscriber, &plan_id);
    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Paused);

    // Attempting to subscribe again while paused must fail with SubscriptionPaused
    let res = client.try_subscribe(&subscriber, &plan_id);
    assert_eq!(res, Err(Ok(crate::errors::Error::SubscriptionPaused)));
}

#[test]
fn test_resubscribe_after_cancel_succeeds() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);
    assert_eq!(token.balance(&subscriber), 9000);

    // Cancel active subscription
    client.cancel_subscription(&subscriber, &plan_id);
    let sub_canceled = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_canceled.status, SubscriptionStatus::Canceled);

    // Fast-forward time
    env.ledger().with_mut(|l| l.timestamp += 5000000);

    // Re-subscribing creates fresh subscription and charges new cycle
    client.subscribe(&subscriber, &plan_id);
    let sub_fresh = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_fresh.status, SubscriptionStatus::Active);
    assert_eq!(token.balance(&subscriber), 8000);
    assert_eq!(token.balance(&merchant), 2000);
}

#[test]
fn test_duplicate_billing_in_same_cycle_rejected_by_contract() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    let meta = default_meta(&env);
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000, &meta);
    client.subscribe(&subscriber, &plan_id);

    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    env.ledger()
        .with_mut(|l| l.timestamp = sub.next_billing_time);

    // First billing execution succeeds
    client.execute_billing(&subscriber, &plan_id);
    assert_eq!(token.balance(&subscriber), 8000);
    assert_eq!(token.balance(&merchant), 2000);

    // Second immediate billing attempt within same cycle MUST fail with BillingTooEarly
    let duplicate_res = client.try_execute_billing(&subscriber, &plan_id);
    assert_eq!(
        duplicate_res,
        Err(Ok(crate::errors::Error::BillingTooEarly))
    );
    // Balance remains unchanged (no double billing)
    assert_eq!(token.balance(&subscriber), 8000);
    assert_eq!(token.balance(&merchant), 2000);
}

#[test]
fn test_plan_metadata_persistence_and_boundary_limits() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();

    // 1. Empty string boundary
    let empty_meta = String::from_str(&env, "");
    let id_empty = client.create_plan(&merchant, &token.address, &100, &3600, &empty_meta);
    let p_empty = client.get_plan(&id_empty).unwrap();
    assert_eq!(p_empty.metadata, empty_meta);
    assert_eq!(p_empty.metadata.len(), 0);

    // 2. IPFS CIDv1 URI reference
    let cid_meta = String::from_str(
        &env,
        "ipfs://bafkreigh2akiscaildcqabsyg3dfr6chu3fgpregiymsck7e7aqa4s52zy",
    );
    let id_cid = client.create_plan(&merchant, &token.address, &500, &86400, &cid_meta);
    let p_cid = client.get_plan(&id_cid).unwrap();
    assert_eq!(p_cid.metadata, cid_meta);

    // 3. Large 1024-byte metadata payload boundary
    let long_bytes = [b'M'; 1024];
    let long_str = core::str::from_utf8(&long_bytes).unwrap();
    let long_meta = String::from_str(&env, long_str);
    let id_long = client.create_plan(&merchant, &token.address, &2500, &604800, &long_meta);
    let p_long = client.get_plan(&id_long).unwrap();
    assert_eq!(p_long.metadata, long_meta);
    assert_eq!(p_long.metadata.len(), 1024);
}
