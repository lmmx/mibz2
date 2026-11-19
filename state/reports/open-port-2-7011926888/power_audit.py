"""Planning audit of the archived first attempt. No fresh timings."""
import collections
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy import stats

ROOT = Path(sys.argv[1])
REPS = 18000
RNG = np.random.default_rng(7011926888)
ALPHA = 0.048 / 40
TARGET = np.log(1.02)
TIER_A = {"runs", "ho-prose-l1", "ho-code-l1", "ho-code-l9"}
GRID = (12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64, 72, 80, 88, 96, 104, 112, 120)
cells = collections.defaultdict(list)
for name in ("runs.jsonl", "sustained.jsonl"):
    pairs = collections.defaultdict(dict)
    for line in (ROOT / name).read_text().splitlines():
        r = json.loads(line)
        pairs[(r["corpus"], r["threads"], r["round"])]["c" if "/build-port/" in r["arm"] else "r"] = r["wall"]
    for (c, t, _), pair in sorted(pairs.items()):
        cells[c, t].append(math.log(pair["c"] / pair["r"]))
cells = {k: np.asarray(x) for k, x in cells.items() if k[0] not in ("wikidata-x4", "ho-wikidata-l9")}
assert len(cells) == 40
resid = {k: x - np.median(x) for k, x in cells.items()}
pool = {c: np.concatenate([x for (cc, _), x in resid.items() if cc == c]) for c, _ in cells}
sym = lambda x: np.concatenate((x, -x))

def old(x):
    n = x.shape[1]
    mean = x.mean(axis=1)
    lo = mean - stats.t.ppf(.975, n - 1) * x.std(axis=1, ddof=1) / math.sqrt(n)
    return (mean > TARGET) | (lo > 0)

def signed(x):
    # Fast screening only. Asymptotic, with SciPy's tie correction.
    return stats.wilcoxon(x, alternative="greater", method="asymptotic", axis=1).pvalue <= ALPHA

def mean_t(x):
    n = x.shape[1]
    return x.mean(axis=1) > stats.t.ppf(1 - ALPHA, n - 1) * x.std(axis=1, ddof=1) / math.sqrt(n)

null_pool, null_cell, allocation = [], [], []
print("corpus,t,planned,old-pool,g2-pool,old-cell,g2-cell,paired-t-cell,rank-screen-n")
for (c, t), x in sorted(resid.items()):
    n = 40 if c == "wikidata-x16" else (120 if c in TIER_A else 60)
    rpool = RNG.choice(sym(pool[c]), size=(REPS, max(GRID)))
    rcell = RNG.choice(sym(x), size=(REPS, max(GRID)))
    o_pool = old(rpool[:, :len(x)] + TARGET).mean()
    o_cell = old(rcell[:, :len(x)] + TARGET).mean()
    null_pool.append(old(rpool[:, :len(x)]).mean())
    null_cell.append(old(rcell[:, :len(x)]).mean())
    p_pool = signed(rpool[:, :n] + TARGET).mean()
    p_cell = signed(rcell[:, :n] + TARGET).mean()
    p_t = mean_t(rcell[:, :n] + TARGET).mean()
    # Screen for the draft's floor of 0.76 plus a one-point Monte Carlo cushion
    # and own-cell old-rule parity. Larger than 120 is deliberately left unresolved.
    target = max(.76, o_cell) + .01
    k = next((j for j in GRID if signed(rcell[:, :j] + TARGET).mean() >= target), -1)
    allocation.append((c, t, n, k))
    print(c, t, n, *(f"{v:.3f}" for v in (o_pool, p_pool, o_cell, p_cell, p_t)), k, sep=",")
print("independent product old null, pooled:", 1 - np.prod(1 - np.asarray(null_pool)))
print("independent product old null, per-cell:", 1 - np.prod(1 - np.asarray(null_cell)))
print("screen extension pairs, original:", sum(n - 12 for c, _, n, _ in allocation if c != "wikidata-x16"))
print("screen extension pairs, preserving planned n at unresolved cells:", sum((k if k > 0 else n) - 12 for c, _, n, k in allocation if c != "wikidata-x16"))
print("screen unresolved:", [(c, t) for c, t, _, k in allocation if k < 0])
# Crude simultaneous dependence check: resample whole within-thread round vectors.
# Different thread counts and the separate sustained invocation are drawn independently.
p_joint = np.zeros(REPS, bool)
for t in (8, 10, 20):
    aligned = np.column_stack([x for (c, tt), x in sorted(resid.items()) if tt == t and c != "wikidata-x16"])
    x = aligned[RNG.integers(len(aligned), size=(REPS, len(aligned)))]
    x *= RNG.choice((-1, 1), size=(REPS, len(aligned), 1))
    for j in range(x.shape[-1]):
        p_joint |= old(x[:, :, j])
x = resid["wikidata-x16", 10]
p_joint |= old(RNG.choice(sym(x), size=(REPS, len(x))))
print("old null with shared within-count resampled rounds/signs:", p_joint.mean())
