#!/bin/bash
# WalletD Crates.io Publish Script
# Run: ./scripts/publish-crates.sh
#
# Prerequisites:
# 1. cargo login <your-token>
# 2. All Cargo.toml files have correct metadata
# 3. Git repo is clean and tagged

set -e

echo "=========================================="
echo "WalletD Crates.io Publishing"
echo "=========================================="

# Check if logged in
if ! cargo login --help > /dev/null 2>&1; then
    echo "ERROR: Please run 'cargo login <token>' first"
    echo "Get token from: https://crates.io/settings/tokens"
    exit 1
fi

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

publish_crate() {
    local crate=$1
    echo -e "${YELLOW}Publishing $crate...${NC}"
    cargo publish -p "$crate" --allow-dirty
    echo -e "${GREEN}✓ $crate published${NC}"
    echo "Waiting 30s for crates.io to index..."
    sleep 30
}

dry_run_crate() {
    local crate=$1
    echo "Dry-run: $crate"
    cargo publish -p "$crate" --dry-run 2>&1 | tail -3
}

# ============================================================================
# TIER 1: Core crates (no internal dependencies)
# ============================================================================
echo ""
echo "=== TIER 1: Core Crates ==="

TIER1_CRATES=(
    "walletd-traits"
    "walletd-error"
    "walletd-core"
)

# ============================================================================
# TIER 2: Provider and coin crates
# ============================================================================
echo ""
echo "=== TIER 2: Provider & Coins ==="

TIER2_CRATES=(
    "walletd-provider"
    "walletd_bitcoin"
    "walletd_ethereum"
    "walletd_base"
    "walletd_arbitrum"
    "walletd_sui"
    "walletd_aptos"
    "walletd_monero"
    "walletd_hedera"
    "walletd_icp"
    "walletd_erc20"
)

# ============================================================================
# TIER 3: Unified crate
# ============================================================================
echo ""
echo "=== TIER 3: Unified SDK ==="

TIER3_CRATES=(
    "walletd"
)

# ============================================================================
# Main
# ============================================================================

MODE=${1:-"dry-run"}

if [ "$MODE" == "publish" ]; then
    echo ""
    echo "⚠️  PUBLISHING TO CRATES.IO (LIVE)"
    echo "Press Ctrl+C to cancel, or Enter to continue..."
    read -r

    for crate in "${TIER1_CRATES[@]}"; do
        publish_crate "$crate"
    done

    for crate in "${TIER2_CRATES[@]}"; do
        publish_crate "$crate"
    done

    for crate in "${TIER3_CRATES[@]}"; do
        publish_crate "$crate"
    done

    echo ""
    echo -e "${GREEN}=========================================="
    echo "✓ All crates published successfully!"
    echo "==========================================${NC}"

else
    echo ""
    echo "DRY RUN MODE (use './publish-crates.sh publish' to publish)"
    echo ""

    for crate in "${TIER1_CRATES[@]}"; do
        dry_run_crate "$crate"
    done

    for crate in "${TIER2_CRATES[@]}"; do
        dry_run_crate "$crate"
    done

    for crate in "${TIER3_CRATES[@]}"; do
        dry_run_crate "$crate"
    done
fi

echo ""
echo "Publish order summary:"
echo "  Tier 1: ${TIER1_CRATES[*]}"
echo "  Tier 2: ${TIER2_CRATES[*]}"
echo "  Tier 3: ${TIER3_CRATES[*]}"
