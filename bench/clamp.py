#!/usr/bin/env python3
"""The clamp run's reading (`just bench-clamp`): the shipped decoder (`mibz2`) against the
same decoder asked for min(THREADS, physical cores) threads (`mibz2-clamp`), on the Wikidata
sample and a long file of copies of it, paired by round.

Four tables, all wall ratios as the geometric mean of the per-round ratios with a 95%
t interval (as `ab summary` computes them):

1. the rule: mibz2-clamp / mibz2 at each thread count above the cores (12, 16, 20 on the
   i9). The verdict is ADOPT only if every one of them, on every corpus, has its interval
   entirely below 1;
2. the A/A control: mibz2-clamp / mibz2 at the cores' count and below (8, 9, 10), where the
   two arms build the same decoder (`--spread` pins a run of T <= cores threads to T cores,
   so the cores the decoder counts are T). If two or more of these exclude 1, the run is
   NOT READ (noise or drift) and is repeated;
3. the harness: mibz2-clamp at each T above the cores over mibz2 at the cores' count. The
   decoder is the same one (asked for the cores' count); what differs is the harness's
   shape past the cores (unpinned, and no held spinner above `--hold-max-threads`);
4. 8 against 10: mibz2 at 8 threads over mibz2 at 10, and at 9 over 10, on each corpus
   (one binary, the same rounds). Reported, not decided on.

    clamp.py RUNS.jsonl [--cores N]     (N: the cores' count; default 10)
"""
import collections
import json
import math
import statistics
import sys

SHIPPED, CLAMP = "mibz2", "mibz2-clamp"
T95 = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
    2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
    2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
]


def ratio_ci(rs):
    """The geometric mean of paired ratios and its 95% interval (`ab`'s `ratio_ci`)"""
    logs = [math.log(r) for r in rs]
    n = len(logs)
    m = sum(logs) / n
    if n < 2:
        return math.exp(m), float("nan"), float("nan")
    sd = math.sqrt(sum((x - m) ** 2 for x in logs) / (n - 1))
    h = (T95[n - 2] if n - 2 < len(T95) else 1.96) * sd / math.sqrt(n)
    return math.exp(m), math.exp(m - h), math.exp(m + h)


def read(path):
    wall, rss, outputs, errors = {}, collections.defaultdict(list), collections.defaultdict(set), 0
    with open(path) as f:
        for line in f:
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("err"):
                errors += 1
                continue
            key = (r.get("corpus", ""), r["arm"], r["threads"])
            wall[key + (r["round"],)] = r["wall"]
            rss[key].append(r.get("rss_kb", 0) / 1024)
            outputs[r.get("corpus", "")].add((r.get("out"), r.get("sha256") or r.get("hash")))
    return wall, rss, outputs, errors


def paired(wall, corpus, a, b):
    """a's wall over b's, round by round, where both ran (a, b: (arm, threads))"""
    rounds = sorted({k[3] for k in wall if k[0] == corpus})
    return [
        wall[(corpus,) + a + (i,)] / wall[(corpus,) + b + (i,)]
        for i in rounds
        if (corpus,) + a + (i,) in wall and (corpus,) + b + (i,) in wall
    ]


def fmt(rs):
    if len(rs) < 2:
        return f"{'-':>8s} {'(fewer than 2 rounds)':>20s}  n={len(rs)}", None
    g, lo, hi = ratio_ci(rs)
    return f"{g:8.3f} [{lo:.3f}, {hi:.3f}]  n={len(rs)}", (g, lo, hi)


def main(argv):
    path = argv[1]
    cores = int(argv[argv.index("--cores") + 1]) if "--cores" in argv else 10
    wall, rss, outputs, errors = read(path)
    differ = sorted(c for c, o in outputs.items() if len(o) != 1)
    print(f"readings {len(wall)}, with an error {errors}, corpora {len(outputs)}, with differing outputs {len(differ)}")
    if errors or differ or not wall:
        print(f"NOT READ: an arm failed, or the arms' outputs differ on a corpus {differ}")
        return 1
    corpora = sorted(outputs)
    threads = sorted({k[2] for k in wall})
    above = [t for t in threads if t > cores]
    at_or_below = [t for t in threads if t <= cores]

    print(f"\n1. the rule: {CLAMP} / {SHIPPED}, wall, above the cores ({cores})")
    rule = []
    for c in corpora:
        for t in above:
            s, ci = fmt(paired(wall, c, (CLAMP, t), (SHIPPED, t)))
            ok = ci is not None and ci[2] < 1
            rule.append(ok)
            rs = lambda arm: statistics.median(rss.get((c, arm, t), [float("nan")]))
            print(f"  {c:16s}{t:4d} threads  {s}  {'below 1' if ok else 'not below 1'}"
                  f"   peak RSS {rs(SHIPPED):.0f} -> {rs(CLAMP):.0f} MB")

    print(f"\n2. the A/A control: {CLAMP} / {SHIPPED} at the cores' count and below (one decoder)")
    excluded = 0
    for c in corpora:
        for t in at_or_below:
            s, ci = fmt(paired(wall, c, (CLAMP, t), (SHIPPED, t)))
            out = ci is not None and not (ci[1] <= 1 <= ci[2])
            excluded += out
            print(f"  {c:16s}{t:4d} threads  {s}  {'EXCLUDES 1' if out else ''}")

    print(f"\n3. the harness past the cores: {CLAMP} at T over {SHIPPED} at {cores} (one decoder, asked for {cores})")
    for c in corpora:
        for t in above:
            s, _ = fmt(paired(wall, c, (CLAMP, t), (SHIPPED, cores)))
            print(f"  {c:16s}{t:4d} threads  {s}")

    print(f"\n4. fewer workers than cores: {SHIPPED} at 8 and 9 threads over {SHIPPED} at {cores}")
    for c in corpora:
        for t in (8, 9):
            if t in threads and cores in threads and t < cores:
                s, _ = fmt(paired(wall, c, (SHIPPED, t), (SHIPPED, cores)))
                print(f"  {c:16s}{t:4d} / {cores}  {s}")

    if excluded >= 2:
        verdict = f"NOT READ: {excluded} of the A/A control's intervals exclude 1; repeat the run"
        status = 1
    elif rule and all(rule):
        verdict = "ADOPT: the clamp is faster at every thread count above the cores on every corpus"
        status = 0
    else:
        verdict = "KEEP: the clamp is not faster at every thread count above the cores on every corpus"
        status = 0
    print(f"\nverdict: {verdict}")
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv))
