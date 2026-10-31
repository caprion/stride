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
#
# The bad values are *assembled* here rather than written literally: this file
# is tracked, and the tree scanners scan every tracked file. A test fixture
# that has to be excluded from the scanners is a hole in the scanners, so
# there is no exclusion — the literals are built at runtime instead.
set -uo pipefail

cd "$(dirname "$0")/.."
SCANNER="$PWD/scripts/scan-pr.sh"
RULES="$PWD/scripts/privacy-rules.sh"
DEPS_LIST="$PWD/scripts/approved-deps.txt"

TMP=$(mktemp -d "${TMPDIR:-/tmp}/stride-scan-pr-selftest.XXXXXX")
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

fail() { echo "scan-pr self-test: $1" >&2; exit 1; }

cd "$TMP"
git init -q .
git config user.email "selftest"
git config user.name "selftest"
mkdir -p scripts
cp "$SCANNER" scripts/scan-pr.sh
# The scanner sources the shared privacy rules; the scratch repo needs them too.
cp "$RULES" scripts/privacy-rules.sh
cp "$DEPS_LIST" scripts/approved-deps.txt

printf '# scratch\n' > README.md
git add -A
git commit -qm base
BASE=$(git rev-parse HEAD)

# --- the bad change ------------------------------------------------------
# Assembled at runtime, so no credential-shaped literal exists in this file.
# The variable names are deliberately neutral: a name like TOKEN_VALUE would
# itself look like a credential to the key-name rule.
S1=$(printf 'GC%s' 'DC_0123456789abcdefghijklmnop')
S2=$(printf 'abcdef%s' '1234567890abcdef')
S3=$(printf '15.%s' '0000')
S4=$(printf '78.%s' '0000')
S5=$(printf 'A Real %s' 'Person')
S6=$(printf 'athlete@%s' 'example.invalid')
S7='a private note about my run'

printf '%s\n' "$S1" > key.txt
printf '{"lat": %s, "lon": %s}\n' "$S3" "$S4" > coords.json
printf '{"%s": "%s"}\n' 'displayName' "$S5" > profile.json
printf '%s = "%s"\n' 'token' "$S2" > config.toml
printf '{"%s": "%s"}\n' 'note' "$S7" > note.json
git add -A
git commit -qm leak
LEAK=$(git rev-parse HEAD)

if bash scripts/scan-pr.sh "$BASE" "$LEAK" >/dev/null 2>&1; then
  fail "the scanner passed a change that adds a credential, a coordinate pair, a Garmin id and a note"
fi

# Each of those must be reported by name, not merely counted.
out=$(bash scripts/scan-pr.sh "$BASE" "$LEAK" 2>&1)
for needle in "credential-shaped" "credential-named field" "coordinate value" "free-text note"; do
  printf '%s' "$out" | grep -q "$needle" || fail "the scanner did not report '$needle'"
done

# --- the good change -----------------------------------------------------
printf 'pub fn add(a: i32, b: i32) -> i32 { a + b }\n' > lib.rs
printf '{"access_token": "placeholder"}\n' > placeholder.json
# A lockfile checksum is full of digit runs and is not a phone number.
printf 'checksum = "%s"\n' "d78c8dee4c7bf0e14673097256fed6142ce9d3b85a408189d07482442145823b" > Cargo.lock
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

# A lockfile checksum must not be reported as personal data.
out=$(bash scripts/scan-pr.sh "$LEAK" "$GOOD" 2>&1)
printf '%s' "$out" | grep -q "coordinate value" \
  && fail "a Cargo.lock checksum was reported as a coordinate"

# --- a dependency outside the approved list -------------------------------
printf '[dependencies]\nrand = "0.8"\n' > Cargo.toml
git add -A
git commit -qm deps
DEPS=$(git rev-parse HEAD)
if bash scripts/scan-pr.sh "$GOOD" "$DEPS" >/dev/null 2>&1; then
  fail "the scanner passed a dependency that is not on the approved list"
fi
out=$(bash scripts/scan-pr.sh "$GOOD" "$DEPS" 2>&1)
printf '%s' "$out" | grep -q "not on the approved list" \
  || fail "the scanner did not report the unapproved dependency"

# An approved dependency must pass.
printf '[dependencies]\nserde = "1"\n' > Cargo.toml
git add -A
git commit -qm approved
APPROVED=$(git rev-parse HEAD)
bash scripts/scan-pr.sh "$DEPS" "$APPROVED" >/dev/null 2>&1 \
  || fail "the scanner blocked an approved dependency"

echo "scan-pr self-test: passed"
exit 0
