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
import statistics
from datetime import datetime, timezone
from typing import Any

from .normalize import blank_activity, normalise_sport, safe_float, safe_int

log = logging.getLogger(__name__)

MAX_FIT_INPUT_BYTES = 20 * 1024 * 1024
MAX_FIT_RECORDS = 250_000


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


def _bounded_number(value: Any, maximum: float, default: float | None = None) -> float | None:
    number = safe_float(value)
    if number is None or number < 0 or number > maximum:
        return default
    return number


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


# Segment classification thresholds.
#
# A record is classified from its own speed and cadence:
#   stand — the athlete is not moving at all
#   walk  — moving, but at a step frequency (or speed) below running
#   run   — everything else
#
# The thresholds are deliberately conservative. Running cadence is stored
# *halved* on many devices (see `_running_cadence`), so a stored value of 80
# is a displayed 160 spm; a stored value below 60 is walking or shuffling.
# Garmin's own activity splits are coarser than the record stream and
# sometimes disagree with it, so the records are the source of truth here.
STAND_SPEED_MPS = 0.30
WALK_SPEED_MPS = 1.55
WALK_STORED_CADENCE = 60


def classify_sample(speed_mps: float | None, stored_cadence: float | None) -> str:
    """Classify one record as 'run', 'walk' or 'stand'."""
    if speed_mps is not None and speed_mps < STAND_SPEED_MPS:
        return "stand"
    if stored_cadence is not None and stored_cadence == 0:
        return "stand"
    if stored_cadence is not None and stored_cadence < WALK_STORED_CADENCE:
        if speed_mps is None or speed_mps < 2.2:
            return "walk"
    if speed_mps is not None and speed_mps < WALK_SPEED_MPS:
        return "walk"
    return "run"

def _semicircles_to_degrees(value: float) -> float:
    """FIT stores positions in semicircles; 2**31 semicircles is 180 degrees.

    ``fitparse`` hands the raw value through, so a Bengaluru run would land
    at 156,000,000 "degrees" without this conversion.
    """
    return round(value * 180.0 / (2**31), 7)


# Sports whose cadence field counts stride cycles (two footfalls) rather than
# steps. Only running is halved — a cyclist's 90 rpm is 90 rpm.
_HALVED_CADENCE_SPORTS = {
    "running",
    "trail_running",
    "treadmill_running",
    "street_running",
    "track_running",
    "virtualrun",
}


def _running_cadence(value: float, sport: str | None = None) -> float:
    """Normalise a FIT cadence to steps per minute.

    Some Garmin devices record running cadence in *stride cycles* rather than
    steps: one cycle is two footfalls. The watch displays 155 spm while the
    file says 77. A running cadence below 100 is therefore halved, and the
    true value is twice what is stored. (A real running cadence is never
    below 100, so the correction cannot fire on genuine data.)

    Cycling cadence is revolutions per minute and is **not** halved, so the
    correction is only applied to running sports.
    """
    if sport is not None and sport not in _HALVED_CADENCE_SPORTS:
        return value
    return value * 2 if 0 < value < 100 else value


def parse_fit(data: bytes, filename: str = "activity.fit") -> dict[str, Any]:
    """Parse an untrusted FIT file with bounded input and clean errors."""
    if not isinstance(data, (bytes, bytearray, memoryview)):
        raise ValueError(f"{filename} is not a readable FIT file: expected bytes")
    if len(data) > MAX_FIT_INPUT_BYTES:
        raise ValueError(f"{filename} exceeds the 20 MB FIT input limit")
    try:
        return _parse_fit_bounded(bytes(data), filename)
    except ValueError:
        raise
    except Exception as exc:
        raise ValueError(f"{filename} is not a readable FIT file: {exc}") from exc


def _parse_fit_bounded(data: bytes, filename: str) -> dict[str, Any]:
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
    # (epoch, distance, speed m/s, stored cadence, heart rate, altitude)
    records: list[tuple] = []
    geo: list[list[float]] = []
    laps: list[dict] = []
    session: dict[str, Any] = {}
    device_name: str | None = None
    sport: str | None = None

    for message in _limited_messages(fit, "file_id", 100, filename):
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

    for message in _limited_messages(fit, "record", MAX_FIT_RECORDS, filename):
        ts = timestamp_of(message)
        if ts is not None:
            start_ts = start_ts if start_ts is not None else ts
            end_ts = ts

        raw_distance = safe_float(value_of(message, "distance"))
        if raw_distance is not None and 0 <= raw_distance <= 10_000_000:
            distance = raw_distance  # metres, cumulative

        lat = safe_float(value_of(message, "position_lat"))
        lon = safe_float(value_of(message, "position_long"))
        if lat is not None and lon is not None and -2**31 <= lat <= 2**31 and -2**31 <= lon <= 2**31:
            geo.append([_semicircles_to_degrees(lat), _semicircles_to_degrees(lon)])

        hr = safe_float(value_of(message, "heart_rate"))
        if hr is not None and not 0 <= hr <= 300:
            hr = None
        if hr is not None:
            hr_samples.append((ts or 0.0, hr))

        cad = safe_float(value_of(message, "cadence"))
        if cad is not None and not 0 <= cad <= 300:
            cad = None
        if cad is not None:
            cadence_samples.append((ts or 0.0, cad))

        speed = safe_float(value_of(message, "speed"))
        if speed is None:
            speed = safe_float(value_of(message, "enhanced_speed"))
        if speed is not None and 0 <= speed <= 150:
            speed_samples.append((ts or 0.0, speed * 3.6))
        else:
            speed = None

        alt = safe_float(value_of(message, "altitude"))
        if alt is None:
            alt = safe_float(value_of(message, "enhanced_altitude"))
        if alt is not None and -1000 <= alt <= 10000:
            elevation_samples.append((ts or 0.0, alt))
        else:
            alt = None

        record_distance = safe_float(value_of(message, "distance"))
        if record_distance is not None and 0 <= record_distance <= 10_000_000:
            distance_samples.append((ts or 0.0, record_distance))
        else:
            record_distance = None

        # One row per record: epoch, distance, speed (m/s), stored cadence,
        # heart rate, altitude. Everything the analysis needs, so every stream
        # can be built from the same axis.
        records.append(
            (
                (ts or 0.0),
                record_distance,
                speed,  # already metres per second
                cad,
                hr,
                alt,
            )
        )

    for message in _limited_messages(fit, "lap", 10_000, filename):
        lap: dict[str, Any] = {}
        for key in _LAP_FIELDS:
            value = value_of(message, key)
            if value is not None:
                lap[key] = value
        if lap:
            laps.append(lap)

    for message in _limited_messages(fit, "session", 100, filename):
        for key in _SESSION_FIELDS:
            value = value_of(message, key)
            if value is not None:
                session[key] = value
        if session.get("sport") is not None:
            sport = enum_name(session["sport"])
        break

    for message in _limited_messages(fit, "device_info", 100, filename):
        name = value_of(message, "product_name") or value_of(message, "manufacturer")
        if name:
            device_name = enum_name(name) or str(name)
            break

    # ------------------------------------------------------------ assemble

    elapsed = _bounded_number(session.get("total_elapsed_time"), 604800)
    timer = _bounded_number(session.get("total_timer_time"), 604800)
    moving = timer if timer else elapsed
    if not moving and start_ts and end_ts:
        moving = end_ts - start_ts

    activity["external_id"] = file_ids[0] if file_ids else filename
    activity["distance_m"] = _bounded_number(session.get("total_distance"), 10_000_000, distance) or distance
    activity["moving_time_s"] = moving or 0.0
    activity["elapsed_time_s"] = elapsed or moving or 0.0
    activity["elevation_gain_m"] = _bounded_number(session.get("total_ascent"), 100_000, 0.0) or 0.0
    activity["avg_hr"] = safe_int(_bounded_number(session.get("avg_heart_rate"), 300))
    activity["max_hr"] = safe_int(_bounded_number(session.get("max_heart_rate"), 300))
    activity["calories"] = _bounded_number(session.get("total_calories"), 1_000_000)
    activity["avg_cadence"] = _bounded_number(session.get("avg_cadence"), 300)
    activity["max_cadence"] = _bounded_number(session.get("max_cadence"), 300)
    activity["avg_speed_ms"] = _bounded_number(session.get("avg_speed"), 150)
    activity["max_speed_ms"] = _bounded_number(session.get("max_speed"), 150)
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
        cumulative_elapsed = 0.0
        for index, lap in enumerate(laps):
            lap_distance = _bounded_number(lap.get("total_distance"), 10_000_000, 0.0) or 0.0
            lap_duration = _bounded_number(lap.get("total_timer_time"), 604800) or _bounded_number(
                lap.get("total_elapsed_time"), 604800, 0.0
            ) or 0.0
            lap_speed = _bounded_number(lap.get("avg_speed"), 150)
            if not lap_speed and lap_distance and lap_duration:
                # Not every device records lap speed; it follows from distance
                # and time.
                lap_speed = lap_distance / lap_duration
                if lap_speed > 150:
                    lap_speed = None
            cadence = _bounded_number(lap.get("avg_cadence"), 300)
            max_cadence = _bounded_number(lap.get("max_cadence"), 300)
            cumulative_elapsed += (
                safe_float(lap.get("total_elapsed_time"))
                or safe_float(lap.get("total_timer_time"))
                or 0.0
            )
            out_laps.append(
                {
                    "lap_index": safe_int(_bounded_number(lap.get("record_number"), 100_000), index + 1) or index + 1,
                    "distance_m": lap_distance,
                    "duration_s": lap_duration,
                    "avg_speed_ms": lap_speed,
                    "max_speed_ms": _bounded_number(lap.get("max_speed"), 150),
                    "avg_hr": safe_int(_bounded_number(lap.get("avg_heart_rate"), 300)),
                    "max_hr": safe_int(_bounded_number(lap.get("max_heart_rate"), 300)),
                    "avg_cadence": _running_cadence(cadence, sport) if cadence else None,
                    "max_cadence": _running_cadence(max_cadence, sport) if max_cadence else None,
                    "stride_length_m": None,
                    "elevation_gain_m": _bounded_number(lap.get("total_ascent"), 100_000),
                    "elevation_loss_m": _bounded_number(lap.get("total_descent"), 100_000),
                    "start_elevation_m": None,
                    "end_elevation_m": None,
                    "elapsed_s": round(cumulative_elapsed, 3),
                }
            )
        activity["laps"] = out_laps

    # ------------------------------------------------------------ segments
    #
    # A run is not a run: it contains walking warm-ups, walking cool-downs and
    # stands at the water fountain. Those parts are not running, and averaging
    # heart rate or cadence across them produces a number that describes
    # nobody. The record stream is segmented, and everything downstream — the
    # charts, the laps, the fingerprint — uses only the running parts.
    (
        segments,
        running_records,
        seg_totals,
        seg_distance,
        classified_records,
    ) = _segment_records(records, start_ts)

    activity["segments"] = segments
    activity["running_time_s"] = round(seg_totals.get("run", 0.0), 1)
    activity["walking_time_s"] = round(seg_totals.get("walk", 0.0), 1)
    activity["stopped_time_s"] = round(seg_totals.get("stand", 0.0), 1)
    activity["running_distance_m"] = round(seg_distance.get("run", 0.0), 1)

    # Streams, re-based to the activity start so the charts line up, and
    # restricted to the running samples.
    if start_ts:
        # The stream builder re-bases timestamps against the activity start, so
        # it needs the epoch, not the elapsed offset the segment helper uses.
        running = [r for r in records if classify_sample(r[2], r[3]) == "run"]
        if running:
            activity["streams"]["hr"] = _series(
                [(r[0], r[4]) for r in running if r[4] is not None], start_ts, "bpm"
            )
            activity["streams"]["cadence"] = _series(
                [(r[0], _running_cadence(r[3], sport)) for r in running if r[3] is not None],
                start_ts,
                "spm",
            )
            pace_rows = [
                (r[0], 1000.0 / r[2]) for r in running if r[2] is not None and r[2] > 0.1
            ]
            if pace_rows:
                activity["streams"]["pace"] = _series(pace_rows, start_ts, "s/km")
            activity["streams"]["distance"] = _series(
                [(r[0], r[1]) for r in running if r[1] is not None], start_ts, "m"
            )
        if running:
            # Elevation rides along with the running samples so every stream
            # shares one axis — the comparison view relies on that.
            running_elevation = [(r[0], r[5]) for r in running if r[5] is not None]
            if running_elevation:
                activity["streams"]["elevation"] = _series(running_elevation, start_ts, "m")

    # Laps recomputed over the running parts only. Garmin's kilometre laps
    # include the walking stretches, which is why a "1 km" lap can be 9:14
    # when the athlete was running at 7:00.
    if classified_records and activity.get("laps"):
        activity["laps"] = _running_laps(activity["laps"], classified_records, sport)

    # Lap boundaries, so the caller can recompute laps over the running parts.
    if start_ts and laps:
        bounds: list[float] = []
        cursor = 0.0
        for lap in laps:
            cursor += (
                safe_float(lap.get("total_elapsed_time"))
                or safe_float(lap.get("total_timer_time"))
                or 0.0
            )
            bounds.append(round(cursor, 3))
        activity["lap_bounds"] = bounds

    return activity


def _limited_messages(fit: Any, name: str, limit: int, filename: str):
    """Yield at most ``limit`` messages; raise before processing excess input."""
    for count, message in enumerate(fit.get_messages(name), 1):
        if count > limit:
            raise ValueError(f"{filename} exceeds the {limit} {name} message limit")
        yield message


def _segment_records(
    records: list[tuple[float, float | None, float | None, float | None, float | None]],
    start_ts: float | None,
) -> tuple[list[dict], list[tuple], dict[str, float], dict[str, float]]:
    """Split the record stream into run / walk / stand segments.

    Returns ``(segments, running_records, seconds_by_kind, metres_by_kind)``.
    """
    segments: list[dict] = []
    running: list[tuple] = []
    classified: list[tuple] = []
    seconds: dict[str, float] = {"run": 0.0, "walk": 0.0, "stand": 0.0}
    metres: dict[str, float] = {"run": 0.0, "walk": 0.0, "stand": 0.0}
    if not records:
        return segments, running, seconds, metres, classified

    base = start_ts if start_ts is not None else records[0][0]
    kinds = [classify_sample(r[2], r[3]) for r in records]

    # A sample represents the interval that follows it, up to the next sample.
    for index, record in enumerate(records):
        elapsed = record[0] - base
        kind = kinds[index]
        next_elapsed = (
            records[index + 1][0] - base if index + 1 < len(records) else elapsed
        )
        span = max(0.0, next_elapsed - elapsed)
        seconds[kind] = seconds.get(kind, 0.0) + span

        if index + 1 < len(records):
            previous = records[index][1]
            current = records[index + 1][1]
            if previous is not None and current is not None and current > previous:
                metres[kind] = metres.get(kind, 0.0) + (current - previous)

        classified.append((elapsed, record[1], record[2], record[3], record[4], kind))
        if kind == "run":
            running.append((elapsed,) + tuple(record[1:]))

        if segments and segments[-1]["kind"] == kind:
            segments[-1]["end_s"] = round(elapsed, 3)
            segments[-1]["samples"] += 1
        else:
            segments.append(
                {"kind": kind, "start_s": round(elapsed, 3), "end_s": round(elapsed, 3), "samples": 1}
            )

    for segment in segments:
        segment["duration_s"] = round(segment["end_s"] - segment["start_s"], 3)

    return segments, running, seconds, metres, classified



def _running_laps(
    laps: list[dict[str, Any]],
    classified: list[tuple],
    sport: str | None = None,
) -> list[dict[str, Any]]:
    """Rebuild each lap from the running samples that fall inside it.

    A lap keeps its index and its distance target, but its time, pace, heart
    rate and cadence describe only the running. A lap that was mostly walking
    comes back with very little running in it — which is the truth.
    """
    out: list[dict[str, Any]] = []
    cursor = 0.0
    for lap in laps:
        end = safe_float(lap.get("elapsed_s")) or cursor
        start = cursor
        cursor = end
        duration = max(0.0, end - start)
        window = [r for r in classified if start <= r[0] <= end]
        running = [r for r in window if r[5] == "run"]

        running_time = 0.0
        distance = 0.0
        heart_rates: list[float] = []
        cadences: list[float] = []
        for index in range(1, len(window)):
            previous, current = window[index - 1], window[index]
            # Only an interval between two running samples counts as running;
            # a gap that spans a rest stop is not running time.
            if previous[5] != "run" or current[5] != "run":
                continue
            running_time += max(0.0, current[0] - previous[0])
            if previous[1] is not None and current[1] is not None:
                distance += max(0.0, current[1] - previous[1])
        for record in running:
            if record[4] is not None:
                heart_rates.append(record[4])
            if record[3] is not None:
                cadences.append(_running_cadence(record[3], sport))

        pace = (running_time / (distance / 1000.0)) if distance > 1 and running_time else None
        out.append(
            {
                **lap,
                "duration_s": round(running_time, 3),
                "running_distance_m": round(distance, 2),
                "avg_speed_ms": (distance / running_time) if running_time > 0 and distance > 0 else None,
                "avg_hr": safe_int(statistics.mean(heart_rates)) if heart_rates else None,
                "max_hr": safe_int(max(heart_rates)) if heart_rates else None,
                "avg_cadence": round(statistics.mean(cadences), 1) if cadences else None,
                "max_cadence": round(max(cadences), 1) if cadences else None,
                "pace_s_per_km": round(pace, 2) if pace else None,
                "elapsed_s": round(duration, 3),
            }
        )
    return out

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
        data = handle.read(MAX_FIT_INPUT_BYTES + 1)
    return parse_fit(data, filename=path.rsplit("/", 1)[-1])
