use crate::types::{DataKey, Plan, Subscription};
use soroban_sdk::{Address, Env};

const DAY_IN_LEDGERS: u32 = 17280; // Assuming ~5 seconds per ledger
const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 15 * DAY_IN_LEDGERS;

pub fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(
        key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn set_initialized(env: &Env) {
    let key = DataKey::Initialized;
    env.storage().persistent().set(&key, &true);
    extend_persistent(env, &key);
}

pub fn is_initialized(env: &Env) -> bool {
    let key = DataKey::Initialized;
    if env.storage().persistent().has(&key) {
        extend_persistent(env, &key);
        true
    } else {
        false
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
