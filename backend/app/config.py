"""Runtime configuration for Stride.

Two layers:

* **Static config** — environment variables, read once at import.
* **Mutable state** — Garmin credentials, OAuth tokens, network route. Persisted to
  ``data/state.json`` so it survives restarts without putting secrets in the DB
  or the environment.

The mutable layer exists because the whole app is a personal, single-user tool:
the user connects Garmin once from the Settings screen and we never ask again.
"""

from __future__ import annotations

import json
import logging
import os
import secrets
import threading
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

BASE_DIR = Path(__file__).resolve().parent.parent.parent
DATA_DIR = Path(os.getenv("STRIDE_DATA_DIR", BASE_DIR / "backend" / "data"))
DATA_DIR.mkdir(parents=True, exist_ok=True)

DB_PATH = DATA_DIR / "stride.db"
STATE_PATH = DATA_DIR / "state.json"

# Bumped when the shape of the state file changes incompatibly.
STATE_VERSION = 1

# Owner-only. The state file holds a live Garmin API key.
SECURE_FILE_MODE = 0o600
SECURE_DIR_MODE = 0o700


def _secure_file(path: Path) -> None:
    """Make a file owner-only, ignoring a filesystem that disagrees."""
    try:
        os.chmod(path, SECURE_FILE_MODE)
    except OSError as exc:  # pragma: no cover - unusual filesystems
        log.warning("Could not set permissions on %s: %s", path, exc)


def _secure_dir(path: Path) -> None:
    try:
        os.chmod(path, SECURE_DIR_MODE)
    except OSError as exc:  # pragma: no cover
        log.warning("Could not set permissions on %s: %s", path, exc)


log = logging.getLogger(__name__)

FRONTEND_DIST = BASE_DIR / "frontend" / "dist"

# Fallback source of the demo dataset that ships with the app.
DEMO_CACHE = Path(os.getenv("STRIDE_DEMO_CACHE", Path.home() / "running" / "sample-data" / "unified-cache.json"))


@dataclass
class AppState:
    """Mutable, persisted application state."""

    # Garmin connection
    garmin_connected: bool = False
    garmin_method: str | None = None  # "developer_key" | "account"
    garmin_athlete: dict[str, Any] = field(default_factory=dict)

    # Developer-key OAuth1 (3-legged) state
    consumer_key: str | None = None
    consumer_secret: str | None = None
    oauth_token: str | None = None
    oauth_token_secret: str | None = None
    oauth_verifier: str | None = None
    pending_oauth_token: str | None = None
    pending_oauth_token_secret: str | None = None

    # Account (email/password) state
    garmin_email: str | None = None
    garmin_password: str | None = None
    garmin_tokenstore: str | None = None

    # Garmin Connect Personal API key — the recommended path. A single
    # long-lived key from connect.garmin.com/app/settings/personalApiToken;
    # it needs no developer registration and covers activities *and* wellness.
    personal_api_key: str | None = None

    # Networking — Garmin blocks datacenter IPs, so we may need the home exit node.
    network_route: str = "auto"  # "auto" | "direct" | "exit_node"
    exit_node: str | None = os.getenv("STRIDE_EXIT_NODE")

    # Sync behaviour
    sync_days: int = 90
    auto_sync: bool = True
    sync_interval_minutes: int = 180
    last_sync_at: str | None = None

    # History windows. ``None`` for activities means "everything the Garmin
    # account holds"; the Activity API has no short retention window.
    history_months: int | None = 12
    health_history_days: int | None = 90

    # Demo activities are synthetic. They are seeded on first boot so the app
    # is never empty, and switched off automatically the moment real Garmin
    # data arrives — mixing synthetic runs with real ones would be misleading.
    demo_visible: bool = True

    # App-level session secret (so the web UI can hold a session cookie).
    session_secret: str = field(default_factory=lambda: secrets.token_hex(32))

    # Device pairing. A browser on the tailnet is not a loopback client, so
    # it cannot mint a capability cookie by loading the app. The operator
    # generates a short code on the machine and types it into the device; the
    # code is exchanged once for the same HttpOnly cookie a loopback browser
    # gets. Single use, short lived, and never a long-lived secret in
    # JavaScript.
    pairing_code: str | None = None
    pairing_expires_at: str | None = None

    _lock: threading.Lock = field(default_factory=threading.Lock, repr=False)

    # ------------------------------------------------------------------ io

    @classmethod
    def load(cls) -> "AppState":
        STATE_PATH.parent.mkdir(parents=True, exist_ok=True)
        _secure_dir(STATE_PATH.parent)
        if not STATE_PATH.exists():
            state = cls()
            state.save()
            return state
        try:
            raw = json.loads(STATE_PATH.read_text())
        except (json.JSONDecodeError, OSError):
            raw = {}
        version = raw.get("schema_version")
        if version is not None and int(version) > STATE_VERSION:
            raise RuntimeError(
                f"state.json was written by a newer Stride (schema v{version}); "
                f"this build understands v{STATE_VERSION}. Refusing to guess."
            )
        known = {f for f in cls.__dataclass_fields__ if not f.startswith("_")}
        # Repair permissions on an existing file, which may predate this
        # hardening and have been world-readable.
        _secure_file(STATE_PATH)
        return cls(**{k: v for k, v in raw.items() if k in known and k != "schema_version"})

    def save(self) -> None:
        """Write the state file with owner-only permissions.

        It holds the Garmin Personal API key, the account password and the OAuth
        secrets, so it must never be readable by another user on the machine —
        regardless of the process umask.
        """
        with self._lock:
            data = {
                k: v
                for k, v in self.__dict__.items()
                if not k.startswith("_") and k != "session_secret" and v is not None
            }
            data["session_secret"] = self.session_secret
            data["schema_version"] = STATE_VERSION
            tmp = STATE_PATH.with_suffix(".tmp")
            # Create with the final mode, so the credential is never briefly
            # world-readable between creation and chmod.
            fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
            try:
                with os.fdopen(fd, "w") as handle:
                    handle.write(json.dumps(data, indent=2))
            except Exception:
                tmp.unlink(missing_ok=True)
                raise
            os.replace(tmp, STATE_PATH)
            _secure_file(STATE_PATH)
            _secure_dir(STATE_PATH.parent)

    def update(self, **kwargs: Any) -> None:
        with self._lock:
            for key, value in kwargs.items():
                setattr(self, key, value)
        self.save()

    # -------------------------------------------------------------- helpers

    @property
    def has_developer_key(self) -> bool:
        return bool(self.consumer_key and self.consumer_secret)

    def public_status(self) -> dict[str, Any]:
        """Status payload safe to hand to the browser (no secrets)."""
        return {
            "garmin_connected": self.garmin_connected,
            "garmin_method": self.garmin_method,
            "athlete": self.garmin_athlete,
            "has_developer_key": self.has_developer_key,
            "has_account_credentials": bool(self.garmin_email and self.garmin_password),
            "has_personal_api_key": bool(self.personal_api_key),
            "pending_authorization": bool(self.pending_oauth_token),
            "network_route": self.network_route,
            "exit_node": self.exit_node,
            "sync_days": self.sync_days,
            "history_months": self.history_months,
            "health_history_days": self.health_history_days,
            "auto_sync": self.auto_sync,
            "sync_interval_minutes": self.sync_interval_minutes,
            "last_sync_at": self.last_sync_at,
            "demo_visible": self.demo_visible,
        }


state = AppState.load()
