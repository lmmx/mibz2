#!/usr/bin/env python3
"""Checks of bench/character_freq.py on synthetic decodes: walls that scale as 1/G with the clock
at two levels give R^2 1 and slope -1, and the frequency-adjusted SD near 0; walls whose noise is
independent of the clock give R^2 near 0 and leave the SD as it was."""

import io
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import character_freq as cf  # noqa: E402


def data(clock_drives: bool, seed=3):
    rnd = random.Random(seed)
    cells = [{"section": "screen", "k": 10, "arm": a, "treat": []} for a in ("base", "A3", "A7")]
    t = {}
    for b in range(16):
        for ci, eff in enumerate((1.0, 1.1, 1.02)):
            g = rnd.choice((4.1, 4.9))
            noise = 1.0 if clock_drives else math.exp(rnd.gauss(0, 0.04))
            wall = 0.2 * eff * (4.9 / g if clock_drives else 1.0) * noise
            ref = 1e9
            t[(ci, b)] = {"wall_s": wall, "user_s": 1.0, "freq_khz": [g * 1e6] * 20,
                          "pmu": {"cycles": [ref * g / cf.NOMINAL_GHZ], "ref-cycles": [ref]}}  # fmt: skip
    return {"cells": cells}, t


def main():
    ok = True
    _, t = data(True)
    res = cf.analyse({"cells": data(True)[0]["cells"]}, t, out=io.StringIO())
    q = res[(10, "q2")]
    good = q["r2_clock"] > 0.99 and abs(q["slope"] + 1) < 0.01 and q["sd_adj"] < 1e-9
    print(
        f"{'ok  ' if good else 'FAIL'} walls as 1/G: R^2 {q['r2_clock']:.3f}, slope {q['slope']:+.2f}, adjusted SD {q['sd_adj']:.4f}"
    )
    ok &= good
    p, t = data(False)
    q = cf.analyse(p, t, out=io.StringIO())[(10, "q2")]
    good = q["r2_clock"] < 0.2 and q["sd_adj"] > 0.8 * q["sd_raw"]
    print(
        f"{'ok  ' if good else 'FAIL'} noise apart from the clock: R^2 {q['r2_clock']:.3f}, adjusted SD {q['sd_adj']:.4f} against {q['sd_raw']:.4f}"
    )
    ok &= good
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
