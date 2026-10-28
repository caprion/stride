#!/usr/bin/env python3
"""Differential fuzz for the comparison profile against the reference."""
from __future__ import annotations

import json
import random
import sys
from pathlib import Path
from types import SimpleNamespace

REPO = Path("/home/sumit/stride")
sys.path.insert(0, str(REPO / "backend"))

from app.services import comparison  # noqa: E402


def random_stream(rng: random.Random, kind: str, count: int):
    values = []
    for _ in range(count):
        roll = rng.random()
        if roll < 0.1:
            values.append(None)
        elif kind == "hr":
            values.append(rng.choice([120, 140, 155, 171]))
        elif kind == "cadence":
            values.append(rng.choice([150.0, 156.0, 171.0]))
        elif kind == "pace":
            values.append(rng.choice([250.0, 300.0, 400.0]))
        else:
            values.append(rng.choice([0.0, 12.5, 60.0]))
    seconds = [float(index) for index in range(count)]
    return SimpleNamespace(kind=kind, seconds=seconds, values=values)


def main() -> int:
    rng = random.Random(int(sys.argv[2]) if len(sys.argv) > 2 else 5)
    cases = []
    for _ in range(300):
        count = rng.randint(0, 40)
        kinds = ["distance"] + rng.sample(["hr", "cadence", "pace", "elevation"], rng.randint(0, 4))
        streams = []
        for kind in kinds:
            size = count if kind == "distance" else rng.randint(0, count + 3)
            if kind == "distance":
                size = count
            streams.append(random_stream(rng, kind, size))
        # The distance stream has to increase for the profile to exist.
        distance = next((s for s in streams if s.kind == "distance"), None)
        if distance is not None:
            value = 0.0
            for index, item in enumerate(distance.values):
                value += rng.choice([0.0, 1.0, 3.5, 10.0])
                distance.values[index] = value
        activity = SimpleNamespace(streams=streams)
        profile = comparison.profile_for(activity)
        fingerprint = SimpleNamespace(
            hr_avg=rng.choice([None, 142.0]),
            hr_drift_bpm=rng.choice([None, 3.0]),
            pace_drift_pct=rng.choice([None, -1.5]),
            decoupling_pct=rng.choice([None, 2.5]),
            cadence_avg=rng.choice([None, 156.0]),
            cadence_cv=rng.choice([None, 1.5]),
            efficiency_first=rng.choice([None, 3.2]),
            form_score=rng.choice([None, 91.0]),
            running_time_s=rng.choice([None, 299.0]),
            walking_time_s=rng.choice([None, 10.0]),
            stopped_time_s=rng.choice([None, 20.0]),
        )
        activity.distance_m = rng.choice([0.0, 443.0, 1980.0])
        activity.moving_time_s = rng.choice([0.0, 150.0, 600.0])
        activity.elevation_gain_m = rng.choice([None, 0.0, 60.0])
        activity.avg_hr = rng.choice([None, 142])
        values = {p["key"]: comparison._value_for(p, activity, fingerprint) for p in comparison.PARAMETERS}
        cases.append({
            "streams": [
                {"kind": s.kind, "seconds": s.seconds, "values": s.values} for s in streams
            ],
            "distance_m": activity.distance_m,
            "moving_time_s": activity.moving_time_s,
            "elevation_gain_m": activity.elevation_gain_m,
            "avg_hr": activity.avg_hr,
            "fingerprint": {
                "hr_avg": fingerprint.hr_avg, "hr_drift_bpm": fingerprint.hr_drift_bpm,
                "pace_drift_pct": fingerprint.pace_drift_pct, "decoupling_pct": fingerprint.decoupling_pct,
                "cadence_avg": fingerprint.cadence_avg, "cadence_cv": fingerprint.cadence_cv,
                "efficiency_first": fingerprint.efficiency_first, "form_score": fingerprint.form_score,
                "running_time_s": fingerprint.running_time_s, "walking_time_s": fingerprint.walking_time_s,
                "stopped_time_s": fingerprint.stopped_time_s,
            },
            "expected": profile,
            "expected_values": values,
        })
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/comparison_fuzz.json")
    out.write_text(json.dumps(cases, indent=1), encoding="utf-8")
    print(f"wrote {len(cases)} cases to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
