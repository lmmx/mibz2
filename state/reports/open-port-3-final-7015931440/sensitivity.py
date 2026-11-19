"""Compare old and revised wall guards using the archived pilot.

Run with PYTHONPATH set to bench/ in the reviewed checkout and pass the directory
containing runs.jsonl and sustained.jsonl from the original timed archive.
"""

import math
import sys
from pathlib import Path

import numpy as np
from scipy.stats import t

import open_port3 as revised
from open_port3 import op


def residuals(log_ratios):
    return np.asarray(log_ratios) - np.mean(log_ratios)


def main(pilot):
    errors = []
    settled = op.load(pilot / "runs.jsonl", list(op.CORPORA), list(op.COUNTS), 12, errors)
    sustained = op.load(pilot / "sustained.jsonl", [op.SUSTAINED], [10], 8, errors)
    if errors:
        raise ValueError(errors)

    rng = np.random.default_rng(7015931440)
    draws = 150_000
    point_cap = math.log(1.02)
    first_holm_alpha = 0.05 / 12
    for threads in op.COUNTS:
        for corpus in revised.B2:
            pool = residuals(revised.logs(settled, corpus, threads, 12))
            before = rng.choice(pool, (draws, 12))
            after = rng.choice(pool, (draws, 48))
            before_mean = before.mean(1) + point_cap
            after_mean = after.mean(1) + point_cap
            before_half = t.ppf(0.975, 11) * before.std(1, ddof=1) / math.sqrt(12)
            after_half = t.ppf(1 - first_holm_alpha, 47) * after.std(1, ddof=1) / math.sqrt(48)
            before_fail = ((before_mean > point_cap) | (before_mean > before_half)).mean()
            after_fail = ((after_mean > point_cap) | (after_mean > after_half)).mean()
            print((corpus, threads), "G/B2 single-cell contribution at +2%",
                  f"{before_fail:.4f}", f"{after_fail:.4f}")

    pool = residuals(revised.logs(sustained, op.SUSTAINED, 10, 8))
    for rounds in (8, 16):
        sample = rng.choice(pool, (draws, rounds))
        mean = sample.mean(1)
        half_width = t.ppf(0.975, rounds - 1) * sample.std(1, ddof=1) / math.sqrt(rounds)
        rates = []
        for slowdown in (0, 0.02, 0.03):
            shifted_mean = mean + math.log(1 + slowdown)
            rates.append(((shifted_mean > point_cap) | (shifted_mean > half_width)).mean())
        print("S", rounds, "false rejection/+2%/+3%", *(f"{rate:.4f}" for rate in rates))


if __name__ == "__main__":
    main(Path(sys.argv[1]))
