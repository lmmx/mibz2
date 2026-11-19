"""Read the direct raw scorer observations and check paired label balance."""
from collections import Counter
import json
import math
from pathlib import Path
import statistics
import sys

root = Path(sys.argv[1])
rows = [json.loads(line) for line in (root / 'runs.jsonl').read_text().splitlines()]
assert len(rows) == 128
index = {(r['round'], r['label']): r for r in rows}
assert len(index) == 128
positions = Counter((r['position'],r['label']) for r in rows)
assert len(positions) == 16 and set(positions.values()) == {8}
pred = Counter((x['label'], y['label']) for x,y in zip(rows,rows[1:]) if x['round'] == y['round'])
assert len(pred) == 12 and set(pred.values()) == {8}
assert {r['out_bytes'] for r in rows} == {209700196}

def interval(logs):
    n = len(logs)
    t = {32:2.0395134464, 8:2.3646242516}[n]
    m = statistics.mean(logs)
    d = t * statistics.stdev(logs) / math.sqrt(n)
    return [math.exp(v) for v in (m,m-d,m+d)]

result = {}
for metric in ('wall_s','cpu_s','peak_rss_kb'):
    effects = {}
    for name, group1, group0 in [('singleton/original', (2,3), (0,1)),
                                 ('originalB/originalA', (1,), (0,)),
                                 ('singletonB/singletonA', (3,), (2,))]:
        logs = []
        for k in range(1,33):
            def group_mean(labels):
                return statistics.mean(math.log(index[k,label][metric]) for label in labels)
            logs.append(group_mean(group1) - group_mean(group0))
        clusters = [statistics.mean(logs[i:i+4]) for i in range(0,32,4)]
        effects[name] = dict(rounds=interval(logs), four_round_clusters=interval(clusters))
    effects['median_by_label'] = [statistics.median(index[k,label][metric] for k in range(1,33)) for label in range(4)]
    result[metric] = effects
print(json.dumps(result,indent=2))
