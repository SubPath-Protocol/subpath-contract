use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Initialized,
    Plan(u64),
    Subscription(Address, u64),
    PlanCounter,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Canceled,
    Paused,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub merchant: Address,
    pub token: Address,
    pub amount: i128,
    pub cycle_seconds: u64,
    pub metadata: soroban_sdk::String,
}

#[contracttype]
pub struct Subscription {
    pub subscriber: Address,
    pub plan_id: u64,
    pub next_billing_time: u64,
    pub status: SubscriptionStatus,
}
