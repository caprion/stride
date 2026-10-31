#!/usr/bin/env bash
# Copy the web bundle into the APK's assets.
#
# The WebView loads the **bundled** app — `file:///android_asset/public` —
# and the phone talks to no server. The bundle is built by the frontend; this
# script puts it where the APK picks it up.
#
#     ./sync-web.sh [the build directory]
#
# The directory may also be given as `STRIDE_WEB_DIST`. The default is the
# frontend's own build output at the repository root.
#
# The placeholder page that is committed is replaced by the bundle. It stays
# in the repository because an APK without a bundle should say so rather than
# show a blank WebView.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
source="${1:-${STRIDE_WEB_DIST:-$here/../frontend/dist}}"
target="$here/app/src/main/assets/public"

if [[ ! -d "$source" ]]; then
  echo "sync-web: there is no bundle at $source" >&2
  echo "sync-web: build the frontend first, or pass the directory as the first argument." >&2
  exit 1
fi

if [[ ! -f "$source/index.html" ]]; then
  echo "sync-web: $source has no index.html, so it is not a web bundle." >&2
  exit 1
fi

echo "sync-web: copying $source into $target"
mkdir -p "$target"
# The placeholder is replaced, not merged with: a bundle carries its own
# asset names and hashes, and a stale file from the last build would ship
# beside it.
find "$target" -mindepth 1 -maxdepth 1 -not -name index.html -delete
rm -f "$target/index.html"
cp -R "$source/." "$target/"

echo "sync-web: done"
