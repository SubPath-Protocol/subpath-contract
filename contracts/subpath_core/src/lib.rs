#![no_std]

mod errors;
mod events;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, Env};
use crate::errors::Error;
use crate::types::{Plan, Subscription};

#[contract]
pub struct SubPathContract;

#[contractimpl]
impl SubPathContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::get_admin(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
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
        
        let sub = Subscription {
            subscriber: subscriber.clone(),
            plan_id,
            next_billing_time,
            status: 1,
        };

        storage::set_subscription(&env, subscriber.clone(), plan_id, &sub);

        // Execute first payment immediately
        let token_client = soroban_sdk::token::Client::new(&env, &plan.token);
        token_client.transfer(&subscriber, &plan.merchant, &plan.amount);

        events::sub_new(&env, subscriber, plan_id);
        Ok(())
    }

    pub fn cancel_subscription(
        env: Env,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<(), Error> {
        subscriber.require_auth();

        let mut sub = storage::get_subscription(&env, subscriber.clone(), plan_id)
            .ok_or(Error::SubscriptionNotFound)?;
        
        sub.status = 0;
        storage::set_subscription(&env, subscriber.clone(), plan_id, &sub);

        events::sub_end(&env, subscriber, plan_id);
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
        
        if sub.status != 1 {
            return Err(Error::SubscriptionCanceled);
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
}
