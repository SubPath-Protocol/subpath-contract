# Changelog

All notable changes to the `subpath-contract` repository will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-08

### Added
* Core Soroban smart contract (`subpath-core`) supporting token-denominated subscription plans.
* Plan creation entrypoint (`create_plan`) with customizable token, amount, and recurring cycle duration.
* Subscriber onboarding (`subscribe`) with immediate first payment execution.
* Subscription lifecycle methods: `pause_subscription`, `resume_subscription`, and `cancel_subscription`.
* Automated permissionless billing execution entrypoint (`execute_billing`) pulling pre-approved SEP-41 allowances.
* Read-only inspection entrypoints: `get_plan`, `get_subscription`, and `next_plan_id`.
* Comprehensive event emissions: `plan_add`, `sub_new`, `sub_pause`, `sub_resume`, `sub_end`, and `sub_billed`.
* Testnet deployment verification under Contract ID `CC4ZFZ64RQ6CG3PTBNDB4A6YB7SJEW2NZ56YNNBBZVC7HDQTNIKWLUV3`.
* Automated CI pipeline verifying `cargo fmt`, `cargo clippy`, unit tests, and WASM contract builds.

### Limitations
* Prototype release evaluated exclusively on Stellar Testnet.
* Contract logic has not received third-party professional security audits.
* Allowance management relies on standard off-chain subscriber approval prior to billing cycles.
