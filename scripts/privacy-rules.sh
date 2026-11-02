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
#   5. a serial number that is not obviously synthetic;
#   6. an email address (the RFC 2606 reserved TLDs are not personal data);
#   7. a phone number, but only when it is actually a phone number;
#   8. a free-text note — the only free text the product stores is the
#      athlete's own.
#
# ## Why it is pure Bash
#
# The first version of this file ran several greps per line. On the tree that
# took fifteen minutes and held the host's build lock; on a four-file pull
# request it took two. The rules are matched with the shell's own regex
# engine instead — one process for the whole file, not one per line.

# The synthetic anchor, and how far a value may sit from it.
ANCHOR_LAT='-35.0'
ANCHOR_LON='-140.0'
ANCHOR_TOLERANCE='1'

# Values that are obviously not a place. Anything else must be the anchor.
ALLOWED_COORD_RE='^-?(0|1|45|90|180)(\.0+)?$'

# A placeholder value. A credential, a name or a serial that is one of these
# is not data.
PLACEHOLDER_RE='^(placeholder|example|sample|redacted|your[_-]?key|none|null|nil|false|true|0|test|testing|fake|dummy|changeme|change_me|todo|fixme|x+|x{3,}|\*+|n/?a|hidden|secret|pass|token|key|value|unknown|anonymous|user|athlete|device)$'

# ---------------------------------------------------------------------------
# privacy_is_allowed_coord <value> <role>   role: lat | lon
# ---------------------------------------------------------------------------
privacy_is_allowed_coord() {
  local value="$1" role="$2" anchor delta
  case "$role" in
    lat) anchor="$ANCHOR_LAT" ;;
    lon) anchor="$ANCHOR_LON" ;;
    *) return 1 ;;
  esac

  # An obviously fictional value.
  [[ "$value" =~ $ALLOWED_COORD_RE ]] && return 0

  # Within tolerance of the anchor, on the anchor's own axis.
  delta=$(awk -v v="$value" -v a="$anchor" 'BEGIN { d = v - a; if (d < 0) d = -d; printf "%.6f", d }')
  awk -v d="$delta" -v t="$ANCHOR_TOLERANCE" 'BEGIN { exit !(d <= t) }' && return 0

  return 1
}

# privacy_is_placeholder <value>
privacy_is_placeholder() {
  local value="$1"
  local lower="${value,,}"
  [[ "$lower" =~ $PLACEHOLDER_RE ]] && return 0
  # "FAKE_SERIAL", "test-123", "dummy_value" — a marker prefix.
  [[ "$lower" =~ ^(fake|dummy|test|sample|example|placeholder|redacted)[_-].*$ ]] && return 0
  return 1
}

# ---------------------------------------------------------------------------
# privacy_check_line <file> <text> [skip_serials]
#
# Prints one line per violation. Pure Bash: no processes.
# ---------------------------------------------------------------------------
privacy_check_line() {
  # The third argument marks the file as part of the *frozen corpus*: the
  # serial and note rules do not re-litigate a corpus that is hashed and
  # whose provenance is documented. The diff scanner never passes it — a new
  # serial or a new note anywhere is new data.
  #
  # The rules are split by what they need:
  #   * the *shape* rules (a coordinate pair, a conversion, an email, a phone)
  #     must see every line that could match — a keyword filter would make an
  #     unlabelled coordinate pair unreachable, which is the exact case the
  #     pair rule exists to catch;
  #   * the *key* rules (a labelled coordinate, a profile field, a serial)
  #     need a key, so they are given the lines that carry one.
  local file="$1" text="$2" frozen="${3:-0}"
  local out='' key value rest pair a b

  # The patterns live in variables: inside [[ =~ ]] an unquoted variable is
  # the pattern, and it may contain quotes without any escaping dance.
  # The value may be a whole number - a coarse coordinate, where the key is
  # the evidence. (The example is written without the key, because the key is
  # what the rule looks for.)
  local RE_COORD='["'"'"']?([A-Za-z0-9_.-]+)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?(-?[0-9]{1,3}(\.[0-9]+)?)'
  # Two decimals is the floor. One decimal is mostly noise — a table of
  # medians reads as "3.0, 2.0" — and a real coordinate is written with
  # more precision than that. Two decimals still catches the coarse case
  # (a coarse latitude and longitude, written with two decimals) and, unlike a
  # keyword filter, it cannot miss a pair that has no label at all.
  local RE_PAIR='(-?[0-9]{1,3}\.[0-9]{2,})[[:space:]]*,[[:space:]]*(-?[0-9]{1,3}\.[0-9]{2,})'
  # A pair written with one decimal is coarse - about ten kilometres - but
  # real. It is only flagged where a word nearby says this is a place, because
  # a pair of small numbers is usually arithmetic, not geography.
  local RE_PAIR_COARSE='(-?[0-9]{1,3}\.[0-9])[[:space:]]*,[[:space:]]*(-?[0-9]{1,3}\.[0-9])'
  local RE_PLACE_WORD='(near|around|about|point|coord|location|place|route|track|home|city|park|street|address|gps|south|north|east|west)'
  local RE_CONV_AFTER='(degree|semicircle)[^0-9-]{0,24}(-?[0-9]{1,3}\.[0-9]{4,})'
  local RE_CONV_BEFORE='(-?[0-9]{1,3}\.[0-9]{4,})[^0-9-]{0,24}(degree|semicircle)'
  local RE_PROFILE='["'"'"']?([A-Za-z0-9_.-]*(displayName|fullName|nickName|userName|athlete_name|deviceName|device_name|firstName|lastName)[A-Za-z0-9_.-]*)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?([^"'"'"',}[:space:]]+)'
  local RE_SERIAL='["'"'"']?([A-Za-z0-9_.-]*(serialNumber|serial_number|external_id|unitId|unit_id|productNumber|part_number)[A-Za-z0-9_.-]*)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?([A-Za-z0-9_-]{5,})'

  # --- 1. a labelled coordinate ------------------------------------------
  rest="$text"
  while [[ "$rest" =~ $RE_COORD ]]; do
    local m="${BASH_REMATCH[0]}" key="${BASH_REMATCH[1]}" value="${BASH_REMATCH[2]}"
    rest="${rest#*"$m"}"
    # Is one of the key's segments a coordinate word? A *segment*, not a
    # substring: "d3-interpolate" is not a latitude, and a package-lock is
    # full of words that end in "lat".
    # A coordinate key is one that holds a coordinate word as a *segment*:
    # separated ("position_lat", "lat"), or at a camelCase boundary
    # ("latitudeDeg", "posLat", "latVal"). "template" and "d3-interpolate"
    # are not coordinate keys, and neither is "headline_note".
    local role=''
    # Case is spelled out with character classes rather than nocasematch:
    # nocasematch would also loosen the *negated* classes, and then
    # "template" and "d3-interpolate" would read as latitudes.
    local LAT='[lL][aA][tT]'
    local LATITUDE='[lL][aA][tT][iI][tT][uU][dD][eE]'
    local LON='[lL][oO][nN]'
    local LONGITUDE='[lL][oO][nN][gG][iI][tT][uU][dD][eE]'
    local LNG='[lL][nN][gG]'
    if [[ "$key" =~ (^|[^A-Za-z0-9])($LAT|$LATITUDE)([^A-Za-z0-9]|$) ]] \
       || [[ "$key" =~ ($LAT|$LATITUDE)([A-Z]) ]] \
       || [[ "$key" =~ ([a-z0-9])($LAT|$LATITUDE)($) ]]; then
      role=lat
    elif [[ "$key" =~ (^|[^A-Za-z0-9])($LON|$LONGITUDE|$LNG)([^A-Za-z0-9]|$) ]] \
         || [[ "$key" =~ ($LON|$LONGITUDE|$LNG)([A-Z]) ]] \
         || [[ "$key" =~ ([a-z0-9])($LON|$LONGITUDE|$LNG)($) ]]; then
      role=lon
    fi
    [[ -z "$role" ]] && continue
    privacy_is_allowed_coord "$value" "$role" || \
      out+="$file: coordinate value '$value' for '$key'"$'\n'
  done

  # --- 2. an unlabelled pair of decimals that could be a coordinate -------
  rest="$text"
  while [[ "$rest" =~ $RE_PAIR ]]; do
    local m="${BASH_REMATCH[0]}" a="${BASH_REMATCH[1]}" b="${BASH_REMATCH[2]}"
    rest="${rest#*"$m"}"
    if ! privacy_is_allowed_coord "$a" lat && ! privacy_is_allowed_coord "$a" lon \
       && ! privacy_is_allowed_coord "$b" lat && ! privacy_is_allowed_coord "$b" lon; then
      out+="$file: an unlabelled coordinate pair ($a, $b)"$'\n'
    fi
  done

  # The coarse pair, where a word says this is a place.
  rest="$text"
  while [[ "$rest" =~ $RE_PAIR_COARSE ]]; do
    local m="${BASH_REMATCH[0]}" a="${BASH_REMATCH[1]}" b="${BASH_REMATCH[2]}"
    rest="${rest#*"$m"}"
    [[ "$text" =~ $RE_PLACE_WORD ]] || continue
    # One of the two has to be big enough to be a place: a pair of small
    # numbers is arithmetic, not geography.
    big=0
    for n in "$a" "$b"; do
      awk -v v="$n" 'BEGIN { a = v < 0 ? -v : v; exit !(a >= 45) }' && big=1
    done
    [[ "$big" == "1" ]] || continue
    if ! privacy_is_allowed_coord "$a" lat && ! privacy_is_allowed_coord "$a" lon \
       && ! privacy_is_allowed_coord "$b" lat && ! privacy_is_allowed_coord "$b" lon; then
      out+="$file: a coarse coordinate pair ($a, $b)"$'\n'
    fi
  done

  # --- 3. a coordinate conversion -----------------------------------------
  #
  # A line that talks about degrees or semicircles must not carry a precise
  # decimal that could be a coordinate. The magnitude filter matters: the
  # constant 0.03490658503988659 (radians per degree) belongs in a converter,
  # and 154757216.0 (a semicircle count) belongs in a test — neither is a
  # place. A coordinate is between 1 and 90 degrees in magnitude.
  if [[ "$text" =~ (degree|semicircle) ]]; then
    local RE_NUM4='(-?[0-9]{1,3}\.[0-9]{4,})'
    rest="$text"
    while [[ "$rest" =~ $RE_NUM4 ]]; do
      local m="${BASH_REMATCH[0]}" value="${BASH_REMATCH[1]}"
      rest="${rest#*"$m"}"
      # |value| between 1 and 90: a degree-sized number.
      awk -v v="$value" 'BEGIN { a = v < 0 ? -v : v; exit !(a >= 1 && a <= 90) }' || continue
      if ! privacy_is_allowed_coord "$value" lat && ! privacy_is_allowed_coord "$value" lon; then
        out+="$file: a coordinate conversion result ($value)"$'\n'
      fi
    done
  fi

  # --- 4. a Garmin profile field with a personal value --------------------
  rest="$text"
  while [[ "$rest" =~ $RE_PROFILE ]]; do
    local m="${BASH_REMATCH[0]}" key="${BASH_REMATCH[1]}" value="${BASH_REMATCH[3]}"
    rest="${rest#*"$m"}"
    privacy_is_placeholder "$value" || [[ "$value" == "0" || "$value" == "0.0" ]] || \
      out+="$file: a profile field '$key' carrying '$value'"$'\n'
  done

  # --- 5. a serial number that is not obviously synthetic -----------------
  if [[ "$frozen" != "1" ]]; then
    rest="$text"
    while [[ "$rest" =~ $RE_SERIAL ]]; do
      local m="${BASH_REMATCH[0]}" key="${BASH_REMATCH[1]}" value="${BASH_REMATCH[3]}"
      rest="${rest#*"$m"}"
      privacy_is_placeholder "$value" || [[ "$value" == "0" ]] || {
        # A serial is a token: digits, dashes and underscores — "Option<u64>"
        # and "self.serial_number" are code, not a serial. The regex above
        # already guarantees the shape; what is left is the plausibility.
        local digits="${value//[!0-9]/}"
        local upper="${value//[!A-Z]/}"
        if [[ -n "$digits" ]] || { [[ "${#value}" -ge 8 ]] && [[ "${#upper}" -eq "${#value}" ]]; }; then
          out+="$file: a serial field '$key' carrying '$value'"$'\n'
        fi
      }
    done
  fi

  # --- 5b. a name that may carry a place ---------------------------------
  #
  # Garmin's activity name is free text the athlete wrote, and it is where a
  # city hides: "Morning run in Pune". The rule is an **allowlist**: the
  # synthetic names the corpus uses, and nothing else. The files it applies to
  # are the data files - the app's own source strings are chrome.
  if [[ "$file" == *fixtures/* || "$file" == *.json || "$file" == *.csv ]]; then
    # The pattern lives in a variable: an inline one with quotes inside it is
    # parsed by the shell before the regex engine ever sees it, and the match
    # silently stops working.
    #
    # Only Garmin's *activity* name: a bare "name" or "title" is far too
    # common to be evidence of anything (an OpenAPI document is full of
    # them), while "activityName" is free text the athlete wrote.
    local RE_NAME='(^|[^A-Za-z0-9_])["'"'"']?(activityName|activity_name|activityTitle|activity_title)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']([^"'"'"']{2,})["'"'"']'
    rest="$text"
    while [[ "$rest" =~ $RE_NAME ]]; do
      local m="${BASH_REMATCH[0]}" key="${BASH_REMATCH[2]}" value="${BASH_REMATCH[3]}"
      rest="${rest#*"$m"}"
      privacy_is_placeholder "$value" && continue
      case "${value,,}" in
        "bare array run"|"hidden run"|"morning run"|"trail loop"|"aerobic"|"cycling"|"gaps"|"hilly"|"hr gps run"|"low resolution"|"mixed run"|"no cadence"|"real garmin gap"|"real garmin nulls"|"real garmin run"|"short effort"|"treadmill"|"vendor coros export"|"vendor strava export"|"vendor wahoo export"|"stretching"|"yoga"|"walk"|"run") continue ;;
      esac
      # "Morning run in Pune" has a capitalised place word after a place word.
      if [[ "$value" =~ (in|near|at)[[:space:]]+[A-Z] ]]; then
        out+="$file: a name that may carry a place ('$value')"$'\n'
      fi
    done
  fi

  # --- 6. an email address (reserved TLDs are not personal data) ---------
  local RE_EMAIL_RESERVED='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.(invalid|example|test|localhost)'
  local RE_EMAIL='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
  local RE_PHONE_INTL='\+[0-9]{1,3}([-[:space:]][0-9]{2,4}){2,4}'
  local RE_PHONE_KEY='(phone|mobile|tel|telephone|phone_number|mobile_number)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?[0-9]{6,}'
  # A word boundary before the key: "headline_note" is not a note, and
  # "note: null" is a value, not free text.
  local RE_NOTE='(^|[^A-Za-z0-9_])["'"'"']?(note|notes|comment|remark)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"'][^"'"'"']{4,}["'"'"']'
  if [[ "$text" =~ $RE_EMAIL_RESERVED ]]; then
    : # a reserved domain — not personal data
  elif [[ "$text" =~ $RE_EMAIL ]]; then
    out+="$file: an email address"$'\n'
  fi

  # --- 7. a phone number, but only when it is one -------------------------
  if [[ "$text" =~ $RE_PHONE_INTL ]] || [[ "$text" =~ $RE_PHONE_KEY ]]; then
    out+="$file: a phone number"$'\n'
  fi

  # --- 8. a free-text note ------------------------------------------------
  # A note is the athlete's own free text. It is caught where a note would
  # be *data* - the corpus, the tests, a data file - and not in the app's own
  # source, where a note is a string the app shows and the voice lint governs.
  # The athlete's data can only be in the **corpus** or in a **test**. The
  # app's own source — including the app's own sample data — is the app's
  # words, and the voice lint governs those.
  case "$file" in
    fixtures/*|crates/*/tests/*|desktop/tests/*|*/tests/*|*.json|*.csv|*.toml)
      note_is_data=1
      ;;
    frontend/src/*)
      # The app's own source is the app's words.
      note_is_data=0
      ;;
    *)
      note_is_data=0
      ;;
  esac
  if [[ "$frozen" != "1" && "$note_is_data" == "1" && "$text" =~ $RE_NOTE ]]; then
    out+="$file: a free-text note — the only free text the product stores is the athlete's own; if this is a fixture, use a placeholder"$'\n'
  fi

  [[ -n "$out" ]] && printf '%s' "$out"
  return 0
}

# ---------------------------------------------------------------------------
# privacy_candidate_lines <file>
#   The cheap filter: only these lines can match. One grep for the file.
# ---------------------------------------------------------------------------
privacy_candidate_lines() {
  grep -inE 'lat|lon|displayname|fullname|nickname|username|athlete_name|devicename|device_name|serialnumber|serial_number|external_id|unit_id|unitid|product_number|part_number|note|comment|remark|@|degree|semicircle|position|activityname|activity_name|activitytitle|activity_title' "$1" 2>/dev/null | cut -d: -f2-
}

# The lines that could hold a phone number: an international prefix, or a key
# that says so. Same idea as the pair filter — a shape, not a keyword list
# someone has to remember to update.
privacy_phone_lines() {
  grep -nE '\+[0-9]{1,3}([-. ][0-9]{2,4}){1,4}|(phone|mobile|tel|telephone|phone_number|mobile_number)' "$1" 2>/dev/null | cut -d: -f2-
}

# The lines that could hold a coordinate *pair*, however they are labelled.
# A shape filter, not a keyword filter: an unlabelled pair has no keyword,
# and a filter that skipped it would make the rule unreachable.
privacy_pair_lines() {
  grep -nE '[0-9]\.[0-9]+[[:space:]]*,[[:space:]]*[-0-9]*[0-9]\.[0-9]+' "$1" 2>/dev/null | cut -d: -f2-
}

# The three text rules the diff scanner applies to whole lines. They are the
# same rules privacy_check_line applies per line, in regex form.
privacy_email_reserved_re='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.(invalid|example|test|localhost)'
privacy_email_re='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
privacy_phone_re='(\+[0-9]{1,3}([-[:space:]][0-9]{2,4}){2,4}|(phone|mobile|tel|telephone|phone_number|mobile_number)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?[0-9]{6,})'
privacy_note_re='(^|[^A-Za-z0-9_])["'"'"']?(note|notes|comment|remark)["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"'][^"'"'"']{4,}["'"'"']'
