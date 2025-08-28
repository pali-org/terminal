#!/bin/bash
# Local CI checks script - matches the GitHub Actions workflow

set -e

echo "🔍 Running local CI checks..."
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

run_check() {
    local name="$1"
    shift
    echo -e "${BLUE}🔄 $name${NC}"
    if "$@"; then
        echo -e "${GREEN}✅ $name passed${NC}"
    else
        echo -e "${RED}❌ $name failed${NC}"
        return 1
    fi
    echo
}

# Main CI checks (matching GitHub Actions workflow)
run_check "Code formatting" cargo fmt --all -- --check
run_check "Clippy linting" cargo clippy --all-targets --all-features -- -D warnings
run_check "Build with all features" cargo build --all-features
run_check "Run tests" cargo test --all-features
run_check "Build release binaries" cargo build --bin pacli --bin patui --release

echo -e "${GREEN}🎉 All checks passed! Your code is ready for CI.${NC}"