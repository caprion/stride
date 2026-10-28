"""The golden harness must be a no-op on the committed tree.

If this test fails, the reference implementation's observable behaviour moved
and every golden file is stale. Regenerate with ``python3 scripts/golden.py
--generate`` and read the diff before committing it: the diff *is* the change
in behaviour.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HARNESS = REPO / "scripts" / "golden.py"


def test_golden_fixtures_are_stable():
    """`scripts/golden.py --check` must pass, and must not write anything."""
    result = subprocess.run(
        [sys.executable, str(HARNESS), "--check"],
        capture_output=True,
        text=True,
        cwd=str(REPO),
    )
    assert result.returncode == 0, f"{result.stdout}\n{result.stderr}"
    assert "GOLDEN CHECK PASSED" in result.stdout


def test_every_case_has_a_golden_file():
    """A case without a golden file is a case nobody is checking."""
    import json

    manifest = json.loads((REPO / "backend" / "tests" / "golden" / "cases.json").read_text())
    golden = REPO / "backend" / "tests" / "golden" / "golden"
    for case_id in manifest["cases"]:
        assert (golden / f"{case_id}.json").exists(), f"missing golden file for {case_id}"
    for name in ("readiness.json", "plan.json"):
        assert (golden / name).exists(), f"missing golden file {name}"
