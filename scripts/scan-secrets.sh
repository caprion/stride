#!/usr/bin/env bash
# Fail if any tracked file carries a secret.
#
# Four layers:
#   * file names that are themselves secrets or their sidecars;
#   * literal token shapes (GitHub/Slack/OpenAI/Google/Garmin/AWS, JWT, private
#     keys, authorization headers, URLs with embedded credentials);
#   * a key-name rule — a key whose name is credential-like
#     (secret/token/password/…, from the old repo's redaction.py and
#     security.py) assigned a non-placeholder value, in any of JSON / TOML /
#     env / YAML / .properties forms.
#
# Scans `git ls-files` (tracked files only, the scanner's own source and the
# intentional bad fixtures excluded) by default, or the files given as
# arguments. Exits non-zero naming the file and the line.
set -uo pipefail

if [[ $# -gt 0 ]]; then
  mapfile -t FILES < <(printf '%s\n' "$@")
else
  mapfile -t FILES < <(
    git ls-files \
      | grep -v '^fixtures/scan-tests/' \
      | grep -v '^scripts/scan-secrets.sh'
  )
fi

found=0
fail() { echo "scan-secrets: $1" >&2; found=1; }

# --- literal token shapes ---
PRIVATE_KEY='-----BEGIN (RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY-----'
GARMIN_KEY='GCDC_[A-Za-z0-9._-]{12,}'
AWS_KEY='AKIA[0-9A-Z]{16}'
GITHUB_TOKEN='gh[pousr]_[A-Za-z0-9]{20,}'
GITHUB_PAT='github_pat_[A-Za-z0-9_]{20,}'
SLACK_TOKEN='xox[baprs]-[A-Za-z0-9-]{10,}'
OPENAI_KEY='sk-[A-Za-z0-9]{20,}'
GOOGLE_KEY='AIza[0-9A-Za-z_-]{35}'
JWT='eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+'
AUTH_HEADER='[Aa]uthorization:[[:space:]]*(Bearer|Basic|Token)[[:space:]]+[A-Za-z0-9._/-]{8,}'
URL_CRED='[a-z][a-z0-9+.-]*://[^/@[:space:]]+:[^/@[:space:]]+@'

# --- the key-name rule (shape from redaction.py + security.py) ---
SECRET_KEY_NAMES='(api[_-]?key|apikey|authorization|bearer|consumer[_-]?secret|cookie|credential|oauth[_-]?token([_-]?secret)?|password|passwd|private[_-]?key|secret|session|token)'
# Two shapes, because the value's shape tells us what the line is:
#   "="  — TOML, .env, .properties. The value may be bare.
#   ":"  — JSON, YAML, or a type declaration. A quoted value is data; an
#          unquoted one is usually a *type* ("session: SessionResponse"), and
#          a type is not a secret. YAML and env files are exempt, because
#          their values are routinely unquoted.
KEY_ASSIGN_EQ="$SECRET_KEY_NAMES[\"']?[[:space:]]*=[[:space:]]*"
KEY_ASSIGN_COLON="$SECRET_KEY_NAMES[\"']?[[:space:]]*:[[:space:]]*"
VALUE_QUOTED='["'"'"']\K[^"'"'"']+'
VALUE_BARE='\K[^"'"'"',}[:space:]]+'

report() {
  local h
  h=$(grep -nI -E -- "$2" "$1" 2>/dev/null | head -1)
  [[ -n "$h" ]] && fail "$1:${h%%:*}: $3"
}

for f in "${FILES[@]}"; do
  [[ -f "$f" ]] || continue

  case "$(basename "$f")" in
    *.jks|*.keystore|*.p12|*.pfx|*.db|*.sqlite|*.sqlite3|*.db-wal|*.sqlite-wal|*.db-journal|*.sqlite-journal|*.ppk|*.pem|*.kdbx|id_rsa|id_ed25519|id_ecdsa|local.properties)
      fail "$f: forbidden secret file name";;
  esac

  grep -qI -- . "$f" 2>/dev/null || continue

  report "$f" "$PRIVATE_KEY"  "private key block"
  report "$f" "$GARMIN_KEY"   "Garmin personal API key"
  report "$f" "$AWS_KEY"      "AWS access key id"
  report "$f" "$GITHUB_TOKEN" "GitHub token"
  report "$f" "$GITHUB_PAT"   "GitHub fine-grained PAT"
  report "$f" "$SLACK_TOKEN"  "Slack token"
  report "$f" "$OPENAI_KEY"   "OpenAI API key"
  report "$f" "$GOOGLE_KEY"   "Google API key"
  report "$f" "$JWT"          "JWT-shaped token"
  report "$f" "$AUTH_HEADER"  "authorization header with a token"
  report "$f" "$URL_CRED"     "URL with embedded credentials"

  # key-name rule: a credential-like key assigned a non-placeholder value
  #
  # A value is a candidate when it is quoted (JSON/YAML/TOML), or when the
  # assignment uses "=" (TOML/env/properties), or when the file is a YAML or
  # env file (their values are routinely unquoted). A bare value after ":"
  # in a source file is a type declaration, not a secret.
  case "$f" in
    *.yml|*.yaml|*.env|*.properties) bare_colon_ok=1 ;;
    *) bare_colon_ok=0 ;;
  esac
  while IFS= read -r line; do
    lineno="${line%%:*}"
    text="${line#*:}"
    value=$(printf '%s' "$text" | grep -ioP "$KEY_ASSIGN_EQ$VALUE_QUOTED" | head -1)
    if [[ -z "$value" ]]; then
      value=$(printf '%s' "$text" | grep -ioP "$KEY_ASSIGN_EQ$VALUE_BARE" | head -1)
    fi
    if [[ -z "$value" ]]; then
      value=$(printf '%s' "$text" | grep -ioP "$KEY_ASSIGN_COLON$VALUE_QUOTED" | head -1)
    fi
    if [[ -z "$value" && "$bare_colon_ok" == "1" ]]; then
      value=$(printf '%s' "$text" | grep -ioP "$KEY_ASSIGN_COLON$VALUE_BARE" | head -1)
    fi
    [[ -z "$value" ]] && continue
    # Only a non-placeholder *string* is a secret: skip placeholders, booleans,
    # null, numbers, objects and arrays (a credential is never `false` or `{`).
    if printf '%s' "$value" | grep -qiE '^(redacted|\[redacted\]|<[^>]+>|changeme|example|your[-_][a-z-]+|x+|\\*+|replace[-_]?me|null|true|false|[0-9]+|\{|\[)$'; then
      continue
    fi
    fail "$f:$lineno: credential-like key with a value"
  done < <(grep -niP "$SECRET_KEY_NAMES[\"']?[[:space:]]*[:=]" "$f" 2>/dev/null)
done

if [[ "$found" -ne 0 ]]; then
  echo "scan-secrets: secrets found in tracked files" >&2
fi
exit "$found"
