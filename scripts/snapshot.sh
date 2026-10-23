#!/usr/bin/env bash
# Snapshot and restore the athlete's data.
#
#   ./scripts/snapshot.sh                 take a snapshot
#   ./scripts/snapshot.sh --list          list the snapshots
#   ./scripts/snapshot.sh --restore NAME  restore one (with a safety copy of the current state)
#
# A snapshot is the whole runtime directory — the SQLite database with its WAL
# sidecars, and the state file that holds the Garmin credentials — plus a
# manifest with checksums. It is the only copy of the athlete's history, so it
# is written with owner-only permissions and verified after the copy.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="${STRIDE_DATA_DIR:-$ROOT/backend/data}"
SNAPSHOT_ROOT="${STRIDE_SNAPSHOT_DIR:-$ROOT/backend/data-snapshots}"

log() { printf '\033[38;5;208m▸\033[0m %s\n' "$*"; }
die() { printf '\033[38;5;167m%s\033[0m\n' "$*" >&2; exit 1; }

list_snapshots() {
  if [[ ! -d "$SNAPSHOT_ROOT" ]]; then
    echo "no snapshots yet"
    return
  fi
  for dir in "$SNAPSHOT_ROOT"/*/; do
    [[ -d "$dir" ]] || continue
    name="$(basename "$dir")"
    size="$(du -sh "$dir" 2>/dev/null | cut -f1)"
    counts="$(python3 -c "
import json,sys
try:
    print(json.load(open('$dir/manifest.json')).get('counts') or '?')
except Exception:
    print('?')
" 2>/dev/null)"
    printf '  %-22s %6s   %s\n' "$name" "$size" "$counts"
  done
}

take_snapshot() {
  local name="${1:-$(date -u +%Y%m%dT%H%M%SZ)}"
  local dest="$SNAPSHOT_ROOT/$name"
  [[ -e "$dest" ]] && die "a snapshot named '$name' already exists"
  mkdir -p "$dest"

  log "snapshotting $DATA_DIR -> $dest"
  # Copy the database and its sidecars together, and checkpoint the WAL first
  # so the snapshot is a consistent set rather than a moving target.
  if [[ -f "$DATA_DIR/stride.db" ]]; then
    if command -v sqlite3 >/dev/null 2>&1; then
      sqlite3 "$DATA_DIR/stride.db" "PRAGMA wal_checkpoint(TRUNCATE);" >/dev/null 2>&1 || true
    fi
  fi
  for item in "$DATA_DIR"/*; do
    [[ -f "$item" ]] || continue
    case "$(basename "$item")" in
      tile-cache) continue ;;
    esac
    cp -p "$item" "$dest/" 2>/dev/null || cp "$item" "$dest/"
  done

  # The state file holds a live Garmin credential: owner-only, always.
  chmod 700 "$dest" 2>/dev/null || true
  chmod 600 "$dest"/state.json 2>/dev/null || true
  chmod 600 "$dest"/*.db 2>/dev/null || true

  # Manifest: what is in here, and a checksum to prove it has not changed.
  local counts=""
  if command -v sqlite3 >/dev/null 2>&1 && [[ -f "$dest/stride.db" ]]; then
    counts=$(sqlite3 "$dest/stride.db" \
      "select 'activity='||(select count(*) from activity)||', physiology='||(select count(*) from daily_physiology)||', fingerprints='||(select count(*) from run_fingerprint)||', plan_sessions='||(select count(*) from plan_session);" 2>/dev/null || echo "")
  fi
  {
    echo "{"
    echo "  \"name\": \"$name\","
    echo "  \"created_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"source_dir\": \"$DATA_DIR\","
    echo "  \"counts\": \"$counts\","
    echo "  \"sha256\": {"
    local first=1
    for f in "$dest"/*.db "$dest"/state.json; do
      [[ -f "$f" ]] || continue
      [[ $first -eq 1 ]] || echo ","
      first=0
      printf '    "%s": "%s"' "$(basename "$f")" "$(sha256sum "$f" | cut -d' ' -f1)"
    done
    echo ""
    echo "  }"
    echo "}"
  } > "$dest/manifest.json"
  chmod 600 "$dest/manifest.json"

  log "verifying"
  if [[ -f "$dest/stride.db" ]]; then
    local ok
    ok=$(sqlite3 "$dest/stride.db" "PRAGMA integrity_check;" 2>/dev/null || echo "unreadable")
    [[ "$ok" == "ok" ]] || die "the snapshot's database failed its integrity check: $ok"
  fi
  if [[ -f "$dest/state.json" ]]; then
    python3 -c "import json,sys; json.load(open('$dest/state.json'))" \
      || die "the snapshot's state file is not valid JSON"
  fi

  log "snapshot '$name' is good: $counts"
  echo "$dest"
}

restore_snapshot() {
  local name="$1"
  local src="$SNAPSHOT_ROOT/$name"
  [[ -d "$src" ]] || die "no snapshot named '$name'"
  [[ -f "$src/stride.db" ]] || die "snapshot '$name' has no database"

  log "restoring '$name' over $DATA_DIR"
  # Keep the current state recoverable before overwriting anything.
  take_snapshot "pre-restore-$(date -u +%Y%m%dT%H%M%SZ)" >/dev/null

  mkdir -p "$DATA_DIR"
  for f in "$src"/*.db "$src"/state.json; do
    [[ -f "$f" ]] || continue
    cp -p "$f" "$DATA_DIR/" || cp "$f" "$DATA_DIR/"
  done
  chmod 700 "$DATA_DIR" 2>/dev/null || true
  chmod 600 "$DATA_DIR"/state.json "$DATA_DIR"/*.db 2>/dev/null || true
  log "restored. restart the app to pick it up."
}

case "${1:-}" in
  --list) list_snapshots ;;
  --restore)
    [[ $# -ge 2 ]] || die "usage: $0 --restore <name>"
    restore_snapshot "$2"
    ;;
  "") take_snapshot ;;
  *) take_snapshot "$1" ;;
esac
