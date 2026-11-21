"""open-port-3's corrected B2 and B4 (the Researcher's HOLD 7015030749), their false-fail rates.

Before any of their data. B2 is G in each of the 12 admitted-guard cells at 32 rounds (not SLOWER
at 95%, a geometric mean at most 1.02); B4 is S at 8 rounds (the same rule). Simulated from the
first attempt's noise (open-port/run-20261119T125538Z: runs.jsonl, 12 rounds a cell; and
sustained.jsonl, 8 rounds), each cell's residuals centred on their mean and drawn with
replacement: from its own cell ("own") or from all the part's ("part").

    python3 oc2.py RUNS.jsonl SUSTAINED.jsonl > oc2.txt
"""

import math
import sys

import numpy as np

sys.path.insert(0, __file__.rsplit("/docs/", 1)[0] + "/bench")
import open_port as op  # noqa: E402
import open_port3 as o3  # noqa: E402

SIMS = 100000
rng = np.random.default_rng(7015030749)
errors = []
rows = op.load(sys.argv[1], list(op.CORPORA), list(op.COUNTS), op.ROUNDS, errors)
srows = op.load(sys.argv[2], [op.SUSTAINED], [10], op.SUSTAINED_ROUNDS, errors)
assert not errors, errors


def fails(draws, n, shift):
    """Per simulation: SLOWER or above 1.02 (G and S)."""
    xs = draws + shift
    m = xs.mean(axis=1)
    h = o3.t_quantile(0.975, n - 1) * xs.std(axis=1, ddof=1) / math.sqrt(n)
    return (m - h > 0) | (m > math.log(o3.GUARD_MAX))


cells = [(c, t) for t in op.COUNTS for c in o3.B2]
res = {k: np.array(o3.logs(rows, *k, op.ROUNDS)) for k in cells}
res = {k: r - r.mean() for k, r in res.items()}
allres = np.concatenate(list(res.values()))
n = o3.B2_ROUNDS
for model in ("own", "part"):
    print(
        f"B2 as G at {n} rounds [{model}]: each cell's false-fail under no effect; power at +2% and +3%"
    )
    anyfail = np.zeros(SIMS, dtype=bool)
    for k in cells:
        pool = res[k] if model == "own" else allres
        d = rng.choice(pool, (SIMS, n))
        f0 = fails(d, n, 0.0)
        anyfail |= f0
        p2 = fails(d, n, math.log(1.02)).mean()
        p3 = fails(d, n, math.log(1.03)).mean()
        print(
            f"  {k[0]} {k[1]}t (sd {res[k].std(ddof=1):.4f}): {f0.mean():.4f}; +2% {p2:.3f}; +3% {p3:.3f}"
        )
    print(f"  any of the 12 under no effect: {anyfail.mean():.4f}")
for m16 in (16,):
    anyfail = np.zeros(SIMS, dtype=bool)
    for k in cells:
        anyfail |= fails(rng.choice(res[k], (SIMS, m16)), m16, 0.0)
    print(f"  (for comparison, the same at {m16} rounds [own]: any of the 12 {anyfail.mean():.4f})")
s = np.array(o3.logs(srows, op.SUSTAINED, 10, op.SUSTAINED_ROUNDS))
s = s - s.mean()
for name, pool in (("own", s), ("all guards'", allres)):
    d = rng.choice(pool, (SIMS, o3.SUSTAINED_ROUNDS))
    print(
        f"B4 as S at {o3.SUSTAINED_ROUNDS} rounds [{name} residuals, sd {pool.std(ddof=1):.4f}]: "
        f"false-fail {fails(d, o3.SUSTAINED_ROUNDS, 0.0).mean():.4f}; +2% {fails(d, o3.SUSTAINED_ROUNDS, math.log(1.02)).mean():.3f}; "
        f"+3% {fails(d, o3.SUSTAINED_ROUNDS, math.log(1.03)).mean():.3f}"
    )

# the two parts of G at 32 rounds, apart [own], and the same under normal noise of each cell's sd
for label, which in (("SLOWER alone", 0), ("the point bound alone", 1)):
    anyfail = np.zeros(SIMS, dtype=bool)
    for k in cells:
        xs = rng.choice(res[k], (SIMS, n))
        m = xs.mean(axis=1)
        h = o3.t_quantile(0.975, n - 1) * xs.std(axis=1, ddof=1) / math.sqrt(n)
        anyfail |= (m - h > 0) if which == 0 else (m > math.log(o3.GUARD_MAX))
    print(f"B2 at {n} rounds [own], {label}: any of the 12 {anyfail.mean():.4f}")
anyfail = np.zeros(SIMS, dtype=bool)
for k in cells:
    anyfail |= fails(rng.normal(0, res[k].std(ddof=1), (SIMS, n)), n, 0.0)
print(f"B2 at {n} rounds, normal noise at each cell's sd: any of the 12 {anyfail.mean():.4f}")
