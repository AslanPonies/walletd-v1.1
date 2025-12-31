#!/bin/bash
# WalletD Security Audit Script
# 
# This script runs security checks on the WalletD codebase.
# Run: ./scripts/security-audit.sh
#
# Requirements:
#   cargo install cargo-audit
#   cargo install cargo-deny (optional, for comprehensive checks)

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              WalletD Security Audit                            ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Known accepted advisories (documented in SECURITY.md)
IGNORED_ADVISORIES=(
    "RUSTSEC-2025-0137"  # ruint - reciprocal_mg10 unsoundness (not used in our code)
)

# Build ignore flags
IGNORE_FLAGS=""
for advisory in "${IGNORED_ADVISORIES[@]}"; do
    IGNORE_FLAGS="$IGNORE_FLAGS --ignore $advisory"
done

echo "Step 1: Running cargo audit..."
echo "─────────────────────────────────────────────────────────────────"

if cargo audit $IGNORE_FLAGS; then
    echo -e "${GREEN}✓ No unacknowledged vulnerabilities found${NC}"
else
    echo -e "${RED}✗ Vulnerabilities detected! Review output above.${NC}"
    exit 1
fi

echo
echo "Step 2: Checking for outdated dependencies..."
echo "─────────────────────────────────────────────────────────────────"

if command -v cargo-outdated &> /dev/null; then
    cargo outdated --root-deps-only || true
else
    echo -e "${YELLOW}⚠ cargo-outdated not installed. Install with: cargo install cargo-outdated${NC}"
fi

echo
echo "Step 3: Running cargo deny (if available)..."
echo "─────────────────────────────────────────────────────────────────"

if command -v cargo-deny &> /dev/null; then
    cargo deny check advisories || true
    cargo deny check licenses || true
else
    echo -e "${YELLOW}⚠ cargo-deny not installed. Install with: cargo install cargo-deny${NC}"
fi

echo
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              Audit Complete                                    ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo
echo "Known Accepted Issues (documented in SECURITY.md):"
for advisory in "${IGNORED_ADVISORIES[@]}"; do
    echo "  - $advisory"
done
echo
echo "For details on ignored advisories, see SECURITY.md"
