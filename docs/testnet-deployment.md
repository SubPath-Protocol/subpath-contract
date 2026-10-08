# SubPath Testnet Deployment Evidence

This document records the verified deployment parameters and on-chain lifecycle transactions for the SubPath core protocol on Stellar Testnet.

## Contract Deployment Details

* **Network**: Stellar Testnet (`Test SDF Network ; September 2015`)
* **Contract ID**: `CC4ZFZ64RQ6CG3PTBNDB4A6YB7SJEW2NZ56YNNBBZVC7HDQTNIKWLUV3`
* **WASM Hash**: `b96d0c3dd54253e4ceec2158d9960df19613101ff62acc8f307635c61888fb49`
* **Deployer / Admin Address**: `GBOWTBFBE5DFOLVESCOQDJERT2K7BAGNOZACMYS3FIA62IXOOGS4SJQU`
* **Deployment Transaction**: `6f16e1c93e583914bef7c87290499acb3879dff02e62e61e10495d83b4a47b5a`
* **Stellar Expert URL**: [Explorer Link](https://stellar.expert/explorer/testnet/tx/6f16e1c93e583914bef7c87290499acb3879dff02e62e61e10495d83b4a47b5a)
* **Native Token SAC Address**: `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`

---

## On-Chain Lifecycle Verification

All core protocol transactions have been executed and verified directly on the live Stellar Testnet:

### 1. Plan Creation (`plan_add`)
* **Transaction Hash**: `3e2b6dbeb3ad0b4ae14e23f7bf9d9ca7158b7be1e5e9a1e92ad013526aa9b801`
* **Plan ID**: `1`
* **Amount**: `1000000` stroops (0.1 XLM)
* **Cycle**: `10` seconds
* **Status**: `SUCCESS`

### 2. Token Allowance Approval (`approve`)
* **Transaction Hash**: `2b8ae158ede758a4a96a74742237cb6938c9b6dfb8bb15d9978503c8a95b2b37`
* **Spender**: `CC4ZFZ64RQ6CG3PTBNDB4A6YB7SJEW2NZ56YNNBBZVC7HDQTNIKWLUV3`
* **Allowance Amount**: `10000000` stroops (1.0 XLM)
* **Status**: `SUCCESS`

### 3. Subscription Initiation (`sub_new`)
* **Transaction Hash**: `ba9e4350c3f31870f574186c7bbb5ae40e99285d4c93aa8ecaa1428777bb7704`
* **Initial Transfer**: `1000000` stroops transferred from subscriber to merchant
* **Next Billing Time**: `1791372787`
* **Status**: `SUCCESS`

### 4. Automated Permissionless Billing (`sub_billed`)
* **Transaction Hash**: `c608d950e375cbd8f406ef02b763685bcab2ac67f86495c1ad7da642f879652a`
* **Recurring Transfer**: `1000000` stroops transferred via allowance
* **Updated Next Billing Time**: `1791372797`
* **Status**: `SUCCESS`

### 5. Pause Lifecycle (`sub_pause`)
* **Transaction Hash**: `c8f187ef460de159ab7607059d7a780c0e9a683a662d853bb80dd91765926a5d`
* **Billing Enforcement**: Re-attempted billing rejected with `Error(Contract, #10)` (`SubscriptionPaused`).
* **Status**: `SUCCESS`

### 6. Resume Lifecycle (`sub_resume`)
* **Transaction Hash**: `83e4d11317e48337f6394cef19fb2237b51de4c0a26edc74c57be7d64a167a21`
* **Status**: `SUCCESS`

### 7. Cancellation Lifecycle (`sub_end`)
* **Transaction Hash**: `aa05bdaea9bc8cf9aaa4cc675c13faa2a1389e68620978cec0c69b24959e0d3d`
* **Billing Enforcement**: Re-attempted billing rejected with `Error(Contract, #5)` (`SubscriptionNotActive`).
* **Status**: `SUCCESS`
