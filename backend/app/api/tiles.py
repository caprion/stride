"""Map tile proxy.

The browser must not talk to a third party. Every tile request to
``tile.openstreetmap.org`` tells OpenStreetMap the athlete's IP address and
that they are looking at a running map — which is exactly the kind of signal a
privacy-forward product should not be handing out.

So the tiles are fetched **by the athlete's own server**, with an identifying
User-Agent, cached on disk, and served back from the same origin. The browser
only ever talks to us.

OpenStreetMap's tile policy asks for a valid User-Agent and discourages bulk
downloading; the on-disk cache here is exactly the polite pattern.
"""

from __future__ import annotations

import hashlib
import logging
import re
import time
from pathlib import Path

import requests
from fastapi import APIRouter, HTTPException, Response

from ..config import DATA_DIR

log = logging.getLogger(__name__)

router = APIRouter(prefix="/tiles", tags=["tiles"])

TILE_URL = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
# A single 256px PNG tile is ~10-20 KB; 2 MB is a generous ceiling.
MAX_TILE_BYTES = 2 * 1024 * 1024
TILE_TIMEOUT = 15
USER_AGENT = "Stride/1.0 (private running log; https://github.com/local/stride)"

CACHE_DIR = Path(DATA_DIR) / "tile-cache"
_CACHE_SECONDS = 30 * 24 * 3600


def _valid(z: str, x: str, y: str) -> bool:
    """Validate tile coordinates against the zoom's grid.

    x and y are indices into a ``2**zoom`` grid, so at zoom 12 they run to
    4095 — they are not small numbers.
    """
    try:
        zoom, xi, yi = int(z), int(x), int(y)
    except (TypeError, ValueError):
        return False
    if zoom < 0 or zoom > 19:
        return False
    limit = 2**zoom
    return 0 <= xi < limit and 0 <= yi < limit


@router.get("/{z}/{x}/{y}.png")
def get_tile(z: str, x: str, y: str) -> Response:
    """Serve an OpenStreetMap tile from the athlete's own cache."""
    if not _valid(z, x, y):
        raise HTTPException(status_code=400, detail="Invalid tile coordinates.")

    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    key = hashlib.sha256(f"{z}/{x}/{y}".encode()).hexdigest()[:32]
    path = CACHE_DIR / f"{key}.png"

    if path.exists() and (time.time() - path.stat().st_mtime) < _CACHE_SECONDS:
        return Response(
            content=path.read_bytes(),
            media_type="image/png",
            headers={"Cache-Control": "public, max-age=86400", "X-Source": "cache"},
        )

    try:
        response = requests.get(
            TILE_URL.format(z=z, x=x, y=y),
            headers={"User-Agent": USER_AGENT, "Accept": "image/png,*/*"},
            timeout=TILE_TIMEOUT,
        )
    except requests.RequestException as exc:
        log.debug("Tile fetch failed for %s/%s/%s: %s", z, x, y, type(exc).__name__)
        raise HTTPException(status_code=502, detail="Map tile unavailable.") from exc

    if response.status_code != 200 or not response.content:
        raise HTTPException(status_code=502, detail="Map tile unavailable.")
    if len(response.content) > MAX_TILE_BYTES:
        raise HTTPException(status_code=502, detail="Map tile too large.")

    try:
        path.write_bytes(response.content)
    except OSError:  # pragma: no cover - read-only filesystem
        pass

    return Response(
        content=response.content,
        media_type="image/png",
        headers={"Cache-Control": "public, max-age=86400", "X-Source": "upstream"},
    )


@router.get("/policy")
def tile_policy() -> dict[str, object]:
    """Where the tiles come from — surfaced so the UI can explain it."""
    return {
        "upstream": "openstreetmap.org",
        "proxied": True,
        "reason": "Tiles are fetched by this server and cached, so the browser never "
        "reveals the athlete's IP address to a third party.",
        "user_agent": USER_AGENT,
        "cache_dir": str(CACHE_DIR),
    }
