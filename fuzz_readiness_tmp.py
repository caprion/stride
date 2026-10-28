#!/usr/bin/env python3
"""Differential fuzz for the readiness score against the reference."""
from __future__ import annotations

import json
import random
import sys
from pathlib import Path
from types import SimpleNamespace

REPO = Path("/home/sumit/stride")
sys.path.insert(0, str(REPO / "backend"))

from app.services import readiness as rs  # noqa: E402


def random_rows(rng: random.Random, day: str) -> list[SimpleNamespace]:
    rows = []
    base = 20
    for offset in range(-40, 2):
        date = f"2026-02-{base + offset:02d}" if base + offset <= 28 else f"2026-03-{base + offset - 28:02d}"
        if offset == 0:
            date = day
        rows.append(SimpleNamespace(
            date=date,
            sleep_s=rng.choice([None, 0.0, 18000.0, 25200.0, 27000.0, 27900.0]),
            sleep_score=rng.choice([None, 0.0, 40, 70, 85, 100]),
            hrv_ms=rng.choice([None, 0.0, 30.0, 36.0, 55.0, 58.0]),
            resting_hr=rng.choice([None, 0.0, 44.0, 48.0, 56.0]),
            stress_avg=rng.choice([None, 0.0, 22.0, 60.0, 99.0]),
            body_battery_avg=rng.choice([None, 0.0, 29.0, 78.0]),
            acute_load=rng.choice([None, 0.0, 300.0, 320.0]),
            chronic_load=rng.choice([None, 0.0, 300.0]),
        ))
    return rows


def main() -> int:
    rng = random.Random(int(sys.argv[2]) if len(sys.argv) > 2 else 7)
    cases = []
    for _ in range(1500):
        day = "2026-03-01"
        rows = random_rows(rng, day)
        row = next((r for r in rows if r.date == day), None)
        baseline = rs.baselines(rows, day)
        payload = rs._derive(day, row, baseline)
        cases.append({
            "day": day,
            "rows": [
                {
                    "date": r.date,
                    "sleep_s": r.sleep_s,
                    "sleep_score": r.sleep_score,
                    "hrv_ms": r.hrv_ms,
                    "resting_hr": r.resting_hr,
                    "stress_avg": r.stress_avg,
                    "body_battery_avg": r.body_battery_avg,
                    "acute_load": r.acute_load,
                    "chronic_load": r.chronic_load,
                }
                for r in rows
            ],
            "expected_baselines": baseline,
            "expected": payload,
        })
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/readiness_fuzz.json")
    out.write_text(json.dumps(cases, indent=1), encoding="utf-8")
    print(f"wrote {len(cases)} cases to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
