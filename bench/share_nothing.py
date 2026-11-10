"""Share-nothing pricing: plain independent workers on Wikidata recompressed at lbzip2 -2 and
-9, at 2 to 10 workers, read as CPU-seconds and wall (docs/journal/2026-11-10-share-nothing-pricing.md,
the registration; the idea is docs/journal/2026-11-10-share-nothing.md §4, falsifier 2).
Diagnostic only: no merge gate reads it.

    python3 bench/share_nothing.py schedule [--dry] --out FILE
    python3 bench/share_nothing.py corpora CORPUS OUTDIR --record JSON
    python3 bench/share_nothing.py static --out FILE
    python3 bench/share_nothing.py preflight DIR
    python3 bench/share_nothing.py drive DIR AB [--dry] [--max-busy C]
    python3 bench/share_nothing.py identity DIR KEY=TEXT... KEY:=JSON...
    python3 bench/share_nothing.py judge DIR [--dry]
    python3 bench/share_nothing.py get NAME

`schedule` writes the registered run order: a warm-up round, then the measured rounds. Each
round is the sentinel (the default plan at 10 threads on the bench's corpus), the ten cells
(mibz2-workers on wikidata-l2 and wikidata-l9 at 2, 4, 6, 8 and 10 workers) in an order drawn
for that round from the registered seed, then the sentinel again. The schedule is a function
of the registered constants alone; its digest is registered (SCHEDULE_SHA256) and the tests
check it.

`corpora` decodes the bench's corpus with bzip2 (libbzip2, not lbzip2) and checks the decoded
bytes' SHA-256 against the registered one; recompresses them with lbzip2 at each level, pinned
to LBZIP2_THREADS threads, and again at one thread, recording whether the thread count changed
the output; decodes each result with bzip2 and checks its digest equals the original's. It
records lbzip2's version, the flags, each file's size and SHA-256, and whether they match the
digests registered for lbzip2 2.5 (a different version is recorded, never refused). Any failed
command or digest mismatch: exit 1.

`static` reads, once and passively (procfs, sysfs and cgroupfs only), the host facts the
preflight needs. `preflight` judges what the recipe collected before any run (DIR/preflight/):
every registered collector completed (exits.txt), the i9-10900K, its declared cpufreq settings
(intel_pstate, powersave, balance_performance, turbo on) in the static facts and in three sysfs
samples, every CPU online and allowed, and no CPU quota established (cgroup_quota: unknown
refuses). PASS or REFUSED.

`drive` runs the schedule: one `ab run` a slot (one arm, one thread count, one file, one round,
--settle 0.25, --max-busy 1.5, --sysfs-state), each under bench/bounded.py's deadline
(CELL_DEADLINE), appending to DIR/runs.jsonl, with one line a slot in DIR/slots.jsonl. It stops
at the first slot that fails or passes its deadline: no retry. Exit 0 if every slot ran, 1 if
one failed, 124 if one passed its deadline, 125 if a slot's group could not be emptied.

`judge` applies the admission checks, failing closed on missing evidence (the identity with its
head, frozen commit and sources; the corpora's digests present on both sides; the archived
preflight replayed; every slot as scheduled with exactly one run line; every run's bytes and hash
the corpus's; every run's telemetry a value or null with its reason, and its times in order),
then the registered analysis: the growth from 4 to 10 workers in CPU-seconds and wall for each
corpus and their ratio of ratios (the level x worker interaction), each a geometric mean of
per-round paired ratios with a moving-block bootstrap interval over the chronological rounds
(block 5; 1 and 10 as sensitivity), conditional on the attempt and that dependence treatment;
the -2 growth against the historical band 1.10-1.44, the -9 growth against the historical 3.50
(compatibility only), R classed favourable, contrary or INCONCLUSIVE; the grid, the waits, the
pre-run state with its missingness, the chronological rounds, and a descriptive split by the
sentinels' walls (point estimates only). Exit 0 VALID or DRY, 2 INVALID (the outcome on the first
line); a failure of the judge itself is INVALID with its reason, never a VALID.
"""

import argparse
import functools
import glob
import hashlib
import json
import math
import os
import random
import re
import statistics as st
import subprocess
import sys

# ---- the registration (fixed before data) -------------------------------------------------

SCHEMA = {
    "schedule": "share_nothing/schedule/1",
    "corpora": "share_nothing/corpora/1",
    "static": "share_nothing/static/1",
    "preflight": "share_nothing/preflight/1",
    "slot": "share_nothing/slot/1",
    "identity": "share_nothing/identity/2",
    "judgement": "share_nothing/judgement/2",
}

ARM = "mibz2-workers"  # THREADS workers of a block each (Plan::Workers), today's decoder
WORKERS = (2, 4, 6, 8, 10)
REF, TOP = 4, 10  # the growth is TOP over REF
LEVELS = {"wikidata-l2": 2, "wikidata-l9": 9}  # the cells' corpora: file stem -> lbzip2 level
CORPORA = tuple(LEVELS)
PRIMARY, COMPARATOR = "wikidata-l2", "wikidata-l9"
# the sentinel: the arm and condition in which the two modes were recorded (the default plan
# at 10 threads on the bench's corpus), first and last in every round
SENTINEL = {"arm": "mibz2", "threads": 10, "corpus": "wikidata"}
LBZIP2_THREADS = 4  # lbzip2 -n, pinned
WARMUP_ROUNDS = 1  # recorded, never analysed
ROUNDS = 60  # measured rounds
DRY_ROUNDS = 2
SCHEDULE_SEED = 202611102
SETTLE_S = 0.25  # ab run --settle, as the run-fed leader's harness ran it
MAX_BUSY = 1.5  # ab run --max-busy, its default, passed explicitly
CELL_DEADLINE = (120, 10)  # one slot's `ab run`, its busy-gate waits included: (secs, grace)

CONTENT_BYTES = 209700196
CONTENT_SHA256 = "3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf"
CORPUS_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
WORKLOAD_HASH = "a8830caee5c511a6"  # ab's hash of the decoded bytes
# lbzip2 2.5 (Ubuntu 2.5-2.3) on the dev VM: -N -n 4 and -N -n 1 gave these, byte-identical
REGISTERED_LBZIP2 = "lbzip2 version 2.5"
REGISTERED_BZ2 = {
    "wikidata-l2": {
        "sha256": "d3bcb14179480e0d811fa2dfa73c20560808252a9199a2679ff14c84a9323045",
        "bytes": 15837840,
    },
    "wikidata-l9": {
        "sha256": "3241376ca757f2fe786fcd07cf28012a6364eb523c821c10a9dc405fec120150",
        "bytes": 11855421,
    },
}

# §2 of the share-nothing entry (2026-10-28-workstation-scaling.md §9): CPU-seconds, 4 -> 10
# plain workers, an earlier decoder
BAND = (1.10, 1.44)  # -1 (0.4 MB a block) to -3 (1.2 MB a block)
S2_L9 = 3.50
# §3's model: ten workers at one thread's 400 MB/s at the all-core clock (0.94), divided by
# the growth factor: 3.76 GB/s / G. MODEL ONLY: the judge restates it with the measured -2
# growth and labels it so; nothing here observes it
MODEL_GB_S_NUMERATOR = 10 * 0.400 * 0.94
MODE_THRESHOLD_S = 0.21  # a sentinel's own wall below it: fast (two_modes.py's separator)
MIN_CLASS = 5  # rounds a mode class needs before its figures are given an interval
BOOT_RESAMPLES = 10000
BOOT_BLOCK = 5  # rounds: the primary dependence treatment
BOOT_SENSITIVITY = (1, 10)
BOOT_SEED = 20261110

HOST_MODEL = "i9-10900K"
HOST_CPUS = 20
DECLARED = {
    "cpufreq_driver": "intel_pstate",
    "governor": "powersave",
    "epp": "balance_performance",
    "turbo": True,
}
PREFLIGHT_SAMPLES = 3
PREFLIGHT_COLLECTORS = ("lscpu", "ab sample", "static facts")

# the sources whose digests the identity records
SOURCES = [
    "bench/share_nothing.py",
    "bench/bounded.py",
    "bench/par_scan_finish.sh",
    ".just/share-nothing.just",
    "docs/journal/2026-11-10-share-nothing-pricing.md",
]

# the stages' deadlines (bench/bounded.py: secs, grace), recorded in the identity
DEADLINES = {
    "fetch": (300, 30),
    "build": (3600, 30),
    "corpus": (1800, 30),
    "corpora": (1800, 30),
    "drive": (5400, 30),
    "judge": (600, 30),
    "bundle": (600, 30),
}


def _schedule(seed, rounds):
    rng = random.Random(seed)
    cells = [(c, w) for c in CORPORA for w in WORKERS]
    out = []
    for r in range(rounds):
        order = cells[:]
        rng.shuffle(order)
        slots = [dict(SENTINEL, role="sentinel")]
        slots += [{"arm": ARM, "threads": w, "corpus": c, "role": "cell"} for c, w in order]
        slots += [dict(SENTINEL, role="sentinel")]
        for pos, s in enumerate(slots):
            out.append({"round": r, "pos": pos, **s})
    return out


def schedule(dry=False):
    """The registered schedule; a DRY schedule is the full one's first rounds"""
    full = _schedule(SCHEDULE_SEED, WARMUP_ROUNDS + ROUNDS)
    if not dry:
        return full
    return [s for s in full if s["round"] < WARMUP_ROUNDS + DRY_ROUNDS]


def digest(obj):
    return hashlib.sha256(
        json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


SCHEDULE_SHA256 = "1fb24aee9592e70f764ee7dc739f2372c18796ef43a30f415eb534a2dc419ad2"

# ---- helpers ------------------------------------------------------------------------------


def num(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


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


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def read_file(path):
    try:
        with open(path) as f:
            return f.read().strip()
    except OSError as e:
        return {"error": e.strerror or str(e)}


def cpu_list(s):
    if not isinstance(s, str):
        return None
    out = []
    try:
        for part in s.strip().split(","):
            if not part:
                continue
            a, _, b = part.partition("-")
            out += list(range(int(a), int(b or a) + 1))
    except ValueError:
        return None
    return out


# ---- corpora ------------------------------------------------------------------------------


def _decode_sha(path):
    """bzip2 -dc PATH's SHA-256 and byte count, or the failure"""
    h = hashlib.sha256()
    n = 0
    p = subprocess.Popen(["bzip2", "-dc", path], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    assert p.stdout is not None
    for b in iter(lambda: p.stdout.read(1 << 20), b""):
        h.update(b)
        n += len(b)
    err = p.stderr.read().decode(errors="replace") if p.stderr else ""
    rc = p.wait()
    if rc != 0:
        return None, n, f"bzip2 -dc {path} exited {rc}: {err.strip()[:300]}"
    return h.hexdigest(), n, None


BLOCK_MAGIC = 0x314159265359  # a bzip2 block header's 48 bits (pi), at any bit offset


def count_blocks(data):
    """The bzip2 block headers in DATA, at any bit offset: a count of the 48-bit magic, for
    context (a coincidental match inside compressed bits is possible and not excluded)"""
    n = 0
    for s in range(8):
        if s == 0:
            pat = BLOCK_MAGIC.to_bytes(6, "big")
            i = data.find(pat)
            while i >= 0:
                n += 1
                i = data.find(pat, i + 1)
            continue
        # the magic at bit s of byte i: (8 - s) bits in byte i, 5 whole bytes, s bits in i + 6
        mid = ((BLOCK_MAGIC >> s) & ((1 << 40) - 1)).to_bytes(5, "big")
        head = BLOCK_MAGIC >> (40 + s)
        tail = BLOCK_MAGIC & ((1 << s) - 1)
        j = data.find(mid, 1)
        while 0 <= j <= len(data) - 6:
            if (data[j - 1] & ((1 << (8 - s)) - 1)) == head and (data[j + 5] >> (8 - s)) == tail:
                n += 1
            j = data.find(mid, j + 1)
    return n


def corpora(args):
    os.makedirs(args.outdir, exist_ok=True)
    rec = {"schema": SCHEMA["corpora"], "ok": False, "problems": [], "files": {}}
    bad = rec["problems"]

    def done(code):
        with open(args.record, "w") as f:
            json.dump(rec, f, indent=1)
        print(json.dumps({"ok": rec["ok"], "problems": bad}))
        return code

    v = subprocess.run(["lbzip2", "--version"], capture_output=True, text=True)
    rec["lbzip2_version"] = (
        (v.stdout + v.stderr).strip().splitlines()[0] if (v.stdout + v.stderr).strip() else None
    )
    rec["lbzip2_registered"] = REGISTERED_LBZIP2
    b = subprocess.run(["bzip2", "--help"], capture_output=True, text=True)
    rec["bzip2_version"] = (
        (b.stderr or b.stdout).strip().splitlines()[0] if (b.stderr or b.stdout).strip() else None
    )
    rec["source"] = {"path": os.path.realpath(args.corpus), "sha256": sha256_file(args.corpus)}
    if rec["source"]["sha256"] != CORPUS_SHA256:
        bad.append(
            f"the source corpus is {rec['source']['sha256']}, not the registered {CORPUS_SHA256}"
        )
        return done(1)
    content = os.path.join(args.outdir, "content")
    with open(content, "wb") as f:
        r = subprocess.run(["bzip2", "-dc", args.corpus], stdout=f, stderr=subprocess.PIPE)
    if r.returncode != 0:
        bad.append(f"bzip2 -dc of the corpus exited {r.returncode}")
        return done(1)
    rec["content"] = {
        "sha256": sha256_file(content),
        "bytes": os.path.getsize(content),
        "decoder": "bzip2 -dc (libbzip2)",
    }
    if rec["content"]["sha256"] != CONTENT_SHA256 or rec["content"]["bytes"] != CONTENT_BYTES:
        bad.append(f"the decoded corpus is {rec['content']}, not the registered {CONTENT_SHA256}")
        return done(1)
    for name, lv in LEVELS.items():
        out = os.path.join(args.outdir, name + ".bz2")
        one = out + ".n1"
        flags = [f"-{lv}", "-n", str(LBZIP2_THREADS), "-c"]
        for path, fl in ((out, flags), (one, [f"-{lv}", "-n", "1", "-c"])):
            with open(path, "wb") as f:
                r = subprocess.run(["lbzip2", *fl, content], stdout=f, stderr=subprocess.PIPE)
            if r.returncode != 0:
                bad.append(f"lbzip2 {' '.join(fl)} exited {r.returncode}")
                return done(1)
        f_sha, one_sha = sha256_file(out), sha256_file(one)
        os.remove(one)
        dsha, dn, why = _decode_sha(out)
        reg = REGISTERED_BZ2[name]
        rec["files"][name] = {
            "level": lv,
            "command": "lbzip2 " + " ".join(flags) + " CONTENT",
            "bytes": os.path.getsize(out),
            "sha256": f_sha,
            "sha256_at_one_thread": one_sha,
            "thread_invariant": f_sha == one_sha,
            "matches_registered": f_sha == reg["sha256"] and os.path.getsize(out) == reg["bytes"],
            "decoded_sha256": dsha,
            "decoded_bytes": dn,
            "blocks_by_magic": count_blocks(open(out, "rb").read()),
        }
        if why:
            bad.append(why)
        elif dsha != CONTENT_SHA256 or dn != CONTENT_BYTES:
            bad.append(
                f"{name} decodes to {dsha} ({dn} bytes), not the original's {CONTENT_SHA256}"
            )
    os.remove(content)
    rec["ok"] = not bad
    return done(0 if not bad else 1)


# ---- static facts and the preflight -------------------------------------------------------


def cgroup_quota(root="/"):
    """The CPU quota over the process's cgroup v2 path, established or not:
    - 'unlimited': the cgroup2 mount is found in mountinfo, every directory on the process's
      path from the mount's root exists, and at each level cpu.max reads "max ..." or is absent
      where the parent's cgroup.subtree_control (read) does not delegate the cpu controller (the
      mount's root has no cpu.max when it is the hierarchy's root);
    - 'limited': some cpu.max on the path sets a quota (the files listed);
    - 'unknown' (why): anything else, a missing mount, path, directory or file included. Absence
      of evidence is never read as no limit, and the preflight refuses 'unknown'.
    Levels above a cgroup namespace's root are not visible to any reader; that limit is stated
    in the registration, not established here."""

    def p(x):
        return os.path.join(root, x.lstrip("/"))

    def unknown(why, **kw):
        return {"state": "unknown", "why": why, **kw}

    cg = read_file(p("/proc/self/cgroup"))
    if not isinstance(cg, str):
        return unknown(f"/proc/self/cgroup: {cg}")
    lines = [ln for ln in cg.splitlines() if ln.startswith("0::")]
    if len(lines) != 1:
        return unknown("not one cgroup v2 (unified) entry in /proc/self/cgroup")
    v1cpu = [
        ln
        for ln in cg.splitlines()
        if not ln.startswith("0::") and "cpu" in ln.split(":")[1].split(",")
    ]
    if v1cpu:
        return unknown(f"the cpu controller is on a cgroup v1 hierarchy ({v1cpu[0]}): not read")
    path = lines[0][3:].strip()
    if not path.startswith("/"):
        return unknown(f"the cgroup path {path!r} is not absolute")
    mi = read_file(p("/proc/self/mountinfo"))
    if not isinstance(mi, str):
        return unknown(f"/proc/self/mountinfo: {mi}", path=path)
    mounts = []
    for ln in mi.splitlines():
        left, sep, right = ln.partition(" - ")
        f = left.split()
        if sep and right.split()[:1] == ["cgroup2"] and len(f) >= 5:
            mounts.append((f[3], f[4]))
    if len(mounts) != 1:
        return unknown(f"{len(mounts)} cgroup2 mounts in mountinfo, not one", path=path)
    mroot, mpoint = mounts[0]
    if mroot != "/" and not (path == mroot or path.startswith(mroot.rstrip("/") + "/")):
        return unknown(f"the cgroup path {path} is outside the mount's root {mroot}", path=path)
    rel = path[len(mroot.rstrip("/")) :] if mroot != "/" else path
    parts = [x for x in rel.split("/") if x]
    files, limits, levels = {}, [], []
    base = p(mpoint)
    if not os.path.isfile(os.path.join(base, "cgroup.controllers")):
        return unknown(f"no cgroup.controllers at the cgroup2 mount {mpoint}", path=path)
    for k in range(len(parts) + 1):
        d = os.path.join(base, *parts[:k])
        shown = os.path.join(mpoint, *parts[:k])
        if not os.path.isdir(d):
            return unknown(f"the cgroup directory {shown} does not exist", path=path, files=files)
        f = os.path.join(d, "cpu.max")
        if os.path.exists(f):
            v = read_file(f)
            files[shown + "/cpu.max"] = v
            if not isinstance(v, str) or not v.split():
                return unknown(f"{shown}/cpu.max: {v!r}", path=path, files=files)
            if v.split()[0] != "max":
                limits.append(shown + "/cpu.max")
            levels.append({"dir": shown, "cpu.max": v})
            continue
        if k == 0 and mroot == "/":
            levels.append({"dir": shown, "cpu.max": None, "why": "the hierarchy's root"})
            continue
        if k == 0:
            return unknown(
                f"no cpu.max at the mount's root {shown} (not the hierarchy's root)", path=path
            )
        sc = read_file(os.path.join(os.path.dirname(d), "cgroup.subtree_control"))
        files[os.path.dirname(shown) + "/cgroup.subtree_control"] = sc
        if not isinstance(sc, str):
            return unknown(
                f"no cpu.max in {shown}, and its parent's subtree_control is unreadable",
                path=path,
                files=files,
            )
        if "cpu" in sc.split():
            return unknown(
                f"the cpu controller is delegated to {shown} but it has no cpu.max",
                path=path,
                files=files,
            )
        levels.append({"dir": shown, "cpu.max": None, "why": "the cpu controller is not delegated"})
    return {
        "state": "limited" if limits else "unlimited",
        "path": path,
        "mount": mpoint,
        "files": files,
        "levels": levels,
        "limits": limits,
    }


def static_facts(root="/"):
    def p(x):
        return os.path.join(root, x.lstrip("/"))

    model = None
    cpuinfo = read_file(p("/proc/cpuinfo"))
    if isinstance(cpuinfo, str):
        for line in cpuinfo.splitlines():
            k, _, v = line.partition(":")
            if k.strip() == "model name":
                model = v.strip()
                break
    status = read_file(p("/proc/self/status"))
    allowed = None
    if isinstance(status, str):
        for line in status.splitlines():
            if line.startswith("Cpus_allowed_list:"):
                allowed = line.split(":", 1)[1].strip()
    policies = {}
    for d in sorted(glob.glob(p("/sys/devices/system/cpu/cpufreq/policy*"))):
        policies[os.path.basename(d)] = {
            x: read_file(os.path.join(d, x))
            for x in (
                "affected_cpus",
                "scaling_driver",
                "scaling_governor",
                "energy_performance_preference",
            )
        }
    return {
        "schema": SCHEMA["static"],
        "model": model,
        "online": read_file(p("/sys/devices/system/cpu/online")),
        "cpus_allowed": allowed,
        "no_turbo": read_file(p("/sys/devices/system/cpu/intel_pstate/no_turbo")),
        "intel_pstate_status": read_file(p("/sys/devices/system/cpu/intel_pstate/status")),
        "policies": policies,
        "quota": cgroup_quota(root),
        "uname": " ".join(os.uname()),
    }


def collector_problems(text):
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


def host_problems(lscpu, static, samples):
    bad = []
    model = None
    for line in (lscpu or "").splitlines():
        k, _, v = line.partition(":")
        if k.strip() == "Model name":
            model = v.strip()
    if not (isinstance(model, str) and HOST_MODEL in model):
        bad.append(f"lscpu's model {model!r}: not the registered {HOST_MODEL}")
    if not isinstance(static, dict) or static.get("schema") != SCHEMA["static"]:
        return bad + ["the static facts are not the registered collector's"]
    if not (isinstance(static.get("model"), str) and HOST_MODEL in static["model"]):
        bad.append(
            f"/proc/cpuinfo's model {static.get('model')!r}: not the registered {HOST_MODEL}"
        )
    every = list(range(HOST_CPUS))
    if cpu_list(static.get("online")) != every:
        bad.append(f"the CPUs online {static.get('online')!r}, not all {HOST_CPUS}")
    if cpu_list(static.get("cpus_allowed")) != every:
        bad.append(f"the CPUs allowed {static.get('cpus_allowed')!r}, not all {HOST_CPUS}")
    q = static.get("quota")
    if not (isinstance(q, dict) and q.get("state") == "unlimited"):
        bad.append(f"the CPU quota is not established unlimited: {q!r}")
    if static.get("no_turbo") != "0":
        bad.append(f"intel_pstate/no_turbo {static.get('no_turbo')!r}, not 0 (turbo on)")
    pol = static.get("policies")
    if not isinstance(pol, dict) or not pol:
        bad.append("no cpufreq policies read")
    else:
        covered = []
        for name, p in sorted(pol.items()):
            for k, want in (
                ("scaling_driver", DECLARED["cpufreq_driver"]),
                ("scaling_governor", DECLARED["governor"]),
                ("energy_performance_preference", DECLARED["epp"]),
            ):
                if p.get(k) != want:
                    bad.append(f"{name}: {k} {p.get(k)!r}, not the declared {want!r}")
            covered += cpu_list(p.get("affected_cpus")) or []
        if sorted(covered) != every:
            bad.append(f"the cpufreq policies cover CPUs {sorted(covered)!r}, not all {HOST_CPUS}")
    if not (isinstance(samples, list) and len(samples) == PREFLIGHT_SAMPLES):
        n = len(samples) if isinstance(samples, list) else samples
        return bad + [f"{n!r} sysfs samples, not {PREFLIGHT_SAMPLES}"]
    for i, smp in enumerate(samples):
        s = smp.get("state") if isinstance(smp, dict) else None
        if not isinstance(s, dict):
            bad.append(f"sample {i}: no state")
            continue
        for k in ("governor", "epp", "turbo"):
            if s.get(k) != DECLARED[k] or type(s.get(k)) is not type(DECLARED[k]):
                bad.append(f"sample {i}: {k} {s.get(k)!r}, not the declared {DECLARED[k]!r}")
    return bad


def preflight_problems(d):
    try:
        with open(os.path.join(d, "exits.txt")) as f:
            exits = f.read()
    except OSError:
        exits = None
    bad = collector_problems(exits)
    try:
        with open(os.path.join(d, "lscpu.txt")) as f:
            lscpu = f.read()
    except OSError as e:
        lscpu = None
        bad.append(f"lscpu: {e.strerror}")
    samples, why = load_jsonl(os.path.join(d, "sample.jsonl"), "ab sample")
    bad += [why] if why else []
    static, why = load_json(os.path.join(d, "static.json"), "static facts")
    bad += [why] if why else []
    return bad + host_problems(lscpu, static, samples)


def preflight(args):
    try:
        bad = preflight_problems(os.path.join(args.dir, "preflight"))
    except Exception as e:  # (the preflight's own failure refuses; it never passes)
        bad = [f"the preflight failed: {type(e).__name__}: {e}"]
    verdict = "REFUSED" if bad else "PASS"
    with open(os.path.join(args.dir, "preflight.json"), "w") as f:
        json.dump({"schema": SCHEMA["preflight"], "verdict": verdict, "problems": bad}, f, indent=1)
    print(verdict + ("" if not bad else ": " + "; ".join(bad)))
    return 0 if not bad else 1


# ---- the drive ----------------------------------------------------------------------------


def ab_argv(ab, out, slot, corpora_dir, max_busy):
    return [
        ab,
        "run",
        out,
        "--arms",
        slot["arm"],
        "--threads",
        str(slot["threads"]),
        "--rounds",
        "1",
        "--settle",
        str(SETTLE_S),
        "--max-busy",
        str(max_busy),
        "--sysfs-state",
        os.path.join(corpora_dir, slot["corpus"] + ".bz2"),
    ]


def count_lines(path):
    try:
        with open(path) as f:
            return sum(1 for line in f if line.strip())
    except OSError:
        return 0


def drive(args):
    import time

    d = args.dir
    if args.max_busy is not None and not args.dry:
        print("drive: --max-busy is for a DRY check only")
        return 2
    max_busy = MAX_BUSY if args.max_busy is None else args.max_busy
    sched = schedule(args.dry)
    runs = os.path.join(d, "runs.jsonl")
    here = os.path.dirname(os.path.abspath(__file__))
    secs, grace = CELL_DEADLINE
    with open(os.path.join(d, "slots.jsonl"), "a") as log:
        for i, slot in enumerate(sched):
            before = count_lines(runs)
            argv = [sys.executable, os.path.join(here, "bounded.py"), str(secs), str(grace)]
            argv += ab_argv(args.ab, runs, slot, os.path.join(d, "corpora"), max_busy)
            t0 = time.time()
            rc = subprocess.run(argv).returncode
            rec = {
                "schema": SCHEMA["slot"],
                "index": i,
                **slot,
                "rc": rc,
                "line": before,
                "lines_after": count_lines(runs),
                "t_start_unix": t0,
                "t_end_unix": time.time(),
            }
            log.write(json.dumps(rec) + "\n")
            log.flush()
            if rc != 0:
                print(
                    f"drive: slot {i} ({slot}) exited {rc}: recorded, not retried; no further slot"
                )
                return rc if rc in (124, 125) else 1
    return 0


# ---- the identity -------------------------------------------------------------------------


HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
# the sysfs readings each run's pre and post state must carry: a value, or null with its reason
TELEMETRY = ("freq_mhz", "governor", "epp", "turbo", "pkg_temp_c")


def is_hex(x, rx):
    return isinstance(x, str) and rx.fullmatch(x) is not None


def is_ns(x):
    return isinstance(x, int) and not isinstance(x, bool) and x > 0


def telemetry_problems(state, what):
    """A sysfs state is evidence only as recorded: each reading a value, or null with its
    reason, and its own timestamp. An empty or reasonless state is not an observation"""
    if not isinstance(state, dict):
        return [f"{what}: no state"]
    bad = []
    for k in TELEMETRY:
        if k not in state:
            bad.append(f"{what}: {k} absent")
        elif state[k] is None and not (
            isinstance(state.get(k + "_why"), str) and state[k + "_why"].strip()
        ):
            bad.append(f"{what}: {k} null without a reason")
    if not is_ns(state.get("t_unix_ns")):
        bad.append(f"{what}: no reading time")
    return bad


def identity(args):
    rec = {"schema": SCHEMA["identity"]}
    for kv in args.kv:
        # KEY=TEXT is a string; KEY:=JSON is a JSON value
        k, _, v = kv.partition("=")
        if k.endswith(":"):
            rec[k[:-1]] = json.loads(v)
        else:
            rec[k] = v
    rec["schedule_sha256"] = digest(schedule(bool(rec.get("dry"))))
    rec["schedule_seed"] = SCHEDULE_SEED
    rec["deadlines"] = DEADLINES
    rec["cell_deadline"] = CELL_DEADLINE
    frozen = rec.get("frozen")

    def at_frozen(path):
        if not is_hex(frozen, HEX40):
            return None
        r = subprocess.run(["git", "show", f"{frozen}:{path}"], capture_output=True)
        return hashlib.sha256(r.stdout).hexdigest() if r.returncode == 0 else None

    rec["sources"] = {
        s: {"head": sha256_file(s) if os.path.exists(s) else None, "frozen": at_frozen(s)}
        for s in SOURCES
    }
    cdir = os.path.join(args.dir, "corpora")
    rec["corpora_after"] = {
        n: sha256_file(os.path.join(cdir, n + ".bz2"))
        if os.path.exists(os.path.join(cdir, n + ".bz2"))
        else None
        for n in (*CORPORA, SENTINEL["corpus"])
    }
    with open(os.path.join(args.dir, "identity.json"), "w") as f:
        json.dump(rec, f, indent=1)
    return 0


# ---- the analysis -------------------------------------------------------------------------


@functools.lru_cache(maxsize=32)
def boot_indices(n, block, resamples, seed):
    """Circular moving-block bootstrap: `resamples` index lists of length n, blocks of `block`
    (the same draws for every statistic of the same length and block)"""
    rng = random.Random(seed)
    block = max(1, min(block, n))
    nb = -(-n // block)
    out = []
    for _ in range(resamples):
        idx = []
        for _ in range(nb):
            s = rng.randrange(n)
            idx += [(s + j) % n for j in range(block)]
        out.append(tuple(idx[:n]))
    return tuple(out)


def pct(xs, q):
    xs = sorted(xs)
    if not xs:
        return None
    k = (len(xs) - 1) * q
    lo, hi = math.floor(k), math.ceil(k)
    return xs[lo] + (xs[hi] - xs[lo]) * (k - lo)


def geo(logs, block=BOOT_BLOCK, resamples=BOOT_RESAMPLES, seed=BOOT_SEED):
    """The geometric mean of per-round ratios (given as logs) with its block-bootstrap 95%
    percentile interval, and the independence (t) interval beside it"""
    n = len(logs)
    if n == 0:
        return {"n": 0, "ratio": None, "lo": None, "hi": None}
    m = sum(logs) / n
    out = {"n": n, "ratio": math.exp(m), "lo": None, "hi": None, "block": block}
    if n >= 2:
        # (a block as long as the rounds only rotates them: no bootstrap interval, said so)
        if block < n:
            idxs = boot_indices(n, block, resamples, seed)
            means = [sum(logs[i] for i in idx) / n for idx in idxs]
            out["lo"], out["hi"] = math.exp(pct(means, 0.025)), math.exp(pct(means, 0.975))
        else:
            out["no_interval"] = f"block {block} is not shorter than the {n} rounds"
        sd = st.stdev(logs)
        t = T95.get(n - 1, 1.96)
        out["t_lo"], out["t_hi"] = (
            math.exp(m - t * sd / math.sqrt(n)),
            math.exp(m + t * sd / math.sqrt(n)),
        )
        if n >= 3 and sd > 0:
            mu = m
            num_ = sum((logs[i] - mu) * (logs[i + 1] - mu) for i in range(n - 1))
            out["lag1_autocorr"] = num_ / sum((x - mu) ** 2 for x in logs)
    return out


T95 = {
    1: 12.706,
    2: 4.303,
    3: 3.182,
    4: 2.776,
    5: 2.571,
    6: 2.447,
    7: 2.365,
    8: 2.306,
    9: 2.262,
    10: 2.228,
    12: 2.179,
    15: 2.131,
    20: 2.086,
    30: 2.042,
    40: 2.021,
    59: 2.001,
    60: 2.000,
}
for _df in range(1, 200):
    if _df not in T95:
        _k = max(k for k in T95 if k < _df)
        T95[_df] = T95[_k] if _df < 60 else 2.0


def band_class(g):
    lo, hi = g.get("lo"), g.get("hi")
    if lo is None or hi is None:
        return "no interval"
    a, b = BAND
    if lo >= a and hi <= b:
        return "within the band"
    if hi < a:
        return "below the band"
    if lo > b:
        return "above the band"
    if lo < a and hi > b:
        return "wider than the band"
    return "overlaps the band's lower edge" if lo < a else "overlaps the band's upper edge"


def fmt(g):
    if g.get("ratio") is None:
        return "n/a"
    s = f"x{g['ratio']:.2f}"
    if g.get("lo") is not None:
        s += f" [{g['lo']:.2f}-{g['hi']:.2f}]"
    return s


def identity_problems(ident, sched, dry):
    bad = []
    if ident.get("schema") != SCHEMA["identity"]:
        bad.append("the identity is not the registered schema")
    if ident.get("dry") is not dry:
        bad.append(f"the identity's dry {ident.get('dry')!r} is not this judgement's {dry}")
    if not is_hex(ident.get("head"), HEX40):
        bad.append(f"the identity's head {ident.get('head')!r} is not a commit")
    fz = ident.get("frozen")
    if not dry:
        if digest(sched) != SCHEDULE_SHA256:
            bad.append("the schedule's digest is not the registered one")
        if not is_hex(fz, HEX40):
            bad.append(f"the identity's frozen {fz!r} is not a commit")
        if ident.get("inputs_match_frozen") is not True:
            bad.append("the inputs did not match the frozen commit")
    elif fz is not None and not is_hex(fz, HEX40):
        bad.append(f"the identity's frozen {fz!r} is neither a commit nor null")
    if ident.get("schedule_sha256") != digest(sched):
        bad.append("the identity's schedule digest is not this schedule's")
    if ident.get("drive_exit") != 0:
        bad.append(f"the drive exited {ident.get('drive_exit')!r}")
    if not (
        is_hex(ident.get("ab_sha256"), HEX64)
        and ident.get("ab_sha256") == ident.get("ab_sha256_after")
    ):
        bad.append("the ab executable's digest is missing, or changed during the run")
    if ident.get("tree_clean_after") is not True:
        bad.append("the checkout was not clean after the run")
    src = ident.get("sources")
    if not isinstance(src, dict):
        bad.append("the identity records no source digests")
    else:
        for s in SOURCES:
            e = src.get(s) if isinstance(src.get(s), dict) else {}
            if not is_hex(e.get("head"), HEX64):
                bad.append(f"no digest of {s} at the head")
            elif not dry and e.get("frozen") != e.get("head"):
                bad.append(f"{s} at the head is not {s} at the frozen commit")
    return bad


def corpora_problems(cor, ident):
    bad = []
    if not cor.get("ok"):
        bad.append(f"the corpora's record is not ok: {cor.get('problems')}")
    content = cor.get("content") if isinstance(cor.get("content"), dict) else {}
    if content.get("sha256") != CONTENT_SHA256 or content.get("bytes") != CONTENT_BYTES:
        bad.append("the corpora's record does not show the registered content")
    if (cor.get("source") or {}).get("sha256") != CORPUS_SHA256:
        bad.append("the corpora's record does not show the registered source corpus")
    after = ident.get("corpora_after") if isinstance(ident.get("corpora_after"), dict) else {}
    files = cor.get("files") if isinstance(cor.get("files"), dict) else {}
    for n in CORPORA:
        f = files.get(n) if isinstance(files.get(n), dict) else {}
        built, now = f.get("sha256"), after.get(n)
        if not (is_hex(built, HEX64) and is_hex(now, HEX64)):
            bad.append(f"{n}: a digest is missing (built {built!r}, after the run {now!r})")
        elif built != now:
            bad.append(f"{n}'s digest after the run is not the one built")
        if f.get("decoded_sha256") != CONTENT_SHA256 or f.get("decoded_bytes") != CONTENT_BYTES:
            bad.append(f"{n}: the record does not show it decoding to the content")
    if after.get(SENTINEL["corpus"]) != CORPUS_SHA256:
        bad.append("the sentinel's corpus after the run is not the registered corpus")
    return bad


def preflight_replay(d, ident, dry):
    """The archived preflight judged again; it must agree with the recorded exit, and pass
    unless DRY"""
    pdir = os.path.join(d, "preflight")
    if not os.path.isdir(pdir):
        return ["no archived preflight (preflight/)"], None
    replay = preflight_problems(pdir)
    verdict = "PASS" if not replay else "REFUSED"
    bad = []
    pe = ident.get("preflight_exit")
    if pe not in (0, 1) or (pe == 0) != (verdict == "PASS"):
        bad.append(f"the archived preflight replays {verdict}, but the recorded exit is {pe!r}")
    if not dry and verdict != "PASS":
        bad.append("the archived preflight does not pass: " + "; ".join(replay[:3]))
    return bad, {"verdict": verdict, "problems": replay}


def admission_problems(d, dry):
    bad = []
    ident, why = load_json(os.path.join(d, "identity.json"), "identity")
    if why or not isinstance(ident, dict):
        return [why or "the identity is not an object"], None, None
    sched = schedule(dry)
    bad += identity_problems(ident, sched, dry)
    cor, why = load_json(os.path.join(d, "corpora.json"), "corpora")
    bad += [why] if why else corpora_problems(cor if isinstance(cor, dict) else {}, ident)
    pbad, replay = preflight_replay(d, ident, dry)
    bad += pbad
    slots, why = load_jsonl(os.path.join(d, "slots.jsonl"), "slots")
    runs, why2 = load_jsonl(os.path.join(d, "runs.jsonl"), "runs")
    bad += [w for w in (why, why2) if w]
    if why or why2:
        return bad, None, replay
    if len(slots) != len(sched):
        bad.append(f"{len(slots)} slots ran, not the {len(sched)} scheduled")
    if len(runs) != len(sched):
        bad.append(f"{len(runs)} run lines, not the {len(sched)} scheduled")
    keys = ("round", "pos", "arm", "threads", "corpus", "role")
    joined = []
    last_exit, last_end = 0, 0.0
    for i, (want, got) in enumerate(zip(sched, slots)):
        if any(got.get(k) != want[k] for k in keys) or got.get("index") != i:
            bad.append(f"slot {i} is not the scheduled {want}")
            continue
        if got.get("rc") != 0:
            bad.append(f"slot {i} exited {got.get('rc')!r}")
            continue
        if got.get("line") != i or got.get("lines_after") != i + 1:
            bad.append(
                f"slot {i} did not add exactly one run line (line {got.get('line')!r}, after {got.get('lines_after')!r})"
            )
            continue
        t0, t1 = got.get("t_start_unix"), got.get("t_end_unix")
        if not (num(t0) and num(t1) and last_end <= t0 <= t1):
            bad.append(f"slot {i}: its start and end times are missing or out of order")
            continue
        last_end = t1
        if i >= len(runs):
            continue
        r = runs[i]
        if (r.get("arm"), r.get("threads"), r.get("corpus")) != (
            want["arm"],
            want["threads"],
            want["corpus"],
        ):
            bad.append(
                f"run line {i} is {r.get('arm')}/{r.get('threads')}/{r.get('corpus')}, not slot {i}'s"
            )
            continue
        if (
            r.get("out") != CONTENT_BYTES
            or r.get("hash") != WORKLOAD_HASH
            or r.get("err") not in ("", None)
        ):
            bad.append(
                f"run {i}: out {r.get('out')!r}, hash {r.get('hash')!r}, err {r.get('err')!r}: not the corpus's bytes"
            )
            continue
        if not (num(r.get("cpu")) and r["cpu"] > 0 and num(r.get("wall")) and r["wall"] > 0):
            bad.append(f"run {i}: no finite CPU and wall")
            continue
        if not (num(r.get("waited_s")) and r["waited_s"] >= -1):
            bad.append(f"run {i}: no busy-gate wait recorded (waited_s)")
            continue
        tb = telemetry_problems(r.get("machine_pre"), f"run {i}'s pre-run state")
        tb += telemetry_problems(r.get("machine_post"), f"run {i}'s post-run state")
        ts, te = r.get("t_spawn_unix_ns"), r.get("t_exit_unix_ns")
        if not (is_ns(ts) and is_ns(te) and last_exit <= ts <= te):
            tb.append(f"run {i}: its spawn and exit times are missing or out of order")
        if tb:
            bad += tb[:2]
            continue
        last_exit = te
        joined.append(
            {
                **want,
                "cpu": r["cpu"],
                "wall": r["wall"],
                "waited_s": r["waited_s"],
                "pre": r["machine_pre"],
                "t_spawn_ns": ts,
                "t_exit_ns": te,
                "slot_s": t1 - t0,
                "t_start": t0,
            }
        )
    return bad, joined, replay


INTERVAL_LABEL = (
    "conditional on this attempt and on the assumed dependence (a circular moving-block "
    "bootstrap over chronological rounds); not a guaranteed 95% coverage: dependence longer "
    "than the block, or a shock common to the whole attempt, is not bounded"
)
R_CLASSES = {
    "below": "below 1: a favourable level x worker interaction (not isolated table capacity)",
    "above": "above 1: a contrary interaction (it closes nothing)",
    "includes": "includes 1: INCONCLUSIVE",
    "none": "no interval",
}


R_TAGS = {
    R_CLASSES["below"]: "a favourable interaction",
    R_CLASSES["above"]: "a contrary interaction",
    R_CLASSES["includes"]: "INCONCLUSIVE",
    R_CLASSES["none"]: "no interval",
}


def r_class(g):
    lo, hi = g.get("lo"), g.get("hi")
    if lo is None or hi is None:
        return R_CLASSES["none"]
    if hi < 1:
        return R_CLASSES["below"]
    if lo > 1:
        return R_CLASSES["above"]
    return R_CLASSES["includes"]


def g9_class(g):
    lo, hi = g.get("lo"), g.get("hi")
    if lo is None or hi is None:
        return "no interval"
    if lo <= S2_L9 <= hi:
        return f"compatible with x{S2_L9} (not equivalence, and not the ladder's validation)"
    return f"excludes x{S2_L9}"


def med_or_none(xs):
    xs = [x for x in xs if num(x)]
    return st.median(xs) if xs else None


def analyse(rows, dry):
    measured = [r for r in rows if r["round"] >= WARMUP_ROUNDS]
    rounds = sorted({r["round"] for r in measured})
    cell = {}
    sentinels = {}
    by_round = {}
    for r in measured:
        by_round.setdefault(r["round"], []).append(r)
        if r["role"] == "cell":
            cell[(r["corpus"], r["threads"], r["round"])] = r
        else:
            sentinels.setdefault(r["round"], []).append(r["wall"])

    def logs(corpus, w, what, rs, ref=REF):
        return [math.log(cell[(corpus, w, k)][what] / cell[(corpus, ref, k)][what]) for k in rs]

    def summary(rs, block=BOOT_BLOCK, intervals=True):
        out = {}
        for what in ("cpu", "wall"):
            l2, l9 = logs(PRIMARY, TOP, what, rs), logs(COMPARATOR, TOP, what, rs)
            rr = [a - b for a, b in zip(l2, l9)]
            if intervals:
                g = {PRIMARY: geo(l2, block), COMPARATOR: geo(l9, block)}
                q = geo(rr, block)
            else:  # a point estimate only (no interval; see the mode split)
                g = {
                    c: {"n": len(x), "ratio": math.exp(sum(x) / len(x)) if x else None}
                    for c, x in ((PRIMARY, l2), (COMPARATOR, l9))
                }
                q = {"n": len(rr), "ratio": math.exp(sum(rr) / len(rr)) if rr else None}
            out[what] = {"growth": g, "ratio_of_ratios": q}
        return out

    res = {"rounds": len(rounds), "warmup_rounds": WARMUP_ROUNDS, "interval_label": INTERVAL_LABEL}
    res["primary"] = summary(rounds)
    res["sensitivity"] = {f"block{b}": summary(rounds, b) for b in BOOT_SENSITIVITY}
    grid = {}
    for c in CORPORA:
        grid[c] = {}
        for w in WORKERS:
            cs = [cell[(c, w, k)] for k in rounds]
            walls = [x["wall"] for x in cs]
            grid[c][w] = {
                "cpu_median": st.median([x["cpu"] for x in cs]),
                "wall_median": st.median(walls),
                "mb_s_median": CONTENT_BYTES / 1e6 / st.median(walls),
                "cpu_growth_vs_4": geo(logs(c, w, "cpu", rounds)) if w != REF else None,
                "pre_freq_mhz_median": med_or_none([x["pre"].get("freq_mhz") for x in cs]),
                "pre_freq_unread": sum(1 for x in cs if x["pre"].get("freq_mhz") is None),
            }
    res["grid"] = grid
    # the pre-run sysfs state: unread readings (null, with their reasons) apart from read ones
    # that differ from the declared settings. Counted, never an exclusion
    unread = {k: 0 for k in TELEMETRY}
    dev = {k: 0 for k in ("governor", "epp", "turbo")}
    for r in measured:
        for k in TELEMETRY:
            if r["pre"].get(k) is None:
                unread[k] += 1
        for k in dev:
            if r["pre"].get(k) is not None and r["pre"][k] != DECLARED[k]:
                dev[k] += 1
    res["pre_state"] = {
        "runs": len(measured),
        "unread": unread,
        "read_not_as_declared": dev,
        "label": "recorded, never an exclusion",
    }
    # the busy gate's waits (ab's waited_s), by cell and by position: described, never an
    # exclusion or an adjustment
    waits = {"by_cell": {}, "by_position": {}}
    for r in measured:
        key = f"{r['corpus']}@{r['threads']}" if r["role"] == "cell" else "sentinel"
        for grp, k in (("by_cell", key), ("by_position", str(r["pos"]))):
            e = waits[grp].setdefault(k, {"runs": 0, "waited": 0, "total_s": 0.0, "max_s": 0.0})
            e["runs"] += 1
            e["waited"] += r["waited_s"] > 0
            e["total_s"] += max(0.0, r["waited_s"])
            e["max_s"] = max(e["max_s"], r["waited_s"])
    res["waits"] = waits
    g2, g9 = res["primary"]["cpu"]["growth"][PRIMARY], res["primary"]["cpu"]["growth"][COMPARATOR]
    rr = res["primary"]["cpu"]["ratio_of_ratios"]
    res["model_check"] = {
        "band": BAND,
        "l2_point_in_band": g2["ratio"] is not None and BAND[0] <= g2["ratio"] <= BAND[1],
        "l2_interval_vs_band": band_class(g2),
        "l9_s2": S2_L9,
        "l9_against_s2": g9_class(g9),
        "ratio_of_ratios": r_class(rr),
        "model_gb_s": {
            "label": "HISTORICAL MODEL (§3 of the share-nothing entry), not observed: "
            "3.76 GB/s / G2, its numerator fixed (10 x one thread's 400 MB/s x 0.94); it "
            "takes the unmeasured 1-to-4 factor as 1, and the Workers arm's 4-to-10 growth "
            "as a finder-fed kernel's, which it does not identify",
            "point": MODEL_GB_S_NUMERATOR / g2["ratio"] if g2["ratio"] else None,
            "lo": MODEL_GB_S_NUMERATOR / g2["hi"] if g2.get("hi") else None,
            "hi": MODEL_GB_S_NUMERATOR / g2["lo"] if g2.get("lo") else None,
        },
    }
    # the descriptive mode split (registered; never an exclusion): a round is fast if both its
    # sentinels' walls are below the threshold, slow if both are at or above it, else mixed.
    # Point estimates only: selecting rounds by class and then block-resampling the selection
    # would compress the time axis, so no class gets an interval
    cls = {}
    for k in rounds:
        ws = sentinels.get(k, [])
        if len(ws) != 2:
            cls[k] = "unread"
        elif all(w < MODE_THRESHOLD_S for w in ws):
            cls[k] = "fast"
        elif all(w >= MODE_THRESHOLD_S for w in ws):
            cls[k] = "slow"
        else:
            cls[k] = "mixed"
    modes = {
        "label": "DESCRIPTIVE: rounds classed by their two sentinels' walls (threshold "
        f"{MODE_THRESHOLD_S} s), an endpoint-bracketed indicator of another plan's state; "
        "not a mode identification of the cells, never an exclusion; point estimates only",
        "counts": {
            c: sum(1 for v in cls.values() if v == c) for c in ("fast", "slow", "mixed", "unread")
        },
        "classes": {},
    }
    for c in ("fast", "slow", "mixed"):
        rs = [k for k in rounds if cls[k] == c]
        if rs:
            modes["classes"][c] = {
                "rounds": len(rs),
                **summary(rs, intervals=False),
                "pre_freq_mhz_median": med_or_none(
                    [r["pre"].get("freq_mhz") for k in rs for r in by_round[k]]
                ),
            }
    res["modes"] = modes
    # the chronological record: every measured round in order, for the time-series reading
    t0 = min(r["t_start"] for r in measured) if measured else 0.0
    chron = []
    for k in rounds:
        rs = by_round[k]
        chron.append(
            {
                "round": k,
                "start_s": round(min(r["t_start"] for r in rs) - t0, 3),
                "duration_s": round(
                    max(r["t_start"] + r["slot_s"] for r in rs) - min(r["t_start"] for r in rs), 3
                ),
                "waited_s": sum(max(0.0, r["waited_s"]) for r in rs),
                "sentinel_walls": sentinels.get(k),
                "class": cls[k],
                "log_r_cpu": {c: logs(c, TOP, "cpu", [k])[0] for c in CORPORA},
                "log_rr_cpu": logs(PRIMARY, TOP, "cpu", [k])[0]
                - logs(COMPARATOR, TOP, "cpu", [k])[0],
                "pre_freq_mhz_median": med_or_none([r["pre"].get("freq_mhz") for r in rs]),
            }
        )
    res["chronological"] = chron
    return res


def _gb(x):
    if x.get("point") is None:
        return "n/a"
    s = f"{x['point']:.2f} GB/s"
    if x.get("lo") is not None:
        s += f" [{x['lo']:.2f}-{x['hi']:.2f}]"
    return s


def _f(x, spec):
    return format(x, spec) if num(x) else "n/a"


def render(outcome, res, bad):
    lines = [outcome, ""]
    if bad:
        lines += ["## Admission", ""] + [f"- {b}" for b in bad]
        return "\n".join(lines) + "\n"
    p = res["primary"]
    lines += [
        f"Rounds analysed: {res['rounds']} (after {res['warmup_rounds']} warm-up). Intervals: "
        f"moving-block bootstrap, block {BOOT_BLOCK} rounds, {BOOT_RESAMPLES} resamples, 95% "
        f"percentile, {res['interval_label']}.",
        "",
        "## Growth from 4 to 10 workers (4-to-10 inflation, not 1-to-10)",
        "",
        "| reading | -2 | -9 | ratio of ratios |",
        "|---|--:|--:|--:|",
    ]
    for what in ("cpu", "wall"):
        g = p[what]["growth"]
        lines.append(
            f"| {what} | {fmt(g[PRIMARY])} | {fmt(g[COMPARATOR])} | {fmt(p[what]['ratio_of_ratios'])} |"
        )
    m = res["model_check"]
    lines += [
        "",
        "## Model check (CPU-seconds)",
        "",
        f"- -2 growth against §2's historical band {BAND[0]}-{BAND[1]}: {m['l2_interval_vs_band']} "
        f"(point in band: {m['l2_point_in_band']})",
        f"- -9 growth against §2's historical x{S2_L9}: {m['l9_against_s2']}",
        f"- ratio of ratios (the level x worker interaction): {m['ratio_of_ratios']}",
        f"- {m['model_gb_s']['label']}: {_gb(m['model_gb_s'])}",
        "",
        "## Sensitivity (block length)",
        "",
    ]
    for k, s in res["sensitivity"].items():
        g = s["cpu"]["growth"]
        lines.append(
            f"- {k}: -2 {fmt(g[PRIMARY])}, -9 {fmt(g[COMPARATOR])}, ratio of ratios {fmt(s['cpu']['ratio_of_ratios'])}"
        )
    pg = p["cpu"]["growth"]
    lines.append(
        "- t intervals (independent rounds) and lag-1 autocorrelation: "
        + ", ".join(
            f"{n} [{_f(g.get('t_lo'), '.2f')}-{_f(g.get('t_hi'), '.2f')}] lag1 {_f(g.get('lag1_autocorr'), '.2f')}"
            for n, g in (
                ("-2", pg[PRIMARY]),
                ("-9", pg[COMPARATOR]),
                ("R", p["cpu"]["ratio_of_ratios"]),
            )
        )
    )
    lines += [
        "",
        "## Grid (medians)",
        "",
        "| corpus | workers | CPU s | wall s | MB/s | CPU growth vs 4 | pre-run clock MHz (unread) |",
        "|---|--:|--:|--:|--:|--:|--:|",
    ]
    for c in CORPORA:
        for w in WORKERS:
            x = res["grid"][c][w]
            gr = fmt(x["cpu_growth_vs_4"]) if x["cpu_growth_vs_4"] else "1"
            lines.append(
                f"| {c} | {w} | {x['cpu_median']:.3f} | {x['wall_median']:.3f} | {x['mb_s_median']:.0f} | {gr} | {_f(x['pre_freq_mhz_median'], '.0f')} ({x['pre_freq_unread']}) |"
            )
    ps = res["pre_state"]
    lines += [
        "",
        f"Pre-run sysfs state ({ps['runs']} runs; {ps['label']}): unread {ps['unread']}; "
        f"read but not as declared {ps['read_not_as_declared']}",
        "",
        "## Busy-gate waits (described, never an exclusion)",
        "",
        "| group | runs | runs that waited | total s | max s |",
        "|---|--:|--:|--:|--:|",
    ]
    for grp in ("by_cell", "by_position"):
        for k, e in sorted(res["waits"][grp].items()):
            lines.append(
                f"| {grp[3:]} {k} | {e['runs']} | {e['waited']} | {e['total_s']:.0f} | {e['max_s']:.0f} |"
            )
    md = res["modes"]
    lines += ["", "## Mode split", "", md["label"], "", f"Rounds: {md['counts']}", ""]
    for c, s in md["classes"].items():
        g = s["cpu"]["growth"]
        lines.append(
            f"- {c} ({s['rounds']} rounds): -2 {fmt(g[PRIMARY])}, -9 {fmt(g[COMPARATOR])}, "
            f"ratio of ratios {fmt(s['cpu']['ratio_of_ratios'])}, pre-run clock median "
            f"{_f(s['pre_freq_mhz_median'], '.0f')} MHz"
        )
    lines += [
        "",
        "## Chronological rounds",
        "",
        "| round | start s | duration s | waited s | sentinel walls | class | log r -2 | log r -9 | log R | pre clock |",
        "|--:|--:|--:|--:|---|---|--:|--:|--:|--:|",
    ]
    for c in res["chronological"]:
        sw = ", ".join(f"{w:.3f}" for w in (c["sentinel_walls"] or []))
        lines.append(
            f"| {c['round']} | {c['start_s']:.1f} | {c['duration_s']:.1f} | {c['waited_s']:.0f} | {sw} | {c['class']} | "
            f"{c['log_r_cpu'][PRIMARY]:.3f} | {c['log_r_cpu'][COMPARATOR]:.3f} | {c['log_rr_cpu']:.3f} | {_f(c['pre_freq_mhz_median'], '.0f')} |"
        )
    return "\n".join(lines) + "\n"


def judge(args):
    d, dry = args.dir, args.dry
    res, bad, replay = None, [], None
    try:
        bad, rows, replay = admission_problems(d, dry)
        if not bad:
            res = analyse(rows, dry)
    except Exception as e:  # (the judge's own failure is INVALID, never a crash or a VALID)
        bad = bad + [f"the judge failed: {type(e).__name__}: {e}"]
        res = None
    if bad:
        outcome = (
            "INVALID: "
            + "; ".join(bad[:5])
            + (f"; and {len(bad) - 5} more" if len(bad) > 5 else "")
        )
    else:
        p = res["primary"]["cpu"]
        m = res["model_check"]
        body = (
            f"-2 CPU growth 4->10 {fmt(p['growth'][PRIMARY])}, against the historical band "
            f"{BAND[0]}-{BAND[1]}: {m['l2_interval_vs_band']}; -9 {fmt(p['growth'][COMPARATOR])}; "
            f"ratio of ratios {fmt(p['ratio_of_ratios'])}: {R_TAGS[m['ratio_of_ratios']]}"
        )
        outcome = ("DRY (not evidence): " if dry else "VALID: ") + body
    out = {
        "schema": SCHEMA["judgement"],
        "outcome": outcome,
        "dry": dry,
        "problems": bad,
        "preflight_replay": replay,
        "result": res,
    }
    with open(os.path.join(d, "judgement.json"), "w") as f:
        json.dump(out, f, indent=1)
    text = render(outcome, res, bad)
    with open(os.path.join(d, "judgement.md"), "w") as f:
        f.write(text)
    print(text, end="")
    return 2 if bad else 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("schedule")
    a.add_argument("--dry", action="store_true")
    a.add_argument("--out", required=True)
    a = sub.add_parser("corpora")
    a.add_argument("corpus")
    a.add_argument("outdir")
    a.add_argument("--record", required=True)
    a = sub.add_parser("static")
    a.add_argument("--out", required=True)
    a = sub.add_parser("preflight")
    a.add_argument("dir")
    a = sub.add_parser("drive")
    a.add_argument("dir")
    a.add_argument("ab")
    a.add_argument("--dry", action="store_true")
    a.add_argument("--max-busy", type=float)
    a = sub.add_parser("identity")
    a.add_argument("dir")
    a.add_argument("kv", nargs="*")
    a = sub.add_parser("judge")
    a.add_argument("dir")
    a.add_argument("--dry", action="store_true")
    a = sub.add_parser("get")
    a.add_argument("name")
    args = ap.parse_args(argv)
    if args.cmd == "schedule":
        s = schedule(args.dry)
        with open(args.out, "w") as f:
            json.dump(
                {
                    "schema": SCHEMA["schedule"],
                    "seed": SCHEDULE_SEED,
                    "dry": args.dry,
                    "sha256": digest(s),
                    "slots": s,
                },
                f,
                indent=1,
            )
        return 0
    if args.cmd == "corpora":
        return corpora(args)
    if args.cmd == "static":
        with open(args.out, "w") as f:
            json.dump(static_facts(), f, indent=1)
        return 0
    if args.cmd == "preflight":
        return preflight(args)
    if args.cmd == "drive":
        return drive(args)
    if args.cmd == "identity":
        return identity(args)
    if args.cmd == "judge":
        return judge(args)
    if args.cmd == "get":
        v = globals()[args.name]
        print(json.dumps(v) if not isinstance(v, str) else v)
        return 0
    return 2


if __name__ == "__main__":
    sys.exit(main())
