#!/usr/bin/env bash
# Fail if any tracked file leaks personal data.
#
# Two layers:
#   * JSON/YAML/TOML text — a *geographic* rule: a latitude in the plausible
#     band AND a longitude in that band, anywhere in the same line or record
#     (not necessarily adjacent), plus Garmin profile fields that carry a
#     personal value, plus email addresses.
#   * FIT binaries — the corpus itself. The Rust reader in
#     crates/stride-testkit (run by `cargo test`) decodes every .fit and asserts
#     every position is within 1° of the synthetic anchor (-35.0, -140.0). This
#     shell scanner cannot see into binaries, so that gate is the FIT-side half.
#
# The band, the example coordinate and the id shape are FICTIONAL — published so
# the rule's shape survives without publishing where the athlete runs.
#
# Scans `git ls-files` (tracked files only, the scanner's own source, the
# sanitisation log and the intentional bad fixtures excluded) by default, or the
# files given as arguments. Exits non-zero naming the file and the line.
set -uo pipefail

# The rules live in one file, shared with scripts/scan-pr.sh, so the tree
# scanner and the diff scanner cannot drift apart.
# shellcheck source=scripts/privacy-rules.sh
source "$(dirname "$0")/privacy-rules.sh" 

if [[ $# -gt 0 ]]; then
  mapfile -t FILES < <(printf '%s\n' "$@")
else
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
# are never published). Canonical example: 15.0000, 78.0000. The `(?<![-0-9.])`
# guard keeps "-14.11" (a negative number) and "114.11" (a longer number) from
# matching the band.
BAND_LAT='(?<![-0-9.])1[4-6]\.[0-9]{2,}'
BAND_LON='(?<![-0-9.])7[6-8]\.[0-9]{2,}'

# A latitude and a longitude both in the band, on the same line, in either
# order, whatever sits between them.
PAIR_SAME_LINE="($BAND_LAT.*$BAND_LON|$BAND_LON.*$BAND_LAT)"

# The same, keyed on JSON field names, for the pretty-printed form where the
# two fields sit on different lines of the same record.
LAT_KEY='"lat(itude)?"[[:space:]]*:[[:space:]]*"?'
LON_KEY='"lon(gitude)?"[[:space:]]*:[[:space:]]*"?'
MULTI_LATLON="$LAT_KEY$BAND_LAT[\s\S]{0,200}?$LON_KEY$BAND_LON"
MULTI_LONLAT="$LON_KEY$BAND_LON[\s\S]{0,200}?$LAT_KEY$BAND_LAT"
# GeoJSON order: {"coordinates": [lon, lat]}.
COORDS_LONLAT='"coordinates"[[:space:]]*:[[:space:]]*\[[[:space:]]*"?'"$BAND_LON"'"?[[:space:]]*,[[:space:]]*"?'"$BAND_LAT"

# A real email address. The RFC 2606 reserved TLDs (.invalid, .example,
# .test, .localhost) are never routed and are not personal data.
EMAIL="$privacy_email_re"
PHONE="$privacy_phone_re"
NOTE="$privacy_note_re"

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

  # The shared rules decide: labelled coordinates, unlabelled pairs, coordinate
  # conversions, profile fields, serials.
  # The frozen corpus may carry serial-shaped values and documentation notes;
  # it is hashed and its provenance is documented, so those two rules do not
  # re-litigate it here. The diff scanner does — a new one is new data.
  frozen=0
  case "$f" in fixtures/*) frozen=1 ;; esac
  # A lockfile is a generated file of checksums and versions: the personal
  # rules do not apply to it (the secret rules still do).
  case "$f" in
    *Cargo.lock|*package-lock.json|*pnpm-lock.yaml) continue ;;
  esac
  # Two filters, and neither can hide a coordinate:
  #   * the *shape* filter for the coordinate pair (an unlabelled pair has no
  #     keyword, so the filter is on the shape of the numbers);
  #   * the *keyword* filter for the rules that need a key.
  while IFS= read -r line; do
    while IFS= read -r violation; do
      [[ -n "$violation" ]] && fail "$violation"
    done < <(privacy_check_line "$f" "$line" "$frozen")
  done < <(privacy_pair_lines "$f")

  while IFS= read -r line; do
    while IFS= read -r violation; do
      [[ -n "$violation" ]] && fail "$violation"
    done < <(privacy_check_line "$f" "$line" "$frozen")
  done < <(privacy_candidate_lines "$f")

  # The phone rules: a shape filter, so a number in any format is seen.
  while IFS= read -r line; do
    while IFS= read -r violation; do
      [[ -n "$violation" ]] && fail "$violation"
    done < <(privacy_check_line "$f" "$line" "$frozen")
  done < <(privacy_phone_lines "$f")

  report "$f" "$ID_BAD" "non-zero Garmin user id"

  grep -qI -- . "$f" 2>/dev/null || continue

  report_pcre "$f" "$EMAIL" "email address"
  report_pcre "$f" "$PAIR_SAME_LINE" "coordinate in the plausible-real band"
  if grep -Pzo "$MULTI_LATLON" "$f" >/dev/null 2>&1 \
     || grep -Pzo "$MULTI_LONLAT" "$f" >/dev/null 2>&1 \
     || grep -Pzo "$COORDS_LONLAT" "$f" >/dev/null 2>&1; then
    fail "$f: coordinate in the plausible-real band (multiline)"
  fi
done

if [[ "$found" -ne 0 ]]; then
  echo "scan-privacy: personal data found in tracked files" >&2
fi
exit "$found"
