"""The bandwidth titration's figures and reading (docs/journal/2026-11-10-bandwidth-titration.md):
from `bw_titrate`'s JSON lines, the decoder's rate at each thread count against the memory
traffic the hogs reached, and the registered reading at the plan's thread count.

    python3 bench/bw_titrate.py runs.jsonl ROUNDS THREADS LEVELS HOGS REPORT.json > summary.md

Exits 1, naming the fault, unless runs.jsonl is exactly the registered grid: every (round,
threads, level) once, threads 0 (the hogs alone) included; whole, finite, positive times; one
decoded size in every decode cell; no hog bytes at level 0, some at every other level.
"""

import json
import math
import statistics
import sys

KEYS = {"round", "threads", "level", "hogs", "wall_ns", "bytes", "hog_bytes"}
# the registered reading (the journal entry's section 4), at the plan's thread count
PLAN, CONTROL = 10, 1
BOUND, FREE, CONTROL_FLOOR, POWER = 0.80, 0.92, 0.90, 8.0


def die(msg: str) -> None:
    sys.exit(f"bw_titrate: {msg}")


def whole(v) -> bool:
    return isinstance(v, int) and not isinstance(v, bool)


def load(path: str, rounds: int, threads: list[int], levels: list[int], hogs: int) -> dict:
    cells: dict = {}
    for n, line in enumerate(open(path), 1):
        try:
            r = json.loads(line)
        except json.JSONDecodeError as e:
            die(f"line {n} is not JSON ({e})")
        if not isinstance(r, dict) or set(r) != KEYS:
            die(f"line {n} does not have exactly the fields {sorted(KEYS)}")
        if not all(whole(r[k]) for k in KEYS):
            die(f"line {n} has a field that is not a whole number")
        key = (r["round"], r["threads"], r["level"])
        if key in cells:
            die(f"{key} appears twice")
        if r["hogs"] != hogs:
            die(f"{key} ran {r['hogs']} hogs, not {hogs}")
        if r["wall_ns"] <= 0:
            die(f"{key} has no time")
        if (r["bytes"] > 0) != (r["threads"] > 0):
            die(f"{key} has {r['bytes']} decoded bytes at {r['threads']} threads")
        if r["hog_bytes"] < 0 or (r["hog_bytes"] > 0) != (r["level"] > 0):
            die(f"{key} has {r['hog_bytes']} hog bytes at level {r['level']}")
        cells[key] = r
    want = {(i, t, lv) for i in range(rounds) for t in [0, *threads] for lv in levels}
    if set(cells) != want:
        missing, extra = sorted(want - set(cells)), sorted(set(cells) - want)
        die(f"not the registered grid: missing {missing[:4]}, unexpected {extra[:4]}")
    sizes = {r["bytes"] for r in cells.values() if r["threads"] > 0}
    if len(sizes) != 1:
        die(f"the decode cells decoded {sorted(sizes)} bytes, not one size")
    return cells


def rate(r: dict) -> float:
    """Decoded MB/s."""
    return r["bytes"] / r["wall_ns"] * 1e3


def hog(r: dict) -> float:
    """The hogs' GB/s."""
    return r["hog_bytes"] / r["wall_ns"]


def report(cells: dict, rounds: int, threads: list[int], levels: list[int]) -> tuple[str, dict]:
    base, top = min(levels), max(levels)
    if base != 0:
        die("the levels must include 0")
    out = [
        "| threads | level | hogs' GB/s | decoded MB/s | against level 0 (paired) | bytes of hog traffic a decoded byte lost |",
        "|--:|--:|--:|--:|--:|--:|",
    ]
    ratios: dict = {}
    for t in [0, *threads]:
        for lv in sorted(levels):
            rows = [cells[(i, t, lv)] for i in range(rounds)]
            h = statistics.median(hog(r) for r in rows)
            if t == 0:
                out.append(f"| hogs alone | {lv} | {h:.2f} | | | |")
                continue
            d = statistics.median(rate(r) for r in rows)
            paired = [rate(cells[(i, t, lv)]) / rate(cells[(i, t, 0)]) for i in range(rounds)]
            ratio = statistics.median(paired)
            ratios[(t, lv)] = (ratio, min(paired), max(paired), h)
            # beta: the hog traffic that costs one decoded byte, where the decoder lost any
            lost = [
                (hog(cells[(i, t, lv)]) * 1e3) / (rate(cells[(i, t, 0)]) - rate(cells[(i, t, lv)]))
                for i in range(rounds)
                if lv > 0 and rate(cells[(i, t, 0)]) > rate(cells[(i, t, lv)])
            ]
            beta = f"{statistics.median(lost):.1f}" if len(lost) == rounds else ""
            out.append(f"| {t} | {lv} | {h:.2f} | {d:.0f} | {ratio:.3f} | {beta} |")
    verdict: dict = {"plan_threads": PLAN, "control_threads": CONTROL, "top_level": top}
    if (PLAN, top) in ratios and (CONTROL, top) in ratios:
        r, lo, hi, h = ratios[(PLAN, top)]
        c = ratios[(CONTROL, top)][0]
        verdict.update(plan_ratio=r, plan_ratio_range=[lo, hi], plan_hog_gbs=h, control_ratio=c)
        if not all(math.isfinite(x) for x in (r, lo, hi, h, c)):
            die("a ratio is not finite")
        if h < POWER:
            reading = (
                f"UNDERPOWERED: the hogs reached {h:.1f} GB/s beside the decoder, under {POWER}"
            )
        elif c < CONTROL_FLOOR:
            reading = f"CONFOUNDED: the one-thread control fell to {c:.3f}, under {CONTROL_FLOOR}"
        elif r <= BOUND:
            reading = (
                f"BOUND: at {PLAN} threads the decoder kept {r:.3f} of its rate, at most {BOUND}"
            )
        elif r >= FREE:
            reading = f"NOT BOUND: at {PLAN} threads the decoder kept {r:.3f} of its rate, at least {FREE}"
        else:
            reading = f"MIXED: at {PLAN} threads the decoder kept {r:.3f} of its rate, between {BOUND} and {FREE}"
    else:
        reading = f"NO READING: the grid lacks {PLAN} or {CONTROL} threads"
    verdict["reading"] = reading
    out += [
        "",
        f"Medians over {rounds} rounds. The last column is the hogs' traffic over the decoder's "
        "loss, round by round, given where every round lost.",
        "",
        f"**Reading: {reading}**",
    ]
    return "\n".join(out), verdict


if __name__ == "__main__":
    path, rounds = sys.argv[1], int(sys.argv[2])
    threads = [int(x) for x in sys.argv[3].split(",")]
    levels = [int(x) for x in sys.argv[4].split(",")]
    text, verdict = report(
        load(path, rounds, threads, levels, int(sys.argv[5])), rounds, threads, levels
    )
    with open(sys.argv[6], "w") as f:
        json.dump(verdict, f, indent=1)
    print(text)
