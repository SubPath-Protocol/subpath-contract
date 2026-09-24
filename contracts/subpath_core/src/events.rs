use soroban_sdk::{symbol_short, Address, Env};

pub fn plan_add(env: &Env, merchant: Address, plan_id: u64) {
    env.events().publish((symbol_short!("plan_add"), merchant), plan_id);
}

pub fn sub_new(env: &Env, subscriber: Address, plan_id: u64) {
    env.events().publish((symbol_short!("sub_new"), subscriber), plan_id);
}

pub fn sub_end(env: &Env, subscriber: Address, plan_id: u64) {
    env.events().publish((symbol_short!("sub_end"), subscriber), plan_id);
}

pub fn sub_billed(env: &Env, subscriber: Address, plan_id: u64) {
    env.events().publish((symbol_short!("sub_billed"), subscriber), plan_id);
}
