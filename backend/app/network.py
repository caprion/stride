"""Egress route management.

Garmin throttles datacentre IPs hard (HTTP 429 on every login and token
request). A server that runs in a datacentre therefore needs a way out:
the app can route its outbound traffic through an **exit node** — another
machine on the operator's network that forwards traffic — so the provider
sees the operator's own address instead of the datacentre one.

Which tool performs that switch is the *deployment's* business, not the
application's. The backend asks for an executable named by
``STRIDE_EXIT_NODE_TOOL`` and speaks a small JSON protocol to it:

``<tool> status --json``
    ``{"exit_node": "<address>|null", "active": true|false}``

``<tool> nodes --json``
    ``[{"ip": "...", "hostname": "...", "online": true, "selected": false}]``

``<tool> set --exit-node=<address|none>``
    exit status 0 when the route was changed.

No particular networking product is named here, and none is detected: the
backend runs no command to work out where the network is, it only runs the
tool the operator told it about. With no tool configured the feature is
simply unavailable, and the app says so rather than guessing.

Modes
-----
``direct``     no exit node; use the machine's normal egress.
``exit_node``  route all egress through the configured exit node.
``auto``       start direct; if Garmin answers 429/403, switch to the exit
               node, retry once, and remember the working route.
"""

from __future__ import annotations

import json
import logging
import os
import shutil
import subprocess
from typing import Any

log = logging.getLogger(__name__)

#: The executable that performs the switch. Unset means "the default name".
TOOL_ENV = "STRIDE_EXIT_NODE_TOOL"

#: The name the tool is looked up under on PATH when the environment does
#: not name one. A deployment provides it; the repository does not.
DEFAULT_TOOL_NAME = "stride-exit-node"

_TOOL_TIMEOUT = 20


def _tool() -> str | None:
    """The configured exit-node tool, resolved to an executable path."""
    configured = os.getenv(TOOL_ENV, "").strip() or DEFAULT_TOOL_NAME
    if os.path.sep in configured:
        return configured if os.path.isfile(configured) else None
    return shutil.which(configured)


def exit_node_tooling_available() -> bool:
    """Is an exit-node tool configured and executable?"""
    return _tool() is not None


def _run(args: list[str], timeout: int = _TOOL_TIMEOUT) -> tuple[int, str]:
    binary = _tool()
    if not binary:
        return 127, f"no exit-node tool found ({TOOL_ENV} or {DEFAULT_TOOL_NAME} on PATH)"
    try:
        proc = subprocess.run(
            [binary, *args], capture_output=True, text=True, timeout=timeout
        )
    except (subprocess.TimeoutExpired, OSError) as exc:
        return 1, str(exc)
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def status() -> dict[str, Any]:
    """The current exit-node configuration of this machine."""
    info: dict[str, Any] = {
        "exit_node_tooling_available": exit_node_tooling_available(),
        "exit_node": None,
        "exit_node_active": False,
        "public_ip": None,
        "public_org": None,
    }
    code, out = _run(["status", "--json"])
    if code == 0:
        try:
            data = json.loads(out)
        except json.JSONDecodeError:
            data = {}
        if isinstance(data, dict):
            address = data.get("exit_node")
            info["exit_node"] = str(address) if address else None
            info["exit_node_active"] = bool(data.get("active"))
    # The *actual* route, as opposed to the configured preference.
    info["route"] = "exit_node" if info["exit_node_active"] else "direct"
    return info


_IDENTITY_CACHE: dict[str, Any] = {"value": None, "at": 0.0}
_IDENTITY_TTL = 600.0


def public_identity(force: bool = False) -> dict[str, str | None]:
    """Best-effort public IP + network operator, for the diagnostics panel.

    The answer changes slowly, so it is cached for ten minutes — the network
    panel is a diagnostic, not a health check, and it must not make an
    external call on every page view.
    """
    import time
    import urllib.request

    now = time.time()
    if not force and _IDENTITY_CACHE["value"] and now - _IDENTITY_CACHE["at"] < _IDENTITY_TTL:
        return _IDENTITY_CACHE["value"]
    try:
        with urllib.request.urlopen("https://ipinfo.io/json", timeout=12) as response:
            data = json.loads(response.read())
            value = {"ip": data.get("ip"), "org": data.get("org")}
    except Exception:
        value = _IDENTITY_CACHE["value"] or {"ip": None, "org": None}
    _IDENTITY_CACHE["value"] = value
    _IDENTITY_CACHE["at"] = now
    return value


def set_exit_node(node: str | None) -> tuple[bool, str]:
    """Route egress through ``node`` (``None`` disables it)."""
    target = node or "none"
    code, out = _run(["set", f"--exit-node={target}"])
    ok = code == 0
    if not ok:
        log.warning("Failed to set exit node to %s: %s", target, out.strip())
    return ok, out.strip()


def clear_exit_node() -> tuple[bool, str]:
    return set_exit_node(None)


def available_exit_nodes() -> list[dict[str, Any]]:
    """The exit nodes the configured tool advertises."""
    code, out = _run(["nodes", "--json"])
    if code != 0:
        return []
    try:
        data = json.loads(out)
    except json.JSONDecodeError:
        return []
    if not isinstance(data, list):
        return []
    nodes: list[dict[str, Any]] = []
    for entry in data:
        if not isinstance(entry, dict):
            continue
        address = entry.get("ip")
        if not address:
            continue
        nodes.append(
            {
                "id": str(entry.get("hostname") or address),
                "ip": str(address),
                "hostname": str(entry.get("hostname") or ""),
                "online": bool(entry.get("online", True)),
                "selected": bool(entry.get("selected", False)),
            }
        )
    return nodes


def resolve_exit_node() -> str | None:
    """Pick an exit node when none is configured.

    The first online node the tool advertises is the one that is used, so a
    deployment with a single residential node works without any extra
    configuration.
    """
    for node in available_exit_nodes():
        if node.get("online") and node.get("ip"):
            return node["ip"]
    return None
