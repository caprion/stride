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

    # Networking — Garmin blocks datacenter IPs, so we may need the home exit node.
    network_route: str = "auto"  # "auto" | "direct" | "exit_node"
    exit_node: str | None = os.getenv("STRIDE_EXIT_NODE")

    # Sync behaviour
    sync_days: int = 90
    auto_sync: bool = True
    sync_interval_minutes: int = 180
    last_sync_at: str | None = None

    # Demo activities are synthetic. They are seeded on first boot so the app
    # is never empty, and switched off automatically the moment real Garmin
    # data arrives — mixing synthetic runs with real ones would be misleading.
    demo_visible: bool = True

    # App-level session secret (so the web UI can hold a session cookie).
    session_secret: str = field(default_factory=lambda: secrets.token_hex(32))

    _lock: threading.Lock = field(default_factory=threading.Lock, repr=False)

    # ------------------------------------------------------------------ io

    @classmethod
    def load(cls) -> "AppState":
        if not STATE_PATH.exists():
            state = cls()
            state.save()
            return state
        try:
            raw = json.loads(STATE_PATH.read_text())
        except (json.JSONDecodeError, OSError):
            raw = {}
        known = {f for f in cls.__dataclass_fields__ if not f.startswith("_")}
        return cls(**{k: v for k, v in raw.items() if k in known})

    def save(self) -> None:
        with self._lock:
            data = {
                k: v
                for k, v in self.__dict__.items()
                if not k.startswith("_") and k != "session_secret"
            }
            data["session_secret"] = self.session_secret
            tmp = STATE_PATH.with_suffix(".tmp")
            tmp.write_text(json.dumps(data, indent=2))
            tmp.replace(STATE_PATH)

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
            "pending_authorization": bool(self.pending_oauth_token),
            "network_route": self.network_route,
            "exit_node": self.exit_node,
            "sync_days": self.sync_days,
            "auto_sync": self.auto_sync,
            "sync_interval_minutes": self.sync_interval_minutes,
            "last_sync_at": self.last_sync_at,
            "demo_visible": self.demo_visible,
        }


state = AppState.load()
