Write-Host "🚀 Starting Phase 8 Deployment for SubPath Contract..." -ForegroundColor Cyan

Write-Host "1. Building the Soroban Contract..."
cargo build --target wasm32-unknown-unknown --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed. Please check your Rust environment." -ForegroundColor Red
    exit 1
}

Write-Host "2. Setting up Admin Identity (Testnet)..."
# Attempt to create key (will safely fail if it already exists)
stellar keys generate admin --network testnet 2>$null
$ADMIN_PUBKEY = stellar keys address admin
if (-not $ADMIN_PUBKEY) {
    Write-Host "❌ Failed to get admin public key. Make sure stellar-cli is installed." -ForegroundColor Red
    exit 1
}
Write-Host "   Admin Address: $ADMIN_PUBKEY" -ForegroundColor Green

Write-Host "3. Deploying Contract to Testnet..."
$CONTRACT_ID = stellar contract deploy --wasm target/wasm32-unknown-unknown/release/subpath_core.wasm --source admin --network testnet
if (-not $CONTRACT_ID) {
    Write-Host "❌ Deployment failed." -ForegroundColor Red
    exit 1
}
Write-Host "   Contract ID: $CONTRACT_ID" -ForegroundColor Green

Write-Host "4. Initializing Contract..."
stellar contract invoke --id $CONTRACT_ID --source admin --network testnet -- initialize --admin $ADMIN_PUBKEY
Write-Host "   Initialization complete!" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Magenta
Write-Host "✅ DEPLOYMENT SUCCESSFUL!" -ForegroundColor Green
Write-Host "Copy and paste the following into your subpath-app/apps/web/.env.local file:" -ForegroundColor Yellow
Write-Host ""
Write-Host "NEXT_PUBLIC_SUBPATH_CONTRACT_ID=$CONTRACT_ID"
Write-Host "========================================" -ForegroundColor Magenta
