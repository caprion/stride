#!/usr/bin/env bash
# scan-pr.sh — the pull-request gate for data and secrets.
#
# The two scanners in this directory look at the *tree*. This one looks at the
# *change*: only the lines a pull request adds, and only the files it touches.
# A leak is introduced by a commit, so the commit is where it has to be caught.
#
# It answers four questions about a proposed change:
#
#   1. Does it add a credential, a token, a key or a password?
#   2. Does it add personal data — a coordinate, a Garmin profile value, an
#      email address, a phone number, a name, a free-text note?
#   3. Does it add a new dependency without saying why?
#   4. Does it add a binary blob, or a data file, without provenance?
#
# Usage:
#   scripts/scan-pr.sh <base> <head>     # e.g. scripts/scan-pr.sh origin/master HEAD
#   scripts/scan-pr.sh                   # defaults to origin/master...HEAD
#
# Fork-safe: it reads the diff, never a secret, and writes nothing.
set -uo pipefail

cd "$(dirname "$0")/.."

BASE="${1:-origin/master}"
HEAD="${2:-HEAD}"

if ! git rev-parse --verify "$BASE" >/dev/null 2>&1; then
  echo "scan-pr: base '$BASE' is not available (fetch it first)" >&2
  exit 2
fi

DIFF=$(git diff --unified=0 "$BASE...$HEAD" 2>/dev/null)
if [[ -z "$DIFF" ]]; then
  echo "scan-pr: no changes between $BASE and $HEAD"
  exit 0
fi

# Only added lines, with their file path.
ADDED=$(printf '%s\n' "$DIFF" | awk '
  /^\+\+\+ b\// { file = substr($0, 7); next }
  /^\+/ && !/^\+\+\+/ { printf "%s\t%s\n", file, substr($0, 2) }
')

FILES_TOUCHED=$(printf '%s\n' "$DIFF" | awk '/^\+\+\+ b\// { print substr($0, 7) }' | sort -u)

found=0
fail() { echo "scan-pr: $1" >&2; found=1; }

# ---------------------------------------------------------------- 1. secrets
# The key-name rule: any of these names, with a non-placeholder value.
SECRET_KEY_RE='(api[_-]?key|secret|password|passwd|token|credential|authorization|private[_-]?key|consumer[_-]?secret|client[_-]?secret|refresh[_-]?token|access[_-]?token|cookie|session[_-]?id)'
# A value long enough to be a credential.
SECRET_VALUE_RE='[:=]\s*["'"'"']?[A-Za-z0-9/+_.-]{12,}'
# A value that is obviously a placeholder. Anchored to the whole value — a
# substring search would let "0" inside a real token excuse it.
PLACEHOLDER_RE='^(placeholder|example|sample|redacted|your[_-]?key|<[^>]+>|none|null|nil|false|true|0|test|testing|fake|dummy|changeme|change_me|todo|fixme|x+|x{3,}|\*+|n/?a|redacted|hidden|secret|pass|token|key|value)+$'

# Literal shapes that are credentials wherever they appear.
LITERAL_PATTERNS=(
  'GCDC_[A-Za-z0-9._-]{12,}'
  'gh[pousr]_[A-Za-z0-9]{16,}'
  'github_pat_[A-Za-z0-9_]{20,}'
  'xox[baprs]-[A-Za-z0-9-]{10,}'
  'sk-[A-Za-z0-9]{20,}'
  'AIza[0-9A-Za-z_-]{30,}'
  'AKIA[0-9A-Z]{16}'
  '-----BEGIN [A-Z ]*PRIVATE KEY-----'
  'eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}'
)

while IFS= read -r pat; do
  hits=$(printf '%s\n' "$ADDED" | grep -Pn -- "$pat" | head -5)
  if [[ -n "$hits" ]]; then
    fail "a credential-shaped string was added:"
    printf '%s\n' "$hits" | cut -f1 | while IFS= read -r h; do
      echo "    ${h}" >&2
    done
  fi
done < <(printf '%s\n' "${LITERAL_PATTERNS[@]}")

# The key-name rule, applied to JSON/TOML/env/YAML-ish lines.
while IFS= read -r line; do
  file="${line%%$'\t'*}"
  text="${line#*$'\t'}"
  name=$(printf '%s' "$text" | grep -oiP '(?<=")[A-Za-z0-9_.-]+(?="\s*:)|^[A-Za-z0-9_.-]+(?=\s*=)' | head -1)
  [[ -n "$name" ]] || continue
  printf '%s' "$name" | grep -qiP -- "$SECRET_KEY_RE" || continue
  printf '%s' "$text" | grep -qP -- "$SECRET_VALUE_RE" || continue
  value=$(printf '%s' "$text" | grep -oP '[:=]\s*["'"'"']?[A-Za-z0-9/+_.-]{8,}' | head -1 \
    | sed -E 's/^[:=]\s*["'"'"']?//')
  printf '%s' "$value" | grep -qiP -- "$PLACEHOLDER_RE" && continue
  fail "a credential-named field with a non-placeholder value was added — $file"
  echo "    ${text:0:160}" >&2
done < <(printf '%s\n' "$ADDED")

# ------------------------------------------------------------- 2. personal data
BAND_LAT='(?<![-0-9.])1[4-6]\.[0-9]{2,}'
BAND_LON='(?<![-0-9.])7[6-8]\.[0-9]{2,}'
PERSONAL=(
  "$BAND_LAT.*$BAND_LON"
  "$BAND_LON.*$BAND_LAT"
  'displayName"?[[:space:]]*:[[:space:]]*"(?!ATHLETE|athlete)[^"]{2,}"'
  'fullName"?[[:space:]]*:[[:space:]]*"(?!ATHLETE)[^"]{2,}"'
  'userProfileId"?[[:space:]]*:[[:space:]]*[0-9]{4,}'
  'userName"?[[:space:]]*:[[:space:]]*"(?!athlete)[^"]{2,}"'
  '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
  # A phone number only when it is actually a phone number: an international
  # prefix, or a key that says so. A bare run of digits is not a phone number -
  # Cargo.lock checksums are full of them.
  '(\+[0-9]{1,3}[- ]?[0-9]{4,}[- ]?[0-9]{4,}|(phone|mobile|tel|telephone|phone_number|mobile_number)"?[[:space:]]*[:=][[:space:]]*"?[0-9]{6,})'
)

# The personal-data rules do not apply to the generated lockfile: a checksum
# contains long digit runs, and a package version is not personal data. The
# secret rules DO apply to it - a registry URL can carry a token.
PERSONAL_SCOPE=$(printf '%s\n' "$ADDED" | grep -v $'\t'"Cargo.lock"$'\t' | grep -vP '\tCargo\.lock\t')

while IFS= read -r pat; do
  hits=$(printf '%s\n' "$PERSONAL_SCOPE" | grep -Pn -- "$pat" | head -5)
  if [[ -n "$hits" ]]; then
    fail "personal data may have been added:"
    printf '%s\n' "$hits" | cut -f1 | while IFS= read -r h; do echo "    ${h}" >&2; done
  fi
done < <(printf '%s\n' "${PERSONAL[@]}")

# A free-text note field, if it is not one of the documented placeholders.
if printf '%s\n' "$PERSONAL_SCOPE" | grep -qiP -- '"(note|notes|comment|remark)"[[:space:]]*:[[:space:]]*"[^"]{4,}"'; then
  fail "a free-text note was added — the only free text the product stores is the athlete's own; if this is a fixture, use a placeholder"
  printf '%s\n' "$PERSONAL_SCOPE" | grep -inP -- '"(note|notes|comment|remark)"[[:space:]]*:[[:space:]]*"[^"]{4,}"' | head -3 | cut -f1 | while IFS= read -r h; do echo "    ${h}" >&2; done
fi

# ------------------------------------------------------------ 3. dependencies
NEW_DEPS=$(printf '%s\n' "$FILES_TOUCHED" | grep -E '(^|/)Cargo\.toml$|package(-lock)?\.json$' || true)
if [[ -n "$NEW_DEPS" ]]; then
  added_dep=$(git diff --unified=0 "$BASE...$HEAD" -- $NEW_DEPS \
    | grep -E '^\+' | grep -vE '^\+\+\+' \
    | grep -E '^(\+[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*|[[:space:]]*"[A-Za-z@/.-]+"[[:space:]]*:)' \
    | grep -vE 'stride-|path[[:space:]]*=' | head -20)
  if [[ -n "$added_dep" ]]; then
    echo "scan-pr: dependencies added by this change (each needs a justification in the PR description):" >&2
    printf '%s\n' "$added_dep" | sed 's/^/    /' >&2
  fi
fi

# ------------------------------------------------------------- 4. provenance
for f in $FILES_TOUCHED; do
  case "$f" in
    fixtures/*)
      if [[ ! "$f" =~ ^fixtures/(scan-tests/|README\.md$) ]] \
         && ! printf '%s\n' "$FILES_TOUCHED" | grep -q '^fixtures/README\.md$'; then
        fail "a corpus file was added or changed ($f) without updating fixtures/README.md — the provenance and the sanitisation log must travel with it"
      fi
      ;;
  esac
done

# A new binary blob, or a big new file, in a place that is not the corpus.
BIG=$(git diff --numstat "$BASE...$HEAD" | awk '$1 == "-" || ($1+0) > 20000 { print $3 }' | grep -v '^fixtures/' || true)
if [[ -n "$BIG" ]]; then
  fail "a large or binary file was added outside the corpus:"
  printf '%s\n' "$BIG" | sed 's/^/    /' >&2
fi

if [[ "$found" -ne 0 ]]; then
  echo "scan-pr: FAILED — the change above must not be merged as it stands." >&2
  exit 1
fi

echo "scan-pr: clean — no credential, no personal data, no unexplained dependency, no unprovenanced file in $BASE...$HEAD"
exit 0
