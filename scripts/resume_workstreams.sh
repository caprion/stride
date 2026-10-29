#!/usr/bin/env bash
# Resume the Stride workstreams that have stalled.
#
# A thread is idle either because it finished or because it is waiting for a
# concurrency slot or a host to free up. The finished ones are removed from
# the list by hand as they report; this nudges whatever is left.
#
# It prints nothing when there is nothing to do, so a quiet tick is recorded
# as a skipped silent run rather than as noise.
set -uo pipefail

REPO=/home/sumit/stride
cd "$REPO" || exit 0

LIST=".agent-prompts/active-threads.txt"
STAMPS=".agent-prompts/nudged.txt"
[ -f "$LIST" ] || exit 0
mkdir -p .agent-prompts
touch "$STAMPS"

now=$(date +%s)
resumed=0
skipped=0

while read -r id; do
  id="${id%%#*}"
  id="$(echo "$id" | tr -d '[:space:]')"
  [ -n "$id" ] || continue

  # Do not pile messages onto a thread that was nudged recently - it may
  # simply be working through the queue.
  last="$(grep -E "^$id " "$STAMPS" 2>/dev/null | tail -1 | awk '{print $2}')"
  if [ -n "$last" ] && [ $((now - last)) -lt 3300 ]; then
    skipped=$((skipped + 1))
    continue
  fi

  status="$(bb thread show "$id" --json 2>/dev/null \
    | python3 -c 'import json,sys; print(json.load(sys.stdin).get("status",""))' 2>/dev/null)"
  case "$status" in
    idle|queued|pending|error|paused|blocked|"")
      if bb thread tell "$id" \
        "Continue where you left off. Re-read your own last report, check whether anything in your scope is still unfinished, and finish it. Run whatever gate or test command your own project uses before you report, and report the numbers - not adjectives." \
        >/dev/null 2>&1; then
        resumed=$((resumed + 1))
        echo "$id $now" >> "$STAMPS"
      fi
      ;;
    *)
      skipped=$((skipped + 1))
      ;;
  esac
done < "$LIST"

if [ "$resumed" -gt 0 ]; then
  echo "resumed $resumed stalled workstream(s); $skipped left alone"
fi
exit 0
