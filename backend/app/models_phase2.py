"""Phase-2 schema: plans, plan-vs-actual results, physiology, fingerprints.

Kept in its own module so the Phase-1 tables in :mod:`app.models` stay
untouched. Both modules share the same declarative ``Base``, so
``init_db`` creates every table in one pass.

Design notes
------------
* ``plan_*`` tables describe intent; ``plan_result`` describes what happened
  and how it compared. They are separate so that a plan can be edited or
  re-scored without losing the history of what was actually run.
* ``daily_physiology`` is keyed by date (not by activity) because readiness is
  a property of the day, not of a run.
* ``run_fingerprint`` holds derived mechanics for one activity. It is written
  by the fingerprint service and never by the ingest layer.
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

# ---------------------------------------------------------------- plan model

SESSION_TYPES = (
    "easy",
    "long",
    "tempo",
    "interval",
    "race",
    "recovery",
    "strength",
    "rest",
)

VERDICTS = (
    "hit",
    "missed_pace",
    "short",
    "missed",
    "substituted",
    "bonus",
)


def utcnow() -> datetime:
    return datetime.now(timezone.utc)


class PlanBlock(Base):
    """A training cycle — e.g. "HM Sub-2:00 Campaign"."""

    __tablename__ = "plan_block"

    id: Mapped[int] = mapped_column(primary_key=True)
    name: Mapped[str] = mapped_column(String(160), default="Training block")
    start_date: Mapped[str] = mapped_column(String(10), index=True)
    end_date: Mapped[str | None] = mapped_column(String(10), nullable=True)
    goal: Mapped[str | None] = mapped_column(String(255), nullable=True)
    status: Mapped[str] = mapped_column(String(24), default="active")  # active|completed|archived
    source: Mapped[str] = mapped_column(String(24), default="manual")  # manual|imported|garmin
    notes: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)

    phases: Mapped[list["PlanPhase"]] = relationship(
        back_populates="block", cascade="all, delete-orphan", order_by="PlanPhase.start_date"
    )


class PlanPhase(Base):
    """A phase within a block — Base, Build, Specific, Taper…"""

    __tablename__ = "plan_phase"

    id: Mapped[int] = mapped_column(primary_key=True)
    block_id: Mapped[int] = mapped_column(
        ForeignKey("plan_block.id", ondelete="CASCADE"), index=True
    )

    name: Mapped[str] = mapped_column(String(80), default="Phase")
    start_date: Mapped[str | None] = mapped_column(String(10), nullable=True)
    end_date: Mapped[str | None] = mapped_column(String(10), nullable=True)

    target_weekly_km_min: Mapped[float | None] = mapped_column(Float, nullable=True)
    target_weekly_km_max: Mapped[float | None] = mapped_column(Float, nullable=True)
    target_sessions: Mapped[int | None] = mapped_column(Integer, nullable=True)

    # Pace bands for this phase, seconds per km.
    pace_easy_low: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_easy_high: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_tempo_low: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_tempo_high: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_interval_low: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_interval_high: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_long_low: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_long_high: Mapped[float | None] = mapped_column(Float, nullable=True)

    hr_zone_target: Mapped[int | None] = mapped_column(Integer, nullable=True)
    notes: Mapped[str | None] = mapped_column(Text, nullable=True)

    block: Mapped[PlanBlock] = relationship(back_populates="phases")
    weeks: Mapped[list["PlanWeek"]] = relationship(
        back_populates="phase", cascade="all, delete-orphan", order_by="PlanWeek.week_number"
    )


class PlanWeek(Base):
    """One week of a phase."""

    __tablename__ = "plan_week"

    id: Mapped[int] = mapped_column(primary_key=True)
    phase_id: Mapped[int] = mapped_column(
        ForeignKey("plan_phase.id", ondelete="CASCADE"), index=True
    )

    week_number: Mapped[int] = mapped_column(Integer, default=1)
    start_date: Mapped[str | None] = mapped_column(String(10), nullable=True)
    end_date: Mapped[str | None] = mapped_column(String(10), nullable=True)
    target_km: Mapped[float | None] = mapped_column(Float, nullable=True)
    key_workout: Mapped[str | None] = mapped_column(String(255), nullable=True)
    notes: Mapped[str | None] = mapped_column(Text, nullable=True)

    phase: Mapped[PlanPhase] = relationship(back_populates="weeks")
    sessions: Mapped[list["PlanSession"]] = relationship(
        back_populates="week", cascade="all, delete-orphan", order_by="PlanSession.date"
    )


class PlanSession(Base):
    """A single planned session on a specific day."""

    __tablename__ = "plan_session"
    __table_args__ = (
        Index("ix_plan_session_date", "date"),
        Index("ix_plan_session_block_date", "block_id", "date"),
    )

    id: Mapped[int] = mapped_column(primary_key=True)
    week_id: Mapped[int | None] = mapped_column(
        ForeignKey("plan_week.id", ondelete="CASCADE"), nullable=True, index=True
    )
    block_id: Mapped[int | None] = mapped_column(
        ForeignKey("plan_block.id", ondelete="CASCADE"), nullable=True
    )

    date: Mapped[str] = mapped_column(String(10))
    session_type: Mapped[str] = mapped_column(String(24), default="easy")
    title: Mapped[str | None] = mapped_column(String(160), nullable=True)
    details: Mapped[str | None] = mapped_column(Text, nullable=True)

    target_distance_m: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_low_s_km: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_high_s_km: Mapped[float | None] = mapped_column(Float, nullable=True)
    hr_zone_target: Mapped[int | None] = mapped_column(Integer, nullable=True)

    status: Mapped[str] = mapped_column(String(24), default="planned")
    source: Mapped[str] = mapped_column(String(24), default="manual")
    notes: Mapped[str | None] = mapped_column(Text, nullable=True)

    week: Mapped[PlanWeek | None] = relationship(back_populates="sessions")
    results: Mapped[list["PlanResult"]] = relationship(
        back_populates="plan_session", cascade="all, delete-orphan"
    )


class PlanResult(Base):
    """The comparison of one planned session against what was actually done."""

    __tablename__ = "plan_result"

    id: Mapped[int] = mapped_column(primary_key=True)
    plan_session_id: Mapped[int] = mapped_column(
        ForeignKey("plan_session.id", ondelete="CASCADE"), index=True
    )
    activity_id: Mapped[int | None] = mapped_column(
        ForeignKey("activity.id", ondelete="SET NULL"), nullable=True, index=True
    )

    verdict: Mapped[str] = mapped_column(String(24), default="missed")
    distance_delta_m: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_delta_s_km: Mapped[float | None] = mapped_column(Float, nullable=True)
    pace_band_hit: Mapped[bool | None] = mapped_column(Boolean, nullable=True)
    distance_hit: Mapped[bool | None] = mapped_column(Boolean, nullable=True)
    adherence_score: Mapped[float | None] = mapped_column(Float, nullable=True)

    cause: Mapped[str | None] = mapped_column(Text, nullable=True)
    cause_confidence: Mapped[float | None] = mapped_column(Float, nullable=True)
    note: Mapped[str | None] = mapped_column(Text, nullable=True)
    computed_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)

    plan_session: Mapped[PlanSession] = relationship(back_populates="results")


# -------------------------------------------------------------- physiology


class DailyPhysiology(Base):
    """The physiological state of one calendar day."""

    __tablename__ = "daily_physiology"

    date: Mapped[str] = mapped_column(String(10), primary_key=True)

    sleep_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    sleep_score: Mapped[float | None] = mapped_column(Float, nullable=True)
    deep_sleep_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    rem_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    light_sleep_s: Mapped[float | None] = mapped_column(Float, nullable=True)
    awake_s: Mapped[float | None] = mapped_column(Float, nullable=True)

    hrv_ms: Mapped[float | None] = mapped_column(Float, nullable=True)
    hrv_status: Mapped[str | None] = mapped_column(String(24), nullable=True)
    resting_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_hr: Mapped[float | None] = mapped_column(Float, nullable=True)
    stress_avg: Mapped[float | None] = mapped_column(Float, nullable=True)
    stress_max: Mapped[float | None] = mapped_column(Float, nullable=True)

    body_battery_min: Mapped[float | None] = mapped_column(Float, nullable=True)
    body_battery_avg: Mapped[float | None] = mapped_column(Float, nullable=True)
    body_battery_max: Mapped[float | None] = mapped_column(Float, nullable=True)

    steps: Mapped[float | None] = mapped_column(Float, nullable=True)
    training_readiness: Mapped[float | None] = mapped_column(Float, nullable=True)
    acute_load: Mapped[float | None] = mapped_column(Float, nullable=True)
    chronic_load: Mapped[float | None] = mapped_column(Float, nullable=True)
    vo2max: Mapped[float | None] = mapped_column(Float, nullable=True)
    endurance_score: Mapped[float | None] = mapped_column(Float, nullable=True)
    running_tolerance: Mapped[float | None] = mapped_column(Float, nullable=True)

    source: Mapped[str] = mapped_column(String(24), default="garmin")
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=utcnow, onupdate=utcnow
    )


class ActivityFact(Base):
    """Extra per-activity detail that only Garmin knows about."""

    __tablename__ = "activity_fact"

    activity_id: Mapped[int] = mapped_column(
        ForeignKey("activity.id", ondelete="CASCADE"), primary_key=True
    )

    aerobic_te: Mapped[float | None] = mapped_column(Float, nullable=True)
    anaerobic_te: Mapped[float | None] = mapped_column(Float, nullable=True)
    training_effect_label: Mapped[str | None] = mapped_column(String(64), nullable=True)
    vo2max: Mapped[float | None] = mapped_column(Float, nullable=True)
    recovery_time_min: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_gct_ms: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_stride_ratio: Mapped[float | None] = mapped_column(Float, nullable=True)
    vertical_ratio: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_power: Mapped[float | None] = mapped_column(Float, nullable=True)
    normalized_power: Mapped[float | None] = mapped_column(Float, nullable=True)
    avg_grade_adjusted_pace: Mapped[float | None] = mapped_column(Float, nullable=True)
    temperature_c: Mapped[float | None] = mapped_column(Float, nullable=True)
    impact_estimate: Mapped[float | None] = mapped_column(Float, nullable=True)
    gear_uuid: Mapped[str | None] = mapped_column(String(64), nullable=True)
    device_name: Mapped[str | None] = mapped_column(String(120), nullable=True)


class RunFingerprint(Base):
    """Derived mechanics for one activity — the athlete's running signature."""

    __tablename__ = "run_fingerprint"

    activity_id: Mapped[int] = mapped_column(
        ForeignKey("activity.id", ondelete="CASCADE"), primary_key=True
    )

    cadence_avg: Mapped[float | None] = mapped_column(Float, nullable=True)
    cadence_cv: Mapped[float | None] = mapped_column(Float, nullable=True)
    cadence_drift_pct: Mapped[float | None] = mapped_column(Float, nullable=True)

    hr_avg: Mapped[float | None] = mapped_column(Float, nullable=True)
    hr_max: Mapped[float | None] = mapped_column(Float, nullable=True)
    hr_drift_pct: Mapped[float | None] = mapped_column(Float, nullable=True)

    efficiency_first: Mapped[float | None] = mapped_column(Float, nullable=True)
    efficiency_last: Mapped[float | None] = mapped_column(Float, nullable=True)
    decoupling_pct: Mapped[float | None] = mapped_column(Float, nullable=True)

    pace_cv: Mapped[float | None] = mapped_column(Float, nullable=True)
    negative_split_pct: Mapped[float | None] = mapped_column(Float, nullable=True)
    stride_length_cm: Mapped[float | None] = mapped_column(Float, nullable=True)
    form_score: Mapped[float | None] = mapped_column(Float, nullable=True)
    zone_distribution: Mapped[dict | None] = mapped_column(JSON, nullable=True)
    notes: Mapped[str | None] = mapped_column(Text, nullable=True)

    computed_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)


class Readiness(Base):
    """Our composite readiness score and its drivers."""

    __tablename__ = "readiness"

    date: Mapped[str] = mapped_column(String(10), primary_key=True)
    score: Mapped[float] = mapped_column(Float, default=0.0)
    grade: Mapped[str] = mapped_column(String(16), default="unknown")  # peak|good|fair|low
    drivers: Mapped[dict | None] = mapped_column(JSON, nullable=True)
    recommendation: Mapped[str | None] = mapped_column(Text, nullable=True)
    rationale: Mapped[str | None] = mapped_column(Text, nullable=True)
    computed_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=utcnow)


class Gear(Base):
    """Shoes. Rotation mileage and retirement — runners care, Strava barely does."""

    __tablename__ = "gear"

    id: Mapped[int] = mapped_column(primary_key=True)
    uuid: Mapped[str] = mapped_column(String(64), unique=True, index=True)
    name: Mapped[str] = mapped_column(String(160), default="Shoes")
    brand: Mapped[str | None] = mapped_column(String(80), nullable=True)
    model: Mapped[str | None] = mapped_column(String(80), nullable=True)
    status: Mapped[str] = mapped_column(String(24), default="active")  # active|retired
    distance_m: Mapped[float] = mapped_column(Float, default=0.0)
    activity_count: Mapped[int] = mapped_column(Integer, default=0)
    first_used: Mapped[str | None] = mapped_column(String(10), nullable=True)
    last_used: Mapped[str | None] = mapped_column(String(10), nullable=True)
    retire_at_m: Mapped[float] = mapped_column(Float, default=800_000.0)
