#!/usr/bin/env python3
"""Generate ``python_schema.json`` — the structural snapshot of the Python schema.

The snapshot is the *second* oracle for ``tests/schema_matches_python.rs``.
The first oracle is ``tests/python_schema.rs``, which parses the Python
sources directly; this one is produced by SQLAlchemy itself, so a bug in the
parser cannot hide a drift on its own.

Usage (needs the backend virtualenv, which has SQLAlchemy)::

    cd backend
    PYTHONPATH=. .venv/bin/python \
        ../crates/stride-store/tests/generate_python_schema.py \
        > ../crates/stride-store/tests/python_schema.json

It only *reads* the Python implementation; it changes nothing.
"""

from __future__ import annotations

import json
import os
import sys

os.environ.setdefault("STRIDE_DATA_DIR", "/tmp/stride-schema-dump")

from sqlalchemy.dialects import sqlite  # noqa: E402

from app.db import Base  # noqa: E402
from app import models, models_phase2  # noqa: E402,F401


def sql_type(col) -> str:
    """The SQLite declared type, the way SQLite itself reports it."""
    return col.type.compile(dialect=sqlite.dialect()).upper()


def main() -> int:
    out: dict[str, object] = {"tables": {}}
    tables = out["tables"]
    assert isinstance(tables, dict)

    for table in sorted(Base.metadata.tables.values(), key=lambda t: t.name):
        columns = []
        for col in table.columns:
            columns.append(
                {
                    "name": col.name,
                    "type": sql_type(col),
                    "notnull": bool(col.nullable is False),
                    "default": None,
                    "primary_key": bool(col.primary_key),
                }
            )

        unique = []
        indexes = []
        for constraint in sorted(
            table.constraints, key=lambda c: (type(c).__name__, c.name or "")
        ):
            if constraint.__class__.__name__ != "UniqueConstraint":
                continue
            unique.append(sorted(col.name for col in constraint.columns))

        for index in sorted(table.indexes, key=lambda i: i.name or ""):
            indexes.append(
                {
                    "name": index.name,
                    "unique": bool(index.unique),
                    "columns": [col.name for col in index.columns],
                }
            )

        foreign_keys = []
        for fk in sorted(table.foreign_keys, key=lambda fk: (fk.parent.name, fk.target_fullname)):
            foreign_keys.append(
                {
                    "column": fk.parent.name,
                    "references_table": fk.target_fullname.split(".")[0],
                    "references_column": fk.column.name,
                    "on_delete": (fk.ondelete or "NO ACTION").upper(),
                }
            )

        tables[table.name] = {
            "columns": columns,
            "unique_constraints": unique,
            "indexes": indexes,
            "foreign_keys": foreign_keys,
        }

    json.dump(out, sys.stdout, indent=2, sort_keys=False)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
