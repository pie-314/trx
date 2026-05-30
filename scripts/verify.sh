#!/usr/bin/env bash
set -euo pipefail
section() {
    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}
section "Formatting Check"
cargo fmt --all -- --check
section "Clippy"
cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- -D warnings -A dead_code -A clippy::type_complexity
section "Tests"
cargo test --workspace --all-features
section "Build"
cargo build --workspace --all-targets --all-features
echo
echo "✓ All verification checks passed"
