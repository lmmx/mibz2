"""The scanner's knee: from `par_scan knee`'s JSON lines (examples/par_scan.rs), the decoder's
rate at each worker count with a sequential scanner thread (`scan`) and with a magic finder and
workers that read their own blocks (`find`), and the registered reading at the largest count.

    python3 bench/par_scan_knee.py runs.jsonl ROUNDS WORKERS REPORT.json EXPECTED_BYTES > summary.md

Exits 1, naming the fault, unless runs.jsonl is exactly the registered grid: every (round, arm,
workers) once; exactly the fields; whole, positive times; and in every cell exactly
EXPECTED_BYTES decoded bytes. EXPECTED_BYTES is the input's decoded size as recorded apart from
the run (bench-par-scan takes it from the corpus manifest, bench/wikidata.json, or from the
records generator, each checked against libbzip2's decode of the input): a run that decoded
some other size in every cell is refused, not read.

FASTER is a descriptive selection threshold on the median of the paired ratios, not a test of
a population gain, and qualifies no production change.
"""

import json
import math
import statistics
import sys

KEYS = {"round", "arm", "workers", "wall_ns", "cpu_ns", "bytes"}
ARMS = ("scan", "find")
# the registered reading, on find's rate over scan's at the largest worker count
FASTER, NOT_FASTER = 1.15, 1.05


def die(msg: str) -> None:
    sys.exit(f"par_scan_knee: {msg}")


def whole(v) -> bool:
    return isinstance(v, int) and not isinstance(v, bool)


def load(path: str, rounds: int, workers: list[int], expected: int) -> dict:
    if rounds < 1 or not workers or len(set(workers)) != len(workers) or min(workers) < 1:
        die("ROUNDS must be at least 1, and WORKERS distinct counts of at least 1")
    if expected < 1:
        die("EXPECTED_BYTES must be at least 1")
    cells: dict = {}
    for n, line in enumerate(open(path), 1):
        try:
            r = json.loads(line)
        except json.JSONDecodeError as e:
            die(f"line {n} is not JSON ({e})")
        if not isinstance(r, dict) or set(r) != KEYS:
            die(f"line {n} does not have exactly the fields {sorted(KEYS)}")
        if not all(whole(r[k]) for k in KEYS - {"arm"}):
            die(f"line {n} has a field that is not a whole number")
        if r["arm"] not in ARMS:
            die(f"line {n} has an arm that is not one of {list(ARMS)}")
        key = (r["round"], r["arm"], r["workers"])
        if key in cells:
            die(f"{key} appears twice")
        if r["wall_ns"] <= 0:
            die(f"{key} has no time")
        if r["cpu_ns"] <= 0:
            die(f"{key} has no CPU time")
        if r["bytes"] <= 0:
            die(f"{key} has {r['bytes']} decoded bytes")
        cells[key] = r
    want = {(i, a, t) for i in range(rounds) for a in ARMS for t in workers}
    if set(cells) != want:
        missing, extra = sorted(want - set(cells)), sorted(set(cells) - want)
        die(f"not the registered grid: missing {missing[:4]}, unexpected {extra[:4]}")
    sizes = {r["bytes"] for r in cells.values()}
    if len(sizes) != 1:
        die(f"the cells decoded {sorted(sizes)} bytes, not one size")
    if sizes != {expected}:
        die(f"the cells decoded {sorted(sizes)} bytes, not the expected {expected}")
    return cells


def rate(r: dict) -> float:
    """Decoded MB/s."""
    return r["bytes"] / r["wall_ns"] * 1e3


def report(cells: dict, rounds: int, workers: list[int]) -> tuple[str, dict]:
    out = [
        "| workers | scan MB/s | find MB/s | scan CPU s | find CPU s | find / scan (paired) | min–max |",
        "|--:|--:|--:|--:|--:|--:|--:|",
    ]
    ratios: dict = {}
    for t in sorted(workers):
        rows = {a: [cells[(i, a, t)] for i in range(rounds)] for a in ARMS}
        mbs = {a: statistics.median(rate(r) for r in rows[a]) for a in ARMS}
        cpu = {a: statistics.median(r["cpu_ns"] for r in rows[a]) / 1e9 for a in ARMS}
        # (one decoded size in every cell, so the rates' ratio is the times', inverted)
        paired = [s["wall_ns"] / f["wall_ns"] for f, s in zip(rows["find"], rows["scan"])]
        ratio, lo, hi = statistics.median(paired), min(paired), max(paired)
        ratios[t] = (ratio, lo, hi)
        out.append(
            f"| {t} | {mbs['scan']:.0f} | {mbs['find']:.0f} | {cpu['scan']:.3f} | {cpu['find']:.3f} "
            f"| {ratio:.3f} | {lo:.3f}–{hi:.3f} |"
        )
    top = max(workers)
    r, lo, hi = ratios[top]
    if not all(math.isfinite(x) for x in (r, lo, hi)):
        die("a ratio is not finite")
    said = f"at {top} workers find ran at {r:.3f} of scan's rate"
    if r >= FASTER:
        reading = f"FASTER: {said}, at least {FASTER}"
    elif r <= NOT_FASTER:
        reading = f"NOT FASTER: {said}, at most {NOT_FASTER}"
    else:
        reading = f"MIXED: {said}, between {NOT_FASTER} and {FASTER}"
    verdict = {"top_workers": top, "ratio": r, "ratio_range": [lo, hi], "reading": reading}
    out += [
        "",
        f"Medians over {rounds} rounds. scan: a scanner thread and the workers; find: a finder "
        "thread and workers that read their own blocks. The ratio is find's MB/s over scan's, "
        "round by round.",
        "",
        f"**Reading: {reading}**",
    ]
    return "\n".join(out), verdict


if __name__ == "__main__":
    if len(sys.argv) != 6:
        die("usage: par_scan_knee.py runs.jsonl ROUNDS WORKERS REPORT.json EXPECTED_BYTES")
    path, rounds = sys.argv[1], int(sys.argv[2])
    workers = [int(x) for x in sys.argv[3].split(",")]
    expected = int(sys.argv[5])
    text, verdict = report(load(path, rounds, workers, expected), rounds, workers)
    verdict["expected_bytes"] = expected
    with open(sys.argv[4], "w") as f:
        json.dump(verdict, f, indent=1)
    print(text)
