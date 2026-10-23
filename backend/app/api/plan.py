"""Plan endpoints.

The importer (:mod:`app.ingest.plan_importer`) and the matching engine
(:mod:`app.services.plan_engine`) are pure logic over plain dictionaries. This
layer is the only place that knows the wire format, so it owns shaping every
response to the contract the frontend is written against.

Two rules shape the responses:

* A missing plan is never reported as zero adherence. ``has_plan`` is false
  and ``overall_pct`` is ``None``.
* A session always comes back with its result and the activity that fulfilled
  it, so the UI never has to join anything itself.
"""

from __future__ import annotations

import logging
from datetime import date, timedelta
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy import func, select
from sqlalchemy.orm import Session

from ..config import state
from ..db import get_db
from ..models import Activity
from ..models_phase2 import PlanBlock, PlanPhase, PlanResult, PlanSession, PlanWeek
from ..ingest.plan_importer import import_plan
from ..services import plan_engine
from ..services.analytics import _duration_label, _pace_s_per_km, demo_filter

log = logging.getLogger(__name__)

router = APIRouter(prefix="/plan", tags=["plan"])

ALL_VERDICTS = ("hit", "missed_pace", "short", "missed", "substituted", "bonus")


# ------------------------------------------------------------------ shapes


def _session(session: PlanSession) -> dict[str, Any]:
    return {
        "id": session.id,
        "week_id": session.week_id,
        "block_id": session.block_id,
        "date": session.date,
        "session_type": session.session_type,
        "title": session.title,
        "details": session.details,
        "target_distance_m": session.target_distance_m,
        "pace_low_s_km": session.pace_low_s_km,
        "pace_high_s_km": session.pace_high_s_km,
        "hr_zone_target": session.hr_zone_target,
        "status": session.status,
        "source": session.source,
        "notes": session.notes,
    }


def _activity(activity: Activity | None) -> dict[str, Any] | None:
    """The subset of an activity the plan surface needs to show."""
    if activity is None:
        return None
    pace = _pace_s_per_km(activity.distance_m, activity.moving_time_s)
    return {
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
        "moving_time_label": _duration_label(activity.moving_time_s),
        "elapsed_time_s": activity.elapsed_time_s,
        "elevation_gain_m": activity.elevation_gain_m,
        "avg_hr": activity.avg_hr,
        "max_hr": activity.max_hr,
        "calories": activity.calories,
        "avg_cadence": activity.avg_cadence,
        "max_cadence": activity.max_cadence,
        "avg_speed_ms": activity.avg_speed_ms,
        "max_speed_ms": activity.max_speed_ms,
        "pace_s_per_km": round(pace, 2) if pace else None,
        "pace_label": None,
        "perceived_exertion": activity.perceived_exertion,
        "device": activity.device,
        "gear": activity.gear,
        "is_trail": activity.is_trail,
        "is_commute": activity.is_commute,
        "visibility": activity.visibility,
        "has_geo": bool(activity.geo_points),
        "route_preview": None,
        "lap_count": len(activity.laps or []),
    }


def _result(row: dict[str, Any]) -> dict[str, Any]:
    """A :class:`PlanResult` shaped from one engine row."""
    return {
        "id": row.get("result_id"),
        "plan_session_id": row["session"]["id"],
        "activity_id": row["activity"]["id"] if row.get("activity") else None,
        "verdict": row.get("verdict", "missed"),
        "distance_delta_m": row.get("distance_delta_m"),
        "pace_delta_s_km": row.get("pace_delta_s_km"),
        "pace_band_hit": row.get("pace_band_hit"),
        "distance_hit": row.get("distance_hit"),
        "adherence_score": row.get("adherence_score"),
        "cause": row.get("cause"),
        "cause_confidence": row.get("cause_confidence"),
        "note": row.get("note"),
    }


def _sessions_with_results(db: Session, start: str, end: str) -> list[dict[str, Any]]:
    """Every planned session in a range, each with its result and activity."""
    rows = plan_engine.match_sessions(db, start, end)
    out: list[dict[str, Any]] = []
    for row in rows:
        session = row["session"]
        out.append(
            {
                **_session(
                    db.get(PlanSession, session["id"]) or _StubSession(session)
                ),
                "result": _result(row),
                "activity": _activity(
                    db.get(Activity, row["activity"]["id"]) if row.get("activity") else None
                ),
            }
        )
    return out


class _StubSession:
    """Adapter so an engine row can be shaped like an ORM row."""

    def __init__(self, payload: dict[str, Any]) -> None:
        self.id = payload.get("id")
        self.week_id = payload.get("week_id")
        self.block_id = payload.get("block_id")
        self.date = payload.get("date")
        self.session_type = payload.get("session_type", "easy")
        self.title = payload.get("title")
        self.details = payload.get("details")
        self.target_distance_m = payload.get("target_distance_m")
        self.pace_low_s_km = payload.get("pace_low_s_km")
        self.pace_high_s_km = payload.get("pace_high_s_km")
        self.hr_zone_target = payload.get("hr_zone_target")
        self.status = payload.get("status", "planned")
        self.source = payload.get("source", "manual")
        self.notes = payload.get("notes")


def _phase(phase: PlanPhase) -> dict[str, Any]:
    return {
        "id": phase.id,
        "block_id": phase.block_id,
        "name": phase.name,
        "start_date": phase.start_date,
        "end_date": phase.end_date,
        "target_weekly_km_min": phase.target_weekly_km_min,
        "target_weekly_km_max": phase.target_weekly_km_max,
        "target_sessions": phase.target_sessions,
        "pace_easy_low": phase.pace_easy_low,
        "pace_easy_high": phase.pace_easy_high,
        "pace_tempo_low": phase.pace_tempo_low,
        "pace_tempo_high": phase.pace_tempo_high,
        "pace_interval_low": phase.pace_interval_low,
        "pace_interval_high": phase.pace_interval_high,
        "pace_long_low": phase.pace_long_low,
        "pace_long_high": phase.pace_long_high,
        "hr_zone_target": phase.hr_zone_target,
        "notes": phase.notes,
    }


def _week_row(week: PlanWeek) -> dict[str, Any]:
    return {
        "id": week.id,
        "phase_id": week.phase_id,
        "week_number": week.week_number,
        "start_date": week.start_date,
        "end_date": week.end_date,
        "target_km": week.target_km,
        "key_workout": week.key_workout,
        "notes": week.notes,
    }


def _block_row(block: PlanBlock) -> dict[str, Any]:
    weeks = [w for p in block.phases for w in p.weeks]
    sessions = [s for w in weeks for s in w.sessions]
    return {
        "id": block.id,
        "name": block.name,
        "start_date": block.start_date,
        "end_date": block.end_date,
        "goal": block.goal,
        "status": block.status,
        "source": block.source,
        "notes": block.notes,
        "phase_count": len(block.phases),
        "week_count": len(weeks),
        "session_count": len(sessions),
    }


def _week_detail(db: Session, week: PlanWeek) -> dict[str, Any]:
    """A week with its sessions, results and the planned-vs-actual totals."""
    start = week.start_date or ""
    end = week.end_date or ""
    sessions = _sessions_with_results(db, start, end) if start and end else []

    planned_km = round(
        sum((s["target_distance_m"] or 0.0) for s in sessions) / 1000.0, 2
    )
    actual_km = round(
        sum((s["activity"]["distance_m"] if s.get("activity") else 0.0) for s in sessions)
        / 1000.0,
        2,
    )
    completed = sum(1 for s in sessions if s.get("activity"))
    phase = db.get(PlanPhase, week.phase_id)
    return {
        "week": _week_row(week),
        "phase": _phase(phase) if phase else None,
        "sessions": sessions,
        "planned_km": planned_km,
        "actual_km": actual_km,
        "planned_count": len(sessions),
        "completed_count": completed,
        "adherence_pct": round(completed / len(sessions) * 100, 1) if sessions else None,
    }


def _block_detail(db: Session, block: PlanBlock) -> dict[str, Any]:
    phases = []
    for phase in block.phases:
        phases.append(
            {**_phase(phase), "weeks": [_week_detail(db, w) for w in phase.weeks]}
        )
    return {"block": _block_row(block), "phases": phases}


def _active_block(db: Session) -> PlanBlock | None:
    block = db.execute(
        select(PlanBlock)
        .where(PlanBlock.status == "active")
        .order_by(PlanBlock.start_date.desc())
    ).scalars().first()
    if block is None:
        block = db.execute(
            select(PlanBlock).order_by(PlanBlock.start_date.desc())
        ).scalars().first()
    return block


def _plan_date_range(db: Session) -> tuple[str, str]:
    """The span of the active plan, widened so results include the edges."""
    block = _active_block(db)
    if block is None:
        today = date.today()
        return today.isoformat(), today.isoformat()
    start = block.start_date or date.today().isoformat()
    end = block.end_date or date.today().isoformat()
    return start, end


# ----------------------------------------------------------------- routes


class ImportBody(BaseModel):
    format: str = "markdown"
    content: str
    name: str | None = None
    # When "activities", shift the imported plan so its final week lines up
    # with the athlete's most recent activity. A plan whose dates do not
    # overlap the training history cannot be compared to anything, and an
    # unanchored import against an existing history is almost always off by
    # however long the history is.
    anchor: str = "activities"


@router.get("/blocks")
def list_blocks(db: Session = Depends(get_db)) -> dict[str, Any]:
    blocks = db.execute(
        select(PlanBlock).order_by(PlanBlock.start_date.desc())
    ).scalars().all()
    return {"items": [_block_row(b) for b in blocks]}


@router.post("/blocks")
def create_block(payload: dict[str, Any], db: Session = Depends(get_db)) -> dict[str, Any]:
    fields = {
        key: value
        for key, value in payload.items()
        if key in {"name", "start_date", "end_date", "goal", "status", "source", "notes"}
    }
    fields.setdefault("name", "Training block")
    fields.setdefault("start_date", date.today().isoformat())
    block = PlanBlock(**fields)
    db.add(block)
    db.commit()
    db.refresh(block)
    return _block_detail(db, block)


@router.get("/blocks/{block_id}")
def get_block(block_id: int, db: Session = Depends(get_db)) -> dict[str, Any]:
    block = db.get(PlanBlock, block_id)
    if block is None:
        raise HTTPException(status_code=404, detail="Plan block not found.")
    return _block_detail(db, block)


@router.post("/blocks/{block_id}/phases")
def add_phase(
    block_id: int, payload: dict[str, Any], db: Session = Depends(get_db)
) -> dict[str, Any]:
    if db.get(PlanBlock, block_id) is None:
        raise HTTPException(status_code=404, detail="Plan block not found.")
    fields = {
        key: value
        for key, value in payload.items()
        if hasattr(PlanPhase, key) and key not in ("id", "block_id")
    }
    phase = PlanPhase(block_id=block_id, **fields)
    db.add(phase)
    db.commit()
    db.refresh(phase)
    return _phase(phase)


@router.post("/phases/{phase_id}/weeks")
def add_week(
    phase_id: int, payload: dict[str, Any], db: Session = Depends(get_db)
) -> dict[str, Any]:
    if db.get(PlanPhase, phase_id) is None:
        raise HTTPException(status_code=404, detail="Plan phase not found.")
    fields = {
        key: value
        for key, value in payload.items()
        if hasattr(PlanWeek, key) and key not in ("id", "phase_id")
    }
    week = PlanWeek(phase_id=phase_id, **fields)
    db.add(week)
    db.commit()
    db.refresh(week)
    return _week_row(week)


@router.post("/weeks/{week_id}/sessions")
def add_session(
    week_id: int, payload: dict[str, Any], db: Session = Depends(get_db)
) -> dict[str, Any]:
    week = db.get(PlanWeek, week_id)
    if week is None:
        raise HTTPException(status_code=404, detail="Plan week not found.")
    fields = {
        key: value
        for key, value in payload.items()
        if hasattr(PlanSession, key) and key not in ("id", "week_id", "block_id")
    }
    session = PlanSession(week_id=week_id, block_id=week.phase.block_id, **fields)
    db.add(session)
    db.commit()
    db.refresh(session)
    return _session(session)


@router.post("/import")
def import_document(body: ImportBody, db: Session = Depends(get_db)) -> dict[str, Any]:
    """Import a plan document (markdown, csv or json) as a new block."""
    try:
        parsed = import_plan(body.content, body.format, body.name)
    except Exception as exc:
        log.warning("Plan import failed: %s", exc)
        return {
            "ok": False,
            "block_id": None,
            "name": body.name,
            "phases": 0,
            "weeks": 0,
            "sessions": 0,
            "warnings": [],
            "error": str(exc),
        }

    raw = dict(parsed["block"])
    raw.setdefault("start_date", date.today().isoformat())
    raw["source"] = "imported"
    block = PlanBlock(
        **{
            key: value
            for key, value in raw.items()
            if key in {"name", "start_date", "end_date", "goal", "status", "source", "notes"}
        }
    )
    db.add(block)
    db.flush()

    phase_map: list[PlanPhase] = []
    for payload in parsed["phases"]:
        phase = PlanPhase(
            block_id=block.id,
            **{
                key: value
                for key, value in payload.items()
                if hasattr(PlanPhase, key) and key not in ("id", "block_id")
            },
        )
        db.add(phase)
        db.flush()
        phase_map.append(phase)

    week_map: dict[int, PlanWeek] = {}
    for payload in parsed["weeks"]:
        index = payload.get("phase_index", 0)
        phase = (
            phase_map[index]
            if 0 <= index < len(phase_map)
            else (phase_map[0] if phase_map else None)
        )
        if phase is None:
            phase = PlanPhase(block_id=block.id, name="Imported")
            db.add(phase)
            db.flush()
            phase_map.append(phase)
        week = PlanWeek(
            phase_id=phase.id,
            **{
                key: value
                for key, value in payload.items()
                if key
                in {"week_number", "start_date", "end_date", "target_km", "key_workout", "notes"}
            },
        )
        db.add(week)
        db.flush()
        week_map[payload["week_number"]] = week

    for payload in parsed["sessions"]:
        week = next(
            (
                w
                for w in week_map.values()
                if w.start_date
                and w.end_date
                and w.start_date <= payload["date"] <= w.end_date
            ),
            None,
        )
        db.add(
            PlanSession(
                week_id=week.id if week else None,
                block_id=block.id,
                **{
                    key: value
                    for key, value in payload.items()
                    if key
                    in {
                        "date",
                        "session_type",
                        "title",
                        "details",
                        "target_distance_m",
                        "pace_low_s_km",
                        "pace_high_s_km",
                        "hr_zone_target",
                        "status",
                        "source",
                        "notes",
                    }
                },
            )
        )

    db.commit()
    db.refresh(block)

    if body.anchor == "activities":
        _anchor_to_history(db, block, phase_map, week_map, parsed)

    db.commit()
    db.refresh(block)
    return {
        "ok": True,
        "block_id": block.id,
        "name": block.name,
        "phases": len(phase_map),
        "weeks": len(week_map),
        "sessions": len(parsed["sessions"]),
        "warnings": parsed["warnings"],
        "error": None,
    }


def _anchor_to_history(
    db: Session,
    block: PlanBlock,
    phase_map: list[PlanPhase],
    week_map: dict[int, PlanWeek],
    parsed: dict[str, Any],
) -> None:
    """Shift an imported plan so its final week ends with the latest activity.

    Without this, a plan written for a past season never overlaps the runs it
    is supposed to govern, and every session reads as "missed". Shifting the
    whole block by whole weeks keeps every internal relationship intact.
    """
    stmt = select(func.max(Activity.local_date))
    if not state.demo_visible:
        # Anchor to the athlete's real runs, not the synthetic demo set.
        stmt = stmt.where(Activity.source != "demo")
    latest = db.execute(stmt).scalar()
    if not latest:
        return
    try:
        latest_date = date.fromisoformat(latest)
    except (TypeError, ValueError):
        return

    weeks = list(week_map.values())
    if not weeks or not block.end_date:
        return
    try:
        plan_end = date.fromisoformat(block.end_date)
    except (TypeError, ValueError):
        return

    latest_week_end = latest_date + timedelta(days=6 - latest_date.weekday())
    shift_days = (latest_week_end - plan_end).days
    shift = round(shift_days / 7) * 7
    if shift == 0:
        return

    def move(value: str | None) -> str | None:
        if not value:
            return value
        try:
            return (date.fromisoformat(value) + timedelta(days=shift)).isoformat()
        except (TypeError, ValueError):
            return value

    block.start_date = move(block.start_date)
    block.end_date = move(block.end_date)
    for phase in phase_map:
        phase.start_date = move(phase.start_date)
        phase.end_date = move(phase.end_date)
    for week in week_map.values():
        week.start_date = move(week.start_date)
        week.end_date = move(week.end_date)
    for session in db.execute(
        select(PlanSession).where(PlanSession.block_id == block.id)
    ).scalars():
        session.date = move(session.date) or session.date
    log.info("Anchored plan block %s by %+d days to the activity history", block.id, shift)


@router.get("/current")
def current(db: Session = Depends(get_db)) -> dict[str, Any]:
    """The active block and the week we are in."""
    block = _active_block(db)
    if block is None:
        return {"block": None, "week": None}
    detail = _block_detail(db, block)
    week = _current_week(db, block)
    return {"block": detail, "week": week}


def _current_week(db: Session, block: PlanBlock) -> dict[str, Any] | None:
    today = date.today()
    weeks = [w for p in block.phases for w in p.weeks]
    if not weeks:
        return None
    dated = [w for w in weeks if w.start_date and w.end_date]
    for week in dated:
        if week.start_date <= today.isoformat() <= week.end_date:
            return _week_detail(db, week)
    nearest = min(dated, key=lambda w: abs((date.fromisoformat(w.start_date) - today).days)) if dated else None
    return _week_detail(db, nearest) if nearest else None


@router.get("/week")
def get_week(day: str = Query(..., alias="date"), db: Session = Depends(get_db)) -> dict[str, Any]:
    """The plan week containing a date."""
    try:
        target = date.fromisoformat(day)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail="date must be YYYY-MM-DD.") from exc
    start = (target - timedelta(days=target.weekday())).isoformat()
    end = (target + timedelta(days=6 - target.weekday())).isoformat()

    week = (
        db.execute(
            select(PlanWeek)
            .where(PlanWeek.start_date <= end, PlanWeek.end_date >= start)
            .order_by(PlanWeek.week_number)
        )
        .scalars()
        .first()
    )
    if week is None:
        return {
            "week": None,
            "phase": None,
            "sessions": _sessions_with_results(db, start, end),
            "planned_km": 0.0,
            "actual_km": 0.0,
            "planned_count": 0,
            "completed_count": 0,
            "adherence_pct": None,
        }
    return _week_detail(db, week)


@router.get("/results")
def results(
    date_from: str | None = Query(None, alias="from"),
    date_to: str | None = Query(None, alias="to"),
    db: Session = Depends(get_db),
) -> dict[str, Any]:
    start, end = _plan_date_range(db)
    rows = plan_engine.match_sessions(
        db, date_from or start, date_to or end
    )
    return {"items": [_result(row) for row in rows]}


@router.get("/adherence")
def get_adherence(
    date_from: str | None = Query(None, alias="from"),
    date_to: str | None = Query(None, alias="to"),
    db: Session = Depends(get_db),
) -> dict[str, Any]:
    start, end = _plan_date_range(db)
    summary = plan_engine.adherence(db, date_from or start, date_to or end)
    counts = summary.get("by_verdict") or {}
    summary["by_verdict"] = {verdict: counts.get(verdict, 0) for verdict in ALL_VERDICTS}
    return summary


@router.get("/today")
def get_today(db: Session = Depends(get_db)) -> dict[str, Any]:
    """Today's plan, today's readiness, and one actionable recommendation."""
    today = date.today().isoformat()
    block = _active_block(db)
    rows = plan_engine.match_sessions(db, today, today)
    sessions = [
        {
            **_session(db.get(PlanSession, row["session"]["id"]) or _StubSession(row["session"])),
            "result": _result(row),
            "activity": _activity(
                db.get(Activity, row["activity"]["id"]) if row.get("activity") else None
            ),
        }
        for row in rows
    ]

    week = (
        db.execute(
            select(PlanWeek)
            .where(PlanWeek.start_date <= today, PlanWeek.end_date >= today)
            .order_by(PlanWeek.week_number)
        )
        .scalars()
        .first()
    )

    readiness = None
    try:
        from ..services.readiness import compute_readiness

        readiness = compute_readiness(db, today)
    except Exception as exc:  # pragma: no cover - readiness is optional
        log.debug("Readiness unavailable for today: %s", exc)

    if not sessions:
        return {
            "date": today,
            "session": None,
            "week": _week_detail(db, week) if week else None,
            "readiness": readiness,
            "recommendation": (
                "No session is planned for today. Import a plan to get a "
                "recommendation here."
            )
            if block is None
            else "Nothing is planned for today — treat it as a rest day.",
            "rationale": (
                "There is no planned session to compare against or adjust."
                if block is None
                else "Your plan does not schedule a session for today."
            ),
            "has_plan": block is not None,
            "has_session": False,
        }

    session = sessions[0]
    recommendation, rationale = _recommend(db, session, week, today, readiness)
    return {
        "date": today,
        "session": session,
        "week": _week_detail(db, week) if week else None,
        "readiness": readiness,
        "recommendation": recommendation,
        "rationale": rationale,
        "has_plan": True,
        "has_session": True,
    }


def _recommend(
    db: Session,
    session: dict[str, Any],
    week: PlanWeek | None,
    today: str,
    readiness: dict[str, Any] | None,
) -> tuple[str, str]:
    """One sentence of advice, with the reason behind it."""
    reasons: list[str] = []

    score = (readiness or {}).get("score")
    if score is not None and score < 50:
        reasons.append(
            f"readiness is {score:.0f}/100 ({readiness.get('grade')})"
        )

    behind = False
    if week is not None and week.target_km and week.start_date:
        logged = (
            db.execute(
                select(func.coalesce(func.sum(Activity.distance_m), 0.0)).where(
                    Activity.local_date >= week.start_date,
                    Activity.local_date <= today,
                    Activity.sport_type.in_(plan_engine.RUN_SPORTS),
                )
            )
            .scalar_one()
            or 0.0
        )
        elapsed = max(1, (date.fromisoformat(today) - date.fromisoformat(week.start_date)).days + 1)
        expected = week.target_km * min(elapsed / 7.0, 1.0)
        behind = logged / 1000.0 < expected * 0.75
        if behind:
            reasons.append(
                f"only {logged / 1000.0:.1f} km of this week's {week.target_km:.0f} km is logged"
            )

    title = session.get("title") or session.get("session_type") or "session"
    kind = session.get("session_type")
    if kind in ("rest", "strength"):
        # Rest and strength are prescriptions too, but "complete the planned
        # rest as written" is not advice anyone needs.
        if reasons:
            return (
                f"Keep today's {kind} light — {reasons[0]}.",
                "Adjusted because " + "; and ".join(reasons) + ".",
            )
        return (
            f"Nothing running is scheduled today; treat it as a planned {kind} day.",
            "Your plan schedules no running for today.",
        )
    if reasons:
        return (
            f"Adjust today's {title} — {reasons[0]}."
            + (f" Also, {reasons[1]}." if len(reasons) > 1 else "")
            + " Keep it easy and make up the volume later in the week.",
            "Adjusted because " + "; and ".join(reasons) + ".",
        )
    return (
        f"Complete the planned {title} as written.",
        "No readiness dip and the week's volume is on track.",
    )
