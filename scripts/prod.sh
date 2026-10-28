#!/usr/bin/env bash
# Stride — production deployment.
#
#   ./scripts/prod.sh setup    first run: schema, vault key, real history, plan
#   ./scripts/prod.sh start    harden permissions, start, health-check
#   ./scripts/prod.sh status   what is running and what is in the database
#   ./scripts/prod.sh doctor   the full pre-flight before you trust it
#   ./scripts/prod.sh pair     mint a one-time code to connect another device
#   ./scripts/prod.sh stop     stop the instance
#
# The production instance is the one at backend/data on :8000. It holds the
# athlete's real Garmin history and nothing synthetic.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="$ROOT/backend/.venv"
PORT="${STRIDE_PORT:-8000}"
DATA_DIR="${STRIDE_DATA_DIR:-$ROOT/backend/data}"
PY="$VENV/bin/python"

log() { printf '\033[38;5;208m▸\033[0m %s\n' "$*"; }
ok()  { printf '\033[38;5;71m  ok\033[0m %s\n' "$*"; }
bad() { printf '\033[38;5;203m  !!\033[0m %s\n' "$*"; }

# The database, its write-ahead log and the shared-memory file all hold
# health data. They must not be readable by anyone else, and neither may the
# directory that holds them.
harden_permissions() {
  chmod 700 "$DATA_DIR" 2>/dev/null || true
  find "$DATA_DIR" -maxdepth 1 -type f -exec chmod 600 {} + 2>/dev/null || true
  find "$DATA_DIR" -maxdepth 1 -type d -exec chmod 700 {} + 2>/dev/null || true
}

health() {
  curl -fsS -m 8 "http://127.0.0.1:$PORT/api/v1/health" 2>/dev/null || echo ""
}

case "${1:-status}" in

  setup)
    harden_permissions
    log "production setup — this resolves the Garmin key from Azure Key Vault"
    (cd "$ROOT/backend" && "$PY" -m tools.production setup --days 365)
    harden_permissions
    log "setup complete"
    ;;

  start)
    harden_permissions
    "$ROOT/scripts/run.sh" start
    harden_permissions
    log "health:"
    health | "$PY" -m json.tool 2>/dev/null || health
    ;;

  pair)
    # Mint a single-use pairing code for a device that is not on this
    # machine — the phone on the tailnet, typically. The code is printed
    # here, exchanged once for the session cookie, and expires in 10
    # minutes.
    if ! curl -fsS -m 5 "http://127.0.0.1:$PORT/api/v1/health" > /dev/null 2>&1; then
      bad "Stride is not answering on :$PORT — start it with scripts/prod.sh start"
      exit 1
    fi
    code="$(curl -fsS -m 10 "http://127.0.0.1:$PORT/api/v1/auth/pair-code" \
      | "$PY" -c 'import json,sys; print(json.load(sys.stdin)["code"])' 2>/dev/null || true)"
    if [[ -z "$code" ]]; then
      bad "could not mint a pairing code"
      exit 1
    fi
    echo
    printf '    \033[1;38;5;208m%s\033[0m\n' "$code"
    echo
    log "Open Stride on the other device and enter that code."
    log "It works once and expires in 10 minutes."
    ;;

  stop)
    "$ROOT/scripts/run.sh" stop
    ;;

  status)
    echo
    echo "  instance"
    printf '    %-18s %s\n' "url" "http://127.0.0.1:$PORT"
    printf '    %-18s %s\n' "data dir" "$DATA_DIR"
    printf '    %-18s %s\n' "permissions" "$(stat -c '%a' "$DATA_DIR")"
    echo
    (cd "$ROOT/backend" && "$PY" -m tools.production status)
    echo
    "$ROOT/scripts/run.sh" status
    ;;

  doctor)
    # Everything a production instance must satisfy before it is trusted.
    fail=0

    log "data directory"
    if [[ -d "$DATA_DIR" ]]; then
      mode="$(stat -c '%a' "$DATA_DIR")"
      if [[ "$mode" == "700" ]]; then ok "data dir is 0700"; else bad "data dir is $mode, expected 700"; fail=1; fi
    else
      bad "data dir missing"; fail=1
    fi

    log "secrets"
    if [[ -f "$DATA_DIR/state.json" ]]; then
      mode="$(stat -c '%a' "$DATA_DIR/state.json")"
      if [[ "$mode" == "600" ]]; then ok "state.json is 0600"; else bad "state.json is $mode, expected 600"; fail=1; fi
    else
      bad "state.json missing"; fail=1
    fi

    log "garmin key"
    key_source="$(cd "$ROOT/backend" && "$PY" -m tools.production status 2>/dev/null | awk '/key source/{print $3}')"
    if [[ "$key_source" == "key_vault" ]]; then
      ok "key resolved from Azure Key Vault"
    else
      bad "key source is '${key_source:-unknown}' — expected key_vault"; fail=1
    fi

    log "data purity"
    demo_count="$(cd "$ROOT/backend" && "$PY" - <<'PY' 2>/dev/null
from app.db import SessionLocal
from app.models import Activity
db = SessionLocal()
print(db.query(Activity).filter(Activity.source == "demo").count())
db.close()
PY
)"
    if [[ "${demo_count:-1}" == "0" ]]; then
      ok "no demo activities in the database"
    else
      bad "$demo_count demo activities still present — run: scripts/prod.sh setup"; fail=1
    fi

    log "schema drift"
    if (cd "$ROOT/backend" && "$PY" -m pytest tests/test_robustness.py -q -k "orphan or migration" > /tmp/stride-doctor-drift.log 2>&1); then
      ok "no NOT NULL orphan columns"
    else
      bad "schema drift check failed"
      tail -10 /tmp/stride-doctor-drift.log
      fail=1
    fi

    log "tests"
    if (cd "$ROOT/backend" && "$PY" -m pytest -q > /tmp/stride-doctor-tests.log 2>&1); then
      ok "backend tests pass"
    else
      bad "backend tests failing"
      tail -20 /tmp/stride-doctor-tests.log
      fail=1
    fi

    log "security gate"
    if "$ROOT/scripts/security_check.sh" > /tmp/stride-doctor-gate.log 2>&1; then
      ok "security gate passed"
    else
      bad "security gate failed — see /tmp/stride-doctor-gate.log"; fail=1
      tail -20 /tmp/stride-doctor-gate.log
    fi

    log "device pairing"
    code="$(curl -fsS -m 8 "http://127.0.0.1:$PORT/api/v1/auth/pair-code" \
      | "$PY" -c 'import json,sys; print(json.load(sys.stdin)["code"])' 2>/dev/null || true)"
    if [[ -n "$code" ]]; then
      ok "a pairing code can be minted on the machine"
      paired="$(curl -s -o /dev/null -w '%{http_code}' -m 8 -X POST \
        -H "Origin: http://127.0.0.1:$PORT" -H 'Content-Type: application/json' \
        -d "{\"code\":\"$code\"}" "http://127.0.0.1:$PORT/api/v1/auth/pair")"
      reused="$(curl -s -o /dev/null -w '%{http_code}' -m 8 -X POST \
        -H "Origin: http://127.0.0.1:$PORT" -H 'Content-Type: application/json' \
        -d "{\"code\":\"$code\"}" "http://127.0.0.1:$PORT/api/v1/auth/pair")"
      if [[ "$paired" == "204" && "$reused" == "401" ]]; then
        ok "the code works once and is refused the second time"
      else
        bad "the pairing exchange behaved oddly (first $paired, second $reused)"
        fail=1
      fi
    else
      bad "no pairing code could be minted"; fail=1
    fi

    log "http"
    if [[ -n "$(health)" ]]; then
      ok "the instance answers on :$PORT"
    else
      bad "nothing answering on :$PORT — start it with scripts/prod.sh start"; fail=1
    fi

    echo
    if [[ "$fail" == "0" ]]; then
      ok "the instance is ready"
    else
      bad "the instance is not ready"
    fi
    exit "$fail"
    ;;

  *)
    echo "usage: $0 {setup|start|stop|status|doctor|pair}" >&2
    exit 2
    ;;
esac
