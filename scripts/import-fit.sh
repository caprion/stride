#!/usr/bin/env bash
# Import Garmin .FIT files into Stride from the command line.
#
#   ./scripts/import-fit.sh ~/Downloads/*.fit
#   ./scripts/import-fit.sh --watch ~/fit-inbox
#
# With --watch, the folder is polled every 30 seconds and new files are
# imported as they appear (useful if you sync FIT files off the watch).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="${STRIDE_API:-http://127.0.0.1:8000}/api/v1/auth/garmin/fit-import"

log() { printf '\033[38;5;208m▸\033[0m %s\n' "$*"; }

import_files() {
  local files=("$@")
  if [[ ${#files[@]} -eq 0 ]]; then
    log "no .fit files found"
    return 0
  fi
  local args=()
  for f in "${files[@]}"; do args+=(-F "files=@$f"); done
  local response
  response="$(curl -fsS -X POST "$API" "${args[@]}")" || {
    log "import failed — is Stride running?"
    return 1
  }
  echo "$response" | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(f\"imported {d['imported']} activity/activities\")
for a in d['activities']:
    print(f\"  · {a['date']}  {a['distance_m']/1000:.2f} km  ({a['name']})\")
for e in d.get('errors', []):
    print(f\"  ! {e['filename']}: {e['error']}\")
"
}

if [[ "${1:-}" == "--watch" ]]; then
  dir="${2:?usage: import-fit.sh --watch <dir>}"
  log "watching $dir every 30s (ctrl-c to stop)"
  declare -A seen=()
  while true; do
    for f in "$dir"/*.fit; do
      [[ -e "$f" ]] || continue
      [[ -n "${seen[$f]:-}" ]] && continue
      seen[$f]=1
      log "new file: $(basename "$f")"
      import_files "$f" || true
    done
    sleep 30
  done
else
  import_files "$@"
fi
