"""The exit-node tool the backend runs.

The backend names no networking product. It runs an executable — the one the
deployment provides — and speaks a small JSON protocol to it. These tests
pin the protocol and the degradation when there is no tool at all.
"""

from __future__ import annotations

import json
import stat
from pathlib import Path

import pytest


@pytest.fixture()
def no_tool(monkeypatch):
    """No exit-node tool: neither named nor on PATH."""
    monkeypatch.setenv("PATH", "")
    monkeypatch.delenv("STRIDE_EXIT_NODE_TOOL", raising=False)


def _tool(tmp_path: Path, body: str, name: str = "fake-exit-node") -> str:
    tmp_path.mkdir(parents=True, exist_ok=True)
    path = tmp_path / name
    path.write_text("#!/bin/sh\n" + body)
    path.chmod(path.stat().st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)
    return str(path)


# --------------------------------------------------------------------------
# No tool at all
# --------------------------------------------------------------------------


def test_without_a_tool_the_feature_is_unavailable(no_tool):
    from app import network

    assert network.exit_node_tooling_available() is False


def test_without_a_tool_the_route_is_direct(no_tool):
    from app import network

    status = network.status()
    assert status["exit_node_tooling_available"] is False
    assert status["exit_node"] is None
    assert status["exit_node_active"] is False
    assert status["route"] == "direct"


def test_without_a_tool_no_node_is_offered(no_tool):
    from app import network

    assert network.available_exit_nodes() == []
    assert network.resolve_exit_node() is None


def test_without_a_tool_the_switch_fails_and_says_why(no_tool):
    from app import network

    ok, message = network.set_exit_node("10.0.0.1")
    assert ok is False
    assert "STRIDE_EXIT_NODE_TOOL" in message


# --------------------------------------------------------------------------
# The protocol
# --------------------------------------------------------------------------


def test_status_reads_the_tool_json(monkeypatch, tmp_path):
    from app import network

    tool = _tool(
        tmp_path,
        'echo \'{"exit_node": "10.0.0.9", "active": true}\'\n',
    )
    monkeypatch.setenv("STRIDE_EXIT_NODE_TOOL", tool)
    status = network.status()
    assert status["exit_node_tooling_available"] is True
    assert status["exit_node"] == "10.0.0.9"
    assert status["exit_node_active"] is True
    assert status["route"] == "exit_node"


def test_nodes_reads_the_tool_json(monkeypatch, tmp_path):
    from app import network

    payload = [
        {"ip": "10.0.0.9", "hostname": "node-a", "online": True, "selected": True},
        {"ip": "10.0.0.10", "hostname": "node-b", "online": False, "selected": False},
    ]
    tool = _tool(tmp_path, "cat <<'EOF'\n" + json.dumps(payload) + "\nEOF\n")
    monkeypatch.setenv("STRIDE_EXIT_NODE_TOOL", tool)
    nodes = network.available_exit_nodes()
    assert [node["ip"] for node in nodes] == ["10.0.0.9", "10.0.0.10"]
    assert nodes[0]["online"] is True
    assert nodes[0]["selected"] is True
    assert nodes[1]["online"] is False
    # The first *online* node is the one that gets used.
    assert network.resolve_exit_node() == "10.0.0.9"


def test_the_switch_is_passed_to_the_tool(monkeypatch, tmp_path):
    from app import network

    tool = _tool(tmp_path, 'echo "$@" > "%s"\n' % (tmp_path / "argv"))
    monkeypatch.setenv("STRIDE_EXIT_NODE_TOOL", tool)
    ok, _ = network.set_exit_node("10.0.0.9")
    assert ok is True
    assert (tmp_path / "argv").read_text().strip() == "set --exit-node=10.0.0.9"


def test_clearing_the_route_asks_for_none(monkeypatch, tmp_path):
    from app import network

    tool = _tool(tmp_path, 'echo "$@" > "%s"\n' % (tmp_path / "argv"))
    monkeypatch.setenv("STRIDE_EXIT_NODE_TOOL", tool)
    ok, _ = network.clear_exit_node()
    assert ok is True
    assert (tmp_path / "argv").read_text().strip() == "set --exit-node=none"


def test_a_tool_that_fails_is_reported_as_a_failure(monkeypatch, tmp_path):
    from app import network

    tool = _tool(tmp_path, "exit 3\n")
    monkeypatch.setenv("STRIDE_EXIT_NODE_TOOL", tool)
    ok, _ = network.set_exit_node("10.0.0.9")
    assert ok is False
    assert network.status()["exit_node"] is None


def test_the_tool_is_looked_up_on_the_path_by_default(monkeypatch, tmp_path):
    """With no name in the environment, the default name is looked up."""
    from app import network

    assert network.DEFAULT_TOOL_NAME == "stride-exit-node"
    _tool(
        tmp_path / "bin",
        'echo \'{"exit_node": null, "active": false}\'\n',
        name="stride-exit-node",
    )
    monkeypatch.setenv("PATH", str(tmp_path / "bin"))
    monkeypatch.delenv("STRIDE_EXIT_NODE_TOOL", raising=False)
    assert network.exit_node_tooling_available() is True
    assert network.status()["exit_node"] is None


def test_nothing_in_the_module_names_a_networking_product():
    """The module configures a network; it does not name one."""
    import app.network

    source = Path(app.network.__file__).read_text(encoding="utf-8").lower()
    for word in ("tail" + "scale", "tail" + "net"):
        assert word not in source


def test_the_protocol_does_not_leak_into_the_state_file():
    """The tool's name is configuration, not state."""
    from app.config import AppState

    fields = set(AppState.__dataclass_fields__)
    assert not [name for name in fields if "tool" in name]
    assert "exit_node_tooling_available" not in fields
