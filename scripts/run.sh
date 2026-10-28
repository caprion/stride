#!/usr/bin/env bash
# Stride — start (or restart) the app.
#
#   ./scripts/run.sh          start the API + serve the built SPA on :8000
#   ./scripts/run.sh dev      API on :8000 with reload, Vite dev server on :5173
#   ./scripts/run.sh stop     stop everything
#   ./scripts/run.sh status   show what is running
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="$ROOT/backend/.venv"
PORT="${STRIDE_PORT:-8000}"
PID_DIR="$ROOT/.run"
mkdir -p "$PID_DIR"

log() { printf '\033[38;5;208m▸\033[0m %s\n' "$*"; }

# Find the real PID of a Stride server.
#
# `setsid` forks when it is already a process group leader, so the PID we
# capture with `$!` is not always the server's. Matching on the command line
# (which includes the port) is the reliable way, and it cannot match this
# script itself because the pattern only appears in the file, not in argv.
server_pid() {
  local port="$1"
  # The server runs as ``python -m app.serve`` (both listeners, one process),
  # but an older build may still be running as plain uvicorn. Match either,
  # so a restart never leaves the old one holding the port.
  local pid
  pid="$(pgrep -f "python -m app.serve" 2>/dev/null | head -1)"
  [[ -n "$pid" ]] || pid="$(pgrep -f "uvicorn app.main:app --host 0.0.0.0 --port $port" 2>/dev/null | head -1)"
  echo "$pid"
}

stop_one() {
  local name="$1"
  local pidfile="$PID_DIR/$name.pid"
  if [[ -f "$pidfile" ]]; then
    local pid
    pid="$(cat "$pidfile")"
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      sleep 0.6
      kill -9 "$pid" 2>/dev/null || true
      log "stopped $name (pid $pid)"
    fi
    rm -f "$pidfile"
  fi
}

stop_server() {
  local port="$1" label="$2"
  local pid
  pid="$(server_pid "$port")"
  if [[ -n "$pid" ]]; then
    kill "$pid" 2>/dev/null || true
    sleep 0.8
    kill -9 "$pid" 2>/dev/null || true
    log "stopped $label (pid $pid)"
  fi
}

stop_all() {
  stop_server "$PORT" api
  stop_one api
  stop_one web
  log "Stride stopped."
}

case "${1:-start}" in
  stop)
    stop_all
    ;;

  status)
    for name in api web; do
      pidfile="$PID_DIR/$name.pid"
      if [[ -f "$pidfile" ]] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
        echo "$name: running (pid $(cat "$pidfile"))"
      else
        echo "$name: stopped"
      fi
    done
    pid="$(server_pid "$PORT")"
    if [[ -n "$pid" ]]; then
      echo "api listener on :$PORT -> pid $pid"
    fi
    ;;

  dev)
    stop_all
    log "starting API with reload on :$PORT"
    (cd "$ROOT/backend" && setsid "$VENV/bin/uvicorn" app.main:app --host 0.0.0.0 --port "$PORT" --reload \
      < /dev/null > "$PID_DIR/api.log" 2>&1 &)
    sleep 4
    server_pid "$PORT" > "$PID_DIR/api.pid" 2>/dev/null || true
    log "starting Vite dev server on :5173"
    (cd "$ROOT/frontend" && setsid npm run dev -- --host 0.0.0.0 --port 5173 \
      < /dev/null > "$PID_DIR/web.log" 2>&1 &)
    sleep 3
    log "API  → http://localhost:$PORT/api/v1/docs"
    log "App  → http://localhost:5173"
    ;;

  start)
    stop_all
    if [[ ! -d "$ROOT/frontend/dist" ]]; then
      log "frontend/dist missing — building the SPA..."
      (cd "$ROOT/frontend" && npm install --silent && npm run build)
    fi
    log "starting Stride on :$PORT"
    # app.serve runs the plain HTTP listener and, when a certificate is
    # present, an HTTPS one in the same process — so there is one scheduler
    # and one state file.
    (cd "$ROOT/backend" && setsid "$VENV/bin/python" -m app.serve \
      < /dev/null > "$PID_DIR/api.log" 2>&1 &)
    sleep 4
    server_pid "$PORT" > "$PID_DIR/api.pid" 2>/dev/null || true
    if curl -fsS -m 5 "http://127.0.0.1:$PORT/api/v1/health" > /dev/null 2>&1; then
      log "Stride is live → http://localhost:$PORT"
      log "logs: $PID_DIR/api.log"
    else
      log "API did not come up — check $PID_DIR/api.log"
      tail -20 "$PID_DIR/api.log" || true
      exit 1
    fi
    ;;

  *)
    echo "usage: $0 {start|dev|stop|status}" >&2
    exit 2
    ;;
esac
