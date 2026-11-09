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
fn test_plan_create_before_initialization_fails() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    // Intentionally do NOT call client.initialize()

    let res = client.try_create_plan(&merchant, &token.address, &1000, &2592000);
    assert_eq!(res, Err(Ok(crate::errors::Error::NotInitialized)));
}

#[test]
fn test_plan_counter_monotonic_and_non_overwriting() {
    let env = Env::default();
    let (client, token, merchant, _, _, _) = setup(&env);
    client.initialize();

    let plan1 = client.create_plan(&merchant, &token.address, &1000, &2592000);
    assert_eq!(plan1, 1);

    let plan2 = client.create_plan(&merchant, &token.address, &2000, &86400);
    assert_eq!(plan2, 2);

    let plan3 = client.create_plan(&merchant, &token.address, &5000, &604800);
    assert_eq!(plan3, 3);

    // Verify each plan remains distinct and intact
    let p1 = client.get_plan(&1).unwrap();
    assert_eq!(p1.amount, 1000);
    let p2 = client.get_plan(&2).unwrap();
    assert_eq!(p2.amount, 2000);
    let p3 = client.get_plan(&3).unwrap();
    assert_eq!(p3.amount, 5000);

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
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
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
