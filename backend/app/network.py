"""Network route management.

Garmin throttles datacentre IPs hard (HTTP 429 on every login and token
request). This VM is Azure/AS8075, so a straight-line sync from here is rate
limited. The fleet already has a residential node — the raspberrypi at home —
which advertises itself as a Tailscale exit node.

So: the app can route its Garmin traffic through the exit node. We do that by
toggling the node's own exit-node setting, which is a local, reversible
operation (``tailscale set --exit-node=<ip>`` / ``--exit-node=none``).

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
import shutil
import subprocess
from typing import Any

log = logging.getLogger(__name__)


def _tailscale() -> str | None:
    return shutil.which("tailscale")


def tailscale_available() -> bool:
    return _tailscale() is not None


def _run(args: list[str], timeout: int = 20) -> tuple[int, str]:
    binary = _tailscale()
    if not binary:
        return 127, "tailscale not installed"
    try:
        proc = subprocess.run(
            [binary, *args], capture_output=True, text=True, timeout=timeout
        )
        return proc.returncode, (proc.stdout or "") + (proc.stderr or "")
    except (subprocess.TimeoutExpired, OSError) as exc:
        return 1, str(exc)


def status() -> dict[str, Any]:
    """Current exit-node configuration of this machine."""
    info: dict[str, Any] = {
        "tailscale_available": tailscale_available(),
        "exit_node": None,
        "exit_node_active": False,
        "public_ip": None,
        "public_org": None,
    }
    code, out = _run(["status", "--json"])
    if code == 0:
        try:
            data = json.loads(out)
            self_state = data.get("Self") or {}
            info["exit_node"] = self_state.get("ExitNodeIP") or None
            info["exit_node_active"] = bool(self_state.get("ExitNodeIP"))
        except json.JSONDecodeError:
            pass
    return info


def public_identity() -> dict[str, str | None]:
    """Best-effort public IP + network operator, for the diagnostics panel."""
    import urllib.request

    try:
        with urllib.request.urlopen("https://ipinfo.io/json", timeout=12) as response:
            data = json.loads(response.read())
            return {"ip": data.get("ip"), "org": data.get("org")}
    except Exception:
        return {"ip": None, "org": None}


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
    """Exit nodes advertised on the tailnet."""
    code, out = _run(["exit-node", "list", "--json"])
    if code != 0:
        return []
    try:
        data = json.loads(out)
    except json.JSONDecodeError:
        return []
    nodes: list[dict[str, Any]] = []
    for entry in data if isinstance(data, list) else []:
        if not isinstance(entry, dict):
            continue
        nodes.append(
            {
                "id": entry.get("ID") or entry.get("id"),
                "ip": entry.get("IPAddresses", [None])[0] if entry.get("IPAddresses") else entry.get("IP"),
                "hostname": entry.get("Hostname") or entry.get("hostname"),
                "online": entry.get("Online", entry.get("online")),
            }
        )
    return nodes
