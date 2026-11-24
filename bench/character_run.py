#!/usr/bin/env python3
"""bench-character's driver (stage 1; docs/journal/2026-11-24-character-stage1-registration.md):
every decode in a process of its own (examples/character.rs), in this order.

1. **The features** (untimed): one copy of the corpus at 1 worker, each block's route, n, r,
   bytes and h1 (`--features`).
2. **The grid, interleaved with the cost check:**
   - first, PREWARM seconds of decodes at 10 workers, discarded (the thermal pre-warm);
   - ROUNDS randomised complete rounds: in each, every count from 1 to 20 once, in an order
     drawn from SEED (never a rising sweep); then rounds to BAND_ROUNDS at 7 to 16 alone (the
     noisy band). An observation is REPS decodes at that count, back to back, each in its own
     process with the ledgers on, after SETTLE seconds of sleep. Each child runs in a fixed
     minimal environment with one argument layout (the count two digits wide), and its line
     records its position in the run and the count before it.
   - Every block's row is kept for the first decode of each observation in the first
     PER_BLOCK_ROUNDS rounds (`blocks-rNN.tsv`).
   - After every second round, one pair of the instrumentation's cost check: at 1, 10 and 20
     workers in a drawn order, REPS decodes with the ledgers off and REPS on, the arm first
     alternating by pair.
3. **The counters:** at the anchors 1, 8, 10, 16 and 20, groups 1 and 2, PMU_REPEATS times
   each, in a drawn order, the ledgers off. Where the first reads nothing (no hardware PMU),
   the rest are skipped and that is recorded.
4. **The memory controller:** where `perf stat -a` can count `uncore_imc/data_reads/` and
   `data_writes/`, each anchor PMU_REPEATS times around the whole decode's process.
5. **The public API:** API_ROUNDS rounds, every requested count from 1 to 20 once in a drawn
   order, `BzReader::parallel` as shipped, one decode each.

Every child's line goes to runs.jsonl with this driver's fields (section, round, order, rep,
pair, arm). A child that fails is recorded as `{"kind": "failed", ...}` and the run goes on.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile
import time

GRID = list(range(1, 21))
# the noisy band (the review decisions §1.10): 16 rounds there, 12 elsewhere
BAND = list(range(7, 17))
COST_COUNTS = [1, 10, 20]
ANCHORS = [1, 8, 10, 16, 20]
# the children's environment: fixed and minimal (the decisions §1.7), whatever the shell's
ENV = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"}
BLOCK_COLS = (
    "place\troute\tn\tr\tsyms_bytes\tout_bytes\tworker\tcpu_scan\tcpu_work\tt_issued\tt_enqueued"
    "\tt_got\tt_ready\tt_taken\tt_scan_start\tt_consumed\tq_issued\tq_enqueued\tq_got\tq_ready"
    "\tq_taken\tq_scan_start\tq_consumed\ts_in_wait\ts_in_maint\ts_A2\ts_A3\ts_slot_wait\ts_issue\ts_resid\tw_A5\tw_pool"
    "\tw_A6a\tw_A6b\tw_A6c\tw_A7\tw_A8\tw_A9\tw_publish\tw_resid\tr_wait"
    "\tv_in_wait\tv_in_maint\tv_A2\tv_A3\tv_slot_wait\tv_r_wait"
)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", required=True)
    ap.add_argument("--corpus", required=True, help="the timed corpus (4 copies)")
    ap.add_argument("--single", required=True, help="one copy, for the features")
    ap.add_argument("--out", required=True)
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--band-rounds", type=int, default=16, help="the rounds at 7-16")
    ap.add_argument(
        "--prewarm", type=float, default=50.0, help="seconds of discarded decodes first"
    )
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--settle", type=float, default=1.0)
    ap.add_argument("--seed", type=int, default=20261124)
    ap.add_argument("--per-block-rounds", type=int, default=4)
    ap.add_argument("--cost-every", type=int, default=2)
    ap.add_argument("--pmu-repeats", type=int, default=3)
    ap.add_argument("--api-rounds", type=int, default=3)
    a = ap.parse_args()
    out = a.out
    runs = open(os.path.join(out, "runs.jsonl"), "a")
    t0 = time.time()

    position = [0]
    prev_k = [0]

    def child(args: list[str], extra: dict, wrap: list[str] | None = None) -> dict | None:
        cmd = (wrap or []) + [a.bin] + args
        p = subprocess.run(cmd, capture_output=True, text=True, env=ENV)
        if extra.get("section") != "prewarm":
            k = int(args[2]) if args[0] in ("--one", "--api") else 0
            extra = dict(extra, position=position[0], prev_k=prev_k[0])
            position[0] += 1
            prev_k[0] = k
        if p.returncode != 0 or not p.stdout.strip():
            rec = {
                "kind": "failed",
                "cmd": args,
                "status": p.returncode,
                "stderr": p.stderr[-2000:],
            }
            rec.update(extra)
            runs.write(json.dumps(rec) + "\n")
            runs.flush()
            print(f"FAILED {extra}: {p.stderr[-300:]}", file=sys.stderr)
            return None
        rec = json.loads(p.stdout.strip().splitlines()[-1])
        rec.update(extra)
        runs.write(json.dumps(rec, separators=(",", ":")) + "\n")
        runs.flush()
        return rec

    # 1. the features
    feat = os.path.join(out, "features.tsv")
    subprocess.run([a.bin, "--features", a.single, feat], check=True)
    print(f"features: {sum(1 for _ in open(feat)) - 1} blocks", flush=True)

    # 2. the grid and the cost check
    tmp = tempfile.mkdtemp(prefix="character-")
    tb = os.path.join(tmp, "b.tsv")

    def one(k: int, on: int, pmu: int = 0) -> list[str]:
        # (one layout for every decode: the count two digits wide, the same block file, the
        # treatment named, so no argument's length changes with the count or the arm)
        if os.path.exists(tb):
            os.remove(tb)
        return ["--one", a.corpus, f"{k:02d}", str(on), tb, str(pmu), "none"]

    # the thermal pre-warm, discarded (the decisions §1.7): decodes at 10 workers
    t_warm = time.time()
    while time.time() - t_warm < a.prewarm:
        subprocess.run([a.bin] + one(10, 0), capture_output=True, env=ENV)
    print(f"pre-warm: {time.time() - t_warm:.0f} s, discarded", flush=True)
    pair = 0
    for r in range(max(a.rounds, a.band_rounds)):
        order = (GRID if r < a.rounds else BAND)[:]
        random.Random(a.seed * 1000 + r).shuffle(order)
        bpath = os.path.join(out, f"blocks-r{r:02d}.tsv")
        if r < a.per_block_rounds:
            with open(bpath, "w") as f:
                f.write("k\t" + BLOCK_COLS + "\n")
        for o, k in enumerate(order):
            time.sleep(a.settle)
            for rep in range(a.reps):
                keep = r < a.per_block_rounds and rep == 0
                rec = child(
                    one(k, 1),
                    {"section": "grid", "round": r, "order": o, "rep": rep},
                )
                if keep and rec is not None and os.path.exists(tb):
                    with open(bpath, "a") as f, open(tb) as g:
                        for line in g:
                            f.write(f"{k}\t{line}")
            print(
                f"grid round {r} count {k} ({o + 1}/{len(order)}), {time.time() - t0:.0f} s",
                flush=True,
            )
        if r < a.rounds and (r + 1) % a.cost_every == 0:
            counts = COST_COUNTS[:]
            random.Random(a.seed * 1000 + 500 + pair).shuffle(counts)
            arms = [1, 0] if pair % 2 == 0 else [0, 1]
            for k in counts:
                for arm in arms:
                    time.sleep(a.settle)
                    for rep in range(a.reps):
                        child(
                            one(k, arm),
                            {"section": "cost", "pair": pair, "round": r, "rep": rep, "arm": arm},
                        )
            print(f"cost pair {pair}, {time.time() - t0:.0f} s", flush=True)
            pair += 1

    # 3. the counters
    cells = [(k, g, i) for k in ANCHORS for g in (1, 2) for i in range(a.pmu_repeats)]
    random.Random(a.seed * 1000 + 900).shuffle(cells)
    pmu_ok = None
    for n, (k, g, i) in enumerate(cells):
        time.sleep(a.settle)
        rec = child(one(k, 0, g), {"section": "pmu", "repeat": i, "order": n})
        if pmu_ok is None and rec is not None:
            pmu_ok = any(v is not None for v in rec.get("pmu", {}).values())
            if not pmu_ok:
                runs.write(
                    json.dumps(
                        {
                            "kind": "note",
                            "section": "pmu",
                            "note": "no hardware counters: the counters' runs skipped",
                        }
                    )
                    + "\n"
                )
                print("counters: none here, skipped", flush=True)
                break
    print(f"counters done, {time.time() - t0:.0f} s", flush=True)

    # 4. the memory controller
    imc = ["-e", "uncore_imc/data_reads/", "-e", "uncore_imc/data_writes/"]
    probe = (
        subprocess.run(
            ["perf", "stat", "-a", "-x", ",", *imc, "--", "true"], capture_output=True, text=True
        )
        if shutil.which("perf")
        else None
    )
    imc_ok = probe is not None and probe.returncode == 0 and "<not" not in probe.stderr
    if not imc_ok:
        runs.write(
            json.dumps(
                {
                    "kind": "note",
                    "section": "imc",
                    "note": "perf stat -a cannot count the memory controller here: skipped",
                    "probe": (probe.stderr[-500:] if probe else "no perf"),
                }
            )
            + "\n"
        )
        print("memory controller: cannot count here, skipped", flush=True)
    else:
        cells = [(k, i) for k in ANCHORS for i in range(a.pmu_repeats)]
        random.Random(a.seed * 1000 + 950).shuffle(cells)
        for n, (k, i) in enumerate(cells):
            time.sleep(a.settle)
            fd, path = tempfile.mkstemp(prefix="imc-")
            os.close(fd)
            rec = child(
                one(k, 0),
                {"section": "imc", "repeat": i, "order": n},
                wrap=["perf", "stat", "-a", "-x", ",", "-o", path, *imc, "--"],
            )
            counts = {}
            for line in open(path):
                f = line.strip().split(",")
                if len(f) >= 3 and f[0] and f[0][0].isdigit():
                    counts[f[2]] = [float(f[0]), f[1]] + f[3:5]
            os.remove(path)
            runs.write(
                json.dumps(
                    {
                        "kind": "imc",
                        "k": k,
                        "repeat": i,
                        "order": n,
                        "counts": counts,
                        "wall_s": rec and rec.get("wall_s"),
                    }
                )
                + "\n"
            )
            runs.flush()
    print(f"memory controller done, {time.time() - t0:.0f} s", flush=True)

    # 5. the public API
    for r in range(a.api_rounds):
        order = GRID[:]
        random.Random(a.seed * 1000 + 990 + r).shuffle(order)
        for o, t in enumerate(order):
            time.sleep(a.settle)
            child(["--api", a.corpus, f"{t:02d}"], {"section": "api", "round": r, "order": o})
    print(f"public API done, {time.time() - t0:.0f} s", flush=True)
    shutil.rmtree(tmp, ignore_errors=True)
    runs.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
