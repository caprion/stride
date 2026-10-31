#!/usr/bin/env bash
# Put the Capacitor Android library where the Gradle build looks for it.
#
# **Capacitor's Android library is an npm package, not a Maven artifact.**
# `@capacitor/android` ships the Gradle project that builds
# `com.getcapacitor.android` — the bridge the app talks through — and the
# only copies of it are the ones npm puts on disk.
#
# The library is fetched into `android/.capacitor/`, which is git-ignored:
# it is a dependency, not a source file of this repository. A build of the
# whole repository that has run `npm install` at the root already has a copy
# in `node_modules/`, and the build finds that one first.
#
#     ./fetch-capacitor.sh
#
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
prefix="$here/.capacitor"

version="${CAPACITOR_ANDROID_VERSION:-^8.0.0}"

echo "fetch-capacitor: installing @capacitor/android@$version into $prefix"
mkdir -p "$prefix"
npm install --prefix "$prefix" --no-audit --no-fund "@capacitor/android@$version"

library="$prefix/node_modules/@capacitor/android/capacitor"
if [[ ! -f "$library/build.gradle" ]]; then
  echo "fetch-capacitor: the library is not at $library" >&2
  exit 1
fi
echo "fetch-capacitor: ready at $library"
