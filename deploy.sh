#!/usr/bin/env bash
set -euo pipefail

echo "🚀 Starting Deployment for SubPath Core Contract..."

echo "1. Building the Soroban Contract..."
stellar contract build

echo "2. Setting up Admin Identity (Testnet)..."
stellar keys generate admin --network testnet 2>/dev/null || true
stellar keys fund admin --network testnet
ADMIN_PUBKEY=$(stellar keys address admin)
echo "   Admin Address: ${ADMIN_PUBKEY}"

echo "3. Deploying Contract to Testnet..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/subpath_core.wasm \
  --source admin \
  --network testnet)

echo "   Contract ID: ${CONTRACT_ID}"

echo "4. Initializing Contract..."
stellar contract invoke \
  --id "${CONTRACT_ID}" \
  --source admin \
  --network testnet \
  -- initialize

echo ""
echo "========================================"
echo "✅ DEPLOYMENT SUCCESSFUL!"
echo "Contract ID: ${CONTRACT_ID}"
echo "Admin Address: ${ADMIN_PUBKEY}"
echo "========================================"
