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


def load():
    import importlib.util

    spec = importlib.util.spec_from_file_location("bounded_mod", HERE / "bounded.py")
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def stuck(monkeypatch, mod, leave_alive):
    """A cleanup that fails: `stop_group` reports a group it could not empty, and leaves the
    command running (so a wait on it would never end)."""
    monkeypatch.setattr(mod, "REAP", 1.0, raising=False)
    monkeypatch.setattr(mod, "stop_group", lambda child, pgid, grace: False)
    leave_alive.append(True)


def unreapable(monkeypatch, mod):
    """A command that cannot be reaped: the group empties, but the wait times out."""
    monkeypatch.setattr(mod, "REAP", 1.0, raising=False)

    def wait(self, timeout=None):
        if timeout is None:
            raise AssertionError("an unbounded wait")
        raise subprocess.TimeoutExpired(self.args, timeout)

    monkeypatch.setattr(mod.subprocess.Popen, "wait", wait)


def killpg_after(pid_file):
    for p in (
        [int(x) for x in Path(pid_file).read_text().split()] if Path(pid_file).exists() else []
    ):
        if alive(p):
            os.kill(p, 9)


def test_cleanup_failure_at_the_deadline(tmp_path, monkeypatch):
    mod = load()
    stuck(monkeypatch, mod, [])
    pids = tmp_path / "pids"
    t0 = time.monotonic()
    try:
        rc = mod.run(0.5, 0.2, ["bash", "-c", 'echo $$ > "$1"; exec sleep 600', "_", str(pids)])
        assert rc == 125 and time.monotonic() - t0 < 10
    finally:
        killpg_after(pids)


def test_unreapable_at_the_deadline(tmp_path, monkeypatch):
    mod = load()
    unreapable(monkeypatch, mod)
    t0 = time.monotonic()
    rc = mod.run(0.5, 0.2, ["sleep", "600"])
    assert rc == 125 and time.monotonic() - t0 < 15


def test_cleanup_failure_after_a_signal(tmp_path, monkeypatch):
    import signal
    import threading

    mod = load()
    stuck(monkeypatch, mod, [])
    pids = tmp_path / "pids"
    old = {s: signal.getsignal(s) for s in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)}
    threading.Timer(0.5, lambda: os.kill(os.getpid(), signal.SIGTERM)).start()
    t0 = time.monotonic()
    try:
        rc = mod.run(60, 0.2, ["bash", "-c", 'echo $$ > "$1"; exec sleep 600', "_", str(pids)])
        assert rc == 125 and time.monotonic() - t0 < 10
    finally:
        for s, h in old.items():
            signal.signal(s, h)
        killpg_after(pids)


def test_signal_with_a_clean_stop(tmp_path):
    # the signal path, succeeding: 143, the group stopped
    import signal

    pids = tmp_path / "pids"
    p = subprocess.Popen(
        [
            sys.executable,
            str(HERE / "bounded.py"),
            "60",
            "1",
            "bash",
            "-c",
            'echo $$ > "$1"; (trap "" TERM; exec sleep 600) & echo $! >> "$1"; wait',
            "_",
            str(pids),
        ]
    )
    while not (pids.exists() and len(pids.read_text().split()) == 2):
        time.sleep(0.05)
    p.send_signal(signal.SIGTERM)
    try:
        assert p.wait(timeout=20) == 143
        assert not any(alive(int(x)) for x in pids.read_text().split())
    finally:
        killpg_after(pids)


def parent_gone_survivors_left(monkeypatch, mod):
    """A cleanup that fails after the direct child has exited: `stop_group` kills and reaps
    the leader, then reports surviving descendants (simulated)."""
    monkeypatch.setattr(mod, "REAP", 1.0, raising=False)

    def stop_group(child, pgid, grace):
        child.kill()
        child.wait()
        return False

    monkeypatch.setattr(mod, "stop_group", stop_group)


def test_exited_parent_survivors_at_the_deadline(monkeypatch):
    mod = load()
    parent_gone_survivors_left(monkeypatch, mod)
    t0 = time.monotonic()
    assert mod.run(0.5, 0.2, ["sleep", "600"]) == 125
    assert time.monotonic() - t0 < 10


def test_exited_parent_survivors_after_a_signal(monkeypatch):
    # the signal branch must not return 143 when the group's cleanup failed
    import signal
    import threading

    mod = load()
    parent_gone_survivors_left(monkeypatch, mod)
    old = {s: signal.getsignal(s) for s in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)}
    threading.Timer(0.5, lambda: os.kill(os.getpid(), signal.SIGTERM)).start()
    t0 = time.monotonic()
    try:
        assert mod.run(60, 0.2, ["sleep", "600"]) == 125
        assert time.monotonic() - t0 < 10
    finally:
        for s, h in old.items():
            signal.signal(s, h)
