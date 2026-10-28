#!/usr/bin/env python3
"""Differential fuzz for the plan engine against the reference.

Builds a random plan (sessions, activities with laps and fingerprints, and
physiology rows) in an in-memory database, runs the reference `match_sessions`
and `adherence`, and writes the inputs and the reference outputs to JSON.
"""
from __future__ import annotations

import json
import random
import sys
from datetime import date, datetime, timedelta
from pathlib import Path

REPO = Path("/home/sumit/stride")
sys.path.insert(0, str(REPO / "backend"))

from app.db import Base  # noqa: E402
from app.models import Activity, ActivityLap  # noqa: E402
from app.models_phase2 import DailyPhysiology, PlanSession, RunFingerprint  # noqa: E402
from app.services import fingerprint as fp  # noqa: E402
from app.services import plan_engine  # noqa: E402
from sqlalchemy import create_engine  # noqa: E402
from sqlalchemy.orm import Session  # noqa: E402

SPORTS = ["running", "trail_running", "treadmill_running", "walking", "cycling", "strength_training"]
SESSION_TYPES = ["easy", "tempo", "interval", "long", "recovery", "strength", "rest", "race"]


def build_case(rng: random.Random) -> dict:
    start = date(2026, 3, 2)
    sessions = []
    for index in range(rng.randint(1, 8)):
        day = start + timedelta(days=rng.randint(0, 16))
        sessions.append({
            "id": index + 1,
            "date": day.isoformat(),
            "session_type": rng.choice(SESSION_TYPES),
            "title": rng.choice(["Easy", "Tempo", "Long run", "Rest day", "Intervals", None]),
            "details": None,
            "target_distance_m": rng.choice([None, 0.0, 400.0, 900.0, 1900.0, 5000.0]),
            "pace_low_s_km": rng.choice([None, 200.0, 290.0, 330.0]),
            "pace_high_s_km": rng.choice([None, 240.0, 320.0, 420.0]),
        })

    activities = []
    for index in range(rng.randint(0, 8)):
        day = start + timedelta(days=rng.randint(-3, 18))
        sport = rng.choice(SPORTS)
        distance = rng.choice([0.0, 232.1, 443.0, 900.0, 1980.0, 5000.0, 15000.0])
        moving = rng.choice([0.0, 60.0, 150.0, 300.0, 1500.0, 20000.0])
        laps = []
        for lap_index in range(rng.randint(0, 4)):
            laps.append({
                "lap_index": lap_index + 1,
                "distance_m": rng.choice([0.0, 300.0, 400.0, 900.0, 1500.0]),
                "duration_s": rng.choice([0.0, 60.0, 300.0]),
            })
        activities.append({
            "id": index + 1,
            "name": rng.choice(["Morning easy", "Tempo run", "Fartlek repeats", "Intervals", "Strength session", "Long run"]),
            "local_date": day.isoformat(),
            "sport_type": sport,
            "distance_m": distance,
            "moving_time_s": moving,
            "laps": laps,
        })

    physiology = []
    for offset in range(-30, 1):
        day = start + timedelta(days=offset)
        row = {"date": day.isoformat()}
        if rng.random() < 0.9:
            row["sleep_s"] = rng.choice([18000.0, 25200.0, 27000.0, 27900.0])
        if rng.random() < 0.8:
            row["hrv_ms"] = rng.choice([30.0, 36.0, 55.0, 58.0])
        if rng.random() < 0.8:
            row["resting_hr"] = rng.choice([44.0, 48.0, 56.0])
        physiology.append(row)

    return {
        "range": [start.isoformat(), (start + timedelta(days=18)).isoformat()],
        "sessions": sessions,
        "activities": activities,
        "physiology": physiology,
    }


def run_case(case: dict) -> dict:
    engine = create_engine("sqlite://")
    Base.metadata.create_all(engine)
    session = Session(engine, expire_on_commit=False)
    try:
        for spec in case["sessions"]:
            session.add(PlanSession(**spec))
        for spec in case["physiology"]:
            session.add(DailyPhysiology(**spec))
        for spec in case["activities"]:
            laps = spec["laps"]
            activity = Activity(
                id=spec["id"],
                name=spec["name"],
                local_date=spec["local_date"],
                sport_type=spec["sport_type"],
                distance_m=spec["distance_m"],
                moving_time_s=spec["moving_time_s"],
                elapsed_time_s=spec["moving_time_s"],
                source="garmin",
                external_id=f"fuzz-{spec['id']}",
                start_time=datetime.fromisoformat(f"{spec['local_date']}T06:00:00+00:00"),
            )
            session.add(activity)
            for lap in laps:
                session.add(ActivityLap(activity_id=spec["id"], **lap))
        session.flush()
        # Fingerprints, so the cause can use the cadence variability.
        for activity in session.query(Activity).all():
            if activity.sport_type in plan_engine.RUN_SPORTS:
                try:
                    fp.compute_fingerprint(session, activity.id)
                except Exception:
                    pass
        session.flush()
        matches = plan_engine.match_sessions(session, *case["range"])
        adherence = plan_engine.adherence(session, *case["range"])
    finally:
        session.close()
        engine.dispose()
    rows = []
    for match in matches:
        rows.append({
            "session": match["session"],
            "activity": match["activity"],
            "verdict": match["verdict"],
            "distance_hit": match["distance_hit"],
            "pace_band_hit": match["pace_band_hit"],
            "distance_delta_m": match["distance_delta_m"],
            "pace_delta_s_km": match["pace_delta_s_km"],
            "adherence_score": match["adherence_score"],
            "cause": match["cause"],
            "cause_confidence": match["cause_confidence"],
        })
    return {"rows": rows, "adherence": adherence}


def main() -> int:
    rng = random.Random(int(sys.argv[2]) if len(sys.argv) > 2 else 11)
    fp.state.garmin_athlete = {}
    cases = []
    for _ in range(400):
        case = build_case(rng)
        result = run_case(case)
        cases.append({"case": case, "expected": result})
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/plan_fuzz.json")
    out.write_text(json.dumps(cases, indent=1), encoding="utf-8")
    print(f"wrote {len(cases)} cases to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
