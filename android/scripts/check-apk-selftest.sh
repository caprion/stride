#!/usr/bin/env bash
# The checker's own self-test.
#
# **A checker that never fails is worse than no checker.** This proves that
# `scripts/check_apk.py` catches a certificate that is not the published
# one, and passes the published one — with a *throwaway* key that is made
# here, used here, and thrown away. It never signs a published artefact:
# the key lives in a temporary directory that is deleted when the script
# ends, and the fingerprint it publishes is written to a temporary file
# that goes with it.
#
# The published certificate in `android/assetlinks.json` is not touched.
#
#     bash android/scripts/check-apk-selftest.sh
#
# It needs `keytool` (the JDK), `apksigner` (the Android SDK's
# build-tools) and `python3`.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
checker="$root/scripts/check_apk.py"

if [[ ! -f "$checker" ]]; then
  echo "selftest: there is no checker at $checker" >&2
  exit 1
fi

# The throwaway key, the throwaway APK and the throwaway fingerprint all
# live here, and all of it goes when the script ends.
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# `apksigner` is in the SDK's build-tools. It is not on the PATH by
# default, so it is looked for.
apksigner=""
for candidate in "${ANDROID_HOME:-}"/build-tools/*/apksigner \
                 "${ANDROID_SDK_ROOT:-}"/build-tools/*/apksigner \
                 /usr/local/lib/android/sdk/build-tools/*/apksigner \
                 "${HOME}/android-sdk"/build-tools/*/apksigner; do
  if [[ -x "$candidate" ]]; then
    apksigner="$candidate"
    break
  fi
done
if [[ -z "$apksigner" ]]; then
  echo "selftest: apksigner was not found; the Android SDK's build-tools are needed" >&2
  exit 2
fi

echo "selftest: using $apksigner"

# The throwaway key.
keytool -genkeypair \
  -keystore "$work/throwaway.jks" \
  -storepass throwaway -keypass throwaway \
  -alias throwaway \
  -keyalg RSA -keysize 2048 -validity 1 \
  -dname "CN=Stride selftest, OU=selftest, O=selftest, L=-, ST=-, C=ZZ" \
  >/dev/null 2>&1

# The fingerprint of the throwaway key, in the form the published file
# uses: colon-separated, upper case.
fingerprint="$(keytool -list -v \
  -keystore "$work/throwaway.jks" -storepass throwaway -alias throwaway \
  | grep -o 'SHA256: .*' | head -1 | cut -d' ' -f2 | tr -d ':' | tr 'a-f' 'A-F')"

if [[ -z "$fingerprint" ]]; then
  echo "selftest: the throwaway key has no SHA256 fingerprint" >&2
  exit 1
fi
echo "selftest: the throwaway key's fingerprint is $fingerprint"

# A minimal "APK": a zip with a manifest, so apksigner will sign it. The
# zip is written with python — the checker needs python anyway, and `zip`
# is not on every machine.
mkdir -p "$work/apk"
printf '<manifest package="app.stride.running"/>\n' > "$work/apk/AndroidManifest.xml"
python3 - "$work/apk" "$work/throwaway.apk" <<'PY'
import os
import sys
import zipfile

source, target = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(target, "w") as archive:
    for name in sorted(os.listdir(source)):
        archive.write(os.path.join(source, name), name)
PY

# Signed with the throwaway key. The signing block is what the checker
# reads.
#
# `--min-sdk-version` is given because the synthetic manifest is text, not
# the binary form apksigner reads; the flag tells it the version instead of
# making it parse one.
"$apksigner" sign \
  --ks "$work/throwaway.jks" \
  --ks-pass pass:throwaway \
  --key-pass pass:throwaway \
  --ks-key-alias throwaway \
  --min-sdk-version 35 \
  --out "$work/signed.apk" \
  "$work/throwaway.apk" >/dev/null 2>&1

# The throwaway's own published certificate — the shape of the real one,
# with the throwaway's fingerprint in it.
published() {
  cat <<EOF
[
  {
    "relation": ["delegate_permission/common.handle_all_urls"],
    "target": {
      "namespace": "android_app",
      "package_name": "app.stride.running",
      "sha256_cert_fingerprints": ["$1"]
    }
  }
]
EOF
}

published "$fingerprint" > "$work/published.json"
published "0000000000000000000000000000000000000000000000000000000000000000" \
  > "$work/other.json"

fail() {
  echo "selftest: $1" >&2
  exit 1
}

# 1. The published certificate matches: the check passes.
if ! python3 "$checker" "$work/signed.apk" \
      --signature-only --fingerprints "$work/published.json" >/dev/null 2>&1; then
  fail "the checker refused an APK signed with the published certificate"
fi
echo "selftest: the published certificate passes"

# 2. The published certificate does not match: the check fails.
if python3 "$checker" "$work/signed.apk" \
     --signature-only --fingerprints "$work/other.json" >/dev/null 2>&1; then
  fail "the checker passed an APK signed with a certificate that is not published"
fi
echo "selftest: a certificate that is not published fails"

# 3. Nothing published at all: the check fails. An APK signed with
#    anything is not the app, and "nothing is published" is not a pass.
: > "$work/empty.json"
if python3 "$checker" "$work/signed.apk" \
     --signature-only --fingerprints "$work/empty.json" >/dev/null 2>&1; then
  fail "the checker passed an APK with nothing published to compare with"
fi
echo "selftest: nothing published fails"

# 4. An unsigned APK: the check fails unless it is told otherwise.
if python3 "$checker" "$work/throwaway.apk" \
     --signature-only --fingerprints "$work/published.json" >/dev/null 2>&1; then
  fail "the checker passed an APK with no signature"
fi
echo "selftest: an unsigned APK fails"

echo "selftest: the checker catches what it must catch"
