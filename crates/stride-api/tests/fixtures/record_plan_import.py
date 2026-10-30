#!/usr/bin/env python3
"""Record the Python importer's answers, so the Rust port can be measured against them.

The plan-import fixture is the counterpart of the API golden file: it
freezes what the *importer* makes of a document, produced from the
reference implementation, so the Rust port can be proved equal to it rather
than believed equal to it.

The documents live in the fixture itself. Add a case with an empty
``expected`` and run this script; it fills the expectation in from the
reference and writes the file back.

Run it from the repository root::

    backend/.venv/bin/python crates/stride-api/tests/fixtures/record_plan_import.py

It writes ``crates/stride-api/tests/fixtures/plan_import.json``.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
OUT = HERE / "plan_import.json"

sys.path.insert(0, str(ROOT / "backend"))

from app.ingest.plan_importer import import_plan  # noqa: E402


def main() -> int:
    data = json.loads(OUT.read_text())
    cases = []
    for case in data["cases"]:
        try:
            expected = import_plan(case["content"], case["format"], None)
            error = None
        except Exception as exc:  # a document the reference refuses
            expected = case.get("expected")
            error = str(exc)
        cases.append(
            {
                "name": case["name"],
                "format": case["format"],
                "content": case["content"],
                "expected": expected,
                "error": error,
            }
        )
    OUT.write_text(json.dumps({"cases": cases}, indent=1))
    print(f"wrote {OUT} ({len(cases)} cases)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
