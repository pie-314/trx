#!/usr/bin/env bash
set -euo pipefail
section() {
    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}
section "Formatting"
cargo fmt --all
section "Applying Clippy Fixes"
cargo clippy \
    --fix \
    --workspace \
    --all-features \
    --allow-dirty \
    --allow-staged
section "Clippy Verification"
cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- -D warnings -A dead_code -A clippy::type_complexity
echo
echo "✓ All fixes applied successfully"
