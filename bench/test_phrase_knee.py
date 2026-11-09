"""bench/phrase_knee.py: the grid must be complete, and the figures are medians."""

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent


def lines(rounds, grid, drop=None):
    out = []
    for i in range(rounds):
        for n in grid:
            for arm, ns in (("production", 2), ("phrase", 4)):
                if (i, arm, n) == drop:
                    continue
                b = 1000 * n
                out.append(
                    json.dumps(
                        {
                            "round": i,
                            "arm": arm,
                            "copies": n,
                            "wall_ns": ns * 1000 * n,
                            "bytes": b,
                            "build_ns": b * ns * n // 2,
                            "walk_ns": b * ns * n // 2,
                        }
                    )
                )
    return "\n".join(out) + "\n"


def run(tmp_path, text):
    p = tmp_path / "runs.jsonl"
    p.write_text(text)
    return subprocess.run(
        [sys.executable, str(HERE / "phrase_knee.py"), str(p), "3", "1,2"],
        capture_output=True,
        text=True,
    )


def test_complete_grid(tmp_path):
    r = run(tmp_path, lines(3, [1, 2]))
    assert r.returncode == 0, r.stderr
    rows = [x for x in r.stdout.splitlines() if x.startswith("| 2 ")]
    # two copies: a thread's cost doubles in both arms, so the rate in all stays at one copy's,
    # and phrase runs at half production's rate
    assert rows == ["| 2 | 4.000 | 500 | 2.00 | 8.000 | 250 | 2.00 | 0.500 |"]


def test_missing_cell_refused(tmp_path):
    r = run(tmp_path, lines(3, [1, 2], drop=(1, "phrase", 2)))
    assert r.returncode == 1 and "missing" in r.stderr


def test_duplicate_refused(tmp_path):
    text = lines(3, [1, 2])
    r = run(tmp_path, text + text.splitlines()[0] + "\n")
    assert r.returncode == 1 and "twice" in r.stderr
