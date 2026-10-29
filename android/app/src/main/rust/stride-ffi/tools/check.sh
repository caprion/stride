#!/usr/bin/env bash
# The phone's crate, checked the way the gate checks the workspace.
#
# The crate is a standalone cargo workspace (the root manifest belongs to the
# orchestrator), so `cargo test --workspace` at the root does not reach it.
# This runs the same four things the gate runs — formatting, clippy with
# warnings denied, the build and the tests — over the phone's crate and the
# binding generator.
#
#     ./tools/check.sh
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE="$(cd "$HERE/.." && pwd)"

cd "$CRATE"

pass=0
fail=0
ok()  { printf '\\033[38;5;71mPASS\\033[0m  %s\\n' "$*"; pass=$((pass + 1)); }
bad() { printf '\\033[38;5;203mFAIL\\033[0m  %s\\n' "$*"; fail=$((fail + 1)); }

# Only this crate and the binding generator: a bare `--all` also *reports*
# the formatting of the path dependencies, which belong to other workstreams.
if cargo fmt -p stride-ffi -p stride-bindgen -- --check > /tmp/stride-ffi-fmt.log 2>&1; then
  ok "the crate is formatted"
else
  bad "rustfmt would change the crate"
  tail -20 /tmp/stride-ffi-fmt.log
fi

if cargo clippy --no-deps --all-targets -- -D warnings > /tmp/stride-ffi-clippy.log 2>&1; then
  ok "clippy is clean with warnings denied"
else
  bad "clippy found something"
  grep -E '^(error|warning)' -A 3 /tmp/stride-ffi-clippy.log | head -15
fi

if cargo build --release > /tmp/stride-ffi-build.log 2>&1; then
  ok "the release build succeeds"
else
  bad "the release build failed"
  grep -E '^error' -A 3 /tmp/stride-ffi-build.log | head -10
fi

if cargo test > /tmp/stride-ffi-test.log 2>&1; then
  passed=$(grep -oE '[0-9]+ passed' /tmp/stride-ffi-test.log | awk '{s+=$1} END {print s+0}')
  ok "the tests pass (${passed} passed)"
else
  bad "the tests failed"
  grep -E '(FAILED|failures:|panicked)' -A 2 /tmp/stride-ffi-test.log | head -15
fi

printf '\\n%d passed, %d failed\\n' "$pass" "$fail"
[[ $fail -eq 0 ]] || exit 1
