"""Whether mibz2's parallel scaling is the decoder's or the machine's (#39).

On the i9-10900K mibz2's throughput peaks at 4-5 workers and falls beyond, at a steady
clock, with IPC halving from 6 to 10 workers. Other parallel bz2 decoders on the same
corpus and machine say which: if they keep scaling past 6 workers where mibz2 falls, the
fall is mibz2's to fix; if they reach a similar knee, it is shared (the workload on this
memory system), and not by itself mibz2's pathology.

`just bench-scaling [--arms mibz2,pbz2,lbzip2] [--threads 2,4,6,8,10,20] [--rounds 6]`
runs examples/ab.rs on the benchmark's Wikidata corpus, every arm at every thread count
each round in a rotated order, back to back (`--settle 0`: no idle wait before a run, as a
release has none), then reports each arm's throughput and its scaling against its own
4-thread runs, paired by round: the geometric mean of the ratios and its 95% interval.
It writes scaling-<UTC time>.jsonl (ab's own records) and scaling-<UTC time>.json.
"""

import argparse
import datetime
import json
import math
import os
import statistics
import subprocess
import sys

from imc import CORPUS, run

AB = "target/release/examples/ab"
T95 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179]


def ratio_ci(xs):
    """the geometric mean of paired ratios and its 95% interval (t over rounds)"""
    logs = [math.log(x) for x in xs if x > 0]
    if len(logs) < 2:
        return (math.exp(logs[0]) if logs else math.nan, math.nan, math.nan)
    m, se = statistics.fmean(logs), statistics.stdev(logs) / math.sqrt(len(logs))
    t = T95[len(logs) - 2] if len(logs) - 2 < len(T95) else 1.96
    return math.exp(m), math.exp(m - t * se), math.exp(m + t * se)


def summarise(raw, arms, threads, ref, labels=None, against=None):
    """ab's records in `raw`: each arm's median throughput, its scaling against its own
    `ref`-thread runs and, given `against` (an arm), its throughput against that arm's at
    each thread count, both paired by round; printed, and returned with whether every run
    gave the same bytes, and the output's size"""
    labels = labels or {}
    name = lambda arm: labels.get(arm, arm)  # noqa: E731
    recs = [json.loads(ln) for ln in open(raw) if ln.strip()]
    bad = [r for r in recs if r.get("broken") or r.get("err")]
    hashes = {r.get("hash") for r in recs if "hash" in r}
    same = len(hashes) == 1 and not bad
    # wall[(arm, threads)][round]
    wall = {}
    for r in recs:
        if "wall" in r and not r.get("err"):
            wall.setdefault((r["arm"], r["threads"]), {})[r["round"]] = r["wall"]
    out_bytes = next((r["out"] for r in recs if "out" in r), None)
    width = max(10, *(len(name(x)) + 1 for x in arms))
    head = f"{'arm':<{width}}" + "".join(f"{t:>9}" for t in threads)
    print(f"\nsame bytes from every run: {'yes' if same else 'NO'}")
    print(f"MB/s (median)\n{head}")
    rows = []
    for arm in arms:
        cells = []
        for t in threads:
            w = list(wall.get((arm, t), {}).values())
            cells.append(out_bytes / statistics.median(w) / 1e6 if w and out_bytes else math.nan)
        print(f"{name(arm):<{width}}" + "".join(f"{c:9.0f}" for c in cells))
        rows.append({"arm": name(arm), "exe": arm, "mb_s": dict(zip(threads, cells))})

    def paired(title, key, base_of):
        print(f"\n{title}\n{head}")
        for row in rows:
            cells, row[key] = [], {}
            for t in threads:
                mine, base = wall.get((row["exe"], t), {}), base_of(row["exe"], t)
                # throughput ratio = the base's wall over this one's, round by round
                c = ratio_ci([base[k] / mine[k] for k in mine if k in base])
                row[key][t] = {"ratio": c[0], "lo": c[1], "hi": c[2]}
                cells.append(f"{c[0]:.2f}")
            print(f"{row['arm']:<{width}}" + "".join(f"{c:>9}" for c in cells))
        print("95% intervals:")
        for row in rows:
            r = row[key]
            print(
                f"{row['arm']:<{width}}"
                + "  ".join(f"{t}: {r[t]['lo']:.2f}-{r[t]['hi']:.2f}" for t in threads)
            )

    paired(
        f"throughput against the arm's own {ref}-thread runs, paired by round (95%)",
        "scaling",
        lambda arm, t: wall.get((arm, ref), {}),
    )
    if against:
        paired(
            f"throughput against {name(against)}'s at the same thread count, paired by round",
            "against",
            lambda arm, t: wall.get((against, t), {}),
        )
    # each arm's best thread count, and its best throughput against `against`'s best,
    # round by round (each round's fastest run of each arm, at whichever thread count)
    print("\nbest thread count (by median), and the best against the best, paired by round")
    for row in rows:
        t_best = max(
            threads, key=lambda t: row["mb_s"][t] if row["mb_s"][t] == row["mb_s"][t] else 0
        )
        row["best"] = {"threads": t_best, "mb_s": row["mb_s"][t_best]}
        line = f"{row['arm']:<{width}} {row['mb_s'][t_best]:6.0f} MB/s at {t_best:>2} threads"
        if against:

            def best(arm):
                per = {}
                for t in threads:
                    for k, w in wall.get((arm, t), {}).items():
                        per[k] = min(per.get(k, w), w)
                return per

            mine, base = best(row["exe"]), best(against)
            c = ratio_ci([base[k] / mine[k] for k in mine if k in base])
            row["best"]["against"] = {"ratio": c[0], "lo": c[1], "hi": c[2]}
            line += f"   {c[0]:.3f} ({c[1]:.3f}-{c[2]:.3f}) of {name(against)}'s best"
        print(line)
    return rows, same, out_bytes


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--arms", default="mibz2,pbz2,lbzip2")
    ap.add_argument("--threads", default="2,4,6,8,10,20")
    ap.add_argument("--rounds", type=int, default=6)
    ap.add_argument("--ref", type=int, default=4, help="the thread count scaling is against")
    ap.add_argument("--file", default=CORPUS, help="the corpus (default the benchmark's)")
    ap.add_argument("--note", default="", help="what was closed or running, for the record")
    a = ap.parse_args()
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [AB, a.file]:
        if not os.path.exists(p):
            sys.exit(f"bench-scaling: no {p}; `just bench-scaling` builds it")
    threads = [int(t) for t in a.threads.split(",")]
    if a.ref not in threads:
        sys.exit(f"bench-scaling: --threads must include --ref {a.ref}")
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    raw = f"scaling-{utc}.jsonl"
    argv = [AB, "run", raw, "--arms", a.arms, "--threads", a.threads]
    argv += ["--rounds", str(a.rounds), "--settle", "0", a.file]
    if subprocess.run(argv).returncode:
        sys.exit("bench-scaling: ab run failed")
    rows, same, out_bytes = summarise(raw, a.arms.split(","), threads, a.ref)
    print(f"(bench-scaling {utc}, note: {a.note or '-'})")
    res = {
        "meta": {
            "utc": utc,
            "note": a.note,
            "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
            "lbzip2": (run(["lbzip2", "--version"]).stdout or "").splitlines()[:1],
            "argv": argv,
            "ref_threads": a.ref,
        },
        "same_bytes": same,
        "output_bytes": out_bytes,
        "arms": rows,
        "records": raw,
    }
    with open(f"scaling-{utc}.json", "w") as f:
        json.dump(res, f, indent=1)
    print(f"\nbench-scaling: wrote scaling-{utc}.json and {raw}")


if __name__ == "__main__":
    main()
