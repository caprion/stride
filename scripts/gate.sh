#!/usr/bin/env bash
# The local gate the plan promises (PLAN.md §7): fmt, clippy, test, the DAG
# check, and both scans. Exits non-zero on the first failure.
set -euo pipefail

cd "$(dirname "$0")/.."

step() { printf '\n== %s ==\n' "$1"; }

step "fmt"
cargo fmt --check

step "clippy"
cargo clippy --workspace --all-targets -- -D warnings

step "test"
cargo test --workspace

step "dependency DAG"
bash scripts/check-deps.sh

step "secret scan"
bash scripts/scan-secrets.sh

step "privacy scan"
bash scripts/scan-privacy.sh

printf '\n== gate: all green ==\n'
