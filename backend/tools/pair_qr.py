"""Render a pairing code as a QR code in the terminal.

The operator is at the desktop and wants another browser signed in. A QR
code turns "read this off the screen and type it" into "point the camera
at the screen" — the difference between a chore and a moment.

**The phone app never needs this.** It has its own database and its own
Garmin key; it is signed in from the first pixel. This is for a second
browser, on another machine, that wants to read the desktop's data.

The code is carried in the URL *fragment*, not the query string: a fragment
is never sent to the server, so it does not land in an access log, and the
app clears it from the address bar the moment it is used.
"""

from __future__ import annotations

import sys

import segno


def render(url: str, scale: int = 2) -> str:
    """A QR code as text, using half-block characters.

    ``scale`` widens each module so the code is big enough for a phone
    camera to read off a screen. A terminal cell is about twice as tall as it
    is wide, so two matrix rows go into one character line — that keeps the
    modules square instead of stretching them.
    """
    qr = segno.make(url, error="m")
    matrix = [list(row) for row in qr.matrix]
    out: list[str] = []
    for index in range(0, len(matrix), 2):
        top = matrix[index]
        bottom = matrix[index + 1] if index + 1 < len(matrix) else [0] * len(matrix)
        line = []
        for left, right in zip(top, bottom):
            if left and right:
                block = "█"
            elif left:
                block = "▀"
            elif right:
                block = "▄"
            else:
                block = " "
            line.append(block * scale)
        out.append("".join(line))
    quiet = "█" * (len(out[0]) + 4 * scale)
    return "\n".join([quiet] + ["█" * 2 * scale + line + "█" * 2 * scale for line in out] + [quiet])


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if not args:
        print("usage: python -m tools.pair_qr <url>", file=sys.stderr)
        return 2
    print(render(args[0]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
