"""ORM models.

Design notes
------------
* ``activity`` is the canonical row. Every ingest path (Garmin OAuth, Garmin
  account, FIT upload) normalises into the same shape before it
  touches the DB, so downstream analytics never branches on source.
* Per-second streams live in a side table as JSON arrays rather than one row per
  sample — a 2-hour run is ~7k samples, and JSON keeps the read path to a single
  query per activity.
* Unique constraint on ``(source, external_id)`` makes re-syncing idempotent.
"""

from __future__ import annotations

from datetime import datetime, timezone

from sqlalchemy import (
    Boolean,
    DateTime,
    Float,
    ForeignKey,
    Index,
    Integer,
    JSON,
    String,
    Text,
    UniqueConstraint,
)
from sqlalchemy.orm import Mapped, mapped_column, relationship

from .db import Base

RUN_SPORTS = {
    "running",
    "trail_running",
    "treadmill_running",
    "virtualrun",
    "street_running",
    "track_running",
    "walking",
    "hiking",
}


def utcnow() -> datetime:
    return datetime.now(timezone.utc)


class Athlete(Base):
    __tablename__ = "athlete"

    id: Mapped[int] = mapped_column(primary_key=True)
    name: Mapped[str] = mapped_column(String(120), default="Athlete")
    email: Mapped[str | None] = mapped_column(String(255), nullable=True)
    timezone: Mapped[str] = mapped_column(String(64), default="Asia/Kolkata")
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)


class Activity(Base):
    __tablename__ = "activity"
    __table_args__ = (
        UniqueConstraint("source", "external_id", name="uq_activity_source_external"),
        Index("ix_activity_start", "start_time"),
        Index("ix_activity_sport", "sport_type"),
    )

    id: Mapped[int] = mapped_column(primary_key=True)
    athlete_id: Mapped[int | None] = mapped_column(ForeignKey("athlete.id"), nullable=True)

    source: Mapped[str] = mapped_column(String(32), default="garmin")
    external_id: Mapped[str] = mapped_column(String(64))

    name: Mapped[str] = mapped_column(String(255), default="Run")
    sport_type: Mapped[str] = mapped_column(String(40), default="running")
    description: Mapped[str | None] = mapped_column(Text, nullable=True)

    start_time: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)
    local_date: Mapped[str] = mapped_column(String(10), index=True)  # YYYY-MM-DD
    day_of_week: Mapped[str | None] = mapped_column(String(16), nullable=True)

    distance_m: Mapped[float] = mapped_column(Float, default=0.0)
    moving_time_s: Mapped[float] = mapped_column(Float, default=0.0)
    elapsed_time_s: Mapped[float] = mapped_column(Float, default=0.0)
    elevation_gain_m: Mapped[float] = mapped_column(Float, default=0.0)

    avg_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    calories: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_cadence: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_cadence: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_speed_ms: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_speed_ms: Mapped[float | None] = mapped_column(Float, nullable=True)

    # A recorded run contains walking warm-ups, walking cool-downs and stands.
    # These hold how much of the activity was actually running, so the analysis
    # and the UI can exclude the rest.
    running_time_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    walking_time_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    stopped_time_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    running_distance_m: Mapped[float | None] = mapped_column(Float, nullable=True)

    perceived_exertion: Mapped[int | None] = mapped_column(Integer, nullable=True)
    device: Mapped[str | None] = mapped_column(String(120), nullable=True)
    gear: Mapped[str | None] = mapped_column(String(120), nullable=True)

    is_trail: Mapped[bool] = mapped_column(Boolean, default=False)
    is_commute: Mapped[bool] = mapped_column(Boolean, default=False)

    # Encoded Google polyline + decoded points. Stored as JSON list of [lat, lon].
    polyline: Mapped[list | None] = mapped_column(JSON, nullable=True)
    geo_points: Mapped[list | None] = mapped_column(JSON, nullable=True)

    # Raw provider payload, for debugging / re-normalisation.
    raw: Mapped[dict | None] = mapped_column(JSON, nullable=True)

    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=utcnow, onupdate=utcnow
    )

    laps: Mapped[list["ActivityLap"]] = relationship(
        back_populates="activity", cascade="all, delete-orphan", order_by="ActivityLap.lap_index"
    )
    streams: Mapped[list["ActivityStream"]] = relationship(
        back_populates="activity", cascade="all, delete-orphan"
    )
    zones: Mapped[list["HrZone"]] = relationship(
        back_populates="activity", cascade="all, delete-orphan", order_by="HrZone.zone_no"
    )


class ActivityLap(Base):
    __tablename__ = "activity_lap"
    __table_args__ = (UniqueConstraint("activity_id", "lap_index", name="uq_lap"),)

    id: Mapped[int] = mapped_column(primary_key=True)
    activity_id: Mapped[int] = mapped_column(ForeignKey("activity.id", ondelete="CASCADE"), index=True)

    lap_index: Mapped[int] = mapped_column(Integer, default=1)
    distance_m: Mapped[float] = mapped_column(Float, default=0.0)
    duration_s: Mapped[float] = mapped_column(Float, default=0.0)
    avg_speed_ms: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_speed_ms: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_cadence: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_cadence: Mapped[float | None] = mapped_column(Float, nullable=True)
    stride_length_m: Mapped[float | None] = mapped_column(Float, nullable=True)
    elevation_gain_m: Mapped[float | None] = mapped_column(Float, nullable=True)
    elevation_loss_m: Mapped[float | None] = mapped_column(Float, nullable=True)
    start_elevation_m: Mapped[float | None] = mapped_column(Float, nullable=True)
    end_elevation_m: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Running-only figures: the distance and time covered while actually
    # running, and the elapsed span of the lap.
    running_distance_m: Mapped[float | None] = mapped_column(Float, nullable=True)
    elapsed_s: Mapped[float | None] = mapped_column(Float, nullable=True)

    activity: Mapped[Activity] = relationship(back_populates="laps")


class ActivityStream(Base):
    """Per-second / per-sample series for one activity."""

    __tablename__ = "activity_stream"
    __table_args__ = (UniqueConstraint("activity_id", "kind", name="uq_stream"),)

    id: Mapped[int] = mapped_column(primary_key=True)
    activity_id: Mapped[int] = mapped_column(ForeignKey("activity.id", ondelete="CASCADE"), index=True)

    kind: Mapped[str] = mapped_column(String(24))  # hr | pace | elevation | distance | cadence
    seconds: Mapped[list] = mapped_column(JSON, default=list)
    values: Mapped[list] = mapped_column(JSON, default=list)
    label: Mapped[str | None] = mapped_column(String(64), nullable=True)
    unit: Mapped[str | None] = mapped_column(String(16), nullable=True)

    activity: Mapped[Activity] = relationship(back_populates="streams")


class HrZone(Base):
    __tablename__ = "hr_zone"
    __table_args__ = (UniqueConstraint("activity_id", "zone_no", name="uq_hrzone"),)

    id: Mapped[int] = mapped_column(primary_key=True)
    activity_id: Mapped[int] = mapped_column(ForeignKey("activity.id", ondelete="CASCADE"), index=True)

    zone_no: Mapped[int] = mapped_column(Integer, default=1)
    name: Mapped[str] = mapped_column(String(64), default="Zone 1")
    low_bpm: Mapped[float | None] = mapped_column(Float, nullable=True)
    high_bpm: Mapped[float | None] = mapped_column(Float, nullable=True)
    time_s: Mapped[float] = mapped_column(Float, default=0.0)

    activity: Mapped[Activity] = relationship(back_populates="zones")


class DailyMetric(Base):
    __tablename__ = "daily_metric"

    date: Mapped[str] = mapped_column(String(10), primary_key=True)  # YYYY-MM-DD
    resting_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    sleep_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    deep_sleep_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    hrv_ms: Mapped[float | None] = mapped_column(Float, nullable=True)
    stress: Mapped[float | None] = mapped_column(Float, nullable=True)
    steps: Mapped[float | None] = mapped_column(Float, nullable=True)
    body_battery: Mapped[float | None] = mapped_column(Float, nullable=True)
    readiness: Mapped[float | None] = mapped_column(Float, nullable=True)
    distance_m: Mapped[float] = mapped_column(Float, default=0.0)


class SyncRun(Base):
    __tablename__ = "sync_run"

    id: Mapped[int] = mapped_column(primary_key=True)
    source: Mapped[str] = mapped_column(String(32), default="garmin")
    started_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)
    finished_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    status: Mapped[str] = mapped_column(String(24), default="running")  # running|ok|error
    days: Mapped[int] = mapped_column(Integer, default=90)
    activities_found: Mapped[int] = mapped_column(Integer, default=0)
    activities_new: Mapped[int] = mapped_column(Integer, default=0)
    activities_updated: Mapped[int] = mapped_column(Integer, default=0)
    message: Mapped[str | None] = mapped_column(Text, nullable=True)
    error: Mapped[str | None] = mapped_column(Text, nullable=True)
