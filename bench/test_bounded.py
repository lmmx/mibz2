"""bench/bounded.py: the deadline stops the command's whole process group, a TERM-ignoring
grandchild under a parent that exits on TERM included, and leaves none of it running."""

import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).parent
# a parent that exits on TERM, with a grandchild that ignores it (or doesn't)
STALL = 'echo $$ > "$1"; ({trap} exec sleep 600) & echo $! >> "$1"; wait'


def alive(pid: int) -> bool:
    try:
        stat = Path(f"/proc/{pid}/stat").read_text()
    except OSError:
        return False
    return stat[stat.rfind(")") + 2] not in "ZX"


def bounded(secs, grace, *argv):
    t0 = time.monotonic()
    r = subprocess.run(
        [sys.executable, str(HERE / "bounded.py"), str(secs), str(grace), *argv],
        capture_output=True,
        text=True,
        timeout=60,
    )
    return r, time.monotonic() - t0


def stall(tmp_path, ignore_term):
    pids = tmp_path / "pids"
    trap = 'trap "" TERM;' if ignore_term else ""
    r, took = bounded(1, 1, "bash", "-c", STALL.format(trap=trap), "_", str(pids))
    got = [int(x) for x in pids.read_text().split()]
    try:
        assert len(got) == 2
        assert r.returncode == 124, (r.returncode, r.stderr)
        assert "passed its deadline" in r.stderr
        assert not any(alive(p) for p in got), [p for p in got if alive(p)]
        assert took < 15
    finally:
        for p in got:
            if alive(p):
                os.kill(p, 9)


def test_status_passes_through():
    assert bounded(10, 1, "true")[0].returncode == 0
    assert bounded(10, 1, "sh", "-c", "exit 3")[0].returncode == 3


def test_responsive_grandchild(tmp_path):
    stall(tmp_path, ignore_term=False)


def test_term_ignoring_grandchild(tmp_path):
    # the case `setsid --wait timeout --kill-after` leaves running: the parent exits on TERM,
    # the grandchild ignores it, and still it is killed at the grace's end
    stall(tmp_path, ignore_term=True)


def test_leftovers_after_a_normal_exit(tmp_path):
    # the command exits 0 before its deadline, leaving a TERM-ignoring descendant: stopped too
    pids = tmp_path / "pids"
    script = 'echo $$ > "$1"; (trap "" TERM; exec sleep 600) & echo $! >> "$1"; exit 0'
    r, _ = bounded(30, 1, "bash", "-c", script, "_", str(pids))
    got = [int(x) for x in pids.read_text().split()]
    try:
        assert r.returncode == 0 and "left processes" in r.stderr, r.stderr
        assert not any(alive(p) for p in got)
    finally:
        for p in got:
            if alive(p):
                os.kill(p, 9)
