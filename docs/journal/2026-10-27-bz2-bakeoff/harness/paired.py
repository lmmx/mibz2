"""paired.py RESULTS.jsonl BASE : each decoder's wall and CPU against BASE's, paired by
round (interleaved runs only: rounds 1.., run 0 hashed). The geometric mean of the
per-round ratios and its 95% interval (t, rounds - 1 degrees of freedom). < 1 is faster."""
import json, math, statistics as st, sys
from collections import defaultdict

T95 = {1: 12.71, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447}


def main() -> None:
    recs = [json.loads(l) for l in open(sys.argv[1])]
    base = sys.argv[2]
    groups = defaultdict(dict)
    for r in recs:
        groups[(r["workload"], r["threads"])][r["decoder"]] = r
    print("| workload | threads | decoder | wall / " + base + " (95% CI) | CPU / " + base + " (95% CI) |")
    print("|---|--:|---|--:|--:|")
    for (w, t), g in groups.items():
        if base not in g:
            continue
        for d, r in g.items():
            if d == base:
                continue
            cells = []
            for k in ("wall_s", "cpu_s"):
                n = min(len(r["runs"]), len(g[base]["runs"]))
                x = [math.log(r["runs"][i][k] / g[base]["runs"][i][k]) for i in range(1, n)]
                if len(x) < 2:
                    cells.append("-")
                    continue
                m, h = st.mean(x), T95[len(x) - 1] * st.stdev(x) / math.sqrt(len(x))
                cells.append(f"{math.exp(m):.2f} ({math.exp(m - h):.2f}-{math.exp(m + h):.2f})")
            print(f"| {w} | {t} | {d} | {cells[0]} | {cells[1]} |")


if __name__ == "__main__":
    main()
