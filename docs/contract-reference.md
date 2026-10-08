# SubPath Contract Reference

This document provides the technical reference for all entrypoints, storage structures, events, and error codes implemented in `subpath-core`.

## Public Entrypoints

### `initialize(env: Env) -> Result<(), Error>`
Initializes the contract storage and sets the initial plan counter to `1`.
* **Authorization**: None required. Callable once only.
* **Errors**: `AlreadyInitialized` if previously invoked.

### `create_plan(env: Env, merchant: Address, token: Address, amount: i128, cycle_seconds: u64) -> Result<u64, Error>`
Creates a new subscription plan with fixed billing cycle and payment terms.
* **Authorization**: `merchant.require_auth()`
* **Parameters**:
  * `merchant`: Address receiving recurring payments.
  * `token`: Address of the SEP-41 token used for payment.
  * `amount`: Base units per billing cycle (must be > 0).
  * `cycle_seconds`: Interval in seconds between recurring payments (must be > 0).
* **Returns**: Unique `plan_id: u64`.
* **Errors**: `InvalidAmount`, `InvalidCycle`.
* **Events Emitted**: `plan_add`

### `subscribe(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error>`
Registers a subscription to a specified plan and pulls the initial cycle payment immediately.
* **Authorization**: `subscriber.require_auth()`
* **Parameters**:
  * `subscriber`: Address paying for the subscription.
  * `plan_id`: Identifier of the plan to subscribe to.
* **Errors**: `PlanNotFound`, `AlreadySubscribed`, token transfer errors.
* **Events Emitted**: `sub_new`

### `pause_subscription(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error>`
Pauses an active subscription, preventing future billing executions.
* **Authorization**: `subscriber.require_auth()`
* **Errors**: `SubscriptionNotFound`, `SubscriptionCanceled`.
* **Events Emitted**: `sub_pause`

### `resume_subscription(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error>`
Resumes a paused subscription back to the `Active` state.
* **Authorization**: `subscriber.require_auth()`
* **Errors**: `SubscriptionNotFound`, `SubscriptionCanceled`.
* **Events Emitted**: `sub_resume`

### `cancel_subscription(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error>`
Permanently cancels an active or paused subscription.
* **Authorization**: `subscriber.require_auth()`
* **Errors**: `SubscriptionNotFound`.
* **Events Emitted**: `sub_end`

### `execute_billing(env: Env, subscriber: Address, plan_id: u64) -> Result<(), Error>`
Executes recurring payment collection for an active subscription once the due time has passed.
* **Authorization**: **Permissionless** (no caller authentication required).
* **Parameters**:
  * `subscriber`: Address of the subscriber.
  * `plan_id`: Identifier of the plan.
* **Errors**: `SubscriptionNotFound`, `SubscriptionCanceled`, `SubscriptionPaused`, `BillingTooEarly`, `PlanNotFound`, token allowance errors.
* **Events Emitted**: `sub_billed`

### Read-Only Helper Functions

* `get_plan(env: Env, plan_id: u64) -> Option<Plan>`: Returns plan details or None.
* `get_subscription(env: Env, subscriber: Address, plan_id: u64) -> Option<Subscription>`: Returns subscription state or None.
* `next_plan_id(env: Env) -> u64`: Returns current value of the plan sequence counter.

---

## Events Reference

All events use standard Soroban event structures:

| Event Name | Topics | Data | Description |
| :--- | :--- | :--- | :--- |
| `plan_add` | `["plan_add", merchant: Address]` | `plan_id: u64` | Emitted when a merchant creates a new plan. |
| `sub_new` | `["sub_new", subscriber: Address]` | `plan_id: u64` | Emitted when a user initiates a subscription. |
| `sub_pause` | `["sub_pause", subscriber: Address]` | `plan_id: u64` | Emitted when a subscription is paused. |
| `sub_resume` | `["sub_resume", subscriber: Address]` | `plan_id: u64` | Emitted when a subscription is resumed. |
| `sub_end` | `["sub_end", subscriber: Address]` | `plan_id: u64` | Emitted when a subscription is canceled. |
| `sub_billed` | `["sub_billed", subscriber: Address]` | `plan_id: u64` | Emitted upon successful recurring billing. |

---

## Error Codes

| Code | Error Variant | Description |
| :--- | :--- | :--- |
| `1` | `AlreadyInitialized` | Protocol initialize was already executed. |
| `2` | `NotInitialized` | Protocol has not been initialized. |
| `3` | `PlanNotFound` | Specified plan_id does not exist in storage. |
| `4` | `SubscriptionNotFound` | No subscription record exists for this pair. |
| `5` | `SubscriptionNotActive` | Subscription is not in an active state. |
| `6` | `InvalidAmount` | Plan amount must be greater than zero. |
| `7` | `InvalidCycle` | Billing cycle seconds must be greater than zero. |
| `8` | `AlreadySubscribed` | User is already actively subscribed to this plan. |
| `9` | `BillingTooEarly` | Current ledger timestamp is before next_billing_time. |
| `10` | `SubscriptionPaused` | Cannot execute billing while subscription is paused. |
| `11` | `SubscriptionCanceled` | Subscription is permanently canceled. |
