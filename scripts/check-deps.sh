#!/usr/bin/env bash
# Assert the workspace dependency DAG fixed in PLAN.md §3.1, derived from
# `cargo metadata` (a text parse of [dependencies] misses the table form
# `[dependencies.stride-core]`, a renamed dependency `core = { package =
# "stride-core" }`, and a seventh member):
#
#   core ← ingest   core ← store   api → {core, ingest, store}
#   desktop → {api, store}
#   stride-testkit is a dev-dependency only, everywhere.
#
# Only *internal* edges count. An external dependency (serde, reqwest, …) is
# not an edge of this graph, so the comparison filters to the workspace's own
# members before it compares anything.
#
# It also asserts no crate overrides the workspace lint table, so a crate
# cannot silently re-allow `unwrap` (PLAN.md §7).
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v jq >/dev/null 2>&1; then
  echo "check-deps: jq is required" >&2
  exit 2
fi

META=$(cargo metadata --format-version 1 --no-deps)

expected() {
  case "$1" in
    stride-core)    printf '';;
    stride-ingest)  printf 'stride-core';;
    stride-store)   printf 'stride-core';;
    stride-api)     printf 'stride-core stride-ingest stride-store';;
    stride-testkit) printf '';;
    stride-desktop) printf 'stride-api stride-store';;
    *) echo "check-deps: unknown workspace member $1" >&2; exit 2;;
  esac
}

fail=0

# The workspace must contain exactly the six planned members. The seventh —
# android/…/stride-ffi — is Slice 1+, so its appearance here is a violation.
members=$(printf '%s' "$META" | jq -r '[.packages[].name] | sort | join(" ")')
expected_members="stride-api stride-core stride-desktop stride-ingest stride-store stride-testkit"
if [[ "$members" != "$expected_members" ]]; then
  echo "check-deps: workspace members [$members]; expected [$expected_members]" >&2
  fail=1
fi

# Internal dependency names of one package, by kind.
#   $1 package name, $2 kind selector: "" (normal), "dev", "build"
internal_deps() {
  printf '%s' "$META" | jq -r --arg n "$1" --arg k "$2" '
    [ .packages[].name ] as $members
    | [ .packages[] | select(.name == $n) | .dependencies[] ]
    | map(select(.name as $d | $members | index($d)))
    | map(select(if $k == "" then (.kind == null) else (.kind == $k) end))
    | map(.name) | sort | unique | join(" ")
  '
}

for name in stride-core stride-ingest stride-store stride-api stride-testkit stride-desktop; do
  actual=$(internal_deps "$name" "")
  exp=$(expected "$name")
  if [[ "$actual" != "$exp" ]]; then
    echo "check-deps: $name depends on [$actual]; expected [$exp]" >&2
    fail=1
  fi

  # A build-dependency is a real edge too: it is compiled into the build
  # script, and it can create a cycle that cargo would not refuse.
  build=$(internal_deps "$name" "build")
  for dep in $build; do
    case " $exp " in
      *" $dep "*) ;;
      *) echo "check-deps: $name has a build-dependency on [$dep]; not in its allowed set [$exp]" >&2; fail=1;;
    esac
  done

  # A dev-dependency may add only what the package may already depend on, or
  # the dev-only testkit crate.
  dev=$(internal_deps "$name" "dev")
  for dep in $dev; do
    case " $exp $build " in
      *" $dep "*) ;;
      *) if [[ "$dep" != "stride-testkit" ]]; then
           echo "check-deps: $name has a dev-dependency on [$dep]; not in its allowed set [$exp] and not the dev-only testkit" >&2
           fail=1
         fi;;
    esac
  done

  # stride-testkit must never be a normal (non-dev) dependency.
  if printf '%s' "$META" | jq -e --arg n "$name" '
    .packages[] | select(.name == $n)
    | any(.dependencies[]; .kind == null and .name == "stride-testkit")
  ' >/dev/null; then
    echo "check-deps: $name lists stride-testkit as a hard dependency (dev-only crate)" >&2
    fail=1
  fi
done

# No crate may override the workspace lint table (`[lints] workspace = true`
# inherits; `[lints.clippy]` / `[lints.rust]` would override).
for manifest in crates/*/Cargo.toml desktop/Cargo.toml; do
  if grep -qE '^\[lints\.(clippy|rust)\]' "$manifest"; then
    echo "check-deps: $manifest overrides the workspace lint table" >&2
    fail=1
  fi
done

exit "$fail"
