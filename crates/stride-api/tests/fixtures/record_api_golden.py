#!/usr/bin/env python3
"""Record the Python app's answers, so the Rust router can be measured against them.

The API golden file is the counterpart of the domain golden files in
``backend/tests/golden``: it freezes what the *endpoints* answer, produced
from the reference implementation, so the Rust port can be proved equal to it
rather than believed equal to it.

What it records:

* ``rows`` — the database, in the seam format the Rust router reads: one
  JSON object per row, keyed by column name.
* ``cases`` — the answer of each sampled route: the status and the body.

Run it from the repository root::

    backend/.venv/bin/python crates/stride-api/tests/fixtures/record_api_golden.py

It writes ``crates/stride-api/tests/fixtures/api_golden.json``.
"""

from __future__ import annotations

import json
import os
import shutil
import sys
import tempfile
from datetime import date, datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
OUT = HERE / "api_golden.json"
FIXTURES = ROOT / "backend" / "tests" / "golden" / "fixtures"


def main() -> int:
    data_dir = Path(tempfile.mkdtemp(prefix="stride-api-golden-"))
    os.environ["STRIDE_DATA_DIR"] = str(data_dir)
    os.environ["STRIDE_KEYVAULT_ENABLED"] = "false"
    os.environ["STRIDE_DISABLE_AUTH"] = "true"
    os.environ["STRIDE_SEED_DEMO"] = "false"
    os.environ["TZ"] = "UTC"
    sys.path.insert(0, str(ROOT / "backend"))

    from fastapi.testclient import TestClient

    from app.db import SessionLocal, init_db
    from app.ingest.fit_importer import parse_fit
    from app.main import app
    from app.models import Activity, ActivityLap, ActivityStream, DailyMetric, HrZone
    from app.models_phase2 import DailyPhysiology, PlanBlock, PlanPhase, PlanSession, PlanWeek
    from app.services.persistence import store_activity

    init_db()
    db = SessionLocal()
    seed_database(db, parse_fit, store_activity)
    rows = dump_rows(db)
    db.close()

    cases = record_cases(TestClient(app))

    payload = {
        "recorded_at": datetime.now(timezone.utc).isoformat(),
        "today": date.today().isoformat(),
        "rows": rows,
        "cases": cases,
    }
    OUT.write_text(json.dumps(payload, indent=1, sort_keys=False, default=json_default))
    shutil.rmtree(data_dir, ignore_errors=True)
    print(f"wrote {OUT} with {len(cases)} cases")
    return 0


def json_default(value):
    """Datetimes and dates as ISO strings, the way the API renders them."""
    if isinstance(value, (datetime, date)):
        return value.isoformat()
    raise TypeError(f"Object of type {type(value).__name__} is not JSON serializable")


def seed_database(db, parse_fit, store_activity) -> None:
    """Seed the database with the golden FIT fixtures, the physiology rows and a plan."""
    from app.models_phase2 import DailyPhysiology, PlanBlock, PlanPhase, PlanSession, PlanWeek
    for path in sorted(FIXTURES.glob("*.fit")):
        activity = parse_fit(path.read_bytes(), filename=path.name)
        store_activity(db, activity)
    db.commit()

    scenarios = json.loads(
        (ROOT / "backend" / "tests" / "golden" / "scenarios" / "readiness.json").read_text()
    )
    rows: dict[str, dict] = {}
    for scenario in scenarios["scenarios"]:
        for row in scenario["physiology"]:
            rows.setdefault(row["date"], {}).update(row)
    for day, row in sorted(rows.items()):
        db.add(DailyPhysiology(date=day, **{
            key: value for key, value in row.items() if key != "date"
        }))
    db.commit()

    plan = json.loads(
        (ROOT / "backend" / "tests" / "golden" / "scenarios" / "plan.json").read_text()
    )
    block = PlanBlock(**plan["block"])
    db.add(block)
    db.flush()
    phase = PlanPhase(block_id=block.id, **plan["phase"])
    db.add(phase)
    db.flush()
    week = PlanWeek(phase_id=phase.id, **plan["week"])
    db.add(week)
    db.flush()
    for session in plan["sessions"]:
        db.add(PlanSession(week_id=week.id, block_id=block.id, **session))
    db.commit()


def dump_rows(db) -> dict:
    """The database, in the seam format the Rust router reads."""
    from app.models import Activity as ActivityModel
    from app.models import DailyMetric
    from app.models_phase2 import (
        DailyPhysiology,
        PlanBlock,
        PlanPhase,
        PlanSession,
        PlanWeek,
    )

    activities = []
    laps = []
    streams = []
    zones = []
    for activity in db.query(ActivityModel).order_by(ActivityModel.id).all():
        activities.append({
            "id": activity.id,
            "source": activity.source,
            "external_id": activity.external_id,
            "name": activity.name,
            "sport_type": activity.sport_type,
            "description": activity.description,
            "start_time": activity.start_time.isoformat() if activity.start_time else None,
            "local_date": activity.local_date,
            "day_of_week": activity.day_of_week,
            "distance_m": activity.distance_m,
            "moving_time_s": activity.moving_time_s,
            "elapsed_time_s": activity.elapsed_time_s,
            "elevation_gain_m": activity.elevation_gain_m,
            "avg_hr": activity.avg_hr,
            "max_hr": activity.max_hr,
            "calories": activity.calories,
            "avg_cadence": activity.avg_cadence,
            "max_cadence": activity.max_cadence,
            "avg_speed_ms": activity.avg_speed_ms,
            "max_speed_ms": activity.max_speed_ms,
            "running_time_s": activity.running_time_s,
            "walking_time_s": activity.walking_time_s,
            "stopped_time_s": activity.stopped_time_s,
            "running_distance_m": activity.running_distance_m,
            "perceived_exertion": activity.perceived_exertion,
            "device": activity.device,
            "gear": activity.gear,
            "is_trail": activity.is_trail,
            "is_commute": activity.is_commute,
            "geo_points": activity.geo_points,
            "lap_count": len(activity.laps or []),
        })
        for lap in activity.laps:
            laps.append({
                "activity_id": activity.id,
                "lap_index": lap.lap_index,
                "distance_m": lap.distance_m,
                "duration_s": lap.duration_s,
                "avg_speed_ms": lap.avg_speed_ms,
                "max_speed_ms": lap.max_speed_ms,
                "avg_hr": lap.avg_hr,
                "max_hr": lap.max_hr,
                "avg_cadence": lap.avg_cadence,
                "max_cadence": lap.max_cadence,
                "stride_length_m": lap.stride_length_m,
                "elevation_gain_m": lap.elevation_gain_m,
                "elevation_loss_m": lap.elevation_loss_m,
                "start_elevation_m": lap.start_elevation_m,
                "end_elevation_m": lap.end_elevation_m,
                "running_distance_m": lap.running_distance_m,
                "elapsed_s": lap.elapsed_s,
            })
        for stream in activity.streams:
            streams.append({
                "activity_id": activity.id,
                "kind": stream.kind,
                "seconds": stream.seconds or [],
                "values": stream.values or [],
                "label": stream.label,
                "unit": stream.unit,
            })
        for zone in activity.zones:
            zones.append({
                "activity_id": activity.id,
                "zone_no": zone.zone_no,
                "name": zone.name,
                "low_bpm": zone.low_bpm,
                "high_bpm": zone.high_bpm,
                "time_s": zone.time_s,
            })

    physiology = []
    for row in db.query(DailyPhysiology).order_by(DailyPhysiology.date).all():
        physiology.append({
            column.name: getattr(row, column.name)
            for column in DailyPhysiology.__table__.columns
        })

    daily = []
    for row in db.query(DailyMetric).order_by(DailyMetric.date).all():
        daily.append({
            column.name: getattr(row, column.name)
            for column in DailyMetric.__table__.columns
        })

    blocks = []
    phases = []
    weeks = []
    sessions = []
    for block in db.query(PlanBlock).order_by(PlanBlock.id).all():
        blocks.append({
            column.name: getattr(block, column.name)
            for column in PlanBlock.__table__.columns
            if column.name != "created_at"
        })
    for phase in db.query(PlanPhase).order_by(PlanPhase.id).all():
        phases.append({
            column.name: getattr(phase, column.name)
            for column in PlanPhase.__table__.columns
        })
    for week in db.query(PlanWeek).order_by(PlanWeek.id).all():
        weeks.append({
            column.name: getattr(week, column.name)
            for column in PlanWeek.__table__.columns
        })
    for session in db.query(PlanSession).order_by(PlanSession.id).all():
        sessions.append({
            column.name: getattr(session, column.name)
            for column in PlanSession.__table__.columns
        })

    return {
        "activities": activities,
        "laps": laps,
        "streams": streams,
        "zones": zones,
        "physiology": physiology,
        "daily_metrics": daily,
        "plan_blocks": blocks,
        "plan_phases": phases,
        "plan_weeks": weeks,
        "plan_sessions": sessions,
    }


# The routes the golden file records. Each entry is (method, path, body).
ROUTES = [
    ("GET", "/api/v1/health", None),
    ("GET", "/api/v1/health/daily", None),
    ("GET", "/api/v1/auth/status", None),
    ("GET", "/api/v1/auth/pair-code", None),
    ("GET", "/api/v1/auth/devices", None),
    ("GET", "/api/v1/activities", None),
    ("GET", "/api/v1/activities?per_page=5&page=1", None),
    ("GET", "/api/v1/activities?sport=running", None),
    ("GET", "/api/v1/activities?sort=distance", None),
    ("GET", "/api/v1/activities/1", None),
    ("GET", "/api/v1/activities/1/geo", None),
    ("GET", "/api/v1/activities/1/streams/hr", None),
    ("GET", "/api/v1/activities/999", None),
    ("GET", "/api/v1/stats/summary", None),
    ("GET", "/api/v1/stats/weekly", None),
    ("GET", "/api/v1/stats/monthly", None),
    ("GET", "/api/v1/stats/pace-trends", None),
    ("GET", "/api/v1/stats/hr-zones", None),
    ("GET", "/api/v1/stats/personal-records", None),
    ("GET", "/api/v1/stats/heatmap", None),
    ("GET", "/api/v1/stats/elevation", None),
    ("GET", "/api/v1/calendar?year=2026&month=3", None),
    ("GET", "/api/v1/sync/status", None),
    ("GET", "/api/v1/sync/runs", None),
    ("GET", "/api/v1/sync/history", None),
    ("GET", "/api/v1/network/status", None),
    ("GET", "/api/v1/plan/blocks", None),
    ("GET", "/api/v1/plan/blocks/1", None),
    ("GET", "/api/v1/plan/current", None),
    ("GET", "/api/v1/plan/week?date=2026-03-04", None),
    ("GET", "/api/v1/plan/results", None),
    ("GET", "/api/v1/plan/adherence", None),
    ("GET", "/api/v1/plan/today", None),
    ("GET", "/api/v1/physiology/daily", None),
    ("GET", "/api/v1/physiology/daily?fields=date,sleep_s", None),
    ("GET", "/api/v1/physiology/latest", None),
    ("GET", "/api/v1/physiology/sleep", None),
    ("GET", "/api/v1/physiology/hrv", None),
    ("GET", "/api/v1/physiology/readiness?from=2026-03-01&to=2026-03-10", None),
    ("GET", "/api/v1/physiology/sources", None),
    ("GET", "/api/v1/fitness/fingerprint", None),
    ("GET", "/api/v1/fitness/activity/1", None),
    ("GET", "/api/v1/fitness/trends", None),
    ("GET", "/api/v1/fitness/zones", None),
    ("GET", "/api/v1/gear", None),
    ("GET", "/api/v1/compare/candidates", None),
    ("GET", "/api/v1/compare?ids=1,2", None),
    ("GET", "/api/v1/tiles/policy", None),
    ("GET", "/api/v1/nonexistent", None),
    ("POST", "/api/v1/activities", None),
    ("PATCH", "/api/v1/activities/1", {"name": "Renamed", "perceived_exertion": 4}),
]


def record_cases(client) -> list:
    """Ask the app every sampled route and record what it answered."""
    cases = []
    for method, path, body in ROUTES:
        if method == "GET":
            response = client.get(path)
        elif method == "POST":
            response = client.post(path, json=body)
        else:
            response = client.patch(path, json=body)
        try:
            parsed = response.json()
        except ValueError:
            parsed = {"__raw__": response.text[:200]}
        cases.append({
            "method": method,
            "path": path,
            "body": body,
            "status": response.status_code,
            "response": parsed,
        })
    return cases


if __name__ == "__main__":
    raise SystemExit(main())
