"""The squared-trace pre-screen's verdict (`just bench-squared-trace-prescreen`).

Reads the output of bench/squared_trace/squared_trace_prescreen.c (MONODROMY's pre-screen,
record 92, docs/journal/monodromy/2026-11-23-refinement-and-composition.md), one line per
invocation:

    m 89874: chase T 3.05 ns/row | gather T^2 0.45 ns/row + chase T^2 1.38 ns/row
        = 1.83 ns/row | ratio 0.600 (chk 123)

(on one line; the "(chk N)" tail is the program's own, left out of the record's copy).

The rule, posted on #145 before the data (LOG 7096050983): at m in {89874, 114000, 156404,
204281}, ratio = T^2 two-ended time / two-chain chase time. DEAD if the ratio is >= 0.95 at
every m; else ALIVE.

Each m is run in several invocations (each the program's best of its repetitions); the
ratio at an m is the median of its invocations' ratios. The spread across invocations is
reported, and whether every invocation's ratio sits on the same side of 0.95 as the median,
but it does not change the verdict.

    python3 bench/squared_trace_verdict.py OUTPUT.txt [--expect N] [--json OUT.json]

Prints the per-m table and one last line: "VERDICT: DEAD ...", "VERDICT: ALIVE ..." (exit
0) or "INVALID: ..." (exit 1).
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import sys
from dataclasses import dataclass

RULE_MS = (89874, 114000, 156404, 204281)
THRESHOLD = 0.95
RULE_LOG = "7096050983"

LINE = re.compile(
    r"^m (?P<m>\d+): chase T (?P<chase>\d+(?:\.\d+)?) ns/row"
    r" \| gather T\^2 (?P<gather>\d+(?:\.\d+)?) ns/row"
    r" \+ chase T\^2 (?P<chase2>\d+(?:\.\d+)?) ns/row"
    r" = (?P<total>\d+(?:\.\d+)?) ns/row"
    r" \| ratio (?P<ratio>\d+(?:\.\d+)?)"
    r"(?: \(chk (?P<chk>\d+)\))?\s*$"
)


class Invalid(Exception):
    """The output can't give a verdict."""


@dataclass
class Reading:
    """One invocation's line."""

    m: int
    chase: float
    gather: float
    chase2: float
    total: float
    ratio: float


def parse(text: str) -> list[Reading]:
    """Every invocation's reading; any other non-blank line is an error."""
    out = []
    for n, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        g = LINE.match(line)
        if not g:
            raise Invalid(f"line {n} is not the pre-screen's output: {line!r}")
        r = Reading(
            m=int(g["m"]),
            chase=float(g["chase"]),
            gather=float(g["gather"]),
            chase2=float(g["chase2"]),
            total=float(g["total"]),
            ratio=float(g["ratio"]),
        )
        # the line's own arithmetic, to its printed rounding (2 decimals for ns, 3 for ratio)
        if r.chase <= 0 or r.total <= 0:
            raise Invalid(f"line {n}: a time is not positive: {line!r}")
        if abs(r.gather + r.chase2 - r.total) > 0.0101:
            raise Invalid(f"line {n}: gather + chase T^2 is not the total: {line!r}")
        tol = r.ratio * (0.0051 / r.total + 0.0051 / r.chase) + 0.0006
        if abs(r.total / r.chase - r.ratio) > tol:
            raise Invalid(f"line {n}: the ratio is not total / chase T: {line!r}")
        out.append(r)
    return out


def judge(readings: list[Reading], expect: int = 1) -> dict:
    """The rule's verdict on the readings, with each m's median, spread and agreement."""
    by_m: dict[int, list[float]] = {}
    for r in readings:
        by_m.setdefault(r.m, []).append(r.ratio)
    missing = [m for m in RULE_MS if len(by_m.get(m, [])) < expect]
    if missing:
        have = {m: len(by_m.get(m, [])) for m in missing}
        raise Invalid(f"fewer than {expect} invocation(s) at m = {have}")
    rows = []
    for m in sorted(by_m):
        ratios = by_m[m]
        med = statistics.median(ratios)
        dead_here = med >= THRESHOLD
        rows.append(
            {
                "m": m,
                "in_rule": m in RULE_MS,
                "invocations": len(ratios),
                "ratios": ratios,
                "median": med,
                "min": min(ratios),
                "max": max(ratios),
                "at_or_above_threshold": dead_here,
                "unanimous": all((x >= THRESHOLD) == dead_here for x in ratios),
            }
        )
    ruled = [r for r in rows if r["in_rule"]]
    dead = all(r["at_or_above_threshold"] for r in ruled)
    below = [r["m"] for r in ruled if not r["at_or_above_threshold"]]
    if dead:
        line = f"VERDICT: DEAD (median ratio >= {THRESHOLD} at every m in {list(RULE_MS)})"
    else:
        line = f"VERDICT: ALIVE (median ratio < {THRESHOLD} at m = {below})"
    return {
        "rule": f"LOG {RULE_LOG}: DEAD if ratio >= {THRESHOLD} at every m in {list(RULE_MS)}",
        "aggregate": "median of the invocations' ratios at each m",
        "verdict": "DEAD" if dead else "ALIVE",
        "verdict_line": line,
        "unanimous": all(r["unanimous"] for r in ruled),
        "rows": rows,
    }


def report(v: dict) -> str:
    """The per-m table and the verdict line, last."""
    lines = [
        f"rule ({v['rule']}); ratio at an m: the {v['aggregate']}",
        f"{'m':>8} {'n':>3} {'median':>7} {'min':>7} {'max':>7}  side of {THRESHOLD}",
    ]
    for r in v["rows"]:
        side = ">=" if r["at_or_above_threshold"] else "<"
        note = "" if r["unanimous"] else "  (invocations disagree)"
        if not r["in_rule"]:
            note += "  (not in the rule)"
        lines.append(
            f"{r['m']:>8} {r['invocations']:>3} {r['median']:>7.3f} {r['min']:>7.3f}"
            f" {r['max']:>7.3f}  {side}{note}"
        )
    if not v["unanimous"]:
        lines.append("note: at some m an invocation's ratio sits on the other side of the median")
    lines.append(v["verdict_line"])
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    """Read the output, print the table and the verdict."""
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("output", help="the program's output (one line per invocation); - for stdin")
    p.add_argument("--expect", type=int, default=1, help="invocations required at each m")
    p.add_argument("--json", help="also write the verdict as JSON here")
    a = p.parse_args(argv)
    text = sys.stdin.read() if a.output == "-" else open(a.output).read()
    try:
        v = judge(parse(text), a.expect)
    except Invalid as e:
        print(f"INVALID: {e}")
        return 1
    print(report(v))
    if a.json:
        with open(a.json, "w") as f:
            json.dump(v, f, indent=1)
            f.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
