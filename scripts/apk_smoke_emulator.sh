#!/usr/bin/env bash
# The emulator smoke test: install the APK, open it, and look at it.
#
#   ./scripts/apk_smoke_emulator.sh [avd] [apk]
#
# **KVM.** The emulator needs /dev/kvm, and the group that owns it has to be
# in the process's group list — `sg kvm -c` does that on a host where the
# user is a member of the group but the session does not carry it. Without
# it the emulator falls back to software emulation, which is where the
# "the UI kept ANR-ing" reports come from.
#
# The screen is set to 390 CSS pixels wide (1080 physical pixels at density
# 443), so what the emulator draws is the phone's width and not a tablet's.
#
# The script is headless and it leaves the emulator running at the end, so
# the screenshots can be taken again without booting it twice.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SDK="${ANDROID_SDK_ROOT:-/home/sumit/android-sdk}"
AVD="${1:-stride-x86}"
APK="${2:-android/app/build/outputs/apk/debug/app-debug.apk}"
OUT="$ROOT/.run/emulator"
BOOT_TIMEOUT="${BOOT_TIMEOUT:-420}"
DENSITY="${DENSITY:-443}"

mkdir -p "$OUT"

# The emulator, with the KVM group in the process's group list.
start_emulator() {
  if pgrep -f "emulator.*-avd $AVD" > /dev/null; then
    echo "the emulator is already running"
    return 0
  fi
  echo "=== booting $AVD ==="
  sg kvm -c "$SDK/emulator/emulator -avd $AVD -no-window -no-audio -no-boot-anim \
    -gpu swiftshader_indirect -no-snapshot-save -memory 2048 -cores 2 \
    -netdelay none -netspeed full" > "$OUT/emulator.log" 2>&1 &
  sleep 3
}

start_emulator

echo "=== waiting for the device ==="
adb wait-for-device || { echo "the device never appeared"; tail -20 "$OUT/emulator.log"; exit 1; }

deadline=$(( $(date +%s) + BOOT_TIMEOUT ))
booted=no
while [[ $(date +%s) -lt $deadline ]]; do
  if [[ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]; then
    booted=yes
    break
  fi
  sleep 5
done

if [[ "$booted" != "yes" ]]; then
  echo "the device did not finish booting in ${BOOT_TIMEOUT}s"
  adb shell getprop sys.boot_completed 2>&1 | head -3
  tail -30 "$OUT/emulator.log"
  exit 1
fi
echo "=== booted in ${BOOT_TIMEOUT}s at most ==="

adb shell settings put global window_animation_scale 0 || true
adb shell settings put global transition_animation_scale 0 || true
adb shell settings put global animator_duration_scale 0 || true
# 1080 physical pixels at density 443 is 390 CSS pixels: the phone's width.
adb shell wm density "$DENSITY" || true
adb shell wm size || true

echo "=== installing $APK ==="
adb install -r "$APK" || exit 1

echo "=== opening the app ==="
adb logcat -c || true
adb shell am start -n app.stride.running/.MainActivity || exit 1
sleep 12

echo "=== what the log says ==="
adb logcat -d -s Stride:V StrideCore:V StrideSync:V AndroidRuntime:E | tail -40

echo "=== the screen ==="
adb exec-out screencap -p > "$OUT/launch.png"
echo "$OUT/launch.png"
