#!/usr/bin/env bash
# The bundle check: the rig carries the app, and it must be the same app.
#
# `desktop/assets/` is the bundle the rig embeds, and `desktop/ASSET-BUNDLE.md`
# says the rule: **when the frontend changes, rebuild it and replace those
# files in the same change**. This check holds the tree to it.
#
#     ./scripts/check-desktop-bundle.sh
#
# It builds the frontend, writes the bundle the way the sync script writes
# it — the same rewrite, in the same place — and compares it with the one in
# the tree. A bundle that is one commit behind is a rig that serves an app
# that is not the one in the source, and this is where that is caught.
#
# The build is deterministic: the same source produces the same files, with
# the same names, so the comparison is a comparison and not a guess.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
target="$root/desktop/assets"

if [[ ! -d "$root/frontend/node_modules" ]]; then
  echo "check-desktop-bundle: installing the app's dependencies (npm --prefix frontend ci)"
  npm --prefix frontend ci
fi

echo "check-desktop-bundle: building the frontend"
(cd "$root/frontend" && npm run build >/dev/null)

staged="$(mktemp -d)"
trap 'rm -rf "$staged"' EXIT
bash "$here/sync-desktop-bundle.sh" "$root/frontend/dist" "$staged" >/dev/null

if [[ ! -f "$target/index.html" ]]; then
  echo "check-desktop-bundle: there is no bundle at $target" >&2
  exit 1
fi

if ! diff -r "$staged" "$target" >/dev/null; then
  echo "check-desktop-bundle: the bundle in desktop/assets is not the build of frontend/." >&2
  echo "check-desktop-bundle: run ./scripts/sync-desktop-bundle.sh and commit what it writes." >&2
  diff -rq "$staged" "$target" >&2 || true
  exit 1
fi

echo "check-desktop-bundle: the rig carries the app in frontend/ — the bundle is in step"
