"""The Cells buffer diet's scaling question (#145, LOG 6881022819, its verdict amended by LOG
6881074255, both before any data): does the 20-byte slot move the Workers + Cells plateau?

Two builds of `examples/ab.rs` from one commit, each with the research feature `dose` at
MIBZ2_DOSE_MB=0 (src/dose.rs: no dose, only its `dose_blocks` counter, the blocks Cells
admitted, which shows Cells engaged), in their own target directories:
- A20: the production layout, Cells' 20-byte slot;
- A24: the research feature `slot24` too, the parent's 24-byte slot.

`just bench-cells-diet` runs them as the arms `mibz2-workers@<each build's ab>` (forced
Workers + Cells) on Wikidata, paired by round, with --spread and the held spinner as
bench-cells-dose. This reads its raw `runs.jsonl`:

    python3 bench/cells_diet.py RUNS.jsonl --arms A20ARM,A24ARM [--threads 1,4,6,10]
        [--rounds 10] [--out STEM]

**Statistic.** A(t) = CPU(t) / CPU(1) for each build, and Q = A20(10) / A24(10); each
round's log Q its pair (log CPU20(10) - log CPU20(1) - log CPU24(10) + log CPU24(1)), the
geometric mean and its 95% t-interval over rounds.

**Verdict** (two-way and exhaustive, as amended by LOG 6881074255):
- MOVES THE PLATEAU: Q <= 0.95 and the interval's upper bound below 1;
- NO SCALING EFFECT: every other outcome. Where 0.95 < Q with the upper bound below 1, a
  real but smaller effect, the verdict line says so ("smaller effect, below threshold").

**Validity** (any failure: INVALID, no verdict): the complete roster (one run of each arm at
every thread count in every round, on the one corpus, nothing else, no duplicates), every
child successful, one output hash (and size, and SHA-256 where recorded) across every run,
and Cells engaged on all 196 blocks in both builds (`dose_blocks` = 196 and `dose_mb` = 0 in
every run).

**Reported, never gating:** the wall time of each build at every t, the paired wall ratio
20 B / 24 B at every t (4 threads, wikidata-pq's cap, called out), A(t) for each build, and
the paired CPU ratio at every t.

A grid other than the registered one (threads 1,4,6,10, 10 rounds) still gets its statistic
and rule, on the most threads against the fewest, its verdict line marked as not the
registered grid (a dry run).
"""

import argparse
import json
import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dose import load, mean_ci  # noqa: E402

BUILDS = ("A20", "A24")
REGISTERED_THREADS = [1, 4, 6, 10]
REGISTERED_ROUNDS = 10
CORPUS = "wikidata"
BLOCKS = 196
MOVES_AT = 0.95
CAP_T = 4


def verdict(q, lo, hi):
    """The amended rule on Q and its interval: (verdict, note)"""
    if any(math.isnan(x) for x in (q, lo, hi)):
        return "NO SCALING EFFECT", ""
    if q <= MOVES_AT and hi < 1.0:
        return "MOVES THE PLATEAU", ""
    if hi < 1.0:
        return "NO SCALING EFFECT", "smaller effect, below threshold"
    return "NO SCALING EFFECT", ""


def num(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and x > 0


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
                    problems.append(f"missing: {arms[arm]} t={t} round {rnd}")
                elif n > 1:
                    problems.append(f"duplicated ({n}): {arms[arm]} t={t} round {rnd}")
    outputs = set()
    for (arm, t, rnd), rs in cells.items():
        for r in rs:
            where = f"{arms[arm]} t={t} round {rnd}"
            if r.get("termination") != "success" or r.get("err") or "broken" in r:
                problems.append(
                    f"not successful: {where} ({r.get('termination')}, err {r.get('err')!r})"
                )
            if not num(r.get("wall")) or not num(r.get("cpu")):
                problems.append(f"no wall or CPU time: {where}")
            if r.get("dose_mb") != 0 or r.get("dose_blocks") != BLOCKS:
                problems.append(
                    f"Cells not engaged on all {BLOCKS} blocks (dose_blocks "
                    f"{r.get('dose_blocks')!r}, dose_mb {r.get('dose_mb')!r}): {where}"
                )
            outputs.add((r.get("hash"), r.get("out"), r.get("sha256")))
    if len(outputs) > 1:
        problems.append(
            f"the output differs across runs: {len(outputs)} distinct (hash, size, sha256)"
        )
    elif outputs and next(iter(outputs))[0] is None:
        problems.append("no output hash recorded")
    return problems, {k: v[0] for k, v in cells.items() if len(v) == 1}


def analyse(runs, arms, threads, rounds):
    by = {b: a for a, b in arms.items()}
    a20, a24 = by["A20"], by["A24"]
    lo_t, hi_t = min(threads), max(threads)

    def lg(arm, t, r, k):
        return math.log(runs[(arm, t, r)][k])

    def amp(arm, r):
        return lg(arm, hi_t, r, "cpu") - lg(arm, lo_t, r, "cpu")

    key = mean_ci([amp(a20, r) - amp(a24, r) for r in range(rounds)])
    v, note = verdict(*key)
    out = {"lo": lo_t, "hi": hi_t, "key": key, "verdict": v, "note": note, "t": {}}
    for t in threads:
        row = {
            "wall_ratio": mean_ci(
                [lg(a20, t, r, "wall") - lg(a24, t, r, "wall") for r in range(rounds)]
            ),
            "cpu_ratio": mean_ci(
                [lg(a20, t, r, "cpu") - lg(a24, t, r, "cpu") for r in range(rounds)]
            ),
        }
        for b, arm in (("A20", a20), ("A24", a24)):
            row[f"wall_{b}"] = mean_ci([lg(arm, t, r, "wall") for r in range(rounds)])
            row[f"amp_{b}"] = mean_ci(
                [lg(arm, t, r, "cpu") - lg(arm, lo_t, r, "cpu") for r in range(rounds)]
            )
        out["t"][t] = row
    return out


def fmt(x):
    m, lo, hi = x
    return f"{m:.4f} [{lo:.4f}, {hi:.4f}]"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--arms", required=True, help="the arms A20 and A24, as in runs.jsonl")
    ap.add_argument("--threads", default=",".join(map(str, REGISTERED_THREADS)))
    ap.add_argument("--rounds", type=int, default=REGISTERED_ROUNDS)
    ap.add_argument("--out", help="write STEM.md and STEM.json")
    a = ap.parse_args(argv)
    names = a.arms.split(",")
    if len(names) != 2 or len(set(names)) != 2:
        ap.error("--arms takes two distinct arms: A20, A24")
    arms = dict(zip(names, BUILDS))
    threads = sorted({int(t) for t in a.threads.split(",")})
    if len(threads) < 2:
        ap.error("--threads takes at least two thread counts")
    registered = threads == REGISTERED_THREADS and a.rounds == REGISTERED_ROUNDS
    recs, bad = load(a.runs)
    problems, runs = validate(recs, bad, arms, threads, a.rounds)
    lines = [
        "# The Cells buffer diet's scaling question (#145, LOG 6881022819, amended 6881074255)",
        "",
        f"- runs: {a.runs} ({len(recs)} records)",
        f"- grid: threads {','.join(map(str, threads))}, {a.rounds} rounds, {CORPUS}"
        + ("" if registered else " (NOT the registered grid: threads 1,4,6,10, 10 rounds)"),
    ] + [f"- {b} ({b[1:]} B slot): `{arm}`" for arm, b in arms.items()]
    result = {
        "runs": a.runs,
        "threads": threads,
        "rounds": a.rounds,
        "registered": registered,
        "arms": {b: arm for arm, b in arms.items()},
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
        lo_t, hi_t = r["lo"], r["hi"]
        lines += [
            "",
            "## Validity: passed",
            "",
            "a complete roster, no duplicates, every child successful, one output hash, "
            f"Cells engaged on all {BLOCKS} blocks in both builds (dose_blocks, dose 0 MB)",
            "",
            "## Reported, not gating",
            "",
            "| t | wall A20 s | wall A24 s | wall 20/24 [95% CI] | CPU 20/24 [95% CI] "
            f"| A20(t) = CPU(t)/CPU({lo_t}) | A24(t) |",
            "|---|---|---|---|---|---|---|",
        ]
        for t in threads:
            x = r["t"][t]
            lines.append(
                f"| {t}{' (cap)' if t == CAP_T else ''} | {fmt(x['wall_A20'])} "
                f"| {fmt(x['wall_A24'])} | {fmt(x['wall_ratio'])} | {fmt(x['cpu_ratio'])} "
                f"| {fmt(x['amp_A20'])} | {fmt(x['amp_A24'])} |"
            )
        if CAP_T in threads:
            lines += ["", f"4-thread wall ratio 20 B / 24 B: {fmt(r['t'][CAP_T]['wall_ratio'])}"]
        m, lo, hi = r["key"]
        note = f" ({r['note']})" if r["note"] else ""
        verdict_line = (
            f"VERDICT: {r['verdict']}: Q = A20({hi_t})/A24({hi_t}) = {m:.4f} "
            f"[{lo:.4f}, {hi:.4f}]{note}{tag}"
        )
        result.update(
            verdict=r["verdict"],
            note=r["note"],
            key=r["key"],
            hi=hi_t,
            lo=lo_t,
            t={str(t): v for t, v in r["t"].items()},
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
