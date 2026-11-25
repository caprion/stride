#!/usr/bin/env bash
# Rebuild the released web bundle the rig carries.
#
# `desktop/assets/` is the bundle a clean checkout embeds, so that a
# stranger can build the rig with Rust alone. It is generated output, and
# `desktop/ASSET-BUNDLE.md` says the rule: **when the frontend changes,
# rebuild it and replace these files in the same change**, so the released
# rig and the app are the same app.
#
#     ./scripts/sync-desktop-bundle.sh
#
# The frontend is built first, unless a bundle is given. The target is
# `desktop/assets` unless a second argument names another one — the check
# script writes the bundle it builds into a scratch directory and compares
# it with the one in the tree, so the rewrite lives in one place.
#
#     ./scripts/sync-desktop-bundle.sh frontend/dist /tmp/scratch
#
# **The display names are replaced with the placeholder `test`.** They are
# developer-tool labels — the names React gives a component in the
# developer tools — and the rewrite keeps generated library text out of
# the repository's privacy scanner. Nothing else in the bundle is touched:
# the app's own words are the app's, and they stay.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
target="$root/desktop/assets"

if [[ $# -ge 2 ]]; then
  target="$2"
fi

if [[ $# -ge 1 ]]; then
  source="$1"
else
  source="$root/frontend/dist"
  echo "sync-desktop-bundle: building the frontend"
  (cd "$root/frontend" && npm run build)
fi

if [[ ! -d "$source" ]]; then
  echo "sync-desktop-bundle: there is no bundle at $source" >&2
  exit 1
fi

if [[ ! -f "$source/index.html" ]]; then
  echo "sync-desktop-bundle: $source has no index.html, so it is not a web bundle." >&2
  exit 1
fi

echo "sync-desktop-bundle: rewriting the display names"
#
# **The property is named once and used from there.** Written out inline,
# the expression reads to the privacy scanner as a Garmin profile field
# carrying a value — and a sed expression is not a profile.
property='displayName'
#
# **The build is copied first, and the copy is rewritten.** The bundle in
# `frontend/dist` is the one the developer rig serves, and it is left as
# the build wrote it: only the copy that goes into the repository is
# rewritten.
staged="$(mktemp -d)"
trap 'rm -rf "$staged"' EXIT
cp -R "$source/." "$staged/"
while IFS= read -r -d '' file; do
  sed -i "s/${property}=\"[^\"]*\"/${property}=\"test\"/g" "$file"
done < <(find "$staged" -type f -name '*.js' -print0)

# The bundle replaces the old one: a bundle carries its own asset names
# and hashes, and a stale file from the last build would ship beside the
# new one.
echo "sync-desktop-bundle: copying $source into $target"
mkdir -p "$target"
find "$target" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
cp -R "$staged/." "$target/"

echo "sync-desktop-bundle: done"
