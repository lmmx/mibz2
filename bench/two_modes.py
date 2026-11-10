"""The two modes' discriminator: its static facts, its preflight and its analysis
(docs/journal/2026-11-10-two-modes-discriminator.md, the registration; the design is §3 of
docs/journal/2026-11-10-two-modes.md). Diagnostic only: no merge gate reads it.

    python3 bench/two_modes.py static --out FILE
    python3 bench/two_modes.py layout ENV_JSON [--dry]
    python3 bench/two_modes.py preflight DIR
    python3 bench/two_modes.py judge DIR [--dry]

`static` reads, once, the facts the run does not change: the transparent huge page settings,
the RAPL constraints under /sys/class/powercap where readable, each CPU's topology, TSC flags
and thermal_throttle files, intel_pstate's and the first cpufreq policy's settings, and the
kernel. Every unreadable file is recorded with its error, never skipped.

`layout` prints the CPUs the run uses on one line: P-idle's, P-sib's ("-" for none) and the
decoder's mask. Registered:
CPU 9 (P-idle), 19 (P-sib, its sibling) and 0-8,10-18 (the decoder), the i9-10900K's last
physical core taken out of the decoder's mask. With --dry, the last physical core of the
machine at hand instead, so the recipe's own check runs anywhere.

`preflight` judges what the recipe collected before any run (DIR/preflight/): the registered
host with its declared settings, every required collector completed (exits.txt), an invariant
TSC on every CPU, the registered probe core and its sibling, the throttle counts readable, and
the probe's selftest pinned to P-idle's CPU and read back. PASS or REFUSED, one pass.

`judge` reads DIR/pass1 and DIR/pass2 (each runs.jsonl, from `ab run`, and probe.jsonl, from
bench/two_modes_probe.c), checks their structure, and writes the §3 table: each run classed
fast (wall < 0.21 s) or slow, each reading's median by class, each row's value by the fixed
rules below, and the column (H1 to H5) whose predictions every readable row meets. Pass 1
(P-idle alone) is matched; pass 2 (P-idle beside P-sib) prices a busy sibling and is reported.

Exit status: 0 VALID or DRY (the outcome on the first line), 2 INVALID, 1 a preflight REFUSED;
a failure of this script itself is INVALID with its reason, never a crash and never a VALID.
"""

import argparse
import glob
import json
import math
import os
import re
import statistics as st
import sys

# ---- the registration (fixed before data) -------------------------------------------------

PROBE_SCHEMA = "two_modes/probe/1"
STATIC_SCHEMA = "two_modes/static/1"
JUDGEMENT_SCHEMA = "two_modes/judgement/1"
PREFLIGHT_SCHEMA = "two_modes/preflight/1"

THRESHOLD_S = 0.21  # fast: the child's own wall below it (fixed before data)
RUNS = 60  # each pass
# each pass's fixed schedule, repeating from its first run: P probed, N no probe (the same
# mask, the probe core left idle). Pass 1: 40 probed and 20 not, interleaved, so the probe's
# own effect on the fast/slow mix can be read (the Optimiser's REVIEW 6756418444, note 1)
SCHEDULE = {1: "PPN", 2: "P"}
DRY_RUNS = 3
ARM = "mibz2-default33"  # the default plan, as the run-fed leader's bundle ran it
THREADS = 10
MIN_CLASS = 5  # fewer fast or slow probed runs than this in pass 1: ONE MODE, nothing matched
SUPPORTING = {"clock"}  # scaling_cur_freq: a supporting reading, never decisive alone
MIN_PROBE_SAMPLES = 10  # P-idle samples inside a run's decode, at least
DECODER_PREFIX = "mibz2-"  # the decoder's threads, by the names they give themselves

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

# The rows' rules. Each compares slow runs with fast runs in pass 1, by medians over runs.
FULL = 0.95  # P-idle's rate or the decoder CPUs' clock, slow / fast: at least this is full
HALF = (0.40, 0.65)  # P-idle's rate, slow / fast, in this interval: half
RISING = 0.85  # early third / late third of a run, median over slow runs: below it, rising
MOST = 0.50  # busy-sibling share in slow runs: at least this, and
MOST_GAIN = 0.15  # at least this much above the fast runs': most
SAME = 0.10  # busy-sibling shares within this of each other: same
RISE_YES = (0.50, 0.25)  # a count rose in >= 50% of slow runs and 25 points more than fast: yes
RISE_NO = 0.10  # the share of slow runs with a rise exceeds the fast share by less: no
THP_YES = 0.50  # thp_fault_alloc rose in this much more of fast runs than slow: yes
THP_NO = 0.25  # within this of each other: no
SYS_YES = 0.05  # system time's share of CPU, slow minus fast, at least: yes
SYS_NO = 0.02  # below: no

ROWS = ["p_idle", "clock", "sibling", "throttle", "power_limit", "thp", "sys_share"]
ROW_TEXT = {
    "p_idle": "P-idle's rate in slow runs",
    "clock": "the decoder CPUs' clock during slow runs",
    "sibling": "threads with a busy sibling",
    "throttle": "throttle counts rise",
    "power_limit": "power-limit counts rise",
    "thp": "thp_fault_alloc in fast runs only",
    "sys_share": "system time's share grows",
}
# each column's prediction for each row: the set of values it admits
COLUMNS = {
    "H1": (
        "the governor's ramp",
        {
            # (P-idle starts with the run: on cores that share one clock it ramps with the
            # decoder's, so H1 admits it full or rising; the registration says why)
            "p_idle": {"full", "rising"},
            "clock": {"low, then rising"},
            "sibling": {"same"},
            "throttle": {"no"},
            "power_limit": {"no"},
            "thp": {"no"},
            "sys_share": {"no"},
        },
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
    ),
}
UNREAD = "unread"
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


def probe_rates(probe, t0, t1, hz):
    """P-x's per-sample rates (Giter/s) whose interval lies inside [t0, t1] (TSC), with each
    interval's end, in order"""
    if not probe or probe.get("first_tsc") is None:
        return []
    k = probe.get("iters_per_sample")
    t = probe["first_tsc"]
    out = []
    for d in probe.get("dtsc", []):
        a, t = t, t + d
        if d > 0 and a >= t0 and t <= t1:
            out.append((t, k * hz / d / 1e9))
    return out


def thirds(pairs, t0, t1):
    """The medians of values in the first and last thirds of [t0, t1], by each pair's time"""
    span = t1 - t0
    early = [v for t, v in pairs if t <= t0 + span / 3]
    late = [v for t, v in pairs if t >= t0 + 2 * span / 3]
    return med(early), med(late)


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
    """One run's readings: its wall and class, the probes' rates during the decode, the
    decoder CPUs' clock, its threads' busy-sibling share, the counters' rises and its paging
    and CPU split"""
    hz = probe["tsc_hz"]
    t0, t1 = probe["tsc"]["exec"], probe["tsc"]["exit"]
    span_us = (probe["mono_ns"]["exit"] - probe["mono_ns"]["exec"]) / 1000
    out = {"wall": rec.get("wall"), "cpu": rec.get("cpu")}
    out["fast"] = num(rec.get("wall")) is not None and rec["wall"] < THRESHOLD_S
    for p in probe.get("probes", []):
        key = {"P-idle": "p_idle", "P-sib": "p_sib"}.get(p.get("name"))
        if not key:
            continue
        rates = probe_rates(p, t0, t1, hz)
        out[key + "_n"] = len(rates)
        out[key] = med([v for _, v in rates])
        out[key + "_early"], out[key + "_late"] = thirds(rates, t0, t1)
    # placement: the decoder's threads, by name
    comm = probe["placement"].get("comm", {})
    dec_tids = {int(t) for t, n in comm.items() if str(n).startswith(DECODER_PREFIX)}
    seen = set()
    shares, shares_r = [0, 0], [0, 0]
    for t_us, tasks in probe["placement"].get("samples", []):
        dec = [(tid, cpu, s) for tid, cpu, s in tasks if tid in dec_tids]
        seen.update(cpu for _, cpu, _ in dec)
        for tid, cpu, s in dec:
            sib = siblings.get(cpu, set())
            busy = any(c in sib for t2, c, _ in dec if t2 != tid)
            shares[0] += busy
            shares[1] += 1
            if s == "R":
                busy_r = any(c in sib for t2, c, s2 in dec if t2 != tid and s2 == "R")
                shares_r[0] += busy_r
                shares_r[1] += 1
    out["placement_samples"] = len(probe["placement"].get("samples", []))
    out["sibling_share"] = shares[0] / shares[1] if shares[1] else None
    out["sibling_share_running"] = shares_r[0] / shares_r[1] if shares_r[1] else None
    out["decoder_cpus_seen"] = sorted(seen)
    # the clock of the CPUs the decoder's threads were seen on, sample by sample
    clk, read_us = [], []
    for t_us, r_us, khz in probe["freq"].get("samples", []):
        read_us.append(r_us)
        if 0 <= t_us <= span_us:
            v = med([khz[c] for c in seen if c < len(khz) and khz[c] is not None])
            if v is not None:
                clk.append((t_us, v / 1e6))
    out["clock_ghz"] = med([v for _, v in clk])
    out["clock_early"], out["clock_late"] = thirds(clk, 0, span_us)
    out["freq_samples"] = len(clk)
    out["freq_read_us"] = med(read_us)
    b, a = probe.get("before"), probe.get("after")
    out["core_throttle"] = throttle_delta(b, a, "core_throttle_count", "sum")
    out["package_throttle"] = throttle_delta(b, a, "package_throttle_count", "max")
    out["core_throttle_ms"] = throttle_delta(b, a, "core_throttle_total_time_ms", "sum")
    out["package_throttle_ms"] = throttle_delta(b, a, "package_throttle_total_time_ms", "max")
    out["core_power_limit"] = throttle_delta(b, a, "core_power_limit_count", "sum")
    out["package_power_limit"] = throttle_delta(b, a, "package_power_limit_count", "max")
    out["thp_fault_alloc"] = vm_delta(b, a, "thp_fault_alloc")
    out["thp_fault_fallback"] = vm_delta(b, a, "thp_fault_fallback")
    ru = probe.get("rusage") or {}
    out["minflt"] = ru.get("minflt")
    out["utime_s"] = ru.get("utime_s")
    out["stime_s"] = ru.get("stime_s")
    tot = (ru.get("utime_s") or 0) + (ru.get("stime_s") or 0)
    out["sys_share"] = ru["stime_s"] / tot if tot and num(ru.get("stime_s")) is not None else None
    sm = probe.get("smaps") or {}
    out["anon_huge_kb"] = sm.get("anon_huge_kb_max") if (sm.get("reads") or 0) > 0 else None
    out["probed"] = probe.get("probed") is not False
    return out


def rose(r, a, b):
    """1 if either count rose, 0 if neither, None if both are unread"""
    xs = [r.get(a), r.get(b)]
    if all(x is None for x in xs):
        return None
    return int(any(x is not None and x > 0 for x in xs))


# ---- the class table and its rows ---------------------------------------------------------

TABLE_KEYS = [
    "wall",
    "cpu",
    "p_idle",
    "p_idle_early",
    "p_idle_late",
    "p_sib",
    "clock_ghz",
    "clock_early",
    "clock_late",
    "sibling_share",
    "sibling_share_running",
    "core_throttle",
    "package_throttle",
    "core_throttle_ms",
    "package_throttle_ms",
    "core_power_limit",
    "package_power_limit",
    "thp_fault_alloc",
    "thp_fault_fallback",
    "minflt",
    "utime_s",
    "stime_s",
    "sys_share",
    "anon_huge_kb",
]


def frac(xs):
    xs = [x for x in xs if x is not None]
    return sum(xs) / len(xs) if xs else None


def class_table(runs):
    out = {}
    for name, fast in (("fast", True), ("slow", False)):
        g = [r for r in runs if r["fast"] == fast]
        row = {"n": len(g)}
        for k in TABLE_KEYS:
            row[k] = med(r.get(k) for r in g)
        row["throttle_rose"] = frac(rose(r, "core_throttle", "package_throttle") for r in g)
        row["power_limit_rose"] = frac(
            rose(r, "core_power_limit", "package_power_limit") for r in g
        )
        row["thp_rose"] = frac(
            None if r.get("thp_fault_alloc") is None else int(r["thp_fault_alloc"] > 0) for r in g
        )
        row["p_idle_rise"] = med(ratio(r.get("p_idle_early"), r.get("p_idle_late")) for r in g)
        row["clock_rise"] = med(ratio(r.get("clock_early"), r.get("clock_late")) for r in g)
        out[name] = row
    return out


def row_values(t):
    """Each row's value from the class table, by the registered rules (None-safe: a row
    whose readings are missing is UNREAD)"""
    f, s = t["fast"], t["slow"]
    v = {}
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
    rc = ratio(s["clock_ghz"], f["clock_ghz"])
    if rc is None:
        v["clock"] = UNREAD
    elif rc >= FULL:
        v["clock"] = "full"
    elif s["clock_rise"] is not None and s["clock_rise"] < RISING:
        v["clock"] = "low, then rising"
    else:
        v["clock"] = "reduced"
    a, b = s["sibling_share"], f["sibling_share"]
    if a is None or b is None:
        v["sibling"] = UNREAD
    elif a >= MOST and a - b >= MOST_GAIN:
        v["sibling"] = "most"
    elif abs(a - b) < SAME:
        v["sibling"] = "same"
    else:
        v["sibling"] = "between"
    for row, key in (("throttle", "throttle_rose"), ("power_limit", "power_limit_rose")):
        a, b = s[key], f[key]
        if a is None or b is None:
            v[row] = UNREAD
        elif a >= RISE_YES[0] and a - b >= RISE_YES[1]:
            v[row] = "yes"
        elif a - b < RISE_NO:
            v[row] = "no"
        else:
            v[row] = "between"
    a, b = f["thp_rose"], s["thp_rose"]
    if a is None or b is None:
        v["thp"] = UNREAD
    elif a - b >= THP_YES:
        v["thp"] = "yes"
    elif abs(a - b) < THP_NO:
        v["thp"] = "no"
    else:
        v["thp"] = "between"
    a, b = s["sys_share"], f["sys_share"]
    if a is None or b is None:
        v["sys_share"] = UNREAD
    elif a - b >= SYS_YES:
        v["sys_share"] = "yes"
    elif a - b < SYS_NO:
        v["sys_share"] = "no"
    else:
        v["sys_share"] = "between"
    return v


def match(values):
    """The columns every readable row of which admits the observed value, and the rows unread"""
    unread = [r for r in ROWS if values[r] == UNREAD]
    hits = [
        h
        for h, (_, pred) in COLUMNS.items()
        if all(r in unread or values[r] in pred[r] for r in ROWS)
    ]
    return hits, unread


def outcome(table, values):
    f, s = table["fast"]["n"], table["slow"]["n"]
    if f < MIN_CLASS or s < MIN_CLASS:
        return (
            f"ONE MODE: {f} fast and {s} slow runs (fewer than {MIN_CLASS} in a class); "
            "the table is described, no column matched"
        )
    hits, unread = match(values)
    note = f" ({', '.join(ROW_TEXT[r] for r in unread)} unread)" if unread else ""
    # (a match rests on the supporting rows when, without them, its column would also admit
    # a slow mode that differs from the fast in nothing: the clock is then all that names it)
    rest = [
        h for h in hits if all(r in SUPPORTING or BASELINE[r] in COLUMNS[h][1][r] for r in ROWS)
    ]
    if rest:
        note += f" (resting on the supporting reading: {', '.join(ROW_TEXT[r] for r in sorted(SUPPORTING))})"
    if len(hits) == 1:
        return f"MATCH {hits[0]} ({COLUMNS[hits[0]][0]}){note}"
    if hits:
        return f"AMBIGUOUS: {', '.join(hits)} all match{note}"
    desc = "; ".join(f"{ROW_TEXT[r]}: {values[r]}" for r in ROWS)
    return f"SIXTH MECHANISM: no column matches; {desc}"


def persistence(runs):
    seq = "".join("F" if r["fast"] else "s" for r in runs)
    ff = sum(1 for a, b in zip(seq, seq[1:]) if a == "F" and b == "F")
    nf = seq[:-1].count("F")
    share = seq.count("F") / len(seq) if seq else None
    return {
        "sequence": seq,
        "fast_after_fast": ff,
        "fast_followed": nf,
        "p_fast_after_fast": ff / nf if nf else None,
        "p_fast": share,
    }


# ---- the judge ------------------------------------------------------------------------------


def pass_problems(recs, probes, n, pass_no, dry, idle, sib, dec):
    bad = []
    if len(recs) != n:
        bad.append(f"pass {pass_no}: {len(recs)} runs recorded, not {n}")
    if len(probes) != len(recs):
        bad.append(f"pass {pass_no}: {len(probes)} probe records for {len(recs)} runs")
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
        if p.get("schema") != PROBE_SCHEMA or p.get("pass") != pass_no:
            bad.append(f"{w}: the probe record is not pass {pass_no}'s ({p.get('schema')!r})")
            continue
        if p.get("status") != {"exit": 0}:
            bad.append(f"{w}: the decode under the probe ended {p.get('status')!r}")
        a, b, t = r.get("t_spawn_unix_ns"), r.get("t_exit_unix_ns"), p.get("t_unix_ns_start")
        if not (isinstance(a, int) and isinstance(b, int) and isinstance(t, int) and a <= t <= b):
            bad.append(
                f"{w}: the probe record does not fall inside the run (spawn {a}, exit {b}, probe {t})"
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
        slot = SCHEDULE[pass_no][i % len(SCHEDULE[pass_no])] == "P"
        if (
            p.get("run_index") != i
            or p.get("schedule") != SCHEDULE[pass_no]
            or p.get("probed") is not slot
        ):
            bad.append(
                f"{w}: run index {p.get('run_index')!r}, schedule {p.get('schedule')!r}, probed "
                f"{p.get('probed')!r}: not slot {i} of {SCHEDULE[pass_no]!r}"
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


def fmt(x, nd=3):
    if x is None:
        return "-"
    if isinstance(x, float):
        return f"{x:.{nd}f}"
    return str(x)


def markdown(j):
    lines = [f"# The two modes' discriminator: {j['outcome']}", ""]
    lines.append(f"Label: **{j['label']}**. Fast: wall < {THRESHOLD_S} s. Diagnostic only.")
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
            + ["throttle_rose", "power_limit_rose", "thp_rose", "p_idle_rise", "clock_rise"]
        ):
            lines.append(f"| {k} | {fmt(t['fast'].get(k))} | {fmt(t['slow'].get(k))} |")
        lines += ["", "| row | observed | " + " | ".join(COLUMNS) + " |"]
        lines.append("|---|---|" + "---|" * len(COLUMNS))
        for r in ROWS:
            preds = [" or ".join(sorted(COLUMNS[h][1][r])) for h in COLUMNS]
            lines.append(f"| {ROW_TEXT[r]} | **{pj['rows'][r]}** | " + " | ".join(preds) + " |")
        m = pj["mix"]
        lines += [
            "",
            f"Fast runs with the probe: {m['probed'][0]} of {m['probed'][1]}; without it "
            f"(schedule {pj['schedule']!r}): {m['unprobed'][0]} of {m['unprobed'][1]}.",
            "The clock row (scaling_cur_freq) is a supporting reading, not a decisive one.",
        ]
        p = pj["persistence"]
        lines += ["", f"Run order: `{p['sequence']}`", ""]
        lines.append(
            f"Fast after fast: {p['fast_after_fast']} of {p['fast_followed']} "
            f"({fmt(p['p_fast_after_fast'])}), against {fmt(p['p_fast'])} overall."
        )
        lines.append(f"Gap between runs (ab's exit to next spawn), median: {fmt(pj['gap_s'])} s.")
    if j.get("sibling_cost"):
        c = j["sibling_cost"]
        lines += ["", "## A busy sibling's price (pass 2 over pass 1, P-idle's rate)", ""]
        for k, v in c.items():
            lines.append(f"- {k}: {fmt(v)}")
    return "\n".join(lines) + "\n"


def judge(args):
    d = args.dir
    problems = []
    ident, why = load_json(os.path.join(d, "identity.json"), "identity")
    if why:
        problems.append(why)
        ident = {}
    dry = args.dry
    if not dry and ident.get("dry"):
        problems.append("the identity says DRY: judge it with --dry")
    n = ident.get("rounds") if dry else RUNS
    if not dry and ident.get("rounds") != RUNS:
        problems.append(f"the identity's rounds {ident.get('rounds')!r}, not {RUNS}")
    lay = ident.get("layout") or {}
    idle, sib, dec = (
        (IDLE_CPU, SIB_CPU, DECODER_CPUS)
        if not dry
        else (lay.get("idle"), lay.get("sib"), lay.get("decoder"))
    )
    if not dry and (lay.get("idle"), lay.get("sib"), lay.get("decoder")) != (
        IDLE_CPU,
        SIB_CPU,
        DECODER_CPUS,
    ):
        problems.append(f"the identity's layout {lay!r} is not the registered one")
    if not dry:
        for k in ("ab_sha256", "windows_sha256"):
            if ident.get(k) != SOURCE[k]:
                problems.append(
                    f"the identity's {k} {ident.get(k)!r} is not the archived harness's"
                )
    static, why = load_json(os.path.join(d, "preflight", "static.json"), "static facts")
    problems += [why] if why else []
    siblings = siblings_of(static)
    out = {"schema": JUDGEMENT_SCHEMA, "threshold_s": THRESHOLD_S, "dry": dry}
    for pass_no in (1, 2):
        name = f"pass{pass_no}"
        if pass_no == 2 and sib is None:
            if dry:
                out[name] = None  # (no sibling on this machine: a DRY check's pass 2 is skipped)
                continue
            problems.append("no P-sib CPU: pass 2 did not run")
            continue
        recs, why1 = load_jsonl(os.path.join(d, name, "runs.jsonl"), f"{name} runs")
        probes, why2 = load_jsonl(os.path.join(d, name, "probe.jsonl"), f"{name} probes")
        if why1 or why2:
            problems += [w for w in (why1, why2) if w]
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
            if x["probed"] and (x.get("p_idle_n") or 0) < MIN_PROBE_SAMPLES:
                problems.append(
                    f"{name} run {i}: {x.get('p_idle_n')} P-idle samples inside the decode"
                )
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
            # (the probe's own effect on the mix: the same pass's runs without it, described)
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
        out["sibling_cost"] = {
            "all runs": ratio(
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
