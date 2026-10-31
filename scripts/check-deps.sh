#!/usr/bin/env bash
# Assert the workspace dependency DAG fixed in PLAN.md §3.1:
#
#   core ← ingest   core ← store   api → {core, ingest, store}
#   desktop → {api, store}
#   stride-testkit is a dev-dependency only, everywhere.
#
# Reads the manifests directly; no extra tooling. Exits non-zero on any edge
# that is missing, extra, or in the wrong direction.
set -uo pipefail

cd "$(dirname "$0")/.."

declare -A EXPECTED
EXPECTED[crates/stride-core]=""
EXPECTED[crates/stride-ingest]="stride-core"
EXPECTED[crates/stride-store]="stride-core"
EXPECTED[crates/stride-api]="stride-core stride-ingest stride-store"
EXPECTED[crates/stride-testkit]=""
EXPECTED[desktop]="stride-api stride-store"

fail=0

hard_deps() {
  # the stride-* entries in the [dependencies] section only
  sed -n '/^\[dependencies\]/,/^\[/p' "$1" | grep -oE '^stride-[a-z]+' | sort -u | tr '\n' ' '
}

for crate in crates/stride-core crates/stride-ingest crates/stride-store \
              crates/stride-api crates/stride-testkit desktop; do
  manifest="$crate/Cargo.toml"
  [[ -f "$manifest" ]] || { echo "check-deps: missing manifest $manifest" >&2; fail=1; continue; }

  actual=$(hard_deps "$manifest" || true)
  actual=${actual% }   # trim trailing space
  expected=${EXPECTED[$crate]}

  if [[ "$actual" != "$expected" ]]; then
    echo "check-deps: $manifest depends on [$actual]; expected [$expected]" >&2
    fail=1
  fi

  # stride-testkit must only ever appear under [dev-dependencies]
  if sed -n '/^\[dependencies\]/,/^\[/p' "$manifest" | grep -qE '^stride-testkit'; then
    echo "check-deps: $manifest lists stride-testkit as a hard dependency (dev-only crate)" >&2
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo "check-deps: dependency graph does not match PLAN.md §3.1" >&2
fi
exit "$fail"
