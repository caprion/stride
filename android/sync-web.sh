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
#
# **The bridge shim is kept.** `stride-bridge.js` is not part of the web
# bundle: it is the shell's half of the seam between the app and the Rust
# core, and Capacitor injects it into the page it serves — before the app's
# own scripts, so the app's first request finds the bridge. The bundle
# replaces everything else in the directory; the shim is put back.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
source="${1:-${STRIDE_WEB_DIST:-$here/../frontend/dist}}"
target="$here/app/src/main/assets/public"
shim="stride-bridge.js"

if [[ ! -d "$source" ]]; then
  echo "sync-web: there is no bundle at $source" >&2
  echo "sync-web: build the frontend first, or pass the directory as the first argument." >&2
  exit 1
fi

if [[ ! -f "$source/index.html" ]]; then
  echo "sync-web: $source has no index.html, so it is not a web bundle." >&2
  exit 1
fi

if [[ ! -f "$target/$shim" ]]; then
  echo "sync-web: the bridge shim is missing from $target" >&2
  echo "sync-web: it is the shell's, and it is committed. Restore it before building." >&2
  exit 1
fi

echo "sync-web: copying $source into $target"
mkdir -p "$target"

# The shim is put aside: the bundle replaces everything else in the
# directory, and a stale file from the last build would ship beside it.
kept="$(mktemp)"
cp "$target/$shim" "$kept"

# The placeholder is replaced, not merged with: a bundle carries its own
# asset names and hashes, and a stale file from the last build would ship
# beside it.
#
# **`rm -rf`, not `find -delete`.** A bundle's assets live in a directory,
# and `find -delete` cannot delete a directory that is not empty — so the
# second sync of a checkout failed on the directory the first one left
# behind, and the APK carried the placeholder instead of the bundle.
find "$target" -mindepth 1 -maxdepth 1 -not -name index.html -exec rm -rf {} +
rm -f "$target/index.html"
cp -R "$source/." "$target/"

# The shim goes back, whatever the bundle brought with it.
cp "$kept" "$target/$shim"
chmod 644 "$target/$shim"
rm -f "$kept"

echo "sync-web: done"
