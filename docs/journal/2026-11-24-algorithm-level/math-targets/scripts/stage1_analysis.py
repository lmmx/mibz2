"""Stage 1 (bench-character) analysis: lower bounds, burst/rate frontiers, starvation episodes, progress.

Usage:
  python3 -I stage1_analysis.py BUNDLE_DIR OUT_DIR

BUNDLE_DIR is an extracted `character/run-*` bundle (it holds tsv/observations.tsv and
blocks-r00..r03.tsv). OUT_DIR receives one TSV per analysis, plus a summary.txt.

What each output is (reading/hybrid-dynamical-systems/relevance/measurements.md):
- lower_bound.tsv (item 3): per count, the resource-work lower bound L = max(scanner busy,
  mean worker busy, reader active) from the grid decodes, the measured wall, and wall / L.
  L is a LOWER bound: wall / L >= 1 is the share not explained by resource work alone
  (dependencies, starvation, blocking). Instrumented decodes (ledgers on); stage 1 flagged
  PERTURBED at 10 and 20.
- burst_frontier.tsv (item 2): per count and event type, the empirical burst requirement
  b(rho) = sup_{s<t} [N(t) - N(s) - rho (t - s)]_+ for rho = f x the decode's mean event rate,
  f in {1.0, 1.1, 1.25, 1.5, 2.0}, from each round's trace. A description of these traces only:
  not a guarantee, and not a window size without a service curve.
- starvation.tsv (item 5): per count, workers' idle episodes between a block's ready and the
  same worker's next got: their number, total and largest, per decode.
- progress.tsv (item 5): per count, blocks consumed per ms in 20 equal windows of the decode,
  and the head-of-line wait: for each block, t_taken - max(t_ready, previous block's t_taken).

Times in the traces are nanoseconds from the decode's start. Counts are reconstructed by
sorting each event type's times; the q_* columns are positions in the shared event order,
not counts, and are used only to check the order.
"""
import csv, glob, math, os, statistics as st, sys
from collections import defaultdict

bundle, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)


def read_tsv(path):
    with open(path) as f:
        return list(csv.DictReader(f, delimiter="\t"))


# ---- 1. lower bounds, from the grid decodes ----
obs = [r for r in read_tsv(os.path.join(bundle, "tsv", "observations.tsv"))
       if r["section"] == "grid" and r.get("on", "1") in ("1", "True", "true")]
by_k = defaultdict(list)
for r in obs:
    try:
        wall = float(r["wall"])
        L = max(float(r["scanner_busy"]), float(r["worker_busy_mean"]), float(r["reader_active"]))
    except (ValueError, KeyError):
        continue
    binding = max(("scanner", float(r["scanner_busy"])), ("workers", float(r["worker_busy_mean"])),
                  ("reader", float(r["reader_active"])), key=lambda x: x[1])[0]
    by_k[int(r["k"])].append((wall, L, binding))
with open(os.path.join(out, "lower_bound.tsv"), "w") as f:
    f.write("k\tn\twall_median_s\tL_median_s\twall_over_L_median\twall_over_L_p10\twall_over_L_p90\tbinding_most\n")
    for k in sorted(by_k):
        v = by_k[k]
        ratios = sorted(w / L for w, L, _ in v)
        b = max(set(x[2] for x in v), key=[x[2] for x in v].count)
        q = lambda p: ratios[min(len(ratios) - 1, int(p * (len(ratios) - 1)))]
        f.write(f"{k}\t{len(v)}\t{st.median(w for w,_,_ in v):.4f}\t{st.median(L for _,L,_ in v):.4f}\t"
                f"{st.median(ratios):.3f}\t{q(.1):.3f}\t{q(.9):.3f}\t{b}\n")

# ---- 2–5. from the per-block traces ----
EVENTS = ["t_scan_start", "t_issued", "t_got", "t_ready", "t_taken", "t_consumed"]
FACTORS = [1.0, 1.1, 1.25, 1.5, 2.0]


def burst(times, rho):
    """sup over s<t of N(t)-N(s)-rho(t-s), on sorted event times (ns); rho in events/ns."""
    # the window [t_j, t_i] holds i - j + 1 events: maximise (i + 1 - rho t_i) + (rho t_j - j) over j <= i
    best, hi = 0.0, -math.inf
    for i, t in enumerate(times):
        hi = max(hi, rho * t - i)
        best = max(best, (i + 1) - rho * t + hi)
    return max(0.0, best)


frontier = defaultdict(list)     # (k, event, factor) -> [burst]
starv = defaultdict(list)        # k -> [(episodes, total_ms, max_ms)]
prog = defaultdict(list)         # k -> [[rate per window]]
hol = defaultdict(list)          # k -> [hol waits ms]
order_ok = defaultdict(int)
for path in sorted(glob.glob(os.path.join(bundle, "blocks-r*.tsv"))):
    rows = read_tsv(path)
    per_k = defaultdict(list)
    for r in rows:
        per_k[int(r["k"])].append(r)
    for k, blocks in per_k.items():
        blocks.sort(key=lambda r: int(r["place"]))
        end = max(float(r["t_consumed"]) for r in blocks)
        # check: within each block, the causal positions increase issued < enqueued < got < ready < taken < consumed
        order_ok[k] += all(int(r["q_issued"]) < int(r["q_enqueued"]) < int(r["q_got"]) < int(r["q_ready"])
                           < int(r["q_taken"]) < int(r["q_consumed"]) for r in blocks)
        for ev in EVENTS:
            ts = sorted(float(r[ev]) for r in blocks)
            span = ts[-1] - ts[0]
            if span <= 0:
                continue
            mean_rate = (len(ts) - 1) / span
            for fct in FACTORS:
                frontier[(k, ev, fct)].append(burst(ts, fct * mean_rate))
        # starvation: per worker, gap from its block's ready to its next got
        byw = defaultdict(list)
        for r in blocks:
            byw[r["worker"]].append((float(r["t_got"]), float(r["t_ready"])))
        eps, tot, mx = 0, 0.0, 0.0
        for w, jobs in byw.items():
            jobs.sort()
            for (g0, r0), (g1, _) in zip(jobs, jobs[1:]):
                gap = g1 - r0
                if gap > 1e4:   # >10 µs counts as an episode
                    eps += 1; tot += gap; mx = max(mx, gap)
        starv[k].append((eps, tot / 1e6, mx / 1e6))
        # progress: consumed per ms in 20 windows
        cons = sorted(float(r["t_consumed"]) for r in blocks)
        w = end / 20
        counts = [0] * 20
        for t in cons:
            counts[min(19, int(t / w))] += 1
        prog[k].append([c / (w / 1e6) for c in counts])
        # head-of-line wait for the reader
        prev = 0.0
        for r in blocks:
            hol[k].append(max(0.0, float(r["t_taken"]) - max(float(r["t_ready"]), prev)) / 1e6)
            prev = float(r["t_taken"])

with open(os.path.join(out, "burst_frontier.tsv"), "w") as f:
    f.write("k\tevent\trate_factor\tburst_median\tburst_max\trounds\n")
    for (k, ev, fct), v in sorted(frontier.items()):
        f.write(f"{k}\t{ev}\t{fct}\t{st.median(v):.1f}\t{max(v):.1f}\t{len(v)}\n")
with open(os.path.join(out, "starvation.tsv"), "w") as f:
    f.write("k\trounds\tepisodes_median\tidle_ms_median\tlongest_ms_median\n")
    for k in sorted(starv):
        v = starv[k]
        f.write(f"{k}\t{len(v)}\t{st.median(e for e,_,_ in v):.0f}\t{st.median(t for _,t,_ in v):.2f}\t{st.median(m for _,_,m in v):.3f}\n")
with open(os.path.join(out, "progress.tsv"), "w") as f:
    f.write("k\t" + "\t".join(f"w{i:02d}" for i in range(20)) + "\tcv_across_windows\thol_wait_ms_median\thol_wait_ms_p99\n")
    for k in sorted(prog):
        med = [st.median(p[i] for p in prog[k]) for i in range(20)]
        core = med[2:18]
        cv = st.pstdev(core) / st.mean(core) if st.mean(core) else float("nan")
        h = sorted(hol[k])
        f.write(f"{k}\t" + "\t".join(f"{x:.2f}" for x in med) +
                f"\t{cv:.3f}\t{st.median(h):.4f}\t{h[int(.99*(len(h)-1))]:.4f}\n")
with open(os.path.join(out, "summary.txt"), "w") as f:
    f.write(f"bundle: {bundle}\n")
    f.write("per-block causal order issued<enqueued<got<ready<taken<consumed holds in decodes: " +
            ", ".join(f"k{k}:{order_ok[k]}" for k in sorted(order_ok)) + "\n")
print("wrote", sorted(os.listdir(out)))
