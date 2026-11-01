"""P87's four-core diagnostic: the registered analysis (docs/journal/2026-11-01-p87-four-core-protocol.md).
No timing here. The statistics, ROW format, A/A calibration and RSS rule are the walk
trial's (research/walk-unroll-decoder bench/walk_decoder.py, f5c492d); the decisions are
P87's own. Subcommands:

- manifest: the trial's manifest (TSV: name, role, path, compressed SHA-256, decoded
  SHA-256, decoded bytes) from frozen.json and the corpora's directory, every hash checked.
- freeze: frozen.json (the corpora copied from the dispatch diagnostic's frozen record,
  the walk trial's files; the harness sources' hashes; the schedule and thresholds).
- summary: stage S1, the uninstrumented paired trial: the mechanism D, the engineering
  gate, the controls, RSS, A/A, correctness and placement balance (§7).
- phases: stage S2, the phase attribution from the trace's marks, and its perturbation
  against paired uninstrumented runs of the same instrument (§8).
- counters: stage S3, DRAM reads and writes (the memory controller's, machine-wide, less an
  idle reading as long) and the core counters (instructions, cycles, reference cycles)
  over whole decodes (§8).
- verdict: S1's decisions with S3's validity check applied, the outcome matrix, and every
  stage's figures beside them; printed and written.
"""
import argparse
import hashlib
import json
import math
import re
import statistics
import sys
from pathlib import Path

# ---------------------------------------------------------------- the registered constants

ROUNDS = 20
ARMS = ["A", "B", "A2"]
PRIMARY = "wikidata-20261026"
CONTROLS = ["records", "random"]
WORKERS = [1, 4]
D_THRESHOLD = 0.10            # CPU-s/GB: the mechanism's material size
PRACTICAL = (1.02, 0.98)      # descriptive: throughput lower95 >=, CPU upper95 <=
NO_HARM = (1.01, 0.99)        # controls: CPU upper95 <=, throughput lower95 >=
RSS_MAX = 1.01                # the walk trial's max rule
INSTR_W1 = 0.005              # S3: |median instructions B/A - 1| at w = 1, a validity check
INSTR_W4 = 0.01               # S3: at w = 4, reported (lock spinning is timing-dependent)
PERTURB = 0.03                # S2: the trace's cost, |median full/off - 1|, for its figures to be read
T = "docs/journal/2026-11-01-p87-four-core"
SOURCES = [
    "Cargo.toml", "Cargo.lock", "src/lib.rs", "src/p87.rs", "src/bwt.rs", "src/trace.rs", "src/mtf.rs",
    "src/parallel.rs", "src/rle.rs", "src/crc.rs", "src/read.rs",
    "examples/p87_decoder.rs", "examples/occupancy.rs", "examples/attribution_decode.rs",
    "tests/common/mod.rs", "tests/p87.rs", "tests/p87_parallel.rs",
    "bench/p87.py", "bench/test_p87.py", "bench/p87_code_audit.py", "bench/attribution_same_code.py",
    "bench/attribution.py", "bench/p87_run.sh", ".just/bench.just",
    "docs/journal/2026-11-01-p87-four-core-protocol.md",
]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def manifest(a):
    f = json.loads(Path(a.frozen).read_text())
    rows = []
    for name in f["order"]:
        c = f["corpora"][name]
        p = Path(a.dir) / f"{name}.bz2"
        if not p.exists() or sha(p) != c["compressed_sha256"]:
            raise SystemExit(f"manifest: {p} missing or differing from the frozen hash")
        rows.append("\t".join([name, c["role"], str(p.resolve()), c["compressed_sha256"], c["decoded_sha256"], str(c["decoded_bytes"])]))
    Path(a.out).write_text("\n".join(rows) + "\n")
    print(f"manifest: {len(rows)} corpora match the frozen SHA-256")


def freeze(a):
    ref = json.loads(Path(a.corpora_from).read_text())
    out = dict(
        protocol="docs/journal/2026-11-01-p87-four-core-protocol.md",
        pin=a.pin, source_base=a.source_base,
        toolchain="rustc 1.97.0 (2d8144b78 2026-07-07), --locked, release profile, empty RUSTFLAGS",
        order=ref["order"], corpora=ref["corpora"],
        corpora_from=dict(record=a.corpora_from, **ref["corpora_from"]),
        schedule=dict(
            S1=dict(rounds=ROUNDS, warmups="one untimed child per arm per cell (round -1)", workers=WORKERS,
                    arms=["A: assembly into rle", "B: assembly into ll", "A2: A again, a second label"],
                    design="per cell: the 6 permutations of (A, B, A2) x3 + (A,B,A2), (B,A,A2), Fisher-Yates; cell order shuffled per round; xorshift seed 0x8720261101_0001",
                    placement="the round's parity: 0 (ll allocated first) in even rounds, 1 (rle first) in odd; the warm-ups 0"),
            S2=dict(reps=6, corpus=PRIMARY, workers=WORKERS, arms=["A", "B"], modes=["off (level 0)", "full (level 2)"],
                    order="per rep: w = 1 then 4; arm order AB on even reps, BA on odd; mode order off-full on even, full-off on odd; placement the rep's parity"),
            S3=dict(reps=6, corpus=PRIMARY, workers=WORKERS, arms=["A", "B"],
                    passes=["IMC machine-wide (an idle reading as long, just before)", "core: cycles, ref-cycles, instructions (the process)"],
                    order="as S2")),
        thresholds=dict(D=D_THRESHOLD, practical=PRACTICAL, no_harm=NO_HARM, rss_max=RSS_MAX,
                        instructions_w1=INSTR_W1, instructions_w4=INSTR_W4, perturbation=PERTURB),
        sources_sha256={p: sha(p) for p in SOURCES})
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(f"freeze: {len(out['order'])} corpora, {len(SOURCES)} sources")


# ---------------------------------------------------------------- statistics (the stage protocol's)

def _betacf(a, b, x):
    qab, qap, qam = a + b, a + 1, a - 1
    c, d = 1.0, 1 - qab * x / qap
    d = 1 / (d if abs(d) > 1e-300 else 1e-300)
    h = d
    for m in range(1, 400):
        m2 = 2 * m
        for aa in (m * (b - m) * x / ((qam + m2) * (a + m2)), -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))):
            d = 1 + aa * d
            d = 1 / (d if abs(d) > 1e-300 else 1e-300)
            c = 1 + aa / c
            c = c if abs(c) > 1e-300 else 1e-300
            h *= d * c
        if abs(d * c - 1) < 1e-15:
            break
    return h


def _ibeta(a, b, x):
    if x <= 0 or x >= 1:
        return float(x >= 1)
    lb = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log(1 - x)
    if x < (a + 1) / (a + b + 2):
        return math.exp(lb) * _betacf(a, b, x) / a
    return 1 - math.exp(lb) * _betacf(b, a, 1 - x) / b


def t_cdf(t, df):
    x = df / (df + t * t)
    tail = 0.5 * _ibeta(df / 2, 0.5, x)
    return 1 - tail if t > 0 else tail


def t_quantile(p, df):
    lo, hi = -1e4, 1e4
    for _ in range(200):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if t_cdf(mid, df) < p else (lo, mid)
    return (lo + hi) / 2


def _interval(x, lag1=True):
    """mean, and a two-sided 95% t interval: for lag1, the lag-1 autocorrelation rho in
    round order, n_eff = R(1-rho)/(1+rho) for rho > 0, df = max(1, floor(n_eff)-1)"""
    r = len(x)
    m = statistics.mean(x)
    sd = statistics.stdev(x) if r > 1 else 0.0
    den = sum((v - m) ** 2 for v in x)
    rho = sum((x[i] - m) * (x[i + 1] - m) for i in range(r - 1)) / den if lag1 and den > 0 else 0.0
    n_eff = r * (1 - rho) / (1 + rho) if rho > 0 else r
    df = max(1, math.floor(n_eff) - 1)
    se = sd / math.sqrt(n_eff)
    return m, se, df, rho, n_eff, t_quantile(0.975, df) * se


def paired(x):
    """per-round log ratios: the ratio and its interval"""
    m, se, df, rho, n_eff, half = _interval(x)
    return dict(mean_log=m, se=se, df=df, rho1=rho, n_eff=n_eff, ratio=math.exp(m),
                lower95=math.exp(m - half), upper95=math.exp(m + half))


def paired_diff(x, lag1=True):
    """per-round (or per-repetition) differences on the raw scale: the mean and its interval"""
    m, se, df, rho, n_eff, half = _interval(x, lag1)
    return dict(mean=m, se=se, df=df, rho1=rho, n_eff=n_eff, lower95=m - half, upper95=m + half)


def binom_tail(m, k, q=0.05):
    return sum(math.comb(m, i) * q ** i * (1 - q) ** (m - i) for i in range(k, m + 1))


def outcome(neg, ok, good="PASS"):
    return "NEGATIVE" if neg else good if ok else "INCONCLUSIVE"


# ---------------------------------------------------------------- S1: the paired trial

def read_raw(path):
    """ROW fields (examples/p87_decoder.rs, the walk trial's with the placement last): round,
    cell position, corpus index, name, w, arm, position, then the child's: wall ns, decode
    user us, decode sys us, bytes, sha ok, blocks, selections, plan, allowed CPUs, physical
    cores, threads (spawned), their snapshot, threads (end), their snapshot; then the
    parent's: life user us, life sys us, maxrss KiB, minflt, majflt, nvcsw, nivcsw, warm-up
    flag, child life ns, placement."""
    rows = []
    for line in Path(path).read_text().splitlines():
        f = line.split("\t")
        if f[0] != "ROW":
            continue
        assert len(f) == 32, f"ROW with {len(f)} fields"
        rows.append(dict(round=int(f[1]), cell_pos=int(f[2]), corpus=int(f[3]), name=f[4], w=int(f[5]),
                         arm=int(f[6]), position=int(f[7]), wall_ns=int(f[8]), user_us=int(f[9]),
                         sys_us=int(f[10]), bytes=int(f[11]), sha_ok=f[12] == "1", plan=f[15], allowed=f[16],
                         phys=int(f[17]), threads_spawned=int(f[18]), threads_spawned_list=f[19],
                         threads_end=int(f[20]), threads_end_list=f[21], life_user_us=int(f[22]),
                         life_sys_us=int(f[23]), maxrss_kb=int(f[24]), minflt=int(f[25]), majflt=int(f[26]),
                         nvcsw=int(f[27]), nivcsw=int(f[28]), warm=f[29] == "1", life_ns=int(f[30]),
                         placement=int(f[31])))
    return rows


def cpu_of(r):
    """the decode's CPU-seconds per decoded GB"""
    return (r["user_us"] + r["sys_us"]) / 1e6 / (r["bytes"] / 1e9)


def mb_s(r):
    return r["bytes"] / r["wall_ns"] * 1e3


def cpu_busy(before, after, alloc, siblings):
    """/proc/stat deltas per CPU: busy fraction (not idle/iowait), steal and irq fractions"""
    def parse(p):
        out = {}
        for l in Path(p).read_text().splitlines():
            f = l.split()
            if f and f[0].startswith("cpu") and f[0] != "cpu":
                out[int(f[0][3:])] = [int(x) for x in f[1:]]
        return out
    b, e = parse(before), parse(after)
    res = {}
    for c in sorted(set(b) & set(e)):
        d = [y - x for x, y in zip(b[c], e[c])]
        total = sum(d[:8]) or 1
        idle = d[3] + d[4]
        role = "allocated" if c in alloc else "idle sibling" if c in siblings else "other"
        res[str(c)] = dict(role=role, ticks=total, busy=(total - idle) / total, irq=(d[5] + d[6]) / total, steal=d[7] / total)
    return res


def mechanism(obs, other):
    """D = [C(A,4) - C(other,4)] - [C(A,1) - C(other,1)] per round, CPU-s/GB on the
    primary; its interval; and the two worker counts' differences apart"""
    def c(w, rd, arm):
        return cpu_of(obs[PRIMARY, w, rd, arm])
    o = ARMS.index(other)
    d1 = [c(1, rd, 0) - c(1, rd, o) for rd in range(ROUNDS)]
    d4 = [c(4, rd, 0) - c(4, rd, o) for rd in range(ROUNDS)]
    dd = [y - x for x, y in zip(d1, d4)]
    res = dict(D=paired_diff(dd), delta_w1=paired_diff(d1), delta_w4=paired_diff(d4),
               by_placement={str(p): statistics.mean(dd[rd] for rd in range(ROUNDS) if rd % 2 == p) for p in (0, 1)})
    D = res["D"]
    res["outcome"] = ("MATERIAL" if D["lower95"] >= D_THRESHOLD else
                      "NOT MATERIAL" if D["upper95"] < D_THRESHOLD else "INCONCLUSIVE")
    return res


def summary(a):
    man = [l.split("\t") for l in Path(a.manifest).read_text().splitlines() if l]
    roles = {m[0]: m[1] for m in man}
    names = [m[0] for m in man]
    rows = read_raw(a.raw)
    alloc_doc = json.loads(Path(a.allocation).read_text())
    alloc = set(alloc_doc["alloc"])
    timed = [r for r in rows if not r["warm"]]
    workers = sorted({r["w"] for r in timed})
    obs = {}
    for r in timed:
        key = (r["name"], r["w"], r["round"], r["arm"])
        assert key not in obs, f"duplicate observation {key}"
        obs[key] = r
    want = {(n, w, rd, arm) for n in names for w in workers for rd in range(ROUNDS) for arm in range(3)}
    assert set(obs) == want, "incomplete or extra observations: no summary"
    correctness = dict(all_sha_ok=all(r["sha_ok"] for r in rows),
                       all_workers_plan=all(r["plan"] == f"Workers({r['w']})" for r in rows),
                       threads=all(r["threads_spawned"] == r["w"] + 3 for r in rows))
    flags = dict(ok=True, exited=0)

    def threads_of(lst, allow_exited):
        out = []
        for t in (t for t in lst.split(";") if t):
            if t.startswith("EXITED:"):
                flags["exited"] += 1
                flags["ok"] &= allow_exited
                continue
            if t.startswith("UNRESOLVED:") or len(t.split("|")) < 3 or not t.split("|")[1]:
                flags["ok"] = False
                continue
            out.append(t.split("|"))
        return out
    confined = True
    for r in rows:
        for t in threads_of(r["threads_spawned_list"], False) + threads_of(r["threads_end_list"], True):
            cpus = set()
            for part in t[1].split(","):
                x, _, y = part.partition("-")
                cpus.update(range(int(x), int(y or x) + 1))
            confined &= bool(cpus) and cpus <= alloc
    correctness["all_threads_within_allocation"] = confined and flags["ok"]
    # placement: the round's parity, every arm alike, each arm half its rounds each way
    placement_ok = all(r["placement"] == (r["round"] % 2 if r["round"] >= 0 else 0) for r in rows)
    balance = {}
    for n in names:
        for w in workers:
            for i, arm in enumerate(ARMS):
                ps = [obs[n, w, rd, i]["placement"] for rd in range(ROUNDS)]
                balance[f"{n} w={w} {arm}"] = [ps.count(0), ps.count(1)]
    placement_ok &= all(v == [ROUNDS // 2, ROUNDS // 2] for v in balance.values())
    out_rows = []
    for n in names:
        for w in workers:
            t = {arm: [obs[n, w, rd, i] for rd in range(ROUNDS)] for i, arm in enumerate(ARMS)}
            x = {arm: [mb_s(r) for r in v] for arm, v in t.items()}
            cpu = {arm: [cpu_of(r) for r in v] for arm, v in t.items()}
            rss = {arm: [r["maxrss_kb"] for r in v] for arm, v in t.items()}
            row = dict(corpus=n, role=roles[n], w=w,
                       MB_s={k: statistics.mean(v) for k, v in x.items()},
                       cpu_s_per_GB={k: statistics.mean(v) for k, v in cpu.items()},
                       maxrss_kb_max={k: max(v) for k, v in rss.items()},
                       minflt_mean={k: statistics.mean(r["minflt"] for r in v) for k, v in t.items()})
            for arm in ("B", "A2"):
                row[f"X {arm}/A"] = paired([math.log(p / q) for p, q in zip(x[arm], x["A"])])
                row[f"CPU {arm}/A"] = paired([math.log(p / q) for p, q in zip(cpu[arm], cpu["A"])])
                lr = [math.log(p / q) for p, q in zip(rss[arm], rss["A"])]
                row[f"RSS {arm}/A"] = dict(paired(lr), max_rule=max(rss[arm]) / max(rss["A"]))
            out_rows.append(row)

    def get(n, w):
        return next(r for r in out_rows if r["corpus"] == n and r["w"] == w)
    complete = PRIMARY in names and set(WORKERS) <= set(workers) and all(n in names for n in CONTROLS)
    res = dict(correctness=correctness, placement=dict(ok=placement_ok, balance=balance), complete=complete)
    if complete:
        res["mechanism"] = mechanism(obs, "B")
        res["mechanism_AA"] = mechanism(obs, "A2")
        # A/A: the D interval, and the walk trial's calibration over every row
        aa = [get(n, w)[k] for n in names for w in workers for k in ("X A2/A", "CPU A2/A")]
        excl = sum(1 for r in aa if not (r["lower95"] <= 1 <= r["upper95"]))
        k_crit = next(k for k in range(len(aa) + 2) if binom_tail(len(aa), k) <= 0.05)
        daa = res["mechanism_AA"]["D"]
        res["aa"] = dict(rows=len(aa), excluding_one=excl, threshold=k_crit, calibration_ok=excl < k_crit,
                         D_AA_contains_0=daa["lower95"] <= 0 <= daa["upper95"])
        res["aa"]["ok"] = res["aa"]["calibration_ok"] and res["aa"]["D_AA_contains_0"]
        p = get(PRIMARY, 4)
        x, c = p["X B/A"], p["CPU B/A"]
        prim = dict(X=x, CPU=c, improves=c["upper95"] < 1 and x["lower95"] > 1,
                    worse=c["lower95"] > 1 or x["upper95"] < 1,
                    practical=x["lower95"] >= PRACTICAL[0] and c["upper95"] <= PRACTICAL[1])
        harm = {}
        for n in names:
            for w in workers:
                if (n, w) == (PRIMARY, 4):
                    continue
                r = get(n, w)
                harm[f"{n} w={w}"] = outcome(r["CPU B/A"]["lower95"] > NO_HARM[0] or r["X B/A"]["upper95"] < NO_HARM[1],
                                             r["CPU B/A"]["upper95"] <= NO_HARM[0] and r["X B/A"]["lower95"] >= NO_HARM[1])
        rss_g = {}
        for n in names:
            for w in workers:
                r = get(n, w)["RSS B/A"]
                rss_g[f"{n} w={w}"] = outcome(r["lower95"] > RSS_MAX, r["max_rule"] <= RSS_MAX)
        res["engineering"] = dict(primary_w4=prim, no_harm=harm, rss=rss_g)
    res["decisions"] = decisions(res)
    host = {}
    if a.stat_before and a.stat_after:
        host = cpu_busy(a.stat_before, a.stat_after, alloc, set(alloc_doc.get("idle_siblings", [])))
    Path(a.out).write_text(json.dumps(dict(protocol="2026-11-01-p87-four-core-protocol", stage="S1", rounds=ROUNDS,
                                           workers=workers, allocation=sorted(alloc), result=res, rows=out_rows,
                                           host=host), indent=1) + "\n")
    print(json.dumps(res["decisions"], indent=1))


def decisions(res, instr=None):
    """S1's two decisions (§7), each on its own; `instr`, S3's instruction check at w = 1
    (None: not measured), can only make the mechanism inconclusive"""
    c = res["correctness"]
    if not all(c.values()):
        bad = "NEGATIVE (correctness)" if not c["all_sha_ok"] else "ABORTED (correctness precondition)"
        return dict(mechanism=bad, engineering=bad, matrix=None)
    if not res.get("complete"):
        return dict(mechanism="NOT APPLICABLE (incomplete design)", engineering="NOT APPLICABLE (incomplete design)", matrix=None)
    if not res["placement"]["ok"]:
        return dict(mechanism="ABORTED (placement unbalanced)", engineering="ABORTED (placement unbalanced)", matrix=None)
    aa_ok = res["aa"]["ok"]
    if not aa_ok:
        mech = "INCONCLUSIVE (A/A calibration)"
    elif instr is not None and not instr:
        mech = "INCONCLUSIVE (the arms' instructions differ at w = 1)"
    else:
        mech = res["mechanism"]["outcome"]
    e = res["engineering"]
    if not aa_ok:
        eng = "INCONCLUSIVE (A/A calibration)"
    elif e["primary_w4"]["worse"] or "NEGATIVE" in e["no_harm"].values() or "NEGATIVE" in e["rss"].values():
        eng = "NEGATIVE"
    elif e["primary_w4"]["improves"] and all(v == "PASS" for v in e["no_harm"].values()) and all(v == "PASS" for v in e["rss"].values()):
        eng = "IMPROVES"
    else:
        eng = "NOT DEMONSTRATED"
    return dict(mechanism=mech, engineering=eng, practical=e["primary_w4"]["practical"],
                matrix=MATRIX.get((mech.split(" (")[0], eng)))


MATRIX = {
    ("MATERIAL", "IMPROVES"): "The cross-worker term is real and B is faster at four workers: a candidate for a production change, which needs its own registered trial (B there would also free rle).",
    ("MATERIAL", "NOT DEMONSTRATED"): "The cross-worker term is real, but B's net effect at four workers is not shown: a single-worker cost offsets it, or the trial lacks power. No engineering change.",
    ("MATERIAL", "NEGATIVE"): "The cross-worker term is real, but B is slower or harms a control: the saving is outweighed. No engineering change; the mechanism is informative for other designs.",
    ("NOT MATERIAL", "IMPROVES"): "B is faster, but not through a cross-worker effect of this size: a local or persistent single-worker effect. An engineering candidate; the four-core contention hypothesis is not supported.",
    ("NOT MATERIAL", "NOT DEMONSTRATED"): "No material cross-worker term and no shown gain: P87's four-core diagnostic closes negative.",
    ("NOT MATERIAL", "NEGATIVE"): "No material cross-worker term, and B is slower or harmful: P87's four-core diagnostic closes negative.",
    ("INCONCLUSIVE", "IMPROVES"): "B is faster, the cross-worker term undetermined: an engineering candidate; the mechanism stays open.",
    ("INCONCLUSIVE", "NOT DEMONSTRATED"): "Neither decision reached: nothing changes; the record says which interval crossed which bound.",
    ("INCONCLUSIVE", "NEGATIVE"): "B is slower or harmful, the cross-worker term undetermined: no engineering change.",
}


# ---------------------------------------------------------------- S2: phases from the marks

PHASES = ["mtf", "build", "traversal", "short_walk", "assembly", "post"]
WORKER = re.compile(r"mibz2-\d+$")


def phase_totals(marks_path):
    """per run: each phase's thread CPU ns and elapsed ns summed over the workers' blocks,
    the blocks, the BWT rows and the decoded bytes. A block's marks in order: DecodeStart,
    InvertStart (len: rows), WalkStart, AssembleStart (segmented blocks only), InvertEnd,
    DecodeEnd (len: decoded bytes). A block whose marks are out of order or unclocked
    makes the run unusable."""
    rows = [l.split("\t") for l in Path(marks_path).read_text().splitlines()[1:] if l]
    by_thread = {}
    for f in rows:
        if WORKER.match(f[1]):
            by_thread.setdefault(f[0], []).append(f)
    cpu = dict.fromkeys(PHASES, 0)
    ela = dict.fromkeys(PHASES, 0)
    blocks = segmented = n_rows = seg_rows = decoded = 0
    errors = []
    for marks in by_thread.values():
        cur = None
        for f in marks:
            kind, ns, c = f[3], int(f[2]), f[5]
            if kind not in ("DecodeStart", "InvertStart", "WalkStart", "AssembleStart", "InvertEnd", "DecodeEnd"):
                continue
            if c == "-":
                errors.append(f"{kind} unclocked")
                cur = None
                continue
            c = int(c)
            if kind == "DecodeStart":
                cur = [("DecodeStart", ns, c, None)]
                continue
            if cur is None:
                errors.append(f"{kind} outside a block")
                continue
            cur.append((kind, ns, c, f[9]))
            if kind != "DecodeEnd":
                continue
            seq = [k for k, *_ in cur]
            at = {k: (n, cc, ln) for k, n, cc, ln in cur}
            if seq not in (["DecodeStart", "InvertStart", "WalkStart", "AssembleStart", "InvertEnd", "DecodeEnd"],
                           ["DecodeStart", "InvertStart", "WalkStart", "InvertEnd", "DecodeEnd"]):
                errors.append(f"marks {seq}")
                cur = None
                continue

            def add(ph, k0, k1):
                cpu[ph] += at[k1][1] - at[k0][1]
                ela[ph] += at[k1][0] - at[k0][0]
            add("mtf", "DecodeStart", "InvertStart")
            add("build", "InvertStart", "WalkStart")
            if "AssembleStart" in at:
                add("traversal", "WalkStart", "AssembleStart")
                add("assembly", "AssembleStart", "InvertEnd")
                segmented += 1
                seg_rows += int(at["InvertStart"][2])
            else:
                add("short_walk", "WalkStart", "InvertEnd")
            add("post", "InvertEnd", "DecodeEnd")
            blocks += 1
            n_rows += int(at["InvertStart"][2])
            decoded += int(at["DecodeEnd"][2])
            cur = None
    return dict(cpu_ns=cpu, elapsed_ns=ela, blocks=blocks, segmented=segmented, rows=n_rows, segmented_rows=seg_rows,
                decoded=decoded, errors=errors[:20], n_errors=len(errors))


def read_pairs(path):
    out = []
    for l in Path(path).read_text().splitlines():
        if not l or l.startswith("#"):
            continue
        f = l.split("\t")
        out.append(dict(w=int(f[0]), rep=int(f[1]), arm=f[2], mode=f[3], wall=float(f[4]), cpu=float(f[5]),
                        out=int(f[6]), rss_kb=int(f[7]), placement=int(f[8]), marks=f[9] if len(f) > 9 else "-"))
    return out


def phases(a):
    pairs = read_pairs(a.pairs)
    reps = sorted({p["rep"] for p in pairs})
    res = dict(perturbation={}, phases={}, errors={})
    for w in WORKERS:
        for arm in ("A", "B"):
            off = {p["rep"]: p for p in pairs if (p["w"], p["arm"], p["mode"]) == (w, arm, "off")}
            full = {p["rep"]: p for p in pairs if (p["w"], p["arm"], p["mode"]) == (w, arm, "full")}
            if set(off) != set(reps) or set(full) != set(reps):
                continue
            rc = [full[r]["cpu"] / off[r]["cpu"] for r in reps]
            rw = [full[r]["wall"] / off[r]["wall"] for r in reps]
            res["perturbation"][f"w={w} {arm}"] = dict(cpu_median=statistics.median(rc), wall_median=statistics.median(rw),
                                                       ok=abs(statistics.median(rc) - 1) <= PERTURB and abs(statistics.median(rw) - 1) <= PERTURB)
    totals = {}
    for p in pairs:
        if p["mode"] == "full" and p["marks"] != "-":
            t = phase_totals(Path(a.runs) / p["marks"])
            totals[p["w"], p["arm"], p["rep"]] = t
            if t["n_errors"] or t["decoded"] != p["out"]:
                res["errors"][f"w={p['w']} {p['arm']} rep={p['rep']}"] = dict(marks=t["errors"], decoded=t["decoded"], out=p["out"])
    for w in WORKERS:
        have = [r for r in reps if (w, "A", r) in totals and (w, "B", r) in totals]
        if not have:
            continue
        row = {}
        for ph in PHASES:
            per = {arm: [totals[w, arm, r]["cpu_ns"][ph] / 1e9 / (totals[w, arm, r]["decoded"] / 1e9) for r in have] for arm in ("A", "B")}
            row[ph] = dict(A=statistics.mean(per["A"]), B=statistics.mean(per["B"]),
                           A_minus_B=paired_diff([x - y for x, y in zip(per["A"], per["B"])], lag1=False) if len(have) > 1 else None)
        # per BWT row on the segmented blocks: traversal, assembly, and the two together
        per_row = {}
        for arm in ("A", "B"):
            t = [totals[w, arm, r] for r in have]
            parts = {"traversal": ("traversal",), "assembly": ("assembly",), "traversal+assembly": ("traversal", "assembly")}
            per_row[arm] = {k: statistics.mean(sum(x["cpu_ns"][ph] for ph in phs) / max(1, x["segmented_rows"]) for x in t)
                            for k, phs in parts.items()}
        same_work = all(totals[w, "A", r]["rows"] == totals[w, "B", r]["rows"] and totals[w, "A", r]["blocks"] == totals[w, "B", r]["blocks"]
                        for r in have)
        res["phases"][f"w={w}"] = dict(cpu_s_per_GB=row, cpu_ns_per_segmented_row=per_row, reps=len(have),
                                       same_rows_and_blocks=same_work, decoded=totals[w, "A", have[0]]["decoded"],
                                       rows=totals[w, "A", have[0]]["rows"], blocks=totals[w, "A", have[0]]["blocks"])
    if "w=1" in res["phases"] and "w=4" in res["phases"]:
        # the traversal's cross-worker part (its own D) beside its single-worker part
        tr = {w: res["phases"][f"w={w}"]["cpu_s_per_GB"]["traversal"] for w in WORKERS}
        res["traversal_direction"] = dict(
            single_worker_A_minus_B=tr[1]["A"] - tr[1]["B"],
            four_worker_A_minus_B=tr[4]["A"] - tr[4]["B"],
            cross_worker_part=(tr[4]["A"] - tr[4]["B"]) - (tr[1]["A"] - tr[1]["B"]),
            note="a block's traversal precedes its own assembly: a traversal difference at w = 1 can only be persistent (an earlier block's assembly leaving a different cache state); the part that grows from w = 1 to 4 is cross-worker")
    res["interpretable"] = bool(res["perturbation"]) and all(v["ok"] for v in res["perturbation"].values()) and not res["errors"]
    Path(a.out).write_text(json.dumps(dict(stage="S2", result=res), indent=1) + "\n")
    print(f"phases: interpretable={res['interpretable']}; perturbation "
          + ", ".join(f"{k} cpu {v['cpu_median']:.3f} wall {v['wall_median']:.3f}" for k, v in res["perturbation"].items()))


# ---------------------------------------------------------------- S3: counters

IMC = ["uncore_imc/data_reads/", "uncore_imc/data_writes/"]
CORE = ["cycles", "ref-cycles", "instructions"]


def parse_stat(path):
    """perf stat -x, output: {event: (value or None, percent running)}; the IMC's
    per-controller lines summed (perf prints one per PMU instance unless aggregated)"""
    out = {}
    for line in Path(path).read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        f = line.split(",")
        if len(f) < 5:
            continue
        val, ev, pct = f[0], f[2], f[4]
        try:
            v = float(val)
        except ValueError:
            v = None
        try:
            p = float(pct.rstrip("%")) if pct else None
        except ValueError:
            p = None
        if ev in out and out[ev][0] is not None and v is not None:
            out[ev] = (out[ev][0] + v, min(p or 100, out[ev][1] or 100))
        else:
            out[ev] = (v, p)
    return out


def counted(vals, events):
    return all(e in vals and vals[e][0] is not None and (vals[e][1] is None or vals[e][1] >= 99.99) for e in events)


def counters(a):
    runs = [json.loads(l) for l in Path(a.runs).read_text().splitlines() if l.strip()]
    res = dict(imc={}, core={}, unavailable=json.loads(Path(a.probe).read_text()).get("unavailable", {}))
    for w in WORKERS:
        for kind, evs in (("imc", IMC), ("core", CORE)):
            per = {}
            for r in runs:
                if r["w"] != w or r["pass"] != kind:
                    continue
                vals = parse_stat(Path(a.dir) / r["csv"])
                ok = r["rc"] == 0 and counted(vals, evs)
                if kind == "imc":
                    idle = parse_stat(Path(a.dir) / r["idle_csv"])
                    ok &= counted(idle, evs)
                if not ok:
                    continue
                v = {e: vals[e][0] for e in evs}
                if kind == "imc":
                    # 64-byte lines; the idle reading scaled to the run's length, subtracted
                    scale = r["wall_s"] / r["idle_wall_s"] if r["idle_wall_s"] > 0 else 1
                    v = {e: (vals[e][0] - idle[e][0] * scale) * 64 / r["out"] for e in evs}  # bytes per decoded byte
                per.setdefault(r["arm"], {})[r["rep"]] = v
            reps = sorted(set(per.get("A", {})) & set(per.get("B", {})))
            if not reps:
                continue
            row = dict(reps=len(reps))
            for e in evs:
                A = [per["A"][rp][e] for rp in reps]
                B = [per["B"][rp][e] for rp in reps]
                if kind == "imc":
                    row[e] = dict(A=statistics.mean(A), B=statistics.mean(B),
                                  B_minus_A=paired_diff([y - x for x, y in zip(A, B)], lag1=False) if len(reps) > 1 else None)
                else:
                    row[e] = dict(A=statistics.mean(A), B=statistics.mean(B), B_over_A_median=statistics.median(y / x for x, y in zip(A, B)))
            if kind == "core":
                row["effective_ratio_cycles_per_ref"] = {arm: statistics.mean(per[arm][rp]["cycles"] / per[arm][rp]["ref-cycles"] for rp in reps) for arm in ("A", "B")}
                lim = INSTR_W1 if w == 1 else INSTR_W4
                row["instructions_equivalent"] = abs(row["instructions"]["B_over_A_median"] - 1) <= lim
            res[kind][f"w={w}"] = row
    res["instructions_w1_ok"] = res["core"].get("w=1", {}).get("instructions_equivalent")
    Path(a.out).write_text(json.dumps(dict(stage="S3", result=res), indent=1) + "\n")
    print(f"counters: imc {sorted(res['imc'])}, core {sorted(res['core'])}; instructions at w = 1 equivalent: {res['instructions_w1_ok']}")


# ---------------------------------------------------------------- the verdict

def verdict(a):
    s1 = json.loads(Path(a.trial).read_text())
    s2 = json.loads(Path(a.phases).read_text())["result"] if a.phases and Path(a.phases).exists() else None
    s3 = json.loads(Path(a.counters).read_text())["result"] if a.counters and Path(a.counters).exists() else None
    instr = s3.get("instructions_w1_ok") if s3 else None
    d = decisions(s1["result"], instr)
    res = s1["result"]
    out = dict(protocol="2026-11-01-p87-four-core-protocol", decisions=d,
               instructions_check="not measured" if instr is None else "pass" if instr else "FAIL",
               historical_P87_criterion="not evaluated: registered at 6-10 workers (occupancy curve >= 1.03x, with DRAM write bytes per output byte); this diagnostic runs four",
               mechanism=res.get("mechanism", {}).get("D"), mechanism_AA=res.get("mechanism_AA", {}).get("D"),
               delta_w1=res.get("mechanism", {}).get("delta_w1"), delta_w4=res.get("mechanism", {}).get("delta_w4"),
               engineering_primary_w4=res.get("engineering", {}).get("primary_w4"),
               phases=dict(interpretable=s2["interpretable"], traversal_direction=s2.get("traversal_direction")) if s2 else "not run",
               dram=s3["imc"] if s3 else "not run")
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="mode", required=True)
    m = sub.add_parser("manifest")
    for n in ["frozen", "dir", "out"]:
        m.add_argument("--" + n, required=True)
    z = sub.add_parser("freeze")
    for n in ["corpora-from", "pin", "source-base", "out"]:
        z.add_argument("--" + n, required=True)
    s = sub.add_parser("summary")
    for n in ["manifest", "raw", "allocation", "out"]:
        s.add_argument("--" + n, required=True)
    s.add_argument("--stat-before")
    s.add_argument("--stat-after")
    ph = sub.add_parser("phases")
    for n in ["pairs", "runs", "out"]:
        ph.add_argument("--" + n, required=True)
    c = sub.add_parser("counters")
    for n in ["runs", "dir", "probe", "out"]:
        c.add_argument("--" + n, required=True)
    v = sub.add_parser("verdict")
    v.add_argument("--trial", required=True)
    v.add_argument("--phases")
    v.add_argument("--counters")
    v.add_argument("--out", required=True)
    a = p.parse_args()
    dict(manifest=manifest, freeze=freeze, summary=summary, phases=phases, counters=counters, verdict=verdict)[a.mode](a)
