"""The successor's guard rule, planned from the first attempt's noise (open-port/run-20261119T125538Z).

    python3 plan.py RUN_DIR [REPS]   (numpy and scipy; planning only, not run by the recipe)

The noise model: per corpus, each cell's paired log ratios (port over v0.5.6) less the cell's
median, pooled over the three counts and symmetrised (no effect: the arms are exchangeable). It is
planning noise, not an outcome. Effects are added as a shift of every round's log ratio.

The registered rule G (timing part): a cell fails if its geometric mean exceeds 1.02, or its
two-sided 95% t-interval lies wholly above 1, from 12 rounds (sustained: 8).
The successor's rule G2: a cell fails if a one-sided exact Wilcoxon signed-rank test of "the port
is slower" gives p <= 0.048/40 = 0.0012, from n rounds (TIER below; sustained 40).
"""

import json, math, collections, sys
import numpy as np
from scipy import stats

RUN = sys.argv[1]
REPS = int(sys.argv[2]) if len(sys.argv) > 2 else 2000
TIER_A = ("ho-prose-l1", "ho-code-l9", "ho-code-l1", "runs")
N = {**{c: 120 for c in TIER_A}, "wikidata-x16": 40}
N_B = 60
FAMILY, ALPHA = 40, 0.048 / 40
L2, L1 = math.log(1.02), math.log(1.01)


def cells_of(path):
    """{(corpus, threads): the rounds' paired log ratios, the port over v0.5.6}."""
    d = collections.defaultdict(dict)
    for line in open(path):
        r = json.loads(line)
        d[(r["corpus"], r["threads"], r["round"])]["c" if "/build-port/" in r["arm"] else "r"] = r
    out = collections.defaultdict(list)
    for (c, t, i), v in sorted(d.items()):
        out[(c, t)].append(math.log(v["c"]["wall"] / v["r"]["wall"]))
    return out


cells = cells_of(f"{RUN}/runs.jsonl")
cells.update(cells_of(f"{RUN}/sustained.jsonl"))
guards = sorted(k for k in cells if k[0] not in ("wikidata-x4", "ho-wikidata-l9"))
assert len(guards) == FAMILY
shape = collections.defaultdict(list)
for (c, t), x in cells.items():
    m = np.median(x)
    shape[c] += [v - m for v in x]
shape = {c: np.concatenate([np.array(v), -np.array(v)]) for c, v in shape.items()}
rng = np.random.default_rng(20261119)


def draw(c, n, d):
    """REPS samples of n rounds from corpus c's noise, shifted by the true effect d."""
    return d + rng.choice(shape[c], size=(REPS, n))


def registered(x):
    """The registered G's timing part: the mean above 1.02, or SLOWER."""
    n = x.shape[1]
    m = x.mean(1)
    se = x.std(1, ddof=1) / math.sqrt(n)
    return (m > L2) | (m - stats.t.ppf(0.975, n - 1) * se > 0)


def g2(x):
    """The successor's G2: a one-sided Wilcoxon signed-rank p at most ALPHA."""
    out = np.empty(x.shape[0], bool)
    for i, r in enumerate(x):
        r = r[r != 0]
        out[i] = (
            len(r) > 0
            and stats.wilcoxon(
                r, alternative="greater", method="exact" if len(r) <= 50 else "approx"
            ).pvalue
            <= ALPHA
        )
    return out


rows, null_reg, null_g2 = [], [], []
for k in guards:
    c, n_reg = k[0], len(cells[k])
    n = N.get(c, N_B)
    p = {}
    for name, d in (
        ("0", 0.0),
        ("+1%", L1),
        ("+2%", L2),
        ("+3%", math.log(1.03)),
        ("+5%", math.log(1.05)),
        ("+10%", math.log(1.10)),
    ):
        p[name] = (registered(draw(c, n_reg, d)).mean(), g2(draw(c, n, d)).mean())
    null_reg.append(p["0"][0])
    null_g2.append(p["0"][1])
    rows.append((k, n_reg, n, p))
print(f"no true effect, P(at least one of the {FAMILY} timing guards fails):")
print(f"  registered G: {1 - np.prod([1 - v for v in null_reg]):.3f}")
print(f"  successor G2: {1 - np.prod([1 - v for v in null_g2]):.3f} (bound by construction: 0.048)")
print()
print("| cell | rounds (registered → successor) | +1% | +2% | +3% | +5% | +10% |")
print("|---|---|--:|--:|--:|--:|--:|")
weaker = []
for k, nr, n, p in rows:
    f = lambda s: f"{p[s][0]:.2f} → {p[s][1]:.2f}"
    print(
        f"| {k[0]} {k[1]}t | {nr} → {n} | {f('+1%')} | {f('+2%')} | {f('+3%')} | {f('+5%')} | {f('+10%')} |"
    )
    for s in ("+2%", "+3%", "+5%", "+10%"):
        if p[s][1] + 0.02 < p[s][0]:
            weaker.append((k, s, p[s]))
print()
print(
    f"cells and effects of 2% or more where G2 catches less often than G (tolerance 0.02, {REPS} draws): {weaker or 'none'}"
)
