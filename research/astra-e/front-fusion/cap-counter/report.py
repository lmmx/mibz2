"""Summed per-round paired thread-CPU contrasts with a paired log t interval."""
import csv
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev
import sys

PLAN=[('counter_on_edge',[-1,1,0,0]),('counter_on_scalar',[0,0,-1,1]),
      ('scalar_on_edge',[-1,0,1,0]),('scalar_on_counter',[0,-1,0,1]),
      ('both_on_edge',[-1,0,0,1]),('interaction',[1,-1,-1,1])]


def report(directory):
    writer=csv.writer(sys.stdout,lineterminator='\n')
    writer.writerow(('mode','measure','population','contrast','rounds','blocks_per_round','ratio','lo95','hi95'))
    for mode,total,tcrit in [('original',40,2.0226909200367604),('meter',32,2.0395134463964077)]:
        data=list(csv.DictReader((directory/(mode+'.csv')).open()))
        assert all(d['mode']==mode for d in data)
        sample=sorted({int(r['stream']) for r in data})
        assert len(data)==total*len(sample)*4 and set(sample)>={41,56,165,270,278}
        key=lambda r:(int(r['round']),int(r['stream']),int(r['arm']))
        assert len({key(r) for r in data})==len(data)
        pos=Counter((int(r['stream']),int(r['arm']),int(r['position'])) for r in data)
        assert set(pos.values())=={total//4} and len(pos)==len(sample)*16
        if mode=='meter':
            assert all(int(r['scan_thread_cpu_ns'])>0 for r in data)
        else:
            assert all(int(r['scan_thread_cpu_ns'])==0 for r in data)
        for metric in (['worker_thread_cpu_ns','scan_thread_cpu_ns'] if mode=='meter' else ['worker_thread_cpu_ns']):
            for group in ['all','ordinary','extra']:
                rows=[r for r in data if group=='all' or r['class']==group]
                sums=defaultdict(float)
                for r in rows:
                    sums[int(r['round']),int(r['arm'])]+=int(r[metric])
                blocks=len(rows)//(total*4)
                for name,weights in PLAN:
                    logs=[sum(w*log(sums[r,k]) for k,w in enumerate(weights)) for r in range(total)]
                    point=mean(logs)
                    margin=tcrit*stdev(logs)/sqrt(total)
                    writer.writerow((mode,metric,group,name,total,blocks,exp(point),exp(point-margin),exp(point+margin)))


if __name__=='__main__':
    report(Path(sys.argv[1]))
