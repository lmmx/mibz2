"""Paired analysis of bench/stages.py output: every round's base and arm figures, the per-round
arm/base ratio, and a 95% t-interval of the mean log ratio (as a percentage)."""
import json, math, sys, statistics as st
T975 = {5: 2.571, 10: 2.228, 14: 2.145, 15: 2.131, 16: 2.120, 19: 2.093, 20: 2.086, 24: 2.064, 29: 2.045}
recs = [json.loads(l) for l in open(sys.argv[1])]
keys = sys.argv[2].split(",") if len(sys.argv) > 2 else ["mtf", "total"]
rounds = sorted({r["round"] for r in recs})
by = {(r["round"], r["arm"]): r for r in recs}
for k in keys:
    print(f"== {k} (ns a byte, phases at 1000 GHz scale /1000)")
    lr = []
    for rd in rounds:
        b, a = by[(rd, "base")]["ps_per_byte"][k] / 1000, by[(rd, "arm")]["ps_per_byte"][k] / 1000
        lr.append(math.log(a / b))
        print(f"  r{rd:2d} base {b:.4f} arm {a:.4f} arm/base {a / b:.4f}")
    n = len(lr); m = st.mean(lr); se = st.stdev(lr) / math.sqrt(n); t = T975.get(n - 1, 2.0)
    pct = lambda x: (math.exp(x) - 1) * 100
    print(f"  n={n} mean ratio {pct(m):+.2f}% 95% CI ({pct(m - t * se):+.2f}%, {pct(m + t * se):+.2f}%); median ratio {math.exp(st.median(lr)):.4f}")
