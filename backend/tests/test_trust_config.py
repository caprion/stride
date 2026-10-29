"""The trust configuration and the listener's address.

Three things are pinned here:

* **The default is safe.** Nothing beyond the loopback interface is
  trusted, and a trusted network only counts when the operator has both
  named it and switched the trust on.
* **The trust is not state.** It is read from the environment at load time
  and it is never written to the state file, so a stale value there cannot
  keep a network trusted after it was turned off.
* **The listener detects nothing.** It binds what the environment says and
  nothing else.
"""

from __future__ import annotations

import ipaddress
import json
from pathlib import Path

import pytest


# --------------------------------------------------------------------------
# The configuration itself
# --------------------------------------------------------------------------


def test_default_trust_is_loopback_only():
    """With nothing configured, no remote address is in a trusted network."""
    from app import config

    assert config.TRUST_REMOTE is False
    assert config.trusted_networks() == ()


def test_trusted_networks_parses_a_comma_separated_list():
    from app.config import _parse_networks

    networks = _parse_networks("10.0.0.0/8, 192.168.1.0/24 ,10.1.2.3")
    assert [str(item) for item in networks] == [
        "10.0.0.0/8",
        "192.168.1.0/24",
        "10.1.2.3/32",
    ]


def test_an_unparseable_network_is_dropped_not_guessed():
    """A typo must not widen the trust boundary, and must not stop the server."""
    from app.config import _parse_networks

    assert _parse_networks("not-a-network") == ()
    networks = _parse_networks("10.0.0.0/8, not-a-network")
    assert [str(item) for item in networks] == ["10.0.0.0/8"]


def test_an_empty_list_trusts_nothing_extra():
    from app.config import _parse_networks

    assert _parse_networks("") == ()
    assert _parse_networks("   ,  ") == ()


# --------------------------------------------------------------------------
# The trust decision
# --------------------------------------------------------------------------


def _request_from(peer: str):
    """A request whose TCP peer is ``peer``."""
    from starlette.requests import Request

    scope = {
        "type": "http",
        "method": "GET",
        "path": "/api/v1/activities",
        "headers": [(b"host", b"stride.local")],
        "client": (peer, 50000),
        "query_string": b"",
        "scheme": "http",
        "server": ("stride.local", 8000),
    }
    return Request(scope)


def test_a_client_outside_the_trusted_networks_is_not_trusted(monkeypatch):
    from app import config, security

    monkeypatch.setattr(config, "TRUST_REMOTE", True)
    monkeypatch.setattr(
        config, "TRUSTED_NETWORKS", (ipaddress.ip_network("10.0.0.0/8"),)
    )
    assert security.is_trusted_network_client(_request_from("10.1.2.3")) is True
    assert security.is_trusted_network_client(_request_from("203.0.113.7")) is False
    assert security.is_trusted_remote_client(_request_from("203.0.113.7")) is False


def test_the_trusted_networks_alone_do_not_trust_anybody(monkeypatch):
    """The switch has to be on as well as the networks being named."""
    from app import config, security

    monkeypatch.setattr(config, "TRUST_REMOTE", False)
    monkeypatch.setattr(
        config, "TRUSTED_NETWORKS", (ipaddress.ip_network("10.0.0.0/8"),)
    )
    assert security.is_trusted_network_client(_request_from("10.1.2.3")) is True
    assert security.is_trusted_remote_client(_request_from("10.1.2.3")) is False


def test_the_switch_alone_trusts_nobody(monkeypatch):
    """No networks named means no remote client is trusted."""
    from app import config, security

    monkeypatch.setattr(config, "TRUST_REMOTE", True)
    monkeypatch.setattr(config, "TRUSTED_NETWORKS", ())
    assert security.is_trusted_remote_client(_request_from("10.1.2.3")) is False


def test_loopback_is_always_trusted():
    """The machine itself has the database anyway."""
    from app import security

    request = _request_from("127.0.0.1")
    # The Host header has to be loopback too — that is the rebinding guard.
    request.scope["headers"] = [(b"host", b"127.0.0.1:8000")]
    assert security.is_trusted_client(request) is True


# --------------------------------------------------------------------------
# The trust is not state
# --------------------------------------------------------------------------


def test_the_trust_configuration_is_not_written_to_the_state_file(tmp_path, monkeypatch):
    """A deployment decision is not state."""
    from app import config, security

    # The hardened writer lives in ``app.security`` and reads the path from
    # there, so both have to point at the same file.
    monkeypatch.setattr(config, "STATE_PATH", tmp_path / "state.json")
    monkeypatch.setattr(security, "STATE_PATH", tmp_path / "state.json")
    state = config.AppState()
    state.save()
    written = json.loads((tmp_path / "state.json").read_text())
    assert "trust_remote" not in written
    assert not [key for key in written if key.startswith("trust")]


def test_a_stale_state_file_cannot_keep_a_network_trusted(tmp_path, monkeypatch):
    """The value in the file is ignored, whichever key it is written under."""
    from app import config

    # The legacy key is spelled in parts: the word it contains is a vendor's
    # term, and it does not appear in this repository.
    legacy_key = "trust_" + "tail" + "net"
    path = tmp_path / "state.json"
    path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                legacy_key: True,
                "trust_remote": True,
                "garmin_connected": False,
            }
        )
    )
    monkeypatch.setattr(config, "STATE_PATH", path)
    loaded = config.AppState.load()
    assert not hasattr(loaded, legacy_key)
    assert not hasattr(loaded, "trust_remote")


# --------------------------------------------------------------------------
# The listener
# --------------------------------------------------------------------------


def test_the_listener_defaults_to_every_interface_on_8443(monkeypatch):
    from app import serve

    monkeypatch.delenv("STRIDE_TLS_HOST", raising=False)
    monkeypatch.delenv("STRIDE_TLS_PORT", raising=False)
    assert serve.tls_listener() == ("0.0.0.0", 8443)


def test_the_listener_binds_what_the_environment_says(monkeypatch):
    from app import serve

    monkeypatch.setenv("STRIDE_TLS_HOST", "10.20.30.40")
    monkeypatch.setenv("STRIDE_TLS_PORT", "9443")
    assert serve.tls_listener() == ("10.20.30.40", 9443)


def test_a_nonsense_port_is_refused(monkeypatch):
    from app import serve

    monkeypatch.setenv("STRIDE_TLS_PORT", "not-a-port")
    with pytest.raises(RuntimeError):
        serve.tls_listener()


# --------------------------------------------------------------------------
# The API's answer to a remote client
# --------------------------------------------------------------------------


def _remote_client(peer: str, monkeypatch):
    """A test client whose TCP peer is a remote address."""
    from fastapi.testclient import TestClient

    from app import main

    monkeypatch.setattr(main, "_bootstrap", lambda: None)
    monkeypatch.setattr(main, "_start_scheduler", lambda: None)
    return TestClient(main.app, client=(peer, 50000))


def _enable_authentication(monkeypatch):
    """Turn the capability check on — the suite normally runs it relaxed."""
    from app import security

    monkeypatch.setattr(security, "running_under_pytest", lambda: False)
    monkeypatch.setattr(security, "auth_enabled", lambda: True)


def test_a_remote_client_without_a_capability_is_refused(monkeypatch):
    from app import config

    monkeypatch.setattr(config, "TRUST_REMOTE", False)
    monkeypatch.setattr(config, "TRUSTED_NETWORKS", ())
    _enable_authentication(monkeypatch)
    with _remote_client("203.0.113.7", monkeypatch) as client:
        response = client.get("/api/v1/auth/status")
    assert response.status_code == 401
    assert response.json()["detail"] == "Capability token required."


def test_a_client_from_a_trusted_network_needs_no_pairing_code(monkeypatch):
    from app import config

    monkeypatch.setattr(config, "TRUST_REMOTE", True)
    monkeypatch.setattr(
        config, "TRUSTED_NETWORKS", (ipaddress.ip_network("203.0.113.0/24"),)
    )
    _enable_authentication(monkeypatch)
    with _remote_client("203.0.113.7", monkeypatch) as client:
        response = client.get("/api/v1/auth/status")
    assert response.status_code == 200


def test_a_client_outside_the_trusted_networks_still_needs_a_pairing_code(monkeypatch):
    from app import config

    monkeypatch.setattr(config, "TRUST_REMOTE", True)
    monkeypatch.setattr(
        config, "TRUSTED_NETWORKS", (ipaddress.ip_network("10.0.0.0/8"),)
    )
    _enable_authentication(monkeypatch)
    with _remote_client("203.0.113.7", monkeypatch) as client:
        response = client.get("/api/v1/auth/status")
    assert response.status_code == 401


def test_the_trusted_networks_alone_do_not_open_the_api(monkeypatch):
    from app import config

    monkeypatch.setattr(config, "TRUST_REMOTE", False)
    monkeypatch.setattr(
        config, "TRUSTED_NETWORKS", (ipaddress.ip_network("203.0.113.0/24"),)
    )
    _enable_authentication(monkeypatch)
    with _remote_client("203.0.113.7", monkeypatch) as client:
        response = client.get("/api/v1/auth/status")
    assert response.status_code == 401


# --------------------------------------------------------------------------
# No networking vendor is named anywhere in the backend
# --------------------------------------------------------------------------
#
# The needles are assembled from parts on purpose: the rule is that the word
# does not appear in the backend at all, and a test that spelled it out would
# break it.


_VENDOR_WORDS = ("tail" + "scale", "tail" + "net")
_HARDCODED_RANGE = "100.64.0.0/10"


def _backend_sources() -> list[Path]:
    import app

    return sorted(Path(app.__file__).resolve().parent.rglob("*.py"))


def test_no_networking_vendor_is_named_in_the_backend():
    offenders = []
    for path in _backend_sources():
        text = path.read_text(encoding="utf-8").lower()
        for word in _VENDOR_WORDS:
            if word in text:
                offenders.append(f"{path.name}: {word}")
        if _HARDCODED_RANGE in text:
            offenders.append(f"{path.name}: {_HARDCODED_RANGE}")
    assert offenders == []


def test_the_listener_runs_no_subprocess():
    """The listener must not work out its address by running a command."""
    import app

    source = (Path(app.__file__).resolve().parent / "serve.py").read_text()
    assert "subprocess" not in source
    assert "import subprocess" not in source
