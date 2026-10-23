"""Run comparison endpoints."""

from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, Query
from sqlalchemy.orm import Session

from ..db import get_db
from ..services import comparison

router = APIRouter(prefix="/compare", tags=["compare"])


@router.get("/candidates")
def list_candidates(
    db: Session = Depends(get_db),
    limit: int = Query(300, ge=1, le=1000),
) -> dict[str, Any]:
    """Runs that can be picked for a comparison."""
    return {"items": comparison.candidates(db, limit=limit)}


@router.get("")
def compare_runs(
    ids: str = Query(..., description="Comma-separated activity ids"),
    db: Session = Depends(get_db),
) -> dict[str, Any]:
    """Compare two or more runs on the same parameters.

    Every run is resampled onto a shared distance axis, so the heart-rate,
    cadence, pace and elevation curves can be read against one another even
    when the runs are different lengths.
    """
    wanted: list[int] = []
    for chunk in ids.split(","):
        chunk = chunk.strip()
        if not chunk:
            continue
        try:
            wanted.append(int(chunk))
        except ValueError:
            continue
    if not wanted:
        return {"runs": [], "parameters": comparison.PARAMETERS, "missing": []}
    if len(wanted) > 8:
        wanted = wanted[:8]
    return comparison.compare(db, wanted)
