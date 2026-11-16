"""Untimed arithmetic replay of archived cap experiments; stdlib only.

Run alongside extracted/ and occupancy/ after verifying downloads against the
SHA-256 manifest. Confidence limits reproduce ab.rs: paired log ratios, t9=2.262.
Cross-file forced/judged comparisons are descriptive, not randomized pairs.
"""
import json, math, pathlib, statistics

ROOT = pathlib.Path(__file__).parent

def read(path):
    rows = [json.loads(line) for line in path.read_text().splitlines()]
    assert all(r['termination'] == 'success' and not r.get('err') for r in rows)
    for corpus in {r['corpus'] for r in rows}:
        assert len({(r['out'], r['hash']) for r in rows if r['corpus'] == corpus}) == 1
    return rows

def compare(rows, base_rows, corpus, threads, arm, base):
    a = {r['round']: r for r in rows if (r['corpus'], r['threads'], r['arm']) == (corpus, threads, arm)}
    b = {r['round']: r for r in base_rows if (r['corpus'], r['threads'], r['arm']) == (corpus, threads, base)}
    assert set(a) == set(b) == set(range(10))
    x = [math.log(a[k]['wall'] / b[k]['wall']) for k in sorted(a)]
    mean = statistics.mean(x)
    half = 2.262 * statistics.stdev(x) / math.sqrt(len(x))
    return dict(corpus=corpus, threads=threads, arm=arm, reference=base,
                ratio=math.exp(mean), low=math.exp(mean-half), high=math.exp(mean+half))

out = []
for p in sorted(ROOT.glob('occupancy/*/runs.jsonl')):
    rows = read(p)
    for corpus in ['wikidata', 'wikidata-x8']:
        out.append(dict(bundle=p.parent.name, **compare(rows, rows, corpus, 20, 'mibz2-workers-lazy-cap', 'mibz2-workers-lazy')))
for p in sorted(ROOT.glob('extracted/*/*/*/runs.jsonl')):
    rows = read(p)
    capped = any(r['arm'] == 'mibz2-workers-lazy-cap' for r in rows)
    for n in [12, 14, 16, 20]:
        out.append(dict(bundle=p.parent.name, **compare(rows, rows, 'wikidata', n,
            'mibz2-workers-lazy-cap' if capped else 'mibz2-workers-lazy',
            'mibz2-workers-lazy' if capped else 'mibz2')))
for p in sorted(ROOT.glob('extracted/stop-walk/*/*/forced.jsonl')):
    rows, judged = read(p), read(p.parent / 'judged.jsonl')
    for n in [12, 20]:
        for arm in sorted({r['arm'] for r in rows}):
            out.append(dict(bundle=p.parent.name, descriptive_cross_phase=True,
                **compare(rows, judged, 'wikidata-x4', n, arm, 'mibz2')))
print(json.dumps(out, indent=2))
