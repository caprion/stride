#!/usr/bin/env bash
# Build the Android artifacts from this branch, under the host-wide heavy lock.
#
#   ./scripts/apk_smoke_build.sh              the release APK (arm64), the
#                                             Kotlin unit tests and the
#                                             emulator variant (x86_64, debug)
#   ./scripts/apk_smoke_build.sh --release    the release APK only
#
# The steps are the ones CI runs (.github/workflows/android.yml): the web
# bundle is built by the caller, `cap sync` has staged it into the Android
# project, and this script runs the Gradle build.
#
# The Gradle build cross-compiles the Rust core for the ABI with cargo-ndk,
# so it is a heavy job and takes the same host-wide lock the Rust gate
# takes.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ALL=1
for argument in "$@"; do
  case "$argument" in
    --release) ALL=0 ;;
    *) echo "unknown argument: $argument" >&2; exit 2 ;;
  esac
done

SDK="${ANDROID_SDK_ROOT:-/home/sumit/android-sdk}"
NDK_VERSION="$(ls "$SDK/ndk" | sort -V | tail -1)"
NDK="$SDK/ndk/$NDK_VERSION"

printf 'sdk.dir=%s\n' "$SDK" > android/local.properties
printf 'ndk.dir=%s\n' "$NDK" >> android/local.properties

export ANDROID_HOME="$SDK"
export ANDROID_SDK_ROOT="$SDK"
export ANDROID_NDK_HOME="$NDK"
export ANDROID_NDK_ROOT="$NDK"
export CARGO_NDK_HOME="$NDK"
export CARGO_BUILD_JOBS=1

cd android

echo "=== the release APK ==="
./gradlew assembleRelease || exit 1

if [[ "$ALL" -eq 1 ]]; then
  echo "=== the Kotlin unit tests ==="
  ./gradlew testDebugUnitTest || exit 1

  echo "=== the emulator variant (x86_64, debug) ==="
  # The release build is arm64 only, and the emulator here is x86_64, so
  # the variant that runs on it is a separate build of the same branch.
  ./gradlew assembleDebug -PstrideEmulatorAbi=x86_64 || exit 1
fi
