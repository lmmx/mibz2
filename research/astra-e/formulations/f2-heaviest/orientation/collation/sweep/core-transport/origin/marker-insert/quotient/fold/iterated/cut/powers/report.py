"""Recheck balanced 24-round native CPU prices and separate fresh-process VmHWM."""
import csv
import json
import math
from pathlib import Path
import statistics
import sys


def main(path):
    p=Path(path)
    arms=('sparse','dense','interval','direct')
    raw=list(csv.DictReader((p/'times.csv').open()))
    assert len(raw)==96
    by={a:[] for a in arms}
    for i in range(24):
        group=raw[4*i:4*i+4]
        seq=[arms[(i+j)%4 if i%8<4 else (i+3-j)%4] for j in range(4)]
        assert [r['arm'] for r in group]==seq and all(int(r['round'])==i for r in group)
        for r in group:
            assert int(r['result_size'])==(899931 if r['arm']=='direct' else 48980)
            by[r['arm']].append(int(r['thread_ns']))
    result={}
    for a in arms:
        with (p/(a+'-rss.csv')).open() as f:
            fresh=list(csv.DictReader(f))
        assert len(fresh)==1 and fresh[0]['arm']==a and int(fresh[0]['maxrss_kib'])>0
        result[a]=dict(n=24,median_ms=statistics.median(by[a])/1e6,
                       VmHWM_KiB=int(fresh[0]['maxrss_kib']),lookups=int(fresh[0]['lookups']))
    ratios={}
    for ref in ('sparse','dense','direct'):
        logs=[math.log(x/y) for x,y in zip(by['interval'],by[ref])]
        mu=statistics.mean(logs)
        rad=2.06865761041904*statistics.stdev(logs)/math.sqrt(24)
        ratios[ref]=[math.exp(mu),math.exp(mu-rad),math.exp(mu+rad)]
    print(json.dumps(dict(arm=result,interval_over_reference=ratios),indent=2))


if __name__=='__main__':
    main(sys.argv[1])
