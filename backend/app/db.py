"""SQLAlchemy engine/session plumbing.

SQLite with WAL so the background sync worker can write while the API reads.
"""

from __future__ import annotations

import logging

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


log = logging.getLogger("stride.db")


def init_db() -> None:
    from . import models  # noqa: F401  (register mappers)
    from . import models_phase2  # noqa: F401  (phase-2 tables)

    Base.metadata.create_all(engine)
    _migrate(engine)
    _secure_database()


def _secure_database() -> None:
    """Make the database and its WAL sidecars owner-only.

    The database holds a year of sleep, heart rate, stress and GPS routes, so
    it must not be readable by another user on the machine — whatever the
    process umask happens to be.
    """
    import os
    import stat
    from pathlib import Path

    for suffix in ("", "-wal", "-shm"):
        path = Path(str(DB_PATH) + suffix)
        if not path.exists():
            continue
        try:
            os.chmod(path, 0o600)
        except OSError:  # pragma: no cover - unusual filesystems
            pass
    try:
        os.chmod(Path(DB_PATH).parent, 0o700)
    except OSError:  # pragma: no cover
        pass


def _migrate(engine) -> None:
    """Additive column migrations for databases created by an earlier build.

    ``create_all`` never alters an existing table, so a column added to a
    model later has to be applied by hand — and a column *removed* from the
    model has to be dropped by hand too. Every step is idempotent.
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
    removals = {
        # The social layer is gone — Stride has no feed, no followers and no
        # publishing. The column was left behind NOT NULL with no default.
        "activity": ["visibility"],
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

        # Columns that were removed from the model but are still in the table.
        # A leftover NOT NULL column with no default breaks every INSERT that
        # does not supply it — which is exactly what happened to `visibility`
        # when the social layer was removed: the app imported nothing new and
        # the failure only appeared at the first write.
        for table, columns in removals.items():
            if table not in existing:
                continue
            present = {col["name"] for col in inspector.get_columns(table)}
            for column in columns:
                if column not in present:
                    continue
                try:
                    connection.execute(text(f"ALTER TABLE {table} DROP COLUMN {column}"))
                except Exception as exc:  # pragma: no cover - needs a rebuild
                    log.warning(
                        "could not drop %s.%s automatically (%s); rebuild the table",
                        table,
                        column,
                        exc,
                    )
