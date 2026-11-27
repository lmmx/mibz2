"""Per-cell head-arm bias of CI's paired bench, from downloaded bench-pr-* artifacts.

usage: python3 analyse.py ARTDIR [OUT.tsv]
ARTDIR holds one subdirectory per run, each the unzipped artifact (runs.jsonl,
result.json). Runs whose head and base differ in the scanner (#1034: 782f4feb,
d8b47e0c) are excluded from the null pool by head SHA prefix. Prints, per
corpus x threads, the geometric-mean head/base wall ratio over all paired rounds
of the null pool, a 95% cluster-bootstrap interval over runs (2000 resamples,
seed 1), and the median head/base peak-RSS ratio. OUT.tsv, if given, gets one row
per run and cell: run head, base, corpus, threads, geometric-mean ratio.
"""
import collections, glob, json, math, os, random, statistics as st, sys

SCANNER = ("782f4feb", "d8b47e0c")
art = sys.argv[1]
per = collections.defaultdict(lambda: collections.defaultdict(list))
rss = collections.defaultdict(list)
rows = []
for d in sorted(glob.glob(os.path.join(art, "*", "runs.jsonl"))):
    meta = json.load(open(os.path.join(os.path.dirname(d), "result.json")))
    head, base = meta["head"], meta["base"]
    by = collections.defaultdict(dict)
    for line in open(d):
        r = json.loads(line)
        by[(r["corpus"], r["threads"], r["round"])][r["arm"]] = r
    cell = collections.defaultdict(list)
    for (c, t, _), arms in by.items():
        if len(arms) != 2:
            continue
        h = next(a for a in arms if "@" not in a)
        b = next(a for a in arms if "@" in a)
        x = arms[h]["wall"] / arms[b]["wall"]
        cell[(c, t)].append(x)
        if not head.startswith(SCANNER):
            per[(c, t)][head].append(x)
            rss[(c, t)].append(arms[h]["rss_kb"] / arms[b]["rss_kb"])
    for (c, t), v in cell.items():
        rows.append((head, base, c, t, math.exp(sum(map(math.log, v)) / len(v))))

random.seed(1)
for k in sorted(per):
    runs = list(per[k])
    gm = lambda rs: math.exp(st.mean(math.log(x) for r in rs for x in per[k][r]))
    bs = sorted(gm([random.choice(runs) for _ in runs]) for _ in range(2000))
    print(f"{k[0]:9}{k[1]} runs={len(runs)} pairs={sum(len(per[k][r]) for r in runs)} "
          f"gm={gm(runs):.4f} CI=({bs[50]:.4f},{bs[1949]:.4f}) rss_med={st.median(rss[k]):.4f}")
if len(sys.argv) > 2:
    with open(sys.argv[2], "w") as f:
        f.write("head\tbase\tcorpus\tthreads\tgm_ratio\n")
        for r in sorted(rows):
            f.write(f"{r[0]}\t{r[1]}\t{r[2]}\t{r[3]}\t{r[4]:.4f}\n")
