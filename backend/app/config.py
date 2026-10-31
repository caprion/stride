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

import ipaddress
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


# ---------------------------------------------------------------------------
# Trust configuration
# ---------------------------------------------------------------------------
#
# Whether a client from a trusted network may use the API without a capability
# code is a *deployment* decision, not application state. It is read from the
# environment once, at load time, and it is never written to the state file:
# a stale value in ``state.json`` must not be able to keep a network trusted
# after the operator turned it off.
#
# ``STRIDE_TRUST_REMOTE``
#     true — clients from the trusted networks are trusted without a capability
#     code. false (the default) — every remote client needs one.
# ``STRIDE_TRUSTED_NETWORKS``
#     comma-separated CIDRs the trusted clients come from. Empty — the
#     default — means nothing extra is trusted; only the loopback interface
#     is.

_TRUTHY = ("1", "true", "yes", "on")


def _env_flag(name: str, default: bool = False) -> bool:
    """Read a boolean flag from the environment."""
    raw = os.getenv(name)
    if raw is None:
        return default
    return raw.strip().lower() in _TRUTHY


def _parse_networks(raw: str) -> tuple[Any, ...]:
    """Parse a comma-separated list of CIDRs.

    A bare address is accepted as a single-host network. An entry that does
    not parse is dropped with a warning: a typo must not widen the trust
    boundary, and it must not stop the server from starting either.
    """
    networks: list[Any] = []
    for item in raw.split(","):
        text = item.strip()
        if not text:
            continue
        try:
            networks.append(ipaddress.ip_network(text, strict=False))
            continue
        except ValueError:
            pass
        try:
            networks.append(ipaddress.ip_network(ipaddress.ip_address(text)))
        except ValueError:
            log.warning(
                "Ignoring %r in STRIDE_TRUSTED_NETWORKS: not a CIDR or an address",
                text,
            )
    return tuple(networks)


#: Whether remote clients from the trusted networks skip the capability.
TRUST_REMOTE = _env_flag("STRIDE_TRUST_REMOTE")

#: The networks those clients come from. Empty means loopback only.
TRUSTED_NETWORKS = _parse_networks(os.getenv("STRIDE_TRUSTED_NETWORKS", ""))


def trust_remote_enabled() -> bool:
    """Are clients from the trusted networks trusted without a capability?"""
    return TRUST_REMOTE


def trusted_networks() -> tuple[Any, ...]:
    """The networks a remote client is trusted from. Empty by default."""
    return TRUSTED_NETWORKS

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

    # Whether the trusted networks are trusted at all is *not* here: it is
    # deployment configuration (``STRIDE_TRUST_REMOTE`` and
    # ``STRIDE_TRUSTED_NETWORKS``), read at load time and never persisted.
    # See the trust configuration above.

    # Networking — Garmin blocks datacenter IPs, so we may need a remote egress node.
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

    # App-level session secret (so the web UI can hold a session cookie).
    session_secret: str = field(default_factory=lambda: secrets.token_hex(32))


    # A re-entrant lock: ``update()`` and ``atomic()`` hold it across the
    # write to disk, and the write itself takes it again.
    _lock: threading.RLock = field(default_factory=threading.RLock, repr=False)

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
        # Unknown keys are dropped, and the trust configuration is not a field
        # at all: it is read from the environment at load time, so a value
        # written into the state file yesterday cannot keep a network trusted
        # after it was turned off today.
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
        # The mutation and the write happen under one lock hold, so two
        # concurrent updates cannot interleave: the file on disk is always a
        # whole state, never a half-applied one.
        with self._lock:
            for key, value in kwargs.items():
                setattr(self, key, value)
            self.save()

    def atomic(self, action: Any) -> Any:
        """Run a read-modify-write of the state under one lock hold.

        ``action(state)`` must return ``(changed, value)``. When ``changed``
        is true the state is written to disk before the lock is released.

        This is the only safe way to touch a shared collection such as the
        paired-device list. Reading the list, editing a copy and writing it
        back — the obvious way — is a lost update: a device paired or revoked
        while another request was mid-flight is silently dropped, and a
        revoked device can be written straight back into the file.
        """
        with self._lock:
            changed, value = action(self)
            if changed:
                self.save()
            return value

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
        }


state = AppState.load()
