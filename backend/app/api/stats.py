"""Statistics endpoints — the numbers behind the Strava stats pages."""

from __future__ import annotations

from datetime import date
from typing import Any

from fastapi import APIRouter, Depends, Query
from sqlalchemy.orm import Session

from ..db import get_db
from ..services import analytics

router = APIRouter(prefix="/stats", tags=["stats"])


@router.get("/summary")
def summary(db: Session = Depends(get_db)) -> dict[str, Any]:
    payload = analytics.summary(db)
    payload["recent"] = analytics.recent_activities(db, limit=8)
    return payload


@router.get("/weekly")
def weekly(db: Session = Depends(get_db), weeks: int = Query(26, ge=1, le=200)) -> dict[str, Any]:
    return {"items": analytics.weekly_volume(db, weeks=weeks)}


@router.get("/monthly")
def monthly(db: Session = Depends(get_db), months: int = Query(12, ge=1, le=60)) -> dict[str, Any]:
    return {"items": analytics.monthly_volume(db, months=months)}


@router.get("/pace-trends")
def pace_trends(db: Session = Depends(get_db), weeks: int = Query(26, ge=1, le=200)) -> dict[str, Any]:
    return {"items": analytics.pace_trends(db, weeks=weeks)}


@router.get("/hr-zones")
def hr_zones(db: Session = Depends(get_db), weeks: int = Query(12, ge=1, le=104)) -> dict[str, Any]:
    return analytics.hr_zones(db, weeks=weeks)


@router.get("/personal-records")
def personal_records(db: Session = Depends(get_db)) -> dict[str, Any]:
    return {"items": analytics.personal_records(db)}


@router.get("/heatmap")
def heatmap(db: Session = Depends(get_db), year: int | None = None) -> dict[str, Any]:
    return analytics.heatmap(db, year=year)


@router.get("/elevation")
def elevation(db: Session = Depends(get_db), weeks: int = Query(26, ge=1, le=200)) -> dict[str, Any]:
    items = analytics.weekly_volume(db, weeks=weeks)
    return {"items": [{"week_key": i["week_key"], "elevation_gain_m": i["elevation_gain_m"]} for i in items]}
