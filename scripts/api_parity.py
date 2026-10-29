#!/usr/bin/env python3
"""Compare the Rust core's API with the Python reference, route by route.

The React app is a consumer of the API, and it is not being rewritten. So the
Rust router has to answer the same paths with the same shapes — and the
shapes were written by four agents working in parallel against a frozen
contract, which is exactly the situation where two implementations drift
apart without either noticing.

This runs both implementations, hits the same routes, and compares the
*shape* of the JSON each returns: the keys, their types, and the nesting.
The values are allowed to differ — the Rust core may be reading a different
database — but the shape may not.

    python3 scripts/api_parity.py                 # both, on the default ports
    python3 scripts/api_parity.py --python http://127.0.0.1:8000 \\
                                 --rust    http://127.0.0.1:8099
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request

# The routes a browser actually visits. The rest of the API is either
# administrative or exercised by the golden tests.
ROUTES: list[tuple[str, str]] = [
    ("GET", "/api/v1/health"),
    ("GET", "/api/v1/auth/status"),
    ("GET", "/api/v1/activities?limit=5"),
    ("GET", "/api/v1/activities/1"),
    ("GET", "/api/v1/activities?limit=5&offset=0&sport=running"),
    ("GET", "/api/v1/calendar"),
    ("GET", "/api/v1/gear"),
    ("GET", "/api/v1/compare/candidates"),
    ("GET", "/api/v1/fitness/fingerprint"),
    ("GET", "/api/v1/fitness/activity/1"),
    ("GET", "/api/v1/fitness/trends?metric=decoupling_pct"),
    ("GET", "/api/v1/fitness/zones"),
    ("GET", "/api/v1/physiology/daily"),
    ("GET", "/api/v1/physiology/hrv"),
    ("GET", "/api/v1/physiology/latest"),
    ("GET", "/api/v1/physiology/readiness"),
    ("GET", "/api/v1/physiology/sleep"),
    ("GET", "/api/v1/physiology/sources"),
    ("GET", "/api/v1/plan/current"),
    ("GET", "/api/v1/plan/today"),
    ("GET", "/api/v1/plan/week?date=2026-10-28"),
    ("GET", "/api/v1/plan/results"),
    ("GET", "/api/v1/plan/adherence"),
    ("GET", "/api/v1/plan/blocks"),
    ("GET", "/api/v1/stats/summary"),
    ("GET", "/api/v1/stats/monthly"),
    ("GET", "/api/v1/stats/weekly"),
    ("GET", "/api/v1/stats/pace-trends"),
    ("GET", "/api/v1/stats/personal-records"),
    ("GET", "/api/v1/stats/heatmap"),
    ("GET", "/api/v1/stats/elevation"),
    ("GET", "/api/v1/stats/hr-zones"),
    ("GET", "/api/v1/sync/status"),
    ("GET", "/api/v1/sync/runs?limit=5"),
    ("GET", "/api/v1/sync/history?limit=5"),
    ("GET", "/api/v1/network/status"),
    ("GET", "/api/v1/tiles/policy"),
    ("GET", "/api/v1/nonexistent-route"),
]


def bootstrap(base: str) -> str:
    """The capability the server mints for a loopback browser.

    The Rust core enforces the capability on every route, so a comparison
    that does not have one is comparing 401s with 200s.
    """
    try:
        with urllib.request.urlopen(f"{base.rstrip('/')}/api/v1/bootstrap", timeout=10) as response:
            cookie = response.headers.get("set-cookie", "")
    except Exception:
        return ""
    return cookie.split(";")[0]


def get(base: str, path: str, cookie: str = "") -> tuple[int, object]:
    url = f"{base.rstrip('/')}{path}"
    request = urllib.request.Request(url, method="GET")
    if cookie:
        request.add_header("Cookie", cookie)
    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            body = response.read().decode("utf-8", "replace")
            status = response.status
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", "replace")
        status = exc.code
    except Exception as exc:  # noqa: BLE001
        return 0, {"error": str(exc)}
    try:
        return status, json.loads(body)
    except json.JSONDecodeError:
        return status, {"__not_json__": body[:200]}


def shape(value: object, depth: int = 0) -> object:
    """The shape of a value: its keys and their types, not its contents."""
    if depth > 6:
        return "..."
    if isinstance(value, dict):
        return {k: shape(v, depth + 1) for k, v in sorted(value.items())}
    if isinstance(value, list):
        return [shape(value[0], depth + 1)] if value else []
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "bool"
    if isinstance(value, (int, float)):
        return "number"
    if isinstance(value, str):
        return "string"
    return type(value).__name__


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--python", default="http://127.0.0.1:8000")
    parser.add_argument("--rust", default="http://127.0.0.1:8099")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args(argv)

    same = 0
    different: list[str] = []
    unreachable: list[str] = []

    py_cookie = bootstrap(args.python)
    rs_cookie = bootstrap(args.rust)

    for method, path in ROUTES:
        py_status, py_body = get(args.python, path, py_cookie)
        rs_status, rs_body = get(args.rust, path, rs_cookie)

        if rs_status == 0:
            unreachable.append(f"{method} {path}: {rs_body}")
            continue
        if py_status != rs_status:
            different.append(
                f"{method} {path}: status {py_status} (python) vs {rs_status} (rust)"
            )
            continue
        py_shape = shape(py_body)
        rs_shape = shape(rs_body)
        if py_shape == rs_shape:
            same += 1
            if args.verbose:
                print(f"  same  {method} {path}")
        else:
            different.append(f"{method} {path}: the response shape differs")
            if args.verbose:
                print(f"  python {method} {path}: {json.dumps(py_shape)[:400]}")
                print(f"  rust   {method} {path}: {json.dumps(rs_shape)[:400]}")

    print()
    print(f"  {same} of {len(ROUTES)} routes answer with the same shape")
    if unreachable:
        print(f"  {len(unreachable)} unreachable:")
        for line in unreachable[:5]:
            print(f"    {line}")
    if different:
        print(f"  {len(different)} different:")
        for line in different:
            print(f"    {line}")
    print()

    if unreachable:
        print("  The Rust implementation is not answering. Is it running?")
        return 2
    if different:
        print("  The two implementations disagree. The React app would break on one of them.")
        return 1
    print("  The two implementations agree, route for route.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
