"""The Cells memory-dose experiment's statistic and verdict (#145, LOG 6879466310, registered
before any data): does memory traffic cause the Workers + Cells plateau?

Three builds of `examples/ab.rs` with the research feature `dose`, at MIBZ2_DOSE_MB = 0, 8
and 16: each block Cells admits also streams through P MB of per-worker scratch (one read
and one write of every 64-byte line; src/dose.rs). `just bench-cells-dose` runs them as the
arms `mibz2-workers@<P=0's ab>`, `@<P=8's>` and `@<P=16's>` (forced Workers + Cells), on
Wikidata, paired by round. This reads its raw `runs.jsonl`:

    python3 bench/dose.py RUNS.jsonl --arms P0ARM,P8ARM,P16ARM [--threads 1,4,6,8,10]
        [--rounds 10] [--out STEM]

**Statistic.** c(t) = wall(P=16) / wall(P=0) at t threads, a paired log-t ratio within
round (the geometric mean over rounds and its 95% t-interval). The key ratio is
c(hi) / c(lo), hi and lo the most and fewest threads (10 and 1 on the registered grid),
each round's log c(hi) - log c(lo) its pair, with its 95% t-interval. The same for P=8,
reported and not gating.

**Rule** (the LOG's, on P=16's c(10)/c(1)):
- LEVER: the ratio >= 1.10 and its interval excludes 1;
- NOT THE LEVER: the ratio <= 1.03 and its interval inside [0.97, 1.06];
- INCONCLUSIVE otherwise.

**Validity** (no verdict, `INVALID`, without all of it): the complete roster (one run of
every arm at every thread count in every round, on the one corpus, and nothing else), no
duplicates, every child successful (termination success, no error, a record that parsed),
the same output hash (and size, and SHA-256 where recorded) across every run, and each
arm's build the dose it is named for (its records' `dose_mb`). Whether the dose engaged
(`dose_blocks`, the blocks Cells admitted and dosed in each run) is reported beside it.

A grid other than the registered one (threads 1,4,6,8,10, 10 rounds) still gets its
statistic and rule, its verdict line marked as not the registered grid (a dry run).
"""

import argparse
import json
import math
import statistics
import sys
from collections import defaultdict

# Student's t, 97.5% quantile, df 1-30
T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042]  # fmt: skip

DOSES = (0, 8, 16)
REGISTERED_THREADS = [1, 4, 6, 8, 10]
REGISTERED_ROUNDS = 10
CORPUS = "wikidata"
LEVER_AT = 1.10
NOT_LEVER_AT = 1.03
NOT_LEVER_BAND = (0.97, 1.06)


def mean_ci(logs):
    """exp of the mean of `logs` and of its 95% t-interval, (mean, low, high)"""
    m = statistics.fmean(logs)
    if len(logs) < 2:
        return math.exp(m), math.nan, math.nan
    h = T975[min(len(logs) - 1, 30) - 1] * statistics.stdev(logs) / math.sqrt(len(logs))
    return math.exp(m), math.exp(m - h), math.exp(m + h)


def verdict(ratio, lo, hi):
    """The LOG's rule on c(10)/c(1) and its interval"""
    if any(math.isnan(x) for x in (ratio, lo, hi)):
        return "INCONCLUSIVE"
    if ratio >= LEVER_AT and lo > 1.0:
        return "LEVER"
    if ratio <= NOT_LEVER_AT and NOT_LEVER_BAND[0] <= lo and hi <= NOT_LEVER_BAND[1]:
        return "NOT THE LEVER"
    return "INCONCLUSIVE"


def load(path):
    """The records, and the lines that did not parse"""
    recs, bad = [], []
    with open(path) as f:
        for i, line in enumerate(f, 1):
            if not line.strip():
                continue
            try:
                recs.append(json.loads(line))
            except json.JSONDecodeError:
                bad.append(i)
    return recs, bad


def validate(recs, bad, arms, threads, rounds):
    """The problems that leave no verdict (empty: valid), and the runs by (arm, t, round)"""
    problems = [f"line {i} is not JSON" for i in bad]
    cells = defaultdict(list)
    for r in recs:
        key = (r.get("arm"), r.get("threads"), r.get("round"))
        if (
            r.get("corpus") != CORPUS
            or key[0] not in arms
            or key[1] not in threads
            or not isinstance(key[2], int)
            or not 0 <= key[2] < rounds
        ):
            problems.append(f"an unexpected run: {r.get('corpus')} {key}")
            continue
        cells[key].append(r)
    for arm in arms:
        for t in threads:
            for rnd in range(rounds):
                n = len(cells.get((arm, t, rnd), []))
                if n == 0:
                    problems.append(f"missing: P={arms[arm]} t={t} round {rnd}")
                elif n > 1:
                    problems.append(f"duplicated ({n}): P={arms[arm]} t={t} round {rnd}")
    hashes = set()
    for (arm, t, rnd), rs in cells.items():
        for r in rs:
            where = f"P={arms[arm]} t={t} round {rnd}"
            if r.get("termination") != "success" or r.get("err") or "broken" in r:
                problems.append(
                    f"not successful: {where} ({r.get('termination')}, err {r.get('err')!r})"
                )
            if not isinstance(r.get("wall"), (int, float)) or not r["wall"] > 0:
                problems.append(f"no wall: {where}")
            if r.get("dose_mb") != arms[arm]:
                problems.append(f"the build's dose is {r.get('dose_mb')} MB: {where}")
            hashes.add((r.get("hash"), r.get("out"), r.get("sha256")))
    if len(hashes) > 1:
        problems.append(
            f"the output differs across runs: {len(hashes)} distinct (hash, size, sha256)"
        )
    elif hashes and next(iter(hashes))[0] is None:
        problems.append("no output hash recorded")
    return problems, {k: v[0] for k, v in cells.items() if len(v) == 1}


def analyse(runs, arms, threads, rounds):
    """c(t) for P=8 and P=16, c(hi)/c(lo) with its interval, and the rule's verdict on P=16's"""
    by_dose = {p: a for a, p in arms.items()}
    ref = by_dose[0]
    lo_t, hi_t = min(threads), max(threads)
    out = {"lo": lo_t, "hi": hi_t, "doses": {}}
    for p in (8, 16):
        arm = by_dose[p]
        logs = {
            t: [
                math.log(runs[(arm, t, r)]["wall"] / runs[(ref, t, r)]["wall"])
                for r in range(rounds)
            ]
            for t in threads
        }
        c = {t: mean_ci(logs[t]) for t in threads}
        cpu = {}
        for t in threads:
            pairs = [
                (runs[(arm, t, r)].get("cpu"), runs[(ref, t, r)].get("cpu")) for r in range(rounds)
            ]
            if all(
                isinstance(a, (int, float)) and isinstance(b, (int, float)) and a > 0 and b > 0
                for a, b in pairs
            ):
                cpu[t] = mean_ci([math.log(a / b) for a, b in pairs])
        key = mean_ci([logs[hi_t][r] - logs[lo_t][r] for r in range(rounds)])
        out["doses"][p] = {"c": c, "cpu": cpu, "key": key}
    out["verdict"] = verdict(*out["doses"][16]["key"])
    k8, k16 = out["doses"][8]["key"][0], out["doses"][16]["key"][0]
    out["monotone"] = (1.0 <= k8 <= k16) or (k16 <= k8 <= 1.0)
    return out


def fmt(x):
    m, lo, hi = x
    return f"{m:.4f} [{lo:.4f}, {hi:.4f}]"


def engagement(runs, arms, threads, rounds):
    """Per arm and thread count, the fewest and most blocks a run dosed"""
    eng = {}
    for arm, p in arms.items():
        for t in threads:
            b = [runs[(arm, t, r)].get("dose_blocks") for r in range(rounds) if (arm, t, r) in runs]
            b = [x for x in b if isinstance(x, int)]
            eng[(p, t)] = (min(b), max(b)) if b else (None, None)
    return eng


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument(
        "--arms",
        required=True,
        help="the arms at P=0, 8 and 16 MB, as in runs.jsonl, comma-separated",
    )
    ap.add_argument("--threads", default=",".join(map(str, REGISTERED_THREADS)))
    ap.add_argument("--rounds", type=int, default=REGISTERED_ROUNDS)
    ap.add_argument("--out", help="write STEM.md and STEM.json")
    a = ap.parse_args(argv)
    names = a.arms.split(",")
    if len(names) != 3 or len(set(names)) != 3:
        ap.error("--arms takes three distinct arms: P=0, P=8, P=16")
    arms = dict(zip(names, DOSES))
    threads = sorted({int(t) for t in a.threads.split(",")})
    if len(threads) < 2:
        ap.error("--threads takes at least two thread counts")
    registered = threads == REGISTERED_THREADS and a.rounds == REGISTERED_ROUNDS
    recs, bad = load(a.runs)
    problems, runs = validate(recs, bad, arms, threads, a.rounds)
    lines = [
        "# The Cells memory-dose experiment (#145, LOG 6879466310)",
        "",
        f"- runs: {a.runs} ({len(recs)} records)",
        f"- grid: threads {','.join(map(str, threads))}, {a.rounds} rounds, {CORPUS}"
        + ("" if registered else " (NOT the registered grid: threads 1,4,6,8,10, 10 rounds)"),
    ] + [f"- P={p} MB: `{arm}`" for arm, p in arms.items()]
    result = {
        "runs": a.runs,
        "threads": threads,
        "rounds": a.rounds,
        "registered": registered,
        "arms": {str(p): arm for arm, p in arms.items()},
        "problems": problems,
    }
    tag = "" if registered else " [not the registered grid: no verdict for the record]"
    if problems:
        lines += ["", f"## Validity: FAILED ({len(problems)})", ""]
        lines += [f"- {p}" for p in problems[:50]]
        if len(problems) > 50:
            lines.append(f"- ... and {len(problems) - 50} more")
        verdict_line = f"VERDICT: INVALID ({len(problems)} validity problems){tag}"
        result["verdict"] = "INVALID"
    else:
        r = analyse(runs, arms, threads, a.rounds)
        eng = engagement(runs, arms, threads, a.rounds)
        lo_t, hi_t = r["lo"], r["hi"]
        lines += [
            "",
            "## Validity: passed",
            "",
            "a complete roster, no duplicates, every child successful, one output hash, "
            "each build at its dose",
            "",
            "## Engagement: blocks Cells admitted and dosed a run (fewest..most)",
            "",
            "| P \\ t | " + " | ".join(map(str, threads)) + " |",
            "|---|" + "---|" * len(threads),
        ]
        for p in DOSES:
            lines.append(
                f"| {p} | "
                + " | ".join(f"{eng[(p, t)][0]}..{eng[(p, t)][1]}" for t in threads)
                + " |"
            )
        for p in (16, 8):
            d = r["doses"][p]
            lines += [
                "",
                f"## P={p} MB against P=0"
                + (" (gating)" if p == 16 else " (reported, not gating)"),
                "",
                "| t | c(t) = wall(P)/wall(0) [95% CI] | CPU(P)/CPU(0) [95% CI] |",
                "|---|---|---|",
            ]
            for t in threads:
                cpu = fmt(d["cpu"][t]) if t in d["cpu"] else "n/a"
                lines.append(f"| {t} | {fmt(d['c'][t])} | {cpu} |")
            lines += ["", f"c({hi_t})/c({lo_t}) = {fmt(d['key'])}"]
        lines += [
            "",
            f"dose response monotone (c({hi_t})/c({lo_t}) at P=8 between 1 and "
            f"P=16's): {'yes' if r['monotone'] else 'no'}",
        ]
        m, lo, hi = r["doses"][16]["key"]
        verdict_line = (
            f"VERDICT: {r['verdict']}: c({hi_t})/c({lo_t}) at P=16 = {m:.4f} "
            f"[{lo:.4f}, {hi:.4f}]{tag}"
        )
        result.update(
            verdict=r["verdict"],
            monotone=r["monotone"],
            engagement={f"{p}@{t}": v for (p, t), v in eng.items()},
            doses={
                str(p): {
                    "c": {str(t): v for t, v in d["c"].items()},
                    "cpu": {str(t): v for t, v in d["cpu"].items()},
                    "key": d["key"],
                }
                for p, d in r["doses"].items()
            },
        )
    lines += ["", "## Verdict", "", verdict_line]
    result["verdict_line"] = verdict_line
    text = "\n".join(lines) + "\n"
    print(text, end="")
    if a.out:
        with open(a.out + ".md", "w") as f:
            f.write(text)
        with open(a.out + ".json", "w") as f:
            json.dump(result, f, indent=1)
            f.write("\n")
    return 0 if not problems else 1


if __name__ == "__main__":
    sys.exit(main())
