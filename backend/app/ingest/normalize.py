"""Canonical activity shape + shared helpers.

Every ingest adapter produces the same dictionary. That is the whole point: the
Garmin OAuth path, the Garmin account path, the FIT importer and the demo seeder
all emit identical structures, so the persistence layer and every analytic
downstream of it are written exactly once.

Field contract (``NormalizedActivity``)::

    external_id: str
    source: str                     # garmin | garmin_fit | demo
    name: str
    sport_type: str
    start_time: str                 # ISO-8601 with offset
    local_date: str                 # YYYY-MM-DD (athlete's local date)
    day_of_week: str
    distance_m: float
    moving_time_s: float
    elapsed_time_s: float
    elevation_gain_m: float
    avg_hr / max_hr: float | None
    calories: float | None
    avg_cadence / max_cadence: float | None
    avg_speed_ms / max_speed_ms: float | None
    perceived_exertion: int | None
    device: str | None
    gear: str | None
    is_trail / is_commute: bool
    description: str | None
    geo_points: list[[lat, lon]] | None
    laps: list[dict]
    streams: dict[kind, {"seconds": [...], "values": [...]}]
    hr_zones: list[dict]
    raw: dict | None
"""

from __future__ import annotations

import math
from datetime import datetime, timedelta, timezone
from typing import Any

# Garmin activity type keys we understand, mapped to our sport vocabulary.
SPORT_MAP = {
    "running": "running",
    "trail_running": "trail_running",
    "treadmill_running": "treadmill_running",
    "street_running": "running",
    "track_running": "running",
    "virtualrun": "virtualrun",
    "walking": "walking",
    "hiking": "hiking",
    "cycling": "cycling",
    "lap_swimming": "swimming",
    "open_water_swimming": "swimming",
    "pool_swimming": "swimming",
    "strength_training": "strength_training",
}


def decode_polyline(encoded: str, precision: int = 5) -> list[list[float]]:
    """Decode a Google/encoded polyline into ``[[lat, lon], ...]``.

    Garmin's Activity API returns polylines in exactly this encoding.
    """
    if not encoded:
        return []
    points: list[list[float]] = []
    index = 0
    lat = 0
    lon = 0
    factor = float(10**precision)
    length = len(encoded)
    while index < length:
        for is_lon in (False, True):
            shift = 0
            result = 0
            while True:
                if index >= length:
                    return points
                byte = ord(encoded[index]) - 63
                index += 1
                result |= (byte & 0x1F) << shift
                shift += 5
                if byte < 0x20:
                    break
            delta = ~(result >> 1) if result & 1 else (result >> 1)
            if is_lon:
                lon += delta
                points.append([lat / factor, lon / factor])
            else:
                lat += delta
    return points


def decode_garmin_polyline(encoded: str) -> list[list[float]]:
    """Decode Garmin's *unencoded* polyline format.

    Garmin's activity-service returns a string of ``lat,lon lat,lon ...`` pairs
    (sometimes comma-separated without spaces). This handles that shape, and
    falls back to the standard encoded format when the input looks encoded.
    """
    if not encoded:
        return []
    text = encoded.strip()
    if "," in text and "?" not in text:
        points: list[list[float]] = []
        for chunk in text.replace(",", " ").split():
            try:
                value = float(chunk)
            except ValueError:
                continue
            points.append([value, 0.0])
        # Values come as flat lat,lon,lat,lon — pair them up.
        paired: list[list[float]] = []
        for i in range(0, len(points) - 1, 2):
            paired.append([points[i][0], points[i + 1][0]])
        if paired:
            return paired
    return decode_polyline(text)


def pace_seconds_per_km(distance_m: float, duration_s: float) -> float | None:
    if not distance_m or distance_m <= 0 or not duration_s or duration_s <= 0:
        return None
    return duration_s / (distance_m / 1000.0)


def format_pace(seconds_per_km: float | None) -> str | None:
    """``372.5`` -> ``"6:13"`` (Strava-style pace, min:sec per km)."""
    if seconds_per_km is None or seconds_per_km <= 0:
        return None
    total = int(round(seconds_per_km))
    return f"{total // 60}:{total % 60:02d}"


def parse_iso(value: Any) -> datetime | None:
    """Best-effort parse of the many timestamp shapes Garmin returns."""
    if value is None or value == "":
        return None
    if isinstance(value, datetime):
        return value
    if isinstance(value, (int, float)):
        # Garmin uses epoch milliseconds for some fields.
        seconds = value / 1000.0 if value > 1e11 else float(value)
        try:
            return datetime.fromtimestamp(seconds, tz=timezone.utc)
        except (OverflowError, OSError, ValueError):
            return None
    text = str(value).strip()
    # "2026-01-04T07:22:33.0" / "2026-01-04 07:22:33"
    for fmt in (
        "%Y-%m-%dT%H:%M:%S.%f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S.%f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S.%fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%d",
    ):
        try:
            return datetime.strptime(text, fmt).replace(tzinfo=timezone.utc)
        except ValueError:
            continue
    try:
        parsed = datetime.fromisoformat(text.replace("Z", "+00:00"))
        return parsed if parsed.tzinfo else parsed.replace(tzinfo=timezone.utc)
    except ValueError:
        return None


def normalise_sport(value: Any) -> str:
    if not value:
        return "running"
    key = str(value).strip().lower().replace(" ", "_")
    if key in SPORT_MAP:
        return SPORT_MAP[key]
    if key.endswith("running"):
        return "running"
    return key or "running"


def safe_float(value: Any, default: float | None = None) -> float | None:
    if value is None or value == "":
        return default
    try:
        result = float(value)
    except (TypeError, ValueError):
        return default
    if math.isnan(result) or math.isinf(result):
        return default
    return result


def safe_int(value: Any, default: int | None = None) -> int | None:
    result = safe_float(value)
    return int(result) if result is not None else default


def blank_activity() -> dict[str, Any]:
    """An empty canonical activity, so adapters only fill what they know."""
    return {
        "external_id": "",
        "source": "garmin",
        "name": "Run",
        "sport_type": "running",
        "description": None,
        "start_time": datetime.now(timezone.utc).isoformat(),
        "local_date": datetime.now(timezone.utc).date().isoformat(),
        "day_of_week": None,
        "distance_m": 0.0,
        "moving_time_s": 0.0,
        "elapsed_time_s": 0.0,
        "elevation_gain_m": 0.0,
        "avg_hr": None,
        "max_hr": None,
        "calories": None,
        "avg_cadence": None,
        "max_cadence": None,
        "avg_speed_ms": None,
        "max_speed_ms": None,
        "perceived_exertion": None,
        "device": None,
        "gear": None,
        "is_trail": False,
        "is_commute": False,
        "visibility": "followers_only",
        "geo_points": None,
        "laps": [],
        "streams": {},
        "hr_zones": [],
        "raw": None,
    }


def day_of_week_for(date_str: str) -> str | None:
    try:
        return datetime.strptime(date_str, "%Y-%m-%d").strftime("%A")
    except (ValueError, TypeError):
        return None


def week_key(date_str: str) -> str:
    """ISO week key, e.g. ``2026-W03``."""
    try:
        dt = datetime.strptime(date_str, "%Y-%m-%d")
    except (ValueError, TypeError):
        return ""
    iso = dt.isocalendar()
    return f"{iso.year}-W{iso.week:02d}"


def month_key(date_str: str) -> str:
    return date_str[:7] if date_str else ""


def duration_to_seconds(value: Any) -> float:
    """Handle Garmin's duration representations (seconds float, or HH:MM:SS)."""
    if value is None:
        return 0.0
    if isinstance(value, str) and ":" in value:
        parts = [float(p) for p in value.split(":")]
        total = 0.0
        for part in parts:
            total = total * 60 + part
        return total
    return safe_float(value, 0.0) or 0.0


def timedelta_seconds(start: datetime | None, end: datetime | None) -> float:
    if not start or not end:
        return 0.0
    return (end - start).total_seconds()


def local_date_from(dt: datetime, offset_hours: float = 5.5) -> str:
    """Athlete-local date. India default (IST) unless Garmin supplies an offset."""
    try:
        shifted = dt.astimezone(timezone(timedelta(hours=offset_hours)))
    except (ValueError, OverflowError):
        shifted = dt
    return shifted.date().isoformat()
