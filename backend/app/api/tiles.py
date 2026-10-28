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
TILE_HOST = "tile.openstreetmap.org"
# A single 256px PNG tile is ~10-20 KB; 2 MB is a generous ceiling.
MAX_TILE_BYTES = 2 * 1024 * 1024
TILE_TIMEOUT = 15
USER_AGENT = "Stride/1.0 (private running log; https://github.com/local/stride)"

CACHE_DIR = Path(DATA_DIR) / "tile-cache"
_CACHE_SECONDS = 30 * 24 * 3600


def _coordinates(z: str, x: str, y: str) -> tuple[int, int, int] | None:
    """Validate tile coordinates against the zoom's grid.

    x and y are indices into a ``2**zoom`` grid, so at zoom 12 they run to
    4095 — they are not small numbers.

    The *parsed* numbers are returned, not the strings: the URL is built from
    them, so nothing off the wire ever reaches the upstream URL.
    """
    try:
        zoom, xi, yi = int(z), int(x), int(y)
    except (TypeError, ValueError):
        return None
    if zoom < 0 or zoom > 19:
        return None
    limit = 2**zoom
    if not (0 <= xi < limit and 0 <= yi < limit):
        return None
    return zoom, xi, yi


def _valid(z: str, x: str, y: str) -> bool:
    return _coordinates(z, x, y) is not None


@router.get("/{z}/{x}/{y}.png")
def get_tile(z: str, x: str, y: str) -> Response:
    """Serve an OpenStreetMap tile from the athlete's own cache."""
    coordinates = _coordinates(z, x, y)
    if coordinates is None:
        raise HTTPException(status_code=400, detail="Invalid tile coordinates.")
    zoom, xi, yi = coordinates

    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    key = hashlib.sha256(f"{zoom}/{xi}/{yi}".encode()).hexdigest()[:32]
    path = CACHE_DIR / f"{key}.png"

    if path.exists() and (time.time() - path.stat().st_mtime) < _CACHE_SECONDS:
        return Response(
            content=path.read_bytes(),
            media_type="image/png",
            headers={"Cache-Control": "public, max-age=86400", "X-Source": "cache"},
        )

    url = TILE_URL.format(z=zoom, x=xi, y=yi)
    try:
        with requests.get(
            url,
            headers={"User-Agent": USER_AGENT, "Accept": "image/png,*/*"},
            timeout=TILE_TIMEOUT,
            # This proxy fetches on the athlete's behalf, so it must fetch
            # exactly the tile it was asked for. A redirect would turn it
            # into a general-purpose fetcher — including one that can be
            # aimed at an internal address or a cloud metadata endpoint.
            allow_redirects=False,
            stream=True,
        ) as response:
            if response.status_code != 200:
                raise HTTPException(status_code=502, detail="Map tile unavailable.")
            # The body is capped while it is read, not after: a hostile or
            # broken upstream cannot make the server buffer it all in RAM.
            body = bytearray()
            for chunk in response.iter_content(64 * 1024):
                body.extend(chunk)
                if len(body) > MAX_TILE_BYTES:
                    raise HTTPException(status_code=502, detail="Map tile too large.")
            content = bytes(body)
    except requests.RequestException as exc:
        log.debug("Tile fetch failed for %s/%s/%s: %s", zoom, xi, yi, type(exc).__name__)
        raise HTTPException(status_code=502, detail="Map tile unavailable.") from exc

    if not content:
        raise HTTPException(status_code=502, detail="Map tile unavailable.")

    try:
        path.write_bytes(content)
    except OSError:  # pragma: no cover - read-only filesystem
        pass

    return Response(
        content=content,
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
