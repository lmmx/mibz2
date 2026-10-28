"""The contention fingerprint: does the decoder slow down unusually beside a co-runner?

The question is whether mibz2 slows down more than plain CPU competition explains when
another workload on another physical core competes for what the cores share. It detects
sensitivity. It does not say which shared structure is responsible, and it says
INCONCLUSIVE when the setup can't answer.

**The victim:** `ab child mibz2 1 FILE` (examples/ab.rs; one JSON line: wall, cpu, out,
hash, and `runq_s` where the build reports it), the benchmark's Wikidata corpus at 1
thread, pinned to the first CPU of physical core A. It holds one T-vector (3.6 MB) and
walks it with 32 lanes. With `--victim NAME=EXE` (repeatable; the first is the
reference) other builds of the harness are victims too, interleaved in the same rounds.

**The competitors** (examples/stress.rs), each pinned to the first CPU of another physical
core B, never A's SMT sibling:

- `none`: the victim alone;
- `compute`: a register-only integer loop with no memory traffic, the control for what any
  busy co-runner does (shared power and clock headroom, scheduler noise). It doesn't
  subtract every effect that isn't the caches': it draws less power than the memory
  stressors, for one;
- `cache`: random read-modify-writes over a working set W of about half the effective
  cache C_eff (Bubble-Up's bubble: 8 xorshift streams, unrolled, independent accesses);
- `stream`: a sequential read and write over two 64 MB arrays (STREAM-like), beyond any
  cache, so it moves data to and from memory and evicts the cache as it goes;
- `twin`: a second copy of the victim, decoding the same file in a loop.

With `--sibling`, `compute-sib` and `cache-sib` put the compute loop and the bubble on A's
own SMT sibling. Those arms are separate and labelled: a sibling shares the core itself.

**Calibration, on each machine, recorded:**

- a pointer chase over 256 KB to 64 MB, run twice (the faster kept at each size): ns an
  access at each size. C_eff is where the chase's latency crosses the geometric mean of
  its fastest and its largest size's latency, interpolated on log-log axes; there is
  none (and the cache stressor is invalid) unless the largest size costs at least 3x the
  fastest and at least 50 ns, a cost no cache level has on the machines we know. Stretches
  the chase ran preempted (thread CPU under 0.95 of wall) are measured again. It isn't read
  from sysfs or CPUID, which on cloud VMs report the host socket's cache. It includes the
  TLB's reach and whatever other tenants leave;
- each stressor's solo rate on B (iterations/s, read-modify-writes/s, GB/s);
- the checks: the bubble at W must run closer, on a log scale, to its own in-cache rate
  (at C_eff/8) than to its out-of-cache rate (at 64 MB), and those two must differ by at
  least 1.5x. The stream's footprint must be at least 2 x C_eff, and its rate at 2 x 64 MB
  at least 1.2x below the same loop's rate at a cache-resident size, and at least 1 GB/s.
  A stressor that fails its check is recorded as invalid and kept out of the verdicts.

**The protocol:** randomized interleaved rounds (Abedi and Brecht, ICPE'17): each round runs
every (victim, condition) once, in a shuffled order. A competitor is started, waited for
(its memory touched), warmed `--warm` seconds, then the victim runs, then the competitor
is stopped and `--settle` seconds pass. The competitor's work counter and CPU time
(/proc/PID/schedstat) are sampled just before and after the victim: its CPU over the run,
and its rate against its solo rate (the "caused" side, as iBench reports it).

**The validity screens**, each a check on the measurement, not on performance:

- fewer than 2 physical cores allowed, or an unreadable topology: everything INCONCLUSIVE;
- victim and competitor not on distinct physical cores (the competitor's affinity, read
  from /proc after it started);
- the competitor ran less than 0.9 of a CPU over the victim's run: less competition than
  the condition claims, so a small slowdown would mean little;
- the victim's cpu/wall below 0.95, or its run-queue wait (`runq_s`, where reported)
  above 5% of its wall: it waited to be scheduled, which would read as contention;
- CPU throttling by the cgroup (cpu.stat's nr_throttled, v1 or v2, this cgroup and its
  ancestors) during the run;
- the stressor failed its calibration check.

A screened run drops its round's pairs. A condition with fewer than 3 valid pairs is
INCONCLUSIVE.

**The figures:** each condition's victim slowdown against solo, paired by round (the
geometric mean of per-round wall ratios, 95% t-interval: bench/report.py's `ratio_ci`);
the excess over the compute control, paired: wall(X) / wall(compute) in the same round;
for `twin`, the two copies' decoded throughput together against one copy alone. Across
victims, each one's slowdown against the reference victim's, paired by round.

**The verdicts:** "sensitive to ..., beyond the compute control" when the excess's
interval excludes 1 and its estimate is at least 1.03 (a 3% floor so that a tiny, merely
significant excess isn't called sensitivity), else "not distinguishable from the compute
control at this run's precision" (or a small excess, stated as such). That means not
detected with this many rounds on this machine, not identical. A verdict names the
co-runner, never a hardware cause.

**The accounting:**

- the victim's CPU is ab child's own `cpu` (getrusage of the process, user + system),
  the authority; its wall is the decode's. Until lmmx/mibz2#69 moves it, ab's per-thread
  sampler (every 50 ms) runs on the victim's pinned CPU: a small cost, the same in every
  condition, so the paired ratios cancel it;
- a competitor's CPU over the victim's run is the difference of cumulative counters read
  just before the victim starts and just after it exits (each thread's schedstat in ns,
  else /proc/PID/stat), never per-thread samples summed. A twin's is each copy's own
  getrusage, spread over that copy's life;
- nothing of the orchestrator's runs during a victim's run: it blocks in a wait (as does
  the twin's loop thread) and reads counters only before and after. On a 2-core runner the
  only CPUs left are A's and B's SMT siblings; it is pinned to B's (else any CPU but A and
  B), never A's.

**What it observes, and what it cannot know:**

- a slowdown beside a stressor can come from shared power or turbo headroom, from memory
  bandwidth, or from cache capacity. The stressors differ in all three, so the arm that
  slows the victim doesn't name the mechanism;
- the compute control removes some package effects, not all: it draws less power than a
  memory-bound co-runner, and none of the memory system's;
- an absence of run-queue wait (or a quiet PSI) never rules out memory contention: a
  thread stalled on memory is running as far as the scheduler can tell;
- C_eff is what one thread sees on this VM, at this moment, with this page size. Other
  tenants share the cache on CI's runners, and their share changes between runs;
- 2 physical cores (CI's runners: 4 vCPUs, SMT siblings 0-1 and 2-3) cannot reproduce a
  10-core knee: one co-runner is not eight. The test asks whether the decoder is
  sensitive to a co-runner, the precondition of a knee, not where the knee would be.

    python3 bench/fingerprint.py [--victim NAME=EXE ...] [--rounds 8] [--out F.json] [--md F.md]

`just bench-fingerprint` builds the harness and the stressors and runs it.
"""

import argparse
import atexit
import datetime
import json
import math
import os
import platform
import random
import select
import signal
import statistics
import subprocess
import sys
import threading
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from report import ratio_ci  # noqa: E402

# The validity screens (on the measurement, not on performance; see the docstring)
MIN_CORES = 2
MIN_COMPETITOR_CPU = 0.9
MIN_VICTIM_CPU = 0.95
MAX_VICTIM_RUNQ = 0.05
MIN_PAIRS = 3
# The verdict's floor: an excess over the compute control is called sensitivity from 3%
SENSITIVE = 1.03
# The stressors' checks
CACHE_FRACTION = 0.5  # W = C_eff x this
CACHE_SEPARATION = 1.5  # the bubble out of cache over in cache, ns an access, at least
STREAM_ARRAY = 64 << 20  # bytes each of the stream's two arrays
STREAM_FOOTPRINT = 2.0  # its footprint over C_eff, at least
STREAM_SEPARATION = 1.2  # its cache-resident rate over its large rate, at least
STREAM_MIN_GBPS = 1.0
DRAM_SET = 64 << 20  # the bubble's out-of-cache reference
CHASE_PASSES = 2
# the chase has left the cache only where its largest size costs this much (an LLC hit
# costs 10-40 ns on the machines we know, a memory access 70-150), and 3x its fastest
OUT_OF_CACHE_NS = 50

CONDITIONS = ["none", "compute", "cache", "stream", "twin"]
SIBLING = ["compute-sib", "cache-sib"]
KIND = {
    "compute": "compute",
    "cache": "cache",
    "stream": "stream",
    "compute-sib": "compute",
    "cache-sib": "cache",
}
# what a verdict calls each competitor, and what the victim is then sensitive to
DESCRIBE = {
    "cache": ("the cache stressor", "a co-runner's cache footprint"),
    "stream": ("the streaming stressor", "a co-runner's streaming memory traffic"),
    "twin": ("a second copy of itself", "a copy of itself on another core"),
    "cache-sib": ("the cache stressor on its SMT sibling", "a cache footprint on its own core"),
}


class HardFailure(Exception):
    """A correctness failure (a decode's bytes differ): the run stops, nothing is concluded."""


# ---------------------------------------------------------------------------------------
# Children: every one in its own process group, every one killed on any exit path

_LIVE: set = set()
_LOCK = threading.Lock()
_STOPPING = threading.Event()


def spawn(argv, **kw):
    """A child in a session (and process group) of its own, registered for cleanup."""
    with _LOCK:
        if _STOPPING.is_set():
            raise RuntimeError("fingerprint: stopping")
        p = subprocess.Popen(argv, start_new_session=True, **kw)
        _LIVE.add(p)
    return p


def reap(p, grace=1.0):
    """The child's process group terminated (SIGTERM, then SIGKILL after `grace` s)."""
    for sig, wait in ((signal.SIGTERM, grace), (signal.SIGKILL, 5.0)):
        if p.poll() is not None:
            break
        try:
            os.killpg(p.pid, sig)
        except (ProcessLookupError, PermissionError):
            pass
        try:
            p.wait(wait)
        except subprocess.TimeoutExpired:
            continue
    else:
        p.poll()
    # (its process group may outlive it: a grandchild)
    try:
        os.killpg(p.pid, signal.SIGKILL)
    except (ProcessLookupError, PermissionError):
        pass
    with _LOCK:
        _LIVE.discard(p)


def cleanup():
    _STOPPING.set()
    with _LOCK:
        live = list(_LIVE)
    for p in live:
        reap(p, 0.3)


def _on_signal(signum, _frame):
    cleanup()
    raise SystemExit(128 + signum)


def install_handlers():
    atexit.register(cleanup)
    for s in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(s, _on_signal)


def readline(f, timeout):
    """One line from a child's pipe within `timeout` s, or "" (closed or late)."""
    if not select.select([f], [], [], timeout)[0]:
        return ""
    return f.readline().decode(errors="replace")


# ---------------------------------------------------------------------------------------
# The machine: topology, a process's CPU, the cgroup's throttling


def topology(allowed=None, base="/sys/devices/system/cpu"):
    """The allowed CPUs by physical core, in CPU order: [[cpu, its siblings...], ...],
    or None when the topology can't be read."""
    allowed = sorted(os.sched_getaffinity(0) if allowed is None else allowed)
    cores: dict[str, list[int]] = {}
    for c in allowed:
        try:
            key = Path(f"{base}/cpu{c}/topology/thread_siblings_list").read_text().strip()
        except OSError:
            return None
        cores.setdefault(key, []).append(c)
    return sorted(cores.values(), key=lambda v: v[0])


def core_of(cpu, cores):
    for i, c in enumerate(cores or []):
        if cpu in c:
            return i
    return None


def placement(cores):
    """(A, B, A's sibling or None), or the reason there is none."""
    if cores is None:
        return None, "the CPU topology is unreadable (/sys/devices/system/cpu)"
    if len(cores) < MIN_CORES:
        return (
            None,
            f"{len(cores)} physical core{'s' if len(cores) != 1 else ''} among the allowed CPUs (at least {MIN_CORES} needed)",
        )
    a, b = cores[0][0], cores[1][0]
    sib = cores[0][1] if len(cores[0]) > 1 else None
    return (a, b, sib), None


def parse_cpus(s):
    out = set()
    for part in s.strip().split(","):
        if "-" in part:
            lo, hi = part.split("-")
            out.update(range(int(lo), int(hi) + 1))
        elif part:
            out.add(int(part))
    return out


def affinity(pid):
    try:
        for line in Path(f"/proc/{pid}/status").read_text().splitlines():
            if line.startswith("Cpus_allowed_list:"):
                return sorted(parse_cpus(line.split(":", 1)[1]))
    except OSError:
        pass
    return None


TICK = os.sysconf("SC_CLK_TCK") if hasattr(os, "sysconf") else 100


def proc_cpu(pid):
    """(CPU seconds, run-queue wait seconds or None) of process `pid`, cumulative: the sum
    of its threads' /proc/PID/task/*/schedstat (ns), else the process's /proc/PID/stat
    user + system (ticks, which count exited threads too). Read before and after a run,
    never sampled during it. (A stressor's threads, its loop and its stdin reader, live as
    long as it does, so the sum over live threads misses nothing.)"""
    try:
        run = wait = 0
        for t in os.listdir(f"/proc/{pid}/task"):
            r, w, _ = Path(f"/proc/{pid}/task/{t}/schedstat").read_text().split()
            run, wait = run + int(r), wait + int(w)
        return run / 1e9, wait / 1e9
    except (OSError, ValueError):
        pass
    try:
        f = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
        return (int(f[11]) + int(f[12])) / TICK, None
    except (OSError, ValueError, IndexError):
        return None, None


def cgroup_stat_files(proc="/proc/self/cgroup", root="/sys/fs/cgroup"):
    """The cpu.stat files of this process's cgroup and its ancestors (v2, or v1's cpu)."""
    files = []
    try:
        lines = Path(proc).read_text().splitlines()
    except OSError:
        return files
    for line in lines:
        hier, ctrls, path = (line.split(":", 2) + ["", ""])[:3]
        if hier == "0" and ctrls == "":
            mounts = [root, f"{root}/unified"]
        elif "cpu" in ctrls.split(","):
            mounts = [f"{root}/{ctrls}", f"{root}/cpu", f"{root}/cpu,cpuacct"]
        else:
            continue
        parts = [p for p in path.split("/") if p]
        for m in mounts:
            for k in range(len(parts), -1, -1):
                f = os.path.join(m, *parts[:k], "cpu.stat")
                if os.path.isfile(f) and f not in files:
                    files.append(f)
    return files


def throttled(files):
    """{file: nr_throttled} now (the files that have the field)."""
    out = {}
    for f in files:
        try:
            for line in Path(f).read_text().splitlines():
                k, _, v = line.partition(" ")
                if k == "nr_throttled":
                    out[f] = int(v)
        except (OSError, ValueError):
            pass
    return out


def throttled_delta(a, b):
    return sum(max(0, b[f] - a[f]) for f in a if f in b)


def steal(cpus):
    """{cpu: steal ticks} from /proc/stat."""
    out = {}
    try:
        for line in Path("/proc/stat").read_text().splitlines():
            f = line.split()
            if f and f[0].startswith("cpu") and f[0][3:].isdigit() and int(f[0][3:]) in cpus:
                out[int(f[0][3:])] = int(f[8]) if len(f) > 8 else 0
    except OSError:
        pass
    return out


# ---------------------------------------------------------------------------------------
# The competitors


class Stressor:
    """examples/stress.rs on one CPU: started, waited for, sampled, stopped."""

    def __init__(self, exe, kind, cpu, nbytes=None, timeout=300.0):
        self.argv = ["taskset", "-c", str(cpu), exe, kind, "--timeout", str(timeout)]
        if nbytes:
            self.argv += ["--bytes", str(int(nbytes))]
        self.kind, self.cpu, self.p, self.ready = kind, cpu, None, None

    def start(self):
        self.p = spawn(
            self.argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        line = readline(self.p.stdout, 30.0)
        if not line:
            err = (
                self.p.stderr.read().decode()
                if self.p.poll() is not None
                else "no ready line in 30 s"
            )
            self.stop()
            raise RuntimeError(f"fingerprint: {' '.join(self.argv)}: {err.strip()}")
        self.ready = json.loads(line)
        self.affinity = affinity(self.p.pid)
        return self

    @property
    def pid(self):
        return self.p.pid

    def sample(self):
        self.p.stdin.write(b"\n")
        self.p.stdin.flush()
        line = readline(self.p.stdout, 10.0)
        if not line:
            raise RuntimeError(f"fingerprint: {self.kind} stopped answering")
        d = json.loads(line)
        cpu, wait = proc_cpu(self.p.pid)
        return {"t": d["t"], "n": d["n"], "cpu": cpu, "wait": wait, "wall": time.perf_counter()}

    def stop(self):
        if self.p is None:
            return
        try:
            self.p.stdin.close()  # (its stdin closed: it exits)
            self.p.wait(2.0)
        except (OSError, subprocess.TimeoutExpired):
            pass
        reap(self.p)


def window(s0, s1):
    """A stressor's CPU over the window, and its rate (work a second)."""
    cpu = None
    if s0["cpu"] is not None and s1["cpu"] is not None:
        cpu = (s1["cpu"] - s0["cpu"]) / max(1e-9, s1["wall"] - s0["wall"])
    rate = (s1["n"] - s0["n"]) / max(1e-9, s1["t"] - s0["t"])
    return cpu, rate


def child(exe, cpu, file):
    return ["taskset", "-c", str(cpu), exe, "child", "mibz2", "1", file]


def parse_child(out):
    lines = [x for x in out.strip().splitlines() if x.startswith("{")]
    if not lines:
        return None
    try:
        return json.loads(lines[-1])
    except ValueError:
        return None


class Twin:
    """A second copy of the victim on CPU `cpu`, decoding the file in a loop."""

    def __init__(self, exe, cpu, file, timeout=300.0):
        self.argv, self.timeout = child(exe, cpu, file), timeout
        self.runs, self.stopped, self.error = [], threading.Event(), None
        self.cpu, self.affinity, self.thread, self.current = cpu, None, None, None

    def _loop(self):
        while not self.stopped.is_set():
            t0 = time.perf_counter()
            try:
                p = spawn(self.argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            except RuntimeError:
                return
            self.current = p
            try:
                out, _ = p.communicate(timeout=self.timeout)
            except subprocess.TimeoutExpired:
                self.error = "the twin's decode passed its timeout"
                reap(p)
                return
            finally:
                reap(p)
            rec = parse_child(out.decode(errors="replace"))
            if rec is None or rec.get("err"):
                self.error = f"the twin's decode failed: {rec and rec.get('err')}"
                return
            self.runs.append((t0, time.perf_counter(), rec))

    def start(self):
        self.thread = threading.Thread(target=self._loop, daemon=True)
        self.thread.start()
        return self

    def sample(self):
        # (its placement read from a copy that has been running a while: taskset has exec'd)
        p = self.current
        if p is not None and p.poll() is None:
            self.affinity = affinity(p.pid) or self.affinity
        return {"wall": time.perf_counter()}

    def stop(self):
        self.stopped.set()
        if self.thread:
            self.thread.join(self.timeout + 10)

    def window(self, t0, t1):
        """Over [t0, t1]: the copies' CPU (each spread over its life), their output a second,
        and the hashes of the decodes that overlapped it."""
        cpu = out = 0.0
        hashes = []
        for a, b, rec in self.runs:
            ov = max(0.0, min(b, t1) - max(a, t0))
            if ov <= 0:
                continue
            frac = ov / max(1e-9, b - a)
            cpu += rec["cpu"] * frac
            out += rec["out"] * frac
            hashes.append(rec["hash"])
        span = max(1e-9, t1 - t0)
        return cpu / span, out / span, hashes


# ---------------------------------------------------------------------------------------
# Calibration


def chase(stress, cpu, passes=CHASE_PASSES, ms=10, most=6):
    """The pointer chase's ns an access at each size: the fastest of at least `passes`
    sweeps, counting only stretches it ran unpreempted (thread CPU >= MIN_VICTIM_CPU of its
    wall), sweeping again (at most `most` sweeps) until every size has one. A size with
    none is marked `preempted`."""
    best: dict[int, float] = {}
    sizes: set[int] = set()
    for k in range(most):
        p = spawn(
            ["taskset", "-c", str(cpu), stress, "chase", "--ms", str(ms)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        try:
            out, err = p.communicate(timeout=60)
        finally:
            reap(p)
        if p.returncode:
            raise RuntimeError(f"fingerprint: stress chase failed: {err.decode().strip()}")
        for line in out.decode().splitlines():
            d = json.loads(line)
            sizes.add(d["bytes"])
            if d.get("cpu", 1.0) >= MIN_VICTIM_CPU:
                best[d["bytes"]] = min(best.get(d["bytes"], math.inf), d["ns"])
        if k + 1 >= passes and set(best) == sizes:
            break
    return [
        {"bytes": b, "ns": best[b]} if b in best else {"bytes": b, "ns": None, "preempted": True}
        for b in sorted(sizes)
    ]


def c_eff(sweep):
    """The effective cache from the chase: where its latency crosses the geometric mean of
    its fastest and its largest size's (the out-of-cache plateau), interpolated on log-log
    axes, at the last crossing (a slow outlier inside the cache region doesn't move it).
    (bytes or None, the detail)."""
    lost = [d["bytes"] for d in sweep if d.get("ns") is None]
    if lost:
        return None, {
            "reason": f"the chase was preempted at every attempt at {', '.join(mb(b) for b in lost)}"
        }
    pts = sorted((d["bytes"], d["ns"]) for d in sweep)
    if len(pts) < 3:
        return None, {"reason": "fewer than 3 sizes in the sweep"}
    lo = min(ns for _, ns in pts)
    hi = pts[-1][1]
    detail = {"fastest_ns": lo, "largest_ns": hi, "largest_bytes": pts[-1][0]}
    if hi < 3 * lo or hi < OUT_OF_CACHE_NS:
        detail["reason"] = (
            f"no knee: the largest size ({pts[-1][0] >> 20} MB) takes {hi:.1f} ns an access, "
            f"{hi / lo:.1f}x the fastest (a knee needs 3x, and at least {OUT_OF_CACHE_NS} ns)"
        )
        return None, detail
    thr = math.sqrt(lo * hi)
    detail["threshold_ns"] = round(thr, 3)
    below = [i for i, (_, ns) in enumerate(pts) if ns <= thr]
    i = below[-1]
    if i == len(pts) - 1:
        detail["reason"] = "the sweep never left the cache"
        return None, detail
    (s0, l0), (s1, l1) = pts[i], pts[i + 1]
    f = (math.log(thr) - math.log(l0)) / (math.log(l1) - math.log(l0))
    c = math.exp(math.log(s0) + f * (math.log(s1) - math.log(s0)))
    detail["between"] = [s0, s1]
    return int(c), detail


def validate_cache(ns_in, ns_w, ns_out):
    """The bubble at W serves from cache: closer (log scale) to its in-cache ns an access
    than to its out-of-cache one, and those two well apart. (valid, reason)."""
    if not all(x and x > 0 for x in (ns_in, ns_w, ns_out)):
        return False, "the cache stressor's calibration has no rate"
    if ns_out < CACHE_SEPARATION * ns_in:
        return False, (
            f"the bubble's out-of-cache rate ({ns_out:.2f} ns an access at 64 MB) is within "
            f"{CACHE_SEPARATION}x of its in-cache rate ({ns_in:.2f} ns): cache and memory can't be told apart"
        )
    mid = math.sqrt(ns_in * ns_out)
    if ns_w > mid:
        return False, (
            f"at W the bubble runs at {ns_w:.2f} ns an access, nearer its out-of-cache "
            f"{ns_out:.2f} than its in-cache {ns_in:.2f}: it would be a memory streamer"
        )
    return (
        True,
        f"{ns_w:.2f} ns an access at W, against {ns_in:.2f} in cache and {ns_out:.2f} out of it",
    )


def validate_stream(gbps_small, gbps_big, footprint, ceff):
    """The stream leaves the cache: its footprint well past C_eff, and its rate there below
    the same loop's cache-resident rate. (valid, reason)."""
    if not ceff:
        return False, "no C_eff to compare the footprint with"
    if footprint < STREAM_FOOTPRINT * ceff:
        return (
            False,
            f"its footprint ({footprint >> 20} MB) is under {STREAM_FOOTPRINT}x C_eff ({ceff / 2**20:.1f} MB)",
        )
    if not gbps_big or gbps_big < STREAM_MIN_GBPS:
        return False, f"it reached only {gbps_big or 0:.2f} GB/s"
    if gbps_small < STREAM_SEPARATION * gbps_big:
        return False, (
            f"{gbps_big:.1f} GB/s over {footprint >> 20} MB is within {STREAM_SEPARATION}x of "
            f"its cache-resident {gbps_small:.1f} GB/s: not shown to leave the cache"
        )
    return (
        True,
        f"{gbps_big:.1f} GB/s over {footprint >> 20} MB, against {gbps_small:.1f} GB/s cache-resident",
    )


def solo_rate(stress, kind, cpu, nbytes, seconds=0.4, warm=0.2, tries=3):
    """A stressor's rate alone on `cpu`, measured again (at most `tries` times) while it got
    less than MIN_COMPETITOR_CPU of the CPU."""
    for _ in range(tries):
        s = Stressor(stress, kind, cpu, nbytes, timeout=30).start()
        try:
            time.sleep(warm)
            a = s.sample()
            time.sleep(seconds)
            b = s.sample()
        finally:
            s.stop()
        cpu_frac, rate = window(a, b)
        if cpu_frac is None or cpu_frac >= MIN_COMPETITOR_CPU:
            break
    return {
        "bytes": s.ready.get("bytes"),
        "unit": s.ready.get("unit"),
        "rate": rate,
        "cpu": cpu_frac,
    }


def preempted(*runs):
    low = [r for r in runs if r.get("cpu") is not None and r["cpu"] < MIN_COMPETITOR_CPU]
    return (
        f"its solo calibration got only {min(r['cpu'] for r in low):.2f} of a CPU" if low else None
    )


def calibrate(stress, a, b):
    sweep = chase(stress, a)
    ceff, detail = c_eff(sweep)
    w = int(ceff * CACHE_FRACTION) if ceff else 8 << 20
    ref = max(128 << 10, (ceff or 8 << 20) // 8)
    cal = {"chase": sweep, "c_eff_bytes": ceff, "c_eff": detail, "cache_bytes": w}
    st = {}
    st["compute"] = solo_rate(stress, "compute", b, None)
    bad = preempted(st["compute"])
    st["compute"].update(
        valid=not bad, check=bad or "the control: no memory traffic, nothing to check"
    )
    cache = {
        k: solo_rate(stress, "cache", b, n) for k, n in (("in", ref), ("w", w), ("out", DRAM_SET))
    }
    ns = {k: 1e9 / v["rate"] if v["rate"] else None for k, v in cache.items()}
    ok, why = validate_cache(ns["in"], ns["w"], ns["out"])
    if not ceff:
        ok, why = False, f"no C_eff to size W from ({detail.get('reason')}); W fell back to 8 MB"
    if preempted(*cache.values()):
        ok, why = False, preempted(*cache.values())
    st["cache"] = {
        **cache["w"],
        "valid": ok,
        "check": why,
        "ns_in": ns["in"],
        "ns_w": ns["w"],
        "ns_out": ns["out"],
        "in_bytes": ref,
        "out_bytes": DRAM_SET,
    }
    small = solo_rate(stress, "stream", b, max(64 << 10, ref))
    big = solo_rate(stress, "stream", b, STREAM_ARRAY)
    ok, why = validate_stream(small["rate"] / 1e9, big["rate"] / 1e9, 2 * STREAM_ARRAY, ceff)
    if preempted(small, big):
        ok, why = False, preempted(small, big)
    st["stream"] = {
        **big,
        "valid": ok,
        "check": why,
        "gbps": big["rate"] / 1e9,
        "gbps_resident": small["rate"] / 1e9,
        "resident_bytes": 2 * small["bytes"],
    }
    st["twin"] = {"valid": True, "check": "a copy of the victim: its solo rate is the victim's"}
    for k in ("compute", "cache"):
        st[f"{k}-sib"] = dict(st[k])
    cal["stressors"] = st
    return cal


# ---------------------------------------------------------------------------------------
# A run, and its screens


def screen(rec, core_a, cores):
    """The validity screens on one run's record: [reasons it is invalid]."""
    why = []
    wall, cpu = rec.get("wall") or 0, rec.get("cpu") or 0
    if wall <= 0:
        return ["the victim has no wall time"]
    if cpu / wall < MIN_VICTIM_CPU:
        why.append(
            f"the victim waited to be scheduled (on a CPU {cpu / wall:.2f} of its wall, < {MIN_VICTIM_CPU})"
        )
    if rec.get("runq_s") is not None and rec["runq_s"] / wall > MAX_VICTIM_RUNQ:
        why.append(
            f"the victim waited on a run queue ({rec['runq_s'] / wall:.0%} of its wall, > {MAX_VICTIM_RUNQ:.0%})"
        )
    if rec.get("throttled"):
        why.append(f"the cgroup throttled the CPU ({rec['throttled']} time(s) during the run)")
    comp = rec.get("competitor")
    if comp:
        if comp.get("cpu") is None:
            why.append("the competitor's CPU time is unreadable")
        elif comp["cpu"] < MIN_COMPETITOR_CPU:
            why.append(
                f"the competitor ran less than {MIN_COMPETITOR_CPU} of a CPU ({comp['cpu']:.2f})"
            )
        aff = comp.get("affinity") or []
        want_sib = rec["condition"].endswith("-sib")
        ca = [core_of(c, cores) for c in aff]
        if not aff:
            why.append("the competitor's placement is unreadable")
        elif want_sib:
            if any(c != core_a for c in ca):
                why.append("the sibling arm's competitor isn't on the victim's core")
        elif core_a in ca:
            why.append("the competitor shares the victim's physical core")
    return why


def run_one(victim, exe, cond, ctx):
    a, b, sib = ctx["placement"]
    cpu = sib if cond.endswith("-sib") else b
    comp = None
    if cond == "twin":
        comp = Twin(exe, cpu, ctx["file"], ctx["timeout"])
    elif cond != "none":
        nbytes = {"cache": ctx["cal"]["cache_bytes"], "stream": STREAM_ARRAY}.get(KIND[cond])
        comp = Stressor(ctx["stress"], KIND[cond], cpu, nbytes, timeout=ctx["timeout"])
    rec = {"victim": victim, "condition": cond}
    try:
        if comp:
            comp.start()
            time.sleep(ctx["warm"])
            s0 = comp.sample()
        th0, st0 = throttled(ctx["cgroup"]), steal({a, cpu})
        t0 = time.perf_counter()
        p = spawn(child(exe, a, ctx["file"]), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            out, err = p.communicate(timeout=ctx["timeout"])
        finally:
            reap(p)
        t1 = time.perf_counter()
        if comp:
            s1 = comp.sample()
        th1, st1 = throttled(ctx["cgroup"]), steal({a, cpu})
    finally:
        if comp:
            comp.stop()
    r = parse_child(out.decode(errors="replace"))
    if r is None or r.get("err"):
        raise HardFailure(
            f"{victim}: the victim's decode failed ({cond}): {r and r.get('err') or err.decode().strip()}"
        )
    rec.update({k: r.get(k) for k in ("wall", "cpu", "out", "hash", "rss_kb", "runq_s")})
    rec["life"] = t1 - t0
    rec["throttled"] = throttled_delta(th0, th1)
    rec["steal_ticks"] = {c: st1.get(c, 0) - st0.get(c, 0) for c in st0}
    if comp:
        if cond == "twin":
            if comp.error:
                raise HardFailure(f"{victim}: {comp.error}")
            c, rate, hashes = comp.window(t0, t1)
            if any(h != rec["hash"] for h in hashes):
                raise HardFailure(f"{victim}: the twin's bytes differ from the victim's")
            rec["competitor"] = {"cpu": c, "rate": rate, "unit": "bytes", "affinity": comp.affinity}
        else:
            c, rate = window(s0, s1)
            rec["competitor"] = {
                "cpu": c,
                "rate": rate,
                "unit": comp.ready.get("unit"),
                "affinity": comp.affinity,
                "wait": (s1["wait"] - s0["wait"])
                if s0["wait"] is not None and s1["wait"] is not None
                else None,
            }
    return rec


# ---------------------------------------------------------------------------------------
# The figures and the verdicts


def fmt(ci):
    return f"{ci[0]:.2f}x ({ci[1]:.2f}-{ci[2]:.2f})"


def pairs(by, num, den):
    """Per-round ratios of wall(num) / wall(den) over the rounds where both are valid."""
    out = []
    for rnd, runs in by.items():
        x, y = runs.get(num), runs.get(den)
        if x and y and x["valid"] and y["valid"]:
            out.append(x["wall"] / y["wall"])
    return out


def verdict(cond, slow, excess, compute, n, total, invalid=None, why=None):
    """One condition's verdict line. Names the co-runner and the figures, never a cause."""
    if invalid:
        return f"INCONCLUSIVE: {invalid}"
    if n < MIN_PAIRS or slow is None:
        more = f" (most often: {why})" if why else ""
        return f"INCONCLUSIVE: only {n} of {total} rounds passed the validity screens{more}"
    if cond in ("compute", "compute-sib"):
        where = (
            "the compute loop on its SMT sibling"
            if cond == "compute-sib"
            else "the compute control"
        )
        if slow[1] > 1:
            return f"slows {fmt(slow)} beside {where}: a busy co-runner alone slows it; the other arms are read against this"
        return f"{fmt(slow)} beside {where}: no slowdown detected at this run's precision"
    name, what = DESCRIBE[cond]
    against = "compute-sib" if cond.endswith("-sib") else "compute"
    head = f"{'slows' if slow[0] >= 1 else 'wall'} {fmt(slow)} beside {name}"
    if compute is not None:
        head += f", {compute[0]:.2f}x beside {'the sibling compute loop' if against == 'compute-sib' else 'compute'}"
    if excess is None:
        return f"{head}: INCONCLUSIVE against the compute control (too few valid pairs)"
    if excess[1] > 1 and excess[0] >= SENSITIVE:
        return f"{head}: sensitive to {what}, beyond the compute control ({fmt(excess)} over it)"
    if excess[1] > 1:
        return f"{head}: a small excess over the compute control, {fmt(excess)}, under the {SENSITIVE:.2f}x this calls sensitive"
    if excess[2] < 1:
        return f"{head}: less slowed than beside the compute control ({fmt(excess)})"
    return (
        f"{head}: not distinguishable from the compute control at this run's precision "
        f"({fmt(excess)}; not detected, which isn't the same as identical)"
    )


def analyse(records, victims, conditions, cal, global_why=None):
    """Per victim and condition: valid pairs, slowdown, excess, the caused side, a verdict;
    for twin the aggregate throughput; across victims the slowdown against the reference."""
    st = cal.get("stressors", {}) if cal else {}
    by = {v: {} for v in victims}
    for r in records:
        by[r["victim"]].setdefault(r["round"], {})[r["condition"]] = r
    total = len({r["round"] for r in records}) if records else 0
    res = {}
    for v in victims:
        rounds = by[v]
        solo = [x["none"] for x in rounds.values() if x.get("none") and x["none"]["valid"]]
        solo_wall = statistics.median([x["wall"] for x in solo]) if solo else None
        solo_life = statistics.median([x["life"] for x in solo]) if solo else None
        out_b = solo[0]["out"] if solo else None
        out = {
            "solo": {
                "n": len(solo),
                "wall": solo_wall,
                "mbps": out_b / solo_wall / 1e6 if solo else None,
            },
            "conditions": {},
        }
        comp_ci = ratio_ci(pairs(rounds, "compute", "none")) if "compute" in conditions else None
        sib_ci = (
            ratio_ci(pairs(rounds, "compute-sib", "none")) if "compute-sib" in conditions else None
        )
        for c in conditions:
            if c == "none":
                continue
            p = pairs(rounds, c, "none")
            slow = ratio_ci(p)
            against = "compute-sib" if c.endswith("-sib") else "compute"
            ex = (
                ratio_ci(pairs(rounds, c, against))
                if c not in ("compute", "compute-sib") and against in conditions
                else None
            )
            reasons = [
                w for x in rounds.values() for k in (c, "none") if x.get(k) for w in x[k]["why"]
            ]
            why = statistics.mode([w.split(" (")[0] for w in reasons]) if reasons else None
            invalid = global_why
            if not invalid and c in st and not st[c].get("valid", True):
                invalid = (
                    f"the {KIND.get(c, c)} stressor failed its calibration check: {st[c]['check']}"
                )
            if not invalid and c.endswith("-sib") and cal and cal.get("sibling") is None:
                invalid = "the victim's core has no SMT sibling among the allowed CPUs"
            runs = [x[c] for x in rounds.values() if x.get(c) and x[c]["valid"]]
            caused = None
            if c == "twin" and solo_life and out_b:
                caused = [x["competitor"]["rate"] / (out_b / solo_life) for x in runs]
            elif c in st and st[c].get("rate"):
                caused = [x["competitor"]["rate"] / st[c]["rate"] for x in runs]
            d = {
                "pairs": len(p),
                "slowdown": slow,
                "excess": ex,
                "competitor_cpu": statistics.median([x["competitor"]["cpu"] for x in runs])
                if runs
                else None,
                "caused": statistics.median(caused) if caused else None,
                "screened": len(rounds) - len(p),
                "most_common_screen": why,
            }
            if c == "twin":
                agg = []
                for x in rounds.values():
                    t, s = x.get("twin"), x.get("none")
                    if t and s and t["valid"] and s["valid"]:
                        agg.append(
                            (t["out"] / t["wall"] + t["competitor"]["rate"])
                            / (s["out"] / s["wall"])
                        )
                d["aggregate"] = ratio_ci(agg)
                d["aggregate_mbps"] = (
                    statistics.median(
                        [x["out"] / x["wall"] / 1e6 + x["competitor"]["rate"] / 1e6 for x in runs]
                    )
                    if runs
                    else None
                )
            d["verdict"] = verdict(
                c,
                slow,
                ex,
                sib_ci if against == "compute-sib" else comp_ci,
                len(p),
                total,
                invalid,
                why,
            )
            if c == "twin" and d.get("aggregate") and not invalid and len(p) >= MIN_PAIRS:
                d["verdict"] += f"; two copies decode {fmt(d['aggregate'])} one copy's throughput"
            out["conditions"][c] = d
        res[v] = out
    # across victims: each one's slowdown against the reference's, paired by round
    ref = victims[0]
    across = {}
    for v in victims[1:]:
        row = {}
        common = set(by[v]) & set(by[ref])
        r = []
        for rnd in common:
            x, y = by[v][rnd].get("none"), by[ref][rnd].get("none")
            if x and y and x["valid"] and y["valid"]:
                r.append(y["wall"] / x["wall"])
        row["solo_throughput"] = ratio_ci(r)
        for c in conditions:
            if c == "none":
                continue
            r = []
            for rnd in common:
                a, b = by[v][rnd], by[ref][rnd]
                if all(k in d and d[k]["valid"] for d in (a, b) for k in (c, "none")):
                    r.append(
                        (a[c]["wall"] / a["none"]["wall"]) / (b[c]["wall"] / b["none"]["wall"])
                    )
            row[c] = ratio_ci(r)
        across[v] = row
    return res, across


# ---------------------------------------------------------------------------------------
# The report


def mb(b):
    return f"{b / 2**20:.1f} MB" if b else "-"


def markdown(res):
    m, cal = res["meta"], res.get("calibration") or {}
    L = ["## Contention fingerprint", ""]
    L.append(
        f"{m['utc']}, {m.get('commit', '')[:10]}, {m['machine'].get('model', '?')}; "
        f"victim on CPU {m.get('victim_cpu')}, competitors on CPU {m.get('competitor_cpu')}"
        f"{', sibling ' + str(m.get('sibling_cpu')) if m.get('sibling_cpu') is not None else ''}; "
        f"{m['rounds']} rounds, warm {m['warm']} s, settle {m['settle']} s; {m.get('seconds', 0):.0f} s in all."
    )
    L.append("")
    if res.get("failure"):
        L += [f"**FAILED:** {res['failure']}", ""]
    if res.get("inconclusive"):
        L += [f"**INCONCLUSIVE:** {res['inconclusive']}", ""]
    if res.get("verdicts"):
        L += ["### Verdicts", ""]
        for v, lines in res["verdicts"].items():
            for c, line in lines.items():
                L.append(f"- **{v}**, {c}: {line}")
        L.append("")
    if cal.get("chase"):
        L += ["### Calibration", ""]
        L.append(
            f"C_eff (the pointer chase's knee): **{mb(cal.get('c_eff_bytes'))}**"
            + (
                f" (between {mb(cal['c_eff']['between'][0])} and {mb(cal['c_eff']['between'][1])}; threshold {cal['c_eff']['threshold_ns']:.1f} ns)"
                if cal.get("c_eff_bytes")
                else f" ({cal['c_eff'].get('reason')})"
            )
            + f"; the cache stressor's W: {mb(cal.get('cache_bytes'))}."
        )
        L.append("")
        L.append(
            "| size | "
            + " | ".join(
                mb(d["bytes"]).replace(".0 MB", " MB")
                if d["bytes"] >= 1 << 20
                else f"{d['bytes'] >> 10} KB"
                for d in cal["chase"]
            )
            + " |"
        )
        L.append("|---|" + "--:|" * len(cal["chase"]))
        L.append("| ns an access | " + " | ".join(f"{d['ns']:.1f}" for d in cal["chase"]) + " |")
        L += ["", "| stressor | solo rate | CPU | valid | check |", "|---|--:|--:|---|---|"]
        for k in ("compute", "cache", "stream"):
            s = cal["stressors"].get(k, {})
            rate = {
                "iterations": lambda r: f"{r / 1e6:.0f} M iterations/s",
                "rmw": lambda r: f"{r / 1e6:.0f} M RMW/s ({1e9 / r:.2f} ns)",
                "bytes": lambda r: f"{r / 1e9:.1f} GB/s",
            }.get(s.get("unit"), lambda r: f"{r:.3g}/s")
            L.append(
                f"| {k} | {rate(s['rate']) if s.get('rate') else '-'} | {s.get('cpu') or 0:.2f} | {'yes' if s.get('valid') else '**no**'} | {s.get('check', '')} |"
            )
        L.append("")
    for v, out in res.get("victims", {}).items():
        solo = out["solo"]
        L += [
            f"### {v}",
            "",
            f"Solo: {solo['n']} valid runs, median wall {solo['wall'] or 0:.3f} s, {solo['mbps'] or 0:.0f} MB/s.",
            "",
        ]
        L += [
            "| condition | pairs | victim slowdown [95% CI] | over compute [95% CI] | competitor CPU | competitor rate / solo | screened (most often) |",
            "|---|--:|--:|--:|--:|--:|---|",
        ]
        for c, d in out["conditions"].items():
            ccpu = "-" if d["competitor_cpu"] is None else f"{d['competitor_cpu']:.2f}"
            caused = "-" if d["caused"] is None else f"{d['caused']:.2f}"
            scr = f"{d['screened']}" + (
                f" ({d['most_common_screen']})" if d["most_common_screen"] else ""
            )
            slow = fmt(d["slowdown"]) if d["slowdown"] else "-"
            ex = fmt(d["excess"]) if d["excess"] else "-"
            L.append(f"| {c} | {d['pairs']} | {slow} | {ex} | {ccpu} | {caused} | {scr} |")
        tw = out["conditions"].get("twin")
        if tw and tw.get("aggregate"):
            L += [
                "",
                f"Two copies together: {tw['aggregate_mbps']:.0f} MB/s, {fmt(tw['aggregate'])} one copy alone.",
            ]
        L.append("")
    if res.get("across"):
        conds = [c for c in res["meta"]["conditions"] if c != "none"]
        ref = res["meta"]["victims"][0]
        L += [
            f"### Across victims (against {ref}, paired by round)",
            "",
            "Solo throughput, and each condition's slowdown, as ratios to the reference's; an interval excluding 1 tells the builds apart.",
            "",
        ]
        L += [
            "| victim | solo throughput | " + " | ".join(f"{c} slowdown" for c in conds) + " |",
            "|---|--:|" + "--:|" * len(conds),
        ]
        for v, row in res["across"].items():
            L.append(
                f"| {v} | {fmt(row['solo_throughput']) if row['solo_throughput'] else '-'} | "
                + " | ".join(
                    fmt(row[c]) + (" *" if row[c] and (row[c][1] > 1 or row[c][2] < 1) else "")
                    if row.get(c)
                    else "-"
                    for c in conds
                )
                + " |"
            )
        L += ["", "(* the interval excludes 1)", ""]
    L += [
        "<details><summary>What this can and cannot tell</summary>",
        "",
        "- A slowdown beside a stressor can come from shared power or turbo headroom, memory bandwidth or cache capacity; the arm that slows the victim doesn't name the mechanism.",
        "- The compute control removes some package effects, not all.",
        "- No run-queue wait (or quiet PSI) never rules out memory contention.",
        "- Two physical cores can't reproduce a 10-core knee: this asks whether the decoder is sensitive to one co-runner.",
        f"- Screens: competitor CPU >= {MIN_COMPETITOR_CPU}, victim cpu/wall >= {MIN_VICTIM_CPU}, run-queue wait <= {MAX_VICTIM_RUNQ:.0%}, no cgroup throttling, distinct physical cores, stressors that passed calibration; at least {MIN_PAIRS} valid pairs.",
        "",
        "</details>",
        "",
    ]
    return "\n".join(L)


# ---------------------------------------------------------------------------------------
# The orchestration


def machine():
    model = "?"
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                model = line.split(":", 1)[1].strip()
                break
    except OSError:
        pass
    thp = None
    try:
        thp = Path("/sys/kernel/mm/transparent_hugepage/enabled").read_text().strip()
    except OSError:
        pass
    return {"model": model, "kernel": platform.release(), "cpus": os.cpu_count(), "thp": thp}


def git_head():
    r = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=HERE)
    return r.stdout.strip()


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--victim",
        action="append",
        default=[],
        help="NAME=EXE, a build of examples/ab.rs (repeatable; the first is the reference)",
    )
    ap.add_argument("--stress", default="target/release/examples/stress")
    ap.add_argument("--file", default="corpora/wikidata.bz2")
    ap.add_argument("--rounds", type=int, default=8)
    ap.add_argument(
        "--warm", type=float, default=0.3, help="seconds a competitor runs before the victim starts"
    )
    ap.add_argument("--settle", type=float, default=0.2, help="idle seconds after each run")
    ap.add_argument("--conditions", default=",".join(CONDITIONS))
    ap.add_argument(
        "--sibling", action="store_true", help="add the SMT-sibling arms (compute-sib, cache-sib)"
    )
    ap.add_argument(
        "--seed",
        type=int,
        default=None,
        help="the rounds' shuffle (default: from the clock, recorded)",
    )
    ap.add_argument(
        "--timeout",
        type=float,
        default=300.0,
        help="a victim's or competitor's hard limit, seconds",
    )
    ap.add_argument("--note", default="")
    ap.add_argument("--out", default=None, help="the JSON (default fingerprint-<UTC>.json)")
    ap.add_argument("--md", default=None, help="the Markdown (default fingerprint-<UTC>.md)")
    a = ap.parse_args(argv)
    install_handlers()
    t_start = time.perf_counter()
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out_json, out_md = a.out or f"fingerprint-{utc}.json", a.md or f"fingerprint-{utc}.md"
    victims = {}
    for v in a.victim or ["this build=target/release/examples/ab"]:
        name, _, exe = v.partition("=")
        if not exe:
            sys.exit(f"fingerprint: --victim {v}: NAME=EXE")
        victims[name] = os.path.abspath(exe)
    conditions = [c for c in a.conditions.split(",") if c]
    unknown = [c for c in conditions if c not in CONDITIONS]
    if unknown:
        sys.exit(f"fingerprint: unknown conditions {unknown}; any of {CONDITIONS}")
    if "none" not in conditions:
        conditions.insert(0, "none")
    if a.sibling:
        conditions += SIBLING
    for p in [*victims.values(), a.stress, a.file]:
        if not os.path.exists(p):
            sys.exit(f"fingerprint: no {p}; `just bench-fingerprint` builds it")
    seed = a.seed if a.seed is not None else int(time.time())
    cores = topology()
    place, why = placement(cores)
    res = {
        "meta": {
            "utc": utc,
            "note": a.note,
            "commit": git_head(),
            "machine": {**machine(), "allowed": sorted(os.sched_getaffinity(0)), "cores": cores},
            "victims": list(victims),
            "victim_exes": victims,
            "conditions": conditions,
            "rounds": a.rounds,
            "warm": a.warm,
            "settle": a.settle,
            "seed": seed,
            "file": a.file,
            "screens": {
                "min_competitor_cpu": MIN_COMPETITOR_CPU,
                "min_victim_cpu": MIN_VICTIM_CPU,
                "max_victim_runq": MAX_VICTIM_RUNQ,
                "min_pairs": MIN_PAIRS,
                "sensitive": SENSITIVE,
            },
        },
        "records": [],
    }

    def write():
        res["meta"]["seconds"] = time.perf_counter() - t_start
        with open(out_json, "w") as f:
            json.dump(res, f, indent=1)
        with open(out_md, "w") as f:
            f.write(markdown(res))
        print(f"fingerprint: wrote {out_json} and {out_md}", flush=True)

    if not place:
        res["inconclusive"] = why
        write()
        print(f"fingerprint: INCONCLUSIVE: {why}")
        return 0
    A, B, sib = place
    res["meta"].update(victim_cpu=A, competitor_cpu=B, sibling_cpu=sib if a.sibling else None)
    # this process off the victim's and the competitor's CPUs (and the victim's sibling)
    rest = set(os.sched_getaffinity(0)) - {A, B}
    if rest:
        os.sched_setaffinity(0, (rest - {sib}) or rest)
    ctx = {
        "placement": place,
        "file": a.file,
        "stress": os.path.abspath(a.stress),
        "warm": a.warm,
        "timeout": a.timeout,
        "cgroup": cgroup_stat_files(),
    }
    res["meta"]["cgroup_cpu_stat"] = ctx["cgroup"] or "unavailable"
    try:
        print(f"fingerprint: calibrating (victim CPU {A}, competitor CPU {B})", flush=True)
        cal = calibrate(ctx["stress"], A, B)
        cal["sibling"] = sib
        ctx["cal"] = res["calibration"] = cal
        print(
            f"fingerprint: C_eff {mb(cal['c_eff_bytes'])}; "
            + "; ".join(
                f"{k} {'valid' if s['valid'] else 'INVALID'}"
                for k, s in cal["stressors"].items()
                if not k.endswith("-sib")
            ),
            flush=True,
        )
        rng = random.Random(seed)
        # (the sibling arms only where the victim's core has a sibling; else INCONCLUSIVE)
        runnable = [c for c in conditions if sib is not None or not c.endswith("-sib")]
        hashes = set()
        for rnd in range(a.rounds):
            order = [(v, c) for v in victims for c in runnable]
            rng.shuffle(order)
            for v, c in order:
                rec = run_one(v, victims[v], c, ctx)
                rec["round"] = rnd
                rec["why"] = screen(rec, core_of(A, cores), cores)
                rec["valid"] = not rec["why"]
                hashes.add(rec["hash"])
                if len(hashes) > 1:
                    raise HardFailure(
                        f"{v} ({c}, round {rnd}): its output's hash {rec['hash']} differs from an earlier run's"
                    )
                res["records"].append(rec)
                comp = rec.get("competitor") or {}
                print(
                    f"r{rnd} {v:<12} {c:<11} wall {rec['wall']:.3f} cpu {rec['cpu']:.3f}"
                    + (f" competitor cpu {comp['cpu']:.2f}" if comp.get("cpu") is not None else "")
                    + ("" if rec["valid"] else f"  SCREENED: {'; '.join(rec['why'])}"),
                    flush=True,
                )
                time.sleep(a.settle)
        res["victims"], res["across"] = analyse(res["records"], list(victims), conditions, cal)
        res["verdicts"] = {
            v: {c: d["verdict"] for c, d in o["conditions"].items()}
            for v, o in res["victims"].items()
        }
        for v, lines in res["verdicts"].items():
            for c, line in lines.items():
                print(f"{v}, {c}: {line}")
    except HardFailure as e:
        res["failure"] = str(e)
        write()
        print(f"fingerprint: FAILED: {e}", file=sys.stderr)
        return 1
    finally:
        cleanup()
    write()
    return 0


if __name__ == "__main__":
    sys.exit(main())
