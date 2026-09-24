use soroban_sdk::{Address, Env};
use crate::types::{DataKey, Plan, Subscription};

const DAY_IN_LEDGERS: u32 = 17280; // Assuming ~5 seconds per ledger
const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 15 * DAY_IN_LEDGERS;

pub fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn set_admin(env: &Env, admin: &Address) {
    let key = DataKey::Admin;
    env.storage().persistent().set(&key, admin);
    extend_persistent(env, &key);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    let key = DataKey::Admin;
    if let Some(admin) = env.storage().persistent().get(&key) {
        extend_persistent(env, &key);
        Some(admin)
    } else {
        None
    }
}

pub fn set_plan_counter(env: &Env, count: u64) {
    let key = DataKey::PlanCounter;
    env.storage().persistent().set(&key, &count);
    extend_persistent(env, &key);
}

pub fn get_plan_counter(env: &Env) -> u64 {
    let key = DataKey::PlanCounter;
    if let Some(count) = env.storage().persistent().get(&key) {
        extend_persistent(env, &key);
        count
    } else {
        1
    }
}

pub fn set_plan(env: &Env, id: u64, plan: &Plan) {
    let key = DataKey::Plan(id);
    env.storage().persistent().set(&key, plan);
    extend_persistent(env, &key);
}

pub fn get_plan(env: &Env, id: u64) -> Option<Plan> {
    let key = DataKey::Plan(id);
    if let Some(plan) = env.storage().persistent().get(&key) {
        extend_persistent(env, &key);
        Some(plan)
    } else {
        None
    }
}

pub fn set_subscription(env: &Env, subscriber: Address, plan_id: u64, sub: &Subscription) {
    let key = DataKey::Subscription(subscriber, plan_id);
    env.storage().persistent().set(&key, sub);
    extend_persistent(env, &key);
}

pub fn get_subscription(env: &Env, subscriber: Address, plan_id: u64) -> Option<Subscription> {
    let key = DataKey::Subscription(subscriber, plan_id);
    if let Some(sub) = env.storage().persistent().get(&key) {
        extend_persistent(env, &key);
        Some(sub)
    } else {
        None
    }
}
