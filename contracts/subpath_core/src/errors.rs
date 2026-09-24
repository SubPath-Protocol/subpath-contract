use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    PlanNotFound = 3,
    SubscriptionNotFound = 4,
    SubscriptionCanceled = 5,
    BillingTooEarly = 6,
}
