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

# The privacy rules live in one file, shared with scripts/scan-privacy.sh.
# shellcheck source=scripts/privacy-rules.sh
source "$(dirname "$0")/privacy-rules.sh" 

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

# `git diff --name-only` sees binary files too; the +++ header does not.
FILES_TOUCHED=$(git diff --name-only "$BASE...$HEAD" | sort -u)

found=0
fail() { echo "scan-pr: $1" >&2; found=1; }

# ---------------------------------------------------------------- 1. secrets
# The key-name rule: any of these names, with a non-placeholder value.
SECRET_KEY_RE='(api[_-]?key|secret|password|passwd|token|credential|authorization|private[_-]?key|consumer[_-]?secret|client[_-]?secret|refresh[_-]?token|access[_-]?token|cookie|session[_-]?id)'
# A value long enough to be a credential.
SECRET_VALUE_RE='[:=]\s*["'"'"']?[A-Za-z0-9/+_.-]{12,}'
# A value that is obviously a placeholder. Anchored to the whole value — a
# substring search would let "0" inside a real token excuse it.
# (PLACEHOLDER_RE itself belongs to scripts/privacy-rules.sh, so the tree
# scanner and this one agree on what a placeholder is.)
SECRET_PLACEHOLDER_RE="$PLACEHOLDER_RE"

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
  printf '%s' "$value" | grep -qiP -- "$SECRET_PLACEHOLDER_RE" && continue
  fail "a credential-named field with a non-placeholder value was added — $file"
  echo "    ${text:0:160}" >&2
done < <(printf '%s\n' "$ADDED")

# ------------------------------------------------------------- 2. personal data
# The shared rules decide what is personal: labelled coordinates, unlabelled
# pairs, coordinate conversions, profile fields, serials, emails, phones and
# free-text notes. The rules are anchored on the synthetic corpus anchor, not
# on a geographic band — a band narrow enough to catch a real coordinate is
# also narrow enough to tell a stranger roughly where the athlete lives.
# Two filters, and neither can hide a coordinate:
#   * the *shape* filter for the coordinate pair — an unlabelled pair has no
#     keyword, so the filter is on the shape of the numbers;
#   * the *keyword* filter for the rules that need a key.
CANDIDATES=$(printf '%s\n' "$ADDED" | grep -E '[0-9]\.[0-9]+[[:space:]]*,[[:space:]]*[-0-9]*[0-9]\.[0-9]+' || true)
KEY_LINES=$(printf '%s\n' "$ADDED" | grep -iE 'lat|lon|displayname|fullname|nickname|username|athlete_name|devicename|device_name|serialnumber|serial_number|external_id|unit_id|unitid|product_number|part_number|note|comment|remark|@|degree|semicircle|position|activityname|activity_name|activitytitle|activity_title' || true)
PHONE_LINES=$(printf '%s\n' "$ADDED" | grep -E '\+[0-9]{1,3}([-. ][0-9]{2,4}){1,4}|(phone|mobile|tel|telephone|phone_number|mobile_number)' || true)

while IFS= read -r line; do
  while IFS= read -r violation; do
    [[ -n "$violation" ]] && fail "$violation"
  done < <(privacy_check_line "${line%%$'\t'*}" "${line#*$'\t'}")
done < <(printf '%s\n' "$CANDIDATES")

while IFS= read -r line; do
  while IFS= read -r violation; do
    [[ -n "$violation" ]] && fail "$violation"
  done < <(privacy_check_line "${line%%$'\t'*}" "${line#*$'\t'}")
done < <(printf '%s\n' "$KEY_LINES")

while IFS= read -r line; do
  while IFS= read -r violation; do
    [[ -n "$violation" ]] && fail "$violation"
  done < <(privacy_check_line "${line%%$'\t'*}" "${line#*$'\t'}")
done < <(printf '%s\n' "$PHONE_LINES")

while IFS= read -r line; do
  text="${line#*$'\t'}"
  printf '%s' "$text" | grep -qP -- "$privacy_email_re" \
    && fail "an email address was added — ${line%%$'\t'*}"
  printf '%s' "$text" | grep -qP -- "$privacy_phone_re" \
    && fail "a phone number was added — ${line%%$'\t'*}"
  printf '%s' "$text" | grep -qP -- "$privacy_note_re" \
    && fail "a free-text note was added — the only free text the product stores is the athlete's own; if this is a fixture, use a placeholder"
done < <(printf '%s\n' "$ADDED")

# ------------------------------------------------------------ 3. dependencies
NEW_DEPS=$(printf '%s\n' "$FILES_TOUCHED" | grep -E '(^|/)Cargo\.toml$|package\.json$' || true)
#
# A dependency is read out of the *manifest*, before and after — not out of
# the diff, where a section header is just another added line. A dependency
# outside the approved list (scripts/approved-deps.txt) fails the build; the
# approved list is the contract's "no new dependency without saying so".
unapproved=()

cargo_deps_in() {
  # $1 = a Cargo.toml file. Prints the dependency names of every dependency
  # section, one per line.
  awk '
    /^[[:space:]]*\[/ { sec = $0; sub(/[[:space:]]*#.*/, "", sec) }
    sec ~ /^\[(dev-|build-)?dependencies\]/ {
      line = $0; sub(/#.*/, "", line)
      if (match(line, /^[[:space:]]*"?[A-Za-z0-9_.-]+"?[[:space:]]*=/)) {
        n = substr(line, RSTART, RLENGTH)
        gsub(/[[:space:]"?=]/, "", n)
        print n
      }
    }' "$1" | sort -u
}

npm_deps_in() {
  # $1 = a package.json file.
  jq -r '((.dependencies // {}) + (.devDependencies // {}) + (.peerDependencies // {})) | keys[]' "$1" 2>/dev/null | sort -u
}

for manifest in $NEW_DEPS; do
  base_file=$(mktemp); head_file=$(mktemp)
  git show "$BASE:$manifest" > "$base_file" 2>/dev/null || : > "$base_file"
  git show "$HEAD:$manifest" > "$head_file" 2>/dev/null || : > "$head_file"

  case "$manifest" in
    *Cargo.toml)
      before=$(cargo_deps_in "$base_file")
      after=$(cargo_deps_in "$head_file")
      ;;
    *package.json)
      before=$(npm_deps_in "$base_file")
      after=$(npm_deps_in "$head_file")
      ;;
  esac

  while IFS= read -r dep; do
    [[ -z "$dep" ]] && continue
    printf '%s\n' "$before" | grep -qxF "$dep" && continue   # not new
    case "$dep" in
      stride-*) continue ;;                                   # an internal crate
    esac
    grep -qxF "$dep" scripts/approved-deps.txt && continue
    unapproved+=("$dep")
  done < <(printf '%s\n' "$after")

  rm -f "$base_file" "$head_file"
done

if [[ ${#unapproved[@]} -gt 0 ]]; then
  fail "a dependency was added that is not on the approved list (scripts/approved-deps.txt):"
  printf '    %s\n' "${unapproved[@]}" >&2
  echo "    A dependency outside the approved list needs a plan change that travels with it." >&2
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
