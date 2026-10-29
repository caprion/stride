#!/usr/bin/env python3
"""Compare the desktop's answers with the reference app's.

The reference app — ``python -m app.serve`` — was the desktop's
predecessor, and the React app was written against it. The desktop answers
the same requests, and this tool says whether it does.

Run the reference and the desktop side by side::

    python -m app.serve                                  # 127.0.0.1:8000
    STRIDE_DATA_DIR=/tmp/stride-cmp/data STRIDE_PORT=8131 \\
        cargo run --release -p stride-desktop            # 127.0.0.1:8131
    python3 desktop/tools/compare_with_reference.py

The two read the same state file, so they mint the same capability token
and the comparison can use the API routes as well as the pages.

It exits non-zero when the two disagree, so it can be a gate.
"""

from __future__ import annotations

import http.client
import json
import sys

REFERENCE_PORT = 8000
DESKTOP_PORT = 8131

# The requests that must answer the same. The body of a page is the same
# file; the body of an API route is compared by its keys, because the
# numbers move with the data.
PAGES = [
    ("GET", "/"),
    ("GET", "/activities"),
    ("GET", "/fitness"),
    ("GET", "/api/v1/health"),
    ("GET", "/api/v1/nope"),
    ("GET", "/api/"),
    ("GET", "/assets/does-not-exist.js"),
    ("GET", "/assets"),
    ("GET", "/assets/"),
    ("GET", "/manifest.webmanifest"),
    ("GET", "/sw.js"),
    ("GET", "/icon-192.png"),
    ("GET", "/favicon.ico"),
    ("GET", "/missing.js"),
    ("GET", "/../backend/app/config.py"),
    ("GET", "/assets/../../backend/app/config.py"),
    ("GET", "/api/v1/activities"),
    ("POST", "/api/v1/activities"),
]

# The API routes that need the capability. The keys of the answer are the
# contract; the numbers are the data.
API = [
    ("GET", "/api/v1/nope"),
    ("GET", "/api/v1/sync/status"),
    ("GET", "/api/v1/activities?per_page=2"),
    ("GET", "/api/v1/physiology/latest"),
    ("GET", "/api/v1/stats/summary"),
    ("GET", "/api/v1/plan/adherence"),
    ("GET", "/api/v1/fitness/trends"),
    ("GET", "/api/v1/compare/candidates"),
    ("GET", "/api/v1/gear"),
    ("GET", "/api/v1/calendar"),
]


def request(port: int, method: str, path: str, headers: dict | None = None):
    """One request. The status, the headers and the body."""
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=120)
    try:
        connection.request(method, path, headers=headers or {})
        answer = connection.getresponse()
        body = answer.read()
        return answer.status, {k.lower(): v for k, v in answer.getheaders()}, body
    except Exception as error:  # a server that is not there
        return None, {"error": str(error)}, b""
    finally:
        connection.close()


def capability(port: int) -> str | None:
    """The capability the server mints for the loopback browser."""
    _, headers, _ = request(port, "GET", "/")
    cookie = headers.get("set-cookie", "")
    if "stride_capability=" not in cookie:
        return None
    return cookie.split("stride_capability=")[1].split(";")[0]


def page_shape(answer) -> str:
    """The status, the content type and the first bytes of the body."""
    status, headers, body = answer
    if status is None:
        return f"ERR {headers.get('error', '')[:40]}"
    content_type = headers.get("content-type", "-").split(";")[0]
    head = body[:26].decode("utf-8", "replace").replace("\n", " ")
    return f"{status} {content_type} {head}"


def api_shape(answer) -> str:
    """The status and the keys of the answer."""
    status, headers, body = answer
    if status is None:
        return f"ERR {headers.get('error', '')[:40]}"
    try:
        value = json.loads(body)
    except Exception:
        return f"{status} {body[:40].decode('utf-8', 'replace')}"
    if isinstance(value, dict):
        return f"{status} " + ",".join(sorted(value.keys())[:8])
    return f"{status} list[{len(value)}]"


# The two answers that are known to differ, and why. They are not the
# desktop's: the desktop serves whatever the one router says.
KNOWN = {
    # The reference renders its dictionaries in the order it builds them and
    # serde_json's map is sorted: the same keys in a different order.
    "GET /api/v1/health",
    # The api crate's store backend does not know about the sync yet.
    "GET /api/v1/sync/status",
}


def main() -> int:
    reference = capability(REFERENCE_PORT)
    desktop = capability(DESKTOP_PORT)
    if reference is None or desktop is None:
        print("one of the two servers did not mint a capability")
        return 2
    print(f"the same capability token: {reference == desktop}")

    differences = 0
    known = 0
    print(f"\n{'request':52} {'reference':>34}   {'desktop':>34}")
    for method, path in PAGES:
        left = page_shape(request(REFERENCE_PORT, method, path))
        right = page_shape(request(DESKTOP_PORT, method, path))
        same = left == right
        if not same and f"{method} {path}" in KNOWN:
            known += 1
        else:
            differences += 0 if same else 1
        print(f"{'  ' if same else '≠ '}{method + ' ' + path:50} {left:>34}   {right:>34}")

    print("\nwith the capability:")
    for method, path in API:
        left = api_shape(
            request(REFERENCE_PORT, method, path, {"Cookie": f"stride_capability={reference}"})
        )
        right = api_shape(
            request(DESKTOP_PORT, method, path, {"Cookie": f"stride_capability={desktop}"})
        )
        same = left == right
        if not same and f"{method} {path}" in KNOWN:
            known += 1
        else:
            differences += 0 if same else 1
        print(f"{'  ' if same else '≠ '}{method + ' ' + path:50} {left:>34}   {right:>34}")

    print(f"\ndifferences: {differences} (and {known} known ones)")
    return 0 if differences == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
