#!/usr/bin/env python3
"""Generate the PWA / app icons.

The mark is the Stride pulse on a deep-azure rounded square, drawn here so
the icon is a build artefact rather than a binary nobody can regenerate.

    python3 scripts/make_icons.py
"""

from __future__ import annotations

import pathlib

from PIL import Image, ImageDraw

ROOT = pathlib.Path(__file__).resolve().parent.parent
PUBLIC = ROOT / "frontend" / "public"

BRAND_700 = (13, 58, 99)  # #0D3A63
BRAND_900 = (5, 29, 51)  # #051D33
AMBER = (242, 176, 30)  # #F2B01E
WHITE = (255, 255, 255)

# The pulse: a run profile — down, up, dip, peak, settle.
PULSE = [
    (0.22, 0.66),
    (0.38, 0.34),
    (0.52, 0.56),
    (0.64, 0.30),
    (0.82, 0.70),
]


def draw_icon(size: int, maskable: bool = False) -> Image.Image:
    """Draw the icon at ``size`` pixels square.

    A maskable icon is cropped to a circle by the launcher, so the mark is
    drawn smaller and centred, with the background filling the whole square.
    """
    scale = 4
    canvas = Image.new("RGBA", (size * scale, size * scale), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)

    side = size * scale
    if maskable:
        draw.rectangle([0, 0, side, side], fill=BRAND_700)
        inset = side * 0.22
    else:
        radius = int(side * 0.22)
        draw.rounded_rectangle([0, 0, side - 1, side - 1], radius=radius, fill=BRAND_700)
        inset = side * 0.16

    left, right = inset, side - inset
    points = [
        (left + (right - left) * x, inset + (right - inset) * y) for x, y in PULSE
    ]
    width = max(2, int(side * (0.055 if not maskable else 0.045)))
    draw.line(points, fill=AMBER, width=width, joint="curve")
    # Round the ends so the stroke reads as a pulse, not a chart.
    radius = width / 2
    for x, y in (points[0], points[-1]):
        draw.ellipse([x - radius, y - radius, x + radius, y + radius], fill=AMBER)

    return canvas.resize((size, size), Image.LANCZOS)


def main() -> int:
    PUBLIC.mkdir(parents=True, exist_ok=True)
    targets = [
        ("icon-192.png", 192, False),
        ("icon-512.png", 512, False),
        ("icon-maskable-512.png", 512, True),
        ("apple-touch-icon.png", 180, False),
    ]
    for name, size, maskable in targets:
        path = PUBLIC / name
        draw_icon(size, maskable).save(path, "PNG", optimize=True)
        print(f"wrote {path.relative_to(ROOT)} ({size}x{size})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
