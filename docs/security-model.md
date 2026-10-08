# SubPath Contract Security Model

## Security Principles

SubPath is designed with non-custodial and bounded authorization principles:

1. **Non-Custodial Design**: The SubPath smart contract never holds custody of user assets. Tokens are transferred directly from the subscriber to the merchant.
2. **Explicit Authorization**:
   * Plan creation requires explicit merchant authorization (`merchant.require_auth()`).
   * Initial subscription requires explicit subscriber authorization (`subscriber.require_auth()`).
   * State transitions (`pause`, `resume`, `cancel`) require subscriber authorization (`subscriber.require_auth()`).
3. **Allowance-Bounded Transfers**: Recurring billing payments rely on SEP-41 token allowances approved by the subscriber directly on the token contract. The contract cannot withdraw more than the subscriber has explicitly approved or more than the plan amount per cycle.
4. **Time-Enforced Cadence**: Automated billing (`execute_billing`) cannot be invoked prematurely. If `ledger().timestamp() < sub.next_billing_time`, the invocation immediately reverts with `Error::BillingTooEarly`.
5. **State Guardrails**: Inactive, paused, or canceled subscriptions immediately revert any attempt to pull funds.

## Threat Analysis & Mitigations

| Threat | Risk Level | Mitigation in Contract |
| :--- | :--- | :--- |
| **Premature Billing** | Medium | Checked on-chain: `env.ledger().timestamp() < sub.next_billing_time` reverts with `BillingTooEarly`. |
| **Over-Billing / Draining** | High | Fixed transfer amount strictly tied to `plan.amount`. Transfers fail if subscriber allowance or balance is insufficient. |
| **Unauthorized Cancellation** | High | Subscriber must authorize cancellation (`subscriber.require_auth()`). |
| **Unauthorized Plan Modification** | High | Plan parameters are immutable once created. Plans cannot be altered after creation. |
| **Contract Custody Theft** | Low | Contract does not hold funds. Payments transfer directly from subscriber to merchant. |
| **Reentrancy** | Low | Soroban runtime does not permit arbitrary reentrant calls, and state updates follow checks-effects-interactions order. |

## Current Audit & Deployment Status

* **Audit Status**: Unaudited prototype (`v0.1.0`). No third-party professional audit has been completed.
* **Network Status**: Deployed exclusively on Stellar Testnet for protocol verification and community feedback.
* **Mainnet Caution**: The contract is NOT audited for Mainnet production deployments. Do not risk mainnet capital.
