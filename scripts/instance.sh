#!/usr/bin/env bash
# Run an isolated instance of Stride.
#
#   ./scripts/instance.sh <name> [port] [--clone] [--dev]
#
# Each instance gets its own data directory, so a new iteration can never
# touch the athlete's live history. The point is to be able to run a parallel
# version — a different port, a different URL, a different database — and
# throw it away afterwards without risking what already exists.
#
#   ./scripts/instance.sh next 8010 --clone   # a copy of the current data
#   ./scripts/instance.sh next 8010            # an empty instance
#   ./scripts/instance.sh --list
#   ./scripts/instance.sh --stop next
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTANCES="$ROOT/backend/instances"
VENV="$ROOT/backend/.venv"

log() { printf '\033[38;5;208m▸\033[0m %s\n' "$*"; }
die() { printf '\033[38;5;167m%s\033[0m\n' "$*" >&2; exit 1; }

usage() {
  sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
}

list_instances() {
  if [[ ! -d "$INSTANCES" ]]; then echo "no instances"; return; fi
  for dir in "$INSTANCES"/*/; do
    [[ -d "$dir" ]] || continue
    name="$(basename "$dir")"
    port="$(cat "$dir/port" 2>/dev/null || echo '?')"
    running="stopped"
    [[ -f "$dir/pid" ]] && kill -0 "$(cat "$dir/pid")" 2>/dev/null && running="running"
    printf '  %-16s port %-6s %-8s %s\n' "$name" "$port" "$running" "$(du -sh "$dir" 2>/dev/null | cut -f1)"
  done
}

stop_instance() {
  local name="$1"
  local dir="$INSTANCES/$name"
  [[ -f "$dir/pid" ]] || { log "instance '$name' is not running"; return; }
  local pid; pid="$(cat "$dir/pid")"
  kill "$pid" 2>/dev/null || true
  sleep 1
  kill -9 "$pid" 2>/dev/null || true
  rm -f "$dir/pid"
  log "stopped instance '$name'"
}

case "${1:-}" in
  --list) list_instances; exit 0 ;;
  --stop) [[ $# -ge 2 ]] || usage; stop_instance "$2"; exit 0 ;;
  "") usage ;;
esac

NAME="$1"
PORT="${2:-8010}"
shift $(( $# > 2 ? 2 : $# )) || true
CLONE=0
DEV=0
for arg in "$@"; do
  [[ "$arg" == "--clone" ]] && CLONE=1
  [[ "$arg" == "--dev" ]] && DEV=1
done

DIR="$INSTANCES/$NAME"
DATA="$DIR/data"
mkdir -p "$DATA"
chmod 700 "$DATA"
echo "$PORT" > "$DIR/port"

if [[ $CLONE -eq 1 ]]; then
  log "cloning the current data into $DATA"
  # The desktop encrypts the database it opens and keeps the plain one
  # beside it, and this is the *Python* instance: it needs a file SQLite
  # can read, so the plain one is the one to clone.
  live_db="$ROOT/backend/data/stride.db"
  if [[ ! -f "$live_db" ]] || ! head -c 16 "$live_db" 2>/dev/null | grep -q "SQLite format 3"; then
    live_db="$ROOT/backend/data/stride.db.plaintext-backup"
  fi
  if [[ -f "$live_db" ]]; then
    cp -p "$live_db" "$DATA/stride.db" 2>/dev/null || true
    cp -p "$ROOT/backend/data/state.json" "$DATA/" 2>/dev/null || true
    chmod 600 "$DATA"/* 2>/dev/null || true
  else
    log "no current database to clone"
  fi
fi

log "starting instance '$NAME' on port $PORT (data: $DATA)"
if [[ $DEV -eq 1 ]]; then
  (
    cd "$ROOT/frontend"
    STRIDE_DATA_DIR="$DATA" VITE_API_TARGET="http://127.0.0.1:$PORT" \
      setsid npm run dev -- --host 0.0.0.0 --port $(( PORT + 1 )) \
      < /dev/null > "$DIR/frontend.log" 2>&1 &
    echo $! > "$DIR/frontend.pid"
  )
  sleep 2
fi

(
  cd "$ROOT/backend"
  STRIDE_DATA_DIR="$DATA" STRIDE_PORT="$PORT" \
    setsid "$VENV/bin/uvicorn" app.main:app --host 127.0.0.1 --port "$PORT" \
    < /dev/null > "$DIR/api.log" 2>&1 &
  echo $! > "$DIR/pid"
)

sleep 4
if curl -fsS -m 5 -o /dev/null "http://127.0.0.1:$PORT/api/v1/health" 2>/dev/null; then
  log "instance '$NAME' is live → http://localhost:$PORT"
  [[ $DEV -eq 1 ]] && log "frontend dev server → http://localhost:$(( PORT + 1 ))"
  log "logs: $DIR/api.log"
else
  log "the instance did not come up — check $DIR/api.log"
  tail -15 "$DIR/api.log" || true
  exit 1
fi
