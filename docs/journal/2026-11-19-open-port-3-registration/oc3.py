"""open-port-3's B2 and B4 after the Coordinator's prospective ruling, before their data.

B2: each of the 12 admitted-guard cells at 32 rounds fails if its one-sided slowdown test is
rejected under Holm across the 12 (family alpha 0.05), or its geometric mean is above 1.02.
B4: S at 16 rounds (not SLOWER at 95%, a geometric mean at most 1.02). Simulated from the first
attempt's noise as oc2.py: each cell's centred residuals drawn with replacement from its own cell
("own") or from the part's pooled ("part"); B4 from the sustained residuals (own) or all the
guards' (part). The overall false NOT ADOPTED rate combines these with B1's rates from oc.txt
(the parts are independent runs).

    python3 oc3.py RUNS.jsonl SUSTAINED.jsonl > oc3.txt
"""

import math
import sys

import numpy as np
from scipy import stats

sys.path.insert(0, __file__.rsplit("/docs/", 1)[0] + "/bench")
import open_port as op  # noqa: E402
import open_port3 as o3  # noqa: E402

SIMS = 50000
rng = np.random.default_rng(7015339412)
errors = []
rows = op.load(sys.argv[1], list(op.CORPORA), list(op.COUNTS), op.ROUNDS, errors)
srows = op.load(sys.argv[2], [op.SUSTAINED], [10], op.SUSTAINED_ROUNDS, errors)
assert not errors, errors
# B1's rates under no effect (oc.txt): the pooled bound fails, any flag
B1 = {"own": (0.015, 0.035), "part": (0.017, 0.000)}


def t_sf(t, df):
    """The one-sided upper tail of Student's t (scipy; the scorer's own is op.betainc)."""
    return stats.t.sf(t, df)


def b2_fails(xs):
    """xs: (sims, 12, n). Per simulation and cell: Holm-adjusted SLOWER or above 1.02."""
    n = xs.shape[2]
    m = xs.mean(axis=2)
    se = xs.std(axis=2, ddof=1) / math.sqrt(n)
    p = t_sf(m / se, n - 1)
    k = p.shape[1]
    order = np.argsort(p, axis=1)
    ps = np.take_along_axis(p, order, axis=1)
    thresh = 0.05 / (k - np.arange(k))
    ok = ps <= thresh
    # Holm: reject the first r sorted, r the first index that fails
    r = np.where(ok.all(axis=1), k, np.argmin(ok, axis=1))
    rej_sorted = np.arange(k)[None, :] < r[:, None]
    rej = np.zeros_like(rej_sorted)
    np.put_along_axis(rej, order, rej_sorted, axis=1)
    return rej | (m > math.log(o3.GUARD_MAX))


def s_fails(xs):
    """S per simulation: SLOWER at 95%, or above 1.02."""
    n = xs.shape[1]
    m = xs.mean(axis=1)
    h = o3.t_quantile(0.975, n - 1) * xs.std(axis=1, ddof=1) / math.sqrt(n)
    return (m - h > 0) | (m > math.log(o3.GUARD_MAX))


cells = [(c, t) for t in op.COUNTS for c in o3.B2]
res = [np.array(o3.logs(rows, c, t, op.ROUNDS)) for c, t in cells]
res = [r - r.mean() for r in res]
allres = np.concatenate(res)
s = np.array(o3.logs(srows, op.SUSTAINED, 10, op.SUSTAINED_ROUNDS))
s = s - s.mean()
n, ns = o3.B2_ROUNDS, o3.SUSTAINED_ROUNDS


def draw(model):
    """(SIMS, 12, n) draws of the B2 cells under `model`."""
    if model == "normal":
        return np.stack([rng.normal(0, r.std(ddof=1), (SIMS, n)) for r in res], axis=1)
    return np.stack([rng.choice(r if model == "own" else allres, (SIMS, n)) for r in res], axis=1)


B1["normal"] = B1["part"]
for model in ("own", "part", "normal"):
    print(f"[{model} residuals]")
    base = draw(model)
    f0 = b2_fails(base)
    b2 = f0.any(axis=1).mean()
    point = (base.mean(axis=2) > math.log(o3.GUARD_MAX)).any(axis=1).mean()
    print(
        f"  B2 at {n} rounds, no effect: some cell fails {b2:.4f} "
        f"(the point bound alone {point:.4f})"
    )
    for e in (0.03, 0.05):
        pw = []
        for j in range(len(cells)):
            xs = base.copy()
            xs[:, j, :] += math.log1p(e)
            pw.append(b2_fails(xs)[:, j].mean())
        print(
            f"  B2, one cell alone at +{e * 100:.0f}%: that cell fails "
            + ", ".join(f"{c} {t}t {p:.3f}" for (c, t), p in zip(cells, pw))
            + f" (min {min(pw):.3f}, median {float(np.median(pw)):.3f})"
        )
    sp = s if model in ("own", "normal") else allres
    d = (
        rng.normal(0, s.std(ddof=1), (SIMS, ns))
        if model == "normal"
        else rng.choice(sp, (SIMS, ns))
    )
    b4 = s_fails(d).mean()
    print(
        f"  B4 at {ns} rounds (sd {sp.std(ddof=1):.4f}): no effect fails {b4:.4f}; "
        f"+2% {s_fails(d + math.log(1.02)).mean():.3f}; +3% {s_fails(d + math.log(1.03)).mean():.3f}"
    )
    p1, fl = B1[model]
    total = 1 - (1 - p1) * (1 - fl) * (1 - b2) * (1 - b4)
    print(
        f"  overall false NOT ADOPTED under no effect: 1 - (1-{p1})(1-{fl})(1-{b2:.4f})(1-{b4:.4f}) = {total:.3f}"
    )
