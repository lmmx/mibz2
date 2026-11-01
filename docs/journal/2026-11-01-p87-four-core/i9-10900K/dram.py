"""S3's DRAM figures in the units they were meant to have (bytes per decoded byte).

bench/p87.py `counters` (frozen at 6463e44) multiplied perf's uncore_imc values by 64, as if
they were cache-line counts; perf reports them already scaled, in MiB (the CSVs' unit column,
counters/*.csv). Its counters.json DRAM figures are therefore 64 / 2^20 times the true bytes per
decoded byte. This recomputes them from the raw CSVs, idle-subtracted exactly as the frozen
analysis does, with the same paired interval over repetitions. The instruction and cycle figures
(no unit scaling) and both registered decisions are unaffected.

    python3 dram.py   (from this directory) > dram.json
"""
import json
import math
import statistics


def t975(df):
    # the two-sided 95% t quantiles needed here (df = reps - 1)
    return {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365}[df]


def val(path, ev):
    for line in open(path):
        f = line.strip().split(",")
        if len(f) > 2 and f[2] == ev:
            assert f[1] == "MiB", f"{path}: unit {f[1]!r}"
            return float(f[0])
    raise KeyError(ev)


EVENTS = ["uncore_imc/data_reads/", "uncore_imc/data_writes/"]
runs = [json.loads(l) for l in open("counter-runs.jsonl") if l.strip()]
per = {}
for r in runs:
    if r["pass"] != "imc" or r["rc"] != 0:
        continue
    scale = r["wall_s"] / r["idle_wall_s"]
    per.setdefault((r["w"], r["arm"]), {})[r["rep"]] = {
        ev: (val(f"counters/{r['csv']}", ev) - val(f"counters/{r['idle_csv']}", ev) * scale) * 2**20 / r["out"]
        for ev in EVENTS}
out = {}
for w in (1, 4):
    reps = sorted(set(per[w, "A"]) & set(per[w, "B"]))
    row = {"reps": len(reps)}
    for ev in EVENTS:
        a = [per[w, "A"][k][ev] for k in reps]
        b = [per[w, "B"][k][ev] for k in reps]
        d = [y - x for x, y in zip(a, b)]
        m, se = statistics.mean(d), statistics.stdev(d) / math.sqrt(len(d))
        h = t975(len(d) - 1) * se
        row[ev] = dict(A=statistics.mean(a), B=statistics.mean(b), B_minus_A=dict(mean=m, lower95=m - h, upper95=m + h),
                       B_over_A=statistics.mean(b) / statistics.mean(a))
    out[f"w={w}"] = row
print(json.dumps(dict(units="bytes moved to or from DRAM per decoded byte, idle-subtracted", result=out), indent=1))
