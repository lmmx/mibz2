"""The phrase-knee diagnostic's figures (docs/journal/2026-11-09-phrase-structure.md, section 6):
from `phrase_walk knee`'s JSON lines, each arm's cost a thread and throughput at each count of
concurrent copies, its inflation against one copy, and the phrase walk against production.

    python3 bench/phrase_knee.py runs.jsonl ROUNDS GRID > summary.md

Exits 1, naming what is missing, unless every (round, arm, copies) cell of ROUNDS rounds over
GRID (as "1,2,4") is present exactly once. It gives no verdict: the registration has none.
"""

import json
import statistics
import sys

ARMS = ("production", "phrase")


def load(path: str, rounds: int, grid: list[int]) -> dict:
    cells: dict = {}
    for line in open(path):
        r = json.loads(line)
        key = (r["round"], r["arm"], r["copies"])
        if key in cells:
            sys.exit(f"phrase_knee: {key} appears twice")
        if r["bytes"] <= 0 or r["wall_ns"] <= 0:
            sys.exit(f"phrase_knee: {key} has no bytes or no time")
        cells[key] = r
    want = {(i, a, n) for i in range(rounds) for a in ARMS for n in grid}
    if set(cells) != want:
        missing = sorted(want - set(cells))
        extra = sorted(set(cells) - want)
        sys.exit(f"phrase_knee: missing {missing[:4]}, unexpected {extra[:4]}")
    return cells


def table(cells: dict, rounds: int, grid: list[int]) -> str:
    def med(arm: str, n: int, f) -> float:
        return statistics.median(f(cells[(i, arm, n)]) for i in range(rounds))

    def per_thread(r: dict) -> float:
        return (r["build_ns"] + r["walk_ns"]) / r["bytes"]

    def rate(r: dict) -> float:
        return r["bytes"] / r["wall_ns"] * 1e3

    out = [
        "| copies | production ns/B a thread | MB/s | inflation | phrase ns/B a thread | MB/s | inflation | phrase / production, MB/s |",
        "|--:|--:|--:|--:|--:|--:|--:|--:|",
    ]
    base = {a: med(a, grid[0], per_thread) for a in ARMS}
    for n in grid:
        row = [str(n)]
        for a in ARMS:
            t = med(a, n, per_thread)
            row += [f"{t:.3f}", f"{med(a, n, rate):.0f}", f"{t / base[a]:.2f}"]
        ratio = statistics.median(
            rate(cells[(i, "phrase", n)]) / rate(cells[(i, "production", n)]) for i in range(rounds)
        )
        out.append("| " + " | ".join(row + [f"{ratio:.3f}"]) + " |")
    out.append("")
    out.append(
        f"Medians over {rounds} rounds; inflation is against {grid[0]} "
        "copy; the last column is the median of the rounds' paired ratios."
    )
    return "\n".join(out)


if __name__ == "__main__":
    path, rounds, grid = sys.argv[1], int(sys.argv[2]), [int(x) for x in sys.argv[3].split(",")]
    print(table(load(path, rounds, grid), rounds, grid))
