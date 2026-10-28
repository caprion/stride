"""The golden fixtures: twelve FIT files, each covering one thing that bites.

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

The last three cases are different in kind: they are not shaped by what a port
might get wrong, they are shaped by what a *watch* actually writes. They were
written after reading thirteen FIT files downloaded from the athlete's own
Garmin account (March 2025 - October 2026), and they reproduce the byte
layout of those files. The data is synthetic: every position is fabricated
and the serial numbers are invented. See the section at the end of this file.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from fit_writer import (
    BASE_BYTE,
    BASE_SINT8,
    BASE_SINT32,
    BASE_UINT8,
    BASE_UINT16,
    BASE_UINT32,
    FitWriter,
    RawField,
    epoch,
)

HERE = Path(__file__).resolve().parent
FIXTURE_DIR = HERE / "fixtures"

GARMIN_MANUFACTURER = 1
PRODUCT_FORERUNNER_265 = 2697
PRODUCT_INDEX = 2625

# Every position in every fixture is fabricated, and they all start from this
# one anchor: a point in the South Pacific, roughly 15 500 km from the
# nearest point of the athlete's real routes (which span 13.07-21.13 N and
# 77.56-79.07 E) and about 4 000 km from the nearest coast. Nothing in the
# committed tree points at a real place.
SYNTHETIC_LAT = -35.0
SYNTHETIC_LON = -140.0

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
            lat=SYNTHETIC_LAT + t * 0.00001,
            lon=SYNTHETIC_LON + t * 0.000012,
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
            lat=SYNTHETIC_LAT + t * 0.00002,
            lon=SYNTHETIC_LON + t * 0.00002,
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
            lat=SYNTHETIC_LAT + t * 0.00003,
            lon=SYNTHETIC_LON + t * 0.00003,
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
            lat=SYNTHETIC_LAT + t * 0.000015,
            lon=SYNTHETIC_LON + t * 0.000015,
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


# ---------------------------------------------------------------------------
# The format a real watch writes
#
# The nine cases above are shaped by what a *port* might get wrong. The three
# cases below are shaped by what a *watch* actually writes. They were written
# after reading thirteen FIT files downloaded from the athlete's own Garmin
# account (March 2025 - October 2026). The byte layout is copied from those
# files; the data is not: every position is fabricated and describes the same
# point in the South Pacific, 15 500 km from the nearest point of the
# athlete's real routes, and the serial numbers are invented.
#
# What the thirteen real files have in common, and what these three cases
# reproduce:
#
#   * a 14-byte header, protocol version 1.0 (0x10), profile version 2196,
#     little endian, a valid header CRC and a valid file CRC;
#   * no compressed-timestamp headers, no developer fields, no 12-byte
#     header and no big-endian file: not one of the thirteen uses them;
#   * a record message that carries `enhanced_speed` (73) and
#     `enhanced_altitude` (78) and *never* `speed` (6) or `altitude` (2) —
#     the fallback the contract describes is the *normal* case on a real
#     watch, and no synthetic case exercised it;
#   * a record message that also carries fields the published FIT profile
#     does not describe (87, 90, 134, 135, 136);
#   * global messages the published FIT profile does not describe at all
#     (233, 288, 326 ...), one of which carries nothing but a four-byte
#     array;
#   * local message numbers reused and redefined: the same local number
#     carries a different global message later in the file, and the record
#     message has two live layouts at the same time;
#   * an *irregular* record stream: the gap between two records runs from
#     one second to fourteen, not the uniform one second the synthetic cases
#     assume;
#   * a `device_info` message with no `product_name` in it, so the watch is
#     reported as its manufacturer.
#
# The local message numbers below are the ones a real file uses: the device
# defines a message, uses it, and then reuses the same number for something
# else. A decoder that keeps one definition per local number for ever will
# mis-read the second half of the file.

# The Forerunner 965 — the watch that wrote all thirteen real files.
GARMIN_PRODUCT_965 = 3914
GARMIN_SOFTWARE_13 = 1300
PROFILE_2196 = 2196
PROTOCOL_FIT_10 = 0x10

# Field definition numbers the published FIT profile does not describe.
FIELD_87 = RawField(87, BASE_UINT16)
FIELD_90 = RawField(90, BASE_SINT8)
FIELD_134 = RawField(134, BASE_UINT8)
FIELD_135 = RawField(135, BASE_UINT8)
FIELD_136 = RawField(136, BASE_UINT8)

# Global message numbers the published FIT profile does not describe.
MESG_UNKNOWN_233 = 233
MESG_UNKNOWN_288 = 288
MESG_UNKNOWN_326 = 326

# The gap, in seconds, between two consecutive records. Not random: a fixed
# cycle, so the fixture is reproducible byte for byte. The thirteen real files
# average 3.7 to 5.4 seconds between records and reach fourteen; this cycle
# averages 3.5 and reaches eight. The 204-second hole of the second case is
# the longest gap the real files contain.
GAP_CYCLE = (1, 2, 3, 4, 5, 2, 6, 3, 1, 4, 7, 5, 2, 3, 8, 4)

# The local message numbers of the file, as a real file numbers them.
LOCAL_FILE_ID = 0
LOCAL_FILE_CREATOR = 1
LOCAL_UNKNOWN_233 = 2
LOCAL_EVENT = 3
LOCAL_DEVICE_INFO = 4
LOCAL_SPORT = 5
LOCAL_GPS_METADATA = 6
LOCAL_RECORD_A = 7
LOCAL_RECORD_B = 8
LOCAL_LAP = 9
LOCAL_SESSION = 10
LOCAL_UNKNOWN_288 = 11
LOCAL_UNKNOWN_326 = 12


def _real_preamble(writer: FitWriter, date: str, serial: int) -> None:
    """The preamble every real file has, in the order it has it."""
    writer.message("file_id", {
        "type": "activity",
        "manufacturer": GARMIN_MANUFACTURER,
        "product": GARMIN_PRODUCT_965,
        "serial_number": serial,
        "time_created": _timestamp(date, 0),
    }, local=LOCAL_FILE_ID)
    writer.message("file_creator", {
        "software_version": 13.0,
    }, local=LOCAL_FILE_CREATOR)
    writer.message("event", {
        "timestamp": _timestamp(date, 0),
        "event": "timer",
        "event_type": "start",
    }, local=LOCAL_EVENT)
    # No `product_name`: a real device_info message does not carry one, so the
    # reference reports the watch as its manufacturer.
    writer.message("device_info", {
        "timestamp": _timestamp(date, 0),
        "serial_number": serial,
        "manufacturer": GARMIN_MANUFACTURER,
        "product": GARMIN_PRODUCT_965,
        "software_version": 13.0,
        "device_index": 0,
    }, local=LOCAL_DEVICE_INFO)
    writer.message("sport", {
        "name": "Run",
        "sport": "running",
        "sub_sport": "generic",
    }, local=LOCAL_SPORT)


def _real_private_messages(writer: FitWriter, date: str) -> None:
    """The messages a real file carries that the FIT profile does not describe."""
    # mesg 233: one field, and it is a four-byte array.
    writer.raw_message(
        MESG_UNKNOWN_233, [(RawField(2, BASE_BYTE, 4), b"\x01\x02\x03\x04")],
        local=LOCAL_UNKNOWN_233,
    )
    # mesg 288: a timestamp and three small integers.
    writer.raw_message(MESG_UNKNOWN_288, [
        (RawField(253, BASE_UINT32), _timestamp(date, 0)),
        (RawField(2, BASE_UINT16), 1017),
        (RawField(0, BASE_SINT8), 28),
        (RawField(1, BASE_SINT8), 45),
    ], local=LOCAL_UNKNOWN_288)
    # mesg 326: a timestamp and two counters.
    writer.raw_message(MESG_UNKNOWN_326, [
        (RawField(253, BASE_UINT32), _timestamp(date, 0)),
        (RawField(0, BASE_UINT32), 49),
        (RawField(1, BASE_UINT32), 19),
    ], local=LOCAL_UNKNOWN_326)


def _real_record_fields(date: str, record: Record, stance: bool = False) -> dict[Any, Any]:
    """The record message a real watch writes.

    The field set is the one the thirteen real files carry, in the order they
    carry it, including the definition numbers the published FIT profile does
    not describe. `enhanced_speed` and `enhanced_altitude` are there and
    `speed` and `altitude` are not — a real watch writes the enhanced pair.
    """
    fields: dict[Any, Any] = {
        "timestamp": _timestamp(date, record.t),
        "position_lat": _semicircles(record.lat),
        "position_long": _semicircles(record.lon),
        "distance": record.distance,
        "enhanced_speed": record.speed,
        "enhanced_altitude": record.alt,
        FIELD_87: 0,
    }
    if stance:
        # The second live layout of the record message: the real files add
        # field 90 to it part-way through the activity.
        fields[FIELD_90] = 0
    fields.update({
        "heart_rate": record.hr,
        "cadence": record.cadence,
        "fractional_cadence": 0.5,
        FIELD_134: 0xFF,
        FIELD_135: 88,
        FIELD_136: record.hr,
    })
    return fields


def _real_gps_metadata(date: str, record: Record) -> dict[str, Any]:
    """The GPS metadata a real watch writes beside every record."""
    return {
        "timestamp": _timestamp(date, record.t),
        "position_lat": _semicircles(record.lat),
        "position_long": _semicircles(record.lon),
        "enhanced_altitude": record.alt,
        "enhanced_speed": record.speed,
    }


def _real_lap_message(writer: FitWriter, date: str, index: int, elapsed_s: float, timer_s: float,
                      distance_m: float, avg_hr: float, max_hr: float, avg_cadence: float,
                      max_cadence: float, avg_speed: float, max_speed: float,
                      ascent: float = 0.0, descent: float = 0.0) -> None:
    """A lap message in the shape a real watch writes it.

    `avg_speed` and `max_speed` are present but *invalid* (0xFFFF): a real
    lap carries the enhanced speeds and leaves the old ones unset, so the
    reference has to fall back to distance over time.
    """
    fields: dict[Any, Any] = {
        "timestamp": _timestamp(date, int(elapsed_s)),
        "start_time": _timestamp(date, 0),
        "total_elapsed_time": elapsed_s,
        "total_timer_time": timer_s,
        "total_distance": distance_m,
        "avg_heart_rate": avg_hr,
        "max_heart_rate": max_hr,
        "avg_cadence": avg_cadence,
        "max_cadence": max_cadence,
        "total_ascent": ascent,
        "total_descent": descent,
        "enhanced_avg_speed": avg_speed,
        "enhanced_max_speed": max_speed,
        RawField(13, BASE_UINT16): 0xFFFF,
        RawField(14, BASE_UINT16): 0xFFFF,
        "event": "lap",
        "event_type": "stop",
        "message_index": index,
    }
    writer.message("lap", fields, local=LOCAL_LAP)


# The blocks of records in the NULL case whose values are the reserved "no
# value" sentinels — one in each of the two record layouts — and the two
# records with a sentinel distance and a sentinel position.
NULL_BLOCKS = ((40, 60), (100, 120))
NULL_DISTANCE_RECORD = 41
NULL_POSITION_RECORD = 45


def _null_record_fields(date: str, record: Record, index: int, enhanced: bool) -> dict[Any, Any]:
    """A record message whose fields may be the reserved "no value" sentinels.

    Every numeric field is written raw, so a field can carry the sentinel
    its base type reserves for "no value". The layout stays the same and only
    the values change, which is what a watch does.

    The file has two record layouts, like the real files do: the first
    carries the classic `speed` and `altitude`, the second the enhanced
    pair. They are kept apart on purpose: the reference decoder expands
    `speed` into a component, so a message that carries both `speed` and
    `enhanced_speed` would have its enhanced value shadowed.
    """
    fields: dict[Any, Any] = {
        "timestamp": _timestamp(date, record.t),
        RawField(0, BASE_SINT32): _semicircles(record.lat),
        RawField(1, BASE_SINT32): _semicircles(record.lon),
        RawField(5, BASE_UINT32): round((record.distance or 0.0) * 100),
    }
    if enhanced:
        fields[RawField(73, BASE_UINT32)] = round(((record.speed or 0.0) + 0.5) * 1000)
        fields[RawField(78, BASE_UINT32)] = round((((record.alt or 0) + 5) + 500) * 5)
    else:
        fields[RawField(6, BASE_UINT16)] = round((record.speed or 0.0) * 1000)
        fields[RawField(2, BASE_UINT16)] = round(((record.alt or 0) + 500) * 5)
    fields[RawField(3, BASE_UINT8)] = record.hr or 0
    fields[RawField(4, BASE_UINT8)] = record.cadence or 0
    fields["fractional_cadence"] = 0.5
    fields[FIELD_87] = 0
    fields[FIELD_134] = 0xFF
    fields[FIELD_135] = 88
    fields[FIELD_136] = record.hr or 0

    nulls = any(start <= index < stop for start, stop in NULL_BLOCKS)
    if nulls:
        # No value at all.
        if enhanced:
            fields[RawField(73, BASE_UINT32)] = 0xFFFFFFFF
            fields[RawField(78, BASE_UINT32)] = 0xFFFFFFFF
        else:
            fields[RawField(6, BASE_UINT16)] = 0xFFFF
            fields[RawField(2, BASE_UINT16)] = 0xFFFF
        fields[RawField(3, BASE_UINT8)] = 0xFF
        fields[RawField(4, BASE_UINT8)] = 0xFF
    if index == NULL_DISTANCE_RECORD:
        fields[RawField(5, BASE_UINT32)] = 0xFFFFFFFF
    if index == NULL_POSITION_RECORD:
        fields[RawField(0, BASE_SINT32)] = 0x7FFFFFFF
        fields[RawField(1, BASE_SINT32)] = 0x7FFFFFFF
    return fields


def _real_session_message(writer: FitWriter, date: str, sport: str, elapsed_s: float, timer_s: float,
                          distance_m: float | None, avg_hr: float, max_hr: float, avg_cadence: float,
                          max_cadence: float, calories: int, ascent: float, descent: float,
                          avg_speed: float, max_speed: float, lap_count: int) -> None:
    """A session message in the shape a real watch writes it.

    `distance_m` of ``None`` writes the reserved "no value" sentinel into
    `total_distance` instead of a number.
    """
    fields: dict[Any, Any] = {
        "timestamp": _timestamp(date, int(elapsed_s)),
        "start_time": _timestamp(date, 0),
        "total_elapsed_time": elapsed_s,
        "total_timer_time": timer_s,
        RawField(9, BASE_UINT32): 0xFFFFFFFF if distance_m is None else round(distance_m * 100),
        "total_calories": calories,
        "avg_heart_rate": avg_hr,
        "max_heart_rate": max_hr,
        "avg_cadence": avg_cadence,
        "max_cadence": max_cadence,
        "total_ascent": ascent,
        "total_descent": descent,
        "sport": sport,
        "sub_sport": "generic",
        "enhanced_avg_speed": avg_speed,
        "enhanced_max_speed": max_speed,
        RawField(14, BASE_UINT16): 0xFFFF,
        RawField(15, BASE_UINT16): 0xFFFF,
        "first_lap_index": 0,
        "num_laps": lap_count,
        "event": "lap",
        "event_type": "stop",
        "message_index": 0,
    }
    writer.message("session", fields, local=LOCAL_SESSION)


def _real_activity_message(writer: FitWriter, date: str, elapsed_s: float) -> None:
    """The activity message — written under the *file_id* local number.

    A real file reuses local message number 0 for the activity message at the
    end, so a decoder that remembers the first definition of local 0 for ever
    reads the activity message as a file id.
    """
    writer.message("activity", {
        "timestamp": _timestamp(date, int(elapsed_s)),
        "total_timer_time": elapsed_s,
        "num_sessions": 1,
        "type": "manual",
        "event": "activity",
        "event_type": "stop",
    }, local=LOCAL_FILE_ID)


def _lap_stats(records: list[Record]) -> tuple[float, float, float, float]:
    """The heart rate and stored cadence of a lap, the way a watch summarises it."""
    heart_rates = [r.hr for r in records if r.hr is not None]
    cadences = [r.cadence for r in records if r.cadence is not None]
    return (
        round(sum(heart_rates) / len(heart_rates), 3),
        float(max(heart_rates)),
        round(sum(cadences) / len(cadences), 3),
        float(max(cadences)),
    )


def build_real_garmin_run() -> tuple[FitWriter, dict[str, Any]]:
    """A steady run in the byte format a real Garmin Forerunner 965 writes.

    The format is the one the thirteen real files use; the run is invented.
    """
    date = "2026-04-20T05:30:00Z"
    serial = 3117010
    records: list[Record] = []
    elapsed = 0
    distance = 0.0
    interval = 0
    for index in range(160):
        gap = GAP_CYCLE[index % len(GAP_CYCLE)]
        if index:
            # The distance covered during the interval that just ended, at the
            # speed of the sample that ended it.
            distance += (records[-1].speed or 0.0) * interval
        records.append(Record(
            elapsed,
            speed=round(3.0 + 0.1 * (index % 5), 2),
            cadence=78 + (index % 3),
            hr=140 + index // 20,
            alt=900 + index // 10,
            lat=round(SYNTHETIC_LAT + index * 0.00002, 7),
            lon=round(SYNTHETIC_LON + index * 0.000015, 7),
            distance=round(distance, 2),
        ))
        elapsed += gap
        interval = gap
    total_elapsed = float(elapsed)
    last_gap = GAP_CYCLE[(len(records) - 1) % len(GAP_CYCLE)]
    total_distance = round(records[-1].distance + (records[-1].speed or 0.0) * last_gap, 2)

    writer = FitWriter(profile_version=PROFILE_2196, protocol=PROTOCOL_FIT_10)
    _real_preamble(writer, date, serial)
    _real_private_messages(writer, date)

    for index, record in enumerate(records):
        # The watch writes its GPS metadata and its private sensor message
        # beside every record.
        writer.message("gps_metadata", _real_gps_metadata(date, record),
                       local=LOCAL_GPS_METADATA, define=index == 0)
        writer.raw_message(
            MESG_UNKNOWN_233, [(RawField(2, BASE_BYTE, 4), b"\x01\x02\x03\x04")],
            local=LOCAL_UNKNOWN_233, define=False,
        )
        # Part-way through the activity the watch starts a second live layout
        # of the record message.
        if index < 100:
            writer.message("record", _real_record_fields(date, record),
                           local=LOCAL_RECORD_A, define=index == 0)
        else:
            writer.message("record", _real_record_fields(date, record, stance=True),
                           local=LOCAL_RECORD_B, define=index == 100)

    for index, (start, stop) in enumerate(((0, 53), (53, 106), (106, 160))):
        lap_records = records[start:stop]
        avg_hr, max_hr, avg_cadence, max_cadence = _lap_stats(lap_records)
        lap_elapsed = (records[stop].t - records[start].t) if stop < len(records) else (
            total_elapsed - records[start].t
        )
        lap_distance = round(lap_records[-1].distance - lap_records[0].distance, 2)
        _real_lap_message(
            writer, date, index, lap_elapsed, lap_elapsed, lap_distance,
            avg_hr, max_hr, avg_cadence, max_cadence,
            round(sum(r.speed for r in lap_records) / len(lap_records), 3),
            round(max(r.speed for r in lap_records), 3),
            ascent=16.0 if index == 0 else 0.0,
            descent=0.0,
        )

    all_hr = [r.hr for r in records]
    all_cadence = [r.cadence for r in records]
    _real_session_message(
        writer, date, "running", total_elapsed, total_elapsed, total_distance,
        round(sum(all_hr) / len(all_hr), 3), float(max(all_hr)),
        round(sum(all_cadence) / len(all_cadence), 3), float(max(all_cadence)),
        520, 16.0, 0.0,
        round(sum(r.speed for r in records) / len(records), 3),
        round(max(r.speed for r in records), 3),
        3,
    )
    _real_activity_message(writer, date, total_elapsed)

    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 965",
        "watch": "records irregularly, one to fourteen seconds apart",
        "notes": "The byte format of a real Garmin Forerunner 965 file (thirteen of them, March 2025 "
                 "to October 2026): FIT protocol 1.0, enhanced speed and altitude instead of speed "
                 "and altitude, fields and messages the published FIT profile does not describe, a "
                 "redefined local message number and an irregular record stream. The format came "
                 "from a real watch; the data did not, and every position is fabricated.",
    }


def build_real_garmin_gap() -> tuple[FitWriter, dict[str, Any]]:
    """The same real-watch format, with the hole the real files contain.

    The longest gap between two records in the thirteen real files is 204
    seconds. It follows a *running* sample, the distance does not advance
    across it (the watch recorded nothing at all), and the sample after it is
    a standstill. So the hole is charged to the running — the interval after
    a sample belongs to that sample — while the distance it covers is not.
    """
    date = "2026-04-21T06:00:00Z"
    serial = 3117011
    records: list[Record] = []
    elapsed = 0
    distance = 0.0
    plan = (
        (30, 1.2, 30, 105),   # walking warm-up
        (90, 3.1, 78, None),  # running
        (10, 0.0, 0, 120),    # standing: the watch came back at the aid station
        (30, 2.4, 80, None),  # running again
    )
    hole_index = 119  # the gap *after* this record is the hole
    index = 0
    interval = 0
    hole_interval = False
    for block, (count, speed, cadence, hr) in enumerate(plan):
        for _ in range(count):
            gap = GAP_CYCLE[index % len(GAP_CYCLE)]
            if index == hole_index:
                # The hole: the watch stops writing records for three and a
                # half minutes.
                gap = 204
            if index and not hole_interval:
                distance += (records[-1].speed or 0.0) * interval
            records.append(Record(
                elapsed,
                speed=speed,
                cadence=cadence,
                hr=(140 + (index - 30) // 10) if hr is None else hr,
                alt=12,
                lat=round(SYNTHETIC_LAT + index * 0.00002, 7),
                lon=round(SYNTHETIC_LON + index * 0.000015, 7),
                distance=round(distance, 2),
            ))
            elapsed += gap
            interval = gap
            hole_interval = index == hole_index
            index += 1
    total_elapsed = float(elapsed)
    last_gap = GAP_CYCLE[(len(records) - 1) % len(GAP_CYCLE)]
    total_distance = round(records[-1].distance + (records[-1].speed or 0.0) * last_gap, 2)

    writer = FitWriter(profile_version=PROFILE_2196, protocol=PROTOCOL_FIT_10)
    _real_preamble(writer, date, serial)
    _real_private_messages(writer, date)

    for index, record in enumerate(records):
        writer.message("gps_metadata", _real_gps_metadata(date, record),
                       local=LOCAL_GPS_METADATA, define=index == 0)
        writer.raw_message(
            MESG_UNKNOWN_233, [(RawField(2, BASE_BYTE, 4), b"\x01\x02\x03\x04")],
            local=LOCAL_UNKNOWN_233, define=False,
        )
        if index < 60:
            writer.message("record", _real_record_fields(date, record),
                           local=LOCAL_RECORD_A, define=index == 0)
        else:
            writer.message("record", _real_record_fields(date, record, stance=True),
                           local=LOCAL_RECORD_B, define=index == 60)

    for index, (start, stop) in enumerate(((0, 120), (120, 160))):
        lap_records = records[start:stop]
        avg_hr, max_hr, avg_cadence, max_cadence = _lap_stats(lap_records)
        lap_elapsed = (records[stop].t - records[start].t) if stop < len(records) else (
            total_elapsed - records[start].t
        )
        lap_distance = round(lap_records[-1].distance - lap_records[0].distance, 2)
        _real_lap_message(
            writer, date, index, lap_elapsed, lap_elapsed, lap_distance,
            avg_hr, max_hr, avg_cadence, max_cadence,
            round(sum(r.speed for r in lap_records) / len(lap_records), 3),
            round(max(r.speed for r in lap_records), 3),
        )

    all_hr = [r.hr for r in records]
    all_cadence = [r.cadence for r in records]
    _real_session_message(
        writer, date, "running", total_elapsed, total_elapsed, total_distance,
        round(sum(all_hr) / len(all_hr), 3), float(max(all_hr)),
        round(sum(all_cadence) / len(all_cadence), 3), float(max(all_cadence)),
        300, 0.0, 0.0,
        round(sum(r.speed for r in records) / len(records), 3),
        round(max(r.speed for r in records), 3),
        2,
    )
    _real_activity_message(writer, date, total_elapsed)

    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 965",
        "watch": "records irregularly, with a 204 second hole",
        "notes": "The same real-watch format as real_garmin_run, with the 204 second hole the "
                 "real files contain: the watch records nothing for three and a half minutes and "
                 "the distance does not advance across it. The interval after a sample belongs to "
                 "that sample, so the hole is charged to the running, while the laps — which "
                 "only count the intervals between two running samples — are not.",
    }


def build_real_garmin_nulls() -> tuple[FitWriter, dict[str, Any]]:
    """The real-watch format, with the reserved "no value" sentinels in it.

    Every FIT base type has a reserved value that means *no value*: 0xFF for
    uint8, 0xFFFF for uint16, 0xFFFFFFFF for uint32, 0x7FFFFFFF for sint32,
    and so on. A decoder that reads one of them as a number turns a missing
    session average speed into 65.535 m/s — a plausible number that is not
    the athlete's, and that no range check catches, because 65.535 is inside
    the range. The real watch files contain exactly that (their lap and
    session messages carry `avg_speed` and `max_speed` as 0xFFFF), and the
    nine synthetic cases never would have found it.

    This case writes the sentinels into the fields the importer reads, so
    the rule stays frozen once the real files are gone:

    * `speed` and `altitude` are NULL while the enhanced pair carries the
      value, so the reference has to fall back to it;
    * `heart_rate` and `cadence` are NULL, so those fields are absent;
    * `distance` is NULL on one record;
    * the position of one record is NULL (sint32), so that record has no
      point;
    * the laps and the session carry NULL `avg_speed` and `max_speed`, and
      the session's `total_distance` is NULL too.

    The record-level sentinels here are synthetic — a real watch omits
    `speed` and `altitude` altogether rather than writing them as NULL — but
    they are the sentinels the real files use elsewhere, and they are the
    only way to keep the rule under test once the real files are gone.
    """
    date = "2026-04-22T05:45:00Z"
    serial = 3117012
    records: list[Record] = []
    elapsed = 0
    distance = 0.0
    interval = 0
    for index in range(160):
        gap = GAP_CYCLE[index % len(GAP_CYCLE)]
        if index:
            distance += (records[-1].speed or 0.0) * interval
        records.append(Record(
            elapsed,
            speed=round(3.0 + 0.1 * (index % 5), 2),
            cadence=78 + (index % 3),
            hr=140 + index // 20,
            alt=900 + index // 10,
            lat=round(SYNTHETIC_LAT + index * 0.00002, 7),
            lon=round(SYNTHETIC_LON + index * 0.000015, 7),
            distance=round(distance, 2),
        ))
        elapsed += gap
        interval = gap
    total_elapsed = float(elapsed)
    last_gap = GAP_CYCLE[(len(records) - 1) % len(GAP_CYCLE)]
    total_distance = round(records[-1].distance + (records[-1].speed or 0.0) * last_gap, 2)

    writer = FitWriter(profile_version=PROFILE_2196, protocol=PROTOCOL_FIT_10)
    _real_preamble(writer, date, serial)
    _real_private_messages(writer, date)

    for index, record in enumerate(records):
        writer.message("gps_metadata", _real_gps_metadata(date, record),
                       local=LOCAL_GPS_METADATA, define=index == 0)
        writer.raw_message(
            MESG_UNKNOWN_233, [(RawField(2, BASE_BYTE, 4), b"\x01\x02\x03\x04")],
            local=LOCAL_UNKNOWN_233, define=False,
        )
        writer.message("record", _null_record_fields(date, record, index, index >= 80),
                       local=LOCAL_RECORD_A if index < 80 else LOCAL_RECORD_B,
                       define=index in (0, 80))

    # Twelve laps, which is where the real files sit (2 to 32, mostly 9-13).
    splits = (13, 26, 40, 53, 66, 80, 93, 106, 120, 133, 146, 160)
    for index, stop in enumerate(splits):
        start = splits[index - 1] if index else 0
        lap_records = records[start:stop]
        avg_hr, max_hr, avg_cadence, max_cadence = _lap_stats(lap_records)
        lap_elapsed = (records[stop].t - records[start].t) if stop < len(records) else (
            total_elapsed - records[start].t
        )
        lap_distance = round(lap_records[-1].distance - lap_records[0].distance, 2)
        _real_lap_message(
            writer, date, index, lap_elapsed, lap_elapsed, lap_distance,
            avg_hr, max_hr, avg_cadence, max_cadence,
            round(sum(r.speed for r in lap_records) / len(lap_records), 3),
            round(max(r.speed for r in lap_records), 3),
        )

    all_hr = [r.hr for r in records]
    all_cadence = [r.cadence for r in records]
    _real_session_message(
        writer, date, "running", total_elapsed, total_elapsed, None,
        round(sum(all_hr) / len(all_hr), 3), float(max(all_hr)),
        round(sum(all_cadence) / len(all_cadence), 3), float(max(all_cadence)),
        520, 16.0, 0.0,
        round(sum(r.speed for r in records) / len(records), 3),
        round(max(r.speed for r in records), 3),
        len(splits),
    )
    _real_activity_message(writer, date, total_elapsed)

    return writer, {
        "date": date,
        "sport": "running",
        "device": "Forerunner 965",
        "watch": "records irregularly, with a block of absent values",
        "notes": "The real-watch format, with the reserved 'no value' sentinels in the fields the "
                 "importer reads: the classic speed and altitude are 0xFFFF in the first half of "
                 "the file and the enhanced pair is 0xFFFFFFFF in the second, the heart rate and "
                 "the cadence are 0xFF, one distance is 0xFFFFFFFF and one position is "
                 "0x7FFFFFFF. A sentinel is an absent field, and it is absent before any range "
                 "check: 0xFFFF as a speed is 65.535 m/s, which is inside the range and would pass "
                 "every check, and 0xFF as a heart rate is 255 bpm, which is inside 0-300. Twelve "
                 "laps, as the real files have.",
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
    ("real_garmin_run", build_real_garmin_run),
    ("real_garmin_gap", build_real_garmin_gap),
    ("real_garmin_nulls", build_real_garmin_nulls),
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
