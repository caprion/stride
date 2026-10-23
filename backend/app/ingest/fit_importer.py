"""FIT file importer.

The escape hatch that needs no Garmin credentials at all: export ``.FIT`` files
from Garmin Connect (or off the watch) and drop them in. This gives full
fidelity — GPS track, per-second heart rate, cadence, elevation and laps.

Parsing is done with ``fitparse``, which is a pure-Python decoder.

Note on the API: ``fitparse`` message getters return ``FieldData`` wrappers, not
plain values, and enum fields come back as objects whose ``str()`` looks like
``sport:fitness_equipment``. Everything below goes through :func:`value_of` so
those two wrinkles are handled in exactly one place.
"""

from __future__ import annotations

import io
import logging
from datetime import datetime, timezone
from typing import Any

from .normalize import blank_activity, normalise_sport, safe_float, safe_int

log = logging.getLogger(__name__)


def value_of(message: Any, name: str) -> Any:
    """Read one field from a fitparse message, unwrapping ``FieldData``."""
    try:
        value = message.get(name)
    except Exception:
        return None
    if value is None:
        return None
    inner = getattr(value, "value", None)
    if inner is not None:
        return inner
    return value


def enum_name(value: Any) -> str | None:
    """``sport:fitness_equipment`` -> ``fitness_equipment``."""
    if value is None:
        return None
    text = str(value)
    if ":" in text:
        text = text.split(":", 1)[1]
    return text or None


# FIT message fields we care about.
_LAP_FIELDS = (
    "record_number",
    "total_elapsed_time",
    "total_timer_time",
    "total_distance",
    "avg_speed",
    "max_speed",
    "avg_heart_rate",
    "max_heart_rate",
    "avg_cadence",
    "max_cadence",
    "total_ascent",
    "total_descent",
)
_SESSION_FIELDS = (
    "total_elapsed_time",
    "total_timer_time",
    "total_distance",
    "avg_speed",
    "max_speed",
    "avg_heart_rate",
    "max_heart_rate",
    "avg_cadence",
    "max_cadence",
    "total_calories",
    "total_ascent",
    "total_descent",
    "sport",
)


def _semicircles_to_degrees(value: float) -> float:
    """FIT stores positions in semicircles; 2**31 semicircles is 180 degrees.

    ``fitparse`` hands the raw value through, so a Bengaluru run would land
    at 156,000,000 "degrees" without this conversion.
    """
    return round(value * 180.0 / (2**31), 7)


def _running_cadence(value: float) -> float:
    """Normalise a FIT cadence to steps per minute.

    Some Garmin devices record running cadence in *stride cycles* rather than
    steps: one cycle is two footfalls. The watch displays 155 spm while the
    file says 77. A running cadence below 100 is therefore halved, and the
    true value is twice what is stored. (A real running cadence is never
    below 100, so the correction cannot fire on genuine data.)
    """
    return value * 2 if 0 < value < 100 else value


def parse_fit(data: bytes, filename: str = "activity.fit") -> dict[str, Any]:
    """Parse a FIT byte string into a canonical activity dict."""
    try:
        from fitparse import FitFile
    except ImportError as exc:  # pragma: no cover
        raise RuntimeError(
            "The 'fitparse' package is not installed; FIT import is unavailable."
        ) from exc

    try:
        fit = FitFile(io.BytesIO(data))
    except Exception as exc:
        raise ValueError(f"{filename} is not a readable FIT file: {exc}") from exc

    activity = blank_activity()
    activity["source"] = "garmin_fit"
    activity["name"] = (
        filename.rsplit(".", 1)[0].replace("_", " ").replace("-", " ").strip() or "Imported run"
    )

    file_ids: list[str] = []
    start_ts: float | None = None
    end_ts: float | None = None

    distance = 0.0
    speed_samples: list[tuple[float, float]] = []
    hr_samples: list[tuple[float, float]] = []
    cadence_samples: list[tuple[float, float]] = []
    elevation_samples: list[tuple[float, float]] = []
    distance_samples: list[tuple[float, float]] = []
    geo: list[list[float]] = []
    laps: list[dict] = []
    session: dict[str, Any] = {}
    device_name: str | None = None
    sport: str | None = None

    for message in fit.get_messages("file_id"):
        serial = value_of(message, "serial_number")
        if serial:
            file_ids.append(str(serial))

    def timestamp_of(message) -> float | None:
        value = value_of(message, "timestamp")
        if value is None:
            return None
        try:
            return value.timestamp()
        except (AttributeError, OSError, OverflowError, ValueError):
            return None

    for message in fit.get_messages("record"):
        ts = timestamp_of(message)
        if ts is not None:
            start_ts = start_ts if start_ts is not None else ts
            end_ts = ts

        raw_distance = safe_float(value_of(message, "distance"))
        if raw_distance is not None:
            distance = raw_distance  # metres, cumulative

        lat = safe_float(value_of(message, "position_lat"))
        lon = safe_float(value_of(message, "position_long"))
        if lat is not None and lon is not None:
            geo.append([_semicircles_to_degrees(lat), _semicircles_to_degrees(lon)])

        hr = safe_float(value_of(message, "heart_rate"))
        if hr is not None:
            hr_samples.append((ts or 0.0, hr))

        cad = safe_float(value_of(message, "cadence"))
        if cad is not None:
            cadence_samples.append((ts or 0.0, cad))

        speed = safe_float(value_of(message, "speed"))
        if speed is None:
            speed = safe_float(value_of(message, "enhanced_speed"))
        if speed is not None:
            speed_samples.append((ts or 0.0, speed * 3.6))

        alt = safe_float(value_of(message, "altitude"))
        if alt is None:
            alt = safe_float(value_of(message, "enhanced_altitude"))
        if alt is not None:
            elevation_samples.append((ts or 0.0, alt))

        if safe_float(value_of(message, "distance")) is not None:
            distance_samples.append((ts or 0.0, safe_float(value_of(message, "distance")) or 0.0))

    for message in fit.get_messages("lap"):
        lap: dict[str, Any] = {}
        for key in _LAP_FIELDS:
            value = value_of(message, key)
            if value is not None:
                lap[key] = value
        if lap:
            laps.append(lap)

    for message in fit.get_messages("session"):
        for key in _SESSION_FIELDS:
            value = value_of(message, key)
            if value is not None:
                session[key] = value
        if session.get("sport") is not None:
            sport = enum_name(session["sport"])
        break

    for message in fit.get_messages("device_info"):
        name = value_of(message, "product_name") or value_of(message, "manufacturer")
        if name:
            device_name = enum_name(name) or str(name)
            break

    # ------------------------------------------------------------ assemble

    elapsed = safe_float(session.get("total_elapsed_time"))
    timer = safe_float(session.get("total_timer_time"))
    moving = timer if timer else elapsed
    if not moving and start_ts and end_ts:
        moving = end_ts - start_ts

    activity["external_id"] = file_ids[0] if file_ids else filename
    activity["distance_m"] = safe_float(session.get("total_distance"), distance) or distance
    activity["moving_time_s"] = moving or 0.0
    activity["elapsed_time_s"] = elapsed or moving or 0.0
    activity["elevation_gain_m"] = safe_float(session.get("total_ascent"), 0.0) or 0.0
    activity["avg_hr"] = safe_int(session.get("avg_heart_rate"))
    activity["max_hr"] = safe_int(session.get("max_heart_rate"))
    activity["calories"] = safe_float(session.get("total_calories"))
    activity["avg_cadence"] = safe_float(session.get("avg_cadence"))
    activity["max_cadence"] = safe_float(session.get("max_cadence"))
    activity["avg_speed_ms"] = safe_float(session.get("avg_speed"))
    activity["max_speed_ms"] = safe_float(session.get("max_speed"))
    activity["device"] = device_name
    if sport:
        activity["sport_type"] = normalise_sport(sport)

    if start_ts:
        activity["start_time"] = datetime.fromtimestamp(start_ts, tz=timezone.utc).isoformat()
        activity["local_date"] = datetime.fromtimestamp(start_ts, tz=timezone.utc).date().isoformat()
        activity["day_of_week"] = datetime.fromtimestamp(start_ts, tz=timezone.utc).strftime("%A")

    if geo:
        activity["geo_points"] = geo

    if laps:
        out_laps = []
        for index, lap in enumerate(laps):
            lap_distance = safe_float(lap.get("total_distance"), 0.0) or 0.0
            lap_duration = safe_float(lap.get("total_timer_time")) or safe_float(
                lap.get("total_elapsed_time"), 0.0
            ) or 0.0
            lap_speed = safe_float(lap.get("avg_speed"))
            if not lap_speed and lap_distance and lap_duration:
                # Not every device records lap speed; it follows from distance
                # and time.
                lap_speed = lap_distance / lap_duration
            cadence = safe_float(lap.get("avg_cadence"))
            max_cadence = safe_float(lap.get("max_cadence"))
            out_laps.append(
                {
                    "lap_index": safe_int(lap.get("record_number"), index + 1) or index + 1,
                    "distance_m": lap_distance,
                    "duration_s": lap_duration,
                    "avg_speed_ms": lap_speed,
                    "max_speed_ms": safe_float(lap.get("max_speed")),
                    "avg_hr": safe_int(lap.get("avg_heart_rate")),
                    "max_hr": safe_int(lap.get("max_heart_rate")),
                    "avg_cadence": _running_cadence(cadence) if cadence else None,
                    "max_cadence": _running_cadence(max_cadence) if max_cadence else None,
                    "stride_length_m": None,
                    "elevation_gain_m": safe_float(lap.get("total_ascent")),
                    "elevation_loss_m": safe_float(lap.get("total_descent")),
                    "start_elevation_m": None,
                    "end_elevation_m": None,
                }
            )
        activity["laps"] = out_laps

    # Streams, re-based to the activity start so the charts line up.
    if start_ts:
        if hr_samples:
            activity["streams"]["hr"] = _series(hr_samples, start_ts, "bpm")
        if speed_samples:
            # A zero-speed sample is the athlete standing still, not a pace;
            # keeping it would flatten the chart's Y axis.
            moving = [(t, v) for t, v in speed_samples if v > 0.1]
            if moving:
                activity["streams"]["pace"] = _series(
                    [(t, 1000.0 / v) for t, v in moving], start_ts, "s/km"
                )
        elif distance_samples:
            # No speed field: derive it from the cumulative distance.
            activity["streams"]["pace"] = _series(
                _speed_from_distance(distance_samples), start_ts, "s/km"
            )
        if elevation_samples:
            activity["streams"]["elevation"] = _series(elevation_samples, start_ts, "m")
        if cadence_samples:
            activity["streams"]["cadence"] = _series(
                [(t, _running_cadence(v)) for t, v in cadence_samples], start_ts, "spm"
            )

    return activity


def _speed_from_distance(samples: list[tuple[float, float]]) -> list[tuple[float, float]]:
    """Speed in km/h between consecutive cumulative-distance samples."""
    out: list[tuple[float, float]] = []
    for index in range(1, len(samples)):
        (t0, d0), (t1, d1) = samples[index - 1], samples[index]
        seconds = t1 - t0
        if seconds <= 0:
            continue
        speed = (d1 - d0) / seconds * 3.6
        # A stationary sample is not a pace; keeping it would stretch the
        # chart's Y axis to zero.
        if speed > 0.1:
            out.append((t1, speed))
    return out


def _series(samples: list[tuple[float, float]], start_ts: float, unit: str) -> dict:
    seconds = [int(round(ts - start_ts)) for ts, _ in samples]
    values = [value for _, value in samples]
    return {"seconds": seconds, "values": values, "unit": unit}


def parse_fit_file(path: str) -> dict[str, Any]:
    with open(path, "rb") as handle:
        return parse_fit(handle.read(), filename=path.rsplit("/", 1)[-1])
