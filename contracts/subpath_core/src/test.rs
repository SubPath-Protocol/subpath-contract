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
    let (client, token, merchant, subscriber, executor, _) = setup(&env);
    client.initialize();
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);

    let sub1 = client.get_subscription(&subscriber, &plan_id).unwrap();

    env.ledger()
        .with_mut(|l| l.timestamp = sub1.next_billing_time);

    client.execute_billing(&executor, &subscriber, &plan_id);

    assert_eq!(token.balance(&subscriber), 8000);
    assert_eq!(token.balance(&merchant), 2000);

    let sub2 = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub2.next_billing_time, sub1.next_billing_time + 2592000);
}

#[test]
fn test_billing_too_early() {
    let env = Env::default();
    let (client, token, merchant, subscriber, executor, _) = setup(&env);
    client.initialize();
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);

    let res = client.try_execute_billing(&executor, &subscriber, &plan_id);
    assert!(res.is_err());
}

#[test]
fn test_billing_canceled() {
    let env = Env::default();
    let (client, token, merchant, subscriber, executor, _) = setup(&env);
    client.initialize();
    token.mint(&subscriber, &10000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);
    client.cancel_subscription(&subscriber, &plan_id);

    let sub1 = client.get_subscription(&subscriber, &plan_id).unwrap();
    env.ledger()
        .with_mut(|l| l.timestamp = sub1.next_billing_time);

    let res = client.try_execute_billing(&executor, &subscriber, &plan_id);
    assert!(res.is_err());
}

#[test]
fn test_billing_late() {
    let env = Env::default();
    let (client, token, merchant, subscriber, executor, _) = setup(&env);
    client.initialize();
    token.mint(&subscriber, &10000);
    token.approve(&subscriber, &client.address, &10000, &2000000);

    let plan_id = client.create_plan(&merchant, &token.address, &1000, &2592000);
    client.subscribe(&subscriber, &plan_id);

    let sub1 = client.get_subscription(&subscriber, &plan_id).unwrap();

    env.ledger()
        .with_mut(|l| l.timestamp = sub1.next_billing_time + 3000000);

    client.execute_billing(&executor, &subscriber, &plan_id);

    assert_eq!(token.balance(&subscriber), 8000);

    let sub2 = client.get_subscription(&subscriber, &plan_id).unwrap();
    assert_eq!(sub2.next_billing_time, sub1.next_billing_time + 2592000);
}
