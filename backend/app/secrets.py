"""Secret resolution.

The project's constitution says secrets come from Azure Key Vault via Managed
Identity, never from a checked-in file. Stride honours that: the Garmin
Personal API Key can be read from the vault at startup, so the app connects
without anyone pasting a key.

Resolution order for the Garmin Personal API Key:

1. ``GARMIN_PERSONAL_API_KEY`` in the environment.
2. Azure Key Vault — ``GARMIN_KEYVAULT_NAME`` (default ``claw-agent``) and
   ``GARMIN_KEYVAULT_SECRET`` (default ``garmin-api``) — fetched with the
   Azure CLI, which authenticates through the managed identity.
3. Whatever the user pasted into Settings, which is stored in ``state.json``.

Nothing here raises: a missing vault or a missing CLI just means "no key",
and the app runs with whatever it already has.
"""

from __future__ import annotations

import logging
import os
import shutil
import subprocess

log = logging.getLogger(__name__)

KEYVAULT_NAME = os.getenv("GARMIN_KEYVAULT_NAME", "claw-agent")
KEYVAULT_SECRET = os.getenv("GARMIN_KEYVAULT_SECRET", "garmin-api")


def from_environment() -> str | None:
    value = os.getenv("GARMIN_PERSONAL_API_KEY")
    return value.strip() if value and value.strip() else None


def from_key_vault(
    vault: str = KEYVAULT_NAME, secret: str = KEYVAULT_SECRET, timeout: int = 20
) -> str | None:
    """Read a secret from Azure Key Vault using the Azure CLI."""
    if not shutil.which("az"):
        return None
    try:
        result = subprocess.run(
            [
                "az",
                "keyvault",
                "secret",
                "show",
                "--vault-name",
                vault,
                "--name",
                secret,
                "--query",
                "value",
                "-o",
                "tsv",
            ],
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except (subprocess.TimeoutExpired, OSError) as exc:
        log.debug("Key Vault lookup failed: %s", exc)
        return None
    if result.returncode != 0:
        log.debug("Key Vault lookup returned %s: %s", result.returncode, result.stderr[:200])
        return None
    value = (result.stdout or "").strip()
    return value or None


def resolve_garmin_personal_key() -> tuple[str | None, str | None]:
    """Return ``(key, source)`` — the key and where it came from."""
    value = from_environment()
    if value:
        return value, "environment"
    value = from_key_vault()
    if value:
        return value, "key_vault"
    return None, None
