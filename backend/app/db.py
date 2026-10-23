"""SQLAlchemy engine/session plumbing.

SQLite with WAL so the background sync worker can write while the API reads.
"""

from __future__ import annotations

from collections.abc import Iterator

from sqlalchemy import create_engine, event
from sqlalchemy.orm import DeclarativeBase, Session, sessionmaker

from .config import DB_PATH

engine = create_engine(
    f"sqlite:///{DB_PATH}",
    connect_args={"check_same_thread": False, "timeout": 30},
    future=True,
)


@event.listens_for(engine, "connect")
def _sqlite_pragmas(dbapi_connection, _record):  # pragma: no cover - driver hook
    cur = dbapi_connection.cursor()
    cur.execute("PRAGMA journal_mode=WAL")
    cur.execute("PRAGMA synchronous=NORMAL")
    cur.execute("PRAGMA foreign_keys=ON")
    cur.execute("PRAGMA busy_timeout=30000")
    cur.close()


SessionLocal = sessionmaker(bind=engine, autoflush=False, expire_on_commit=False, future=True)


class Base(DeclarativeBase):
    pass


def get_db() -> Iterator[Session]:
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()


def init_db() -> None:
    from . import models  # noqa: F401  (register mappers)
    from . import models_phase2  # noqa: F401  (phase-2 tables)

    Base.metadata.create_all(engine)
    _migrate(engine)


def _migrate(engine) -> None:
    """Additive column migrations for databases created by an earlier build.

    ``create_all`` never alters an existing table, so a column added to a
    model later has to be applied by hand. Every step is idempotent.
    """
    from sqlalchemy import inspect, text

    inspector = inspect(engine)
    existing = set(inspector.get_table_names())
    additions = {
        "daily_physiology": [("max_hr", "REAL")],
        "activity": [
            ("running_time_s", "REAL"),
            ("walking_time_s", "REAL"),
            ("stopped_time_s", "REAL"),
            ("running_distance_m", "REAL"),
        ],
        "activity_lap": [
            ("running_distance_m", "REAL"),
            ("elapsed_s", "REAL"),
        ],
        "run_fingerprint": [
            ("running_time_s", "REAL"),
            ("walking_time_s", "REAL"),
            ("stopped_time_s", "REAL"),
            ("hr_drift_bpm", "REAL"),
            ("pace_drift_pct", "REAL"),
            ("first_half_hr", "REAL"),
            ("second_half_hr", "REAL"),
            ("first_half_pace_s_km", "REAL"),
            ("second_half_pace_s_km", "REAL"),
        ],
    }
    with engine.begin() as connection:
        for table, columns in additions.items():
            if table not in existing:
                continue
            present = {col["name"] for col in inspector.get_columns(table)}
            for column, kind in columns:
                if column in present:
                    continue
                connection.execute(text(f"ALTER TABLE {table} ADD COLUMN {column} {kind}"))
