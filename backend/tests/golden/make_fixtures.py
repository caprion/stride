"""The golden fixtures: nine FIT files, each covering one thing that bites.

Every case is described in plain Python below and rendered into a FIT file by
:mod:`fit_writer`. There is no randomness and no clock reading anywhere, so
regenerating the fixtures reproduces them byte for byte; ``cases.json`` records
the SHA-256 of each file so a drift is visible immediately.

The cases were chosen because each one breaks a plausible port:

``mixed_run``       walking cadence stored halved, a walk segment and a stand
                    segment in the same file — the segmentation case that
                    matters, and the one where a half-cadence bug hides.
``hr_gps_run``      a plain outdoor run with heart rate, GPS and elevation.
``treadmill``       no GPS at all: the port must not assume a position stream.
``no_cadence``      a watch with no cadence sensor: cadence metrics must be
                    absent, not zero.
``short_effort``    a 60-second effort: too few laps for thirds, so the
                    drift metrics must be null rather than invented.
``cycling``         a non-running sport whose cadence is *not* halved.
``low_resolution``  a watch that records every five seconds, not every second.
``hilly``           an elevation profile that climbs and descends.
``gaps``            a paused watch: the record stream has holes in it.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from fit_writer import FitWriter, epoch

HERE = Path(__file__).resolve().parent
FIXTURE_DIR = HERE / "fixtures"

GARMIN_MANUFACTURER = 1
PRODUCT_FORERUNNER_265 = 2697
PRODUCT_INDEX = 2625

# The reference importer reads the *record* stream, so the sample times below
# are the ground truth; the session/lap messages are the summary a device
# would also write.
DAY = "2026-03-02"


def _semicircles(degrees: float) -> int:
    return round(degrees * (2**31) / 180.0)


def _timestamp(start: str, seconds: int) -> int:
    """Seconds since the FIT epoch (1989-12-31T00:00:00Z)."""
    return int(epoch(start).timestamp() + seconds) - 631065600


class Record:
    """One record message."""

    __slots__ = ("t", "speed", "cadence", "hr", "alt", "lat", "lon", "distance")

    def __init__(self, t, speed=None, cadence=None, hr=None, alt=None, lat=None, lon=None, distance=None):
        self.t = t
        self.speed = speed
        self.cadence = cadence
        self.hr = hr
        self.alt = alt
        self.lat = lat
        self.lon = lon
        self.distance = distance


def _write_records(writer: FitWriter, records: list[Record], date: str) -> None:
    for record in records:
        fields: dict[str, Any] = {"timestamp": _timestamp(date, record.t)}
        if record.lat is not None:
            fields["position_lat"] = _semicircles(record.lat)
            fields["position_long"] = _semicircles(record.lon)
        if record.alt is not None:
            fields["altitude"] = record.alt
        if record.hr is not None:
            fields["heart_rate"] = record.hr
        if record.cadence is not None:
            fields["cadence"] = record.cadence
        if record.distance is not None:
            fields["distance"] = record.distance
        if record.speed is not None:
            fields["speed"] = record.speed
        writer.message("record", fields)


def _lap_message(writer: FitWriter, date: str, index: int, elapsed_s: float, timer_s: float,
                 distance_m: float, avg_speed: float, max_speed: float, avg_hr: float,
                 max_hr: float, avg_cadence: float | None, max_cadence: float | None,
                 ascent: float = 0.0, descent: float = 0.0) -> None:
    fields: dict[str, Any] = {
        "timestamp": _timestamp(date, int(elapsed_s)),
        "total_elapsed_time": elapsed_s,
        "total_timer_time": timer_s,
        "total_distance": distance_m,
        "avg_speed": avg_speed,
        "max_speed": max_speed,
        "avg_heart_rate": avg_hr,
        "max_heart_rate": max_hr,
        "total_ascent": ascent,
        "total_descent": descent,
    }
    if avg_cadence is not None:
        fields["avg_cadence"] = avg_cadence
        fields["max_cadence"] = max_cadence
    writer.message("lap", fields)


def _session_message(writer: FitWriter, date: str, sport: str, elapsed_s: float, timer_s: float,
                     distance_m: float, avg_speed: float, max_speed: float, avg_hr: float,
                     max_hr: float, avg_cadence: float | None, max_cadence: float | None,
                     calories: int = 0, ascent: float = 0.0, descent: float = 0.0) -> None:
    fields: dict[str, Any] = {
        "timestamp": _timestamp(date, int(elapsed_s)),
        "sport": sport,
        "total_elapsed_time": elapsed_s,
        "total_timer_time": timer_s,
        "total_distance": distance_m,
        "avg_speed": avg_speed,
        "max_speed": max_speed,
        "avg_heart_rate": avg_hr,
        "max_heart_rate": max_hr,
        "total_calories": calories,
        "total_ascent": ascent,
        "total_descent": descent,
    }
    if avg_cadence is not None:
        fields["avg_cadence"] = avg_cadence
        fields["max_cadence"] = max_cadence
    writer.message("session", fields)


def _header(writer: FitWriter, date: str, serial: int, device: str) -> None:
    writer.message("file_id", {
        "type": "activity",
        "manufacturer": GARMIN_MANUFACTURER,
        "product": PRODUCT_FORERUNNER_265,
        "serial_number": serial,
        "time_created": _timestamp(date, 0),
    })
    writer.message("device_info", {
        "device_index": 0,
        "device_type": 120,
        "manufacturer": GARMIN_MANUFACTURER,
        "product": PRODUCT_FORERUNNER_265,
        "serial_number": serial,
        "product_name": device,
    })


def _cumulative(records: list[Record]) -> None:
    """Fill in the cumulative distance of every record from the speeds."""
    total = 0.0
    for record in records:
        record.distance = round(total, 2)
        total += record.speed or 0.0


def _stats(values: list[float]) -> tuple[float, float]:
    return round(sum(values) / len(values), 3), round(max(values), 3)


# ---------------------------------------------------------------- the cases


def build_mixed_run() -> tuple[FitWriter, dict[str, Any]]:
    """Running cadence stored halved, with a walking warm-up and a stand."""
    date = "2026-03-02T05:30:00Z"
    records: list[Record] = []
    # 0-9 s: walking warm-up (speed 1.3 m/s, stored cadence 30 -> 60 spm).
    for t in range(0, 10):
        records.append(Record(t, speed=1.3, cadence=30, hr=112 + t // 5, alt=812))
    # 10-99 s: running (stored cadence 78 -> 156 spm).
    for t in range(10, 100):
        records.append(Record(t, speed=3.0, cadence=78, hr=131 + (t - 10) // 5, alt=812))
    # 100-119 s: standing still at the water fountain.
    for t in range(100, 120):
        records.append(Record(t, speed=0.0, cadence=0, hr=141, alt=812))
    # 120-169 s: running again.
    for t in range(120, 170):
        records.append(Record(t, speed=3.2, cadence=80, hr=151 + (t - 120) // 10, alt=812))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117001, device="Forerunner 265")
    _write_records(writer, records, date)

    run_records = [r for r in records if r.speed >= 1.55]
    avg_speed, max_speed = _stats([r.speed for r in run_records])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in run_records])

    _lap_message(writer, date, 1, 85.0, 85.0, records[85].distance, avg_speed, max_speed,
                 avg_hr, max_hr, avg_cadence, max_cadence)
    _lap_message(writer, date, 2, 85.0, 75.0, records[-1].distance - records[85].distance,
                 avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _session_message(writer, date, "running", 170.0, 150.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence, calories=210)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 265",
        "watch": "records every second",
        "notes": "Walk warm-up, run, 20 s stand, run. Stored cadence is halved.",
    }


def build_hr_gps_run() -> tuple[FitWriter, dict[str, Any]]:
    """Ten minutes outdoors: heart rate, GPS, steady cadence."""
    date = "2026-03-03T06:15:00Z"
    records: list[Record] = []
    for t in range(0, 600):
        records.append(Record(
            t,
            speed=3.3,
            cadence=80,
            hr=141 + t // 60,
            alt=704,
            lat=12.9716 + t * 0.00001,
            lon=77.5946 + t * 0.000012,
        ))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117002, device="Forerunner 265")
    _write_records(writer, records, date)

    avg_speed, max_speed = _stats([r.speed for r in records])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in records])
    for index in range(6):
        start = index * 100
        lap_records = records[start:start + 100]
        _lap_message(writer, date, index + 1, 100.0, 100.0,
                     lap_records[-1].distance - lap_records[0].distance,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _session_message(writer, date, "running", 600.0, 600.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence, calories=640)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 265",
        "watch": "records every second",
        "notes": "GPS present, heart rate present, cadence stored halved (80 -> 160 spm).",
    }


def build_treadmill() -> tuple[FitWriter, dict[str, Any]]:
    """A treadmill run: no GPS whatsoever."""
    date = "2026-03-04T18:45:00Z"
    records: list[Record] = []
    for t in range(0, 300):
        records.append(Record(t, speed=3.0, cadence=85, hr=136 + t // 60, alt=0))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117003, device="Forerunner 265")
    _write_records(writer, records, date)

    avg_speed, max_speed = _stats([r.speed for r in records])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in records])
    for index in range(3):
        start = index * 100
        lap_records = records[start:start + 100]
        _lap_message(writer, date, index + 1, 100.0, 100.0,
                     lap_records[-1].distance - lap_records[0].distance,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _session_message(writer, date, "running", 300.0, 300.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence, calories=330)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 265",
        "watch": "records every second",
        "notes": "Treadmill: no position fields at all. The FIT session sport is 'running'; "
                 "the reference importer ignores sub_sport, so a treadmill run is 'running'.",
    }


def build_no_cadence() -> tuple[FitWriter, dict[str, Any]]:
    """A watch with no cadence sensor."""
    date = "2026-03-05T07:05:00Z"
    records: list[Record] = []
    for t in range(0, 300):
        records.append(Record(
            t,
            speed=3.4,
            hr=144 + t // 60,
            alt=31,
            lat=12.9716 + t * 0.00002,
            lon=77.5946 + t * 0.00002,
        ))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117004, device="Edge 1040")
    _write_records(writer, records, date)

    avg_speed, max_speed = _stats([r.speed for r in records])
    avg_hr, max_hr = _stats([r.hr for r in records])
    for index in range(3):
        start = index * 100
        lap_records = records[start:start + 100]
        _lap_message(writer, date, index + 1, 100.0, 100.0,
                     lap_records[-1].distance - lap_records[0].distance,
                     avg_speed, max_speed, avg_hr, max_hr, None, None)
    _session_message(writer, date, "running", 300.0, 300.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, None, None, calories=340)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Edge 1040",
        "watch": "records every second, no cadence sensor",
        "notes": "No cadence field anywhere: every cadence metric must be null, never zero.",
    }


def build_short_effort() -> tuple[FitWriter, dict[str, Any]]:
    """A 60-second effort: too few laps for thirds."""
    date = "2026-03-06T12:20:00Z"
    records: list[Record] = []
    for t in range(0, 60):
        records.append(Record(t, speed=4.5, cadence=82, hr=157 + t // 20, alt=52))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117005, device="Forerunner 265")
    _write_records(writer, records, date)

    avg_speed, max_speed = _stats([r.speed for r in records])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in records])
    _lap_message(writer, date, 1, 60.0, 60.0, records[-1].distance + records[-1].speed,
                 avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _session_message(writer, date, "running", 60.0, 60.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence, calories=95)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 265",
        "watch": "records every second",
        "notes": "One lap only: thirds are empty, so drift and CV metrics are null.",
    }


def build_cycling() -> tuple[FitWriter, dict[str, Any]]:
    """Cycling: cadence is revolutions per minute and is NOT halved."""
    date = "2026-03-07T17:30:00Z"
    records: list[Record] = []
    for t in range(0, 300):
        records.append(Record(
            t,
            speed=8.33,
            cadence=90,
            hr=142 + t // 60,
            alt=45,
            lat=12.9716 + t * 0.00003,
            lon=77.5946 + t * 0.00003,
        ))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117006, device="Edge 1040")
    _write_records(writer, records, date)

    avg_speed, max_speed = _stats([r.speed for r in records])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in records])
    for index in range(3):
        start = index * 100
        lap_records = records[start:start + 100]
        _lap_message(writer, date, index + 1, 100.0, 100.0,
                     lap_records[-1].distance - lap_records[0].distance,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _session_message(writer, date, "cycling", 300.0, 300.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence, calories=520)
    return writer, {
        "date": date,
        "sport": "cycling",
        "device": "Edge 1040",
        "watch": "records every second",
        "notes": "Cycling cadence is 90 rpm and must stay 90: the halving correction applies to "
                 "running sports only.",
    }


def build_low_resolution() -> tuple[FitWriter, dict[str, Any]]:
    """A watch that records every five seconds, with a walk and a stand."""
    date = "2026-03-08T06:40:00Z"
    records: list[Record] = []
    for index in range(0, 120):
        t = index * 5
        if index <= 40:
            records.append(Record(t, speed=1.2, cadence=28, hr=104 + index // 10, alt=18))
        elif index <= 99:
            records.append(Record(t, speed=3.1, cadence=79, hr=133 + (index - 41) // 10, alt=18))
        else:
            records.append(Record(t, speed=0.0, cadence=0, hr=138, alt=18))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117007, device="Venu 3")
    _write_records(writer, records, date)

    running = [r for r in records if r.speed >= 1.55]
    avg_speed, max_speed = _stats([r.speed for r in running])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in running])
    _lap_message(writer, date, 1, 300.0, 300.0, records[59].distance, avg_speed, max_speed,
                 avg_hr, max_hr, avg_cadence, max_cadence)
    _lap_message(writer, date, 2, 300.0, 290.0, records[-1].distance - records[59].distance,
                 avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _session_message(writer, date, "running", 600.0, 590.0, records[-1].distance,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence, calories=560)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Venu 3",
        "watch": "records every five seconds",
        "notes": "Five-second sampling: intervals are five seconds wide, not one.",
    }


def build_hilly() -> tuple[FitWriter, dict[str, Any]]:
    """An elevation profile: 60 m up, 60 m down."""
    date = "2026-03-09T05:55:00Z"
    records: list[Record] = []
    for t in range(0, 300):
        if t < 100:
            alt = 600 + round(t * 0.6)
            speed = 2.8
        elif t < 200:
            alt = 660 - round((t - 100) * 0.6)
            speed = 3.4
        else:
            alt = 600
            speed = 3.1
        records.append(Record(
            t,
            speed=speed,
            cadence=77,
            hr=140 + t // 50,
            alt=alt,
            lat=12.9716 + t * 0.000015,
            lon=77.5946 + t * 0.000015,
        ))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117008, device="Forerunner 965")
    _write_records(writer, records, date)

    avg_speed, max_speed = _stats([r.speed for r in records])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in records])
    for index in range(3):
        start = index * 100
        lap_records = records[start:start + 100]
        _lap_message(writer, date, index + 1, 100.0, 100.0,
                     lap_records[-1].distance - lap_records[0].distance,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence,
                     ascent=60.0 if index < 2 else 0.0, descent=0.0 if index < 2 else 60.0)
    _session_message(writer, date, "running", 300.0, 300.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence,
                     calories=380, ascent=60.0, descent=60.0)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 965",
        "watch": "records every second",
        "notes": "Climbs 60 m then descends 60 m; the elevation stream follows the running samples.",
    }


def build_gaps() -> tuple[FitWriter, dict[str, Any]]:
    """A paused watch: the record stream has holes in it."""
    date = "2026-03-10T19:10:00Z"
    records: list[Record] = []
    # 0-98 s: running.
    for t in range(0, 99):
        records.append(Record(t, speed=3.0, cadence=79, hr=138 + t // 40, alt=90))
    # 99 s: standing (the watch is paused immediately afterwards).
    records.append(Record(99, speed=0.0, cadence=0, hr=141, alt=90))
    # 200-299 s: running again, after a 100 second hole in the stream.
    for t in range(200, 300):
        records.append(Record(t, speed=3.0, cadence=79, hr=142 + (t - 200) // 40, alt=90))
    # 400-439 s: running again, after another 100 second hole.
    for t in range(400, 440):
        records.append(Record(t, speed=3.0, cadence=79, hr=145, alt=90))
    _cumulative(records)

    writer = FitWriter()
    _header(writer, date, serial=3117009, device="Forerunner 965")
    _write_records(writer, records, date)

    running = [r for r in records if r.speed >= 1.55]
    avg_speed, max_speed = _stats([r.speed for r in running])
    avg_hr, max_hr = _stats([r.hr for r in records])
    avg_cadence, max_cadence = _stats([r.cadence for r in running])
    _lap_message(writer, date, 1, 200.0, 200.0, records[99].distance, avg_speed, max_speed,
                 avg_hr, max_hr, avg_cadence, max_cadence)
    _lap_message(writer, date, 2, 100.0, 100.0, records[199].distance - records[99].distance,
                 avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _lap_message(writer, date, 3, 140.0, 140.0, records[-1].distance - records[199].distance,
                 avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence)
    _session_message(writer, date, "running", 440.0, 440.0, records[-1].distance + records[-1].speed,
                     avg_speed, max_speed, avg_hr, max_hr, avg_cadence, max_cadence, calories=520)
    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 965",
        "watch": "records every second, but the watch was paused twice",
        "notes": "Two 100 second holes. The interval after a sample belongs to that sample, so a "
                 "hole that follows a running sample is charged to running time.",
    }


BUILDERS = (
    ("mixed_run", build_mixed_run),
    ("hr_gps_run", build_hr_gps_run),
    ("treadmill", build_treadmill),
    ("no_cadence", build_no_cadence),
    ("short_effort", build_short_effort),
    ("cycling", build_cycling),
    ("low_resolution", build_low_resolution),
    ("hilly", build_hilly),
    ("gaps", build_gaps),
)


def write_fixtures(target: Path | None = None) -> dict[str, Any]:
    """Write every fixture and return the descriptor for each of them."""
    directory = Path(target) if target else FIXTURE_DIR
    directory.mkdir(parents=True, exist_ok=True)
    descriptors: dict[str, Any] = {}
    for case_id, builder in BUILDERS:
        writer, meta = builder()
        data = writer.to_bytes()
        path = directory / f"{case_id}.fit"
        path.write_bytes(data)
        merged = {**meta, "start": meta["date"], "date": meta["date"][:10]}
        descriptors[case_id] = {
            "case_id": case_id,
            "file": f"backend/tests/golden/fixtures/{case_id}.fit",
            "sha256": hashlib.sha256(data).hexdigest(),
            "bytes": len(data),
            **merged,
        }
    return descriptors


def write_manifest(target: Path | None = None) -> dict[str, Any]:
    """Write ``cases.json`` — the frozen description of every input."""
    directory = Path(target) if target else HERE
    descriptors = write_fixtures(directory / "fixtures" if target else FIXTURE_DIR)
    manifest = {
        "generator": "backend/tests/golden/make_fixtures.py",
        "command": "python3 scripts/golden.py --fixtures",
        "cases": descriptors,
    }
    path = directory / "cases.json"
    path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest


if __name__ == "__main__":
    manifest = write_manifest()
    for case_id, meta in manifest["cases"].items():
        print(f"{case_id:16s} {meta['bytes']:7d} bytes  sha256={meta['sha256']}")
