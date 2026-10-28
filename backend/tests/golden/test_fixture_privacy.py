"""The committed fixtures must not point at a real place.

The repository is public and the athlete's real routes are not ours to
publish. Every position in every fixture is therefore fabricated, and they
all describe the same point: 35° south, 140° west, in the South Pacific,
about 15 500 km from the nearest point of the athlete's real routes and
4 000 km from the nearest coast.

The nine synthetic cases used to start at 12.9716/77.5946 — the centre of
Bengaluru, and 10.7 km from a real route point. That was fixed by hand
once; this test is what keeps it fixed.

It is a guard, not a description: the rule is "every position in the
committed tree is the synthetic anchor", and the athlete's route box is
spelled out separately so the reason survives even if the anchor moves.
"""

from __future__ import annotations

import json
import math
from pathlib import Path

import fitparse

HERE = Path(__file__).resolve().parent
FIXTURE_DIR = HERE / "fixtures"
GOLDEN_DIR = HERE / "golden"

# The synthetic anchor: the South Pacific.
ANCHOR_LAT = -35.0
ANCHOR_LON = -140.0

# How far a position may stray from the anchor and still be synthetic. The
# tracks span less than 0.01 degrees, so this is generous by three orders of
# magnitude and still 4 000 km from any coast.
ANCHOR_RADIUS_DEGREES = 1.0

# The athlete's real routes, measured from the thirteen files downloaded from
# their Garmin account: 13.067-21.126 N, 77.559-79.072 E. Nothing committed
# may fall inside this box, widened by a tenth of a degree.
REAL_ROUTE_LAT = (13.067, 21.126)
REAL_ROUTE_LON = (77.559, 79.072)
REAL_ROUTE_MARGIN = 0.1


def _degrees(semicircles: float) -> float:
    """FIT stores positions in semicircles."""
    return semicircles * 180.0 / (2**31)


def _fixture_positions(path: Path) -> list[tuple[float, float]]:
    """Every position in a FIT file, in degrees."""
    positions: list[tuple[float, float]] = []
    for message in fitparse.FitFile(str(path)).get_messages("record"):
        lat = message.get("position_lat")
        lon = message.get("position_long")
        if lat is None or lon is None:
            continue
        latitude = getattr(lat, "value", None)
        longitude = getattr(lon, "value", None)
        if latitude is None or longitude is None:
            continue
        positions.append((_degrees(latitude), _degrees(longitude)))
    return positions


def _golden_positions(path: Path) -> list[tuple[float, float]]:
    """Every position a golden file records, in degrees."""
    payload = json.loads(path.read_text(encoding="utf-8"))
    geo = payload.get("import", {}).get("geo_points", {})
    points: list[tuple[float, float]] = []
    for group in (geo.get("first") or [], geo.get("last") or []):
        for point in group:
            points.append((float(point[0]), float(point[1])))
    return points


def _distance_degrees(a: tuple[float, float], b: tuple[float, float]) -> float:
    """A plain degree distance; the anchor is far enough away that the
    curvature of the earth does not matter here."""
    return math.hypot(a[0] - b[0], a[1] - b[1])


def test_every_fixture_position_is_the_synthetic_anchor():
    """No committed fixture may carry a coordinate that is not the anchor."""
    fixtures = sorted(FIXTURE_DIR.glob("*.fit"))
    assert fixtures, "the fixtures are missing"
    for path in fixtures:
        for latitude, longitude in _fixture_positions(path):
            distance = _distance_degrees((latitude, longitude), (ANCHOR_LAT, ANCHOR_LON))
            assert distance <= ANCHOR_RADIUS_DEGREES, (
                f"{path.name}: the position {latitude}, {longitude} is {distance:.3f} degrees "
                f"from the synthetic anchor {ANCHOR_LAT}, {ANCHOR_LON}"
            )


def test_no_committed_position_is_near_the_athletes_routes():
    """The rule above, stated as the reason: nothing real is published."""
    for path in sorted(FIXTURE_DIR.glob("*.fit")):
        for latitude, longitude in _fixture_positions(path):
            inside = (
                REAL_ROUTE_LAT[0] - REAL_ROUTE_MARGIN
                <= latitude
                <= REAL_ROUTE_LAT[1] + REAL_ROUTE_MARGIN
                and REAL_ROUTE_LON[0] - REAL_ROUTE_MARGIN
                <= longitude
                <= REAL_ROUTE_LON[1] + REAL_ROUTE_MARGIN
            )
            assert not inside, (
                f"{path.name}: the position {latitude}, {longitude} is inside the athlete's "
                "real route box"
            )


def test_every_golden_position_is_the_synthetic_anchor():
    """The golden files carry the positions too, so they get the same guard."""
    for path in sorted(GOLDEN_DIR.glob("*.json")):
        for latitude, longitude in _golden_positions(path):
            distance = _distance_degrees((latitude, longitude), (ANCHOR_LAT, ANCHOR_LON))
            assert distance <= ANCHOR_RADIUS_DEGREES, (
                f"{path.name}: the position {latitude}, {longitude} is {distance:.3f} degrees "
                f"from the synthetic anchor {ANCHOR_LAT}, {ANCHOR_LON}"
            )
