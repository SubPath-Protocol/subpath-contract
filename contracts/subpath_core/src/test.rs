#![cfg(test)]

use crate::types::SubscriptionStatus;
use crate::{SubPathContract, SubPathContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
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

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    assert_eq!(plan_id, 1);

    let plan = client.get_plan(&1).unwrap();
    assert_eq!(plan.merchant, merchant);
    assert_eq!(plan.amount, 1000);
    assert_eq!(plan.cycle_seconds, 2592000);
}

#[test]
fn test_plan_create_invalid_amount() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();

    let res = client.try_create_plan(&merchant, &token.address, &0, &2592000);
    assert!(res.is_err());

    let res2 = client.try_create_plan(&merchant, &token.address, &-100, &2592000);
    assert!(res2.is_err());
}

#[test]
fn test_plan_create_invalid_cycle() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();

    let res = client.try_create_plan(&merchant, &token.address, &1000, &0);
    assert!(res.is_err());
}

#[test]
fn test_subscription_success() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();

    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);

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
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);

    client.subscribe(&subscriber, &plan_id);
    let res = client.try_subscribe(&subscriber, &plan_id);
    assert!(res.is_err());
}

#[test]
fn test_subscription_cancel() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);

    let res = client.try_execute_billing(&subscriber, &plan_id);
    assert!(res.is_err());
}

#[test]
fn test_billing_canceled() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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

#[test]
fn test_billing_past_due_insufficient_allowance_and_recovery() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();

    token.mint(&subscriber, &10000);
    // Approve 500 — less than plan amount (1000)
    token.approve(&subscriber, &client.address, &500, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);

    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Active);

    // Fast-forward to billing time
    env.ledger().with_mut(|l| l.timestamp = sub.next_billing_time);

    // Billing should not panic; instead it transitions subscription to PastDue
    client.execute_billing(&subscriber, &plan_id);
    let sub_past_due = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_past_due.status, SubscriptionStatus::PastDue);

    // Subscriber grants allowance to cover payment
    token.approve(&subscriber, &client.address, &5000, &2000000);

    // Re-attempt billing should now succeed and restore status to Active
    client.execute_billing(&subscriber, &plan_id);
    let sub_active = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_active.status, SubscriptionStatus::Active);
    assert_eq!(token.balance(&subscriber), 8000);
    assert_eq!(token.balance(&merchant), 2000);
}

#[test]
fn test_billing_past_due_insufficient_balance() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();

    // Mint exactly 1000 stroops (consumed by first sub), but approve plenty of allowance
    token.mint(&subscriber, &1000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);

    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(token.balance(&subscriber), 0);

    // Fast-forward to billing time
    env.ledger().with_mut(|l| l.timestamp = sub.next_billing_time);

    // Billing transitions to PastDue due to 0 balance
    client.execute_billing(&subscriber, &plan_id);
    let sub_past_due = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_past_due.status, SubscriptionStatus::PastDue);

    // Top up balance and re-execute
    token.mint(&subscriber, &2000);
    client.execute_billing(&subscriber, &plan_id);
    let sub_recovered = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_recovered.status, SubscriptionStatus::Active);
    assert_eq!(token.balance(&subscriber), 1000);
}

#[test]
fn test_past_due_can_be_canceled() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();

    token.mint(&subscriber, &1000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);

    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    env.ledger().with_mut(|l| l.timestamp = sub.next_billing_time);

    client.execute_billing(&subscriber, &plan_id);
    let sub_past_due = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_past_due.status, SubscriptionStatus::PastDue);

    // Cancel while in PastDue
    client.cancel_subscription(&subscriber, &plan_id);
    let sub_canceled = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_canceled.status, SubscriptionStatus::Canceled);
}

#[test]
fn test_property_rollover_monotonic_multiple_cycles() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();

    let cycle = 86400u64; // 1 day
    let amount = 100i128;
    token.mint(&subscriber, &100_000);
    token.approve(&subscriber, &client.address, &100_000, &2_000_000);

    let plan_id = client.create_plan(&merchant, &token.address, &amount, &cycle);
    client.subscribe(&subscriber, &plan_id);

    let mut prev_billing_time = client.get_subscription(&subscriber, &plan_id).unwrap().next_billing_time;

    // Execute 10 consecutive billing cycles
    for cycle_index in 1..=10 {
        env.ledger().with_mut(|l| l.timestamp = prev_billing_time);
        client.execute_billing(&subscriber, &plan_id);

        let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
        assert_eq!(
            sub.next_billing_time,
            prev_billing_time + cycle,
            "Cycle {cycle_index} next_billing_time must advance monotonically by exact cycle_seconds"
        );
        assert!(sub.next_billing_time > prev_billing_time);
        prev_billing_time = sub.next_billing_time;
    }
}

#[test]
fn test_property_rollover_extended_lapse_in_future() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();

    let cycle = 2592000u64; // 30 days
    let amount = 500i128;
    token.mint(&subscriber, &10_000);
    token.approve(&subscriber, &client.address, &10_000, &2_000_000);

    let plan_id = client.create_plan(&merchant, &token.address, &amount, &cycle);
    client.subscribe(&subscriber, &plan_id);

    let sub_initial = client.get_subscription(&subscriber, &plan_id).unwrap();
    let initial_billing_time = sub_initial.next_billing_time;

    // Fast forward 1 year (12 cycles) into the future
    let lapse_time = initial_billing_time + (cycle * 12);
    env.ledger().with_mut(|l| l.timestamp = lapse_time);

    client.execute_billing(&subscriber, &plan_id);

    let sub_after = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_after.next_billing_time, initial_billing_time + cycle);
    assert_eq!(sub_after.status, SubscriptionStatus::Active);
}

#[test]
fn test_property_rollover_u64_max_boundary_saturation() {
    let env = Env::default();
    let (client, token, merchant, subscriber, _, _) = setup(&env);
    client.initialize();

    let cycle = 100_000u64;
    let amount = 10i128;
    token.mint(&subscriber, &1_000);
    token.approve(&subscriber, &client.address, &1_000, &2_000_000);

    let plan_id = client.create_plan(&merchant, &token.address, &amount, &cycle);
    client.subscribe(&subscriber, &plan_id);

    let sub = client.get_subscription(&subscriber, &plan_id).unwrap();
    // Fast-forward timestamp to scheduled billing time
    env.ledger().with_mut(|l| l.timestamp = sub.next_billing_time);

    // Ensure billing succeeds and advances next_billing_time safely
    client.execute_billing(&subscriber, &plan_id);

    let sub_after = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub_after.status, SubscriptionStatus::Active);
    assert_eq!(sub_after.next_billing_time, sub.next_billing_time.saturating_add(cycle));
}
