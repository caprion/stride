"""Credential redaction.

Garmin payloads, exception messages and log lines can carry credentials.
Nothing that leaves the process should contain one, so every payload that is
persisted or logged goes through here first.
"""

from __future__ import annotations

import re
from typing import Any

# Keys whose values are credentials, matched case-insensitively on the whole
# key (not a substring, so `hrv_status` does not match `token`).
SECRET_KEY_PATTERN = re.compile(
    r"(^|[_.-])(api[_-]?key|consumer[_-]?secret|oauth[_-]?token[_-]?secret|"
    r"oauth[_-]?token|password|passwd|secret|token|apikey|bearer|authorization|"
    r"credential|private[_-]?key)([_.-]|$)",
    re.IGNORECASE,
)

# Values that look like a Garmin Personal API key.
GARMIN_KEY_PATTERN = re.compile(r"GCDC_[A-Za-z0-9._-]{20,}")

REDACTED = "[redacted]"


def is_secret_key(key: str) -> bool:
    """True when a field name suggests the value is a credential."""
    return bool(SECRET_KEY_PATTERN.search(key))


def redact_text(value: Any) -> Any:
    """Scrub credential-shaped substrings out of a string."""
    if not isinstance(value, str):
        return value
    return GARMIN_KEY_PATTERN.sub(REDACTED, value)


def redact(payload: Any, _depth: int = 0) -> Any:
    """Return a copy of ``payload`` with credentials removed.

    Works on nested dictionaries and lists, and scrubs credential-shaped
    strings wherever they appear — including inside free-text error messages.
    """
    if _depth > 12:
        return REDACTED
    if isinstance(payload, dict):
        out: dict[Any, Any] = {}
        for key, value in payload.items():
            if isinstance(key, str) and is_secret_key(key):
                out[key] = REDACTED if value not in (None, "") else None
            else:
                out[key] = redact(value, _depth + 1)
        return out
    if isinstance(payload, list):
        return [redact(item, _depth + 1) for item in payload]
    if isinstance(payload, tuple):
        return tuple(redact(item, _depth + 1) for item in payload)
    if isinstance(payload, str):
        return redact_text(payload)
    return payload


def safe_error(exc: BaseException, limit: int = 400) -> str:
    """A one-line, credential-free description of an exception."""
    text = f"{type(exc).__name__}: {exc}"
    return redact_text(text)[:limit]
