"""Evaluate the locked #635 cutoffs with fresh Monte Carlo draws from #631 residuals."""
import collections
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy import stats

ROOT = Path(sys.argv[1])
R = 1_000_000
RNG = np.random.default_rng(5013154415)
L1, L2, L3 = np.log([1.01, 1.02, 1.03])
S = ('wikidata-x16', 10)
cells = collections.defaultdict(list)
for name in ('runs.jsonl', 'sustained.jsonl'):
    pairs = collections.defaultdict(dict)
    for line in (ROOT / name).read_text().splitlines():
        r = json.loads(line)
        if r['corpus'] not in ('wikidata-x4', 'ho-wikidata-l9'):
            pairs[(r['corpus'], r['threads'], r['round'])]['c' if '/build-port/' in r['arm'] else 'r'] = r['wall']
    for (c, t, _), pair in sorted(pairs.items()):
        cells[c, t].append(math.log(pair['c'] / pair['r']))

own = {k: np.asarray(v) - np.mean(v) for k, v in cells.items()}
pooled = {c: np.concatenate([v for (cc, _), v in own.items() if cc == c]) for c, _ in own}
allres = np.concatenate([v for k, v in own.items() if k != S])
plan = {(e['corpus'], e['threads']): (e['rounds'], e['c']) for e in json.loads(Path('open-port-2-schedule.json').read_text())['cells']}
assert plan.keys() == own.keys()
assert len(plan) == 40
rejected = []
print('cell,n,c,G,G2,target,delta,MC_se', flush=True)

def draws(x, n, fmt):
    out = np.empty(R)
    if fmt == 'blocks':
        assert n % 4 == 0 and len(x) % 4 == 0
        x = x.reshape(-1, 4).mean(axis=1)
        n //= 4
    step = max(1, 2_000_000 // n)
    for i in range(0, R, step):
        a = RNG.choice(x, size=(min(step, R-i), n))
        if fmt == 'stress':
            hit = RNG.random(a.shape) < .02
            a = np.where(hit, RNG.uniform(.2, .4, a.shape) * RNG.choice((-1, 1), a.shape), a)
        out[i:i+len(a)] = a.mean(axis=1)
    return out

def old(x, n, d):
    a = RNG.choice(x, size=(R, n)) + d
    m = a.mean(1)
    h = stats.t.ppf(.975, n - 1) * a.std(1, ddof=1) / np.sqrt(n)
    return ((m > L2) | (m - h > 0)).mean()


for k, nxt in [(('ho-structured-l9', 8), 96), (('ho-structured-l9', 10), 96), (('ho-wikidata-l1', 20), 84)]:
    n,c = plan[k]
    x = draws(own[k], n, 'own') + L2
    print(k, 'locked n,c,power', n,c, (x>c).mean(), flush=True)
    limits = [np.quantile(draws(m, nxt, 'own'), 1-.048/40) for m in (own[k],pooled[k[0]],allres)]
    nc = math.ceil(10000*max(limits))/10000
    v = draws(own[k], nxt, 'own')
    print(k, 'next grid n,c,power, own-size', nxt,nc, (v+L2>nc).mean(), (v>nc).mean(), flush=True)
