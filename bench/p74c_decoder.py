"""The P74-C full-decoder trial's helpers (docs/journal/2026-10-31-p74c-decoder-protocol.md).

No timing here. Subcommands:
- fetch: HTTP range windows of a dump, behind the range guard (protocol §5): status 206,
  Content-Range exactly `bytes a-b/DUMP_BYTES`, exactly b - a + 1 bytes; a 200 or any
  full-object or mismatched response aborts at once, without retry; transport errors are
  retried at most 3 times. Each window is checked against the spec's SHA-256, or, with
  --register, its hash is recorded (the different-dump freeze).
- allocation: the CPU allocation from sysfs (protocol §4), as JSON.
- manifest: the trial's manifest (TSV) from frozen.json and the corpus paths.
- freeze: frozen.json from the corpora, their bzip2 -dc output, the selector audit and the
  harness sources (run once, at the freeze).
- summary: the registered analysis of a trial's raw rows (protocol §7-8).
"""
import argparse
import hashlib
import json
import math
import os
import statistics
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

UA = "mibz2-p74c-decoder-trial/1.0 (https://github.com/lmmx/mibz2; research benchmark, range requests)"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class RangeRefused(Exception):
    """A response the guard refuses: never retried."""


def fetch_range(url, start, length, total):
    end = start + length - 1
    req = urllib.request.Request(url, headers={"Range": f"bytes={start}-{end}", "User-Agent": UA})
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                if r.status != 206:
                    raise RangeRefused(f"status {r.status}, not 206 Partial Content: refusing a full-object response")
                cr = r.headers.get("Content-Range")
                want = f"bytes {start}-{end}/{total}"
                if cr != want:
                    raise RangeRefused(f"Content-Range {cr!r} != {want!r}")
                body = r.read(length + 1)
                if len(body) != length:
                    raise RangeRefused(f"{len(body)} bytes, not {length}")
                return body
        except RangeRefused:
            raise
        except urllib.error.HTTPError as e:
            raise RangeRefused(f"HTTP {e.code}") from e
        except (urllib.error.URLError, ConnectionError, TimeoutError, OSError) as e:
            if attempt == 3:
                raise
            print(f"fetch: transport error ({e}); retry {attempt + 1} of 3", file=sys.stderr)
            time.sleep(5 * (attempt + 1))
    raise AssertionError("unreachable")


def fetch(a):
    spec = json.loads(Path(a.spec).read_text())
    d = Path(a.dir)
    d.mkdir(parents=True, exist_ok=True)
    registered = []
    for w in spec["windows"]:
        p = d / f"{w['name']}.bin"
        want = w.get("sha256")
        if p.exists() and want and sha(p) == want:
            registered.append(dict(w, sha256=want))
            continue
        body = fetch_range(spec["url"], w["offset"], w["length"], spec["dump_bytes"])
        h = hashlib.sha256(body).hexdigest()
        if want:
            assert h == want, f"{w['name']}: {h} != spec {want}"
        elif not a.register:
            raise SystemExit(f"{w['name']}: the spec has no hash and --register was not given")
        p.write_bytes(body)
        registered.append(dict(w, sha256=h))
        print(f"fetch: {w['name']} at {w['offset']}: {h}")
    if a.register:
        Path(a.register).write_text(json.dumps(dict(spec, windows=registered), indent=1) + "\n")


def allocation(a):
    """Up to 4 physical cores, those other than CPU 0's first (CPU order), CPU 0's core only
    if fewer than 4 others; the lowest CPU of each core; siblings idle."""
    allowed = sorted(os.sched_getaffinity(0))
    sys_cpu = Path("/sys/devices/system/cpu")

    def siblings(c):
        try:
            text = (sys_cpu / f"cpu{c}/topology/thread_siblings_list").read_text().strip()
        except OSError:
            return (c,)
        out = []
        for part in text.split(","):
            x, _, y = part.partition("-")
            out.extend(range(int(x), int(y or x) + 1))
        return tuple(sorted(out))

    cores = {}
    for c in allowed:
        cores.setdefault(siblings(c), []).append(c)
    zero = siblings(0)
    order = sorted(cores, key=lambda s: (s == zero, min(s)))
    chosen = order[: a.cores]
    cpus = sorted(min(cores[s]) for s in chosen)
    idle = sorted(x for s in chosen for x in s if x not in cpus)
    out = dict(alloc=cpus, idle_siblings=idle, physical_cores_allowed=len(cores),
               cores=[list(s) for s in chosen], includes_cpu0_core=zero in chosen,
               smt=any(len(s) > 1 for s in cores))
    print(json.dumps(out))


def manifest(a):
    f = json.loads(Path(a.frozen).read_text())
    paths = Path(a.files).read_text().split()
    rows = []
    for p in paths:
        name = Path(p).name[:-4]
        c = f["corpora"][name]
        assert sha(p) == c["compressed_sha256"], f"{name}: compressed hash"
        rows.append("\t".join([name, c["role"], p, c["compressed_sha256"], c["decoded_sha256"], str(c["decoded_bytes"])]))
    assert [r.split("\t")[0] for r in rows] == f["order"], "corpus order differs from the frozen order"
    Path(a.out).write_text("\n".join(rows) + "\n")
    print(f"manifest: {len(rows)} corpora match the frozen SHA-256")


SOURCES = [
    "Cargo.toml", "Cargo.lock", "src/lib.rs", "src/mtf.rs", "src/mtf/p74.rs", "src/mtf/p74_c.rs",
    "src/mtf/p74_c/tests.rs", "src/mtf/p74_c/tests/gate1.rs", "src/parallel.rs", "src/bwt.rs",
    "examples/p74c_decoder.rs", "examples/p74_suite.rs", "tests/common/mod.rs", "tests/p74c_decoder.rs",
    "tests/p74c_selection.rs", "tests/mtf_p74_c.rs", "bench/p74c_decoder.py", "bench/p74c_decoder_asm_audit.py",
    "bench/p74c_decoder_run.sh", ".just/bench.just", "bench/wikidata.json",
    "docs/journal/2026-10-31-p74c-decoder-protocol.md",
    "docs/journal/2026-10-31-p74c-decoder/heldout-spec.json",
    "docs/journal/2026-10-31-p74c-decoder/different-dump/spec.json",
    "docs/journal/2026-10-31-p74c-decoder/different-dump/windows.json",
]


def freeze(a):
    import subprocess
    stage = json.loads(Path(a.stage_frozen).read_text())
    stage_suite = {x["name"]: x for x in stage["suite"]}
    audit = {}
    for l in Path(a.audit).read_text().splitlines():
        if l.startswith("AUDITSUM\t"):
            n, b, by, te, h = l.split("\t")[1:]
            audit[n] = dict(blocks=int(b), bypassed=int(by), p74_to_end=int(te), handoffs=int(h))
    roles = dict([("wikidata-fitted", "secondary"), ("wikidata-heldout", "secondary"), ("wikidata-20261026", "primary"),
                  ("random", "control"), ("records", "control"), ("mixed-independent", "control"), ("text", "control")])
    corpora, order = {}, []
    for p in Path(a.files).read_text().split():
        name = Path(p).name[:-4]
        z = Path(p).read_bytes()
        d = subprocess.run(["bzip2", "-dc", p], check=True, capture_output=True).stdout
        c = dict(role=roles.get(name, "adversarial"), compressed_sha256=hashlib.sha256(z).hexdigest(),
                 compressed_bytes=len(z), decoded_sha256=hashlib.sha256(d).hexdigest(), decoded_bytes=len(d),
                 selector_audit=audit[name])
        if name in stage_suite:
            assert c["compressed_sha256"] == stage_suite[name]["compressed_sha256"], f"{name}: suite differs from the stage trial's"
            assert c["decoded_sha256"] == stage_suite[name]["decoded_sha256"], name
        if name in stage["corpora"] and name != "text":
            assert c["compressed_sha256"] == stage["corpora"][name], f"{name}: differs from the stage trial's"
        corpora[name] = c
        order.append(name)
    out = dict(protocol="docs/journal/2026-10-31-p74c-decoder-protocol.md", order=order, corpora=corpora,
               different_dump=json.loads(Path("docs/journal/2026-10-31-p74c-decoder/different-dump/windows.json").read_text()),
               schedule=dict(rounds=ROUNDS, arms=["A shipped", "B P74-C", "A2 shipped, second label"],
                             design="per cell: the 6 permutations x3 + (A,B,A2), (B,A,A2), Fisher-Yates; cell order shuffled per round; xorshift seed 0x74c0202610310201",
                             warmups="one untimed child per arm per cell"),
               toolchain="rustc 1.90.0, --locked, release profile, empty RUSTFLAGS",
               frozen_kernel="research/p74-c-implementation 8b6caf1e48eb4dc605abe3eb97de680891340ff9 (with 2e11dc1)",
               sources_sha256={p: sha(p) for p in SOURCES})
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(f"freeze: {len(order)} corpora, {len(SOURCES)} sources")


# ---------------------------------------------------------------- statistics (stage protocol)

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


def paired(x):
    """The stage protocol's: mean of per-round log ratios; lag-1 autocorrelation rho in round
    order; for rho > 0, n_eff = R(1-rho)/(1+rho), SE = sd/sqrt(n_eff), df = max(1,
    floor(n_eff)-1); two-sided 95% t interval (each end a one-sided 0.025 bound)."""
    r = len(x)
    m = statistics.mean(x)
    sd = statistics.stdev(x)
    den = sum((v - m) ** 2 for v in x)
    rho = sum((x[i] - m) * (x[i + 1] - m) for i in range(r - 1)) / den if den > 0 else 0.0
    n_eff = r * (1 - rho) / (1 + rho) if rho > 0 else r
    df = max(1, math.floor(n_eff) - 1)
    se = sd / math.sqrt(n_eff)
    half = t_quantile(0.975, df) * se
    return dict(mean_log=m, se=se, df=df, rho1=rho, n_eff=n_eff, ratio=math.exp(m),
                lower95=math.exp(m - half), upper95=math.exp(m + half))


def binom_tail(m, k, q=0.05):
    return sum(math.comb(m, i) * q ** i * (1 - q) ** (m - i) for i in range(k, m + 1))


ROUNDS = 20
ARMS = ["A", "B", "A2"]
PRIMARY = "wikidata-20261026"
SECONDARY = ["wikidata-fitted", "wikidata-heldout"]
CONTROLS = ["random", "records", "mixed-independent", "text"]
GATED = [PRIMARY, *SECONDARY, *CONTROLS]
R_PLAIN = {"i9-10900K": 0.819, "Zen 3 (EPYC 7763)": 0.719, "Zen 4 (EPYC 9V74), low": 0.722, "Zen 4 (EPYC 9V74), high": 0.724}


def outcome(neg, ok):
    return "NEGATIVE" if neg else "PASS" if ok else "INCONCLUSIVE"


def read_raw(path):
    """ROW fields (examples/p74c_decoder.rs): round, cell position, corpus index, name, w,
    arm, position, then the child's: wall ns, decode user us, decode sys us, bytes, sha ok,
    blocks, selections, default plan, allowed CPUs, physical cores, threads (spawned), their
    snapshot, threads (end), their snapshot; then the parent's: life user us, life sys us,
    maxrss KiB, minflt, majflt, nvcsw, nivcsw, warm-up flag, child life ns."""
    rows = []
    for line in Path(path).read_text().splitlines():
        f = line.split("\t")
        if f[0] != "ROW":
            continue
        rows.append(dict(round=int(f[1]), cell_pos=int(f[2]), corpus=int(f[3]), name=f[4], w=int(f[5]),
                         arm=int(f[6]), position=int(f[7]), wall_ns=int(f[8]), user_us=int(f[9]),
                         sys_us=int(f[10]), bytes=int(f[11]), sha_ok=f[12] == "1", blocks=int(f[13]),
                         selections=[int(x) for x in f[14].split(",")], plan=f[15], allowed=f[16],
                         phys=int(f[17]), threads_spawned=int(f[18]), threads_spawned_list=f[19],
                         threads_end=int(f[20]), threads_end_list=f[21], life_user_us=int(f[22]),
                         life_sys_us=int(f[23]), maxrss_kb=int(f[24]), minflt=int(f[25]), majflt=int(f[26]),
                         nvcsw=int(f[27]), nivcsw=int(f[28]), warm=f[29] == "1", life_ns=int(f[30])))
    return rows


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
    for c in sorted(set(alloc) | set(siblings)):
        if c not in b or c not in e:
            continue
        d = [y - x for x, y in zip(b[c], e[c])]
        total = sum(d[:8]) or 1
        idle = d[3] + d[4]
        res[str(c)] = dict(role="allocated" if c in alloc else "idle sibling", ticks=total,
                           busy=(total - idle) / total, irq=(d[5] + d[6]) / total, steal=d[7] / total)
    return res


def summary(a):
    man = [l.split("\t") for l in Path(a.manifest).read_text().splitlines() if l]
    roles = {m[0]: m[1] for m in man}
    rows = read_raw(a.raw)
    alloc = set(json.loads(Path(a.allocation).read_text())["alloc"])
    timed = [r for r in rows if not r["warm"]]
    workers = sorted({r["w"] for r in timed})
    obs = {}
    for r in timed:
        key = (r["name"], r["w"], r["round"], r["arm"])
        assert key not in obs, f"duplicate observation {key}"
        obs[key] = r
    names = [m[0] for m in man]
    want = {(n, w, rd, arm) for n in names for w in workers for rd in range(ROUNDS) for arm in range(3)}
    assert set(obs) == want, "incomplete or extra observations: no summary"
    # correctness and confinement, re-checked from the raw rows
    correctness = dict(all_sha_ok=all(r["sha_ok"] for r in rows), all_workers_plan=all(r["plan"] == f"Workers({r['w']})" for r in rows))
    def threads_of(lst):
        return [t.split("|") for t in lst.split(";") if t]
    confined, names_seen, psr = True, {}, {}
    for r in rows:
        for t in threads_of(r["threads_spawned_list"]) + threads_of(r["threads_end_list"]):
            cpus = set()
            for part in t[1].split(","):
                x, _, y = part.partition("-")
                cpus.update(range(int(x), int(y or x) + 1))
            confined &= bool(cpus) and cpus <= alloc
            kind = "worker" if t[0].startswith("mibz2-") and t[0] != "mibz2-scan" else t[0]
            names_seen[kind] = names_seen.get(kind, 0) + 1
            if t[2]:
                psr.setdefault(kind, {}).setdefault(t[2], 0)
                psr[kind][t[2]] += 1
    correctness["all_threads_within_allocation"] = confined
    selections_ok = all((r["selections"][2] == r["blocks"]) if r["arm"] == 1 else (r["selections"][0] == r["blocks"]) for r in rows)
    correctness["selections_match_arm"] = selections_ok
    out_rows = []
    for n in names:
        for w in workers:
            t = {arm: [obs[n, w, rd, i] for rd in range(ROUNDS)] for i, arm in enumerate(ARMS)}
            x = {arm: [r["bytes"] / r["wall_ns"] * 1e3 for r in v] for arm, v in t.items()}  # MB/s
            cpu = {arm: [(r["user_us"] + r["sys_us"]) / 1e6 / (r["bytes"] / 1e9) for r in v] for arm, v in t.items()}
            rss = {arm: [r["maxrss_kb"] for r in v] for arm, v in t.items()}
            row = dict(corpus=n, role=roles[n], w=w,
                       MB_s={k: statistics.mean(v) for k, v in x.items()},
                       cpu_s_per_GB={k: statistics.mean(v) for k, v in cpu.items()},
                       maxrss_kb_max={k: max(v) for k, v in rss.items()},
                       life_cpu_s_mean={k: statistics.mean((r["life_user_us"] + r["life_sys_us"]) / 1e6 for r in v) for k, v in t.items()},
                       blocks=t["A"][0]["blocks"],
                       p74c_bypass_blocks=t["B"][0]["selections"][4], p74c_combined_blocks=t["B"][0]["selections"][5],
                       threads_spawned=sorted({r["threads_spawned"] for v in t.values() for r in v}))
            for arm in ("B", "A2"):
                row[f"X {arm}/A"] = paired([math.log(p / q) for p, q in zip(x[arm], x["A"])])
                row[f"CPU {arm}/A"] = paired([math.log(p / q) for p, q in zip(cpu[arm], cpu["A"])])
                lr = [math.log(p / q) for p, q in zip(rss[arm], rss["A"])]
                row[f"RSS {arm}/A"] = dict(paired(lr), median=math.exp(statistics.median(lr)), min=math.exp(min(lr)), max=math.exp(max(lr)),
                                          max_rule=max(rss[arm]) / max(rss["A"]))
            out_rows.append(row)
    def get(n, w):
        return next(r for r in out_rows if r["corpus"] == n and r["w"] == w)
    present = set(names)
    gated = [n for n in GATED if n in present]
    missing = [n for n in GATED if n not in present]
    gates = {}
    w4 = 4 in workers and PRIMARY in present
    # 1. primary
    if w4:
        p = get(PRIMARY, 4)["X B/A"]
        gates["1_primary"] = dict(row=p, outcome=outcome(p["upper95"] < 1.02, p["lower95"] >= 1.02),
                                  demonstrated_harm=p["upper95"] < 1.00, lower_above_1=p["lower95"] > 1.00)
    # 2. secondary flags
    flags = {}
    for n in [n for n in SECONDARY if n in present]:
        rows_n = {w: get(n, w)["X B/A"] for w in workers}
        f = None
        if w4:
            r4 = rows_n[4]
            f = r4["ratio"] < 1.00 or r4["lower95"] < 0.97
        flags[n] = dict(by_w={w: dict(ratio=r["ratio"], lower95=r["lower95"], upper95=r["upper95"]) for w, r in rows_n.items()},
                        material_deterioration=f)
    gates["2_secondary_flags"] = flags
    # 3. controls, every w
    ctl = {}
    for n in [n for n in CONTROLS if n in present]:
        for w in workers:
            r = get(n, w)["X B/A"]
            ctl[f"{n} w={w}"] = outcome(r["upper95"] < 0.97, r["lower95"] >= 0.97)
    gates["3_controls"] = dict(rows=ctl, outcome=outcome("NEGATIVE" in ctl.values(), all(v == "PASS" for v in ctl.values())))
    # 4. CPU-s/GB at w = 4
    if w4:
        cg = {}
        sens = {}
        for n in gated:
            r = get(n, 4)["CPU B/A"]
            cg[n] = outcome(r["lower95"] > 1.01, r["ratio"] <= 1.00 and r["upper95"] <= 1.01)
            sens[n] = dict(ratio=r["ratio"], upper95=r["upper95"], upper_le_1_00=r["upper95"] <= 1.00, upper_le_1_02=r["upper95"] <= 1.02)
        gates["4_cpu"] = dict(rows=cg, outcome=outcome("NEGATIVE" in cg.values(), all(v == "PASS" for v in cg.values())),
                              sensitivity_descriptive=sens)
    # 5. RSS, every corpus 1-7 and w
    rg = {}
    for n in gated:
        for w in workers:
            r = get(n, w)["RSS B/A"]
            rg[f"{n} w={w}"] = outcome(r["lower95"] > 1.01, r["max_rule"] <= 1.01)
    gates["5_rss"] = dict(rows=rg, outcome=outcome("NEGATIVE" in rg.values(), all(v == "PASS" for v in rg.values())))
    # 6. A/A calibration
    aa = [get(n, w)[k] for n in gated for w in workers for k in ("X A2/A", "CPU A2/A")]
    excl = sum(1 for r in aa if not (r["lower95"] <= 1 <= r["upper95"]))
    k_crit = next(k for k in range(len(aa) + 2) if binom_tail(len(aa), k) <= 0.05) if aa else 0
    gates["6_aa"] = dict(rows=len(aa), excluding_one=excl, threshold=k_crit, outcome="PASS" if excl < k_crit else "INCONCLUSIVE")
    correct = all(correctness.values())
    if not correct:
        overall = "NEGATIVE (correctness)" if not correctness["all_sha_ok"] else "ABORTED (correctness precondition)"
    elif missing:
        overall = f"NOT APPLICABLE (corpora missing: {missing})"
    elif not w4:
        overall = "NOT APPLICABLE (no w = 4: exploratory host)"
    else:
        outs = [gates[g]["outcome"] for g in ("1_primary", "3_controls", "4_cpu", "5_rss", "6_aa")]
        overall = outcome("NEGATIVE" in outs, all(o == "PASS" for o in outs))
    # the model, descriptive: implied s_MTF = (1 - 1/R)/(1 - r_plain)
    model = {}
    for n in [n for n in [PRIMARY, *SECONDARY] if n in present]:
        for w in workers:
            R = get(n, w)["X B/A"]["ratio"]
            model[f"{n} w={w}"] = {h: (1 - 1 / R) / (1 - r) for h, r in R_PLAIN.items()}
    verdict = dict(overall=overall, correctness=correctness, gates=gates)
    host = {}
    if a.stat_before and a.stat_after:
        host = cpu_busy(a.stat_before, a.stat_after, sorted(alloc), json.loads(Path(a.allocation).read_text()).get("idle_siblings", []))
        verdict["host"] = host
    Path(a.out).write_text(json.dumps(dict(protocol="2026-10-31-p74c-decoder-protocol", rounds=ROUNDS, workers=workers,
        allocation=sorted(alloc), verdict=verdict, rows=out_rows, host=host,
        threads=dict(names=names_seen, psr_by_kind=psr),
        model=dict(formula="dT/T ~ s_MTF (1 - r_plain); implied s_MTF = (1 - 1/R)/(1 - r_plain), R the B/A throughput ratio",
                   r_plain=R_PLAIN, implied_s_mtf=model)), indent=1) + "\n")
    print(json.dumps(verdict, indent=1))


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="mode", required=True)
    f = sub.add_parser("fetch")
    f.add_argument("--spec", required=True)
    f.add_argument("--dir", required=True)
    f.add_argument("--register", help="write the spec with the fetched windows' hashes here")
    c = sub.add_parser("allocation")
    c.add_argument("--cores", type=int, default=4)
    m = sub.add_parser("manifest")
    for n in ["frozen", "files", "out"]:
        m.add_argument("--" + n, required=True)
    z = sub.add_parser("freeze")
    for n in ["files", "audit", "stage-frozen", "out"]:
        z.add_argument("--" + n, required=True)
    s = sub.add_parser("summary")
    for n in ["manifest", "raw", "allocation", "out"]:
        s.add_argument("--" + n, required=True)
    s.add_argument("--stat-before")
    s.add_argument("--stat-after")
    a = p.parse_args()
    dict(fetch=fetch, allocation=allocation, manifest=manifest, freeze=freeze, summary=summary)[a.mode](a)
