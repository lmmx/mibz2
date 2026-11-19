"""open-port-3's operating characteristics, before any of its data.

B1's and B2's false-fail rates and power, simulated from the first attempt's noise (open-port/run-20261119T125538Z,
runs.jsonl, 12 rounds a cell; its NOT ADOPTED stands). Each cell's 12 log ratios are centred on
their mean (the residuals), and a simulated cell draws 16 of them with replacement:
- own: from its own cell's residuals;
- part: from all the part's residuals pooled (every cell the same spread).
The pooled bound is the scorer's stratified percentile bootstrap (`BOOTS` resamples here, the
scorer uses 20,000); the flag is the scorer's one-sided t bound at 1 - 0.05/k.

    python3 oc.py RUNS.jsonl > oc.txt
"""

import json, math, sys
import numpy as np

sys.path.insert(0, __file__.rsplit("/docs/", 1)[0] + "/bench")
import open_port as op  # noqa: E402
import open_port3 as o3  # noqa: E402

SIMS, BOOTS, N = 2000, 2000, o3.GUARD_ROUNDS
rng = np.random.default_rng(7013818037)
errors = []
rows = op.load(sys.argv[1], list(op.CORPORA), list(op.COUNTS), op.ROUNDS, errors)
assert not errors, errors


def sim(res, k, shift, model, effect_cell=None, effect=0.0):
    """Rates over SIMS: pooled upper bound above 1.01, any flag, and the effect cell's flag."""
    tq = o3.t_quantile(1 - 0.05 / k, N - 1)
    allres = np.concatenate(res)
    pool_fail = any_flag = cell_flag = 0
    for _ in range(SIMS):
        xs = np.stack(
            [
                rng.choice(res[j] if model == "own" else allres, N)
                + shift
                + (effect if j == effect_cell else 0.0)
                for j in range(k)
            ]
        )
        means = xs.mean(axis=1)
        idx = rng.integers(0, N, size=(BOOTS, k, N))
        boot = (
            np.take_along_axis(np.broadcast_to(xs, (BOOTS, k, N)), idx, axis=2)
            .mean(axis=2)
            .mean(axis=1)
        )
        hi = np.sort(boot)[int(math.ceil(0.975 * BOOTS)) - 1]
        pool_fail += hi > math.log(o3.POOLED_MAX)
        lb = means - tq * xs.std(axis=1, ddof=1) / math.sqrt(N)
        flags = lb > math.log(o3.FLAG_MIN)
        any_flag += flags.any()
        if effect_cell is not None:
            cell_flag += flags[effect_cell]
    return pool_fail / SIMS, any_flag / SIMS, cell_flag / SIMS


def need(sd, k, power=0.8):
    """The true slowdown a cell of residual spread sd needs for its flag to fire with `power`."""
    z = {0.8: 0.8416}[power]
    return math.exp(
        math.log(o3.FLAG_MIN) + (o3.t_quantile(1 - 0.05 / k, N - 1) + z) * sd / math.sqrt(N)
    )


for name, corpora in (("B1", o3.B1), ("B2", o3.B2)):
    cells = [(c, t) for t in op.COUNTS for c in corpora]
    k = len(cells)
    res = [np.array(o3.logs(rows, c, t, op.ROUNDS)) for c, t in cells]
    res = [r - r.mean() for r in res]
    print(
        f"{name}: {k} cells, {N} rounds; residual sd per cell (12 first-attempt rounds), median "
        f"{np.median([r.std(ddof=1) for r in res]):.4f}, max {max(r.std(ddof=1) for r in res):.4f}"
    )
    for model in ("own", "part"):
        f0 = sim(res, k, 0.0, model)
        print(f"  [{model}] no effect: pooled bound fails {f0[0]:.3f}; any flag {f0[1]:.3f}")
        for s in (0.005, 0.01, 0.015):
            f = sim(res, k, math.log1p(s), model)
            print(
                f"  [{model}] every cell +{s * 100:.1f}%: pooled bound fails {f[0]:.3f}; any flag {f[1]:.3f}"
            )
    worst = max(range(k), key=lambda j: res[j].std(ddof=1))
    best = min(range(k), key=lambda j: res[j].std(ddof=1))
    for j in (best, worst):
        for e in (0.02, 0.05):
            f = sim(res, k, 0.0, "own", effect_cell=j, effect=math.log1p(e))
            print(
                f"  [own] {cells[j][0]} {cells[j][1]}t alone +{e * 100:.0f}% (sd {res[j].std(ddof=1):.4f}): "
                f"its flag {f[2]:.3f}; pooled bound fails {f[0]:.3f}"
            )
    print(
        f"  a single cell's slowdown for 80% flag power: "
        + ", ".join(f"{c} {t}t x{need(r.std(ddof=1), k):.3f}" for (c, t), r in zip(cells, res))
    )
