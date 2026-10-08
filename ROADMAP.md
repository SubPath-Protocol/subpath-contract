# SubPath Contract Roadmap

This roadmap outlines the milestones and prospective areas of contribution for the SubPath smart contracts.

## Current Milestone (v0.1.0 - Testnet Release)

* Core subscription lifecycle (plan creation, subscribe, pause, resume, cancel).
* Permissionless billing execution via SEP-41 token allowances.
* Complete event emissions for off-chain indexing.
* Verified Testnet deployment and automated CI testing.

## Near-Term Maintenance

* Automated gas optimization and footprint profiling across recurring billing invocations.
* Extended Soroban unit and property-based test coverage.
* Formal interface definition documentation and SDK code generation synchronization.

## Post-Approval Contribution Areas

* **Configurable Relayer Fees**: Optional fee incentives rewarding third-party relayers executing billing transactions.
* **Batch Billing Invocations**: Multi-subscriber billing processing within a single Soroban transaction invocation.
* **Tiered & Prorated Billing Models**: Support for tiered usage plans and mid-cycle plan upgrades.
* **Security & Formal Verification**: Preparation for comprehensive third-party smart contract audits prior to Mainnet readiness.
