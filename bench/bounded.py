"""Run a command under a deadline, in a process group of its own; then make a bounded attempt to
stop whatever of that group is left, and refuse, within a bounded time, if it cannot
(bench-par-scan; docs/journal/2026-11-10-parallel-scan.md, sections 9 and 10).

    python3 bench/bounded.py SECS GRACE COMMAND [ARG...]

- The command starts in a new session, so its process group's id is its own pid, and every
  descendant that doesn't start a session of its own is in that group.
- At SECS seconds, the whole group gets TERM. After GRACE more seconds it gets KILL, whether or
  not the command itself has exited by then: a descendant that ignores TERM, under a parent that
  exits on it, is still killed.
- Whenever the command exits, before the deadline or after, any of its group still running is
  stopped the same way (TERM, GRACE, KILL).
- After the KILL it waits, for at most REAP seconds, for no live process to be left in the
  group (a killed process not yet reaped counts as gone).
- A TERM, INT or HUP sent to this supervisor stops the group the same way, then exits 143.
- The cleanup's result is checked before any wait on the command, and every wait is bounded.
  If the group is not empty, or the command cannot be reaped, within REAP seconds of the KILL,
  it names the survivors and returns 125, on every path: the deadline, a normal exit, a signal
  (a failed cleanup is never reported as 124 or 143). It does not promise that every process
  disappears: a process blocked in the kernel may outlive any user-space supervisor. It
  promises a bounded attempt and a bounded refusal, and the caller must not carry on after 125.

Exit: the command's own status if it finished before the deadline (128 + N for a signal N);
124 if the deadline passed; 143 if this supervisor was signalled; 125 if the group could not
be emptied or the command not reaped (whatever else happened).

Out of scope: a descendant that starts a session of its own leaves the group, and with it this
supervision. The diagnostic's children do not do that.
"""

import os
import signal
import subprocess
import sys
import time

TIMED_OUT, NOT_EMPTIED, SIGNALLED = 124, 125, 143
# seconds to wait for the group to empty, and for the command to be reaped, after the KILL
REAP = 10.0


def say(msg: str) -> None:
    print(f"bounded: {msg}", file=sys.stderr, flush=True)


def members(pgid: int) -> list[int]:
    """The live (not zombie) processes in process group `pgid`."""
    found = []
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        try:
            with open(f"/proc/{name}/stat") as f:
                stat = f.read()
        except OSError:
            continue
        # (the command's name is in parentheses and may hold anything: split after the last)
        fields = stat[stat.rfind(")") + 2 :].split()
        state, pgrp = fields[0], int(fields[2])
        if pgrp == pgid and state not in ("Z", "X"):
            found.append(int(name))
    return found


def signal_group(pgid: int, sig: int) -> None:
    try:
        os.killpg(pgid, sig)
    except ProcessLookupError:
        pass


def stop_group(child: subprocess.Popen, pgid: int, grace: float) -> bool:
    """TERM to the group, GRACE seconds, KILL to the group, whatever the command's state; then
    wait for it to be empty. True if it is."""
    signal_group(pgid, signal.SIGTERM)
    end = time.monotonic() + grace
    while time.monotonic() < end:
        child.poll()
        if not members(pgid):
            break
        time.sleep(0.05)
    signal_group(pgid, signal.SIGKILL)
    end = time.monotonic() + REAP
    while time.monotonic() < end:
        child.poll()
        if not members(pgid):
            return True
        time.sleep(0.05)
    return False


def run(secs: float, grace: float, argv: list[str]) -> int:
    child = subprocess.Popen(argv, start_new_session=True)
    pgid = child.pid
    stopping = {"signal": None}

    def on_signal(sig, _frame):
        stopping["signal"] = sig

    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, on_signal)
    end = time.monotonic() + secs
    status = None
    while time.monotonic() < end and stopping["signal"] is None:
        status = child.poll()
        if status is not None:
            break
        time.sleep(0.05)
    if stopping["signal"] is not None:
        say(f"stopped by signal {stopping['signal']}: stopping the group")
        emptied = stop_group(child, pgid, grace)
        return finish(child, pgid, emptied, SIGNALLED)
    timed_out = status is None
    if timed_out:
        say(f"{argv[0]} passed its deadline of {secs:g} s: stopping its process group")
    elif members(pgid):
        say(f"{argv[0]} exited and left processes in its group: stopping them")
    emptied = stop_group(child, pgid, grace) if timed_out or members(pgid) else True
    return finish(child, pgid, emptied, TIMED_OUT if timed_out else None)


def finish(child: subprocess.Popen, pgid: int, emptied: bool, code: int | None) -> int:
    """The exit, after the group was stopped (or needed no stopping): 125 if it isn't empty or
    the command can't be reaped within REAP seconds, never a wait without end; else `code`,
    or the command's own status where `code` is None."""
    if not emptied:
        say(f"process group {pgid} still has live members {members(pgid)} after KILL")
        return NOT_EMPTIED
    try:
        rc = child.wait(timeout=REAP)
    except subprocess.TimeoutExpired:
        say(f"the command (pid {child.pid}) could not be reaped within {REAP:g} s of KILL")
        return NOT_EMPTIED
    if code is not None:
        return code
    return rc if rc >= 0 else 128 - rc


if __name__ == "__main__":
    if len(sys.argv) < 4:
        sys.exit("usage: bounded.py SECS GRACE COMMAND [ARG...]")
    sys.exit(run(float(sys.argv[1]), float(sys.argv[2]), sys.argv[3:]))
