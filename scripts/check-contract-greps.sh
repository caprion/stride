#!/usr/bin/env bash
# The frozen-contract greps that PLAN.md §6/§7 promise and docs/contract/
# api-paths.md names: no pairing/QR in the route table, and no string-built SQL
# anywhere in the Rust tree. These run over crates/ (the router and store live
# there from Slice 1+); today the crates are empty, so they pass trivially —
# the point is that the claim in the frozen contract is true at freeze time.
set -euo pipefail

cd "$(dirname "$0")/.."

fail=0

# No pairing codes / pairing / QR anywhere in the route table.
if grep -rniE 'pair[-_]?code|pairing|[^a-z]qr[^a-z]' crates/ 2>/dev/null; then
  echo "check-contract-greps: pairing/QR string found in crates/" >&2
  fail=1
fi

# No string-built SQL (a parameterised query has no format/concatenation in the
# SQL text). Heuristic: a SQL keyword directly followed by a string that embeds
# a format placeholder or concatenation.
if grep -rniE '(SELECT|INSERT|UPDATE|DELETE)[[:space:]]*["'"'"'][^"'"'"']*[{}]' crates/ 2>/dev/null; then
  echo "check-contract-greps: string-built SQL found in crates/" >&2
  fail=1
fi

# No `unsafe` outside the FFI crate. The seam is the one place it is allowed:
# the phone's Rust core is called across a C boundary, and that boundary is
# where the panic-catching lives. Everywhere else — the libraries, the
# desktop, and the phone's tree outside the seam — it stays out.
if grep -rnE '\bunsafe\b' --include='*.rs' --include='*.kt' --include='*.kts' crates/ desktop/ 2>/dev/null; then
  echo "check-contract-greps: unsafe found outside the FFI crate" >&2
  fail=1
fi
if grep -rnE '\bunsafe\b' --include='*.rs' --include='*.kt' --include='*.kts' android/ 2>/dev/null \
     | grep -v '^android/app/src/main/rust/stride-ffi/'; then
  echo "check-contract-greps: unsafe found outside the FFI crate" >&2
  fail=1
fi

exit "$fail"
