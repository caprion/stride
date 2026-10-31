#!/usr/bin/env bash
# The local gate — the single authoritative list of checks, run by CI and by a
# human alike. If a gate is added, add it here and it runs everywhere.
# Exits non-zero on the first failure.
set -euo pipefail

cd "$(dirname "$0")/.."

step() { printf '\n== %s ==\n' "$1"; }

step "fmt"
cargo fmt --check

step "clippy"
cargo clippy --workspace --all-targets -- -D warnings

step "test (includes the FIT position privacy gate)"
cargo test --workspace

step "dependency DAG + lint-table check"
bash scripts/check-deps.sh

step "corpus integrity (FIT SHA-256)"
bash scripts/check-corpus.sh

step "contract greps (no pairing/QR routes, no string-built SQL)"
bash scripts/check-contract-greps.sh

step "secret scan"
bash scripts/scan-secrets.sh

step "privacy scan"
bash scripts/scan-privacy.sh

step "scanner self-tests"
# A scanner that never fails is worse than no scanner: prove each one catches
# its bad fixtures and passes its good ones.
for bad in fixtures/scan-tests/secrets-bad*.txt; do
  if bash scripts/scan-secrets.sh "$bad" >/dev/null 2>&1; then
    echo "gate: scan-secrets did not catch $bad" >&2; exit 1
  fi
done
bash scripts/scan-secrets.sh fixtures/scan-tests/secrets-good.txt
for bad in fixtures/scan-tests/privacy-bad.txt fixtures/scan-tests/privacy-bad.json; do
  if bash scripts/scan-privacy.sh "$bad" >/dev/null 2>&1; then
    echo "gate: scan-privacy did not catch $bad" >&2; exit 1
  fi
done
bash scripts/scan-privacy.sh fixtures/scan-tests/privacy-good.txt
echo "scanner self-tests passed"

printf '\n== gate: all green ==\n'
