"""The general plan policy's three diagnostic registrations on the workstation
(docs/journal/2026-11-07-general-plan-policy.md §5): the reports and their outcome maps,
each predicate checked here. Diagnostic only (RULES C1): no outcome of any of them can
qualify a production change.

    python3 bench/plan_diagnostics.py boundaries W [--out STEM]   bench-plan-boundaries
    python3 bench/plan_diagnostics.py bytes W [--out STEM]        bench-plan-bytes
    python3 bench/plan_diagnostics.py levels W [--out STEM]       bench-plan-levels

and, for the recipes, the registered cells as they run on this machine (one line a cell,
`NAME MASK THREADS ARMS CORPORA`, MASK `all` or a CPU list; with --dry, a mask wider than the
machine is narrowed to its cores, and the run can't be VALID), whether `lscpu`'s output is the
ten-core topology (its exit status), and a corpus's registered SHA-256:

    python3 bench/plan_diagnostics.py cells boundaries|bytes|levels ENV.json [--dry]
    python3 bench/plan_diagnostics.py topology LSCPU.txt
    python3 bench/plan_diagnostics.py sha CORPUS

W is the attempt's directory, as its recipe wrote it: `runs-CELL.jsonl` (one `ab run` a cell),
`masks.txt` (`CELL MASK`, `all` where unmasked), `env.json`, `lscpu.txt`, `corpus.sha256`,
`commit.txt`, `git-status.txt`, and, where they apply, `DRY-RUN.txt` and `NOT-FROZEN.txt`.

**A run is VALID only as the registered one:**
- the registered 20 rounds in every cell, each arm's rounds 0-19 once each, and no other arm,
  thread count, corpus or cell;
- from a clean tree (`git-status.txt` empty), no `NOT-FROZEN.txt` and no `DRY-RUN.txt`, its
  `commit.txt` the checkout's head;
- on the ten-core topology: 1 socket, 10 cores, 2 threads a core, a 20 MiB L3 (`lscpu.txt`);
- each corpus the registered one, by SHA-256 (`corpus.sha256`);
- every run the same bytes (every level of the content decodes to the same 209.7 MB), none
  failed;
- every run under its cell's registered mask (`allowed`, the CPUs `ab run` was given), the
  mask itself the registered shape on this machine's cores (`env.json`'s `core_cpus`);
- every run's plan the registered one, read both from the threads that ran (`thread_cpu`)
  and from every thread the parent `ab` saw (`observed_threads`): leaders `mibz2-N`, a plain
  team's helpers `mibz2-coop-N`, a pipelined team's `mibz2-coop-pN`. **A cell whose observed
  plan differs from its registered plan makes the run INVALID.**

**The comparisons** are paired by round: the speed ratio is the reference's wall over the
arm's (above 1: the arm is faster), its geometric mean and a Bonferroni interval over the
registration's family (each interval at 1 - 0.05/(2m) a tail, m the family's comparisons, t
at 19 degrees of freedom). The words, as the journal defines them:
- **FASTER:** a point estimate of at least 1.03 and a lower bound above 1;
- **SLOWER:** an upper bound below 1;
- **UNRESOLVED:** anything else.

CPU (the arm's median over the reference's) and each arm's median frequency during its runs
(`freq_mhz`) are reported and not judged.
"""

import argparse
import json
import math
import os
import re
import statistics
import subprocess
import sys

ROUNDS = 20
FASTER_POINT = 1.03
# Student's t at 19 degrees of freedom, upper quantiles (computed by numerical integration of
# the t density, rounded to 4 places), keyed by twice the comparisons: 1 - 0.05/2 (one
# comparison, the plain 95% interval of a control), 1 - 0.05/14, 1 - 0.05/12, 1 - 0.05/24
T19 = {2: 2.0930, 14: 3.0136, 12: 2.9439, 24: 3.2550}
LEVELS_SHA = {
    # the bench's Wikidata sample (bench/wikidata.json), decoded and recompressed as one stream
    # by `ab recompress` (libbzip2 1.0.8, the bzip2 crate's bundled source): deterministic
    "wikidata": "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915",
    "wikidata-l1": "2ae39adffbfbaab688d89b7597e57e79a551e25cba6db1c21f9599093d0d9e78",
    "wikidata-l2": "aa55dcfa36b98b5da4269893e512a9b86e1fba398a851a784490657435371f6a",
    "wikidata-l3": "380cbfda616222f8b26831653c845a0cd605d3962ce74ec527b445cd3ae7c5a5",
    "wikidata-l6": "f3bbccf5d914280a40c27b311a2694893329937251617e98dbba280594158502",
    "wikidata-l7": "58c8897f3da919c95001a8f60f823d6b9652405f1baf8cd7bcb2352ea4dd68a1",
    "wikidata-l9": "817e6d89f4ab7df83d91d6d356ea8aa4b0d0e7af9ea11e21541a08c9d06adc32",
}
# the base of bench-plan-levels: v0.2.4's release commit, by its SHA
V024 = "1e644f9a327cd7282e79b40d9f870abc440e33e5"

# plans as (leaders, plain helpers, pipelined helpers)
P42, P32, P24P, P33P, P33 = (4, 4, 0), (3, 3, 0), (2, 0, 6), (3, 0, 6), (3, 6, 0)


class Cell:
    """A registered cell: its mask ("all", or (cores, "one"|"both")), the threads asked for,
    and its arms with the plan each must run, by corpus where the plan depends on it"""

    def __init__(self, name, mask, threads, arms, corpora=("wikidata",)):
        self.name, self.mask, self.threads, self.arms, self.corpora = name, mask, threads, arms, corpora

    def plan(self, arm, corpus):
        p = self.arms[arm]
        return p(corpus) if callable(p) else p


# Registration 1, bench-plan-boundaries: the boundary table of §4
BOUNDARIES = [
    Cell("U8", "all", 8, {"mibz2-plan-4x2": P42, "mibz2-plan-3x2": P32, "mibz2-plan-2x4p": P24P}),
    Cell("U9", "all", 9, {"mibz2-plan-4x2": P42, "mibz2-plan-3x3p": P33P}),
    Cell("M8", (8, "one"), 8, {"mibz2-plan-4x2": P42, "mibz2-plan-3x2": P32}),
    Cell("M8S", (8, "both"), 16, {"mibz2-plan-4x2": P42, "mibz2-plan-3x2": P32}),
    Cell("M9", (9, "one"), 9, {"mibz2-plan-4x2": P42, "mibz2-plan-3x2": P32, "mibz2-plan-3x3p": P33P}),
    # the controls, where every rule picks 3 x 2: today's default against 3 x 2 forced
    Cell("C6", "all", 6, {"mibz2": P32, "mibz2-plan-3x2": P32}),
    Cell("C6M", (6, "one"), 6, {"mibz2": P32, "mibz2-plan-3x2": P32}),
]
# the family: the 7 comparisons against 4 x 2, (cell, arm)
BOUNDARY_FAMILY = [
    ("U8", "mibz2-plan-3x2"),
    ("U8", "mibz2-plan-2x4p"),
    ("U9", "mibz2-plan-3x3p"),
    ("M8", "mibz2-plan-3x2"),
    ("M8S", "mibz2-plan-3x2"),
    ("M9", "mibz2-plan-3x2"),
    ("M9", "mibz2-plan-3x3p"),
]
EIGHT = ("U8", "M8", "M8S")

# Registration 2, bench-plan-bytes: N = 10 on the level-6, -7 and -9 recompressions
BYTES = [
    Cell(
        "B10",
        "all",
        10,
        {"mibz2-plan-4x2": P42, "mibz2-plan-3x3": P33, "mibz2-plan-3x3p": P33P},
        corpora=("wikidata-l6", "wikidata-l7", "wikidata-l9"),
    )
]
BYTES_FAMILY = [(f"wikidata-l{lv}", arm) for lv in (6, 7, 9) for arm in ("mibz2-plan-3x3", "mibz2-plan-3x3p")]

# Registration 5, bench-plan-levels: 0.2.6's default, 0.2.6 forced to 4 x 2, 0.2.4's default,
# at 10 and 20 threads on the level-1, -2 and -3 recompressions. At levels 1 and 2 every block
# is below MIN_COOP, so no team ever starts its helpers: the decoders are the leaders alone
LEVELS_CORPORA = ("wikidata-l1", "wikidata-l2", "wikidata-l3")


def coop(corpus):
    return corpus == "wikidata-l3"


def p26(corpus):
    return P33P if coop(corpus) else (3, 0, 0)


def p42(corpus):
    return P42 if coop(corpus) else (4, 0, 0)


def levels_cells():
    return [
        Cell(f"L{t}", "all", t, {"mibz2": p26, "mibz2-plan-4x2": p42, "base": p42}, corpora=LEVELS_CORPORA)
        for t in (10, 20)
    ]


LEVELS = levels_cells()
# (name, reference, arm): S, the slowdown; K, the decoder count; R, everything else
LEVELS_COMPARISONS = [("S", "base", "mibz2"), ("K", "mibz2", "mibz2-plan-4x2"), ("R", "base", "mibz2-plan-4x2")]
INFERRED = (0.76, 0.83)


def plan_of(threads):
    """(leaders, plain helpers, pipelined helpers) from `thread_cpu` ("name=seconds ...") or
    `observed_threads` ("name ...")"""
    names = [p.partition("=")[0] for p in threads.split()]
    return (
        sum(1 for n in names if re.fullmatch(r"mibz2-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-p\d+", n)),
    )


def label(p):
    leaders, plain, piped = p
    helpers = plain + piped
    if not helpers:
        return f"{leaders} decoders, no helpers"
    if leaders and helpers % leaders == 0:
        kind = "p" if piped and not plain else "" if plain and not piped else " mixed"
        return f"{leaders}x{1 + helpers // leaders}{kind}"
    return f"{leaders} leaders + {plain} plain, {piped} pipelined helpers"


def cpu_list(s):
    out = []
    for part in s.strip().split(","):
        if not part:
            continue
        a, _, b = part.partition("-")
        out.extend(range(int(a), int(b or a) + 1))
    return out


def mask_cpus(mask, core_cpus):
    """The registered mask on this machine: the first c cores in the kernel's order (`ab env`'s
    `core_cpus`), one CPU each (its lowest-numbered) or every CPU of each"""
    if mask == "all":
        return None
    c, kind = mask
    cores = [sorted(cpu_list(x)) for x in core_cpus][:c]
    if len(cores) < c:
        return None
    return sorted(cpu for core in cores for cpu in (core[:1] if kind == "one" else core))


def arm_key(arm):
    """an arm's registered name: a base build's `mibz2@PATH` is "base" """
    return "base" if isinstance(arm, str) and arm.startswith("mibz2@") else arm


def stats(ratios, t):
    """(geometric mean, lower, upper, log mean, log half-width) of paired ratios"""
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    hw = t * statistics.stdev(logs) / math.sqrt(len(logs))
    return math.exp(m), math.exp(m - hw), math.exp(m + hw), m, hw


def word(g, lo, hi):
    if g >= FASTER_POINT and lo > 1.0:
        return "FASTER"
    if hi < 1.0:
        return "SLOWER"
    return "UNRESOLVED"


def lscpu_field(lscpu, name):
    m = re.search(rf"^{name}:\s*(.+)$", lscpu, re.M)
    return m.group(1).strip() if m else None


def ten_core(lscpu):
    sockets = lscpu_field(lscpu, r"Socket\(s\)")
    per = lscpu_field(lscpu, r"Core\(s\) per socket")
    tpc = lscpu_field(lscpu, r"Thread\(s\) per core")
    l3 = lscpu_field(lscpu, "L3 cache") or lscpu_field(lscpu, "L3") or ""
    ok = sockets == "1" and per == "10" and tpc == "2" and l3.split()[0:2] in (["20", "MiB"], ["20"])
    return ok, f"lscpu: {sockets} socket, {per} cores, {tpc} threads a core, L3 {l3 or None}"


def load(w, cells):
    """the attempt's files: (texts, rows by cell, unreadable lines)"""

    def text(name):
        try:
            with open(os.path.join(w, name)) as f:
                return f.read().strip()
        except OSError:
            return None

    rows, unreadable = {}, []
    for c in cells:
        rows[c.name] = []
        path = os.path.join(w, f"runs-{c.name}.jsonl")
        try:
            with open(path) as f:
                for n, line in enumerate(f, 1):
                    if not line.strip():
                        continue
                    try:
                        r = json.loads(line)
                    except ValueError:
                        unreadable.append(f"runs-{c.name}.jsonl line {n}")
                        continue
                    rows[c.name].append(r)
        except OSError:
            pass
    return text, rows, unreadable


def head_commit():
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def validity(w, cells, base_sha=None):
    """(checks [(text, ok)], runs by (cell, arm, corpus) -> {round: row})"""
    text, rows, unreadable = load(w, cells)
    checks = []
    checks.append((f"every line of the runs files JSON (unreadable: {', '.join(unreadable[:5]) or 'none'})", not unreadable))
    checks.append(("a clean tree (`git-status.txt` present and empty)", text("git-status.txt") == ""))
    checks.append(("no `NOT-FROZEN.txt` (the frozen inputs unchanged)", text("NOT-FROZEN.txt") is None))
    checks.append(("no `DRY-RUN.txt` (not a dry run)", text("DRY-RUN.txt") is None))
    commit, head = text("commit.txt"), head_commit()
    checks.append((f"`commit.txt` ({commit}) is the checkout's head ({head})", bool(commit) and commit == head))
    ok, what = ten_core(text("lscpu.txt") or "")
    checks.append((f"the ten-core topology: 1 socket, 10 cores, 2 threads a core, a 20 MiB L3 ({what})", ok))
    if base_sha:
        commits = text("commits.txt") or ""
        checks.append(
            (f"the base is v0.2.4's release commit {base_sha[:8]}, by its SHA (`commits.txt`)", f"base {base_sha}" in commits)
        )
    # the corpora, by digest
    want = sorted({corpus for c in cells for corpus in c.corpora})
    sums = {}
    for line in (text("corpus.sha256") or "").splitlines():
        parts = line.split()
        if len(parts) == 2:
            sums[os.path.basename(parts[1]).removesuffix(".bz2")] = parts[0]
    for corpus in want:
        checks.append(
            (f"the corpus `{corpus}` is the registered one (SHA-256 {LEVELS_SHA[corpus][:16]}...)", sums.get(corpus) == LEVELS_SHA[corpus])
        )
    try:
        env = json.loads(text("env.json") or "")
    except ValueError:
        env = {}
    core_cpus = env.get("core_cpus") or []
    masks = {}
    for line in (text("masks.txt") or "").splitlines():
        parts = line.split()
        if len(parts) == 2:
            masks[parts[0]] = parts[1]
    every = [r for c in cells for r in rows[c.name]]
    good = [r for r in every if isinstance(r, dict) and not r.get("err")]
    outputs = {(r.get("hash"), r.get("out")) for r in good}
    checks.append(
        (
            f"every run the same bytes, none failed ({len(every)} runs, {len(every) - len(good)} failed, {len(outputs)} outputs)",
            bool(every) and len(good) == len(every) and len(outputs) == 1,
        )
    )
    runs = {}
    for c in cells:
        rs = [r for r in rows[c.name] if isinstance(r, dict)]
        registered = mask_cpus(c.mask, core_cpus)
        want_mask = "all" if registered is None and c.mask == "all" else registered
        mask_line = masks.get(c.name)
        if c.mask == "all":
            mask_ok = mask_line == "all"
            allowed_ok = all(sorted(cpu_list(r.get("allowed", ""))) == sorted(sum((cpu_list(x) for x in core_cpus), [])) for r in rs)
        else:
            mask_ok = registered is not None and mask_line is not None and sorted(cpu_list(mask_line)) == registered
            allowed_ok = registered is not None and all(sorted(cpu_list(r.get("allowed", ""))) == registered for r in rs)
        shape = "every CPU" if c.mask == "all" else f"{c.mask[0]} cores, {'one CPU a core' if c.mask[1] == 'one' else 'both CPUs of each'}"
        checks.append(
            (
                f"`{c.name}`: the registered mask ({shape}: {want_mask if want_mask != 'all' else 'all'}), "
                f"recorded `{mask_line}`, every run under it",
                mask_ok and allowed_ok,
            )
        )
        stray = sorted({f"{arm_key(r.get('arm'))}@{r.get('threads')} on {r.get('corpus')}" for r in rs
                        if arm_key(r.get("arm")) not in c.arms or r.get("threads") != c.threads or r.get("corpus") not in c.corpora})
        checks.append((f"`{c.name}`: no arm, thread count or corpus beyond the registered (seen: {', '.join(stray) or 'none'})", not stray))
        if base_sha:
            bases = sorted({r.get("arm") for r in rs if arm_key(r.get("arm")) == "base"})
            checks.append((f"`{c.name}`: one base binary (seen {len(bases)})", len(bases) == 1))
        for arm in c.arms:
            for corpus in c.corpora:
                mine = [r for r in rs if arm_key(r.get("arm")) == arm and r.get("corpus") == corpus and r.get("threads") == c.threads]
                rounds = sorted(r.get("round") for r in mine)
                want_plan = c.plan(arm, corpus)
                seen = sorted(
                    {plan_of(r.get(k, "")) for r in mine for k in ("thread_cpu", "observed_threads") if isinstance(r.get(k), str)}
                )
                plans_ok = bool(mine) and all(
                    isinstance(r.get(k), str) and plan_of(r[k]) == want_plan for r in mine for k in ("thread_cpu", "observed_threads")
                )
                where = f"`{c.name}` `{arm}`" + (f" on `{corpus}`" if len(c.corpora) > 1 else "")
                checks.append((f"{where}: rounds 0-{ROUNDS - 1} once each (seen {len(rounds)})", rounds == list(range(ROUNDS))))
                checks.append(
                    (
                        f"{where}: every run's observed plan {label(want_plan)} (seen: {', '.join(map(label, seen)) or 'none'})",
                        plans_ok,
                    )
                )
                runs[(c.name, arm, corpus)] = {r["round"]: r for r in mine if isinstance(r.get("round"), int)}
    return checks, runs


def compare(runs, key_ref, key_arm, t):
    """the paired comparison of two cells' runs, or None where they don't pair fully"""
    ref, arm = runs.get(key_ref, {}), runs.get(key_arm, {})
    rr = [ref[i]["wall"] / arm[i]["wall"] for i in range(ROUNDS) if i in ref and i in arm
          and ref[i].get("wall", 0) > 0 and arm[i].get("wall", 0) > 0]
    if len(rr) != ROUNDS:
        return None
    g, lo, hi, m, hw = stats(rr, t)
    med = lambda rs, k: statistics.median(r.get(k) or math.nan for r in rs.values())  # noqa: E731
    return {
        "speed": g,
        "lo": lo,
        "hi": hi,
        "log": m,
        "hw": hw,
        "word": word(g, lo, hi),
        "cpu": med(arm, "cpu") / med(ref, "cpu"),
        "freq_ref": med(ref, "freq_mhz"),
        "freq_arm": med(arm, "freq_mhz"),
    }


def fmt(c):
    if not c:
        return "- (incomplete)"
    return f"{c['speed']:.3f} [{c['lo']:.3f}, {c['hi']:.3f}] **{c['word']}**, CPU {c['cpu']:.3f}"


def mhz(x):
    return "-" if x is None or (isinstance(x, float) and math.isnan(x)) else f"{x:.0f}"


# ---- registration 1 -------------------------------------------------------------------


def read_boundaries(res):
    """The registration's readings, mechanically, from the family's words"""
    w = lambda cell, arm: (res.get((cell, arm)) or {}).get("word", "UNRESOLVED")  # noqa: E731
    nine = w("U9", "mibz2-plan-3x3p")
    shared = {
        "FASTER": "SUPPORTS BOTH RULES (unmasked 9 threads: 3x3p faster than 4x2)",
        "SLOWER": "FALSIFIES BOTH RULES (unmasked 9 threads: 3x3p slower than 4x2)",
    }.get(nine, "LEAVES BOTH OPEN (unmasked 9 threads unresolved)")
    alts = [(c, a) for c in EIGHT for a in ("mibz2-plan-3x2", "mibz2-plan-2x4p") if (c, a) in res]
    a_false = [f"{c} {a}" for c, a in alts if w(c, a) == "FASTER"]
    rule_a = f"FALSIFIED AT 8 ({', '.join(a_false)} faster than 4x2)" if a_false else "NOT FALSIFIED AT 8"
    b_cells = [res.get((c, "mibz2-plan-3x2")) for c in EIGHT]
    if all(b and b["word"] == "FASTER" for b in b_cells):
        rule_b = "SUPPORTED AT 8 (3x2 faster at all three 8-thread cells)"
    elif all(b and b["hi"] < FASTER_POINT for b in b_cells):
        rule_b = "FALSIFIED AT 8 (3x2's Bonferroni upper bound below 1.03 at all three 8-thread cells)"
    else:
        rule_b = "NEITHER FALSIFIED NOR SUPPORTED AT 8"
    if not a_false and rule_b.startswith("NEITHER"):
        eight = (
            "NEITHER RESOLVED: the run could not separate the rules at 8 threads. This does not favour "
            "rule A; the default stays 0.2.6's because of the envelope, not because of this run"
        )
    else:
        eight = f"rule A {rule_a}; rule B {rule_b}"
    m9 = w("M9", "mibz2-plan-3x3p")
    core = (
        "FASTER: the core-left-over assumption is unnecessary; the 9-core shape becomes a candidate "
        "envelope entry, by its own registration"
        if m9 == "FASTER"
        else f"{m9}: nothing follows"
    )
    return {"nine": shared, "rule_a": rule_a, "rule_b": rule_b, "eight": eight, "nine_core_mask": core}


def boundaries(w):
    checks, runs = validity(w, BOUNDARIES)
    t = T19[2 * len(BOUNDARY_FAMILY)]
    res = {}
    for cell, arm in BOUNDARY_FAMILY:
        res[(cell, arm)] = compare(runs, (cell, "mibz2-plan-4x2", "wikidata"), (cell, arm, "wikidata"), t)
    controls = {c: compare(runs, (c, "mibz2", "wikidata"), (c, "mibz2-plan-3x2", "wikidata"), T19[2]) for c in ("C6", "C6M")}
    valid = all(ok for _, ok in checks)
    readings = read_boundaries(res) if valid else None
    lines = [
        "# bench-plan-boundaries: the general plan policy's boundaries on the i9 (diagnostic)",
        "",
        "Registration 1 of docs/journal/2026-11-07-general-plan-policy.md "
        "(docs/journal/2026-11-07-plan-boundaries-registration.md). Diagnostic only (RULES C1): "
        "it can't qualify a production change.",
        "",
        "## Validity",
        "",
    ]
    lines += [f"- {txt}: **{'yes' if ok else 'NO'}**" for txt, ok in checks]
    lines += ["", f"Run: **{'VALID' if valid else 'INVALID'}**", ""]
    lines += [
        "## The family: 7 comparisons against 4x2 (speed: 4x2's wall over the arm's; Bonferroni, "
        f"each interval at 1 - 0.05/14 a tail, t19 = {t})",
        "",
        "| cell | arm | speed [interval] word, CPU | MHz, 4x2 → arm | role |",
        "|---|---|---|---|---|",
    ]
    roles = {
        ("U9", "mibz2-plan-3x3p"): "the shared 9-thread cell",
        ("M9", "mibz2-plan-3x3p"): "information only (the core-left-over assumption)",
        ("M9", "mibz2-plan-3x2"): "in the family; enters no listed reading",
    }
    for cell, arm in BOUNDARY_FAMILY:
        c = res[(cell, arm)]
        role = roles.get((cell, arm), "8 threads: rules A and B")
        f = f"{mhz(c['freq_ref'])} → {mhz(c['freq_arm'])}" if c else "-"
        lines.append(f"| {cell} | `{arm}` | {fmt(c)} | {f} | {role} |")
    lines += ["", "## Controls (reported, not judged): today's default against 3x2 forced, the same plan (95% intervals, outside the family)", ""]
    for c, v in controls.items():
        lines.append(f"- {c}: `mibz2-plan-3x2` against `mibz2`: {fmt(v)}")
    lines += ["", "## Readings", ""]
    if readings:
        lines += [
            f"- **The shared 9-thread cell:** {readings['nine']}",
            f"- **Rule A at 8 threads:** {readings['rule_a']}",
            f"- **Rule B at 8 threads:** {readings['rule_b']}",
            f"- **At 8 threads:** {readings['eight']}",
            f"- **The 9-core mask (information only):** {readings['nine_core_mask']}",
            "- **What it does not show:** anything about the L3 extrapolation.",
        ]
    else:
        lines.append("None: the run is INVALID, so nothing is read from it.")
    out = {
        "verdict": "VALID" if valid else "INVALID",
        "checks": [[txt, ok] for txt, ok in checks],
        "comparisons": {f"{c} {a}": v for (c, a), v in res.items()},
        "controls": controls,
        "readings": readings,
    }
    return out, lines


# ---- registration 2 -------------------------------------------------------------------


def read_bytes(res):
    p6, p9 = res.get(("wikidata-l6", "mibz2-plan-3x3")), res.get(("wikidata-l9", "mibz2-plan-3x3"))
    if not p9 or not p9["lo"] > 1.0:
        return "INCONCLUSIVE: level 9 does not reproduce the three-team advantage (plain 3x3's lower bound not above 1)"
    if p6 and p6["lo"] > FASTER_POINT:
        return (
            "PREMISE CONTRADICTED: the three-team advantage survives tables that fit four to the L3 "
            "(level 6's lower bound above 1.03); this counts against rule A's L3 scaling and rule B's curve"
        )
    if p6 and p6["hi"] < FASTER_POINT and (p9["log"] - p6["log"]) > (p6["hw"] + p9["hw"]):
        return (
            "PREMISE SUPPORTED: level 6's upper bound below 1.03, and its estimate below level 9's by more than "
            "both intervals' half-widths combined; this fails to refute the premise on one machine, and "
            "validates no extrapolation"
        )
    return "UNRESOLVED"


def bytes_report(w):
    checks, runs = validity(w, BYTES)
    t = T19[2 * len(BYTES_FAMILY)]
    res = {}
    for corpus, arm in BYTES_FAMILY:
        res[(corpus, arm)] = compare(runs, ("B10", "mibz2-plan-4x2", corpus), ("B10", arm, corpus), t)
    valid = all(ok for _, ok in checks)
    reading = read_bytes(res) if valid else None
    lines = [
        "# bench-plan-bytes: does the table's size move the three-team advantage? (diagnostic)",
        "",
        "Registration 2 of docs/journal/2026-11-07-general-plan-policy.md "
        "(docs/journal/2026-11-07-plan-bytes-registration.md). Diagnostic only (RULES C1).",
        "",
        "## Validity",
        "",
    ]
    lines += [f"- {txt}: **{'yes' if ok else 'NO'}**" for txt, ok in checks]
    lines += ["", f"Run: **{'VALID' if valid else 'INVALID'}**", ""]
    lines += [
        f"## The family: 6 comparisons against 4x2 at 10 threads (Bonferroni, each interval at 1 - 0.05/12 a tail, t19 = {t})",
        "",
        "| level | arm | speed [interval] word, CPU | MHz, 4x2 → arm | role |",
        "|---|---|---|---|---|",
    ]
    for corpus, arm in BYTES_FAMILY:
        c = res[(corpus, arm)]
        role = "the decision rule" if arm == "mibz2-plan-3x3" and corpus != "wikidata-l7" else "reported"
        if corpus == "wikidata-l7" and arm == "mibz2-plan-3x3":
            role = "the boundary: reported, not judged"
        f = f"{mhz(c['freq_ref'])} → {mhz(c['freq_arm'])}" if c else "-"
        lines.append(f"| {corpus[-1]} | `{arm}` | {fmt(c)} | {f} | {role} |")
    lines += ["", "## Reading (plain 3x3 against 4x2)", "", reading or "None: the run is INVALID, so nothing is read from it."]
    out = {
        "verdict": "VALID" if valid else "INVALID",
        "checks": [[txt, ok] for txt, ok in checks],
        "comparisons": {f"{c} {a}": v for (c, a), v in res.items()},
        "reading": reading,
    }
    return out, lines


# ---- registration 5 -------------------------------------------------------------------


def read_levels(res):
    """per level 1 and 2: the slowdown, and if shown, what it comes from"""
    w = lambda name, corpus, t: (res.get((name, corpus, t)) or {}).get("word", "UNRESOLVED")  # noqa: E731
    out = {}
    for corpus in ("wikidata-l1", "wikidata-l2"):
        s = [w("S", corpus, t) for t in (10, 20)]
        if s == ["SLOWER", "SLOWER"]:
            k = [w("K", corpus, t) for t in (10, 20)]
            r = [w("R", corpus, t) for t in (10, 20)]
            count, rest = k == ["FASTER", "FASTER"], r == ["SLOWER", "SLOWER"]
            if count and rest:
                why = "BOTH: forcing four decoders is faster than 0.2.6's default, and 0.2.6 forced to 4x2 is still slower than 0.2.4"
            elif count and "SLOWER" not in r:
                why = "THE DECODER COUNT: the fix to register is the scheduling one (the journal's registration 5)"
            elif rest and "FASTER" not in k:
                why = "OTHER CHANGES between the releases, not the decoder count"
            else:
                why = "NEITHER RESOLVED: the run could not separate the decoder count from the other changes"
            out[corpus] = f"SLOWDOWN SHOWN (0.2.6 slower than 0.2.4 at 10 and 20 threads); from: {why}"
        else:
            his = [res.get(("S", corpus, t)) for t in (10, 20)]
            beyond = all(h and h["lo"] > INFERRED[1] for h in his)
            out[corpus] = "SLOWDOWN NOT SHOWN" + (
                f" (both intervals lie wholly above the inferred {INFERRED[0]}-{INFERRED[1]})" if beyond else ""
            )
    return out


def levels(w):
    checks, runs = validity(w, LEVELS, base_sha=V024)
    res = {}
    for name, ref, arm in LEVELS_COMPARISONS:
        for corpus in LEVELS_CORPORA:
            t = T19[24] if corpus != "wikidata-l3" else T19[12]
            for th in (10, 20):
                cell = f"L{th}"
                res[(name, corpus, th)] = compare(runs, (cell, ref, corpus), (cell, arm, corpus), t)
    valid = all(ok for _, ok in checks)
    readings = read_levels(res) if valid else None
    lines = [
        "# bench-plan-levels: levels 1 and 2 on 0.2.6 against 0.2.4 (diagnostic)",
        "",
        "Registration 5 of docs/journal/2026-11-07-general-plan-policy.md "
        "(docs/journal/2026-11-07-plan-levels-registration.md). Diagnostic only (RULES C1).",
        "",
        "## Validity",
        "",
    ]
    lines += [f"- {txt}: **{'yes' if ok else 'NO'}**" for txt, ok in checks]
    lines += ["", f"Run: **{'VALID' if valid else 'INVALID'}**", ""]
    what = {
        "S": "0.2.6's default against 0.2.4's (the slowdown)",
        "K": "0.2.6 forced to 4x2 against 0.2.6's default (the decoder count)",
        "R": "0.2.6 forced to 4x2 against 0.2.4's default (the same plan: everything else)",
    }
    for title, corpora, t in (
        (f"Levels 1 and 2: the family of 12 (Bonferroni, each interval at 1 - 0.05/24 a tail, t19 = {T19[24]})", ("wikidata-l1", "wikidata-l2"), None),
        (f"Level 3, a separate question: its own family of 6 (each interval at 1 - 0.05/12 a tail, t19 = {T19[12]}), in no reading", ("wikidata-l3",), None),
    ):
        lines += ["", f"## {title}", "", "| level | threads | comparison | speed [interval] word, CPU | MHz, ref → arm |", "|---|--:|---|---|---|"]
        for corpus in corpora:
            for th in (10, 20):
                for name, _, _ in LEVELS_COMPARISONS:
                    c = res[(name, corpus, th)]
                    f = f"{mhz(c['freq_ref'])} → {mhz(c['freq_arm'])}" if c else "-"
                    lines.append(f"| {corpus[-1]} | {th} | {name}: {what[name]} | {fmt(c)} | {f} |")
    lines += ["", "## Readings (levels 1 and 2)", ""]
    if readings:
        lines += [f"- **Level {c[-1]}:** {r}" for c, r in readings.items()]
        lines.append(f"- The inference under test: 0.2.6 at {INFERRED[0]}-{INFERRED[1]} of 0.2.4 (§1.6).")
    else:
        lines.append("None: the run is INVALID, so nothing is read from it.")
    out = {
        "verdict": "VALID" if valid else "INVALID",
        "checks": [[txt, ok] for txt, ok in checks],
        "comparisons": {f"{n} {c} {t}": v for (n, c, t), v in res.items()},
        "readings": readings,
    }
    return out, lines


CELLS = {"boundaries": BOUNDARIES, "bytes": BYTES, "levels": LEVELS}


def cells_here(which, env_path, dry):
    """`NAME MASK THREADS ARMS CORPORA` a cell, the mask on this machine's cores"""
    core_cpus = json.load(open(env_path)).get("core_cpus") or []
    out = []
    for c in CELLS[which]:
        if c.mask == "all":
            mask = "all"
        else:
            cpus = mask_cpus(c.mask, core_cpus)
            if cpus is None:
                if not dry:
                    raise SystemExit(f"{c.name}: this machine has {len(core_cpus)} cores, fewer than the mask's {c.mask[0]}")
                cpus = mask_cpus((len(core_cpus), c.mask[1]), core_cpus)
                print(f"DRY: {c.name}'s mask narrowed to this machine's {len(core_cpus)} cores", file=sys.stderr)
            mask = ",".join(map(str, cpus))
        out.append(f"{c.name} {mask} {c.threads} {','.join(c.arms)} {','.join(c.corpora)}")
    return out


def main():
    if len(sys.argv) > 1 and sys.argv[1] in ("cells", "topology", "sha"):
        cmd, args = sys.argv[1], sys.argv[2:]
        if cmd == "cells":
            print("\n".join(cells_here(args[0], args[1], "--dry" in args[2:])))
            return 0
        if cmd == "topology":
            ok, what = ten_core(open(args[0]).read())
            print(what)
            return 0 if ok else 1
        print(LEVELS_SHA[args[0]])
        return 0
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("which", choices=["boundaries", "bytes", "levels"])
    ap.add_argument("dir")
    ap.add_argument("--out")
    a = ap.parse_args()
    out, lines = {"boundaries": boundaries, "bytes": bytes_report, "levels": levels}[a.which](a.dir)
    md = "\n".join(lines) + "\n"
    print(md)
    if a.out:
        with open(a.out + ".md", "w") as f:
            f.write(md)
        with open(a.out + ".json", "w") as f:
            json.dump(out, f, indent=1, default=str)
    return 0 if out["verdict"] == "VALID" else 1


if __name__ == "__main__":
    sys.exit(main())
