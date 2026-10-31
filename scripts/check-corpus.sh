#!/usr/bin/env bash
# Corpus integrity: every FIT file's SHA-256 must match the frozen value in
# fixtures/golden/cases.json. This is the slice-0 half of "golden parity" — it
# proves the corpus was not modified. Full engine parity (comparing computed
# numbers against the golden JSON) arrives with the differ in Slice 2.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v jq >/dev/null 2>&1; then
  echo "check-corpus: jq is required" >&2
  exit 2
fi

fail=0
while IFS=$'\t' read -r file sha; do
  basename=$(basename "$file")
  fit="fixtures/fit/$basename"
  if [[ ! -f "$fit" ]]; then
    echo "check-corpus: missing FIT file $fit" >&2
    fail=1
    continue
  fi
  actual=$(sha256sum "$fit" | cut -d' ' -f1)
  if [[ "$actual" != "$sha" ]]; then
    echo "check-corpus: $fit sha256 $actual != $sha" >&2
    fail=1
  fi
done < <(jq -r '.cases[] | [.file, .sha256] | @tsv' fixtures/golden/cases.json)

if [[ "$fail" -eq 0 ]]; then
  echo "check-corpus: all $(jq '.cases | length' fixtures/golden/cases.json) FIT files match their frozen SHA-256"
fi
exit "$fail"
