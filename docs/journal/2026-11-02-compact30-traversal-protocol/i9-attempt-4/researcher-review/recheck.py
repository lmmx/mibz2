#!/usr/bin/env python3
"""Independent archive audit of the pre-registered c30 traversal experiment.
Reads source data; does not execute any benchmark or external process.
"""

import json, math, statistics, hashlib, tarfile, pathlib
from collections import defaultdict, Counter
from scipy.stats import t as student_t, spearmanr

root = pathlib.Path("/mnt/data/c30_archive_audit/c30-traversal-20261102T200732Z")
rows = [json.loads(x) for x in (root / "c30-traversal.jsonl").open()]
meta = next(r for r in rows if r["kind"] == "meta")
walks = [r for r in rows if r["kind"] == "walk"]
calibs = [r for r in rows if r["kind"] == "calibration"]
registered = json.loads((root / "c30-analysis.json").read_text())
R = meta["rounds"]
blocks = meta["block_rows"]
n = len(blocks)
cpus = meta["cpus"]
total = sum(blocks)
assert R == 20 and n == 434 and total == 207006515
assert sorted(set(r["kind"] for r in rows)) == ["calibration", "meta", "walk"]
assert len(walks) == (R + 1) * 4 * n
assert len(calibs) == 2 and {b["when"] for b in calibs} == {"before", "after"}
for ca in calibs:
    assert sorted(x["cpu"] for x in ca["cpus"]) == sorted(cpus)
    for x in ca["cpus"]:
        assert x["n"] == 20000
        for q in ("cpu_ns", "wall_ns"):
            z = x[q]
            assert len(z) == 5 and all(0 <= z[i] <= z[i + 1] for i in range(4))

order = sorted(range(n), key=lambda i: (-blocks[i], i))
assignment = [[] for _ in range(4)]
for s in range(0, n, 4):
    for j, idx in enumerate(order[s : s + 4]):
        assignment[j if (s // 4) % 2 == 0 else 3 - j].append(idx)
assert assignment == meta["w4_lists"]
assert [sum(blocks[i] for i in q) for q in assignment] == meta["w4_rows"]
expected = {}
for r in range(-1, R):
    for a in "PC":
        for b, nn in enumerate(blocks):
            expected[(r, "w1", a, 0, b)] = (b, nn)
        for k, lst in enumerate(assignment):
            for s, b in enumerate(lst):
                expected[(r, "w4", a, k, s)] = (b, blocks[b])
observed = {}
for w in walks:
    key = (w["round"], w["mode"], w["arm"], w["worker"], w["step"])
    assert key not in observed
    observed[key] = w
    assert w["exact"] is True
    assert w["cpu_before"] == w["cpu_after"] == cpus[w["worker"] if w["mode"] == "w4" else 0]
    assert (
        w["prep_start"]
        <= w["prep_end"]
        <= w["start"]
        <= w["end"]
        <= w["check_start"]
        <= w["check_end"]
    )
    assert 0 <= w["cpu_end"] - w["cpu_start"] <= w["end"] - w["start"] + 2000
assert set(observed) == set(expected)
for k, (bid, rr) in expected.items():
    assert observed[k]["block"] == bid and observed[k]["rows"] == rr

nsteps = max(map(len, assignment))
per = {}
details = []
gate2 = []
fullstepids = [
    s
    for s in range(nsteps)
    if len([k for k in range(4) if s < len(assignment[k])]) == 4
    and all(blocks[assignment[k][s]] >= 899000 for k in range(4))
]
for r in range(-1, R):
    for a in "PC":
        # W1 intra pass
        seq = [observed[(r, "w1", a, 0, i)] for i in range(n)]
        for x, y in zip(seq, seq[1:]):
            assert x["check_end"] <= y["prep_start"]
        steps = []
        prev = None
        for s in range(nsteps):
            v = [observed[(r, "w4", a, k, s)] for k in range(4) if s < len(assignment[k])]
            assert max(x["prep_end"] for x in v) <= min(x["start"] for x in v)
            assert max(x["end"] for x in v) <= min(x["check_start"] for x in v)
            if prev:
                assert max(x["end"] for x in prev) <= min(x["prep_start"] for x in v)
                for x in v:
                    p = observed[(r, "w4", a, x["worker"], s - 1)]
                    assert p["check_end"] <= x["prep_start"]
            steps.append(v)
            prev = v
        if r >= 0:
            w4 = [x for v in steps for x in v]
            w1 = seq

            def c(v):
                return sum(x["cpu_end"] - x["cpu_start"] for x in v)

            def wa(v):
                return sum(x["end"] - x["start"] for x in v)

            e = sum(max(x["end"] for x in v) - min(x["start"] for x in v) for v in steps)
            skval = sum(
                sum(x["rows"] for x in steps[s])
                * (max(x["start"] for x in steps[s]) - min(x["start"] for x in steps[s]))
                / (max(x["end"] for x in steps[s]) - min(x["start"] for x in steps[s]))
                for s in fullstepids
            ) / sum(sum(x["rows"] for x in steps[s]) for s in fullstepids)
            gate2.append({"round": r, "arm": a, "weighted_skew": skval})
            per[(r, a)] = {
                "w1_cpu_ns_row": c(w1) / total,
                "w4_cpu_ns_row": c(w4) / total,
                "e4_ns": e,
                "w1_wall_ns_row": wa(w1) / total,
                "w4_wall_ns_row": wa(w4) / total,
                "skew": skval,
                "w4_running": c(w4) / wa(w4),
                "w1_running": c(w1) / wa(w1),
            }
            details.append((r, a, seq, steps))
    # pass ordering, W1 then W4, arms depend on r even/odd
    arms = ("P", "C") if r % 2 == 0 else ("C", "P")
    passspans = []
    for mode in ("w1", "w4"):
        for a in arms:
            v = [x for x in walks if x["round"] == r and x["mode"] == mode and x["arm"] == a]
            passspans.append((min(x["prep_start"] for x in v), max(x["check_end"] for x in v)))
    if r == -1:
        prev_end = None
    if prev_end is not None:
        assert prev_end <= passspans[0][0]
    for x, y in zip(passspans, passspans[1:]):
        assert x[1] <= y[0]
    prev_end = passspans[-1][1]

D = {}
for name, key in [
    ("w1", "w1_cpu_ns_row"),
    ("w4", "w4_cpu_ns_row"),
    ("e4", "e4_ns"),
    ("w1_wall", "w1_wall_ns_row"),
    ("w4_wall", "w4_wall_ns_row"),
]:
    rat = [per[(r, "C")][key] / per[(r, "P")][key] for r in range(R)]
    l = [math.log(x) for x in rat]
    m = statistics.mean(l)
    se = statistics.stdev(l) / math.sqrt(R)
    tcrit = student_t.ppf(0.975, R - 1)
    D[name] = {
        "point": math.exp(m),
        "lo": math.exp(m - tcrit * se),
        "hi": math.exp(m + tcrit * se),
        "round_ratios": rat,
        "min": min(rat),
        "max": max(rat),
    }
    for tag in ("point", "lo", "hi"):
        assert abs(D[name][tag] - registered["ratios"][name][tag]) < 4e-6, (
            name,
            tag,
            D[name][tag],
            registered["ratios"][name][tag],
        )
control_stability = {}
for name, key in [("w1", "w1_cpu_ns_row"), ("w4", "w4_cpu_ns_row"), ("e4", "e4_ns")]:
    z = [per[(r, "P")][key] for r in range(R)]
    control_stability[name] = max(z) / min(z)
assert max(control_stability.values()) <= 1.05
assert max(x["weighted_skew"] for x in gate2) <= 0.02

medianmax = max(c["cpu_ns"][1] for batch in calibs for c in batch["cpus"])
shares = {}
for mode in ("w1", "w4"):
    for arm in "PC":
        sh = []
        shortsh = []
        for r in range(R):
            w = [v for v in walks if v["round"] == r and v["mode"] == mode and v["arm"] == arm]
            cw = sum(v["cpu_end"] - v["cpu_start"] for v in w)
            sh.append(len(w) * medianmax / cw)
            s = [v for v in w if v["rows"] < 899000]
            shortsh.append(len(s) * medianmax / sum(v["cpu_end"] - v["cpu_start"] for v in s))
        shares[mode + arm] = {"max": max(sh), "short_max": max(shortsh)}
assert max(s["max"] for s in shares.values()) <= 0.005

summaries = {}
for mode in ("w1", "w4"):
    for arm in "PC":
        values = [per[(r, arm)][mode + "_cpu_ns_row"] for r in range(R)]
        summaries[mode + arm] = {
            "cpu_ns_per_row_mean": statistics.mean(values),
            "min": min(values),
            "max": max(values),
            "sd": statistics.stdev(values),
            "running_fraction_mean": statistics.mean(
                per[(r, arm)][mode + "_running"] for r in range(R)
            ),
        }
for arm in "PC":
    values = [per[(r, arm)]["e4_ns"] for r in range(R)]
    summaries["e4" + arm] = {
        "elapsed_ms_mean": statistics.mean(values) / 1e6,
        "min": min(values) / 1e6,
        "max": max(values) / 1e6,
    }

# Detailed per-block W4 comparisons, 20 paired rounds, by category
blockby = {}
for mode in ("w1", "w4"):
    for arm in "PC":
        for b in range(n):
            vals = []
            for r in range(R):
                if mode == "w1":
                    val = observed[(r, mode, arm, 0, b)]
                else:
                    k, s = next((k, l.index(b)) for k, l in enumerate(assignment) if b in l)
                    val = observed[(r, mode, arm, k, s)]
                vals.append(val["cpu_end"] - val["cpu_start"])
            blockby[(mode, arm, b)] = statistics.mean(vals)

categories = {}
for mode in ("w1", "w4"):
    for sizecut in ("full", "short", "all"):
        bset = [
            b for b in range(n) if sizecut == "all" or (blocks[b] >= 899000) == (sizecut == "full")
        ]
        P = sum(blockby[(mode, "P", b)] for b in bset)
        C = sum(blockby[(mode, "C", b)] for b in bset)
        categories[mode + "_" + sizecut] = {
            "blocks": len(bset),
            "rows": sum(blocks[b] for b in bset),
            "P_cpu_ns": P,
            "C_cpu_ns": C,
            "ratio": C / P,
            "P_ns_row": P / sum(blocks[b] for b in bset),
            "C_ns_row": C / sum(blocks[b] for b in bset),
        }

# paired steps: compare summed worker CPU & makespan C/P for each step (averaged across 20 rounds)
stepdata = []
for s in range(nsteps):
    for r in range(R):
        pa = [observed[(r, "w4", "P", k, s)] for k in range(4) if s < len(assignment[k])]
        ca = [observed[(r, "w4", "C", k, s)] for k in range(4) if s < len(assignment[k])]
        cp = lambda v: sum(x["cpu_end"] - x["cpu_start"] for x in v)
        span = lambda v: max(x["end"] for x in v) - min(x["start"] for x in v)
        stepdata.append(
            {
                "s": s,
                "r": r,
                "rows": sum(x["rows"] for x in pa),
                "n": len(pa),
                "full": s in fullstepids,
                "cpu_ratio": cp(ca) / cp(pa),
                "completion_ratio": span(ca) / span(pa),
                "cpu_p": cp(pa),
                "cpu_c": cp(ca),
                "e_p": span(pa),
                "e_c": span(ca),
            }
        )
stepcats = {}
for tag, filter_ in [
    ("all", lambda x: True),
    ("all_full", lambda x: x["full"]),
    ("mixed", lambda x: x["s"] == 57),
    ("short", lambda x: x["s"] > 57),
]:
    v = [x for x in stepdata if filter_(x)]
    stepcats[tag] = {
        "n_steps": len(v),
        "cpu_ratio_mean": statistics.mean(x["cpu_ratio"] for x in v),
        "e_ratio_mean": statistics.mean(x["completion_ratio"] for x in v),
        "P_time_ms_mean_round": sum(x["e_p"] for x in v) / R / 1e6,
        "C_time_ms_mean_round": sum(x["e_c"] for x in v) / R / 1e6,
        "e_ratio_aggregate": sum(x["e_c"] for x in v) / sum(x["e_p"] for x in v),
        "n_cpu_wins": sum(x["cpu_ratio"] < 1 for x in v),
        "n_elapsed_wins": sum(x["completion_ratio"] < 1 for x in v),
        "max_cpu_ratio": max(x["cpu_ratio"] for x in v),
        "min_cpu_ratio": min(x["cpu_ratio"] for x in v),
    }

# Per-block ratios for full blocks & correlation to RAW density: exploratory only
fullb = [b for b in range(n) if blocks[b] >= 899000]
br = [blockby[("w4", "C", b)] / blockby[("w4", "P", b)] for b in fullb]
raw = [meta["block_raw_groups"][b] for b in fullb]
sp = spearmanr(raw, br)
blockind = {
    "full_block_n": len(fullb),
    "full_block_ratio_p10": sorted(br)[int(0.1 * (len(br) - 1))],
    "full_block_ratio_median": statistics.median(br),
    "full_block_ratio_p90": sorted(br)[int(0.9 * (len(br) - 1))],
    "full_block_ratio_max": max(br),
    "full_block_ratio_min": min(br),
    "n_full_blocks_candidate_faster": sum(x < 1 for x in br),
    "raw_correlation_spearman": float(sp.statistic),
    "raw_correlation_pvalue": float(sp.pvalue),
}

# Arm order effects on actual 20 paired ratios
orderdata = {}
for name, key in [("w1", "w1_cpu_ns_row"), ("w4", "w4_cpu_ns_row"), ("e4", "e4_ns")]:
    orderdata[name] = {
        "P_first": statistics.mean(
            per[(r, "C")][key] / per[(r, "P")][key] for r in range(R) if r % 2 == 0
        ),
        "C_first": statistics.mean(
            per[(r, "C")][key] / per[(r, "P")][key] for r in range(R) if r % 2 == 1
        ),
    }

bmax = lambda w: 1 + (1.847 * (1 - w) - 0.25) / 0.592
interpret = {
    "predict_S_at_b1": 1.847 * (1 - D["w4"]["point"]),
    "target_S": 0.25,
    "budget_point": bmax(D["w4"]["point"]),
    "budget_conservative": bmax(D["w4"]["hi"]),
    "budget_optimistic": bmax(D["w4"]["lo"]),
    "completion_throughput_gain_pct": 100 * (1 / D["e4"]["point"] - 1),
    "worker_rate_gain_pct": 100 * (1 / D["w4"]["point"] - 1),
    "single_core_rate_loss_pct": 100 * (1 - 1 / D["w1"]["point"]),
    "p_cpu_scaling": summaries["w4P"]["cpu_ns_per_row_mean"]
    / summaries["w1P"]["cpu_ns_per_row_mean"],
    "c_cpu_scaling": summaries["w4C"]["cpu_ns_per_row_mean"]
    / summaries["w1C"]["cpu_ns_per_row_mean"],
    "table_physical_shipped_mb": 4 * 900000 / 1e6,
    "table_candidate_wikidata_mb": 2.1642 * 900000 / 1e6,
}

output = {
    "verification": {
        "walk_count": len(walks),
        "calibration_count": len(calibs),
        "rows": total,
        "blocks": n,
        "full_blocks": len(fullb),
        "full_step_count": len(fullstepids),
        "total_step_count": nsteps,
        "control_stability": control_stability,
        "G2_max": max(x["weighted_skew"] for x in gate2),
        "G2_median": statistics.median(x["weighted_skew"] for x in gate2),
        "median_empty_cpu_ns": medianmax,
        "clock_shares": shares,
    },
    "ratios": D,
    "summaries": summaries,
    "categories": categories,
    "step_categories": stepcats,
    "block_indicators": blockind,
    "arm_order_ratios": orderdata,
    "planning": interpret,
}
path = pathlib.Path("/mnt/data/compact30_final_audit/independent_stats.json")
path.write_text(json.dumps(output, indent=2))
print("VERIFIED", json.dumps(output["verification"], indent=2))
print("SUMMARY", json.dumps(summaries, indent=2))
print("CATEGORIES", json.dumps(categories, indent=2))
print("STEP", json.dumps(stepcats, indent=2))
print("BLOCK", json.dumps(blockind, indent=2))
print("ORDER", json.dumps(orderdata, indent=2))
print("PLAN", json.dumps(interpret, indent=2))
print(
    "POINTS",
    {
        name: {k: v for k, v in d.items() if k in ("point", "lo", "hi", "min", "max")}
        for name, d in D.items()
    },
)
