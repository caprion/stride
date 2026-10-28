#!/usr/bin/env bash
# Ask DeepSeek a hard question.
#
# The fallback for when the local model's window is spent, or when a problem
# needs a second opinion. It uses the Anthropic-compatible endpoint, so the
# key comes from the same Key Vault as everything else and is never written
# to disk.
#
#   ./scripts/ask_deepseek.sh "why does this panic?"
#   ./scripts/ask_deepseek.sh "review this" file.rs
#   ./scripts/ask_deepseek.sh --diff "review the change"
#
# The key is read from Azure Key Vault on every call, so it never lands in a
# file, a shell history or a log.
set -uo pipefail

MODEL="${DEEPSEEK_MODEL:-deepseek-v4-pro[1m]}"
ENDPOINT="https://api.deepseek.com/anthropic/v1/messages"

usage() {
  cat <<'EOF'
usage: ask_deepseek.sh [--diff] [--model NAME] <question> [file ...]

  --diff        include the working tree's git diff in the prompt
  --model NAME  the model to use (default: deepseek-v4-pro[1m])
EOF
}

WITH_DIFF=0
FILES=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --diff) WITH_DIFF=1; shift ;;
    --model) MODEL="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) break ;;
  esac
done

QUESTION="${1:-}"
[[ $# -gt 0 ]] && shift
FILES=("$@")

if [[ -z "$QUESTION" ]]; then
  usage
  exit 2
fi

# The key comes from the environment if it is there, and from Key Vault
# otherwise. Never from a file in the repository.
KEY="${DEEPSEEK_API_KEY:-}"
if [[ -z "$KEY" ]]; then
  if ! command -v az >/dev/null 2>&1; then
    echo "No key: set DEEPSEEK_API_KEY or install the Azure CLI." >&2
    exit 3
  fi
  KEY="$(az keyvault secret show --vault-name "${VAULT_NAME:-claw-agent}" \
    --name DEEPSEEK-API-KEY --query value -o tsv 2>/dev/null)"
fi
if [[ -z "$KEY" ]]; then
  echo "Could not read DEEPSEEK-API-KEY from Key Vault." >&2
  exit 3
fi

BODY_FILE="$(mktemp)"
trap 'rm -f "$BODY_FILE"' EXIT

{
  echo "$QUESTION"
  echo
  echo "--- context ---"
  for file in "${FILES[@]}"; do
    [[ -f "$file" ]] || continue
    echo
    echo "### $file"
    cat "$file"
  done
  if [[ "$WITH_DIFF" == "1" ]]; then
    echo
    echo "### git diff"
    git diff 2>/dev/null
    git diff --cached 2>/dev/null
  fi
} > "$BODY_FILE"

PAYLOAD="$(python3 - "$BODY_FILE" "$MODEL" <<'PY'
import json, sys
content = open(sys.argv[1], encoding="utf-8", errors="replace").read()
print(json.dumps({
    "model": sys.argv[2],
    "max_tokens": 4096,
    "messages": [{"role": "user", "content": content}],
}))
PY
)"

RESPONSE="$(curl -sS -m 600 "$ENDPOINT" \
  -H "x-api-key: $KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  --data "$PAYLOAD" 2>&1)"

python3 - "$RESPONSE" <<'PY'
import json, sys
raw = sys.argv[1]
try:
    data = json.loads(raw)
except json.JSONDecodeError:
    print(raw[:2000])
    raise SystemExit(1)
if "content" in data:
    for block in data["content"]:
        if block.get("type") == "text":
            print(block["text"])
else:
    print(json.dumps(data)[:2000])
PY
