# Issue 1: Pause and Resume Subscription Semantics

## Summary
Currently, users can only completely cancel a subscription. We need to introduce the ability for a user to pause their subscription indefinitely without losing their plan state, and resume it later.

## Why it matters
Subscribers often want to take a break from a service without entirely deleting their subscription history or going through the full setup process again. Allowing a temporary pause increases user retention and provides a better UX.

## Scoped Implementation
1. Add a `Paused` state to `SubscriptionStatus` in `types.rs`.
2. Implement `pause_subscription(subscriber, plan_id)` that changes status to `Paused` if currently `Active`.
3. Implement `resume_subscription(subscriber, plan_id)` that changes status to `Active` and updates `next_billing_time` appropriately (e.g., current time + cycle_seconds).
4. Update `execute_billing` to explicitly reject billing for `Paused` subscriptions.
5. Add relevant events (`sub_paused`, `sub_resumed`).

## Acceptance Criteria
- [ ] Users can pause an active subscription.
- [ ] Paused subscriptions cannot be billed.
- [ ] Users can resume a paused subscription, resetting the billing cycle.
- [ ] Comprehensive unit tests cover all new state transitions.

## Tech Stack
- Rust
- Soroban SDK
