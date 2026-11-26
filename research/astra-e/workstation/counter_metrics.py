"""Independent five-arm target replay. Reads original receipts, retaining the round as the sampling unit."""
import argparse
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path, PurePosixPath
import statistics
import tarfile

from scalar_metrics import EXPECTED_BYTES, WORKERS, compare, interval, require, student_critical

ARMS = ("lazy", "opend", "edge", "scalar", "both")
REFS = ("edge", "scalar")
ROUNDS = 40
T39 = student_critical(ROUNDS - 1)


def layout():
    for r in range(ROUNDS):
        for w in WORKERS:
            reverse, offset = divmod(r + w, len(ARMS))
            labels = ARMS[offset:] + ARMS[:offset]
            for arm in reversed(labels) if reverse % 2 else labels:
                yield r, w, arm


def analyze(rows):
    require(len(rows) == 800, "expected 800 process receipts")
    cells = {(w, arm): [] for w in WORKERS for arm in ARMS}
    for (r, w, arm), row in zip(layout(), rows, strict=True):
        require((row["round"], row["workers"], row["arm"]) == (r, w, arm), "incorrect rotor")
        build, compiled = ("edge", arm) if arm in ("lazy", "opend") else (arm, "candidate")
        require((row["build"], row["compiled_arm"]) == (build, compiled), "wrong build/decoder")
        require(row["out_bytes"] == EXPECTED_BYTES, "wrong output length")
        require(all(type(row[k]) in (float, int) and math.isfinite(row[k]) and row[k] > 0
                    for k in ("wall_s", "cpu_s", "peak_rss_kb")), "invalid measured quantity")
        cells[w, arm].append(row)
    pairs = [compare(w, a, b, cells, ROUNDS, T39)
             for w in WORKERS for b, a in itertools.combinations(ARMS, 2)]
    results = {(p["workers"], p["reference"]): p for p in pairs if p["treatment"] == "both"}
    memory = {}
    target = {}
    for w in WORKERS:
        p = results[w, "lazy"]
        memory[w] = dict(paired_upper=p["peak_rss_kb_hi"] <= 1, median=p["median_rss_ratio"] <= 1,
                         maximum=p["peak_rss_kb_max"] <= p["peak_rss_kb_ref_max"])
        target[w] = p["wall_s_hi"] <= .80 and all(memory[w].values())
    direct = {}
    for ref in REFS:
        logs = [statistics.fmean(math.log(cells[w, "both"][r]["wall_s"]/cells[w, ref][r]["wall_s"])
                                 for w in WORKERS) for r in range(ROUNDS)]
        stat = interval(logs, ROUNDS, T39)
        guards = {w: results[w, ref]["wall_s_hi"] <= 1.02 for w in WORKERS}
        direct[ref] = dict(wall=stat, round_ratios=list(map(math.exp, logs)), guards=guards,
                           wall_screen_met=stat["hi"] < 1 and all(guards.values()))
    return dict(samples=len(rows), rounds=ROUNDS, df=39, critical=T39, direct=direct, memory=memory,
                target=target, stretch=all(target.values()),
                incremental=all(d["wall_screen_met"] for d in direct.values()) and
                all(all(m.values()) for m in memory.values())), pairs


def reconcile(decision, pairs, official):
    def near(x, y):
        require(math.isclose(x, y, abs_tol=2e-12, rel_tol=2e-12), f"estimates disagree {x} {y}")

    def bounds(source, published, prefix=""):
        require(published["n"] == ROUNDS, "incorrect sample size")
        for a, b in (("ratio", "geomean"), ("lo", "lo"), ("hi", "hi")):
            near(source[prefix + a], published[b])

    for ref, d in decision["direct"].items():
        out = official["direct"][ref]
        bounds(d["wall"], out["wall"])
        require(d["wall_screen_met"] == out["wall_screen_met"], "pooled wall disagreement")
        for a, b in zip(d["round_ratios"], out["equal_weight_round_ratios"], strict=True):
            near(a, b)
        require(all(out["per_worker_guard"][str(w)] == g for w, g in d["guards"].items()), "worker guard")
    for p in pairs:
        if p["treatment"] != "both":
            continue
        w = str(p["workers"])
        out = official["workers"][w][p["reference"]]
        bounds(p, out["wall"], "wall_s_")
        bounds(p, out["rss"], "peak_rss_kb_")
        near(p["median_rss_ratio"], out["median_peak_ratio"])
        near(p["peak_rss_kb_max"], out["max_peak_kb"])
        near(p["peak_rss_kb_ref_max"], out["ref_max_peak_kb"])
    for w in WORKERS:
        require(decision["memory"][w] == official["peak_vs_lazy"][str(w)], "RSS disagreement")
        require(decision["target"][w] == official["workers"][str(w)]["target_met"], "stretch worker")
    require(official["joint_incremental_met"] == decision["incremental"], "incremental decision")
    require(official["i9_criterion_met"] == decision["stretch"], "stretch decision")


def self_test():
    require(abs(T39 - 2.02269092003676) < 1e-11, "t39 quantile")
    rows = []
    walls = dict(lazy=1., opend=.9, edge=.84, scalar=.82, both=.78)
    for r, w, a in layout():
        b, c = ("edge", a) if a in ("lazy", "opend") else (a, "candidate")
        rows.append(dict(round=r, workers=w, arm=a, wall_s=walls[a], cpu_s=w*walls[a],
                         peak_rss_kb=80 if a=="both" else 100, out_bytes=EXPECTED_BYTES, build=b, compiled_arm=c))
    def trial(change):
        return analyze([dict(row, **change(row)) for row in rows])[0]
    clean = trial(lambda row: {})
    require(clean["incremental"] and clean["stretch"], "favorable controls")
    for ref in REFS:
        bad = trial(lambda row: dict(wall_s=.779) if row["arm"]==ref else {})
        require(not bad["incremental"] and bad["stretch"] and
                not bad["direct"][ref]["wall_screen_met"] and all(bad["direct"][ref]["guards"].values()),
                "one injured comparator")
    guard = trial(lambda row: dict(wall_s=.838) if row["workers"]==7 and row["arm"]=="both" else {})
    require(not guard["incremental"] and not guard["direct"]["scalar"]["guards"][7] and
            guard["direct"]["scalar"]["wall"]["hi"]<1, "single worker guard")
    peak = trial(lambda row: dict(peak_rss_kb=101) if row["workers"]==8 and row["arm"]=="both" and row["round"]==0 else {})
    require(peak["memory"][8]==dict(paired_upper=True, median=True, maximum=False)
            and not peak["incremental"] and not peak["stretch"], "single maximum")
    correlated = trial(lambda row: dict(wall_s=.82*(.96 if row["round"]%2 else 1.028))
                       if row["arm"]=="both" else {})
    d = correlated["direct"]["scalar"]
    require(d["wall"]["hi"] > 1 and all(d["guards"].values()), "round correlation")
    # Treating the four identical worker measurements within each round as independent reverses this decision.
    duplicate_logs = [math.log(x) for x in d["round_ratios"] for _ in WORKERS]
    require(interval(duplicate_logs, 160, student_critical(159))["hi"] < 1, "pseudoreplication test")
    for bad in (rows[:-1], [rows[1],rows[0],*rows[2:]], [dict(rows[0],cpu_s=math.nan),*rows[1:]]):
        try:
            analyze(bad)
        except ValueError:
            continue
        raise ValueError("accepted damaged record set")
    print("40 paired rounds, 39 df, both pooled tests, individual worker and peak guards, correlated rounds and rotor verified")


def archive(path):
    with tarfile.open(path) as bundle:
        def read(name):
            found = [m for m in bundle if m.isfile() and PurePosixPath(m.name).name == name]
            require(len(found)==1, f"missing or duplicated {name}")
            return bundle.extractfile(found[0]).read()
        require(read("mode.txt").strip()==b"measure", "requires timed measurement")
        rows = list(map(json.loads, read("runs.jsonl").splitlines()))
        decision, pairs = analyze(rows)
        reconcile(decision, pairs, json.loads(read("decision.json")))
        decision["hardware_perf_events"] = sorted({k for r in rows for k in r.get("counters", {})})
    with path.open("rb") as original:
        decision["archive_sha256"] = hashlib.file_digest(original, "sha256").hexdigest()
    decision["official_decision_reconciled"] = True
    return decision, pairs


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("bundle", nargs="?", type=Path)
    parser.add_argument("--csv", type=Path)
    args = parser.parse_args()
    if args.self_test:
        self_test()
    if args.bundle:
        decision, pairs = archive(args.bundle)
        if args.csv:
            with args.csv.open("w", newline="") as output:
                writer = csv.DictWriter(output, dict.fromkeys(k for p in pairs for k in p), lineterminator="\n")
                writer.writeheader()
                writer.writerows(pairs)
        else:
            decision["pairs"] = pairs
        print(json.dumps(decision, allow_nan=False, indent=2))
    elif not args.self_test or args.csv:
        parser.error("provide a timed archive or --self-test")
