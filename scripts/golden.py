#!/usr/bin/env python3
"""The golden-fixture harness: generate, check, and describe Stride's domain logic.

    python3 scripts/golden.py --check      regenerate everything and compare
    python3 scripts/golden.py --generate    (re)write the golden files
    python3 scripts/golden.py --fixtures    (re)write the FIT fixtures
    python3 scripts/golden.py --list        list the cases

``--check`` exits non-zero if anything moved. It is the proof that the
reference implementation is stable, and the target a port is measured against.

Everything it writes is deterministic: the same inputs produce the same bytes
on any machine. Where the reference implementation is timezone-sensitive (the
FIT importer converts timestamps through the host timezone) the harness pins
``TZ=UTC`` and says so in the contract.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
from datetime import date
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
BACKEND = REPO / "backend"
GOLDEN_DIR = BACKEND / "tests" / "golden"
FIXTURE_DIR = GOLDEN_DIR / "fixtures"
OUTPUT_DIR = GOLDEN_DIR / "golden"
SCENARIO_DIR = GOLDEN_DIR / "scenarios"
VENV_PYTHON = BACKEND / ".venv" / "bin" / "python"

sys.path.insert(0, str(BACKEND))
sys.path.insert(0, str(GOLDEN_DIR))


# --------------------------------------------------------------------- setup


def _reexec_into_venv() -> None:
    """Re-run under the backend virtualenv if the modules are missing."""
    if os.environ.get("STRIDE_GOLDEN_REEXEC") == "1":
        return
    try:
        import sqlalchemy  # noqa: F401
        import fitparse  # noqa: F401
    except ImportError:
        if not VENV_PYTHON.exists():
            return
        env = dict(os.environ, STRIDE_GOLDEN_REEXEC="1")
        raise SystemExit(subprocess.call([str(VENV_PYTHON), str(Path(__file__).resolve()), *sys.argv[1:]], env=env))


def pin_environment() -> None:
    """Make the reference implementation deterministic.

    * ``TZ=UTC`` — the FIT importer converts FIT timestamps through the *host*
      timezone (fitparse returns naive datetimes), so the same file parses to
      different wall-clock times on different machines. The contract requires a
      port to treat FIT timestamps as UTC.
    * ``STRIDE_DATA_DIR`` — a throwaway directory, so the harness never reads
      or writes the athlete's real database or state file.
    * ``STRIDE_KEYVAULT_ENABLED=false`` — never touch Azure.
    """
    os.environ["TZ"] = "UTC"
    if hasattr(time, "tzset"):
        time.tzset()
    os.environ["STRIDE_DATA_DIR"] = tempfile.mkdtemp(prefix="stride-golden-")
    os.environ["STRIDE_KEYVAULT_ENABLED"] = "false"


def _import_app():
    """Import the reference implementation (after the environment is pinned)."""
    from app.db import Base
    from app.ingest import fit_importer
    from app.models import Activity, ActivityLap, ActivityStream, HrZone  # noqa: F401
    from app.models_phase2 import (  # noqa: F401
        DailyPhysiology,
        PlanBlock,
        PlanPhase,
        PlanSession,
        PlanWeek,
        RunFingerprint,
    )
    from app.services import fingerprint as fingerprint_service
    from app.services import plan_engine, readiness as readiness_service
    from app.services.persistence import store_activity
    from sqlalchemy import create_engine
    from sqlalchemy.orm import Session

    return {
        "Base": Base,
        "fit_importer": fit_importer,
        "fingerprint": fingerprint_service,
        "plan_engine": plan_engine,
        "readiness": readiness_service,
        "store_activity": store_activity,
        "create_engine": create_engine,
        "Session": Session,
        "DailyPhysiology": DailyPhysiology,
        "PlanBlock": PlanBlock,
        "PlanPhase": PlanPhase,
        "PlanSession": PlanSession,
        "PlanWeek": PlanWeek,
    }


# ------------------------------------------------------------------ helpers


def dump_json(payload: Any) -> str:
    """The canonical golden file format.

    UTF-8, two-space indent, one key per line, insertion order preserved
    (never sorted — the order is part of the contract), LF line endings, and a
    trailing newline. ``allow_nan=False`` refuses to write NaN or infinity,
    which would not be valid JSON.
    """
    return json.dumps(payload, indent=2, ensure_ascii=False, allow_nan=False) + "\n"


def write_golden(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(dump_json(payload), encoding="utf-8", newline="\n")


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _round_floats(value: Any, digits: int = 10) -> Any:
    """Normalise floats so the golden bytes do not depend on the last ULP.

    The reference implementation is deterministic on IEEE-754 doubles, but the
    *representation* of a double is not part of the contract. Rounding to ten
    decimal places keeps every meaningful digit and removes the noise that a
    port's summation order could produce. Values whose magnitude is large
    (timestamps, distances) keep their precision through the relative test
    below.
    """
    if isinstance(value, float):
        if value != value or value in (float("inf"), float("-inf")):
            raise ValueError(f"non-finite float in golden output: {value!r}")
        if value == 0.0:
            return 0.0
        magnitude = abs(value)
        if magnitude >= 1e6:
            return value
        return round(value, digits)
    if isinstance(value, dict):
        return {key: _round_floats(item, digits) for key, item in value.items()}
    if isinstance(value, list):
        return [_round_floats(item, digits) for item in value]
    return value


# ------------------------------------------------------------- import stage


def parse_case(case_id: str, path: Path, app) -> dict[str, Any]:
    """Parse one FIT fixture and capture what the reference implementation did.

    The per-sample classification is captured by spying on the importer's own
    ``_segment_records`` call, so the rows below are exactly the tuples the
    reference used — not a re-implementation of them.
    """
    fit_importer = app["fit_importer"]
    data = path.read_bytes()
    captured: dict[str, Any] = {}
    original = fit_importer._segment_records

    def spy(records, start_ts):
        result = original(records, start_ts)
        captured["records"] = records
        captured["classified"] = result[4]
        return result

    fit_importer._segment_records = spy
    try:
        activity = fit_importer.parse_fit(data, filename=path.name)
    finally:
        fit_importer._segment_records = original

    return {"activity": activity, "records": captured["records"], "classified": captured["classified"]}


def import_section(case_id: str, parsed: dict[str, Any], sport: str) -> dict[str, Any]:
    """The frozen import result: the activity, its segmentation and streams."""
    activity = parsed["activity"]
    records = parsed["records"]
    classified = parsed["classified"]
    from app.ingest.fit_importer import _running_cadence

    samples = []
    for index, row in enumerate(classified):
        elapsed, distance, speed, stored_cadence, heart_rate, kind = row
        altitude = records[index][5] if index < len(records) else None
        samples.append({
            "index": index,
            "elapsed_s": elapsed,
            "kind": kind,
            "distance_m": distance,
            "speed_ms": speed,
            "stored_cadence": stored_cadence,
            "cadence_spm": _running_cadence(stored_cadence, sport) if stored_cadence is not None else None,
            "heart_rate_bpm": heart_rate,
            "altitude_m": altitude,
        })

    scalars = {
        key: activity[key]
        for key in (
            "external_id", "source", "name", "sport_type", "description", "start_time",
            "local_date", "day_of_week", "distance_m", "moving_time_s", "elapsed_time_s",
            "elevation_gain_m", "avg_hr", "max_hr", "calories", "avg_cadence", "max_cadence",
            "avg_speed_ms", "max_speed_ms", "perceived_exertion", "device", "gear",
            "is_trail", "is_commute", "running_time_s", "walking_time_s", "stopped_time_s",
            "running_distance_m",
        )
    }

    geo = activity.get("geo_points") or []
    return {
        "activity": scalars,
        "segments": activity["segments"],
        "totals": {
            "running_time_s": activity["running_time_s"],
            "walking_time_s": activity["walking_time_s"],
            "stopped_time_s": activity["stopped_time_s"],
            "running_distance_m": activity["running_distance_m"],
        },
        "samples": samples,
        "streams": activity.get("streams", {}),
        "laps": activity.get("laps", []),
        "lap_bounds": activity.get("lap_bounds", []),
        "hr_zones": activity.get("hr_zones", []),
        "geo_points": {
            "count": len(geo),
            "first": geo[:3],
            "last": geo[-3:],
        },
    }


# --------------------------------------------------------- fingerprint stage


def fingerprint_section(case_id: str, parsed: dict[str, Any], app) -> dict[str, Any]:
    """Persist the activity exactly as the import path does, then fingerprint it."""
    from app.services.fingerprint import max_hr_reference

    engine = app["create_engine"]("sqlite://")
    app["Base"].metadata.create_all(engine)
    session = app["Session"](engine, expire_on_commit=False)
    try:
        row, _ = app["store_activity"](session, parsed["activity"])
        session.flush()
        reference = max_hr_reference(session, row.athlete_id)
        result = app["fingerprint"].compute_fingerprint(session, row.id)
        zones = app["fingerprint"].athlete_zones(reference)
        payload = {key: value for key, value in result.items() if key in app["fingerprint"].METRICS or key in (
            "cadence_avg", "cadence_cv", "cadence_drift_pct", "hr_avg", "hr_max", "hr_drift_pct",
            "efficiency_first", "efficiency_last", "decoupling_pct", "pace_cv", "negative_split_pct",
            "stride_length_cm", "form_score", "zone_distribution", "running_time_s", "walking_time_s",
            "stopped_time_s", "hr_drift_bpm", "pace_drift_pct", "first_half_hr", "second_half_hr",
            "first_half_pace_s_km", "second_half_pace_s_km", "notes",
        )}
        section = {
            "inputs": {
                "max_hr_reference": reference,
                "zone_model": zones,
                "lap_count": len(row.laps),
                "athlete_id": row.athlete_id,
                "activity_source": row.source,
            },
            "fingerprint": payload,
            "activity": {
                "id": row.id,
                "name": row.name,
                "local_date": row.local_date,
                "distance_m": row.distance_m,
                "moving_time_s": row.moving_time_s,
                "pace_s_per_km": result.get("pace_s_per_km"),
                "source": result.get("source"),
            },
        }
    finally:
        session.close()
        engine.dispose()
    return section


# ----------------------------------------------------------- readiness stage


def readiness_section(app) -> dict[str, Any]:
    """Run the readiness scenarios from ``scenarios/readiness.json``."""
    scenario_file = SCENARIO_DIR / "readiness.json"
    scenarios = json.loads(scenario_file.read_text(encoding="utf-8"))["scenarios"]
    results = []
    for scenario in scenarios:
        engine = app["create_engine"]("sqlite://")
        app["Base"].metadata.create_all(engine)
        session = app["Session"](engine, expire_on_commit=False)
        try:
            for row in scenario["physiology"]:
                session.add(app["DailyPhysiology"](**row))
            session.flush()
            payload = app["readiness"].compute_readiness(session, scenario["day"])
            baselines = app["readiness"].baselines(
                [app["DailyPhysiology"](**row) for row in scenario["physiology"]], scenario["day"]
            )
        finally:
            session.close()
            engine.dispose()
        results.append({
            "id": scenario["id"],
            "description": scenario["description"],
            "day": scenario["day"],
            "baselines": baselines,
            "readiness": payload,
        })
    return {"scenarios": results}


# --------------------------------------------------------------- plan stage


def plan_section(app, parsed_cases: dict[str, Any]) -> dict[str, Any]:
    """Run the plan-engine scenarios from ``scenarios/plan.json``."""
    scenario_file = SCENARIO_DIR / "plan.json"
    scenario = json.loads(scenario_file.read_text(encoding="utf-8"))

    engine = app["create_engine"]("sqlite://")
    app["Base"].metadata.create_all(engine)
    session = app["Session"](engine, expire_on_commit=False)
    try:
        block = app["PlanBlock"](**scenario["block"])
        session.add(block)
        session.flush()
        phase = app["PlanPhase"](block_id=block.id, **scenario["phase"])
        session.add(phase)
        session.flush()
        week = app["PlanWeek"](phase_id=phase.id, **scenario["week"])
        session.add(week)
        session.flush()
        for row in scenario["physiology"]:
            session.add(app["DailyPhysiology"](**row))
        for spec in scenario["sessions"]:
            session.add(app["PlanSession"](block_id=block.id, week_id=week.id, **spec))
        session.flush()

        activities = []
        for spec in scenario["activities"]:
            if "case_id" in spec:
                activity = dict(parsed_cases[spec["case_id"]]["activity"])
                activity["name"] = spec.get("name", activity["name"])
                activity["sport_type"] = spec.get("sport_type", activity["sport_type"])
                activity["local_date"] = spec.get("date", activity["local_date"])
                activity["start_time"] = spec.get("start_time", activity["start_time"])
            else:
                activity = dict(spec["activity"])
            row, _ = app["store_activity"](session, activity)
            activities.append(row)
        session.flush()

        # Fingerprints exist for the matched activities, so the plan engine can
        # compare this run's cadence variability with the athlete's baseline.
        for row in activities:
            if row.sport_type in app["plan_engine"].RUN_SPORTS:
                try:
                    app["fingerprint"].compute_fingerprint(session, row.id)
                except Exception:  # pragma: no cover - defensive
                    pass
        session.flush()

        # The day the plan is judged against is fixed in the scenario, so a
        # session dated after it is *not yet* — the golden file does not move
        # with the wall clock.
        today = date.fromisoformat(scenario["today"])
        matches = app["plan_engine"].match_sessions(
            session, scenario["range"][0], scenario["range"][1], today
        )
        adherence = app["plan_engine"].adherence(
            session, scenario["range"][0], scenario["range"][1], today
        )
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
    return {
        "scenario": scenario["id"],
        "range": scenario["range"],
        "today": scenario["today"],
        "sessions": rows,
        "adherence": adherence,
    }


# ------------------------------------------------------------- the pipeline


def build_all() -> dict[str, Any]:
    """Produce every golden payload from the current implementation."""
    app = _import_app()
    manifest = json.loads((GOLDEN_DIR / "cases.json").read_text(encoding="utf-8"))

    payloads: dict[str, Any] = {}
    parsed_cases: dict[str, Any] = {}
    for case_id, meta in manifest["cases"].items():
        path = REPO / meta["file"]
        parsed = parse_case(case_id, path, app)
        parsed_cases[case_id] = parsed
        import_block = import_section(case_id, parsed, meta["sport"])
        fingerprint_block = fingerprint_section(case_id, parsed, app)
        payloads[f"{case_id}.json"] = {
            "case_id": case_id,
            "fixture": {
                "file": meta["file"],
                "sha256": meta["sha256"],
                "bytes": meta["bytes"],
                "sport": meta["sport"],
                "device": meta["device"],
                "watch": meta["watch"],
                "start": meta["start"],
                "date": meta["date"],
            },
            "import": import_block,
            "fingerprint": fingerprint_block,
        }

    payloads["readiness.json"] = readiness_section(app)
    payloads["plan.json"] = plan_section(app, parsed_cases)
    return {name: _round_floats(payload) for name, payload in payloads.items()}


def check_fixtures() -> list[str]:
    """Verify the committed fixtures still match their recorded hashes."""
    manifest = json.loads((GOLDEN_DIR / "cases.json").read_text(encoding="utf-8"))
    problems = []
    for case_id, meta in manifest["cases"].items():
        path = REPO / meta["file"]
        if not path.exists():
            problems.append(f"{case_id}: missing fixture {meta['file']}")
            continue
        digest = sha256_of(path)
        if digest != meta["sha256"]:
            problems.append(
                f"{case_id}: fixture hash moved\n  expected {meta['sha256']}\n  actual   {digest}"
            )
    return problems


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--check", action="store_true", help="regenerate and compare (exit non-zero on any drift)")
    group.add_argument("--generate", action="store_true", help="write the golden files")
    group.add_argument("--fixtures", action="store_true", help="write the FIT fixtures and cases.json")
    group.add_argument("--list", action="store_true", help="list the cases")
    args = parser.parse_args(argv)

    pin_environment()
    _reexec_into_venv()

    if args.fixtures:
        from make_fixtures import write_manifest
        manifest = write_manifest()
        for case_id, meta in manifest["cases"].items():
            print(f"{case_id:16s} {meta['bytes']:7d} bytes  sha256={meta['sha256']}")
        return 0

    if args.list:
        manifest = json.loads((GOLDEN_DIR / "cases.json").read_text(encoding="utf-8"))
        for case_id, meta in manifest["cases"].items():
            print(f"{case_id:16s} {meta['bytes']:7d} bytes  {meta['device']:16s} {meta['notes']}")
        return 0

    payloads = build_all()

    if args.generate:
        for name, payload in payloads.items():
            write_golden(OUTPUT_DIR / name, payload)
            print(f"wrote {OUTPUT_DIR.relative_to(REPO) / name}")
        return 0

    # --check (the default)
    problems = check_fixtures()
    for name, payload in payloads.items():
        path = OUTPUT_DIR / name
        if not path.exists():
            problems.append(f"{name}: missing golden file")
            continue
        expected = dump_json(payload)
        actual = path.read_text(encoding="utf-8")
        if expected != actual:
            problems.append(f"{name}: golden file moved")
            _show_diff(actual, expected)

    if problems:
        print("GOLDEN CHECK FAILED")
        for problem in problems:
            print(f"  - {problem}")
        return 1
    print(f"GOLDEN CHECK PASSED: {len(payloads)} golden files, {len(check_fixtures()) or 'all'} fixtures unchanged")
    return 0


def _show_diff(actual: str, expected: str) -> None:
    import difflib
    diff = list(difflib.unified_diff(
        actual.splitlines(), expected.splitlines(), "committed", "regenerated", lineterm="", n=1
    ))
    for line in diff[:40]:
        print(f"      {line}")


if __name__ == "__main__":
    raise SystemExit(main())
