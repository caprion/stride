#!/usr/bin/env bash
# Stride security gate.
#
#   ./scripts/security_check.sh            run every check
#   ./scripts/security_check.sh --quick    skip the slow dependency scans
#
# This is part of the working model: every change to the project ends with a
# green run of this script. See docs/security/CHECKLIST.md for the human
# checklist behind these automated checks.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

QUICK=0
[[ "${1:-}" == "--quick" ]] && QUICK=1

PASS=0
FAIL=0
WARN=0

ok()   { printf '  \033[38;5;71mPASS\033[0m  %s\n' "$1"; PASS=$((PASS+1)); }
bad()  { printf '  \033[38;5;167mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }
warn() { printf '  \033[38;5;179mWARN\033[0m  %s\n' "$1"; WARN=$((WARN+1)); }
head_() { printf '\n\033[1m%s\033[0m\n' "$1"; }

check() {  # check <name> <command...>
  local name="$1"; shift
  if "$@" >/dev/null 2>&1; then ok "$name"; else bad "$name"; fi
}

head_ "1. Secrets"
# The real Garmin Personal API key is 133 characters, so require a long match
# rather than the bare prefix (which also appears in the docs and in a UI
# placeholder).
SECRET_RE='(GCDC_[A-Za-z0-9._-]{60,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----|sk-[A-Za-z0-9]{40,})'
if git grep -nIE "$SECRET_RE" -- ':!*.md' ':!backend/tests' ':!docs' ':!.agent-prompts' >/dev/null 2>&1; then
  bad "a credential-shaped literal is committed in the working tree"
  git grep -nIE "$SECRET_RE" -- ':!*.md' ':!backend/tests' ':!docs' ':!.agent-prompts' | head -5
else
  ok "no credential-shaped literals in the working tree"
fi

if git log --all -p 2>/dev/null | grep -qE "$SECRET_RE"; then
  bad "a credential-shaped literal is in the git history"
else
  ok "no credential-shaped literals in the git history"
fi

if git check-ignore -q backend/data/state.json 2>/dev/null && git check-ignore -q backend/data/stride.db 2>/dev/null; then
  ok "runtime state and database are gitignored"
else
  bad "backend/data is not gitignored"
fi

head_ "2. Filesystem permissions"
for f in backend/data/state.json backend/data/stride.db; do
  if [[ -f "$f" ]]; then
    mode=$(stat -c '%a' "$f")
    if [[ "$mode" == "600" || "$mode" == "400" ]]; then
      ok "$f is $mode"
    else
      bad "$f is $mode — expected 600 (it holds credentials and health data)"
    fi
  else
    warn "$f not present"
  fi
done
if [[ -d backend/data ]]; then
  mode=$(stat -c '%a' backend/data)
  if [[ "$mode" == "700" ]]; then ok "backend/data is 700"; else bad "backend/data is $mode — expected 700"; fi
fi

head_ "3. Application configuration"
if grep -rn "verify=False" backend/app --include="*.py" >/dev/null 2>&1; then
  bad "TLS verification is disabled somewhere"
else
  ok "TLS verification is never disabled"
fi
if grep -rn "allow_origins=\[\"\*\"\]\|allow_origins=\['\*'\]" backend/app --include="*.py" >/dev/null 2>&1; then
  bad "CORS allows every origin"
else
  ok "CORS is not a wildcard"
fi
if grep -rn "0\.0\.0\.0" backend/app --include="*.py" >/dev/null 2>&1; then
  warn "0.0.0.0 appears in the backend — confirm it requires authentication"
else
  ok "the backend does not default to a public bind"
fi
if grep -rn "dangerouslySetInnerHTML\|[^.]eval(" frontend/src --include="*.tsx" --include="*.ts" >/dev/null 2>&1; then
  bad "the frontend uses dangerouslySetInnerHTML or eval"
else
  ok "no dangerouslySetInnerHTML or eval in the frontend"
fi
if grep -rn "http://" frontend/src --include="*.tsx" --include="*.ts" | grep -vE "127\.0\.0\.1|localhost|w3\.org|schemas\.|openstreetmap\.org/(copyright|license)|creativecommons" >/dev/null 2>&1; then
  bad "the frontend references a plain-http origin"
else
  ok "no plain-http origins in the frontend"
fi
if grep -rn "fonts.googleapis.com\|fonts.gstatic.com" frontend/index.html frontend/src 2>/dev/null >/dev/null; then
  bad "the frontend loads fonts from Google — that leaks the athlete's IP"
else
  ok "no third-party font requests"
fi
if grep -rn "tile.openstreetmap.org" frontend/src frontend/index.html 2>/dev/null >/dev/null; then
  bad "the browser talks to OpenStreetMap directly — that leaks the athlete's IP"
else
  ok "map tiles are proxied, not fetched from the browser"
fi
if grep -rn "Content-Security-Policy" backend/app frontend/index.html >/dev/null 2>&1; then
  ok "a Content-Security-Policy is declared"
else
  bad "no Content-Security-Policy"
fi
if grep -rn "allow_origins=.*\*" backend/app --include="*.py" >/dev/null 2>&1; then
  bad "CORS allows every origin"
else
  ok "CORS is not a wildcard"
fi

head_ "4. Dependency hygiene"
if [[ $QUICK -eq 0 ]]; then
  audit_bin=""
  if [[ -x "$ROOT/backend/.venv/bin/pip-audit" ]]; then
    audit_bin="$ROOT/backend/.venv/bin/pip-audit"
  elif command -v pip-audit >/dev/null 2>&1; then
    audit_bin="pip-audit"
  fi
  if [[ -n "$audit_bin" ]]; then
    if (cd backend && "$audit_bin" -r requirements.txt >/dev/null 2>&1); then ok "pip-audit: no known vulnerabilities"; else bad "pip-audit reported something"; fi
  else
    warn "pip-audit not installed — skipped (pip install pip-audit)"
  fi
  if command -v npm >/dev/null 2>&1; then
    if (cd frontend && npm audit --omit=dev --audit-level=high >/dev/null 2>&1); then
      ok "npm audit: no high-severity issues in production dependencies"
    else
      warn "npm audit reported issues in production dependencies"
    fi
  fi
else
  warn "dependency scans skipped (--quick)"
fi

head_ "5. Tests"
if (cd backend && .venv/bin/python -m pytest -q >/dev/null 2>&1); then
  ok "backend test suite passes"
else
  bad "backend test suite fails"
fi
if (cd frontend && npm run typecheck >/dev/null 2>&1); then
  ok "frontend typechecks"
else
  bad "frontend typecheck fails"
fi

head_ "6. Live service (skipped when the app is not running)"
BASE="${STRIDE_CHECK_URL:-http://127.0.0.1:8000}"
BASE_PORT="${STRIDE_CHECK_PORT:-8000}"
if curl -fsS -m 4 -o /dev/null "$BASE/api/v1/health" 2>/dev/null; then
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 6 "$BASE/api/v1/activities")
  if [[ "$code" == "401" || "$code" == "403" ]]; then
    ok "the API rejects an unauthenticated request ($code)"
  else
    bad "the API answered $code to an unauthenticated request — expected 401 or 403"
  fi
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 6 -X POST -H 'Origin: https://evil.example' \
    -H 'Content-Type: application/json' -d '{}' "$BASE/api/v1/sync/run")
  if [[ "$code" == "401" || "$code" == "403" ]]; then
    ok "a cross-origin state-changing request is rejected ($code)"
  else
    bad "a cross-origin POST was accepted ($code)"
  fi
  if curl -fsS -m 6 -D - -o /dev/null "$BASE/" 2>/dev/null | grep -qi "content-security-policy"; then
    ok "responses carry a Content-Security-Policy"
  else
    bad "no Content-Security-Policy on the served app"
  fi

  # The pairing exchange is the one endpoint that answers without a
  # capability token, so it gets its own checks.
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 6 -X POST -H "Origin: $BASE" \
    -H 'Content-Type: application/json' -d '{"code":"ZZZZ-ZZZZ"}' "$BASE/api/v1/auth/pair")
  if [[ "$code" == "401" || "$code" == "403" ]]; then
    ok "the pairing exchange rejects a wrong code ($code)"
  else
    bad "the pairing exchange accepted a wrong code ($code)"
  fi

  # A paired device must be revocable, and revoking one must not touch the
  # machine it was paired from.
  # The gate pairs a throwaway device and revokes *that one*. It must never
  # revoke a device the athlete is actually using.
  # A jar with the operator's own capability, so the device list is readable.
  master_jar=$(mktemp)
  curl -s -c "$master_jar" -o /dev/null -m 6 "$BASE/api/v1/bootstrap"
  before=$(curl -s -b "$master_jar" -m 6 "$BASE/api/v1/auth/devices" 2>/dev/null | tr ',' '\n' | grep -o '"id":"[a-f0-9]*"' | sort)
  pair_code=$(curl -s -m 6 "$BASE/api/v1/auth/pair-code" 2>/dev/null \
    | grep -o '"code":"[A-Z0-9-]*"' | head -1 | cut -d'"' -f4)
  if [[ -n "$pair_code" ]]; then
    jar=$(mktemp)
    curl -s -c "$jar" -o /dev/null -m 6 -X POST -H "Origin: $BASE" \
      -H 'Content-Type: application/json' -d "{\"code\":\"$pair_code\"}" \
      "$BASE/api/v1/auth/pair"
    code=$(curl -s -b "$jar" -o /dev/null -w '%{http_code}' -m 6 "$BASE/api/v1/activities")
    if [[ "$code" == "200" ]]; then
      ok "a paired device can read the API ($code)"
    else
      bad "a paired device could not read the API ($code)"
    fi
    after=$(curl -s -b "$master_jar" -m 6 "$BASE/api/v1/auth/devices" 2>/dev/null | tr ',' '\n' | grep -o '"id":"[a-f0-9]*"' | sort)
    device_id=$(comm -13 <(echo "$before") <(echo "$after") | head -1 | cut -d'"' -f4)
    if [[ -n "$device_id" ]]; then
      curl -s -b "$master_jar" -o /dev/null -m 6 -X DELETE -H "Origin: $BASE" \
        "$BASE/api/v1/auth/devices/$device_id"
      code=$(curl -s -b "$jar" -o /dev/null -w '%{http_code}' -m 6 "$BASE/api/v1/activities")
      if [[ "$code" == "401" ]]; then
        ok "a revoked device is signed out ($code)"
      else
        bad "a revoked device still has access ($code)"
      fi
    else
      warn "could not identify the device it paired to revoke it"
    fi
    rm -f "$jar" "$master_jar"
  else
    warn "could not mint a pairing code — the device checks were skipped"
    rm -f "$master_jar"
  fi

  # A pairing code must only be mintable from the machine itself. The
  # non-loopback address of this host stands in for any other device on the
  # tailnet.
  LAN_IP=$(ip -4 addr show 2>/dev/null | awk '/inet /{print $2}' | cut -d/ -f1 \
    | grep -v '^127\.' | head -1)
  if [[ -n "$LAN_IP" ]]; then
    code=$(curl -s -o /dev/null -w '%{http_code}' -m 6 "http://$LAN_IP:8000/api/v1/auth/pair-code")
    if [[ "$code" == "401" || "$code" == "403" ]]; then
      ok "a pairing code cannot be minted from another device ($code)"
    else
      bad "a remote client minted a pairing code ($code)"
    fi
  else
    warn "no non-loopback address found — the remote pairing check was skipped"
  fi
else
  warn "the app is not running at $BASE — live checks skipped"
fi

printf '\n\033[1m%d passed, %d failed, %d warnings\033[0m\n' "$PASS" "$FAIL" "$WARN"
[[ $FAIL -eq 0 ]] || { printf '\n\033[38;5;167mThe security gate failed. Fix it before shipping.\033[0m\n'; exit 1; }
printf '\n\033[38;5;71mThe security gate is green.\033[0m\n'
