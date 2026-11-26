"""Check Williams position balance, aggregate within rounds, use disjoint chronological clusters."""
import csv, json, math, statistics, sys
from collections import Counter, defaultdict
from pathlib import Path
here=Path(__file__).resolve().parent
configs={
    'graph':('slices/compact raw/compact raw/slices compact/direct slices/direct raw/direct',6),
    'joint':('joint_a/stream_a joint_b/stream_b stream_b/stream_a joint_b/joint_a',6),
    'original':('original/compact_stream original/joint original/raw_graph joint/compact_stream raw_graph/compact_stream',4),
}
out={}
for key,(contrasts,groups) in configs.items():
    if not (here/f'{key}.csv').is_file():continue
    q=list(csv.DictReader((here/f'{key}.csv').open()))
    assert len(q)==groups*4*4*10
    balance=Counter((int(x['cluster']),x['block'],int(x['position']),x['arm']) for x in q)
    assert len(balance)==len(q) and set(balance.values())=={1}
    phase=['total_ns','graph_ns','publish_ns','power_ns','emit_ns']
    totals=defaultdict(Counter)
    for x in q:
        r=int(x['round']);assert int(x['cluster'])==r//4
        for p in phase:totals[r,x['arm']][p]+=int(x[p])
    arms=sorted({x['arm'] for x in q})
    med={arm:{p:round(statistics.median(totals[r,arm][p] for r in range(groups*4))/1e6,6) for p in phase} for arm in arms}
    ratios={}
    for cmp in contrasts.split():
        a,b=cmp.split('/');ratios[cmp]={}
        for p in phase:
            if any(not totals[r,a][p] or not totals[r,b][p] for r in range(groups*4)):continue
            logs=[math.log(totals[r,a][p]/totals[r,b][p]) for r in range(groups*4)]
            clusters=[statistics.mean(logs[r:r+4]) for r in range(0,len(logs),4)]
            mu=statistics.mean(clusters);hw=({6:2.5705818366,4:3.1824463053}[groups])*statistics.stdev(clusters)/math.sqrt(groups)
            ratios[cmp][p]=[round(math.exp(v),6) for v in (mu,mu-hw,mu+hw)]
    out[key]={'rows':len(q),'rounds':groups*4,'ten_block_stage_median_ms':med,'ratios_95':ratios}
for group in ('rss','raw-rss'):
    mem={}
    for p in sorted(here.glob(f'{group}-[0-9].csv')):
        r=list(csv.DictReader(p.open()))
        if not r:continue
        assert len(r)==10
        oracle={x['block']:(x['output_bytes'],x['fnv64']) for x in r}
        if mem:assert oracle==previous
        previous=oracle
        mem[r[0]['arm']]=max(int(x['process_hwm_kib']) for x in r)
    if mem:out[group]=mem
print(json.dumps(out,indent=2))
