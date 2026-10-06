# Issue 3: Billing Grace Periods and Failure Handling

## Summary
If a subscriber's allowance or balance is insufficient at the time of `execute_billing`, the transaction fails completely. We should introduce a grace period or explicit failure state that allows the subscription to remain active but flagged, rather than silently reverting.

## Why it matters
A failed payment shouldn't necessarily cancel a subscription immediately. A grace period allows merchants to re-try billing or notify the user to top up their wallet, creating a more robust recurring payment system.

## Scoped Implementation
1. Add a `PastDue` state to `SubscriptionStatus`.
2. Update `execute_billing` so that if the `token_client.transfer_from` fails, it catches the error (if possible in Soroban, or uses a balance check beforehand) and sets the status to `PastDue`.
3. Allow `execute_billing` to be called on `PastDue` subscriptions. If successful, revert status to `Active`.
4. Allow merchants to configure a `grace_period` on a `Plan`.

## Acceptance Criteria
- [ ] Subscriptions can enter a `PastDue` state if balance/allowance is insufficient.
- [ ] `PastDue` subscriptions can be brought back to `Active` upon successful billing.
- [ ] Tests cover insufficient balance and allowance scenarios transitioning to `PastDue`.

## Tech Stack
- Rust
- Soroban SDK
