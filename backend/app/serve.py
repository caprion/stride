"""Run Stride's API on two listeners, in one process.

- **Plain HTTP on :8000** — the operator's own machine, and the tailnet.
- **HTTPS on :8443** — the phone.

The phone needs HTTPS for one concrete reason: a service worker only
registers in a *secure context*. On plain HTTP the browser would refuse to
install the app, so the athlete's phone would be stuck with a browser tab
instead of a full-screen app on the home screen. The certificate comes from
Tailscale (``tailscale cert``), so it is a real, publicly trusted one and the
phone shows no warning.

Both listeners serve the same application in the same process, so there is
one scheduler, one database and one state file. Two processes would mean two
auto-syncs racing each other against the Garmin API.

    python -m app.serve
"""

from __future__ import annotations

import logging
import os
import threading
from pathlib import Path

import uvicorn

log = logging.getLogger("stride.serve")

TLS_DIR = Path(os.getenv("STRIDE_TLS_DIR", "data/tls"))
CERT_NAME = "stride.crt"
KEY_NAME = "stride.key"


def _tls_files(directory: Path) -> tuple[Path, Path] | None:
    cert = directory / CERT_NAME
    key = directory / KEY_NAME
    if cert.exists() and key.exists():
        return cert, key
    return None


def _close_inherited_descriptors() -> None:
    """Close every descriptor above stderr.

    A daemon that inherits a pipe keeps it open, and whoever wrote to that
    pipe waits for an end-of-file that never comes. It is the reason a
    start script can appear to hang when its output goes to a pipe rather
    than to a file.

    Nothing above stderr is needed: the log file is already open as stdout
    and stderr.
    """
    try:
        import os

        keep = {0, 1, 2}
        for fd in sorted(os.listdir("/proc/self/fd"), reverse=True):
            try:
                number = int(fd)
            except ValueError:
                continue
            if number in keep:
                continue
            try:
                os.close(number)
            except OSError:
                pass
    except (ImportError, OSError):  # pragma: no cover - not Linux
        pass


def _tailscale_ipv4() -> str | None:
    """This machine's address on the tailnet, if it has one."""
    import subprocess

    try:
        result = subprocess.run(
            ["tailscale", "ip", "-4"],
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    address = result.stdout.strip().splitlines()
    return address[0].strip() if address else None


def main() -> int:
    _close_inherited_descriptors()

    from app.main import app
    from app.security import ensure_safe_bind_configuration

    http_port = int(os.getenv("STRIDE_PORT", "8000"))
    tls_port = int(os.getenv("STRIDE_TLS_PORT", "8443"))
    # One address for both listeners. ``STRIDE_BIND`` is the documented
    # control; ``STRIDE_HOST`` is the older name and still honoured.
    # The plain listener is for the browser on this machine, so it binds
    # loopback. The TLS listener is for the phone, so it binds the tailnet
    # address. Nothing is ever bound on the public interface.
    host = os.getenv("STRIDE_BIND") or os.getenv("STRIDE_HOST") or "127.0.0.1"
    tls_host = (
        os.getenv("STRIDE_TLS_HOST")
        or os.getenv("STRIDE_TLS_BIND")
        or _tailscale_ipv4()
        or "127.0.0.1"
    )
    if tls_host == "127.0.0.1":
        log.warning(
            "no tailnet address found - the HTTPS listener is on loopback only, "
            "so a phone cannot reach it"
        )

    # The safety check has to see the address this process is *about* to
    # bind. It used to read a different variable, so the guard was inert
    # here: ``STRIDE_DISABLE_AUTH=true python -m app.serve`` bound every
    # interface and served the whole API to the network with no
    # authentication at all, while the check believed the bind was
    # 127.0.0.1.
    #
    # Both listeners are checked. The TLS one is the one the phone reaches,
    # so it is the one that matters: a loopback plain listener with a
    # tailnet TLS listener is still an exposed API.
    ensure_safe_bind_configuration(host)
    ensure_safe_bind_configuration(tls_host)

    tls = _tls_files(TLS_DIR)
    if tls is None:
        log.warning(
            "no TLS certificate in %s — the app will be reachable over plain "
            "HTTP only, and a phone cannot install it",
            TLS_DIR,
        )

    # The TLS listener runs in this thread; the plain one in the other. The
    # thread that calls ``serve`` owns the process, so give it the listener
    # that matters least.
    if tls is None:
        uvicorn.run(app, host=host, port=http_port, log_level="info", proxy_headers=False)
        return 0

    cert, key = tls
    http_config = uvicorn.Config(
        app,
        host=host,
        port=http_port,
        log_level="warning",
        access_log=False,
        # uvicorn trusts X-Forwarded-For from 127.0.0.1 by default. Stride
        # is not behind a proxy, and a proxy on this machine would be able to
        # make a remote client look like a loopback one — which is exactly
        # the check that guards pairing codes and the capability cookie.
        proxy_headers=False,
    )
    http_server = uvicorn.Server(http_config)
    thread = threading.Thread(target=http_server.run, name="stride-http", daemon=True)
    thread.start()
    log.info("plain HTTP on :%d, HTTPS on :%d", http_port, tls_port)

    uvicorn.run(
        app,
        host=tls_host,
        port=tls_port,
        ssl_certfile=str(cert),
        ssl_keyfile=str(key),
        log_level="info",
        proxy_headers=False,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
