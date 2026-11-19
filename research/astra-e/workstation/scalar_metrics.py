"""Independent statistical audit of the registered 48-round scalar i9 experiment.

Only the standard library and the bundle's raw rows enter the estimates. The
published decision is compared after recomputation. scalar-verify.py separately
authenticates the machine, binaries, input, dry gates and scorer projection.
"""
import argparse
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path, PurePosixPath
import statistics
import tarfile


ROUNDS = 48
WORKERS = (7, 8, 9, 10)
ARMS = ("lazy", "opend", "predecessor", "candidate")
BUILDS = ("base", "base", "frozen predecessor", "head")
EXPECTED_BYTES = 838_800_784


def require(value, message):
    if not value:
        raise ValueError(message)


def student_critical():
    """Invert the normalized cos^46 integral for Student t with 47 df.

    x = sqrt(47) tan(theta) transforms the t density into cos^46(theta)
    times a constant. Integration by parts evaluates its integral from 0.
    """
    def integral(theta):
        value = theta
        cosine, sine = math.cos(theta), math.sin(theta)
        for n in range(2, 47, 2):
            value = ((n - 1) * value + sine * cosine ** (n - 1)) / n
        return value

    lo, hi = 0.0, math.pi / 2
    goal = 0.95 * integral(hi)
    for _ in range(55):
        mid = (lo + hi) / 2
        if integral(mid) < goal:
            lo = mid
        else:
            hi = mid
    return math.sqrt(47) * math.tan((lo + hi) / 2)


T47 = student_critical()


def interval(logs):
    require(len(logs) == ROUNDS, "expected 48 matched round observations")
    mean = statistics.fmean(logs)
    radius = T47 * statistics.stdev(logs) / math.sqrt(ROUNDS)
    return {"ratio": math.exp(mean), "lo": math.exp(mean - radius), "hi": math.exp(mean + radius)}


def log_ratios(a, b):
    return [math.log(x / y) for x, y in zip(a, b, strict=True)]


def layout():
    # Eight-round blocks rotate by one slot and alternate direction after
    # four rotations. Index by round+workers, as specified by the recipe.
    for r in range(ROUNDS):
        for w in WORKERS:
            offset = (r + w) % 4
            order = ARMS[offset:] + ARMS[:offset]
            if (r + w) % 8 >= 4:
                order = order[::-1]
            for arm in order:
                yield r, w, arm


def by_cell(rows):
    require(len(rows) == ROUNDS * len(WORKERS) * len(ARMS), "wrong record count")
    result = {(w, arm): [] for w in WORKERS for arm in ARMS}
    for (r, w, arm), row in zip(layout(), rows, strict=True):
        require((row["round"], row["workers"], row["arm"]) == (r, w, arm), "unexpected roster")
        compiled = "candidate" if arm == "predecessor" else arm
        require((row["build"], row["compiled_arm"]) == (BUILDS[ARMS.index(arm)], compiled),
                "unexpected binary or compiled decoder arm")
        require(row["out_bytes"] == EXPECTED_BYTES, "wrong output byte count")
        for k in ("wall_s", "cpu_s", "peak_rss_kb"):
            x = row[k]
            require(type(x) in (int, float) and math.isfinite(x) and x > 0, f"invalid {k}")
        result[w, arm].append(row)
    return result


def compare(w, treatment, ref, cells):
    a, b = cells[w, treatment], cells[w, ref]
    result = dict(workers=w, treatment=treatment, reference=ref, n=ROUNDS, df=47)
    keys = ("wall_s", "cpu_s", "occupancy", "peak_rss_kb")
    for name in keys:
        if name == "occupancy":
            x, y = ([r["cpu_s"] / r["wall_s"] for r in side] for side in (a, b))
        else:
            x, y = ([r[name] for r in side] for side in (a, b))
        stat = interval(log_ratios(x, y))
        result.update({name + "_" + key: value for key, value in stat.items()})
        result[name + "_median"] = statistics.median(x)
        result[name + "_ref_median"] = statistics.median(y)
        result[name + "_lower_pairs"] = sum(i < j for i, j in zip(x, y))
        result[name + "_equal_pairs"] = sum(i == j for i, j in zip(x, y))
    # The registered sweep did not request perf. Retain coverage and totals by
    # arm if events were additionally recorded, including genuine zero counts.
    for event in sorted({key for row in a+b for key in row.get("counters", {})}):
        prefix = "perf_" + event
        matched = [(i["counters"][event], j["counters"][event])
                   for i, j in zip(a, b) if event in i.get("counters", {}) and event in j.get("counters", {})]
        result[prefix + "_paired_n"] = len(matched)
        for index, suffix in ((0, "_sum"), (1, "_ref_sum")):
            result[prefix + suffix] = sum(pair[index] for pair in matched)
        if len(matched) == ROUNDS and all(min(pair) > 0 for pair in matched):
            values = interval([math.log(i / j) for i, j in matched])
            result.update({prefix + "_" + k: v for k, v in values.items()})
    result["throughput_ratio"] = 1 / result["wall_s_ratio"]
    result["throughput_lo"] = 1 / result["wall_s_hi"]
    result["throughput_hi"] = 1 / result["wall_s_lo"]
    result["peak_rss_kb_max"] = max(r["peak_rss_kb"] for r in a)
    result["peak_rss_kb_ref_max"] = max(r["peak_rss_kb"] for r in b)
    result["median_rss_ratio"] = result["peak_rss_kb_median"] / result["peak_rss_kb_ref_median"]
    return result


def analyze(rows):
    cells = by_cell(rows)
    pairs = [compare(w, treatment, ref, cells) for w in WORKERS
             for ref, treatment in itertools.combinations(ARMS, 2)]
    worker_outcomes = {}
    for w in WORKERS:
        refs = {p["reference"]: p for p in pairs if p["workers"] == w and p["treatment"] == "candidate"}
        lazy, edge = refs["lazy"], refs["predecessor"]
        checks = dict(rss_upper=lazy["peak_rss_kb_hi"] <= 1,
                      rss_median=lazy["peak_rss_kb_median"] <= lazy["peak_rss_kb_ref_median"],
                      rss_max=lazy["peak_rss_kb_max"] <= lazy["peak_rss_kb_ref_max"])
        worker_outcomes[w] = dict(**checks, peak_pass=all(checks.values()),
                                  incremental_guard=edge["wall_s_hi"] <= 1.02,
                                  stretch_wall=lazy["wall_s_hi"] <= 0.80)
    paired_logs = [log_ratios([r["wall_s"] for r in cells[w, "candidate"]],
                              [r["wall_s"] for r in cells[w, "predecessor"]]) for w in WORKERS]
    pooled_logs = [statistics.fmean(round_logs) for round_logs in zip(*paired_logs, strict=True)]
    pooled = interval(pooled_logs)
    incremental = pooled["hi"] < 1 and all(c["incremental_guard"] and c["peak_pass"]
                                           for c in worker_outcomes.values())
    stretch = all(c["stretch_wall"] and c["peak_pass"] for c in worker_outcomes.values())
    return dict(rows=len(rows), n_rounds=ROUNDS, df=47, critical=T47,
                pooled=pooled, round_ratios=[math.exp(g) for g in pooled_logs],
                per_worker=worker_outcomes, incremental=incremental, stretch=stretch), pairs


def audit_decision(result, pairs, published):
    """Match the recorded decision only after independently estimating all quantities."""
    def near(x, y):
        require(math.isclose(x, y, abs_tol=2e-12, rel_tol=2e-12), f"decision mismatch: {x} vs {y}")

    def bounds(a, b, prefix=""):
        near(a[prefix+"ratio"], b["geomean"])
        near(a[prefix+"lo"], b["lo"])
        near(a[prefix+"hi"], b["hi"])

    primary = published["incremental_primary"]
    bounds(result["pooled"], primary["wall"])
    for x, y in zip(result["round_ratios"], primary["equal_weight_round_ratios"], strict=True):
        near(x, y)
    for pair in pairs:
        if pair["treatment"] != "candidate":
            continue
        cell = published["workers"][str(pair["workers"])][pair["reference"]]
        bounds(pair, cell["wall"], "wall_s_")
        bounds(pair, cell["rss"], "peak_rss_kb_")
        near(pair["median_rss_ratio"], cell["median_peak_ratio"])
        near(pair["peak_rss_kb_max"], cell["max_peak_kb"])
        near(pair["peak_rss_kb_ref_max"], cell["ref_max_peak_kb"])
    for w, flags in result["per_worker"].items():
        guard = primary["per_w_guard"][str(w)]
        near(guard["predecessor_wall_hi"], next(
            p["wall_s_hi"] for p in pairs if p["workers"] == w and
            p["treatment"] == "candidate" and p["reference"] == "predecessor"))
        require(guard["peak_vs_lazy_ok"] is flags["peak_pass"], "per-worker peak disagreement")
        require(published["workers"][str(w)]["target_met"] is
                (flags["stretch_wall"] and flags["peak_pass"]), "per-worker stretch disagreement")
    require(primary["criterion_met"] is result["incremental"], "incremental rule disagreement")
    require(published["i9_criterion_met"] is result["stretch"], "stretch rule disagreement")


def archive(path):
    with tarfile.open(path) as tf:
        def member(name):
            found = [m for m in tf if PurePosixPath(m.name).name == name and m.isfile()]
            require(len(found) == 1, f"expected one {name}")
            return tf.extractfile(found[0]).read()
        require(member("mode.txt").strip() == b"measure", "expected a timed archive")
        rows = [json.loads(line) for line in member("runs.jsonl").splitlines()]
        report, pairs = analyze(rows)
        audit_decision(report, pairs, json.loads(member("decision.json")))
        report["hardware_perf_events"] = sorted({key for r in rows for key in r.get("counters", {})})
    with path.open("rb") as f:
        report["archive_sha256"] = hashlib.file_digest(f, "sha256").hexdigest()
    report["official_decision_reconciled"] = True
    return report, pairs


def self_test():
    require(abs(T47 - 2.0117405137297655) < 1e-10, "df47 critical value")
    for v in (0.75, 1.0, 1.25):
        require(all(abs(x-v) < 1e-13 for x in interval([math.log(v)]*48).values()),
                "constant-ratio interval")
    rows = []
    wall = {"lazy": 1.0, "opend": .9, "predecessor": .8, "candidate": .75}
    for r, w, arm in layout():
        rows.append(dict(round=r, workers=w, arm=arm, out_bytes=EXPECTED_BYTES,
                         build=BUILDS[ARMS.index(arm)], compiled_arm="candidate" if arm=="predecessor" else arm,
                         wall_s=wall[arm], cpu_s=wall[arm]*w, peak_rss_kb=80 if arm=="candidate" else 100))

    def trial(change):
        return analyze([dict(row, **change(row)) for row in rows])[0]

    base = trial(lambda r: {})
    require(base["stretch"] and base["incremental"], "clean fixture")
    guard = trial(lambda r: dict(wall_s=.825) if r["workers"]==9 and r["arm"]=="candidate" else {})
    require(guard["pooled"]["hi"] < 1 and not guard["per_worker"][9]["incremental_guard"] and
            not guard["incremental"] and not guard["stretch"], "one unfavorable W")
    stretch_only = trial(lambda r: dict(wall_s=.805) if r["workers"]==9 and r["arm"]=="candidate" else {})
    require(stretch_only["incremental"] and not stretch_only["stretch"], "separate stretch threshold")
    # Keep the Lazy memory suite satisfied and all per-W guards true, while
    # making the equal-weight same-round incremental interval cross one.
    corr = trial(lambda r: dict(wall_s=.8 * (.965 if r["round"] % 2 else 1.024))
                 if r["arm"]=="candidate" else {})
    # Independently tabulated: treating 192 copies as independent would give
    # an upper bound of 0.998280990; 48 correlated four-W means give 1.002755447.
    require(abs(corr["pooled"]["hi"] - 1.002755447) < 1e-9 and not corr["incremental"] and
            all(x["incremental_guard"] for x in corr["per_worker"].values()), "round covariance")
    # Each row pair is (48 candidate peaks, 48 Lazy peaks). In each fixture
    # exactly one of the three registered independent memory tests fails.
    fixtures = {
        "rss_max": ([90]*47+[201], .906252992),
        "rss_median": ([80]*22+[101]*25+[190], .937697333),
        "rss_upper": ([100]*46+[110,200], 1.005998116),
    }
    l = [100]*47+[200]
    for key, (c, expected_upper) in fixtures.items():
        require(abs(interval(log_ratios(c, l))["hi"] - expected_upper) < 1e-9, f"known {key} bound")
        def change(r):
            if r["workers"] == 9 and r["arm"] in ("candidate", "lazy"):
                return dict(peak_rss_kb=(c if r["arm"]=="candidate" else l)[r["round"]])
            return {}
        result = trial(change)
        flags = result["per_worker"][9]
        require({k for k in fixtures if not flags[k]} == {key}, f"isolated {key} guard")
        require(not result["incremental"] and not result["stretch"], f"failed {key} must block both")
    for damaged in (rows[:-1], [rows[1], rows[0], *rows[2:]],
                    [dict(rows[0], out_bytes=1), *rows[1:]],
                    [dict(rows[0], compiled_arm="wrong"), *rows[1:]],
                    [dict(rows[0], cpu_s=float("nan")), *rows[1:]]):
        try:
            analyze(damaged)
        except ValueError:
            continue
        raise AssertionError("accepted corrupt rotor")
    print(f"48-round independent t(47) 0.975 = {T47:.14f}; favorable, adverse, "
          "three isolated RSS guards, correlated rounds and corrupted rotor fixtures passed.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path, nargs="?")
    parser.add_argument("--csv", type=Path, help="all six direct arm-pair comparisons at each W")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
    if args.archive:
        report, comparisons = archive(args.archive)
        if args.csv:
            with args.csv.open("w", newline="") as out:
                writer = csv.DictWriter(out, dict.fromkeys(k for p in comparisons for k in p), lineterminator="\n")
                writer.writeheader()
                writer.writerows(comparisons)
        else:
            report["comparisons"] = comparisons
        print(json.dumps(report, indent=2, allow_nan=False))
    elif not args.self_test or args.csv:
        parser.error("provide a timed archive or --self-test")


if __name__ == "__main__":
    main()
