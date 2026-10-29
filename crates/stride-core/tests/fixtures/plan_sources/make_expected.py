#!/usr/bin/env python3
"""Regenerate the plan-source golden file from the Python reference.

    python3 make_expected.py

The Rust test in ``crates/stride-core/tests/plan_sources.rs`` compares the
Rust port against the file this writes, so the two implementations are
held to the same numbers. The Python is the reference: it is the
implementation the Rust port is measured against, exactly as every other
crate in this workspace is.

The inputs are ``inputs.json`` — a synthetic athlete's history, and the
scheduled Garmin workouts. They are static, so the outputs are too.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
sys.path.insert(0, str(REPO / "backend"))

from app.services.plan_sources import (  # noqa: E402
    infer_plan,
    workouts_plan,
)


def main() -> None:
    inputs = json.loads((HERE / "inputs.json").read_text())

    expected = {
        "inferred": infer_plan(
            inputs["inferred"]["activities"],
            inputs["inferred"]["from"],
            inputs["inferred"]["to"],
        ),
        "inferred_thin": infer_plan(
            inputs["inferred_thin"]["activities"],
            inputs["inferred_thin"]["from"],
            inputs["inferred_thin"]["to"],
        ),
        "workouts": workouts_plan(
            inputs["workouts"]["scheduled"], inputs["workouts"]["details"]
        ),
    }

    path = HERE / "expected.json"
    path.write_text(json.dumps(expected, indent=2, sort_keys=True) + "\n")
    print(f"wrote {path}")


if __name__ == "__main__":
    main()
