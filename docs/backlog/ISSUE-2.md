# Issue 2: Merchant Plan Metadata Extension

## Summary
Currently, `Plan` only holds numerical values (amount, cycle). We need a way for merchants to attach human-readable metadata (e.g., plan name, description, tier) to their plans.

## Why it matters
Dapps built on top of SubPath need to display plan details (like "Premium Tier") to users. While this can be done off-chain, having an on-chain reference (like an IPFS hash or a short string) ensures decentralized and verifiable plan information.

## Scoped Implementation
1. Modify the `Plan` struct in `types.rs` to include a `metadata` field (e.g., a `soroban_sdk::String` or `Bytes` representing an IPFS CID).
2. Update `create_plan` to accept this `metadata` parameter.
3. Update `plan_add` event to emit the metadata or hash.

## Acceptance Criteria
- [ ] `create_plan` successfully saves the metadata string.
- [ ] The metadata is retrievable via `get_plan`.
- [ ] The `plan_add` event includes the metadata reference.
- [ ] Tests verify metadata storage and retrieval.

## Tech Stack
- Rust
- Soroban SDK
