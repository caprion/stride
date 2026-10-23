"""Security controls for Stride's local API.

The capability stored here is deliberately *not* an account login.  It is a
random secret held by the local athlete's browser in an HttpOnly cookie (or
presented explicitly in an Authorization header).  It prevents other local
processes and tailnet peers from reading health data or spending the athlete's
Garmin API quota merely because they can reach the port.
"""

from __future__ import annotations

import hmac
import ipaddress
import json
import logging
import os
import re
import secrets
import sys
import threading
import time
from collections import defaultdict, deque
from collections.abc import Mapping
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

from fastapi import HTTPException, Request
from fastapi.responses import JSONResponse, Response
from starlette.middleware.base import BaseHTTPMiddleware

from .config import DATA_DIR, DB_PATH, STATE_PATH, state

API_PREFIX = "/api/v1"
BOOTSTRAP_PATH = f"{API_PREFIX}/bootstrap"
AUTH_COOKIE = "stride_capability"

log = logging.getLogger("stride.security")

_TRUE = {"1", "true", "yes", "on"}
_SENSITIVE_NAME = re.compile(
    r"(?i)(api[_-]?key|authorization|bearer|consumer[_-]?secret|cookie|"
    r"oauth[_-]?token(?:[_-]?secret)?|password|secret|session|token)"
)
_QUERY_SECRET = re.compile(
    r"(?i)([?&](?:api[_-]?key|authorization|consumer[_-]?secret|cookie|"
    r"oauth[_-]?(?:token(?:[_-]?secret)?|verifier)|password|secret|session|token|code)=[^&#\s]*)"
)
_HEADER_SECRET = re.compile(
    r"(?i)((?:authorization|cookie|x-stride-token)\s*[:=]\s*)([^\s,;]+)"
)
_BEARER_SECRET = re.compile(r"(?i)\b(?:bearer|apikey)\s+[^\s,;]+")
_GARMIN_PERSONAL_KEY = re.compile(r"GCDC_[A-Za-z0-9._-]{12,}")


def _env_truthy(name: str) -> bool:
    return os.getenv(name, "").strip().lower() in _TRUE


def running_under_pytest() -> bool:
    """Keep the existing TestClient suite hermetic without weakening normal runs."""
    return bool(os.getenv("PYTEST_CURRENT_TEST")) or "pytest" in sys.modules


def auth_enabled() -> bool:
    return not _env_truthy("STRIDE_DISABLE_AUTH") and not running_under_pytest()


def configured_bind() -> str:
    """The bind address the server will actually use.

    ``STRIDE_BIND`` is the documented control, but an operator can also pass
    ``--host``/``--bind`` to uvicorn directly, which would otherwise slip past
    the check. Read the command line too, so there is one effective answer.
    """
    import sys

    argv = sys.argv
    for index, argument in enumerate(argv):
        value: str | None = None
        if argument in ("--host", "--bind") and index + 1 < len(argv):
            value = argv[index + 1]
        elif argument.startswith("--host=") :
            value = argument.split("=", 1)[1]
        elif argument.startswith("--bind="):
            value = argument.split("=", 1)[1]
        if value:
            return value.strip()
    return os.getenv("STRIDE_BIND", "127.0.0.1").strip() or "127.0.0.1"


def is_loopback_bind(bind: str | None = None) -> bool:
    value = (bind or configured_bind()).strip().strip("[]")
    if value == "localhost":
        return True
    try:
        return ipaddress.ip_address(value).is_loopback
    except ValueError:
        return False


def ensure_safe_bind_configuration() -> None:
    """Fail closed when the caller explicitly exposes the service unauthenticated."""
    if not is_loopback_bind() and not auth_enabled():
        raise RuntimeError(
            "Refusing STRIDE_BIND=%s while STRIDE_DISABLE_AUTH is enabled. "
            "A non-loopback Stride API requires capability authentication." % configured_bind()
        )


def redact_text(value: object) -> str:
    """Remove credential values before a value reaches a log or error response."""
    text = str(value)
    text = _QUERY_SECRET.sub(lambda match: match.group(1).split("=", 1)[0] + "=<redacted>", text)
    text = _HEADER_SECRET.sub(r"\1<redacted>", text)
    text = _BEARER_SECRET.sub("<redacted>", text)
    return _GARMIN_PERSONAL_KEY.sub("<redacted>", text)


def redact_payload(value: Any) -> Any:
    """Return a log-safe copy of a request/response payload.

    This helper is intentionally recursive so callers can safely log a JSON
    body without having to remember every Garmin credential field.
    """
    if isinstance(value, Mapping):
        return {
            str(key): "<redacted>" if _SENSITIVE_NAME.search(str(key)) else redact_payload(item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [redact_payload(item) for item in value]
    if isinstance(value, tuple):
        return tuple(redact_payload(item) for item in value)
    return redact_text(value) if isinstance(value, str) else value


class _RedactLogFilter(logging.Filter):
    def filter(self, record: logging.LogRecord) -> bool:
        # Render first so %-format arguments cannot bypass redaction.
        try:
            message = record.getMessage()
        except Exception:  # pragma: no cover - defensive logging path
            message = str(record.msg)
        record.msg = redact_text(message)
        record.args = ()
        if record.exc_info and record.exc_info[1] is not None:
            exception_text = str(record.exc_info[1])
            if redact_text(exception_text) != exception_text:
                # traceback formatting would otherwise emit the unredacted
                # exception string after this filter has run.
                record.exc_info = None
                record.msg = f"{record.msg} [exception details redacted]"
        return True


def install_log_redaction() -> None:
    """Install once on app and uvicorn loggers, including their access logger."""
    for name in ("", "stride", "uvicorn", "uvicorn.access", "uvicorn.error"):
        logger = logging.getLogger(name)
        if not any(isinstance(item, _RedactLogFilter) for item in logger.filters):
            logger.addFilter(_RedactLogFilter())
        for handler in logger.handlers:
            if not any(isinstance(item, _RedactLogFilter) for item in handler.filters):
                handler.addFilter(_RedactLogFilter())


def _safe_state_save(self: Any) -> None:
    """Persist state atomically with mode 0600, independent of process umask."""
    with self._lock:
        data = {key: value for key, value in self.__dict__.items() if not key.startswith("_")}
        temporary = STATE_PATH.with_name(f".{STATE_PATH.name}.{os.getpid()}.tmp")
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                json.dump(data, handle, indent=2)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, STATE_PATH)
            os.chmod(STATE_PATH, 0o600)
        finally:
            try:
                temporary.unlink()
            except FileNotFoundError:
                pass


def _read_persisted_capability() -> str | None:
    try:
        raw = json.loads(STATE_PATH.read_text(encoding="utf-8"))
    except (OSError, ValueError, TypeError):
        return None
    token = raw.get("capability_token") if isinstance(raw, dict) else None
    return token if isinstance(token, str) and len(token) >= 32 else None


def harden_storage() -> None:
    """Protect existing files and ensure future SQLite sidecars inherit mode 0600."""
    os.umask(0o077)
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    os.chmod(DATA_DIR, 0o700)
    if STATE_PATH.exists():
        os.chmod(STATE_PATH, 0o600)
    for path in (DB_PATH, Path(f"{DB_PATH}-wal"), Path(f"{DB_PATH}-shm")):
        if path.exists():
            os.chmod(path, 0o600)


def install_capability_state() -> None:
    """Add the persistent capability field without changing the state schema module."""
    harden_storage()
    # config.py predates this field and discards unknown JSON keys when loading,
    # so recover it directly before replacing save() with the hardened writer.
    capability = _read_persisted_capability() or secrets.token_urlsafe(48)
    setattr(state, "capability_token", capability)
    # AppState persists public instance attributes.  Assigning a bound method
    # to ``state.save`` would make that method itself non-serialisable JSON.
    # Patch the class, retaining normal method binding without adding state.
    type(state).save = _safe_state_save
    state.save()


def capability_token() -> str:
    return str(getattr(state, "capability_token", ""))


def _provided_token(request: Request) -> tuple[str | None, bool]:
    authorization = request.headers.get("authorization", "")
    if authorization.lower().startswith("bearer "):
        return authorization[7:].strip(), True
    custom = request.headers.get("x-stride-token")
    if custom:
        return custom.strip(), True
    return request.cookies.get(AUTH_COOKIE), False


def has_valid_capability(request: Request) -> tuple[bool, bool]:
    provided, is_explicit_bearer = _provided_token(request)
    expected = capability_token()
    return bool(provided and expected and hmac.compare_digest(provided, expected)), is_explicit_bearer


def is_loopback_client(request: Request) -> bool:
    client = request.client
    if not client:
        return False
    try:
        return ipaddress.ip_address(client.host).is_loopback
    except ValueError:
        return False


def cors_origins() -> list[str]:
    raw = os.getenv("STRIDE_CORS_ORIGINS", "http://localhost:5173")
    origins = [item.strip().rstrip("/") for item in raw.split(",") if item.strip()]
    if not origins or "*" in origins:
        raise RuntimeError("STRIDE_CORS_ORIGINS must be a non-empty explicit origin list, never '*'.")
    return origins


def _origin_matches_host(request: Request) -> bool:
    origin = request.headers.get("origin")
    if not origin:
        return False
    parsed = urlparse(origin)
    host = request.headers.get("host", "").lower()
    return bool(parsed.scheme and parsed.netloc and parsed.netloc.lower() == host)


def is_trusted_same_origin(request: Request, allowed_origins: list[str]) -> bool:
    """Allow the served SPA and an explicitly configured local Vite origin."""
    origin = request.headers.get("origin", "").rstrip("/")
    return _origin_matches_host(request) or origin in allowed_origins


def bootstrap_permitted(request: Request, allowed_origins: list[str]) -> bool:
    """Permit a local top-level bootstrap navigation as well as fetch().

    Top-level navigation does not normally send Origin; it cannot read the
    response, and the Strict HttpOnly cookie uses a server-minted value.
    Cross-origin browser fetches do send Origin and are rejected.
    """
    return is_loopback_client(request) and (
        not request.headers.get("origin") or is_trusted_same_origin(request, allowed_origins)
    )


def set_capability_cookie(response: Response, request: Request) -> None:
    # Local-first Stride commonly runs over http://127.0.0.1, where Secure
    # cookies would not work. Operators terminating TLS can force it on.
    secure = _env_truthy("STRIDE_COOKIE_SECURE") or request.url.scheme == "https"
    response.set_cookie(
        AUTH_COOKIE,
        capability_token(),
        httponly=True,
        samesite="strict",
        secure=secure,
        path="/",
    )


class RateLimiter:
    """Small, process-local fixed-window limiter for Garmin-costly operations."""

    def __init__(self) -> None:
        self._events: dict[tuple[str, str], deque[float]] = defaultdict(deque)
        self._lock = threading.Lock()

    def allowed(self, client: str, bucket: str, limit: int, window: float = 60.0) -> bool:
        now = time.monotonic()
        with self._lock:
            events = self._events[(client, bucket)]
            while events and events[0] <= now - window:
                events.popleft()
            if len(events) >= limit:
                return False
            events.append(now)
            return True


_rate_limiter = RateLimiter()


def _rate_bucket(path: str) -> tuple[str, int] | None:
    if path in {f"{API_PREFIX}/sync/run", f"{API_PREFIX}/sync/history"}:
        return ("sync", 4)
    if path.startswith(f"{API_PREFIX}/auth/garmin/"):
        return ("garmin-auth", 10)
    return None


class SecurityMiddleware(BaseHTTPMiddleware):
    """Enforce capabilities, CSRF, rate limits, and common response headers."""

    def __init__(self, app: Any, allowed_origins: list[str]) -> None:
        super().__init__(app)
        self.allowed_origins = allowed_origins

    async def dispatch(self, request: Request, call_next: Any) -> Response:
        path = request.url.path
        method = request.method.upper()
        is_api = path == API_PREFIX or path.startswith(f"{API_PREFIX}/")
        testing = running_under_pytest()

        # CORS preflight carries neither a cookie nor a bearer header. Let the
        # CORS middleware decide it from the explicit origin allowlist.
        if method != "OPTIONS" and is_api and path not in {f"{API_PREFIX}/health", BOOTSTRAP_PATH}:
            # Garmin's OAuth provider redirects cross-site and therefore cannot
            # carry our Strict cookie. Its random pending request token is a
            # one-time transaction capability and is checked again by the route.
            callback_transaction = (
                path == f"{API_PREFIX}/auth/garmin/callback"
                and bool(state.pending_oauth_token)
                and hmac.compare_digest(
                    request.query_params.get("oauth_token", ""), state.pending_oauth_token
                )
            )
            valid, explicit_bearer = has_valid_capability(request)
            if auth_enabled() and not testing and not (valid or callback_transaction):
                return self._protected_response(request, 401, "Capability token required.")

            if method in {"POST", "PUT", "PATCH", "DELETE"} and not testing:
                # An explicit bearer header is not ambient browser authority;
                # cookie authentication must prove it came from our origin.
                if not explicit_bearer and not is_trusted_same_origin(request, self.allowed_origins):
                    return self._protected_response(request, 403, "Same-origin request required.")

            bucket = _rate_bucket(path)
            if bucket and not testing:
                client = request.client.host if request.client else "unknown"
                if not _rate_limiter.allowed(client, *bucket):
                    response = self._protected_response(request, 429, "Too many Garmin requests; retry shortly.")
                    response.headers["Retry-After"] = "60"
                    return response

        if path == BOOTSTRAP_PATH and auth_enabled() and not testing:
            if not bootstrap_permitted(request, self.allowed_origins):
                return self._protected_response(request, 403, "Bootstrap is available only from the local Stride origin.")

        response = await call_next(request)

        # Loading the production SPA establishes the HttpOnly session without
        # putting the capability into HTML or JavaScript. Only loopback clients
        # may mint it; a tailnet browser must present the bearer capability.
        if (
            auth_enabled()
            and not testing
            and method == "GET"
            and not is_api
            and is_loopback_client(request)
            and response.headers.get("content-type", "").startswith("text/html")
        ):
            set_capability_cookie(response, request)

        self._apply_response_headers(response, is_api)
        return response

    def _protected_response(self, request: Request, status: int, detail: str) -> Response:
        response = JSONResponse({"detail": detail}, status_code=status)
        self._apply_response_headers(response, True)
        return response

    @staticmethod
    def _apply_response_headers(response: Response, is_api: bool) -> None:
        response.headers.setdefault(
            "Content-Security-Policy",
            # No third-party origins at all. Fonts are self-hosted; map tiles
            # are proxied by this server so the athlete's IP address is never
            # revealed to a tile provider.
            "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; "
            "form-action 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; "
            "font-src 'self' data:; img-src 'self' data: blob:; connect-src 'self'",
        )
        response.headers.setdefault("X-Content-Type-Options", "nosniff")
        response.headers.setdefault("Referrer-Policy", "no-referrer")
        response.headers.setdefault("X-Frame-Options", "DENY")
        response.headers.setdefault("Permissions-Policy", "camera=(), microphone=(), geolocation=()")
        if is_api:
            response.headers.setdefault("Cache-Control", "no-store")


async def redacted_http_exception_handler(_: Request, exc: HTTPException) -> JSONResponse:
    """Do not reflect a credential should an upstream exception include one."""
    return JSONResponse({"detail": redact_payload(exc.detail)}, status_code=exc.status_code, headers=exc.headers)


# This happens during application import, before any state mutation by routes.
install_capability_state()
