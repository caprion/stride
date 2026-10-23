"""Calendar, sync, network and daily-health endpoints."""

from __future__ import annotations

from datetime import date
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.orm import Session

from ..config import state
from ..db import get_db
from ..models import Activity, DailyMetric
from ..services import analytics, sync
from .. import network

calendar_router = APIRouter(prefix="/calendar", tags=["calendar"])
sync_router = APIRouter(prefix="/sync", tags=["sync"])
network_router = APIRouter(prefix="/network", tags=["network"])
health_router = APIRouter(prefix="/health", tags=["health"])


# ---------------------------------------------------------------- calendar


@calendar_router.get("")
def calendar(
    db: Session = Depends(get_db),
    year: int = Query(default_factory=lambda: date.today().year),
    month: int = Query(default_factory=lambda: date.today().month, ge=1, le=12),
) -> dict[str, Any]:
    return analytics.calendar(db, year=year, month=month)


# ------------------------------------------------------------------- sync


class SyncBody(BaseModel):
    days: int | None = None
    background: bool = True


@sync_router.post("/run")
def run_sync(body: SyncBody | None = None) -> dict[str, Any]:
    body = body or SyncBody()
    return sync.run_sync(days=body.days, background=body.background)


@sync_router.get("/status")
def sync_status() -> dict[str, Any]:
    return {
        **sync.status(),
        "last_sync_at": state.last_sync_at,
        "connected": state.garmin_connected,
        "method": state.garmin_method,
    }


@sync_router.get("/runs")
def sync_runs(db: Session = Depends(get_db), limit: int = Query(20, ge=1, le=100)) -> dict[str, Any]:
    return {"items": sync.recent_runs(db, limit=limit)}


class SettingsBody(BaseModel):
    sync_days: int | None = None
    auto_sync: bool | None = None
    sync_interval_minutes: int | None = None
    network_route: str | None = None
    exit_node: str | None = None
    demo_visible: bool | None = None


@sync_router.patch("/settings")
def patch_settings(body: SettingsBody) -> dict[str, Any]:
    updates: dict[str, Any] = {}
    if body.sync_days is not None:
        updates["sync_days"] = max(1, min(body.sync_days, 3650))
    if body.auto_sync is not None:
        updates["auto_sync"] = body.auto_sync
    if body.sync_interval_minutes is not None:
        updates["sync_interval_minutes"] = max(15, min(body.sync_interval_minutes, 1440))
    if body.network_route is not None:
        if body.network_route not in ("auto", "direct", "exit_node"):
            raise HTTPException(status_code=400, detail="Unknown network route.")
        updates["network_route"] = body.network_route
    if body.exit_node is not None:
        updates["exit_node"] = body.exit_node
    if body.demo_visible is not None:
        updates["demo_visible"] = body.demo_visible
    if updates:
        state.update(**updates)
    return state.public_status()


# ---------------------------------------------------------------- network


@network_router.get("/status")
def network_status() -> dict[str, Any]:
    payload = network.status()
    payload["public_identity"] = network.public_identity()
    payload["route"] = state.network_route
    payload["nodes"] = network.available_exit_nodes()
    return payload


class RouteBody(BaseModel):
    route: str


@network_router.post("/route")
def set_route(body: RouteBody) -> dict[str, Any]:
    if body.route == "direct":
        ok, message = network.clear_exit_node()
        if ok:
            state.update(network_route="direct")
    elif body.route == "exit_node":
        node = state.exit_node
        if not node:
            raise HTTPException(status_code=400, detail="No exit node is configured.")
        ok, message = network.set_exit_node(node)
        if ok:
            state.update(network_route="exit_node")
    else:
        raise HTTPException(status_code=400, detail="Route must be 'direct' or 'exit_node'.")
    return {"ok": ok, "message": message, "status": network.status()}


# ----------------------------------------------------------------- health


@health_router.get("/daily")
def daily(
    db: Session = Depends(get_db),
    date_from: str | None = Query(None, alias="from"),
    date_to: str | None = Query(None, alias="to"),
) -> dict[str, Any]:
    stmt = select(DailyMetric).order_by(DailyMetric.date)
    if date_from:
        stmt = stmt.where(DailyMetric.date >= date_from)
    if date_to:
        stmt = stmt.where(DailyMetric.date <= date_to)
    rows = db.execute(stmt).scalars().all()
    return {
        "items": [
            {
                "date": row.date,
                "resting_hr": row.resting_hr,
                "max_hr": row.max_hr,
                "sleep_s": row.sleep_s,
                "deep_sleep_s": row.deep_sleep_s,
                "hrv_ms": row.hrv_ms,
                "stress": row.stress,
                "steps": row.steps,
                "body_battery": row.body_battery,
                "readiness": row.readiness,
                "distance_m": row.distance_m,
            }
            for row in rows
        ]
    }
