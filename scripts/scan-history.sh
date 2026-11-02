#!/usr/bin/env bash
# scan-history.sh — the history scan the merge gate promises.
#
# The tree scanners read the tip of the tree; the pull-request scanner reads a
# diff. Neither looks at what a commit *added* and a later commit removed —
# and that is exactly where a leak can hide: a personal coordinate, a name, a
# Garmin id or a credential in a commit that is no longer in the tree.
#
# So this walks **every commit on every ref** and applies the same rules to the
# lines each commit added. It publishes nothing: the rules are the rules, and
# the values they look for are never written down here.
#
# It is the same rules, with the same exclusions — the scanners' own sources,
# the deliberate bad fixtures and the generated bundle are not leaks — and the
# same cheap candidate filter, so the whole history is walked in seconds
# rather than in an hour.
#
# Usage: scripts/scan-history.sh [base..head]   (default: every ref)
set -uo pipefail

cd "$(dirname "$0")/.."

# shellcheck source=scripts/privacy-rules.sh
source "$(dirname "$0")/privacy-rules.sh"

found=0
report() { echo "scan-history: $1" >&2; found=1; }

# The files that are not leaks: the scanners' own sources (they hold the
# patterns), the deliberate bad fixtures (they exist to be caught), and the
# generated bundle (machine output, built from source that is scanned).
is_excluded() {
  case "$1" in
    fixtures/scan-tests/*) return 0 ;;
    scripts/scan-secrets.sh|scripts/scan-privacy.sh|scripts/scan-pr.sh|scripts/scan-pr-self-test.sh|scripts/scan-history.sh) return 0 ;;
    desktop/assets/*) return 0 ;;
  esac
  return 1
}

# The commits to walk.
if [[ $# -gt 0 ]]; then
  COMMITS=$(git rev-list "$1")
else
  COMMITS=$(git rev-list --all)
fi

total=$(printf '%s\n' "$COMMITS" | grep -c . || true)
echo "scan-history: walking $total commits"

index=0
while IFS= read -r commit; do
  [[ -n "$commit" ]] || continue
  index=$((index + 1))
  subject=$(git log -1 --format=%s "$commit" 2>/dev/null | head -1)

  # The lines this commit added, with the file they are in.
  added=$(git show --unified=0 --format= "$commit" 2>/dev/null | awk '
    /^\+\+\+ b\// { file = substr($0, 7); next }
    /^\+/ && !/^\+\+\+/ { printf "%s\t%s\n", file, substr($0, 2) }
  ')
  [[ -n "$added" ]] || continue

  # The credential shapes: one grep over the whole commit's added lines.
  if printf '%s\n' "$added" | grep -vE '^(fixtures/scan-tests/|desktop/assets/|scripts/scan-[a-z-]*\.sh)' \
     | grep -qE 'GCDC_[A-Za-z0-9._-]{12,}|gh[pousr]_[A-Za-z0-9]{16,}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}|sk-[A-Za-z0-9]{20,}|AIza[0-9A-Za-z_-]{30,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----|eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}'; then
    report "$commit ($subject): a credential-shaped string was added"
  fi

  # The privacy rules: only the lines that could match, and only the files
  # that are not excluded. The corpus is exempt from the serial and the note
  # rules, exactly as it is in the tree scan.
  while IFS= read -r line; do
    file="${line%%$'\t'*}"
    text="${line#*$'\t'}"
    is_excluded "$file" && continue
    frozen=0
    case "$file" in fixtures/*) frozen=1 ;; esac
    while IFS= read -r violation; do
      [[ -n "$violation" ]] && report "$commit ($subject): $violation"
    done < <(privacy_check_line "$file" "$text" "$frozen")
  done < <(printf '%s\n' "$added" \
    | grep -vE '^(fixtures/scan-tests/|desktop/assets/|scripts/scan-(secrets|privacy|pr|history)\.sh|scripts/scan-pr-self-test\.sh)' \
    | grep -iE 'lat|lon|displayname|fullname|nickname|username|athlete_name|devicename|device_name|serialnumber|serial_number|external_id|unit_id|unitid|product_number|part_number|note|comment|remark|@|degree|semicircle|position|activityname|activity_name|activitytitle|activity_title')
done < <(printf '%s\n' "$COMMITS")

if [[ "$found" -ne 0 ]]; then
  echo "scan-history: FAILED — the history carries something the tree scanners would have refused." >&2
  exit 1
fi

echo "scan-history: clean — $index commits, every added line, nothing the tree scanners would refuse"
exit 0
