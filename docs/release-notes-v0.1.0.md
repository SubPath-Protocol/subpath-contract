# SubPath Core Contracts v0.1.0 Release Notes

**Release Date**: 2026-10-08  
**Repository**: `SubPath-Protocol/subpath-contract`  
**Network**: Stellar Testnet (`Test SDF Network ; September 2015`)  
**License**: MIT  

---

## Overview

SubPath v0.1.0 is the initial public prototype release of the SubPath core protocol on Stellar Soroban. The smart contract provides a decentralized, non-custodial recurring payments engine enabling merchants to create token subscription plans and allowing subscribers to authorize recurring payments pulled via standard SEP-41 token allowances.

---

## Verified On-Chain Deployment

* **Contract ID**: [`CC4ZFZ64RQ6CG3PTBNDB4A6YB7SJEW2NZ56YNNBBZVC7HDQTNIKWLUV3`](https://stellar.expert/explorer/testnet/contract/CC4ZFZ64RQ6CG3PTBNDB4A6YB7SJEW2NZ56YNNBBZVC7HDQTNIKWLUV3)
* **WASM Hash**: `b96d0c3dd54253e4ceec2158d9960df19613101ff62acc8f307635c61888fb49`
* **Deployer Address**: `GBOWTBFBE5DFOLVESCOQDJERT2K7BAGNOZACMYS3FIA62IXOOGS4SJQU`
* **Native Token SAC**: `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`
* **Deployment Transaction**: `6f16e1c93e583914bef7c87290499acb3879dff02e62e61e10495d83b4a47b5a`

---

## What is Included

1. **Plan Configuration (`create_plan`)**: Merchants configure subscription plans specifying token address, billing amount, and cadence duration in seconds.
2. **Subscriber Onboarding (`subscribe`)**: Subscribers opt in, and the initial cycle billing payment is transferred immediately to the merchant.
3. **Automated Permissionless Billing (`execute_billing`)**: Relayers or autonomous cron workers pull recurring installments via pre-approved SEP-41 allowances once `ledger_timestamp >= next_billing_time`.
4. **Subscriber Controls**: Subscribers retain sovereign key control and can call `pause_subscription`, `resume_subscription`, or `cancel_subscription` at any time.
5. **Event Emission**: Emits structured topics for all state transitions (`plan_add`, `sub_new`, `sub_billed`, `sub_pause`, `sub_resume`, `sub_end`).
6. **Automated CI Validation**: Verified via automated GitHub Actions workflow running formatting, clippy checks, unit test suites, and WASM contract builds.

---

## Verification Evidence & Documentation

* [README.md](../README.md): Project overview and quick start.
* [Testnet Deployment Evidence](testnet-deployment.md): Complete transaction evidence of the on-chain subscription lifecycle.
* [Contract Reference](contract-reference.md): Full entrypoint interface, parameters, events, and error codes.
* [Architecture Guide](architecture.md): Protocol storage layout and state transition model.
* [Security Model](security-model.md): Non-custodial guarantees and threat mitigations.
* [Changelog](../CHANGELOG.md): Historical change records.

---

## Security & Audit Status

* **Status**: Unaudited Prototype.
* **Network**: Deployed exclusively on Stellar Testnet for testing and integration.
* **Mainnet Caution**: Not audited for Mainnet financial use. Do not risk mainnet capital.
* **Vulnerability Reporting**: Report security findings privately to `security@subpath-protocol.com`. See [SECURITY.md](../SECURITY.md).

---

## Upgrade & Setup Notes

* Developers integrating SubPath can target contract `CC4ZFZ64RQ6CG3PTBNDB4A6YB7SJEW2NZ56YNNBBZVC7HDQTNIKWLUV3` on Stellar Testnet RPC `https://soroban-testnet.stellar.org`.
* Subscribers must submit a token `approve` transaction granting spending allowance to the contract before or upon subscribing.
