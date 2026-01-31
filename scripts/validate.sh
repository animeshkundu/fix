#!/bin/bash
# validate.sh - Run all code quality checks
# Usage: ./scripts/validate.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CLI_DIR="$PROJECT_ROOT/fix-cli"

echo "==> Running validation checks..."
echo ""

# Check if fix-cli directory exists
if [ ! -d "$CLI_DIR" ]; then
    echo "Error: fix-cli directory not found at $CLI_DIR"
    exit 1
fi

cd "$CLI_DIR"

# Formatting check
echo "==> Checking formatting..."
if ! cargo fmt --check; then
    echo ""
    echo "Error: Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi
echo "    Formatting OK"
echo ""

# Clippy linting
echo "==> Running clippy..."
if ! cargo clippy -- -D warnings; then
    echo ""
    echo "Error: Clippy found warnings. Fix them before committing."
    exit 1
fi
echo "    Clippy OK"
echo ""

# Tests
echo "==> Running tests..."
if ! cargo test; then
    echo ""
    echo "Error: Tests failed."
    exit 1
fi
echo "    Tests OK"
echo ""

# Build check (optional, tests already build)
echo "==> Checking build..."
if ! cargo build --release 2>/dev/null; then
    echo ""
    echo "Error: Build failed."
    exit 1
fi
echo "    Build OK"
echo ""

echo "==> All checks passed!"
