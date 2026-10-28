#!/usr/bin/env bash
# The Rust gate.
#
# The Python side has scripts/security_check.sh; this is the equivalent for
# the Rust core. It is the acceptance test for every workstream: a crate is
# finished when this is green.
#
#   ./scripts/rust_gate.sh          the whole workspace
#   ./scripts/rust_gate.sh stride-fit   one crate
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PASS=0
FAIL=0
WARN=0
ok()  { printf '\033[38;5;71mPASS\033[0m  %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '\033[38;5;203mFAIL\033[0m  %s\n' "$*"; FAIL=$((FAIL + 1)); }
warn(){ printf '\033[38;5;179mWARN\033[0m  %s\n' "$*"; WARN=$((WARN + 1)); }

TARGET="${1:---workspace}"
SCOPE=(--workspace)
if [[ "$TARGET" != "--workspace" ]]; then
  SCOPE=(-p "$TARGET")
fi

printf '\n\033[1m1. Formatting\033[0m\n'
if cargo fmt --all -- --check > /tmp/rust-gate-fmt.log 2>&1; then
  ok "the workspace is formatted"
else
  bad "rustfmt would change the workspace — run cargo fmt --all"
  tail -20 /tmp/rust-gate-fmt.log
fi

printf '\n\033[1m2. Lints\033[0m\n'
if cargo clippy "${SCOPE[@]}" --all-targets -- -D warnings > /tmp/rust-gate-clippy.log 2>&1; then
  ok "clippy is clean with warnings denied"
else
  bad "clippy found something"
  grep -E "^(error|warning)" /tmp/rust-gate-clippy.log | head -15
fi

printf '\n\033[1m3. The build\033[0m\n'
if cargo build --release "${SCOPE[@]}" > /tmp/rust-gate-build.log 2>&1; then
  ok "the release build succeeds"
else
  bad "the release build failed"
  grep -E "^error" /tmp/rust-gate-build.log | head -10
fi

printf '\n\033[1m4. Tests, including the golden fixtures\033[0m\n'
if cargo test "${SCOPE[@]}" > /tmp/rust-gate-test.log 2>&1; then
  passed=$(grep -oE "[0-9]+ passed" /tmp/rust-gate-test.log | awk '{s+=$1} END {print s+0}')
  ok "the tests pass (${passed} passed)"
  if grep -q "golden" /tmp/rust-gate-test.log; then
    golden=$(grep -cE "golden.*ok|ok.*golden" /tmp/rust-gate-test.log)
    ok "the golden fixtures are exercised ($golden golden assertions)"
  else
    warn "no golden test ran — a crate with no golden test is not finished"
  fi
else
  bad "the tests failed"
  grep -E "^(FAILED|failures:|test .* FAILED)" /tmp/rust-gate-test.log | head -15
fi

printf '\n\033[1m5. No panics in the library\033[0m\n'
# A panic in a library is a process kill on a phone. The FFI boundary catches
# them, but the rule is that the library does not raise them at all.
# The workspace denies unwrap/expect/panic, so clippy is the authority here
# and not a grep: it knows what is test code.
if cargo clippy "${SCOPE[@]}" --all-targets -- -D warnings > /tmp/rust-gate-panic.log 2>&1; then
  ok "no unwrap, expect or panic outside test code"
else
  bad "unwrap, expect or panic outside test code"
  grep -E "^error" -A 2 /tmp/rust-gate-panic.log | grep -E "^error|-->" | head -12
fi

# `unsafe` is allowed in exactly one place: the C ABI.
unsafe_elsewhere=$(grep -rn --include=*.rs -E '\bunsafe\b' crates/ \
  | grep -v 'crates/stride-api/src/ffi.rs' \
  | grep -vE 'unsafe_code|unsafe impl|// |unsafe block' | head -5)
if [[ -z "$unsafe_elsewhere" ]]; then
  ok "unsafe is confined to the FFI boundary"
else
  bad "unsafe appears outside the FFI boundary"
  echo "$unsafe_elsewhere" | head -5
fi

printf '\n\033[1m6. The dependency set\033[0m\n'
allowed="serde serde_json chrono thiserror anyhow uuid rusqlite reqwest uniffi"
extra=$(awk '/^\[dependencies\]/,/^\[/' crates/*/Cargo.toml desktop/Cargo.toml 2>/dev/null \
  | grep -oE '^[a-z0-9_-]+' | sort -u \
  | grep -vE "^(${allowed})$|^(dependencies|workspace|features|serde|serde_json|chrono|thiserror|anyhow|uuid|rusqlite|reqwest|uniffi)$" \
  | grep -v '^$' || true)
if [[ -z "$extra" ]]; then
  ok "no dependency outside the approved set"
else
  warn "a dependency outside the approved set: $extra"
fi

printf '\n\033[1m7. Size\033[0m\n'
if [[ -f "$ROOT/target/release/stride" ]]; then
  size=$(du -h "$ROOT/target/release/stride" | cut -f1)
  bytes=$(stat -c%s "$ROOT/target/release/stride")
  if [[ $bytes -lt 15000000 ]]; then
    ok "the desktop binary is $size"
  else
    bad "the desktop binary is $size — over the 15 MB budget"
  fi
else
  warn "no release binary to measure"
fi

printf '\n\033[1m%d passed, %d failed, %d warnings\033[0m\n' "$PASS" "$FAIL" "$WARN"
[[ $FAIL -eq 0 ]] || { printf '\033[38;5;203mThe Rust gate failed. Fix it before shipping.\033[0m\n'; exit 1; }
printf '\033[38;5;71mThe Rust gate is green.\033[0m\n'
