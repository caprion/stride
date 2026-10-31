#!/usr/bin/env bash
# privacy-rules.sh — the privacy rules, shared.
#
# Sourced by scripts/scan-privacy.sh (the tree) and scripts/scan-pr.sh (the
# diff), so the two cannot drift apart. Two scanners with two copies of a
# rule is how a rule stops being a rule.
#
# ## The rule, in one line
#
# The only coordinates this repository may contain are the **synthetic anchor**
# and a short list of obviously-fictional values. Not a geographic band: a
# band narrow enough to catch a real coordinate is also narrow enough to tell
# a stranger roughly where the athlete lives. So the rule is anchored, not
# regional — and it publishes nothing about anybody.
#
# The anchor is (-35.0, -140.0): the South Pacific, far from any coast and
# from any real route. A value within 1° of it is the corpus's own geometry.
#
# ## What is checked
#
#   1. a *labelled* coordinate — a key containing lat / lon / latitude /
#      longitude / position_lat / position_long, with a decimal value;
#   2. an *unlabelled pair* of decimals that could be a coordinate (three or
#      more decimal places, one inside [-90, 90] and one inside [-180, 180]);
#   3. a decimal next to the words "degree" or "semicircle" — a coordinate
#      conversion in code, where the value is the answer and the label is the
#      function name;
#   4. a Garmin profile field carrying a personal value (a name that is not a
#      placeholder, a non-zero user id);
#   5. an email address (the RFC 2606 reserved TLDs are not personal data);
#   6. a phone number, but only when it is actually a phone number;
#   7. a free-text note — the only free text the product stores is the
#      athlete's own;
#   8. a device name or a serial number, where the schema says they live.

# The synthetic anchor, and how far a value may sit from it.
ANCHOR_LAT='-35.0'
ANCHOR_LON='-140.0'
ANCHOR_TOLERANCE='1'

# Values that are obviously not a place. Anything else must be the anchor.
ALLOWED_COORD_VALUES='^-?(0|1|45|90|180)(\.0+)?$'

# A coordinate-ish decimal: a signed number with a fractional part.
NUM='-?[0-9]{1,3}\.[0-9]{1,}'

# Keys whose value is a coordinate.
COORD_KEY='(lat|lon|latitude|longitude|position_lat|position_long|lat_semicircles|long_semicircles|start_lat|start_long|end_lat|end_long)'
# Keys whose value is a Garmin profile identity.
PROFILE_NAME_KEY='(displayName|fullName|nickName|userName|athlete_name|deviceName|device_name|firstName|lastName)'
# Keys whose value is a serial number.
SERIAL_KEY='(serialNumber|serial_number|external_id|unitId|unit_id|productNumber|part_number)'

# The placeholder set. A value that is one of these is not data.
PLACEHOLDER_RE='^(placeholder|example|sample|redacted|your[_-]?key|<[^>]+>|none|null|nil|false|true|0|test|testing|fake|dummy|changeme|change_me|todo|fixme|x+|x{3,}|\*+|n/?a|hidden|secret|pass|token|key|value|unknown|anonymous|user|athlete|device)+$|^((fake|dummy|test|sample|example|placeholder|redacted)[_-].*)$'

# ---------------------------------------------------------------------------
# privacy_is_allowed_coord <value> <role>
#   role: "lat" or "lon". Returns 0 when the value may appear.
# ---------------------------------------------------------------------------
privacy_is_allowed_coord() {
  local value="$1" role="$2" anchor
  case "$role" in
    lat) anchor="$ANCHOR_LAT" ;;
    lon) anchor="$ANCHOR_LON" ;;
    *) return 1 ;;
  esac

  # An obviously fictional value.
  if printf '%s' "$value" | grep -qiE -- "$ALLOWED_COORD_VALUES"; then
    return 0
  fi

  # Within tolerance of the anchor, on the anchor's own axis.
  if printf '%s' "$value" | awk -v v="$value" -v a="$anchor" -v t="$ANCHOR_TOLERANCE" \
       'BEGIN { d = v - a; if (d < 0) d = -d; exit !(d <= t) }'; then
    return 0
  fi

  return 1
}

# ---------------------------------------------------------------------------
# privacy_check_line <file> <text>
#   Prints one line per violation; returns 0 when the line is clean.
# ---------------------------------------------------------------------------
privacy_check_line() {
  local file="$1" text="$2" out='' key value skip_serials="${3:-0}"

  # 1. a labelled coordinate
  while IFS= read -r kv; do
    key="${kv%%=*}"
    value="${kv#*=}"
    [[ -n "$value" ]] || continue
    if [[ "$key" =~ [Ll]at|[Ll]on|[Ll]ongitude|[Ll]atitude ]]; then
      case "$key" in
        *[Ll]at) role=lat ;;
        *) role=lon ;;
      esac
      privacy_is_allowed_coord "$value" "$role" || out+="coordinate value '$value' for '$key'"$'\n'
    fi
  done < <(printf '%s' "$text" | grep -oP -- "(?i)[\"']?$COORD_KEY[\"']?[[:space:]]*[:=][[:space:]]*[\"']?$NUM" \
    | sed -E 's/^["'"'"']?//; s/["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?/=/')

  # 2. an unlabelled pair of decimals that could be a coordinate
  while IFS= read -r pair; do
    a="${pair%%,*}"
    b="${pair#*,}"
    ok=1
    privacy_is_allowed_coord "$a" lat && ok=0
    privacy_is_allowed_coord "$b" lon && ok=0
    privacy_is_allowed_coord "$a" lon && ok=0
    privacy_is_allowed_coord "$b" lat && ok=0
    [[ "$ok" == "1" ]] && out+="an unlabelled coordinate pair ($pair)"$'\n'
  done < <(printf '%s' "$text" | grep -oP -- "$NUM[[:space:]]*,[[:space:]]*$NUM" | grep -P "\.[0-9]{3,}" | head -3)

  # 3. a coordinate conversion: a precise decimal near "degree"/"semicircle"
  while IFS= read -r val; do
    # Only a value that could be a latitude: the semicircles constant beside
    # it (216.0) is not a place.
    printf '%s' "$val" | awk -v v="$val" 'BEGIN { a = v < 0 ? -v : v; exit !(a <= 90) }' || continue
    if ! privacy_is_allowed_coord "$val" lat && ! privacy_is_allowed_coord "$val" lon; then
      out+="a coordinate conversion result ($val)"$'\n'
    fi
  done < <(printf '%s' "$text" | grep -oiP -- ".{0,60}(degree|semicircle).{0,60}$NUM" \
    | grep -oP -- "$NUM" | head -3)

  # 4. a Garmin profile field with a personal value
  while IFS= read -r kv; do
    key="${kv%%=*}"
    value="${kv#*=}"
    [[ -n "$value" ]] || continue
    if printf '%s' "$value" | grep -qiP -- "^$PLACEHOLDER_RE$"; then
      continue
    fi
    case "$value" in
      0|0.0) continue ;;
    esac
    out+="a profile field '$key' carrying '$value'"$'\n'
  done < <(printf '%s' "$text" | grep -oP -- "(?i)[\"']?($PROFILE_NAME_KEY)[\"']?[[:space:]]*[:=][[:space:]]*[\"']?[^\"',}[:space:]]+" \
    | sed -E 's/^["'"'"']?//; s/["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?/=/')

  # 5. a serial number that is not obviously synthetic
  #
  # The frozen corpus legitimately carries serial-shaped values (a FIT file
  # has one, and the schema's external_id holds it). The corpus is frozen,
  # hashed and documented, so the tree scanner does not re-litigate it; the
  # diff scanner does, because a *new* serial anywhere is new data.
  #
  # A real device serial is long (8+ digits) or alphanumeric (six characters
  # or more). The corpus uses short invented numbers, which are documented as
  # invented in fixtures/README.md; those are not personal data.
  while IFS= read -r kv; do
    [[ "$skip_serials" == "1" ]] && continue
    key="${kv%%=*}"
    value="${kv#*=}"
    [[ -n "$value" ]] || continue
    case "$value" in
      *:*|*=*) continue ;;
    esac
    if printf '%s' "$value" | grep -qiP -- "^$PLACEHOLDER_RE$"; then
      continue
    fi
    case "$value" in
      0|0.0) continue ;;
    esac
    # A serial is a token: digits, dashes and underscores, nothing else.
    # "Option<u64>" and "self.serial_number" are code, not a serial.
    printf '%s' "$value" | grep -qP -- '^[A-Za-z0-9_-]{5,}$' || continue
    digits=$(printf '%s' "$value" | tr -cd '0-9' | wc -c)
    letters=$(printf '%s' "$value" | tr -cd '[:alpha:]' | wc -c)
    len=${#value}
    if [[ "$digits" -gt 0 ]] || { [[ "$letters" -eq "$len" ]] && [[ "$len" -ge 8 ]]; }; then
      out+="a serial field '$key' carrying '$value'"$'\n'
    fi
  done < <(printf '%s' "$text" | grep -oP -- "(?i)[\"']?($SERIAL_KEY)[\"']?[[:space:]]*[:=][[:space:]]*[\"']?[^\"',}[:space:]]+" \
    | sed -E 's/^["'"'"']?//; s/["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?/=/')

  if [[ -n "$out" ]]; then
    printf '%s' "$out" | while IFS= read -r v; do
      [[ -n "$v" ]] && echo "$file: $v"
    done
    return 1
  fi
  return 0
}

# ---------------------------------------------------------------------------
# privacy_check_email <text>  → 0 when clean
# ---------------------------------------------------------------------------
privacy_email_re='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.(?!invalid\b|example\b|test\b|localhost\b)[A-Za-z]{2,}'
privacy_phone_re='(\+[0-9]{1,3}[- ]?[0-9]{4,}[- ]?[0-9]{4,}|(phone|mobile|tel|telephone|phone_number|mobile_number)"?[[:space:]]*[:=][[:space:]]*"?[0-9]{6,})'
privacy_note_re='"(note|notes|comment|remark)"[[:space:]]*:[[:space:]]*"[^"]{4,}"'
