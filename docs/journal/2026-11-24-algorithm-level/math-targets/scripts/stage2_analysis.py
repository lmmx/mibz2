"""Stage 2a (bench-character-stage2) analysis: is the screen's local model predictive, and what does it price?

Usage:
  python3 -I stage2_analysis.py REPORT_OUT_DIR RUNS_JSONL PLAN_JSON OUT_DIR

REPORT_OUT_DIR holds the recipe report's stage2.tsv (the screen coefficients, gates, memory and
placement contrasts). RUNS_JSONL and PLAN_JSON are the bundle's own. OUT_DIR receives:

- validation.tsv: the held-out arm (A3 and A7 both +10%, at k 8 and 10) against the prediction
  of the log-additive local model T/T0 = prod (1 + d_i)^C_i built from the one-at-a-time screen.
  The prediction's interval treats the two coefficients' intervals as independent normals.
  A predictor, judged on data it was not fitted to (journal §70: lower bounds prune, validated
  predictors rank, proved upper bounds certify).
- surface.tsv: the 3x3 surface at k* read as elasticities: the scanner's (workers undosed) and
  the workers' (scanner undosed), slowing only. Max-plus caution: these are elasticities to
  SLOWING. A speed-up of the binding activity buys at most its elasticity, and less once the
  other term binds (bottleneck migration, §61).
- pricing.tsv: for a cut of f in the scanner's main activity (A3), the wall implied by the
  screen's coefficient, (1 - f)^C, at k 8 and 10, as an optimistic bound under the caveat above.
"""
import csv, json, math, os, statistics as st, sys
from collections import defaultdict

rep, runs_path, plan_path, out = sys.argv[1:5]
os.makedirs(out, exist_ok=True)

rows = list(csv.DictReader(open(os.path.join(rep, "stage2.tsv")), delimiter="\t"))
screen = {(int(r["k"]), r["coord"]): r for r in rows if r["section"] == "screen"}


def coef(k, c):
    r = screen[(k, c)]
    C, lo, hi = float(r["C"]), float(r["lo"]), float(r["hi"])
    return C, (hi - lo) / (2 * 1.96)


plan = json.load(open(plan_path))
cells = {i: c for i, c in enumerate(plan["cells"])}
walls = defaultdict(dict)  # (section, k, arm) -> {block: wall}
for line in open(runs_path):
    d = json.loads(line)
    if d.get("kind") != "decode" or d.get("on") != 0:
        continue
    c = cells.get(d.get("cell"))
    if c is None:
        continue
    walls[(c["section"], c["k"], c["arm"])][d.get("block")] = d["wall_s"]


def paired(sec_a, arm_a, sec_b, arm_b, k):
    a, b = walls.get((sec_a, k, arm_a), {}), walls.get((sec_b, k, arm_b), {})
    logs = [math.log(a[x] / b[x]) for x in sorted(set(a) & set(b))]
    if len(logs) < 2:
        return float("nan"), float("nan"), float("nan"), len(logs)
    m, s = st.fmean(logs), st.stdev(logs) / len(logs) ** 0.5
    t = {1: 12.71, 2: 4.30, 3: 3.18, 4: 2.78, 5: 2.57, 6: 2.45, 7: 2.36}.get(len(logs) - 1, 2.0)
    return math.exp(m), math.exp(m - t * s), math.exp(m + t * s), len(logs)


with open(os.path.join(out, "validation.tsv"), "w") as f:
    f.write("k\tobserved\tobs_lo\tobs_hi\tn\tpredicted\tpred_lo\tpred_hi\tinside\n")
    for k in (8, 10):
        o, olo, ohi, n = paired("validation", "A3+A7_10", "screen", "base", k)
        if math.isnan(o):
            # the validation arm's reference may live in its own section
            o, olo, ohi, n = paired("validation", "A3+A7_10", "validation", "base", k)
        (c3, s3), (c7, s7) = coef(k, "A3"), coef(k, "A7")
        L = math.log(1.10)
        mu, sd = (c3 + c7) * L, math.hypot(s3, s7) * L
        p, plo, phi = math.exp(mu), math.exp(mu - 1.96 * sd), math.exp(mu + 1.96 * sd)
        inside = "yes" if not math.isnan(o) and plo <= o <= phi else ("n/a" if math.isnan(o) else "no")
        f.write(f"{k}\t{o:.4f}\t{olo:.4f}\t{ohi:.4f}\t{n}\t{p:.4f}\t{plo:.4f}\t{phi:.4f}\t{inside}\n")

# the surface at k*: rows scanner 0/10/20%, columns workers 0/10/20% (mean log slowdown)
ks = plan["kstar"]
surf = {}
for (sec, k, arm), w in walls.items():
    if sec == "surface" and k == ks and arm.startswith("s"):
        surf[arm] = w
base = surf.get("s00w00", {})
with open(os.path.join(out, "surface.tsv"), "w") as f:
    f.write("arm\tscanner_dose\tworker_dose\tmean_log_slowdown\telasticity_if_one_axis\n")
    for arm in sorted(surf):
        s, w = int(arm[1:3]) / 100, int(arm[4:6]) / 100
        logs = [math.log(surf[arm][b] / base[b]) for b in sorted(set(surf[arm]) & set(base))]
        m = st.fmean(logs) if logs else float("nan")
        el = ""
        if (s == 0) != (w == 0):
            el = f"{m / math.log(1 + max(s, w)):.3f}"
        f.write(f"{arm}\t{s}\t{w}\t{m:+.4f}\t{el}\n")

with open(os.path.join(out, "pricing.tsv"), "w") as f:
    f.write("k\tC_A3\tC_lo\tcut_in_A3\twall_ratio_at_C\twall_ratio_at_C_lo\n")
    for k in (8, 10):
        C, sd = coef(k, "A3")
        lo = float(screen[(k, "A3")]["lo"])
        for cut in (0.05, 0.08, 0.10, 0.20):
            f.write(f"{k}\t{C:.3f}\t{lo:.3f}\t{cut}\t{(1 - cut) ** C:.4f}\t{(1 - cut) ** lo:.4f}\n")

print("wrote", sorted(os.listdir(out)))
