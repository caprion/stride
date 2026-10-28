#!/usr/bin/env python3
"""Regenerate `expected.json` from the Python reference implementation.

Every number in `expected.json` is produced by **calling the reference
implementation's own functions** — not by transcribing them:

* `app.ingest.garmin_personal._as_list` — the envelope unwrapping;
* `app.ingest.garmin_personal._bounded_date_range` — the window clamping;
* `app.services.personal_sync._canonical_from_detail` — the canonical
  activity;
* `app.api.auth._athlete_from_personal` — the athlete's name.

So the file is the reference's output. If the Rust port disagrees with it,
the port is wrong.

Usage:

    python3 crates/stride-garmin/tests/fixtures/make_expected.py

The reference implementation is not modified by this script; it is imported.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]

# The reference implementation creates its data directory at import time; keep
# that out of the repository.
os.environ.setdefault("STRIDE_DATA_DIR", tempfile.mkdtemp(prefix="stride-golden-"))
sys.path.insert(0, str(ROOT / "backend"))

from app.api.auth import _athlete_from_personal  # noqa: E402
from app.ingest.garmin_personal import _as_list, _bounded_date_range  # noqa: E402
from app.services.personal_sync import _canonical_from_detail  # noqa: E402

CANONICAL_KEYS = [
    "external_id",
    "name",
    "sport_type",
    "start_time",
    "local_date",
    "day_of_week",
    "distance_m",
    "moving_time_s",
    "elapsed_time_s",
    "elevation_gain_m",
    "avg_hr",
    "max_hr",
    "min_hr",
    "calories",
    "avg_cadence",
    "max_cadence",
    "avg_speed_ms",
    "max_speed_ms",
    "device",
]

LIST_KEYS = [
    "external_id",
    "name",
    "start_time",
    "local_date",
    "day_of_week",
]


def load(name: str) -> object:
    return json.loads((HERE / name).read_text(encoding="utf-8"))


def canonical(activity: dict) -> dict:
    return {key: activity.get(key) for key in CANONICAL_KEYS}


def listed(activity: dict) -> dict:
    """The facts the reference reads from a list item on its own."""
    return {key: activity.get(key) for key in LIST_KEYS}


def main() -> None:
    profile = load("profile.json")
    activity_list = load("activity_list.json")
    detail = load("activity_detail.json")
    detail_fallback = load("activity_detail_fallback.json")
    workouts = load("workouts.json")
    zones = load("heart_rate_zones.json")
    sleep = load("sleep.json")
    hrv = load("hrv.json")
    daily_heart_rate = load("daily_heart_rate.json")
    daily_summary = load("daily_summary.json")
    body_battery = load("body_battery.json")
    stress = load("stress.json")
    steps = load("steps.json")
    hrv_range = load("hrv_range.json")
    sleep_range = load("sleep_range.json")
    weight = load("weight.json")

    items = activity_list["activityList"]
    first, second = items[0], items[1]

    profile_raw = {
        key: value
        for key, value in profile.items()
        if key not in ("displayName", "fullName", "userName", "name")
    }

    expected = {
        "profile": {
            "athlete_name": _athlete_from_personal(profile).get("name"),
            "raw": profile_raw,
        },
        # The canonical activity, from the detail payload.
        "activity_detail": canonical(_canonical_from_detail(detail, first)),
        # The same, for a detail payload that carries neither a name nor a
        # start time: both fall back to the list item.
        "activity_detail_fallback": canonical(
            _canonical_from_detail(detail_fallback, second)
        ),
        # The facts the reference reads from a list item on its own.
        "activity_list": [listed(_canonical_from_detail({}, item)) for item in items],
        "activity_list_ids": [str(item["activityId"]) for item in items],
        "activity_list_names": [item["activityName"] for item in items],
        "activity_list_sports": [item["activityTypeDTO"]["typeKey"] for item in items],
        # The envelopes, unwrapped by the reference's `_as_list`.
        "lists": {
            "activities": _as_list(
                activity_list, ("activityList", "activities", "activitySummaries")
            ),
            "workouts": _as_list(
                workouts, ("workouts", "activityWorkouts", "workoutSummaries")
            ),
            "body_battery": _as_list(
                body_battery, ("bodyBatteryReports", "bodyBatteryList", "reports")
            ),
            "stress": _as_list(
                stress, ("stressValuesArray", "stressDailyList", "stressList")
            ),
            "steps": _as_list(
                steps, ("stepsDailyList", "stepsList", "stepsValuesArray")
            ),
            "hrv_range": _as_list(
                hrv_range, ("hrvDailySummaries", "hrvSummaries", "hrvList")
            ),
            "sleep_range": _as_list(
                sleep_range, ("sleepSummaries", "dailySleepSummaries")
            ),
            "weight": _as_list(weight, ("dateWeightList", "weightList", "weights")),
        },
        # The window clamping.
        "dates": {
            "short": list(_bounded_date_range("2026-01-01", "2026-01-31")),
            "long": list(_bounded_date_range("2025-01-01", "2026-01-31")),
            "inverted": list(_bounded_date_range("2026-03-10", "2026-03-01")),
        },
        # The raw wellness payloads: the reference returns them untouched.
        "payloads": {
            "sleep": sleep,
            "hrv": hrv,
            "daily_heart_rate": daily_heart_rate,
            "daily_summary": daily_summary,
            "zones": zones,
        },
    }

    out = HERE / "expected.json"
    out.write_text(json.dumps(expected, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
