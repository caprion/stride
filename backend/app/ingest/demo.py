"""Demo dataset seeder.

Ships the app with a full year of realistic running data so the UI is alive on
first boot — no Garmin account, no key, no setup. The dataset comes from the
existing ``~/running`` project (161 activities across 12 months with per-km
splits, heart rate and elevation).

The same loader is reused to import a legacy ``unified-cache.json`` written by
the old Streamlit pipeline.
"""

from __future__ import annotations

import json
import logging
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ..config import DEMO_CACHE
from .normalize import (
    blank_activity,
    day_of_week_for,
    duration_to_seconds,
    format_pace,
    normalise_sport,
    parse_iso,
    safe_float,
    safe_int,
)

log = logging.getLogger(__name__)


def cache_path() -> Path | None:
    path = Path(DEMO_CACHE)
    return path if path.exists() else None


def load_cache(path: Path | None = None) -> dict[str, Any]:
    target = path or cache_path()
    if not target or not target.exists():
        return {}
    try:
        return json.loads(target.read_text())
    except (OSError, json.JSONDecodeError) as exc:
        log.warning("Could not read activity cache %s: %s", target, exc)
        return {}


def activities_from_cache(cache: dict[str, Any], rebase: bool = True) -> list[dict]:
    """Convert a legacy unified-cache payload into canonical activities.

    ``rebase`` shifts every activity so the newest one lands two days ago.
    The bundled dataset was captured a year ago; without rebasing the
    dashboard would open on an empty "this week" and the streaks would read
    zero. Rebasing keeps the demo alive regardless of when it is seeded.
    """
    out: list[dict] = []
    raw_items = cache.get("activities", []) or []
    shift_days = _rebase_shift(raw_items) if rebase else 0
    for raw in raw_items:
        activity = blank_activity()
        external_id = raw.get("id")
        if external_id is None:
            continue
        activity["external_id"] = str(external_id)
        activity["source"] = "demo"
        activity["name"] = raw.get("name") or "Run"
        activity["sport_type"] = normalise_sport(raw.get("type"))

        start = parse_iso(raw.get("date"))
        if start:
            if shift_days:
                from datetime import timedelta

                start = start + timedelta(days=shift_days)
            activity["start_time"] = start.isoformat()
            activity["local_date"] = start.date().isoformat()
            activity["day_of_week"] = start.strftime("%A")
        else:
            activity["local_date"] = str(raw.get("date", ""))[:10]
            activity["day_of_week"] = day_of_week_for(activity["local_date"])

        activity["distance_m"] = (safe_float(raw.get("distance_km"), 0.0) or 0.0) * 1000.0
        activity["moving_time_s"] = duration_to_seconds(raw.get("duration_seconds"))
        activity["elapsed_time_s"] = activity["moving_time_s"]
        activity["elevation_gain_m"] = safe_float(raw.get("elevation_gain_m"), 0.0) or 0.0
        activity["avg_hr"] = safe_int(raw.get("avg_hr"))
        activity["max_hr"] = safe_int(raw.get("max_hr"))
        activity["calories"] = safe_float(raw.get("calories"))
        activity["avg_cadence"] = safe_float(raw.get("avg_cadence"))

        pace = raw.get("avg_pace_min_km")
        if isinstance(pace, str) and ":" in pace:
            minutes, _, seconds = pace.partition(":")
            try:
                activity["avg_speed_ms"] = 1000.0 / (int(minutes) * 60 + int(seconds))
            except (TypeError, ValueError):
                pass

        # Splits -> laps
        splits = raw.get("splits") or {}
        laps_raw = splits.get("lapDTOs") if isinstance(splits, dict) else None
        if isinstance(laps_raw, list):
            laps = []
            for index, lap in enumerate(laps_raw):
                if not isinstance(lap, dict):
                    continue
                laps.append(
                    {
                        "lap_index": safe_int(lap.get("lapIndex"), index + 1) or index + 1,
                        "distance_m": safe_float(lap.get("distance"), 0.0) or 0.0,
                        "duration_s": duration_to_seconds(lap.get("duration")),
                        "avg_speed_ms": safe_float(lap.get("averageSpeed")),
                        "max_speed_ms": safe_float(lap.get("maxSpeed")),
                        "avg_hr": safe_int(lap.get("averageHR")),
                        "max_hr": safe_int(lap.get("maxHR")),
                        "avg_cadence": safe_float(lap.get("averageRunCadence")),
                        "max_cadence": safe_float(lap.get("maxRunCadence")),
                        "stride_length_m": safe_float(lap.get("strideLength")),
                        "elevation_gain_m": safe_float(lap.get("elevationGain")),
                        "elevation_loss_m": safe_float(lap.get("elevationLoss")),
                        "start_elevation_m": safe_float(lap.get("startElevation")),
                        "end_elevation_m": safe_float(lap.get("endElevation")),
                    }
                )
            activity["laps"] = laps

        # Build HR / pace / elevation series from the splits so the charts work.
        activity["streams"] = _streams_from_laps(
            activity["laps"], _synthesize_elevation(activity)
        )
        activity["geo_points"] = _synthesize_route(activity)
        activity["raw"] = {"demo": True}
        out.append(activity)
    return out


def _rebase_shift(raw_items: list[dict]) -> int:
    """Days to add so the newest activity sits two days before today."""
    from datetime import timedelta

    dates = []
    for raw in raw_items:
        parsed = parse_iso(raw.get("date"))
        if parsed:
            dates.append(parsed)
    if not dates:
        return 0
    newest = max(dates)
    target = datetime.now(timezone.utc).replace(hour=7, minute=0, second=0, microsecond=0) - timedelta(days=2)
    return (target - newest).days


# ---------------------------------------------------------------- routes

# A handful of plausible training grounds so demo routes do not all overlap.
_DEMO_CENTRES = [
    (12.9716, 77.5946, "Bengaluru"),
    (13.0827, 80.2707, "Chennai"),
    (18.5204, 73.8567, "Pune"),
    (17.3850, 78.4867, "Hyderabad"),
    (19.0760, 72.8777, "Mumbai"),
]


def _synthesize_route(activity: dict[str, Any]) -> list[list[float]] | None:
    """Deterministic pseudo-GPS track for a demo activity.

    Demo activities carry no real route, and a Strava-style activity page with
    an empty map looks broken. We synthesise a plausible loop whose length
    matches the activity distance, seeded deterministically from the activity
    id so the same activity always draws the same route.
    """
    import math
    import random

    external_id = str(activity.get("external_id") or "0")
    try:
        seed = int(external_id)
    except ValueError:
        seed = abs(hash(external_id))

    rng = random.Random(seed)
    centre_lat, centre_lon, _ = _DEMO_CENTRES[seed % len(_DEMO_CENTRES)]

    distance_m = activity.get("distance_m") or 0.0
    if distance_m <= 0:
        return None

    # Pick a loop radius that yields roughly the right circumference.
    points = max(60, min(320, int(distance_m / 120)))
    radius_m = max(180.0, distance_m / (2 * math.pi) / 1.15)

    # Wobble the radius so the loop is not a perfect circle.
    lobes = rng.choice([2, 3, 3, 4, 5])
    phase = rng.uniform(0, math.tau)
    wobble = rng.uniform(0.10, 0.28)

    track: list[list[float]] = []
    for index in range(points + 1):
        angle = math.tau * index / points
        r = radius_m * (1 + wobble * math.sin(lobes * angle + phase))
        lat = centre_lat + (r * math.cos(angle)) / 111_320.0
        lon = centre_lon + (r * math.sin(angle)) / (
            111_320.0 * max(0.2, math.cos(math.radians(centre_lat)))
        )
        # A little jitter so the track does not look mathematically clean.
        lat += rng.uniform(-0.00012, 0.00012)
        lon += rng.uniform(-0.00012, 0.00012)
        track.append([round(lat, 6), round(lon, 6)])
    return track


def _synthesize_elevation(activity: dict[str, Any]) -> list[float]:
    """Elevation profile matching the activity's recorded climb."""
    import math
    import random

    laps = activity.get("laps") or []
    if laps:
        values = [lap.get("end_elevation_m") or 0.0 for lap in laps]
        if any(values):
            return values
    gain = activity.get("elevation_gain_m") or 0.0
    seed = abs(hash(str(activity.get("external_id"))))
    rng = random.Random(seed)
    count = max(8, min(40, int((activity.get("distance_m") or 5000) / 1000)))
    profile = [
        round(
            (gain / 2) * (1 + math.sin(math.tau * i / count + rng.uniform(0, math.tau))),
            1,
        )
        for i in range(count)
    ]
    return profile


def _streams_from_laps(
    laps: list[dict], elevation_profile: list[float] | None = None
) -> dict[str, dict]:
    if not laps:
        return {}
    seconds: list[int] = []
    hr: list[float] = []
    pace: list[float] = []
    elevation: list[float] = []
    distance: list[float] = []
    clock = 0.0
    cumulative = 0.0
    for lap in laps:
        duration = lap.get("duration_s") or 0.0
        clock += duration
        cumulative += lap.get("distance_m") or 0.0
        seconds.append(int(round(clock)))
        distance.append(round(cumulative, 2))
        if lap.get("avg_hr") is not None:
            hr.append(float(lap["avg_hr"]))
        speed = lap.get("avg_speed_ms")
        pace.append(round(1000.0 / speed, 2) if speed else 0.0)
        elevation.append(lap.get("end_elevation_m") or 0.0)

    # The bundled dataset has no per-lap elevation, so fall back to a
    # synthesised profile that matches the recorded climb.
    if not any(elevation):
        if elevation_profile and len(elevation_profile) >= len(elevation):
            step = len(elevation_profile) / len(elevation)
            elevation = [
                elevation_profile[min(int(i * step), len(elevation_profile) - 1)]
                for i in range(len(elevation))
            ]
        else:
            elevation = [0.0] * len(elevation)

    streams: dict[str, dict] = {"distance": {"seconds": seconds, "values": distance, "unit": "m"}}
    if any(hr):
        streams["hr"] = {"seconds": seconds, "values": hr, "unit": "bpm"}
    if any(pace):
        streams["pace"] = {"seconds": seconds, "values": pace, "unit": "s/km"}
    if any(elevation):
        streams["elevation"] = {"seconds": seconds, "values": elevation, "unit": "m"}
    return streams
