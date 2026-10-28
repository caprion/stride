#!/usr/bin/env python3
"""Differential fuzz for the fingerprint: random activities, reference outputs.

Runs the *reference* implementation (`backend/app/services/fingerprint.py`) over
randomly generated activities and writes the inputs and the reference outputs
to a JSON file. The Rust side then recomputes the same inputs and compares.

This is not a golden fixture; it is a way of covering the edges the nine
golden cases do not reach (missing laps, zero cadences, walked laps, activity
zone rows, a heart-rate stream fallback, and so on).
"""
from __future__ import annotations

import json
import random
import sys
from pathlib import Path
from types import SimpleNamespace

REPO = Path(__file__).resolve().parent
sys.path.insert(0, str(REPO / "backend"))

from app.services import fingerprint as fp  # noqa: E402


def random_activity(rng: random.Random) -> SimpleNamespace:
    lap_count = rng.choice([0, 1, 2, 3, 4, 5, 6, 7, 9])
    laps = []
    for index in range(lap_count):
        distance = rng.choice([0.0, 120.0, 380.0, 400.0, 900.0, 1500.0])
        duration = rng.choice([0.0, 30.0, 95.0, 300.0])
        laps.append(SimpleNamespace(
            lap_index=index + 1,
            distance_m=distance,
            duration_s=duration,
            avg_speed_ms=rng.choice([None, 0.0, 2.0, 2.2, 3.1, 4.4]),
            avg_hr=rng.choice([None, 0.0, 120, 138, 151, 168]),
            max_hr=rng.choice([None, 130, 155, 171]),
            avg_cadence=rng.choice([None, 0.0, 90.0, 124.0, 124.9, 125.0, 156.0, 159.1, 171.0]),
            stride_length_m=rng.choice([None, 0.0, 0.9, 1.2, 3.0, 3.4]),
        ))
    zone_rows = []
    if rng.random() < 0.3:
        for zone_no in rng.sample([1, 2, 3, 4, 5], rng.randint(1, 3)):
            zone_rows.append(SimpleNamespace(zone_no=zone_no, time_s=rng.choice([0.0, 12.5, 99.0])))
    streams = []
    if rng.random() < 0.3:
        streams.append(SimpleNamespace(
            kind="hr",
            values=[rng.choice([None, 120, 140, 155]) for _ in range(rng.randint(0, 5))],
        ))
    return SimpleNamespace(
        laps=laps,
        zones=zone_rows,
        streams=streams,
        avg_hr=rng.choice([None, 0.0, 142, 155]),
        max_hr=rng.choice([None, 0.0, 155, 171]),
        distance_m=rng.choice([0.0, 443.0, 1980.0, 5000.0]),
        running_time_s=rng.choice([None, 0.0, 139.0, 599.0]),
        walking_time_s=rng.choice([None, 0.0, 10.0]),
        stopped_time_s=rng.choice([None, 0.0, 20.0]),
        source=rng.choice(["garmin_fit", "garmin", "demo"]),
    )


def main() -> int:
    rng = random.Random(int(sys.argv[2]) if len(sys.argv) > 2 else 20260311)
    fp.state.garmin_athlete = {}
    cases = []
    for _ in range(1500):
        activity = random_activity(rng)
        reference = rng.choice([120.0, 155.0, 190.0, 200.0])
        payload = fp._derive(activity, reference)
        cases.append({
            "reference": reference,
            "activity": {
                "avg_hr": activity.avg_hr,
                "max_hr": activity.max_hr,
                "distance_m": activity.distance_m,
                "running_time_s": activity.running_time_s,
                "walking_time_s": activity.walking_time_s,
                "stopped_time_s": activity.stopped_time_s,
                "source": activity.source,
                "laps": [
                    {
                        "lap_index": lap.lap_index,
                        "distance_m": lap.distance_m,
                        "duration_s": lap.duration_s,
                        "avg_speed_ms": lap.avg_speed_ms,
                        "avg_hr": lap.avg_hr,
                        "max_hr": lap.max_hr,
                        "avg_cadence": lap.avg_cadence,
                        "stride_length_m": lap.stride_length_m,
                    }
                    for lap in activity.laps
                ],
                "zones": [{"zone_no": z.zone_no, "time_s": z.time_s} for z in activity.zones],
                "streams": [
                    {"kind": s.kind, "values": s.values} for s in activity.streams
                ],
            },
            "expected": payload,
        })
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/fingerprint_fuzz.json")
    out.write_text(json.dumps(cases, indent=1), encoding="utf-8")
    print(f"wrote {len(cases)} cases to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
