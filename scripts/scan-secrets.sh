#!/usr/bin/env bash
# Fail if any tracked file carries a secret.
#
# Scans `git ls-files` (tracked files only, scan-tests excluded) by default, or
# the files given as arguments. Exits non-zero with a message naming the file
# and the line for every hit.
#
# A scanner that never fails is worse than no scanner: scripts/gate.sh runs this
# against fixtures/scan-tests/* to prove it catches the bad cases.
set -uo pipefail

if [[ $# -gt 0 ]]; then
  mapfile -t FILES < <(printf '%s\n' "$@")
else
  # Exclude this scanner's own source (it necessarily contains the secret
  # patterns it detects) and the intentional bad fixtures.
  mapfile -t FILES < <(git ls-files | grep -v '^fixtures/scan-tests/' | grep -v '^scripts/scan-secrets.sh')
fi

found=0
fail() { echo "scan-secrets: $1" >&2; found=1; }

# Private key blocks (PEM and OpenSSH).
KEYBLOCK='-----BEGIN (RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY-----'
# Garmin Connect Personal API key (the format the old repo validates).
GARMIN_KEY='GCDC_[A-Za-z0-9._-]{12,}'
# AWS access key id.
AWS_KEY='AKIA[0-9A-Z]{16}'
# An Authorization header carrying a token.
AUTH_HEADER='[Aa]uthorization:[[:space:]]*(Bearer|Basic|Token)[[:space:]]+[A-Za-z0-9._/-]{8,}'

report() {
  local file="$1" pattern="$2" label="$3"
  local hit
  # `--` stops grep from reading a pattern that begins with `-` as options.
  hit=$(grep -nI -E -- "$pattern" "$file" 2>/dev/null | head -1)
  if [[ -n "$hit" ]]; then
    fail "$file:${hit%%:*}: $label"
  fi
}

for f in "${FILES[@]}"; do
  [[ -f "$f" ]] || continue

  # Secrets that are identified by file name.
  case "$(basename "$f")" in
    *.jks|*.keystore|*.p12|*.db|*.sqlite|*.sqlite3|id_rsa|id_ed25519|id_ecdsa|*.pfx)
      fail "$f: forbidden secret file name";;
  esac

  # Skip the content scan for binary files.
  grep -qI -- . "$f" 2>/dev/null || continue

  report "$f" "$KEYBLOCK"   "private key block"
  report "$f" "$GARMIN_KEY" "Garmin personal API key"
  report "$f" "$AWS_KEY"    "AWS access key id"
  report "$f" "$AUTH_HEADER" "authorization header with a token"
done

if [[ "$found" -ne 0 ]]; then
  echo "scan-secrets: secrets found in tracked files" >&2
fi
exit "$found"
