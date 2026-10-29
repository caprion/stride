"""The sync engine.

Responsibilities
----------------
* Pick the right Garmin client for the currently configured auth method.
* Work around Garmin's datacentre-IP throttling by routing through the home
  exit node when needed.
* Pull activities (+ splits, streams, route) and persist them idempotently.
* Record every run in ``sync_run`` so the UI can show what happened.
"""

from __future__ import annotations

import logging
import threading
from datetime import datetime, timezone
from typing import Any, Callable

from sqlalchemy.orm import Session

from ..config import DATA_DIR, state
from ..db import SessionLocal
from ..ingest.garmin_account import GarminAccountClient, GarminAccountError
from ..ingest.garmin_oauth import (
    GarminApiError,
    GarminAuthError,
    GarminOAuthClient,
)
from ..models import SyncRun
from .. import network
from .persistence import store_activity

log = logging.getLogger(__name__)

MAX_SYNC_DAYS = 365
MAX_HISTORY_MONTHS = 12


def _bounded_sync_days(value: Any, default: int) -> int:
    try:
        days = int(value)
    except (TypeError, ValueError, OverflowError):
        days = default
    return max(1, min(days, MAX_SYNC_DAYS))

_sync_lock = threading.Lock()
_sync_state: dict[str, Any] = {"running": False, "started_at": None, "message": None}


def status() -> dict[str, Any]:
    return dict(_sync_state)


def _is_throttle_error(exc: Exception) -> bool:
    text = str(exc).lower()
    return "rate limit" in text or "429" in text or "bot challenge" in text or "403" in text


def _switch_to_exit_node(reason: str) -> bool:
    """Route egress via the residential exit node. Returns success."""
    node = state.exit_node
    if not node:
        log.warning("No exit node configured; cannot work around %s", reason)
        return False
    ok, out = network.set_exit_node(node)
    if ok:
        log.info("Switched network route to exit node %s (%s)", node, reason)
    else:
        log.warning("Could not switch to exit node: %s", out)
    return ok


def _fetch_with_oauth(days: int) -> list[dict]:
    client = GarminOAuthClient(
        state.consumer_key or "",
        state.consumer_secret or "",
        state.oauth_token,
        state.oauth_token_secret,
    )
    activities = client.fetch_activities(days=days)
    for activity in activities:
        external_id = activity["external_id"]
        try:
            detail = client.activity_details(external_id)
            splits = client.activity_splits(external_id)
        except Exception as exc:
            log.debug("Detail fetch failed for %s: %s", external_id, exc)
            continue
        activity.update(_normalise_oauth_detail(detail, splits))
    return activities


def _normalise_oauth_detail(detail: dict, splits: dict) -> dict[str, Any]:
    """Fold Garmin's activity details + splits payload into canonical fields."""
    from ..ingest.garmin_oauth import points_from_encoded
    from ..ingest.normalize import safe_float, safe_int

    out: dict[str, Any] = {}
    laps: list[dict[str, Any]] = []
    for index, lap in enumerate(splits.get("lapDTOs") or []):
        if not isinstance(lap, dict):
            continue
        laps.append(
            {
                "lap_index": safe_int(lap.get("lapIndex"), index + 1) or index + 1,
                "distance_m": safe_float(lap.get("distance"), 0.0) or 0.0,
                "duration_s": safe_float(lap.get("duration"), 0.0) or 0.0,
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
    if laps:
        out["laps"] = laps

    # Route
    polyline = None
    polyline_dto = detail.get("activityPolylineDTO") or {}
    entries = polyline_dto.get("polylineDTOList") or polyline_dto.get("polylineDTO") or []
    if isinstance(entries, list):
        for entry in entries:
            if isinstance(entry, dict) and entry.get("polyline"):
                polyline = entry["polyline"]
                break
    if polyline:
        out["geo_points"] = points_from_encoded(polyline)

    # Streams
    streams: dict[str, Any] = {}
    hr = detail.get("activityDetailDTO", {}).get("heartRateDTOs") if isinstance(detail, dict) else None
    if isinstance(hr, list) and hr:
        seconds = [int(row.get("timestamp", index)) for index, row in enumerate(hr)]
        values = [safe_float(row.get("value")) for row in hr]
        if any(v is not None for v in values):
            streams["hr"] = {"seconds": seconds, "values": values, "unit": "bpm"}

    if streams:
        out["streams"] = streams
    return out


def _fetch_with_account(days: int) -> list[dict]:
    tokenstore = str(DATA_DIR / "garmin_tokens")
    client = GarminAccountClient(state.garmin_email or "", state.garmin_password or "", tokenstore)
    client.login()
    activities = client.fetch_activities(days=days)
    for activity in activities:
        try:
            activity.update(client.fetch_detail(activity["external_id"]))
        except Exception as exc:
            log.debug("Detail fetch failed for %s: %s", activity["external_id"], exc)
    return activities


def _persist(activities: list[dict], db: Session) -> tuple[int, int]:
    created = 0
    updated = 0
    for activity in activities:
        _, is_new = store_activity(db, activity)
        if is_new:
            created += 1
        else:
            updated += 1
    db.commit()
    return created, updated


def run_sync(
    days: int | None = None,
    background: bool = True,
    history: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Kick off a sync. Returns immediately with a status dict.

    ``history`` loads a chosen span of history rather than the default
    window — see :func:`app.api.misc.load_history`.
    """
    if _sync_lock.locked():
        return {"started": False, "reason": "A sync is already running."}
    if background:
        thread = threading.Thread(
            target=_run_sync_blocking, args=(days, history), daemon=True
        )
        thread.start()
        return {"started": True}
    _run_sync_blocking(days, history)
    return {"started": True, **status()}


def _run_sync_blocking(days: int | None = None, history: dict[str, Any] | None = None) -> None:
    days = _bounded_sync_days(days, _bounded_sync_days(state.sync_days, 90))
    if history:
        history = dict(history)
        if "activities" in history:
            if history["activities"] is not None:
                try:
                    history["activities"] = max(1, min(int(history["activities"]), MAX_HISTORY_MONTHS))
                except (TypeError, ValueError, OverflowError):
                    history["activities"] = 12
        if "health_days" in history:
            history["health_days"] = _bounded_sync_days(history["health_days"], 30)
    with _sync_lock:
        _sync_state.update({"running": True, "started_at": datetime.now(timezone.utc).isoformat(), "message": None})
        db = SessionLocal()
        run = SyncRun(source=state.garmin_method or "garmin", days=days, status="running")
        db.add(run)
        db.commit()

        try:
            method = state.garmin_method
            if method == "personal":
                # The Personal API pulls activities and wellness together.
                from .personal_sync import run as personal_run

                def progress(done: int, total: int) -> None:
                    _sync_state["message"] = f"Pulled {done} of {total} activities…"

                result = personal_run(days=days, history=history, progress=progress)
                run.activities_found = result["activities_found"]
                run.activities_new = result["activities_new"]
                run.activities_updated = result["activities_updated"]
                run.status = "ok"
                run.message = (
                    f"Synced {result['activities_new']} new and "
                    f"{result['activities_updated']} updated activities, plus "
                    f"{result['wellness_days']} days of health data "
                    f"({result.get('wellness_days_skipped', 0)} days already held, "
                    f"{result.get('wellness_requests', 0)} requests)."
                )
                state.update(last_sync_at=datetime.now(timezone.utc).isoformat())
                if result["activities_new"] and state.demo_visible:
                    state.update(demo_visible=False)
                    run.message += " Demo activities hidden."
                _sync_state.update({"running": False, "message": run.message})
                log.info(run.message)
                return
            if method == "developer_key":
                fetcher = _fetch_with_oauth
            elif method == "account":
                fetcher = _fetch_with_account
            else:
                raise GarminAuthError(
                    "No Garmin connection is configured. Add a Personal API key, a "
                    "developer key or an account in Settings, or import FIT files directly."
                )

            try:
                activities = fetcher(days)
            except (GarminAuthError, GarminApiError, GarminAccountError) as exc:
                if _is_throttle_error(exc):
                    log.info("Garmin throttled us; retrying via exit node")
                    if _switch_to_exit_node(str(exc)):
                        activities = fetcher(days)
                    else:
                        raise
                else:
                    raise

            run.activities_found = len(activities)
            created, updated = _persist(activities, db)
            run.activities_new = created
            run.activities_updated = updated
            run.status = "ok"
            run.message = f"Synced {created} new and {updated} updated activities."
            state.update(last_sync_at=datetime.now(timezone.utc).isoformat())
            # Real activities arrived: retire the synthetic demo set so the
            # feed does not mix fake runs with real ones.
            if created and state.demo_visible:
                state.update(demo_visible=False)
                run.message += " Demo activities hidden."
            _sync_state.update({"running": False, "message": run.message})
            log.info(run.message)

            # Physiology is a separate pull: it needs the account session and
            # the Health API endpoints, and it must never fail the activity sync.
            _sync_health_in_background()
        except Exception as exc:
            db.rollback()
            run.status = "error"
            run.error = str(exc)[:2000]
            _sync_state.update({"running": False, "message": None, "error": str(exc)})
            log.error("Sync failed: %s", exc)
        finally:
            run.finished_at = datetime.now(timezone.utc)
            db.commit()
            db.close()
            _sync_state["running"] = False


def _sync_health_in_background() -> None:
    """Pull sleep/HRV/body-battery data without blocking the activity sync.

    Physiology failures must never fail the sync that produced the activities,
    so this runs on its own thread and swallows its own errors.
    """
    if state.garmin_method != "account":
        return

    def worker() -> None:
        try:
            from .health_sync import sync_health

            result = sync_health(days=30)
            log.info(
                "Health sync: %s days written, %s days already held, %s requests, %s errors",
                result.get("days_written"),
                result.get("days_skipped"),
                result.get("requests"),
                result.get("errors"),
            )
        except Exception as exc:
            log.warning("Health sync failed: %s", exc)

    threading.Thread(target=worker, daemon=True).start()


def recent_runs(db: Session, limit: int = 20) -> list[dict[str, Any]]:
    rows = db.query(SyncRun).order_by(SyncRun.started_at.desc()).limit(limit).all()
    return [
        {
            "id": row.id,
            "source": row.source,
            "started_at": row.started_at.isoformat() if row.started_at else None,
            "finished_at": row.finished_at.isoformat() if row.finished_at else None,
            "status": row.status,
            "days": row.days,
            "activities_found": row.activities_found,
            "activities_new": row.activities_new,
            "activities_updated": row.activities_updated,
            "message": row.message,
            "error": row.error,
        }
        for row in rows
    ]
