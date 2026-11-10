"""The two modes' discriminator: its static facts, its preflight, its input check and its
analysis (docs/journal/2026-11-10-two-modes-discriminator.md, the registration; the design is
§3 of docs/journal/2026-11-10-two-modes.md, revised by the Researcher's REVISE, #145
6757560564). Diagnostic only: no merge gate reads it, and nothing it reports identifies a
mechanism: its outcomes are compatibility with registered signatures.

    python3 bench/two_modes.py static --out FILE
    python3 bench/two_modes.py layout ENV_JSON [--dry]
    python3 bench/two_modes.py concat CORPUS COPIES OUT --record JSON
    python3 bench/two_modes.py digest FILE
    python3 bench/two_modes.py preflight DIR
    python3 bench/two_modes.py judge DIR [--dry]

`static` reads, once, the facts the run does not change: the transparent huge page settings,
the RAPL constraints under /sys/class/powercap where readable, each CPU's topology, TSC flags
and thermal_throttle files, intel_pstate's and the first cpufreq policy's settings, and the
kernel. Every unreadable file is recorded with its error, never skipped.

`layout` prints the CPUs the run uses on one line: P-idle's, P-sib's ("-" for none) and the
decoder's mask. Registered: CPU 9 (P-idle), 19 (P-sib, its sibling) and 0-8,10-18 (the
decoder), the i9-10900K's last physical core taken out of the decoder's mask. With --dry, the
last physical core of the machine at hand instead, so the recipe's own check runs anywhere.

`concat` makes (or validates a cached) OUT, the corpus's bytes repeated COPIES times, by
content: the digest of the corpus streamed COPIES times is computed independently of the file
and compared with the file's own; a cache whose digest differs is rebuilt. The record says how
it was made, from what, and its digest.

`preflight` judges what the recipe collected before any run (DIR/preflight/): the registered
host with its declared settings, every required collector completed (exits.txt), an invariant
TSC on every CPU, the registered probe core and its sibling, the throttle counts readable, and
the probe's selftest pinned to P-idle's CPU and read back. PASS or REFUSED, one pass.

`judge` applies the admission contract (`admission_problems`: the identity's provenance, every
gated stage's success, every executable's and input's identity before and after, the
registered layout, schedule and counts, and every run's and interval's containment) and then
writes the table: each probed run of pass 1 classed fast (wall < 0.21 s) or slow, each
reading's median by class with its coverage, each row's value by the fixed rules below, and
each registered signature's status (compatible, contradicted or untestable). Missing evidence
never makes a signature compatible. Pass 2 (P-idle beside P-sib) is a serial contrast, reported.

Exit status: 0 VALID or DRY (the outcome on the first line), 2 INVALID, 1 a preflight REFUSED;
a failure of this script itself is INVALID with its reason, never a crash and never a VALID.
"""

import argparse
import glob
import hashlib
import json
import math
import os
import random
import re
import statistics as st
import sys

# ---- the registration (fixed before data) -------------------------------------------------

PROBE_SCHEMA = "two_modes/probe/2"
SUSTAINED_SCHEMA = "two_modes/sustained/1"
STATIC_SCHEMA = "two_modes/static/1"
IDENTITY_SCHEMA = "two_modes/identity/2"
JUDGEMENT_SCHEMA = "two_modes/judgement/2"
PREFLIGHT_SCHEMA = "two_modes/preflight/1"
CONCAT_SCHEMA = "two_modes/concat/1"

THRESHOLD_S = 0.21  # fast: the child's own wall below it (a descriptive separator, fixed)
RUNS = 60  # each pass
DRY_RUNS = 3
# Pass 1's probed (P) and unprobed (N) slots: 40 and 20 in an order drawn once, before data,
# from a fixed seed (recorded here and in the identity), so that a periodic nuisance cannot
# align with the schedule. The probed/unprobed contrast is randomised but descriptive: with
# persistence between runs its uncertainty is wider than independent draws would give, and
# carryover from a probed run to the next is not excluded. Pass 2: every run probed.
SCHEDULE_SEED = 20261110


def _schedule(seed):
    slots = ["P"] * 40 + ["N"] * 20
    random.Random(seed).shuffle(slots)
    return "".join(slots)


SCHEDULE = {1: _schedule(SCHEDULE_SEED), 2: "P"}
ARM = "mibz2-default33"  # the default plan, as the run-fed leader's bundle ran it
THREADS = 10
# MIN_CLASS is a display floor for a class (and for a row's observations in it), not a
# precision guarantee: below it the rows are described and nothing is called compatible
MIN_CLASS = 5
SUPPORTING = {"clock"}  # scaling_cur_freq: a supporting reading
DECODER_PREFIX = "mibz2-"  # the decoder's threads, by the names they give themselves

# per-run coverage a reading needs to be read at all (else it is missing, never imputed)
MIN_COVERAGE = 0.90  # P-idle's samples must span this share of the decode window
MIN_FREQ_SAMPLES = 3  # aligned clock readings in a run
MIN_RUNNING_OBS = 20  # running decoder-thread observations in a run

HOST_MODEL = "i9-10900K"
HOST_CPUS = 20
HOST_CORES = 10
IDLE_CPU = 9
SIB_CPU = 19
DECODER_CPUS = "0-8,10-18"
DECLARED = {
    "cpufreq_driver": "intel_pstate",
    "governor": "powersave",
    "epp": "balance_performance",
    "turbo": True,
}
FACTS_SCHEMA = "ab env --facts/1"
PREFLIGHT_SAMPLES = 3
PREFLIGHT_COLLECTORS = ("ab env --facts", "lscpu", "ab sample", "static facts", "probe selftest")

# the harness that produced the two modes: the run-fed leader's second attempt's own
# executables, from its archived bundle (bench 3.6.0, built at 1af3888a, frozen 2fab12ca)
SOURCE = {
    "evidence_commit": "65ab4b95fd659e9d910736233d95479f6f3b452e",
    "path": "run-fed-leader/20261109T201013Z.tar.gz",
    "sha256": "7e36a5a75903e31be1c1722382ef0098a9505033a5831b5b3e92f25a811be188",
    "ab_sha256": "ce1eea5306487a0797b00645ba341bbd6712e6a34a2d93e6f5edc67cb841b1bb",
    "windows_sha256": "cf5d74a6173d0cbdc91b5b366e1cdb6888755b7523f534124ba0ff6d1d884fd2",
}
WORKLOAD = {"out": 209700196, "hash": "a8830caee5c511a6"}
CORPUS_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
CORPUS_BYTES = 11854933  # the corpus's compressed size (bench/wikidata.json)
# the sources whose digests the identity records at the head and at the frozen commit
SOURCES = [
    "bench/two_modes_probe.c",
    "bench/two_modes.py",
    "bench/bounded.py",
    ".just/two-modes.just",
    "docs/journal/2026-11-10-two-modes-discriminator.md",
]

# Deadlines, each enforced by bench/bounded.py (the group TERMed, then KILLed after the grace,
# descendants included; 124 a timeout, 125 a group that could not be emptied). A stage that
# times out is recorded and never retried; the attempt is then INVALID.
DEADLINES = {
    "fetch": (120, 10),
    "corpus": (900, 10),
    "concat": (600, 10),
    "sustained": (300, 10),
    "pass": (900, 10),  # each pass, the archived ab's busy-gate waits included
    "judge": (300, 10),
}
# The sustained block, before the probe passes: one process, ab's own child, unpinned and with
# no probe, decodes the corpus concatenated SUSTAINED_COPIES times. Its input consumption
# (rchar) is read every 10 ms; it is reported raw (100 ms windows) and classified only by an
# error-aware rule on 1 s windows (sustained_reading).
SUSTAINED_COPIES = 50
DRY_SUSTAINED_COPIES = 3
SUSTAINED_REST_S = 60  # a fixed rest between the block and pass 1 (not a demonstrated reset)
SUSTAINED_SKIP_S = 1.0  # the initial read-ahead fill
SUSTAINED_WINDOW_S = 1.0
SUSTAINED_MIN_WINDOWS = 4
SUSTAINED_SAMPLE_S = 0.01

# The rows' rules: unvalidated descriptive thresholds, fixed before data. Each compares slow
# runs with fast runs in pass 1's probed runs, by medians over runs.
FULL = 0.95  # P-idle's progress or the decoder CPUs' clock, slow / fast: at least this, full
HALF = (0.40, 0.65)  # P-idle's progress, slow / fast, in this interval: half
RISING = 0.85  # early third / late third of a run, median over slow runs: below it, rising
MOST = 0.50  # running busy-sibling share in slow runs: at least this, and
MOST_GAIN = 0.15  # at least this much above the fast runs': most
SAME = 0.10  # running busy-sibling shares within this of each other: same
RISE_YES = (0.50, 0.25)  # a count rose in >= 50% of slow runs and 25 points more than fast: yes
RISE_NO = 0.10  # the share of slow runs with a rise exceeds the fast share by less: no
THP_YES = 0.50  # the child held huge pages in this much more of fast runs than slow: yes
THP_NO = 0.25  # within this of each other: no
SYS_YES = 0.05  # system time's share of CPU, slow minus fast, at least: yes
SYS_NO = 0.02  # below: no

ROWS = ["p_idle", "clock", "sibling", "throttle", "power_limit", "thp", "sys_share"]
ROW_TEXT = {
    "p_idle": "P-idle's elapsed-time progress in slow runs",
    "clock": "the decoder CPUs' clock during slow runs (supporting)",
    "sibling": "running decoder threads with a running sibling (a proxy)",
    "throttle": "throttle counts rise",
    "power_limit": "power-limit counts rise",
    "thp": "the child's huge pages in fast runs only",
    "sys_share": "system time's share grows",
}
UNREAD = "unread"
INSUFFICIENT = "insufficient"
MISSING = (UNREAD, INSUFFICIENT)
# the rows' values when the slow runs differ from the fast in nothing
BASELINE = {
    "p_idle": "full",
    "clock": "full",
    "sibling": "same",
    "throttle": "no",
    "power_limit": "no",
    "thp": "no",
    "sys_share": "no",
}
# each signature's prediction for each row (the set of values it admits), and its decisive
# rows: the observations without which it cannot be called compatible
COLUMNS = {
    "H1": (
        "the governor's ramp",
        {
            # (P-idle starts cold with each run: if the cores share one clock it ramps with
            # the decoder's, so H1 admits it full or rising; a conditional prediction)
            "p_idle": {"full", "rising"},
            "clock": {"low, then rising"},
            "sibling": {"same"},
            "throttle": {"no"},
            "power_limit": {"no"},
            "thp": {"no"},
            "sys_share": {"no"},
        },
        {"clock"},
    ),
    "H2": (
        "sibling packing",
        {
            "p_idle": {"full"},
            "clock": {"full"},
            "sibling": {"most"},
            "throttle": {"no"},
            "power_limit": {"no"},
            "thp": {"no"},
            "sys_share": {"no"},
        },
        {"sibling"},
    ),
    "H3": (
        "stop-clock throttling",
        {
            "p_idle": {"half"},
            "clock": {"full"},
            "sibling": {"same"},
            "throttle": {"yes"},
            "power_limit": {"no"},
            "thp": {"no"},
            "sys_share": {"no"},
        },
        {"p_idle", "throttle"},
    ),
    "H4": (
        "huge pages",
        {
            "p_idle": {"full"},
            "clock": {"full"},
            "sibling": {"same"},
            "throttle": {"no"},
            "power_limit": {"no"},
            "thp": {"yes"},
            "sys_share": {"yes"},
        },
        {"thp", "sys_share"},
    ),
    "H5": (
        "the package power limit",
        {
            "p_idle": {"reduced"},
            "clock": {"reduced"},
            "sibling": {"same"},
            "throttle": {"no"},
            "power_limit": {"yes"},
            "thp": {"no"},
            "sys_share": {"no"},
        },
        {"p_idle", "clock", "power_limit"},
    ),
}


# ---- small helpers ------------------------------------------------------------------------


def num(x):
    return (
        x if isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) else None
    )


def med(xs):
    xs = [x for x in xs if num(x) is not None]
    return st.median(xs) if xs else None


def ratio(a, b):
    return a / b if num(a) is not None and num(b) not in (None, 0) else None


def cpu_list(s):
    """A CPU list ("0-8,10-18") as a sorted list of ints; None if malformed"""
    out = set()
    if not isinstance(s, str) or not s.strip():
        return None
    for part in s.strip().split(","):
        a, _, b = part.partition("-")
        if not (a.isdigit() and (b == "" or b.isdigit())):
            return None
        out.update(range(int(a), int(b or a) + 1))
    return sorted(out)


def load_json(path, what):
    try:
        with open(path) as f:
            return json.load(f), None
    except (OSError, ValueError) as e:
        return None, f"{what}: {path} does not load: {e}"


def load_jsonl(path, what):
    try:
        with open(path) as f:
            return [json.loads(line) for line in f if line.strip()], None
    except (OSError, ValueError) as e:
        return None, f"{what}: {path} does not load: {e}"


def write(obj, stem, md=None):
    with open(stem + ".json", "w") as f:
        json.dump(obj, f, indent=1)
    if md is not None:
        with open(stem + ".md", "w") as f:
            f.write(md)


# ---- static facts -------------------------------------------------------------------------


def read_file(path):
    try:
        with open(path) as f:
            return f.read().strip()
    except OSError as e:
        return {"error": e.strerror or str(e)}


def static_facts(root="/"):
    def p(x):
        return os.path.join(root, x.lstrip("/"))

    files = {}
    for x in (
        "/sys/kernel/mm/transparent_hugepage/enabled",
        "/sys/kernel/mm/transparent_hugepage/defrag",
        "/sys/kernel/mm/transparent_hugepage/shmem_enabled",
        "/sys/kernel/mm/transparent_hugepage/use_zero_page",
        "/sys/kernel/mm/transparent_hugepage/hpage_pmd_size",
        "/sys/kernel/mm/transparent_hugepage/khugepaged/defrag",
    ):
        files[x] = read_file(p(x))
    # the RAPL zones and their constraints, every file each zone has (energy_uj is often
    # root-only: its error is recorded like any other)
    for zone in sorted(glob.glob(p("/sys/class/powercap/intel-rapl*"))):
        for f in sorted(glob.glob(os.path.join(zone, "*"))):
            if os.path.isfile(f) and os.path.basename(f) != "uevent":
                files["/" + os.path.relpath(f, root)] = read_file(f)
    for f in sorted(glob.glob(p("/sys/devices/system/cpu/intel_pstate/*"))):
        if os.path.isfile(f):
            files["/" + os.path.relpath(f, root)] = read_file(f)
    for x in (
        "scaling_driver",
        "scaling_governor",
        "energy_performance_preference",
        "cpuinfo_min_freq",
        "cpuinfo_max_freq",
        "base_frequency",
    ):
        y = "/sys/devices/system/cpu/cpufreq/policy0/" + x
        files[y] = read_file(p(y))
    cpus = {}
    flags = {}
    model = None
    cpuinfo = read_file(p("/proc/cpuinfo"))
    if isinstance(cpuinfo, str):
        cur = None
        for line in cpuinfo.splitlines():
            k, _, v = line.partition(":")
            k = k.strip()
            if k == "processor":
                cur = v.strip()
            elif k == "model name" and model is None:
                model = v.strip()
            elif k == "flags" and cur is not None:
                fl = set(v.split())
                flags[cur] = {f: f in fl for f in ("constant_tsc", "nonstop_tsc", "ht")}
    for d in sorted(
        glob.glob(p("/sys/devices/system/cpu/cpu[0-9]*")), key=lambda x: int(x.rsplit("cpu", 1)[1])
    ):
        c = d.rsplit("cpu", 1)[1]
        tt = os.path.join(d, "thermal_throttle")
        cpus[c] = {
            "siblings": read_file(os.path.join(d, "topology/thread_siblings_list")),
            "core_id": read_file(os.path.join(d, "topology/core_id")),
            "thermal_throttle": sorted(os.listdir(tt)) if os.path.isdir(tt) else None,
            "tsc_flags": flags.get(c),
        }
    u = os.uname()
    return {
        "schema": STATIC_SCHEMA,
        "model": model,
        "uname": {"sysname": u.sysname, "release": u.release, "version": u.version},
        "files": files,
        "cpus": cpus,
    }


def siblings_of(static):
    """Each CPU's siblings (itself excluded), from the static facts' topology"""
    out = {}
    for c, info in (static or {}).get("cpus", {}).items():
        lst = cpu_list(info.get("siblings")) if isinstance(info, dict) else None
        if lst is not None and c.isdigit():
            out[int(c)] = set(lst) - {int(c)}
    return out


# ---- the layout -----------------------------------------------------------------------------


def layout(env, dry):
    """(P-idle's CPU, P-sib's CPU or None, the decoder's CPU list): registered, or with dry
    the last physical core of the machine at hand"""
    if not dry:
        return IDLE_CPU, SIB_CPU, DECODER_CPUS
    cores = [cpu_list(g) for g in env.get("core_cpus", [])]
    allowed = env.get("cpus_allowed")
    if not cores or any(c is None for c in cores) or not allowed or len(cores) < 2:
        raise ValueError("the machine's cores are not read (or there is only one)")
    last = cores[-1]
    rest = [c for c in allowed if c not in last]
    return last[0], (last[1] if len(last) > 1 else None), ",".join(map(str, rest))


# ---- the preflight ------------------------------------------------------------------------


def collector_problems(text):
    """The required collectors' recorded completion: absent, malformed, duplicated or
    unregistered status lines, and any status but 0"""
    if text is None:
        return ["the collectors' exit statuses (preflight/exits.txt) are missing"]
    bad, seen = [], {}
    for line in text.splitlines():
        if not line.strip():
            continue
        m = re.fullmatch(r"(.+): exit ([0-9]+)", line)
        if not m or m.group(1) not in PREFLIGHT_COLLECTORS:
            bad.append(f"exits.txt: an unregistered or malformed line {line!r}")
            continue
        if m.group(1) in seen:
            bad.append(f"exits.txt: {m.group(1)} recorded twice")
            continue
        seen[m.group(1)] = int(m.group(2))
    for name in PREFLIGHT_COLLECTORS:
        if name not in seen:
            bad.append(f"{name}: no exit status recorded")
        elif seen[name] != 0:
            how = {124: " (timed out)", 137: " (killed)"}.get(seen[name], "")
            bad.append(f"{name} exited {seen[name]}{how}: its output is not admitted")
    return bad


def host_problems(env, lscpu, samples):
    bad = []
    model = None
    for line in (lscpu or "").splitlines():
        k, _, v = line.partition(":")
        if k.strip() == "Model name":
            model = v.strip()
    if not (isinstance(model, str) and HOST_MODEL in model):
        bad.append(f"lscpu's model {model!r}: not the registered {HOST_MODEL}")
    if not isinstance(env, dict):
        return bad + ["ab env: not an object"]
    if env.get("facts") != FACTS_SCHEMA:
        bad.append(f"ab env: {env.get('facts')!r}, not the passive {FACTS_SCHEMA!r}")
    if env.get("cpus") != HOST_CPUS or env.get("cores") != HOST_CORES:
        bad.append(
            f"ab env: {env.get('cpus')!r} CPUs and {env.get('cores')!r} cores, "
            f"not {HOST_CPUS} and {HOST_CORES}"
        )
    if env.get("cpus_allowed") != list(range(HOST_CPUS)):
        bad.append(f"ab env: the CPUs allowed {env.get('cpus_allowed')!r}, not all {HOST_CPUS}")
    cores = env.get("core_cpus")
    if not (
        isinstance(cores, list)
        and f"{IDLE_CPU},{SIB_CPU}" in cores
        and cores[-1] == f"{IDLE_CPU},{SIB_CPU}"
    ):
        bad.append(
            f"ab env: core_cpus {cores!r}: the probe core {IDLE_CPU},{SIB_CPU} is not the last"
        )
    q = env.get("cpu_quota")
    if not (isinstance(q, dict) and q.get("state") == "unlimited"):
        bad.append(f"ab env: the CPU quota is not established unlimited: {q!r}")
    m = env.get("machine") if isinstance(env.get("machine"), dict) else {}
    for k, want in DECLARED.items():
        if not (type(m.get(k)) is type(want) and m.get(k) == want):
            bad.append(f"ab env: {k} {m.get(k)!r}, not the declared {want!r}")
    if not (isinstance(samples, list) and len(samples) == PREFLIGHT_SAMPLES):
        n = len(samples) if isinstance(samples, list) else samples
        return bad + [f"{n!r} sysfs samples, not {PREFLIGHT_SAMPLES}"]
    for i, smp in enumerate(samples):
        s = smp.get("state") if isinstance(smp, dict) else None
        if not isinstance(s, dict):
            bad.append(f"sample {i}: no state")
            continue
        for k in ("governor", "epp", "turbo"):
            if s.get(k) != DECLARED[k]:
                bad.append(f"sample {i}: {k} {s.get(k)!r}, not the declared {DECLARED[k]!r}")
    return bad


def static_problems(static):
    if not isinstance(static, dict) or static.get("schema") != STATIC_SCHEMA:
        return ["the static facts are not the registered collector's"]
    bad = []
    cpus = static.get("cpus", {})
    if len(cpus) != HOST_CPUS:
        bad.append(f"the static facts list {len(cpus)} CPUs, not {HOST_CPUS}")
    no_tsc = [
        c
        for c, i in cpus.items()
        if not (
            isinstance(i.get("tsc_flags"), dict)
            and i["tsc_flags"].get("constant_tsc")
            and i["tsc_flags"].get("nonstop_tsc")
        )
    ]
    if no_tsc:
        bad.append(f"CPUs {sorted(no_tsc, key=int)[:6]} lack constant_tsc or nonstop_tsc")
    sib = siblings_of(static)
    if sib.get(IDLE_CPU) != {SIB_CPU} or sib.get(SIB_CPU) != {IDLE_CPU}:
        bad.append(f"CPUs {IDLE_CPU} and {SIB_CPU} are not one physical core's two siblings")
    unread = [
        c
        for c, i in cpus.items()
        if not (
            isinstance(i.get("thermal_throttle"), list)
            and "core_throttle_count" in i["thermal_throttle"]
            and "package_throttle_count" in i["thermal_throttle"]
        )
    ]
    if unread:
        bad.append(f"CPUs {sorted(unread, key=int)[:6]}: no core and package throttle counts")
    return bad


def selftest_problems(st_rec):
    if not isinstance(st_rec, dict):
        return ["the probe's selftest wrote no record"]
    if not (
        st_rec.get("cpu") == IDLE_CPU
        and st_rec.get("pinned") is True
        and st_rec.get("readback") == str(IDLE_CPU)
        and isinstance(st_rec.get("samples"), int)
        and st_rec["samples"] >= 2
    ):
        return [f"the probe's selftest was not pinned to CPU {IDLE_CPU} and read back: {st_rec!r}"]
    return []


def preflight(args):
    d = os.path.join(args.dir, "preflight")
    bad = []
    try:
        try:
            with open(os.path.join(d, "exits.txt")) as f:
                exits = f.read()
        except OSError:
            exits = None
        bad += collector_problems(exits)
        env, why = load_json(os.path.join(d, "env.json"), "ab env")
        bad += [why] if why else []
        try:
            with open(os.path.join(d, "lscpu.txt")) as f:
                lscpu = f.read()
        except OSError as e:
            lscpu = None
            bad.append(f"lscpu: {e.strerror}")
        samples, why = load_jsonl(os.path.join(d, "sample.jsonl"), "ab sample")
        bad += [why] if why else []
        bad += host_problems(env, lscpu, samples)
        static, why = load_json(os.path.join(d, "static.json"), "static facts")
        bad += [why] if why else static_problems(static)
        sel, why = load_json(os.path.join(d, "selftest.json"), "probe selftest")
        bad += [why] if why else selftest_problems(sel)
    except Exception as e:  # (the preflight's own failure refuses; it never passes)
        bad.append(f"the preflight failed: {type(e).__name__}: {e}")
    verdict = "REFUSED" if bad else "PASS"
    write(
        {"schema": PREFLIGHT_SCHEMA, "verdict": verdict, "problems": bad},
        os.path.join(args.dir, "preflight"),
    )
    print(verdict + ("" if not bad else ": " + "; ".join(bad)))
    return 0 if not bad else 1


# ---- one run's readings -------------------------------------------------------------------


def probe_samples(probe):
    """A probe's sample times (TSC), in order: its first sample, then each one after"""
    if not probe or probe.get("first_tsc") is None:
        return []
    t = probe["first_tsc"]
    out = [t]
    for d in probe.get("dtsc", []):
        t += d
        out.append(t)
    return out


def progress(probe, a, b, hz):
    """P-x's elapsed-time progress over [a, b] (TSC): the iterations of every interval whose
    both ends lie inside, over the TSC span from the first such sample to the last, in
    Giter/s; with its coverage (that span over b - a), the gaps at each boundary and the
    longest interval inside, in ms. The interval median (the active-loop rate) beside it. A
    pause inside the span lowers the progress; a pause across a boundary lowers the coverage.
    Why the loop made no progress (descheduled, halted or slowed) is not read from the TSC."""
    ts = [t for t in probe_samples(probe) if a <= t <= b]
    k = probe.get("iters_per_sample") if probe else None
    span = b - a
    if len(ts) < 2 or not k or span <= 0:
        return {"progress": None, "coverage": 0.0 if span > 0 else None, "n": max(len(ts) - 1, 0)}
    d = [y - x for x, y in zip(ts, ts[1:])]
    return {
        "progress": (len(ts) - 1) * k * hz / (ts[-1] - ts[0]) / 1e9,
        "active_median": med(k * hz / x / 1e9 for x in d if x > 0),
        "coverage": (ts[-1] - ts[0]) / span,
        "gap_start_ms": (ts[0] - a) / hz * 1e3,
        "gap_end_ms": (b - ts[-1]) / hz * 1e3,
        "longest_ms": max(d) / hz * 1e3,
        "n": len(ts) - 1,
    }


def throttle_delta(before, after, key, how):
    """Summed (core counts) or the largest (package counts) rise over CPUs; None if any CPU
    lacks the count before or after"""
    b = (before or {}).get("thermal_throttle") or {}
    a = (after or {}).get("thermal_throttle") or {}
    if not b or set(b) != set(a):
        return None
    ds = []
    for c in b:
        x, y = (b[c] or {}).get(key), (a[c] or {}).get(key)
        if not (isinstance(x, int) and isinstance(y, int)):
            return None
        ds.append(y - x)
    return sum(ds) if how == "sum" else max(ds)


def vm_delta(before, after, key):
    x = ((before or {}).get("vmstat") or {}).get(key)
    y = ((after or {}).get("vmstat") or {}).get(key)
    return y - x if isinstance(x, int) and isinstance(y, int) else None


def readings(rec, probe, siblings):
    """One run's readings, each with its coverage, or None where it is not covered"""
    hz = probe["tsc_hz"]
    t0, t1 = probe["tsc"]["exec"], probe["tsc"]["exit"]
    span_us = (probe["mono_ns"]["exit"] - probe["mono_ns"]["exec"]) / 1000
    out = {"wall": rec.get("wall"), "cpu": rec.get("cpu")}
    out["fast"] = num(rec.get("wall")) is not None and rec["wall"] < THRESHOLD_S
    out["probed"] = probe.get("probed") is not False
    third = (t1 - t0) / 3
    for p in probe.get("probes", []):
        key = {"P-idle": "p_idle", "P-sib": "p_sib"}.get(p.get("name"))
        if not key:
            continue
        whole = progress(p, t0, t1, hz)
        early = progress(p, t0, t0 + third, hz)
        late = progress(p, t1 - third, t1, hz)
        ok = whole["coverage"] is not None and whole["coverage"] >= MIN_COVERAGE
        out[key] = whole["progress"] if ok else None
        out[key + "_active"] = whole.get("active_median")
        out[key + "_coverage"] = whole["coverage"]
        out[key + "_n"] = whole["n"]
        out[key + "_longest_ms"] = whole.get("longest_ms")
        out[key + "_gaps_ms"] = [whole.get("gap_start_ms"), whole.get("gap_end_ms")]
        out[key + "_early"] = early["progress"] if ok else None
        out[key + "_late"] = late["progress"] if ok else None
        cpu_ns = p.get("cpu_ns")
        out[key + "_cpu_share"] = (
            cpu_ns / 1e9 / ((p["last_tsc"] - p["first_tsc"]) / hz)
            if isinstance(cpu_ns, int) and p.get("last_tsc") and p["last_tsc"] > p["first_tsc"]
            else None
        )
    # placement: the decoder's threads, by name; a running (R) thread whose sibling CPU also
    # shows a running decoder thread in the same sample. The task files are read one after
    # another, not as one snapshot: a proxy for simultaneous occupancy, never proof of it.
    # A sleeping thread's last CPU is never counted as occupied
    comm = probe["placement"].get("comm", {})
    dec_tids = {int(t) for t, n in comm.items() if str(n).startswith(DECODER_PREFIX)}
    running_at = []  # (t_us, set of CPUs with a running decoder thread)
    busy, obs, last_cpu_busy, last_cpu_obs = 0, 0, 0, 0
    for t_us, tasks in probe["placement"].get("samples", []):
        dec = [(tid, cpu, s) for tid, cpu, s in tasks if tid in dec_tids]
        run = [(tid, cpu) for tid, cpu, s in dec if s == "R"]
        running_at.append((t_us, {cpu for _, cpu in run}))
        for tid, cpu in run:
            sib = siblings.get(cpu, set())
            busy += any(c in sib for t2, c in run if t2 != tid)
            obs += 1
        for tid, cpu, _ in dec:  # (the last-CPU coincidence, reported only)
            sib = siblings.get(cpu, set())
            last_cpu_busy += any(c in sib for t2, c, _ in dec if t2 != tid)
            last_cpu_obs += 1
    out["placement_samples"] = len(probe["placement"].get("samples", []))
    out["running_obs"] = obs
    out["sibling_running"] = busy / obs if obs >= MIN_RUNNING_OBS else None
    out["sibling_last_cpu"] = last_cpu_busy / last_cpu_obs if last_cpu_obs else None
    # the clock: at each frequency reading, the median over the CPUs where the placement sample
    # nearest it (within 2 ms) saw a running decoder thread; scaling_cur_freq is an average
    # over the kernel's last updates, not an instantaneous clock
    clk, read_us = [], []
    for t_us, r_us, khz in probe["freq"].get("samples", []):
        read_us.append(r_us)
        if not 0 <= t_us <= span_us or not running_at:
            continue
        tn, cpus = min(running_at, key=lambda x: abs(x[0] - t_us))
        if abs(tn - t_us) > 2000 or not cpus:
            continue
        v = med([khz[c] for c in cpus if c < len(khz) and khz[c] is not None])
        if v is not None:
            clk.append((t_us, v / 1e6))
    ok = len(clk) >= MIN_FREQ_SAMPLES
    out["freq_samples"] = len(clk)
    out["clock_ghz"] = med(v for _, v in clk) if ok else None
    out["clock_early"] = med(v for t, v in clk if t <= span_us / 3) if ok else None
    out["clock_late"] = med(v for t, v in clk if t >= 2 * span_us / 3) if ok else None
    out["freq_read_us"] = med(read_us)
    b, a = probe.get("before"), probe.get("after")
    out["core_throttle"] = throttle_delta(b, a, "core_throttle_count", "sum")
    out["package_throttle"] = throttle_delta(b, a, "package_throttle_count", "max")
    out["core_throttle_ms"] = throttle_delta(b, a, "core_throttle_total_time_ms", "sum")
    out["package_throttle_ms"] = throttle_delta(b, a, "package_throttle_total_time_ms", "max")
    out["core_power_limit"] = throttle_delta(b, a, "core_power_limit_count", "sum")
    out["package_power_limit"] = throttle_delta(b, a, "package_power_limit_count", "max")
    # (system-wide: reported, never attributed to the child)
    out["thp_fault_alloc_global"] = vm_delta(b, a, "thp_fault_alloc")
    out["thp_fault_fallback_global"] = vm_delta(b, a, "thp_fault_fallback")
    sm = probe.get("smaps") or {}
    out["anon_huge_kb"] = sm.get("anon_huge_kb_max") if (sm.get("reads") or 0) > 0 else None
    ru = probe.get("rusage") or {}
    out["minflt"] = ru.get("minflt")
    out["utime_s"] = ru.get("utime_s")
    out["stime_s"] = ru.get("stime_s")
    tot = (ru.get("utime_s") or 0) + (ru.get("stime_s") or 0)
    out["sys_share"] = ru["stime_s"] / tot if tot and num(ru.get("stime_s")) is not None else None
    return out


def rose(r, a, b):
    """1 if either count rose, 0 if neither, None if both are unread"""
    xs = [r.get(a), r.get(b)]
    if all(x is None for x in xs):
        return None
    return int(any(x is not None and x > 0 for x in xs))


# ---- the class table, its rows and the signatures -----------------------------------------

TABLE_KEYS = [
    "wall",
    "cpu",
    "p_idle",
    "p_idle_active",
    "p_idle_coverage",
    "p_idle_longest_ms",
    "p_idle_early",
    "p_idle_late",
    "p_idle_cpu_share",
    "p_sib",
    "clock_ghz",
    "clock_early",
    "clock_late",
    "sibling_running",
    "sibling_last_cpu",
    "running_obs",
    "core_throttle",
    "package_throttle",
    "core_throttle_ms",
    "package_throttle_ms",
    "core_power_limit",
    "package_power_limit",
    "thp_fault_alloc_global",
    "anon_huge_kb",
    "minflt",
    "utime_s",
    "stime_s",
    "sys_share",
]
# each row's per-run inputs: its observations are the runs where all of them are read
ROW_INPUTS = {
    "p_idle": ["p_idle"],
    "clock": ["clock_ghz"],
    "sibling": ["sibling_running"],
    "throttle": ["throttle_rose"],
    "power_limit": ["power_limit_rose"],
    "thp": ["anon_huge_kb"],
    "sys_share": ["sys_share"],
}


def frac(xs):
    xs = [x for x in xs if x is not None]
    return sum(xs) / len(xs) if xs else None


def class_table(runs):
    out = {}
    for name, fast in (("fast", True), ("slow", False)):
        g = [dict(r) for r in runs if r["fast"] == fast]
        for r in g:
            r["throttle_rose"] = rose(r, "core_throttle", "package_throttle")
            r["power_limit_rose"] = rose(r, "core_power_limit", "package_power_limit")
        row = {"n": len(g)}
        for k in TABLE_KEYS:
            row[k] = med(r.get(k) for r in g)
        row["throttle_rose"] = frac(r["throttle_rose"] for r in g)
        row["power_limit_rose"] = frac(r["power_limit_rose"] for r in g)
        row["thp_held"] = frac(
            None if r.get("anon_huge_kb") is None else int(r["anon_huge_kb"] > 0) for r in g
        )
        row["p_idle_rise"] = med(ratio(r.get("p_idle_early"), r.get("p_idle_late")) for r in g)
        row["clock_rise"] = med(ratio(r.get("clock_early"), r.get("clock_late")) for r in g)
        row["coverage"] = {
            k: sum(1 for r in g if all(r.get(x) is not None for x in xs))
            for k, xs in ROW_INPUTS.items()
        }
        out[name] = row
    return out


def row_values(t):
    """Each row's value from the class table, by the registered rules. A row with fewer than
    MIN_CLASS observations in either class is INSUFFICIENT; one never read is UNREAD"""
    f, s = t["fast"], t["slow"]
    v = {}
    for r in ROWS:
        cf, cs = f["coverage"][r], s["coverage"][r]
        if cf == 0 and cs == 0:
            v[r] = UNREAD
        elif cf < MIN_CLASS or cs < MIN_CLASS:
            v[r] = INSUFFICIENT
    if "p_idle" not in v:
        r = ratio(s["p_idle"], f["p_idle"])
        rising = s["p_idle_rise"] is not None and s["p_idle_rise"] < RISING
        if r is None:
            v["p_idle"] = UNREAD
        elif rising and r < FULL:
            v["p_idle"] = "rising"
        elif r >= FULL:
            v["p_idle"] = "full"
        elif HALF[0] <= r <= HALF[1]:
            v["p_idle"] = "half"
        elif r > HALF[1]:
            v["p_idle"] = "reduced"
        else:
            v["p_idle"] = "below half"
    if "clock" not in v:
        rc = ratio(s["clock_ghz"], f["clock_ghz"])
        if rc is None:
            v["clock"] = UNREAD
        elif rc >= FULL:
            v["clock"] = "full"
        elif s["clock_rise"] is not None and s["clock_rise"] < RISING:
            v["clock"] = "low, then rising"
        else:
            v["clock"] = "reduced"
    if "sibling" not in v:
        a, b = s["sibling_running"], f["sibling_running"]
        if a >= MOST and a - b >= MOST_GAIN:
            v["sibling"] = "most"
        elif abs(a - b) < SAME:
            v["sibling"] = "same"
        else:
            v["sibling"] = "between"
    for row, key in (("throttle", "throttle_rose"), ("power_limit", "power_limit_rose")):
        if row in v:
            continue
        a, b = s[key], f[key]
        if a >= RISE_YES[0] and a - b >= RISE_YES[1]:
            v[row] = "yes"
        elif a - b < RISE_NO:
            v[row] = "no"
        else:
            v[row] = "between"
    if "thp" not in v:
        a, b = f["thp_held"], s["thp_held"]
        if a - b >= THP_YES:
            v["thp"] = "yes"
        elif abs(a - b) < THP_NO:
            v["thp"] = "no"
        else:
            v["thp"] = "between"
    if "sys_share" not in v:
        a, b = s["sys_share"], f["sys_share"]
        if a - b >= SYS_YES:
            v["sys_share"] = "yes"
        elif a - b < SYS_NO:
            v["sys_share"] = "no"
        else:
            v["sys_share"] = "between"
    return {r: v[r] for r in ROWS}


def signatures(values):
    """Each registered signature's status: contradicted (a read row it does not admit),
    untestable (none of its decisive rows read), compatible (every read row admitted, every
    decisive row read), or compatible in part (admitted, some decisive rows missing)"""
    out = {}
    for h, (_, pred, decisive) in COLUMNS.items():
        missing = [r for r in ROWS if values[r] in MISSING]
        against = [r for r in ROWS if r not in missing and values[r] not in pred[r]]
        dmiss = sorted(r for r in decisive if r in missing)
        if against:
            out[h] = {"status": "contradicted", "by": against}
        elif len(dmiss) == len(decisive):
            out[h] = {"status": "untestable", "missing": dmiss}
        elif dmiss:
            out[h] = {"status": "compatible in part", "missing": dmiss}
        else:
            out[h] = {"status": "compatible", "missing": []}
    return out


def outcome(table, values):
    f, s = table["fast"]["n"], table["slow"]["n"]
    if f < MIN_CLASS or s < MIN_CLASS:
        return (
            f"INSUFFICIENT CLASS SUPPORT: {f} fast and {s} slow probed runs (a floor of "
            f"{MIN_CLASS} a class); the rows are described and no signature is assessed"
        )
    sig = signatures(values)
    comp = [h for h, x in sig.items() if x["status"].startswith("compatible")]
    unt = [h for h, x in sig.items() if x["status"] == "untestable"]
    notes = []
    for h in comp:
        if sig[h]["missing"]:
            notes.append(
                f"{h} in part: {', '.join(ROW_TEXT[r] for r in sig[h]['missing'])} missing"
            )
        if COLUMNS[h][2] <= SUPPORTING:
            notes.append(f"{h} rests on the supporting clock reading")
    if unt:
        notes.append("untestable: " + ", ".join(unt))
    note = f" ({'; '.join(notes)})" if notes else ""
    if comp:
        return f"COMPATIBLE WITH {', '.join(comp)}{note}"
    if len(unt) == len(COLUMNS):
        return f"INSUFFICIENT MEASUREMENT SUPPORT: no signature's decisive rows were read{note}"
    desc = "; ".join(f"{ROW_TEXT[r]}: {values[r]}" for r in ROWS)
    return f"NO REGISTERED SIGNATURE MATCHES{note}; {desc}"


# ---- the sustained block --------------------------------------------------------------------


def sustained_reading(rec):
    """The sustained block's input consumption: raw 100 ms windows, then 1 s windows' rates
    (bytes/s) with an error bound, and a shape classified only beyond that bound.

    The bound: the decoder reads its input in chunks of up to C bytes (the largest increment
    seen after the read-ahead fill), refilling when its buffer falls by a chunk. A window's
    count of arrived bytes differs from the bytes consumed in it by less than one chunk at its
    edges and one chunk of refill jitter, and the 10 ms sampling adds two samples' worth:
    e = (2C + 2 * 0.01 s * rate) / W. A difference between windows is a change only beyond
    2e. Rates are input bytes; no compression ratio converts them, and no burst threshold
    classifies them."""
    pts = [(t / 1e6, v) for t, v in rec["io"]["samples"] if isinstance(v, int) and v >= 0]
    if len(pts) < 2:
        return {"shape": "unread", "why": "no input consumption was read"}

    def at(t):
        v = pts[0][1]
        for x, y in pts:
            if x > t:
                break
            v = y
        return v

    incs = [(x, y - y0) for (x, y), (_, y0) in zip(pts[1:], pts)]
    after = [d for x, d in incs if x > SUSTAINED_SKIP_S and d > 0]
    chunk = max(after) if after else max((d for _, d in incs), default=0)
    reads = [x for x, d in incs if chunk and d > chunk / 2]
    last = max(reads) if reads else None
    end = pts[-1][0]
    raw = []
    t = 0.0
    while t + 0.1 <= end + 1e-9:
        raw.append(at(t + 0.1) - at(t))
        t += 0.1
    rates = []
    t = SUSTAINED_SKIP_S
    while last is not None and t + SUSTAINED_WINDOW_S <= last + 1e-9:
        rates.append((at(t + SUSTAINED_WINDOW_S) - at(t)) / SUSTAINED_WINDOW_S)
        t += SUSTAINED_WINDOW_S
    out = {
        "chunk_bytes": chunk,
        "last_read_s": last,
        "span_s": rec.get("span_s"),
        "windows_100ms_bytes": raw,
        "rates_bytes_per_s": rates,
    }
    if len(rates) < SUSTAINED_MIN_WINDOWS:
        out.update(shape="too short", error_bytes_per_s=None)
        return out
    e = (2 * chunk + 2 * SUSTAINED_SAMPLE_S * med(rates)) / SUSTAINED_WINDOW_S
    steps = [b - a for a, b in zip(rates, rates[1:]) if abs(b - a) > 2 * e]
    reversals = sum(1 for a, b in zip(steps, steps[1:]) if (a > 0) != (b > 0))
    if max(rates) - min(rates) <= 2 * e:
        shape = "constant within error"
    elif reversals >= 2:
        shape = "alternating beyond error"
    elif steps and all(x < 0 for x in steps) or not steps and rates[-1] < rates[0] - 2 * e:
        shape = "a sustained change, down"
    elif steps and all(x > 0 for x in steps) or not steps and rates[-1] > rates[0] + 2 * e:
        shape = "a sustained change, up"
    else:
        shape = "varies beyond error"
    out.update(
        shape=shape,
        error_bytes_per_s=e,
        significant_steps=len(steps),
        reversals=reversals,
        first_rate=rates[0],
        last_rate=rates[-1],
    )
    return out


# ---- the concatenation ------------------------------------------------------------------------


def sha256_file(path, block=1 << 20):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while b := f.read(block):
            h.update(b)
    return h.hexdigest()


def repeated_digest(corpus, copies, block=1 << 20):
    """The SHA-256 of `corpus`'s bytes repeated `copies` times, computed from the corpus alone"""
    with open(corpus, "rb") as f:
        data = f.read()
    h = hashlib.sha256()
    for _ in range(copies):
        h.update(data)
    return h.hexdigest(), len(data) * copies


def concat(corpus, copies, out):
    """Make or validate OUT as the corpus repeated: by content, never by size"""
    want, size = repeated_digest(corpus, copies)
    cached = os.path.exists(out) and os.path.getsize(out) == size and sha256_file(out) == want
    if not cached:
        tmp = out + ".tmp"
        with open(corpus, "rb") as f:
            data = f.read()
        with open(tmp, "wb") as f:
            for _ in range(copies):
                f.write(data)
        os.replace(tmp, out)
    got = sha256_file(out)
    return {
        "schema": CONCAT_SCHEMA,
        "method": "the corpus's bytes repeated COPIES times, nothing between",
        "corpus_sha256": sha256_file(corpus),
        "corpus_bytes": os.path.getsize(corpus),
        "copies": copies,
        "bytes": os.path.getsize(out),
        "expected_sha256": want,
        "sha256": got,
        "cached": cached,
        "ok": got == want,
    }


# ---- the admission contract ---------------------------------------------------------------


def hex40(x):
    return isinstance(x, str) and re.fullmatch(r"[0-9a-f]{40}", x) is not None


def hex64(x):
    return isinstance(x, str) and re.fullmatch(r"[0-9a-f]{64}", x) is not None


def admission_problems(ident, d, dry):
    """The one contract the recipe's record must meet and the judge enforces on replay: the
    freeze and the sources' provenance, every gated stage's success, every executable's and
    input's identity before and after, and the registered layout, counts and schedule. In a
    DRY check the registered-host and freeze clauses are waived, never the consistency ones"""
    bad = []
    if not isinstance(ident, dict) or ident.get("schema") != IDENTITY_SCHEMA:
        return [f"the identity is not {IDENTITY_SCHEMA}"]
    if bool(ident.get("dry")) != bool(dry):
        bad.append(f"the identity's dry {ident.get('dry')!r} is not this judgement's {dry!r}")
    if not dry:
        if not hex40(ident.get("frozen")):
            bad.append(f"no frozen commit ({ident.get('frozen')!r})")
        if ident.get("inputs_match_frozen") is not True:
            bad.append("the inputs did not match the frozen commit")
        for p, s in (ident.get("sources") or {}).items():
            if not (
                isinstance(s, dict) and hex64(s.get("head")) and s.get("head") == s.get("frozen")
            ):
                bad.append(f"{p}: its digest at the head is not its digest at the frozen commit")
        if sorted(ident.get("sources") or {}) != sorted(SOURCES):
            bad.append("the identity does not record every source's digests")
        if ident.get("preflight_exit") != 0:
            bad.append(f"the preflight exited {ident.get('preflight_exit')!r}")
        pf, why = load_json(os.path.join(d, "preflight.json"), "the preflight's judgement")
        if why or (pf or {}).get("verdict") != "PASS":
            bad.append(why or f"the preflight's verdict is {(pf or {}).get('verdict')!r}")
        h = ident.get("harness") or {}
        if (h.get("evidence_commit"), h.get("path"), h.get("bundle_sha256")) != (
            SOURCE["evidence_commit"],
            SOURCE["path"],
            SOURCE["sha256"],
        ):
            bad.append(f"the harness's provenance {h!r} is not the registered bundle's")
        for k in ("ab_sha256", "windows_sha256"):
            if ident.get(k) != SOURCE[k]:
                bad.append(f"{k} {ident.get(k)!r} is not the archived executable's")
        if ident.get("corpus_sha256") != CORPUS_SHA256:
            bad.append(f"the corpus {ident.get('corpus_sha256')!r} is not the registered one")
        if ident.get("rounds") != RUNS:
            bad.append(f"rounds {ident.get('rounds')!r}, not {RUNS}")
        lay = ident.get("layout") or {}
        if (lay.get("idle"), lay.get("sib"), lay.get("decoder")) != (
            IDLE_CPU,
            SIB_CPU,
            DECODER_CPUS,
        ):
            bad.append(f"the layout {lay!r} is not the registered one")
        if ident.get("passes") != [1, 2]:
            bad.append(f"passes {ident.get('passes')!r}, not [1, 2]")
    if ident.get("tree_clean_before") is not True or ident.get("tree_clean_after") is not True:
        bad.append("the checkout was not clean before and after the run")
    for k in ("ab", "windows", "probe", "corpus"):
        a, b = ident.get(f"{k}_sha256"), ident.get(f"{k}_sha256_after")
        if not (hex64(a) and a == b):
            bad.append(f"{k}: its digest before ({a!r}) and after ({b!r}) differ or are missing")
    if (
        ident.get("schedule") != {str(k): v for k, v in SCHEDULE.items()}
        or ident.get("schedule_seed") != SCHEDULE_SEED
    ):
        bad.append("the identity's schedule is not the registered one")
    if (ident.get("arm"), ident.get("threads"), ident.get("settle"), ident.get("sysfs_state")) != (
        ARM,
        THREADS,
        0.25,
        True,
    ):
        bad.append("the arm, threads, settle or sysfs state is not the registered configuration")
    sus = ident.get("sustained") or {}
    want = SUSTAINED_COPIES if not dry else DRY_SUSTAINED_COPIES
    cat = sus.get("concat") or {}
    if sus.get("exit") != 0:
        bad.append(f"the sustained block exited {sus.get('exit')!r}")
    if sus.get("copies") != want or cat.get("copies") != want:
        bad.append(f"the sustained block's copies {sus.get('copies')!r}, not {want}")
    if not (
        cat.get("ok") is True
        and hex64(cat.get("expected_sha256"))
        and cat.get("sha256") == cat.get("expected_sha256") == sus.get("sha256_after")
        and cat.get("corpus_sha256") == ident.get("corpus_sha256")
    ):
        bad.append("the sustained input is not the corpus repeated, unchanged before and after")
    for p in ident.get("passes") or []:
        x = (ident.get("pass_exits") or {}).get(str(p))
        if x != 0:
            bad.append(f"pass {p}'s ab run exited {x!r}" + (" (its deadline)" if x == 124 else ""))
    return bad


def pass_problems(recs, probes, n, pass_no, dry, idle, sib, dec):
    """Each pass's records: the registered count, one probe record a run in order, each inside
    its run, in its schedule slot, its decode exec before its exit, the mask and the probes read
    back, nothing dropped"""
    bad = []
    if len(recs) != n:
        bad.append(f"pass {pass_no}: {len(recs)} runs recorded, not {n}")
    if len(probes) != len(recs):
        bad.append(f"pass {pass_no}: {len(probes)} probe records for {len(recs)} runs")
    prev = None
    for i, (r, p) in enumerate(zip(recs, probes)):
        w = f"pass {pass_no} run {i}"
        if r.get("threads") != THREADS or not str(r.get("arm", "")).startswith(ARM + "@"):
            bad.append(
                f"{w}: {r.get('arm')!r} at {r.get('threads')!r} threads, not {ARM} at {THREADS}"
            )
        if r.get("termination") != "success" or r.get("exit") != 0 or r.get("err"):
            bad.append(f"{w}: did not succeed ({r.get('termination')!r}, {r.get('err')!r})")
        if not dry and (r.get("out"), r.get("hash")) != (WORKLOAD["out"], WORKLOAD["hash"]):
            bad.append(
                f"{w}: output {r.get('out')!r} {r.get('hash')!r}, not the registered workload"
            )
        if num(r.get("wall")) is None:
            bad.append(f"{w}: no wall")
        a, b = r.get("t_spawn_unix_ns"), r.get("t_exit_unix_ns")
        if not (
            isinstance(a, int) and isinstance(b, int) and a < b and (prev is None or prev <= a)
        ):
            bad.append(f"{w}: its spawn and exit ({a}, {b}) are not ordered after the run before")
        prev = b if isinstance(b, int) else prev
        if p.get("schema") != PROBE_SCHEMA or p.get("pass") != pass_no:
            bad.append(f"{w}: the probe record is not pass {pass_no}'s ({p.get('schema')!r})")
            continue
        if p.get("status") != {"exit": 0}:
            bad.append(f"{w}: the decode under the probe ended {p.get('status')!r}")
        t = p.get("t_unix_ns_start")
        if not (isinstance(a, int) and isinstance(b, int) and isinstance(t, int) and a <= t <= b):
            bad.append(
                f"{w}: the probe record does not fall inside the run (spawn {a}, exit {b}, probe {t})"
            )
        ts, mo = p.get("tsc") or {}, p.get("mono_ns") or {}
        if not (
            all(isinstance(ts.get(k), int) for k in ("start", "fork", "exec", "exit", "end"))
            and ts["start"] <= ts["fork"] <= ts["exec"] < ts["exit"] <= ts["end"]
            and all(isinstance(mo.get(k), int) for k in ("start", "exec", "exit", "end"))
            and mo["start"] <= mo["exec"] < mo["exit"] <= mo["end"]
        ):
            bad.append(f"{w}: its timestamps are not ordered start, fork, exec, exit, end")
        slot = SCHEDULE[pass_no][i % len(SCHEDULE[pass_no])] == "P"
        if (
            p.get("run_index") != i
            or p.get("schedule") != SCHEDULE[pass_no]
            or p.get("probed") is not slot
        ):
            bad.append(
                f"{w}: run index {p.get('run_index')!r}, probed {p.get('probed')!r}: not slot {i} "
                f"of the registered schedule"
            )
        if (
            p.get("decoder_cpus") != dec
            or cpu_list(p.get("decoder_cpus_readback")) != cpu_list(dec)
            or cpu_list(p.get("decoder_cpus_status")) != cpu_list(dec)
        ):
            bad.append(
                f"{w}: the decoder's mask read back {p.get('decoder_cpus_readback')!r} / "
                f"{p.get('decoder_cpus_status')!r}, not {dec}"
            )
        want = ([("P-idle", idle)] + ([("P-sib", sib)] if pass_no == 2 else [])) if slot else []
        got = [(q.get("name"), q.get("cpu")) for q in p.get("probes", [])]
        if got != want:
            bad.append(f"{w}: probes {got!r}, not {want!r}")
        for q in p.get("probes", []):
            if not (q.get("pinned") is True and q.get("readback") == str(q.get("cpu"))):
                bad.append(
                    f"{w}: {q.get('name')} not pinned to CPU {q.get('cpu')} (read back {q.get('readback')!r})"
                )
            if q.get("dropped") != 0:
                bad.append(f"{w}: {q.get('name')}'s ring dropped {q.get('dropped')!r} samples")
        if num(p.get("tsc_hz")) is None or p["tsc_hz"] <= 0:
            bad.append(f"{w}: no TSC rate")
    return bad


def persistence(runs):
    seq = "".join("F" if r["fast"] else "s" for r in runs)
    ff = sum(1 for a, b in zip(seq, seq[1:]) if a == "F" and b == "F")
    nf = seq[:-1].count("F")
    return {
        "sequence": seq,
        "fast_after_fast": ff,
        "fast_followed": nf,
        "p_fast_after_fast": ff / nf if nf else None,
        "p_fast": seq.count("F") / len(seq) if seq else None,
    }


# ---- the judge ------------------------------------------------------------------------------


def fmt(x, nd=3):
    if x is None:
        return "-"
    if isinstance(x, float):
        return f"{x:.{nd}f}"
    return str(x)


def markdown(j):
    lines = [f"# The two modes' discriminator: {j['outcome']}", ""]
    lines.append(
        f"Label: **{j['label']}**. Fast: wall < {THRESHOLD_S} s. Diagnostic only; the row "
        "cutoffs are unvalidated descriptive thresholds."
    )
    if j["problems"]:
        lines += ["", "Problems:"] + [f"- {p}" for p in j["problems"]]
    for name in ("pass1", "pass2"):
        pj = j.get(name)
        if not pj:
            continue
        t = pj["table"]
        lines += ["", f"## {name}: {pj['probes']}", ""]
        lines.append("| reading | fast | slow |")
        lines.append("|---|--:|--:|")
        for k in (
            ["n"]
            + TABLE_KEYS
            + ["throttle_rose", "power_limit_rose", "thp_held", "p_idle_rise", "clock_rise"]
        ):
            lines.append(f"| {k} | {fmt(t['fast'].get(k))} | {fmt(t['slow'].get(k))} |")
        lines.append(
            "| observations per row | "
            + ", ".join(f"{k} {v}" for k, v in t["fast"]["coverage"].items())
            + " | "
            + ", ".join(f"{k} {v}" for k, v in t["slow"]["coverage"].items())
            + " |"
        )
        lines += ["", "| row | observed | " + " | ".join(COLUMNS) + " |"]
        lines.append("|---|---|" + "---|" * len(COLUMNS))
        for r in ROWS:
            preds = [
                " or ".join(sorted(COLUMNS[h][1][r]))
                + (" **(decisive)**" if r in COLUMNS[h][2] else "")
                for h in COLUMNS
            ]
            lines.append(f"| {ROW_TEXT[r]} | **{pj['rows'][r]}** | " + " | ".join(preds) + " |")
        lines.append("")
        for h, x in pj["signatures"].items():
            lines.append(
                f"- {h} ({COLUMNS[h][0]}): {x['status']}"
                + (
                    f", {x.get('by') or x.get('missing')}"
                    if x.get("by") or x.get("missing")
                    else ""
                )
            )
        m = pj["mix"]
        lines += [
            "",
            f"Fast runs with the probe: {m['probed'][0]} of {m['probed'][1]}; without it: "
            f"{m['unprobed'][0]} of {m['unprobed'][1]} (a randomised schedule, seed "
            f"{SCHEDULE_SEED}; descriptive, with persistence widening its uncertainty).",
        ]
        p = pj["persistence"]
        lines += ["", f"Run order: `{p['sequence']}`", ""]
        lines.append(
            f"Fast after fast: {p['fast_after_fast']} of {p['fast_followed']} "
            f"({fmt(p['p_fast_after_fast'])}), against {fmt(p['p_fast'])} overall."
        )
        lines.append(f"Gap between runs (ab's exit to next spawn), median: {fmt(pj['gap_s'])} s.")
    su = j.get("sustained") or {}
    lines += ["", "## The sustained block (first, no probe): input consumption", ""]
    lines.append(
        f"Shape: **{su.get('shape')}**; 1 s windows (bytes/s) {[round(x) for x in su.get('rates_bytes_per_s') or []]}, "
        f"error bound {fmt(su.get('error_bytes_per_s'), 0)}, chunk {su.get('chunk_bytes')} bytes."
    )
    if j.get("serial_contrast"):
        lines += ["", "## Pass 2 against pass 1 (serial, not a causal price of a busy sibling)", ""]
        for k, v in j["serial_contrast"].items():
            lines.append(f"- {k}: {fmt(v)}")
    return "\n".join(lines) + "\n"


def judge(args):
    d = args.dir
    dry = args.dry
    problems = []
    ident, why = load_json(os.path.join(d, "identity.json"), "identity")
    if why:
        problems.append(why)
        ident = {}
    problems += admission_problems(ident, d, dry)
    n = RUNS if not dry else ident.get("rounds")
    lay = ident.get("layout") or {}
    idle, sib, dec = (
        (IDLE_CPU, SIB_CPU, DECODER_CPUS)
        if not dry
        else (lay.get("idle"), lay.get("sib"), lay.get("decoder"))
    )
    static, why = load_json(os.path.join(d, "preflight", "static.json"), "static facts")
    problems += [why] if why else []
    siblings = siblings_of(static)
    out = {"schema": JUDGEMENT_SCHEMA, "threshold_s": THRESHOLD_S, "dry": dry}
    for pass_no in (1, 2):
        name = f"pass{pass_no}"
        if pass_no == 2 and sib is None:
            out[name] = None
            if not dry:
                problems.append("no P-sib CPU: pass 2 did not run")
            continue
        recs, why1 = load_jsonl(os.path.join(d, name, "runs.jsonl"), f"{name} runs")
        probes, why2 = load_jsonl(os.path.join(d, name, "probe.jsonl"), f"{name} probes")
        if why1 or why2:
            problems += [w for w in (why1, why2) if w]
            out[name] = None
            continue
        problems += pass_problems(recs, probes, n, pass_no, dry, idle, sib, dec)
        runs = []
        for i, (r, p) in enumerate(zip(recs, probes)):
            try:
                x = readings(r, p, siblings)
            except (KeyError, TypeError, ValueError, ZeroDivisionError) as e:
                problems.append(
                    f"{name} run {i}: its readings do not compute: {type(e).__name__}: {e}"
                )
                continue
            x["round"] = r.get("round")
            runs.append(x)
        gaps = [
            (b["t_spawn_unix_ns"] - a["t_exit_unix_ns"]) / 1e9
            for a, b in zip(recs, recs[1:])
            if isinstance(a.get("t_exit_unix_ns"), int)
            and isinstance(b.get("t_spawn_unix_ns"), int)
        ]
        probed = [r for r in runs if r["probed"]]
        unprobed = [r for r in runs if not r["probed"]]
        table = class_table(probed)
        values = row_values(table)
        out[name] = {
            "probes": "P-idle alone" if pass_no == 1 else "P-idle beside P-sib",
            "schedule": SCHEDULE[pass_no],
            "table": table,
            "rows": values,
            "signatures": signatures(values),
            "unprobed_table": class_table(unprobed) if unprobed else None,
            "mix": {
                "probed": [sum(r["fast"] for r in probed), len(probed)],
                "unprobed": [sum(r["fast"] for r in unprobed), len(unprobed)],
            },
            "persistence": persistence(runs),
            "gap_s": med(gaps),
            "runs": runs,
        }
    p1, p2 = out.get("pass1"), out.get("pass2")
    if p1 and p2:
        a, b = p1["table"], p2["table"]
        out["serial_contrast"] = {
            "P-idle progress, pass 2 over pass 1, all probed runs": ratio(
                med(r.get("p_idle") for r in p2["runs"]),
                med(r.get("p_idle") for r in p1["runs"] if r["probed"]),
            ),
            "fast runs": ratio(b["fast"]["p_idle"], a["fast"]["p_idle"]),
            "slow runs": ratio(b["slow"]["p_idle"], a["slow"]["p_idle"]),
            "P-sib over P-idle, pass 2": ratio(
                med(r.get("p_sib") for r in p2["runs"]), med(r.get("p_idle") for r in p2["runs"])
            ),
        }
    result = outcome(p1["table"], p1["rows"]) if p1 else "no pass 1 table"
    srec, why = load_json(os.path.join(d, "sustained", "sustained.json"), "the sustained block")
    child, why2 = load_json(os.path.join(d, "sustained", "child.json"), "the sustained decode")
    copies = (ident.get("sustained") or {}).get("copies")
    if why or why2:
        problems += [w for w in (why, why2) if w]
        sus = {"shape": "unread"}
    else:
        if srec.get("schema") != SUSTAINED_SCHEMA or srec.get("status") != {"exit": 0}:
            problems.append(f"the sustained block ended {srec.get('status')!r}")
        if (
            not isinstance(child, dict)
            or child.get("err")
            or (isinstance(copies, int) and child.get("out") != copies * WORKLOAD["out"])
        ):
            problems.append(
                f"the sustained decode did not decode {copies!r} corpora ({str(child)[:120]})"
            )
        sus = sustained_reading(srec)
        if isinstance(copies, int) and num(srec.get("span_s")):
            sus["decoded_bytes_per_s_end_to_end"] = copies * WORKLOAD["out"] / srec["span_s"]
    out["sustained"] = sus
    result += f"; sustained block: {sus.get('shape')}"
    out["result"] = result
    out["problems"] = problems
    if dry:
        out["label"] = "DRY"
        first = f"DRY (not evidence): {result}" + (
            f"; problems: {'; '.join(problems)}" if problems else ""
        )
        rc = 0
    elif problems:
        out["label"] = "INVALID"
        first = (
            "INVALID: "
            + "; ".join(problems[:8])
            + (f" (and {len(problems) - 8} more)" if len(problems) > 8 else "")
        )
        rc = 2
    else:
        out["label"] = "VALID"
        first = f"VALID: {result}"
        rc = 0
    out["outcome"] = first
    write(out, os.path.join(d, "judgement"), markdown(out))
    print(first)
    if p1:
        for r in ROWS:
            print(f"  {ROW_TEXT[r]}: {p1['rows'][r]}")
    return rc


# ---- the command line -----------------------------------------------------------------------


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("static")
    s.add_argument("--out", required=True)
    s = sub.add_parser("layout")
    s.add_argument("env")
    s.add_argument("--dry", action="store_true")
    s = sub.add_parser("concat")
    s.add_argument("corpus")
    s.add_argument("copies", type=int)
    s.add_argument("out")
    s.add_argument("--record", required=True)
    s = sub.add_parser("digest")
    s.add_argument("file")
    s = sub.add_parser("get")
    s.add_argument("name")
    s = sub.add_parser("preflight")
    s.add_argument("dir")
    s = sub.add_parser("judge")
    s.add_argument("dir")
    s.add_argument("--dry", action="store_true")
    args = ap.parse_args(argv)
    if args.cmd == "static":
        with open(args.out, "w") as f:
            json.dump(static_facts(), f, indent=1)
        return 0
    if args.cmd == "get":  # (a registered constant, for the recipe: JSON for a dict)
        v = globals()[args.name]
        print(json.dumps({str(k): x for k, x in v.items()}) if isinstance(v, dict) else v)
        return 0
    if args.cmd == "digest":
        print(sha256_file(args.file))
        return 0
    if args.cmd == "concat":
        rec = concat(args.corpus, args.copies, args.out)
        with open(args.record, "w") as f:
            json.dump(rec, f, indent=1)
        print(json.dumps(rec))
        return 0 if rec["ok"] else 1
    if args.cmd == "layout":
        env, why = load_json(args.env, "ab env")
        if why:
            print(why, file=sys.stderr)
            return 1
        try:
            idle, sib, dec = layout(env, args.dry)
        except ValueError as e:
            print(f"layout: {e}", file=sys.stderr)
            return 1
        print(idle, "-" if sib is None else sib, dec)
        return 0
    if args.cmd == "preflight":
        return preflight(args)
    try:
        return judge(args)
    except Exception as e:  # (the judge's own failure is INVALID, never a crash or a VALID)
        print(f"INVALID: the judge failed: {type(e).__name__}: {e}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
