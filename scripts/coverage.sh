#!/usr/bin/env bash
# Coverage script using cargo-llvm-cov
# Usage: ./scripts/coverage.sh

set -e

echo "Running coverage analysis..."

# Run llvm-cov with workspace-wide coverage
cargo llvm-cov \
  --workspace \
  --html \
  --output-dir target/coverage \
  --color always

echo ""
echo "Coverage reports generated:"
echo "  - HTML: target/coverage/html/index.html"
echo "  - LCOV: target/coverage/lcov.info"
echo ""
echo "To view HTML report:"
echo "  open target/coverage/html/index.html  # macOS"
echo "  xdg-open target/coverage/html/index.html  # Linux"
