#!/bin/bash
# WalletD WASM Build Script
#
# Usage:
#   ./scripts/build-wasm.sh          # Build for web (default)
#   ./scripts/build-wasm.sh bundler  # Build for bundlers (webpack, etc.)
#   ./scripts/build-wasm.sh nodejs   # Build for Node.js
#   ./scripts/build-wasm.sh all      # Build all targets

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_CRATE="$PROJECT_ROOT/crates/walletd-wasm"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check dependencies
check_deps() {
    if ! command -v wasm-pack &> /dev/null; then
        echo -e "${YELLOW}Installing wasm-pack...${NC}"
        cargo install wasm-pack
    fi
    
    if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
        echo -e "${YELLOW}Adding wasm32-unknown-unknown target...${NC}"
        rustup target add wasm32-unknown-unknown
    fi
}

# Build for specific target
build_target() {
    local target=$1
    local out_dir=$2
    
    echo -e "${BLUE}Building for target: ${target}${NC}"
    
    cd "$WASM_CRATE"
    
    if [ -n "$out_dir" ]; then
        wasm-pack build --target "$target" --release --out-dir "$out_dir"
    else
        wasm-pack build --target "$target" --release
    fi
    
    echo -e "${GREEN}✅ Built: $target${NC}"
}

# Show output sizes
show_sizes() {
    echo ""
    echo -e "${BLUE}📦 Output Sizes:${NC}"
    
    if [ -d "$WASM_CRATE/pkg" ]; then
        SIZE=$(ls -lh "$WASM_CRATE/pkg"/*.wasm 2>/dev/null | awk '{print $5}' | head -1)
        echo "  web/bundler: $SIZE"
    fi
    
    if [ -d "$WASM_CRATE/pkg-node" ]; then
        SIZE=$(ls -lh "$WASM_CRATE/pkg-node"/*.wasm 2>/dev/null | awk '{print $5}' | head -1)
        echo "  nodejs: $SIZE"
    fi
}

# Main
main() {
    cd "$PROJECT_ROOT"
    check_deps
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}       WalletD WASM Build${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    case "${1:-web}" in
        web)
            build_target "web"
            ;;
        bundler)
            build_target "bundler"
            ;;
        nodejs|node)
            build_target "nodejs" "pkg-node"
            ;;
        all)
            build_target "web"
            build_target "bundler" "pkg-bundler"
            build_target "nodejs" "pkg-node"
            ;;
        clean)
            echo -e "${YELLOW}Cleaning WASM build artifacts...${NC}"
            rm -rf "$WASM_CRATE/pkg" "$WASM_CRATE/pkg-node" "$WASM_CRATE/pkg-bundler"
            echo -e "${GREEN}✅ Cleaned${NC}"
            exit 0
            ;;
        *)
            echo "WalletD WASM Build Script"
            echo ""
            echo "Usage: $0 <target>"
            echo ""
            echo "Targets:"
            echo "  web       Build for browsers without bundler (default)"
            echo "  bundler   Build for bundlers (webpack, vite, etc.)"
            echo "  nodejs    Build for Node.js"
            echo "  all       Build all targets"
            echo "  clean     Remove build artifacts"
            echo ""
            echo "Examples:"
            echo "  $0                    # Build for web"
            echo "  $0 bundler            # Build for webpack"
            echo "  $0 all                # Build all"
            exit 0
            ;;
    esac
    
    show_sizes
    
    echo ""
    echo -e "${GREEN}✅ Build complete!${NC}"
    echo ""
    echo "Output directory: $WASM_CRATE/pkg/"
    echo ""
    echo "Usage in HTML:"
    echo '  <script type="module">'
    echo '    import init, { EthereumWallet } from "./pkg/walletd_wasm.js";'
    echo '    await init();'
    echo '  </script>'
}

main "$@"
