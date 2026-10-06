#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use crate::errors::Error;
use crate::types::{Plan, Subscription, SubscriptionStatus};
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct SubPathContract;

#[contractimpl]
impl SubPathContract {
    pub fn initialize(env: Env) -> Result<(), Error> {
        if storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_initialized(&env);
        storage::set_plan_counter(&env, 1);
        Ok(())
    }

    pub fn create_plan(
        env: Env,
        merchant: Address,
        token: Address,
        amount: i128,
        cycle_seconds: u64,
    ) -> Result<u64, Error> {
        merchant.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if cycle_seconds == 0 {
            return Err(Error::InvalidCycle);
        }

        let plan_id = storage::get_plan_counter(&env);
        let plan = Plan {
            merchant: merchant.clone(),
            token,
            amount,
            cycle_seconds,
        };

        storage::set_plan(&env, plan_id, &plan);
        storage::set_plan_counter(&env, plan_id + 1);

        events::plan_add(&env, merchant, plan_id);
        Ok(plan_id)
    }

    pub fn subscribe(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error> {
        subscriber.require_auth();

        let plan = storage::get_plan(&env, plan_id).ok_or(Error::PlanNotFound)?;

        let next_billing_time = env.ledger().timestamp().saturating_add(plan.cycle_seconds);

        if let Some(existing_sub) = storage::get_subscription(&env, subscriber.clone(), plan_id) {
            if existing_sub.status == SubscriptionStatus::Active {
                return Err(Error::AlreadySubscribed);
            }
        }

        let sub = Subscription {
            subscriber: subscriber.clone(),
            plan_id,
            next_billing_time,
            status: SubscriptionStatus::Active,
        };

        storage::set_subscription(&env, subscriber.clone(), plan_id, &sub);

        // Execute first payment immediately
        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        token_client.transfer(&subscriber, &plan.merchant, &plan.amount);

        events::sub_new(&env, subscriber, plan_id);
        Ok(())
    }

    pub fn cancel_subscription(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error> {
        subscriber.require_auth();

        let mut sub = storage::get_subscription(&env, subscriber.clone(), plan_id)
            .ok_or(Error::SubscriptionNotFound)?;

        sub.status = SubscriptionStatus::Canceled;
        storage::set_subscription(&env, subscriber.clone(), plan_id, &sub);

        events::sub_end(&env, subscriber, plan_id);
        Ok(())
    }

    pub fn pause_subscription(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error> {
        subscriber.require_auth();

        let mut sub = storage::get_subscription(&env, subscriber.clone(), plan_id)
            .ok_or(Error::SubscriptionNotFound)?;

        if sub.status == SubscriptionStatus::Canceled {
            return Err(Error::SubscriptionCanceled);
        }

        sub.status = SubscriptionStatus::Paused;
        storage::set_subscription(&env, subscriber.clone(), plan_id, &sub);

        // Emit an event here if we had one, but we'll reuse sub_end or assume off-chain tracks it
        Ok(())
    }

    pub fn resume_subscription(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error> {
        subscriber.require_auth();

        let mut sub = storage::get_subscription(&env, subscriber.clone(), plan_id)
            .ok_or(Error::SubscriptionNotFound)?;

        if sub.status == SubscriptionStatus::Canceled {
            return Err(Error::SubscriptionCanceled);
        }

        sub.status = SubscriptionStatus::Active;
        storage::set_subscription(&env, subscriber.clone(), plan_id, &sub);

        Ok(())
    }

    pub fn execute_billing(
        env: Env,
        caller: Address,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<(), Error> {
        caller.require_auth();

        let mut sub = storage::get_subscription(&env, subscriber.clone(), plan_id)
            .ok_or(Error::SubscriptionNotFound)?;

        if sub.status == SubscriptionStatus::Canceled {
            return Err(Error::SubscriptionCanceled);
        }
        if sub.status == SubscriptionStatus::Paused {
            return Err(Error::SubscriptionPaused);
        }

        if env.ledger().timestamp() < sub.next_billing_time {
            return Err(Error::BillingTooEarly);
        }

        let plan = storage::get_plan(&env, plan_id).ok_or(Error::PlanNotFound)?;

        // Pull funds via allowance
        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &subscriber,
            &plan.merchant,
            &plan.amount,
        );

        sub.next_billing_time = sub.next_billing_time.saturating_add(plan.cycle_seconds);
        storage::set_subscription(&env, subscriber.clone(), plan_id, &sub);

        events::sub_billed(&env, subscriber, plan_id);
        Ok(())
    }

    pub fn get_plan(env: Env, plan_id: u64) -> Option<Plan> {
        storage::get_plan(&env, plan_id)
    }

    pub fn get_subscription(env: Env, subscriber: Address, plan_id: u64) -> Option<Subscription> {
        storage::get_subscription(&env, subscriber, plan_id)
    }

    pub fn next_plan_id(env: Env) -> u64 {
        storage::get_plan_counter(&env)
    }
}
