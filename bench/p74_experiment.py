"""Freeze inputs/host without timing, or summarize an approved run's TSV. No runner dispatch."""
import argparse
import bz2
import hashlib
import json
import math
import statistics
import subprocess
from pathlib import Path


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def interval(values):
    # Registered 12 paired rounds, two-sided 95% Student t, df=11.
    assert len(values) == 12
    mean = statistics.mean(values)
    half = 2.200985 * statistics.stdev(values) / math.sqrt(12)
    return {"mean": mean, "lower95": mean-half, "upper95": mean+half}


def manifest(a):
    files = [Path(s) for s in Path(a.files).read_text().splitlines()]
    corpora = []
    meta = {}
    for line in Path(a.check).read_text().splitlines():
        tag, index, blocks, symbols, last, decoded, checkpoints = line.split("\t")
        assert tag == "META"
        meta[int(index)] = dict(blocks=int(blocks), encoded_symbols=int(symbols), last_column_bytes=int(last), decoded_bytes=int(decoded), checkpoint_records=int(checkpoints))
    audits = [[] for _ in files]
    for line in Path(a.audit).read_text().splitlines():
        if line.startswith("AUDIT\t"):
            fields = list(map(int, line.split("\t")[1:]))
            index, block, cp, symbols, explicit, high, bypass, at, n, previous, digits = fields
            audits[index].append(dict(block=block, checkpoint_interval=cp, symbols=symbols, explicit=explicit,
                high=high, bypass=bool(bypass), switch_consumed=at or None, output_before_switch=n,
                p74_high_before_switch=previous, run_digits=digits))
    roles = Path(a.roles).read_text().split() if getattr(a, "roles", None) else [None] * len(files)
    for i, p in enumerate(files):
        digest = hashlib.sha256()
        size = 0
        with bz2.open(p, "rb") as f:
            while part := f.read(1 << 20):
                digest.update(part)
                size += len(part)
        assert size == meta[i]["decoded_bytes"]
        assert len(audits[i]) == 2 * meta[i]["blocks"]
        name = p.stem
        if name == "wikidata-fitted":
            spec = json.loads(Path("bench/wikidata.json").read_text())
            assert sha(p) == spec["corpus_sha256"] and size == spec["decoded_bytes"]
        if name == "wikidata-heldout":
            spec = json.loads(Path("docs/journal/2026-10-31-p74-c-confirmatory/heldout/spec.json").read_text())
            assert sha(p) == spec["corpus_sha256"] and meta[i]["blocks"] == spec["blocks"]
        corpora.append(dict(name=name, path=str(p.resolve()), compressed_bytes=p.stat().st_size,
            compressed_sha256=sha(p), decoded_sha256=digest.hexdigest(), coverage=meta[i],
            independent=name == "mixed-independent", fitted=name == "wikidata-fitted",
            role=roles[i] or name, audit=audits[i]))
    source = {str(p): sha(p) for p in [Path("examples/p74_experiment.rs"), Path("bench/p74_experiment.py"),
        Path("src/mtf/p74_c.rs"), Path("src/mtf/p74_c/tests.rs"), Path("tests/common/mod.rs")]}
    quota = Path("/sys/fs/cgroup/cpu.max")
    result = dict(schema=1, implementation_reference="1bf554db61a82ff86f832b8697b90675e6a6ba46",
        preparation_commit=command("git", "rev-parse", "HEAD"), working_tree=command("git", "status", "--porcelain"),
        sources_sha256=source, binary_sha256=sha(a.binary), compiler=command("rustc", "-Vv"),
        flags="Cargo release: fat LTO, codegen-units=1; no RUSTFLAGS; baseline target + function AVX2",
        host=dict(cpu=command("lscpu", "-J"), topology=command("lscpu", "-e=CPU,CORE,SOCKET,NODE"),
            affinity=command("taskset", "-pc", str(__import__('os').getpid())),
            quota=quota.read_text().strip() if quota.exists() else "unavailable",
            suitable_for_timing=False, approval="NOT GRANTED; preparation host only"),
        protocol=dict(rounds=12, passes_per_row=2, warmup_passes_per_arm=2,
            ordering="six permutations twice, Fisher-Yates xorshift seed 0x74c0202610310002",
            variant_order=["plain", "checkpoint"], corpus_order=[c["name"] for c in corpora],
            statistic="arithmetic mean of within-round ratios; two-sided 95% Student t df=11, critical=2.200985; no exclusions, fastest-pass selection or optional stopping"),
        assembly_gate="FAILED originally; amended for this experiment only to accept documented +1/+1 instruction discrepancy with six low-path conditional branches and no additional YMM spills; no claim of old gate passing",
        policy="S<16384 or 2*S>=max_len => standalone; else switch iff F+1>floor(consumed/32)+123, trigger included; permanent",
        corpora=corpora, independent_wikidata="not available; robustness qualification remains incomplete")
    Path(a.out).write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps({"manifest": a.out, "sha256": sha(a.out), "corpora": len(corpora)}))


def summarize(a):
    m = json.loads(Path(a.manifest).read_text())
    groups = {}
    positions = {}
    for line in Path(a.raw).read_text().splitlines():
        if not line.startswith("ROW\t"):
            continue
        f, cp, r, position, arm, ns, n = map(int, line.split("\t")[1:])
        key = f, cp
        d = groups.setdefault(key, {})
        assert (r, arm) not in d and ns > 0
        assert n == 2*m["corpora"][f]["coverage"]["last_column_bytes"]
        d[r, arm] = (ns/2, n/2)
        assert (f, cp, r, position) not in positions
        positions[f, cp, r, position] = arm
    assert set(groups) == {(i,cp) for i in range(len(m["corpora"])) for cp in [0,1]}
    result = []
    for (f,cp), d in sorted(groups.items()):
        assert set(d) == {(r,arm) for r in range(12) for arm in range(3)}
        for r in range(12):
            assert sorted(positions[f,cp,r,p] for p in range(3)) == [0,1,2]
        rows = {}
        for arm in range(3):
            rows[f"T{arm}"] = dict(latency_ns=interval([d[r,arm][0] for r in range(12)]),
                throughput_MB_s=interval([d[r,arm][1]/d[r,arm][0]*1000 for r in range(12)]))
        for arm, base in [(2,0),(1,0),(2,1)]:
            rows[f"T{arm}/T{base}"] = interval([d[r,arm][0]/d[r,base][0] for r in range(12)])
            rows[f"T{arm}-T{base}_ns"] = interval([d[r,arm][0]-d[r,base][0] for r in range(12)])
        main = rows["T2/T0"]
        tax = rows["T1/T0"]
        name = m["corpora"][f]["name"]
        verdict = "descriptive"
        if name in ["random","records","mixed-independent"]:
            verdict = "pass" if main["upper95"] <= 1.03 else "fail"
        elif name == "wikidata-fitted" and cp:
            verdict = "pass" if main["upper95"] <= .95 and 1-main["upper95"] > 2*max(abs(1-tax["lower95"]),abs(1-tax["upper95"])) else "fail"
        result.append(dict(corpus=name, variant="checkpoint" if cp else "plain", rows=rows, verdict=verdict))
    Path(a.out).write_text(json.dumps(dict(results=result, whole_decoder="unmeasured; model speedup=1/(1-f+f*r), with independently established stage fraction f and stage ratio r; do not infer f from these runs"), indent=2)+"\n")



# ---------------------------------------------------------------------------------------
# Confirmatory protocol (2026-10-31-p74-c-confirmatory-protocol.md): five arms, 20 rounds,
# paired log-time ratios, AR(1) variance inflation, Holm families, A/A empirical null.
# Frozen before any confirmatory timing. Pure Python (no SciPy).
ARMS = ["T0", "T1", "T2", "Free", "AA"]
ROUNDS = 20
ALPHA = 0.025  # one-sided, the upper end of a two-sided 95% interval
CONTROLS = ["random", "records", "mixed-independent"]
COMPARISONS = [("T2", "T0"), ("T1", "T0"), ("T2", "T1"), ("Free", "T0"), ("T2", "Free"), ("AA", "T0")]


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
    """Mean of per-round log ratios; lag-1 autocorrelation rho in round order; for
    rho > 0, n_eff = R(1-rho)/(1+rho), SE = sd/sqrt(n_eff), df = max(1, floor(n_eff)-1)."""
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


def p_upper_exceeds(row, limit):
    """One-sided p for H0: ratio >= limit (small p supports ratio < limit)."""
    if row["se"] == 0:
        return 0.0 if row["mean_log"] < math.log(limit) else 1.0
    return t_cdf((row["mean_log"] - math.log(limit)) / row["se"], row["df"])


def holm(ps):
    order = sorted(range(len(ps)), key=lambda i: ps[i])
    adj, running = [0.0] * len(ps), 0.0
    for j, i in enumerate(order):
        running = max(running, min(1.0, (len(ps) - j) * ps[i]))
        adj[i] = running
    return adj


def binom_tail(m, k, q=0.05):
    return sum(math.comb(m, i) * q ** i * (1 - q) ** (m - i) for i in range(k, m + 1))


def confirm(a):
    m = json.loads(Path(a.manifest).read_text())
    files = m["corpora"]
    obs, blocks, positions = {}, {}, set()
    for line in Path(a.raw).read_text().splitlines():
        f = line.split("\t")
        if f[0] == "ROW":
            fi, cp, r, pos, arm, ns, n = map(int, f[1:])
            key = (fi, cp, r, arm)
            assert key not in obs and ns > 0 and (fi, cp, r, pos) not in positions
            assert n == 2 * files[fi]["coverage"]["last_column_bytes"]
            obs[key] = ns / 2
            positions.add((fi, cp, r, pos))
        elif f[0] == "BLOCK":
            fi, cp, r, arm, b, ns = map(int, f[1:])
            blocks[fi, cp, r, arm, b] = ns / 2
    want = {(fi, cp, r, arm) for fi in range(len(files)) for cp in (0, 1) for r in range(ROUNDS) for arm in range(5)}
    assert set(obs) == want, "incomplete or extra observations: no summary"
    rows = []
    for fi, c in enumerate(files):
        for cp in (0, 1):
            t = {arm: [obs[fi, cp, r, i] for r in range(ROUNDS)] for i, arm in enumerate(ARMS)}
            row = dict(corpus=c["name"], role=c["role"], variant="checkpoint" if cp else "plain",
                       bypass_blocks=sum(x["bypass"] for x in c["audit"] if x["checkpoint_interval"] == (16384 if cp else 0)),
                       switch_blocks=sum(bool(x["switch_consumed"]) for x in c["audit"] if x["checkpoint_interval"] == (16384 if cp else 0)),
                       latency_ns={k: statistics.mean(v) for k, v in t.items()},
                       MB_s={k: c["coverage"]["last_column_bytes"] / statistics.mean(v) * 1000 for k, v in t.items()})
            for x, y in COMPARISONS:
                row[f"{x}/{y}"] = paired([math.log(p / q) for p, q in zip(t[x], t[y])])
            rows.append(row)
    # A/A calibration
    aa_excl = sum(1 for r in rows if not (r["AA/T0"]["lower95"] <= 1 <= r["AA/T0"]["upper95"]))
    k_crit = next(k for k in range(len(rows) + 2) if binom_tail(len(rows), k) <= 0.05)
    calibrated = aa_excl < k_crit
    # Family 1: negative controls, joint <= +3%
    fam1 = [r for r in rows if r["corpus"] in CONTROLS]
    adj = holm([p_upper_exceeds(r["T2/T0"], 1.03) for r in fam1])
    for r, q in zip(fam1, adj):
        r["control"] = dict(limit=1.03, holm_p=q, passes=q <= ALPHA)
    # Family 2: adversarial rows; limits fixed by the protocol
    fam2 = [r for r in rows if r["role"] == "adversarial"]
    lims = [1.01 if r["bypass_blocks"] else 1.03 for r in fam2]
    adj = holm([p_upper_exceeds(r["T2/T0"], l) for r, l in zip(fam2, lims)])
    for r, q, l in zip(fam2, adj, lims):
        r["adversarial"] = dict(limit=l, holm_p=q, passes=q <= ALPHA)
    # Primary and held-out (same criterion, never pooled)
    primary = {}
    for name in ["wikidata-fitted", "wikidata-heldout"]:
        r = next((r for r in rows if r["corpus"] == name and r["variant"] == "checkpoint"), None)
        if r is None:
            continue
        noise = max(abs(1 - r[k][e]) for k in ["T1/T0", "AA/T0"] for e in ["lower95", "upper95"])
        up = r["T2/T0"]["upper95"]
        primary[name] = dict(T2_T0_upper95=up, noise=noise, passes=up <= 0.95 and 1 - up > 2 * noise)
    # Per-block decomposition (adversarial files): paired per-round ns differences
    decomposition = []
    for fi, c in enumerate(files):
        if c["role"] != "adversarial":
            continue
        for cp in (0, 1):
            audit = [x for x in c["audit"] if x["checkpoint_interval"] == (16384 if cp else 0)]
            for b, x in enumerate(audit):
                d = {}
                for p, q in [("T2", "T1"), ("T2", "Free"), ("T1", "T0"), ("Free", "T0"), ("AA", "T0")]:
                    v = [blocks[fi, cp, r, ARMS.index(p), b] - blocks[fi, cp, r, ARMS.index(q), b] for r in range(ROUNDS)]
                    mean, sd = statistics.mean(v), statistics.stdev(v)
                    half = t_quantile(0.975, ROUNDS - 1) * sd / math.sqrt(ROUNDS)
                    d[f"{p}-{q}_ns"] = dict(mean=mean, lower95=mean - half, upper95=mean + half)
                decomposition.append(dict(corpus=c["name"], variant="checkpoint" if cp else "plain", block=b,
                    bypass=x["bypass"], switch_consumed=x["switch_consumed"], **d))
    verdict = dict(
        calibrated=calibrated, aa_rows_excluding_one=aa_excl, aa_threshold=k_crit,
        primary=primary,
        controls_joint=all(r["control"]["passes"] for r in fam1) if fam1 else None,
        adversarial_joint=all(r["adversarial"]["passes"] for r in fam2) if fam2 else None)
    if not calibrated:
        verdict["note"] = "A/A miscalibrated: economic criteria are not evaluated (inconclusive)"
    Path(a.out).write_text(json.dumps(dict(protocol="2026-10-31-p74-c-confirmatory-protocol", rounds=ROUNDS,
        verdict=verdict, rows=rows, decomposition=decomposition,
        identified=dict(
            T2_minus_T1="P74 prefix exposure + handoff H + monitor M (suffix code identical); terms not separable",
            T2_minus_Free="on never-switching non-bypassed blocks: monitor M + combined-skeleton/layout; not separable",
            T1_minus_T0="refactor and layout tax of the combined function in shipped mode",
            not_identified="H, M and per-event cost separately; prefix exposure separately from H")), indent=1) + "\n")
    print(json.dumps(verdict, indent=1))


def suite(a):
    """Freeze the adversarial suite: hashes, design, and the audited actual handoffs."""
    d = Path(a.dir)
    intent = json.loads((d / "intent.json").read_text())
    paths = Path(a.files).read_text().split()
    audits = [[] for _ in paths]
    for line in Path(a.audit).read_text().splitlines():
        if line.startswith("AUDIT\t"):
            f = list(map(int, line.split("\t")[1:]))
            audits[f[0]].append(dict(block=f[1], checkpoint_interval=f[2], symbols=f[3], bypass=bool(f[6]),
                switch_consumed=f[7] or None, output_before_switch=f[8], f_before=f[9]))
    files, failures = [], []
    for i, (p, it) in enumerate(zip(paths, intent["files"])):
        assert Path(p).stem == it["name"]
        decoded = bz2.decompress(Path(p).read_bytes())
        acts = audits[i]
        assert len(acts) == 2 and all(x["block"] == 0 for x in acts)
        o = it["oracle_switch"]
        ok = all((x["switch_consumed"] == (o["consumed"] if o else None)) and
                 (not o or x["output_before_switch"] == o["output"] and x["f_before"] == o["f_before"]) and
                 x["bypass"] == it["bypass"] for x in acts)
        expect = it["design"]["expect"]
        if expect.startswith("switch"):
            ok &= o is not None and o["output"] >= it["suffix_output_start"]
        else:
            ok &= o is None
        if not ok:
            failures.append(it["name"])
        files.append(dict(name=it["name"], compressed_sha256=sha(p), compressed_bytes=Path(p).stat().st_size,
            decoded_sha256=hashlib.sha256(decoded).hexdigest(), decoded_bytes=len(decoded), intent=it,
            audit=acts, handoff_verified=ok))
    out = dict(generator="examples/p74_suite.rs", base_seed=intent["base_seed"],
        command="cargo +1.90.0 run --locked --release --example p74_suite -- DIR",
        files=files, failures=failures)
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps(dict(files=len(files), failures=failures)))
    assert not failures


if __name__ == "__main__":
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="mode",required=True)
    m=sub.add_parser("manifest")
    for name in ["files","check","audit","binary","out"]:
        m.add_argument("--"+name,required=True)
    m.add_argument("--roles", help="file of one role per corpus line (confirmatory)")
    s=sub.add_parser("summarize", help="the 2026-10-31 exploratory three-arm analysis")
    for name in ["manifest","raw","out"]:
        s.add_argument("--"+name,required=True)
    c=sub.add_parser("confirm", help="the confirmatory five-arm analysis")
    for name in ["manifest","raw","out"]:
        c.add_argument("--"+name,required=True)
    u=sub.add_parser("suite", help="freeze the adversarial suite")
    for name in ["dir","files","audit","out"]:
        u.add_argument("--"+name,required=True)
    a=p.parse_args()
    dict(manifest=manifest, summarize=summarize, confirm=confirm, suite=suite)[a.mode](a)
