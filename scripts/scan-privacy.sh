#!/usr/bin/env bash
# Fail if any tracked file leaks personal data.
#
# The corpus anchors every position at the synthetic anchor -35.0, -140.0 (the
# South Pacific). This gate fails on any coordinate that is NOT the anchor and
# falls in the "plausibly real" band, on a Garmin profile field that carries a
# personal value, or on an email address. The band, the example coordinate and
# the id shape below are FICTIONAL — published so the rule's shape survives
# without publishing where the athlete runs.
#
# Scans `git ls-files` (tracked files only, the scanner's own source, the
# sanitisation log and the intentional bad fixtures excluded) by default, or
# the files given as arguments. Exits non-zero with a message naming the file
# and the line.
set -uo pipefail

if [[ $# -gt 0 ]]; then
  mapfile -t FILES < <(printf '%s\n' "$@")
else
  # Exclude the files that must name the rule to do their job: this scanner's
  # own source, the intentional bad fixtures, and the sanitisation log.
  mapfile -t FILES < <(
    git ls-files \
      | grep -v '^fixtures/scan-tests/' \
      | grep -v '^scripts/scan-privacy.sh' \
      | grep -v '^fixtures/README.md'
  )
fi

found=0
fail() { echo "scan-privacy: $1" >&2; found=1; }

# Fictional "plausibly real coordinate" band — a populated region chosen to
# stand in for "somewhere real", deliberately not the athlete's routes (which
# are never published). Canonical example: 15.0000, 78.0000.
BAND_LAT='1[4-6]\.[0-9]{2,}'   # 14.00 – 16.99
BAND_LON='7[6-8]\.[0-9]{2,}'   # 76.00 – 78.99
PAIR_LINE="($BAND_LAT)[[:space:]]*,[[:space:]]*($BAND_LON)"
# The pretty-printed JSON shape: latitude, newline, longitude.
PAIR_MULTI="($BAND_LAT)[[:space:]]*,[[:space:]]*\n[[:space:]]*($BAND_LON)"

EMAIL='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'

# Garmin profile fields. A name field must hold a placeholder (ATHLETE /
# athlete); a user id must be zero.
NAME_BAD='"(displayName|fullName|nickName|userName|athlete_name)"[[:space:]]*:[[:space:]]*"(?!ATHLETE"|athlete")[^"]*"'
ID_BAD='"(userProfileId)"[[:space:]]*:[[:space:]]*[1-9][0-9]*'

report() {
  local h
  h=$(grep -nI -E -- "$2" "$1" 2>/dev/null | head -1)
  [[ -n "$h" ]] && fail "$1:${h%%:*}: $3"
}
report_pcre() {
  local h
  h=$(grep -nI -P -- "$2" "$1" 2>/dev/null | head -1)
  [[ -n "$h" ]] && fail "$1:${h%%:*}: $3"
}

for f in "${FILES[@]}"; do
  [[ -f "$f" ]] || continue

  report_pcre "$f" "$NAME_BAD" "Garmin profile name field is not a placeholder"
  report "$f" "$ID_BAD" "non-zero Garmin user id"

  grep -qI -- . "$f" 2>/dev/null || continue
  report "$f" "$EMAIL"     "email address"
  report "$f" "$PAIR_LINE" "coordinate in the plausible-real band"
  if grep -Pzo "$PAIR_MULTI" "$f" >/dev/null 2>&1; then
    fail "$f: coordinate in the plausible-real band (multiline)"
  fi
done

if [[ "$found" -ne 0 ]]; then
  echo "scan-privacy: personal data found in tracked files" >&2
fi
exit "$found"
