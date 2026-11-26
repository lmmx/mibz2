#!/usr/bin/env python3
"""bench-kernel-count's report (docs/journal/2026-11-25-kernel-count-registration.md). A
DIAGNOSTIC (RULES C1a): it decides nothing, and prints no verdict.

    python3 bench/kernel_count.py RUNS.jsonl --rounds 12 --reps 3 [--tsv-dir DIR]

From bench/kernel_count_run.py's records:
- **An observation** is a cell in a round: the mean of its REPS decodes' wall, CPU and peak
  RSS (`ab child`'s `wall`, `cpu`, `rss_kb`). A cell's figure is the median of its rounds'
  observations.
- **A paired ratio** of cell A to cell B is taken round by round (A's observation over B's in
  the same round), over the rounds both have. Its point is the geometric mean, with a 95%
  t-interval on the logs (n - 1 degrees of freedom); the median of the rounds' ratios and the
  rounds A was lower are beside it.
- **Lazy against eager at k = 1..9:** lazy-kNN over eager-kNN, wall, CPU and peak RSS.
- **The cap:** lazy k = 7, 9 and 10 over lazy 8, and lazy 10 over lazy 7.
- **The public API at t = 3..12 asked:** what it built (every probe's threads, window, lazy,
  service, pipelined: one configuration, or each one seen), its figures, its ratio to the
  explicit cell of the same build (where it built workers with a window of 4 a worker), and to
  lazy 8.
- **Validity:** every decode succeeded with one output length and one hash, every cell has
  every round, every explicit probe built as asked. Anything else is printed as INCOMPLETE or
  INVALID with the reason; the figures are printed either way.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import statistics
import sys

# two-sided 95% t quantiles, by degrees of freedom
T975 = {
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
    11: 2.201,
    12: 2.179,
    13: 2.160,
    14: 2.145,
    15: 2.131,
    16: 2.120,
    17: 2.110,
    18: 2.101,
    19: 2.093,
    20: 2.086,
    21: 2.080,
    22: 2.074,
    23: 2.069,
    24: 2.064,
    25: 2.060,
    26: 2.056,
    27: 2.052,
    28: 2.048,
    29: 2.045,
    30: 2.042,
}
LAZY_KS = list(range(1, 11))
EAGER_KS = list(range(1, 10))
API_TS = list(range(3, 13))
FIGS = ("wall", "cpu", "rss_kb")


def t975(df):
    return T975.get(df, 1.96 if df > 30 else float("nan"))


def ratio_stats(pairs):
    """pairs: [(a, b)] by round -> dict(n, gm, lo, hi, med, a_lower)"""
    rs = [a / b for a, b in pairs if a > 0 and b > 0]
    n = len(rs)
    if n == 0:
        return {"n": 0}
    logs = [math.log(r) for r in rs]
    m = statistics.fmean(logs)
    out = {
        "n": n,
        "gm": math.exp(m),
        "med": statistics.median(rs),
        "a_lower": sum(r < 1 for r in rs),
    }
    if n >= 2:
        h = t975(n - 1) * statistics.stdev(logs) / math.sqrt(n)
        out["lo"], out["hi"] = math.exp(m - h), math.exp(m + h)
    return out


def fmt_ratio(s):
    if not s or s.get("n", 0) == 0:
        return "n/a"
    ci = f" [{s['lo']:.3f}, {s['hi']:.3f}]" if "lo" in s else " [n<2]"
    return f"{s['gm']:.3f}{ci}"


def load(path):
    recs = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                recs.append(json.loads(line))
    return recs


def observations(recs):
    """{cell: {round: {fig: mean}}} from the timed sections' successful decodes"""
    acc = {}
    for r in recs:
        if r.get("section") not in ("explicit", "api") or not r.get("ok"):
            continue
        acc.setdefault(r["cell"], {}).setdefault(r["round"], []).append(r)
    obs = {}
    for cell, by in acc.items():
        for rnd, rs in by.items():
            obs.setdefault(cell, {})[rnd] = {
                f: statistics.fmean(float(x[f]) for x in rs) for f in FIGS
            } | {"reps": len(rs), "others_busy": max(x.get("others_busy", 0) for x in rs)}
    return obs


def paired(obs, a, b, fig):
    oa, ob = obs.get(a, {}), obs.get(b, {})
    return [(oa[r][fig], ob[r][fig]) for r in sorted(oa) if r in ob]


def med(obs, cell, fig):
    v = [o[fig] for o in obs.get(cell, {}).values()]
    return statistics.median(v) if v else float("nan")


def lazy(k):
    return f"lazy-k{k:02d}"


def eager(k):
    return f"eager-k{k:02d}"


def api(t):
    return f"api-t{t:02d}"


def report(recs, rounds, reps, tsv_dir=None, out=sys.stdout):
    p = lambda *a: print(*a, file=out)  # noqa: E731
    obs = observations(recs)
    problems, invalid = [], []
    timed = [r for r in recs if r.get("section") in ("explicit", "api")]
    failed = [r for r in timed if not r.get("ok")]
    if failed:
        invalid.append(f"{len(failed)} decodes failed ({sorted({r['cell'] for r in failed})})")
    outs = {r.get("out") for r in timed if r.get("ok")}
    hashes = {r.get("hash") for r in timed if r.get("ok")}
    if len(outs) > 1 or len(hashes) > 1:
        invalid.append(
            f"the decodes' outputs differ: lengths {sorted(outs)}, hashes {sorted(hashes)}"
        )
    want = [lazy(k) for k in LAZY_KS] + [eager(k) for k in EAGER_KS] + [api(t) for t in API_TS]
    for c in want:
        have = obs.get(c, {})
        short = [r for r in range(rounds) if r not in have or have[r]["reps"] < reps]
        if short:
            problems.append(f"{c}: rounds without all {reps} decodes: {short}")
    probes = [r for r in recs if r.get("section") == "probe" and "as_asked" in r]
    not_asked = [r["cell"] for r in probes if not r["as_asked"]]
    if not_asked:
        invalid.append(f"explicit cells not built as asked: {not_asked}")
    if not probes:
        problems.append("no probes of the explicit cells")
    built = {}
    for r in recs:
        if r.get("section") == "api-built" or (
            r.get("section") == "probe" and r.get("cell", "").startswith("api-")
        ):
            key = (
                r.get("threads"),
                r.get("window"),
                r.get("lazy"),
                r.get("service"),
                r.get("pipelined"),
            )
            built.setdefault(r["cell"], {}).setdefault(key, 0)
            built[r["cell"]][key] += 1
    for t in API_TS:
        if len(built.get(api(t), {})) > 1:
            problems.append(f"{api(t)}: the API built more than one configuration")
        if not built.get(api(t)):
            problems.append(f"{api(t)}: not probed")
    busy = sum(1 for c in obs.values() for o in c.values() if o["others_busy"] > 1.0)

    p("# bench-kernel-count: lazy against eager below 8, and the worker-count cap")
    p()
    p("DIAGNOSTIC (RULES C1a): decides nothing. No verdict is printed; a later adoption")
    p("registration decides any production default.")
    p()
    status = "INVALID" if invalid else ("INCOMPLETE" if problems else "COMPLETE")
    p(f"Status: {status}")
    for x in invalid + problems:
        p(f"- {x}")
    p(
        f"- decodes: {len(timed)} timed ({len(failed)} failed); output {sorted(outs)} bytes, {len(hashes)} distinct hash(es)"
    )
    p(f"- observations with the machine's other work above 1 core: {busy}")
    p()
    p("An observation is the mean of a cell's decodes in a round; a cell's figure is the median")
    p("of its rounds. Ratios are paired by round: geometric mean [95% t-interval], the median")
    p("ratio, and the rounds the first arm was lower (of n).")
    p()

    def row_ratio(a, b, fig):
        s = ratio_stats(paired(obs, a, b, fig))
        return s

    tsv = []
    p("## 1. Lazy against eager, k workers, window 4k (`BzReader::parallel_workers`)")
    p()
    p(
        "| k | lazy wall s | eager wall s | wall lazy/eager [95%] | median ratio | lazy lower | lazy MB | eager MB | RSS lazy/eager [95%] | lazy CPU s | eager CPU s | CPU lazy/eager [95%] |"
    )
    p("|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|")
    for k in EAGER_KS:
        a, b = lazy(k), eager(k)
        w, r_, c = (row_ratio(a, b, f) for f in FIGS)
        p(
            f"| {k} | {med(obs, a, 'wall'):.4f} | {med(obs, b, 'wall'):.4f} | {fmt_ratio(w)} | "
            f"{w.get('med', float('nan')):.3f} | {w.get('a_lower', 0)}/{w.get('n', 0)} | "
            f"{med(obs, a, 'rss_kb') / 1024:.1f} | {med(obs, b, 'rss_kb') / 1024:.1f} | {fmt_ratio(r_)} | "
            f"{med(obs, a, 'cpu'):.3f} | {med(obs, b, 'cpu'):.3f} | {fmt_ratio(c)} |"
        )
        for fig, s in zip(FIGS, (w, r_, c)):
            tsv.append(("lazy_vs_eager", k, a, b, fig, s))
    p()
    p("## 2. The cap: lazy workers at k = 7, 8, 9, 10 (window 4k)")
    p()
    p(
        "| k | wall s | MB | CPU s | wall k/8 [95%] | median | k lower | RSS k/8 [95%] | CPU k/8 [95%] |"
    )
    p("|--:|--:|--:|--:|--:|--:|--:|--:|--:|")
    for k in (7, 8, 9, 10):
        a = lazy(k)
        if k == 8:
            p(
                f"| 8 | {med(obs, a, 'wall'):.4f} | {med(obs, a, 'rss_kb') / 1024:.1f} | {med(obs, a, 'cpu'):.3f} | (reference) | | | | |"
            )
            continue
        w, r_, c = (row_ratio(a, lazy(8), f) for f in FIGS)
        p(
            f"| {k} | {med(obs, a, 'wall'):.4f} | {med(obs, a, 'rss_kb') / 1024:.1f} | {med(obs, a, 'cpu'):.3f} | "
            f"{fmt_ratio(w)} | {w.get('med', float('nan')):.3f} | {w.get('a_lower', 0)}/{w.get('n', 0)} | "
            f"{fmt_ratio(r_)} | {fmt_ratio(c)} |"
        )
        for fig, s in zip(FIGS, (w, r_, c)):
            tsv.append(("cap_vs_8", k, a, lazy(8), fig, s))
    w = row_ratio(lazy(10), lazy(7), "wall")
    tsv.append(("cap_10_vs_7", 10, lazy(10), lazy(7), "wall", w))
    p()
    p(
        f"Lazy 10 over lazy 7, wall: {fmt_ratio(w)} (median {w.get('med', float('nan')):.3f}, 10 lower in {w.get('a_lower', 0)}/{w.get('n', 0)})"
    )
    p()
    p("## 3. The public API, `BzReader::parallel(r, t)` as shipped")
    p()
    p(
        "| t asked | built: workers / window / lazy / service / pipelined (probes) | wall s | MB | CPU s | wall / same-build explicit [95%] | wall / lazy 8 [95%] | median / lazy 8 | API lower |"
    )
    p("|--:|---|--:|--:|--:|--:|--:|--:|--:|")
    for t in API_TS:
        a = api(t)
        seen = built.get(a, {})
        desc = (
            "; ".join(
                f"{th} / {wi} / {'lazy' if lz else 'eager'} / {'svc' if sv else '-'} / {'pipe' if pi else '-'} ({n})"
                for (th, wi, lz, sv, pi), n in sorted(seen.items(), key=lambda x: -x[1])
            )
            or "not probed"
        )
        same = "n/a"
        if len(seen) == 1:
            ((th, wi, lz, sv, pi),) = seen.keys()
            twin = (
                (lazy(th) if lz else eager(th))
                if not sv and not pi and th and wi == 4 * th
                else None
            )
            if twin and twin in obs:
                s = row_ratio(a, twin, "wall")
                same = f"{fmt_ratio(s)} vs {twin}"
                tsv.append(("api_vs_same_build", t, a, twin, "wall", s))
        s8 = row_ratio(a, lazy(8), "wall")
        tsv.append(("api_vs_lazy8", t, a, lazy(8), "wall", s8))
        p(
            f"| {t} | {desc} | {med(obs, a, 'wall'):.4f} | {med(obs, a, 'rss_kb') / 1024:.1f} | "
            f"{med(obs, a, 'cpu'):.3f} | {same} | {fmt_ratio(s8)} | {s8.get('med', float('nan')):.3f} | "
            f"{s8.get('a_lower', 0)}/{s8.get('n', 0)} |"
        )
    p()
    p("## 4. Every cell")
    p()
    p("| cell | rounds | wall median s | wall min s | wall max s | MB median | CPU median s |")
    p("|---|--:|--:|--:|--:|--:|--:|")
    for c in want:
        v = [o["wall"] for o in obs.get(c, {}).values()]
        if not v:
            p(f"| {c} | 0 | | | | | |")
            continue
        p(
            f"| {c} | {len(v)} | {statistics.median(v):.4f} | {min(v):.4f} | {max(v):.4f} | {med(obs, c, 'rss_kb') / 1024:.1f} | {med(obs, c, 'cpu'):.3f} |"
        )
    if tsv_dir:
        os.makedirs(tsv_dir, exist_ok=True)
        with open(os.path.join(tsv_dir, "ratios.tsv"), "w") as f:
            f.write("family\tn_or_t\tnum\tden\tfig\trounds\tgm\tlo\thi\tmedian\tnum_lower\n")
            for fam, k, a, b, fig, s in tsv:
                f.write(
                    f"{fam}\t{k}\t{a}\t{b}\t{fig}\t{s.get('n', 0)}\t{s.get('gm', '')}\t{s.get('lo', '')}\t"
                    f"{s.get('hi', '')}\t{s.get('med', '')}\t{s.get('a_lower', '')}\n"
                )
        with open(os.path.join(tsv_dir, "observations.tsv"), "w") as f:
            f.write("cell\tround\treps\twall\tcpu\trss_kb\tothers_busy\n")
            for c in sorted(obs):
                for rnd in sorted(obs[c]):
                    o = obs[c][rnd]
                    f.write(
                        f"{c}\t{rnd}\t{o['reps']}\t{o['wall']:.5f}\t{o['cpu']:.5f}\t{o['rss_kb']:.0f}\t{o['others_busy']:.3f}\n"
                    )
    return status


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--tsv-dir")
    a = ap.parse_args(argv)
    status = report(load(a.runs), a.rounds, a.reps, a.tsv_dir)
    return 1 if status == "INVALID" else 0


if __name__ == "__main__":
    sys.exit(main())
