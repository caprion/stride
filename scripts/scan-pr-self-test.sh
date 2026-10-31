#!/usr/bin/env bash
# The self-test for scripts/scan-pr.sh.
#
# A scanner that never fails is worse than no scanner. This proves the
# pull-request scanner catches what it claims to catch, and that it does not
# block an ordinary change.
#
# It builds a throwaway git repository in a temp directory, makes one commit
# with a credential, a coordinate pair, a Garmin profile value and a free-text
# note, and one ordinary commit — and asserts the scanner fails the first and
# passes the second.
set -uo pipefail

cd "$(dirname "$0")/.."
SCANNER="$PWD/scripts/scan-pr.sh"

TMP=$(mktemp -d "${TMPDIR:-/tmp}/stride-scan-pr-selftest.XXXXXX")
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

fail() { echo "scan-pr self-test: $1" >&2; exit 1; }

cd "$TMP"
git init -q .
git config user.email "selftest@example.invalid"
git config user.name "selftest"
mkdir -p scripts
cp "$SCANNER" scripts/scan-pr.sh

printf '# scratch\n' > README.md
git add -A
git commit -qm base
BASE=$(git rev-parse HEAD)

# --- the bad change ------------------------------------------------------
printf 'GCDC_0123456789abcdefghijklmnop\n' > key.txt
printf '{"lat": 15.0000, "lon": 78.0000}\n' > coords.json
printf '{"displayName": "A Real Person"}\n' > profile.json
printf '{"userProfileId": 12345678}\n' > profile-id.json
printf 'token = "abcdef1234567890abcdef"\n' > config.toml
printf '{"note": "a private note about my run"}\n' > note.json
git add -A
git commit -qm leak
LEAK=$(git rev-parse HEAD)

if bash scripts/scan-pr.sh "$BASE" "$LEAK" >/dev/null 2>&1; then
  fail "the scanner passed a change that adds a credential, a coordinate pair, a Garmin id and a note"
fi

# Each of those must be reported by name, not merely counted.
out=$(bash scripts/scan-pr.sh "$BASE" "$LEAK" 2>&1)
for needle in "credential-shaped" "personal data" "free-text note"; do
  printf '%s' "$out" | grep -q "$needle" || fail "the scanner did not report '$needle'"
done

# --- the good change -----------------------------------------------------
printf 'pub fn add(a: i32, b: i32) -> i32 { a + b }\n' > lib.rs
printf '{"access_token": "placeholder"}\n' > placeholder.json
git add -A
git commit -qm good
GOOD=$(git rev-parse HEAD)

if ! bash scripts/scan-pr.sh "$LEAK" "$GOOD" >/dev/null 2>&1; then
  fail "the scanner blocked an ordinary change"
fi

# A placeholder value must not be reported as a credential.
out=$(bash scripts/scan-pr.sh "$LEAK" "$GOOD" 2>&1)
printf '%s' "$out" | grep -q "credential-named field" \
  && fail "a placeholder value was reported as a credential"

echo "scan-pr self-test: passed"
exit 0
