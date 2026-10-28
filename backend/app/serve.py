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


def main() -> int:
    from app.main import app

    http_port = int(os.getenv("STRIDE_PORT", "8000"))
    tls_port = int(os.getenv("STRIDE_TLS_PORT", "8443"))
    host = os.getenv("STRIDE_HOST", "0.0.0.0")

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
        uvicorn.run(app, host=host, port=http_port, log_level="info")
        return 0

    cert, key = tls
    http_config = uvicorn.Config(
        app, host=host, port=http_port, log_level="warning", access_log=False
    )
    http_server = uvicorn.Server(http_config)
    thread = threading.Thread(target=http_server.run, name="stride-http", daemon=True)
    thread.start()
    log.info("plain HTTP on :%d, HTTPS on :%d", http_port, tls_port)

    uvicorn.run(
        app,
        host=host,
        port=tls_port,
        ssl_certfile=str(cert),
        ssl_keyfile=str(key),
        log_level="info",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
