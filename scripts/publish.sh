#!/bin/bash
# WalletD Crates.io Publish Script
#
# This script publishes all WalletD crates to crates.io in dependency order.
# 
# Prerequisites:
# 1. Run `cargo login` with your crates.io API token
# 2. Ensure all versions are correct and unique
# 3. Run with --dry-run first to verify
#
# Usage:
#   ./scripts/publish.sh --dry-run   # Test without publishing
#   ./scripts/publish.sh             # Actually publish

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
DRY_RUN=""
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN="--dry-run"
    echo -e "${YELLOW}🔍 DRY RUN MODE - No packages will be published${NC}"
fi

# Crates in dependency order (leaf crates first)
CRATES=(
    # Tier 1: No internal dependencies
    "walletd-traits"
    "walletd-error"
    "walletd-provider"
    
    # Tier 2: Depends on traits
    "walletd-core"
    "walletd_icp_api"
    
    # Tier 3: Individual chains (depend on traits/core)
    "walletd_bitcoin"
    "walletd_ethereum"
    "walletd_hedera"
    "walletd_icp"
    "walletd_monero"
    "walletd_base"
    "walletd_erc20"
    "walletd_sui"
    "walletd-prasaga-avio"
    
    # Tier 4: Unified SDK (depends on all)
    "walletd"
    
    # Tier 5: WASM (optional, publish separately to npm)
    # "walletd-wasm"
)

# Delay between publishes (crates.io rate limit)
PUBLISH_DELAY=30

cd "$PROJECT_ROOT"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}       WalletD Crates.io Publisher${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check login status
if [[ -z "$DRY_RUN" ]]; then
    echo -e "${YELLOW}Checking crates.io authentication...${NC}"
    if ! cargo login --help > /dev/null 2>&1; then
        echo -e "${RED}❌ Please run 'cargo login' first${NC}"
        exit 1
    fi
fi

# Show versions
echo -e "${BLUE}📦 Crates to publish:${NC}"
for crate in "${CRATES[@]}"; do
    version=$(grep "^version = " **/Cargo.toml */**/Cargo.toml 2>/dev/null | grep "/$crate/" | head -1 | cut -d'"' -f2)
    if [[ -n "$version" ]]; then
        echo "  - $crate v$version"
    else
        # Try to get version from cargo metadata
        version=$(cargo pkgid -p "$crate" 2>/dev/null | rev | cut -d'#' -f1 | rev | cut -d':' -f2)
        echo "  - $crate v$version"
    fi
done
echo ""

# Confirm
if [[ -z "$DRY_RUN" ]]; then
    echo -e "${YELLOW}⚠️  This will publish to crates.io (permanent!)${NC}"
    read -p "Continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

# Publish each crate
FAILED=()
SUCCEEDED=()

for crate in "${CRATES[@]}"; do
    echo ""
    echo -e "${BLUE}━━━ Publishing $crate ━━━${NC}"
    
    if cargo publish -p "$crate" $DRY_RUN 2>&1; then
        SUCCEEDED+=("$crate")
        echo -e "${GREEN}✅ $crate published successfully${NC}"
        
        # Wait between publishes (skip for dry-run)
        if [[ -z "$DRY_RUN" && "$crate" != "${CRATES[-1]}" ]]; then
            echo -e "${YELLOW}Waiting ${PUBLISH_DELAY}s for crates.io indexing...${NC}"
            sleep $PUBLISH_DELAY
        fi
    else
        FAILED+=("$crate")
        echo -e "${RED}❌ Failed to publish $crate${NC}"
        
        if [[ -z "$DRY_RUN" ]]; then
            echo -e "${YELLOW}Continue with remaining crates? [y/N]${NC}"
            read -p "" -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                break
            fi
        fi
    fi
done

# Summary
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}       Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [[ ${#SUCCEEDED[@]} -gt 0 ]]; then
    echo -e "${GREEN}✅ Succeeded (${#SUCCEEDED[@]}):${NC}"
    for crate in "${SUCCEEDED[@]}"; do
        echo "   - $crate"
    done
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo -e "${RED}❌ Failed (${#FAILED[@]}):${NC}"
    for crate in "${FAILED[@]}"; do
        echo "   - $crate"
    done
    exit 1
fi

if [[ -n "$DRY_RUN" ]]; then
    echo ""
    echo -e "${YELLOW}This was a dry run. Run without --dry-run to publish.${NC}"
else
    echo ""
    echo -e "${GREEN}🎉 All crates published successfully!${NC}"
    echo ""
    echo "View on crates.io:"
    echo "  https://crates.io/crates/walletd"
fi
