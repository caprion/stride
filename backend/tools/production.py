"""Production setup for a Stride instance.

Run as a module:

    cd ~/stride/backend && .venv/bin/python -m tools.production setup
    cd ~/stride/backend && .venv/bin/python -m tools.production status
    cd ~/stride/backend && .venv/bin/python -m tools.production purge-demo
    cd ~/stride/backend && .venv/bin/python -m tools.production import-plan

`setup` is the first-run path for a real deployment: it creates the database,
resolves the Garmin Personal API Key from Azure Key Vault, removes the synthetic
demo activities, pulls the athlete's real history and wellness data, and imports
the training plan.
"""

from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

logging.basicConfig(level=logging.INFO, format="%(levelname)-7s %(message)s")
log = logging.getLogger("production")

DEFAULT_PLAN_GLOB = "~/running/seasons/*/plan.md"


# --------------------------------------------------------------------- utils


def _db():
    from app.db import SessionLocal

    return SessionLocal()


def _summary(db) -> dict:
    from app.models import Activity
    from app.models_phase2 import DailyPhysiology, PlanSession, RunFingerprint

    return {
        "activities": db.query(Activity).count(),
        "from_garmin": db.query(Activity).filter(Activity.source != "demo").count(),
        "demo": db.query(Activity).filter(Activity.source == "demo").count(),
        "physiology_days": db.query(DailyPhysiology).count(),
        "fingerprints": db.query(RunFingerprint).count(),
        "plan_sessions": db.query(PlanSession).count(),
    }


def _print_summary(summary: dict) -> None:
    print("\n  instance")
    for key, value in summary.items():
        print(f"    {key:18s} {value}")


# ------------------------------------------------------------------ commands


def cmd_prune_sync_history(args) -> int:
    """Keep the sync log short and drop failures that have been resolved.

    The sync history is an operational log, not an audit record. A run that
    failed because of a bug which has since been fixed is noise on the
    Settings page — and it can be a lot of noise, because a broken sync
    repeats on the schedule.

    A failure is *resolved* once a later run has succeeded, so those rows go.
    The most recent failure is kept, because it is still the truth about the
    last attempt.
    """
    from app.models import SyncRun

    db = _db()
    try:
        total = db.query(SyncRun).count()
        rows = db.query(SyncRun).order_by(SyncRun.started_at.desc()).all()
        keep_ids = {row.id for row in rows[: args.keep]}

        latest_ok = next(
            (row for row in rows if (row.status or "").lower() == "ok"),
            None,
        )
        removed = 0
        for row in rows:
            status = (row.status or "").lower()
            # A failure is resolved once a later run has succeeded.
            if (
                status == "error"
                and latest_ok is not None
                and row.started_at < latest_ok.started_at
            ):
                db.delete(row)
                removed += 1

        for row in rows[args.keep :]:
            if row.id in keep_ids:
                continue
            db.delete(row)
            removed += 1
        db.commit()
    finally:
        db.close()
    print(f"sync history: kept {min(args.keep, total)}, removed {removed} of {total}")
    return 0


def _recompute_fingerprints() -> tuple[int, int]:
    """Recompute every activity's fingerprint and persist it.

    ``compute_many`` deliberately leaves the transaction to its caller — it
    flushes but never commits — so this has to commit explicitly. Forgetting
    that looks exactly like success: the count is right inside the process
    and the rows are gone after it exits.
    """
    from app.db import SessionLocal
    from app.models import Activity
    from app.services.fingerprint import compute_many

    db = SessionLocal()
    try:
        ids = [
            row[0]
            for row in db.query(Activity.id).filter(Activity.source == "garmin_personal").all()
        ]
        items = compute_many(db, ids).get("items") or []
        db.commit()
    finally:
        db.close()
    computed = sum(1 for item in items if item)
    return computed, len(ids)


def cmd_backfill_fit(args) -> int:
    """Re-download the FIT file for activities that only have chart data."""
    from app.services.personal_sync import backfill_fit_files

    result = backfill_fit_files(limit=args.limit)
    print(f"backfill: {result['rebuilt']} rebuilt, {result['failed']} unavailable")

    computed, total = _recompute_fingerprints()
    print(f"fingerprints: {computed} of {total} activities")
    return 0 if result["failed"] == 0 else 1


def cmd_status(_args) -> int:
    from app.config import state

    db = _db()
    try:
        summary = _summary(db)
    finally:
        db.close()
    print("\n  Garmin")
    print(f"    {'connected':18s} {state.garmin_connected}")
    print(f"    {'method':18s} {state.garmin_method}")
    print(f"    {'key source':18s} {'key_vault' if state.personal_api_key else 'none'}")
    print(f"    {'last sync':18s} {state.last_sync_at}")
    _print_summary(summary)
    return 0


def cmd_purge_demo(_args) -> int:
    """Remove the synthetic demo activities from a production instance."""
    from app.models import Activity, ActivityLap, ActivityStream, HrZone
    from app.models_phase2 import RunFingerprint

    db = _db()
    try:
        demo_ids = [
            row[0] for row in db.query(Activity.id).filter(Activity.source == "demo").all()
        ]
        if not demo_ids:
            print("no demo activities to remove")
            return 0
        for model in (ActivityLap, ActivityStream, HrZone, RunFingerprint):
            column = model.activity_id
            db.query(model).filter(column.in_(demo_ids)).delete(synchronize_session=False)
        removed = db.query(Activity).filter(Activity.id.in_(demo_ids)).delete(
            synchronize_session=False
        )
        db.commit()
        print(f"removed {removed} demo activities and their children")
    finally:
        db.close()
    return 0


def cmd_import_plan(args) -> int:
    """Import a training plan document and anchor it to the activity history."""
    from app.api.plan import import_document
    from app.ingest.plan_importer import import_plan as parse_plan
    from app.models_phase2 import PlanBlock

    pattern = args.path
    if "*" in pattern:
        paths = sorted(Path("~").expanduser().glob(pattern.replace("~/", "", 1)))
    else:
        candidate = Path(pattern).expanduser()
        paths = [candidate] if candidate.exists() else []
    if not paths:
        print(f"no plan document matched {args.path}")
        return 1

    path = paths[-1]
    text = path.read_text()
    db = _db()
    try:
        existing = db.query(PlanBlock).filter(PlanBlock.status == "active").all()
        for block in existing:
            block.status = "archived"
        db.commit()
        result = import_document(
            _ImportBody(
                format="markdown",
                content=text,
                name=_plan_name(text, path),
                anchor="activities",
            ),
            db,
        )
        db.commit()
    finally:
        db.close()
    print(f"imported {path}: {result['phases']} phases, {result['weeks']} weeks, "
          f"{result['sessions']} sessions")
    for warning in result.get("warnings", [])[:5]:
        print(f"  ! {warning}")
    return 0 if result.get("ok") else 1


def _plan_name(text: str, path: Path) -> str:
    """Name the block after the campaign, not after the file.

    The document's title is "20-Week Integrated Training Plan: HM Sub-2:00
    Campaign" — the campaign is the part after the last colon. Falling back
    to the file name would leave a block called "plan".
    """
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("# "):
            title = stripped[2:].strip()
            if ": " in title:
                return title.rsplit(": ", 1)[1].strip()
            return title
    return path.parent.name.replace("-", " ").title()


class _ImportBody:
    """Minimal stand-in for the API's request body."""

    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)


def cmd_setup(args) -> int:
    """First run for a real deployment."""
    from app.config import DATA_DIR, state
    from app.db import init_db
    from app.secrets import resolve_garmin_personal_key

    log.info("data directory: %s", DATA_DIR)
    init_db()
    log.info("schema ready")

    key, source = resolve_garmin_personal_key()
    if not key:
        log.error(
            "no Garmin key found. Set GARMIN_PERSONAL_API_KEY, or put one in "
            "Key Vault (%s/%s).",
            "claw-agent",
            "garmin-api",
        )
        return 1
    log.info("Garmin key resolved from %s", source)

    from app.ingest.garmin_personal import GarminPersonalClient

    client = GarminPersonalClient(key)
    try:
        client.validate()
    except Exception as exc:
        log.error("the Garmin key did not validate: %s", exc)
        return 1
    state.update(personal_api_key=key, garmin_connected=True, garmin_method="personal")
    log.info("Garmin connected (Personal API)")

    cmd_purge_demo(args)

    if args.skip_sync:
        log.info("skipping the history pull as requested")
    else:
        from app.services.personal_sync import run

        result = run(days=args.days)
        log.info(
            "pulled %d activities (%d new) and %d days of health data",
            result["activities_found"],
            result["activities_new"],
            result["wellness_days"],
        )

    if not args.skip_plan:
        cmd_import_plan(args)

    if not args.skip_fingerprints:
        computed, total = _recompute_fingerprints()
        log.info("fingerprints: %d of %d activities", computed, total)

    db = _db()
    try:
        summary = _summary(db)
    finally:
        db.close()
    _print_summary(summary)
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="tools.production", description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    setup = sub.add_parser("setup", help="first run for a real deployment")
    setup.add_argument("--days", type=int, default=365, help="how much history to pull")
    setup.add_argument("--path", default=DEFAULT_PLAN_GLOB, help="plan document to import")
    setup.add_argument("--skip-sync", action="store_true")
    setup.add_argument("--skip-plan", action="store_true")
    setup.add_argument("--skip-fingerprints", action="store_true")
    setup.set_defaults(func=cmd_setup)

    status = sub.add_parser("status", help="summarise the instance")
    status.set_defaults(func=cmd_status)

    purge = sub.add_parser("purge-demo", help="remove the synthetic demo activities")
    purge.set_defaults(func=cmd_purge_demo)

    plan = sub.add_parser("import-plan", help="import a training plan")
    plan.add_argument("--path", default=DEFAULT_PLAN_GLOB)
    plan.set_defaults(func=cmd_import_plan)

    backfill = sub.add_parser(
        "backfill-fit", help="re-download the FIT for activities that only have chart data"
    )
    backfill.add_argument("--limit", type=int, default=None)
    backfill.set_defaults(func=cmd_backfill_fit)

    prune = sub.add_parser("prune-sync-history", help="keep only the most recent sync runs")
    prune.add_argument("--keep", type=int, default=20)
    prune.set_defaults(func=cmd_prune_sync_history)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
