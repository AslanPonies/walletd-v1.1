#!/bin/bash
# WalletD Code Coverage Script
# Run: ./scripts/coverage.sh [OPTIONS] [crate_name]
#
# Options:
#   --html        Generate HTML report
#   --json        Generate JSON report  
#   --lcov        Generate LCOV report (for Codecov)
#   --tarpaulin   Use cargo-tarpaulin instead of llvm-cov
#   --open        Open HTML report in browser
#   -p, --crate   Specific crate to test
#
# Examples:
#   ./scripts/coverage.sh                    # Summary for all crates
#   ./scripts/coverage.sh --html --open      # HTML report, open browser
#   ./scripts/coverage.sh -p walletd_bitcoin # Coverage for Bitcoin crate

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default settings
OUTPUT_FORMAT="summary"
USE_TARPAULIN=false
OPEN_HTML=false
SPECIFIC_CRATE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --html)
            OUTPUT_FORMAT="html"
            shift
            ;;
        --json)
            OUTPUT_FORMAT="json"
            shift
            ;;
        --lcov)
            OUTPUT_FORMAT="lcov"
            shift
            ;;
        --tarpaulin)
            USE_TARPAULIN=true
            shift
            ;;
        --open)
            OPEN_HTML=true
            shift
            ;;
        -p|--crate)
            SPECIFIC_CRATE="$2"
            shift 2
            ;;
        --help|-h)
            echo "WalletD Code Coverage Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --html          Generate HTML report"
            echo "  --json          Generate JSON report"
            echo "  --lcov          Generate LCOV report (for Codecov)"
            echo "  --tarpaulin     Use cargo-tarpaulin (slower but more compatible)"
            echo "  --open          Open HTML report in browser"
            echo "  -p, --crate     Generate coverage for specific crate"
            echo "  --help, -h      Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                           # Summary for all crates"
            echo "  $0 --html --open             # HTML report, open in browser"
            echo "  $0 -p walletd_bitcoin        # Coverage for Bitcoin crate"
            echo "  $0 --tarpaulin --html        # Use tarpaulin for HTML"
            exit 0
            ;;
        *)
            # Assume it's a crate name if not an option
            if [[ ! "$1" =~ ^- ]]; then
                SPECIFIC_CRATE="$1"
            else
                echo -e "${RED}Unknown option: $1${NC}"
                exit 1
            fi
            shift
            ;;
    esac
done

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}        WalletD Code Coverage Report${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Install required tools
if [ "$USE_TARPAULIN" = true ]; then
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
        cargo install cargo-tarpaulin
    fi
    COV_TOOL="tarpaulin"
else
    if ! command -v cargo-llvm-cov &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-llvm-cov...${NC}"
        cargo install cargo-llvm-cov
    fi
    if ! rustup component list --installed | grep -q llvm-tools; then
        echo -e "${YELLOW}Installing llvm-tools-preview...${NC}"
        rustup component add llvm-tools-preview
    fi
    COV_TOOL="llvm-cov"
fi

echo -e "${CYAN}Using: cargo-${COV_TOOL}${NC}"
echo ""

# Create coverage directory
mkdir -p coverage

# Define crates to test (excluding problematic ones)
CRATES=(
    "walletd-traits"
    "walletd_bitcoin"
    "walletd_ethereum"
    "walletd_base"
    "walletd_erc20"
    "walletd_icp"
    "walletd_hedera"
    "walletd_monero"
)

# Build command based on tool
if [ "$USE_TARPAULIN" = true ]; then
    run_coverage() {
        local crate=$1
        local output_format=$2
        
        case $output_format in
            html)
                cargo tarpaulin -p "$crate" --out Html --output-dir coverage 2>&1
                ;;
            json)
                cargo tarpaulin -p "$crate" --out Json --output-dir coverage 2>&1
                ;;
            lcov)
                cargo tarpaulin -p "$crate" --out Lcov --output-dir coverage 2>&1
                ;;
            *)
                cargo tarpaulin -p "$crate" --out Stdout 2>&1
                ;;
        esac
    }
    
    run_workspace_coverage() {
        local output_format=$1
        local packages=""
        for crate in "${CRATES[@]}"; do
            packages="$packages -p $crate"
        done
        
        case $output_format in
            html)
                cargo tarpaulin $packages --out Html --output-dir coverage 2>&1
                ;;
            json)
                cargo tarpaulin $packages --out Json --output-dir coverage 2>&1
                ;;
            lcov)
                cargo tarpaulin $packages --out Lcov --output-dir coverage 2>&1
                ;;
            *)
                cargo tarpaulin $packages --out Stdout 2>&1
                ;;
        esac
    }
else
    run_coverage() {
        local crate=$1
        local output_format=$2
        
        case $output_format in
            html)
                cargo llvm-cov -p "$crate" --html --output-dir coverage
                ;;
            json)
                cargo llvm-cov -p "$crate" --json --output-path coverage/"$crate"-coverage.json
                ;;
            lcov)
                cargo llvm-cov -p "$crate" --lcov --output-path coverage/"$crate".lcov
                ;;
            *)
                cargo llvm-cov -p "$crate"
                ;;
        esac
    }
    
    run_workspace_coverage() {
        local output_format=$1
        
        case $output_format in
            html)
                cargo llvm-cov --workspace \
                    --exclude walletd_icp_cli \
                    --exclude walletd-prasaga-avio \
                    --html --output-dir coverage
                ;;
            json)
                cargo llvm-cov --workspace \
                    --exclude walletd_icp_cli \
                    --exclude walletd-prasaga-avio \
                    --json --output-path coverage/workspace-coverage.json
                ;;
            lcov)
                cargo llvm-cov --workspace \
                    --exclude walletd_icp_cli \
                    --exclude walletd-prasaga-avio \
                    --lcov --output-path coverage/lcov.info
                ;;
            *)
                cargo llvm-cov --workspace \
                    --exclude walletd_icp_cli \
                    --exclude walletd-prasaga-avio
                ;;
        esac
    }
fi

# Run coverage
if [ -n "$SPECIFIC_CRATE" ]; then
    echo -e "${GREEN}Running coverage for: ${SPECIFIC_CRATE}${NC}"
    echo ""
    run_coverage "$SPECIFIC_CRATE" "$OUTPUT_FORMAT"
else
    echo -e "${GREEN}Running coverage for workspace...${NC}"
    echo ""
    run_workspace_coverage "$OUTPUT_FORMAT"
fi

# Handle output
echo ""
case $OUTPUT_FORMAT in
    html)
        if [ "$USE_TARPAULIN" = true ]; then
            REPORT_PATH="coverage/tarpaulin-report.html"
        else
            REPORT_PATH="coverage/html/index.html"
        fi
        echo -e "${GREEN}✓ HTML report: ${REPORT_PATH}${NC}"
        
        if [ "$OPEN_HTML" = true ]; then
            if command -v xdg-open &> /dev/null; then
                xdg-open "$REPORT_PATH" 2>/dev/null &
            elif command -v open &> /dev/null; then
                open "$REPORT_PATH"
            else
                echo -e "${YELLOW}Cannot auto-open. Please open manually: ${REPORT_PATH}${NC}"
            fi
        fi
        ;;
    json)
        if [ -n "$SPECIFIC_CRATE" ]; then
            echo -e "${GREEN}✓ JSON report: coverage/${SPECIFIC_CRATE}-coverage.json${NC}"
        else
            echo -e "${GREEN}✓ JSON report: coverage/workspace-coverage.json${NC}"
        fi
        ;;
    lcov)
        echo -e "${GREEN}✓ LCOV report: coverage/lcov.info${NC}"
        echo -e "${CYAN}Upload to Codecov: codecov -f coverage/lcov.info${NC}"
        ;;
esac

echo ""
echo -e "${GREEN}✓ Coverage complete!${NC}"
