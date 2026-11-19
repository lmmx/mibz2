"""Check balanced fixed-horizon calls and report clustered paired log ratios."""
import csv,sys
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev

def rows(path):
    with path.open() as f:return list(csv.DictReader(f))

def report(root):
    data=rows(root/'phase.csv');arm_order=('named_a','named_b','recognize','fold')
    assert len(data)==1920
    slots=Counter();cells=set();totals=defaultdict(int)
    metrics=[k for k in data[0] if k.endswith('_ns')]
    for r in data:
        a=r['arm'];c,rep,p,s=map(int,[r['cluster'],r['rep'],r['position'],r['stream']])
        assert a in arm_order and 0<=rep<48 and c==rep//4 and p in range(4)
        assert (a,rep,s) not in cells
        cells.add((a,rep,s));slots[c,s,a,p]+=1
        v={k:int(r[k]) for k in metrics}
        v['first_ns']=v['setup_ns']+v['discover_ns']+v['merge_ns']
        v['post_root_ns']=v['total_ns']-v['first_ns']
        v['second_total_ns']=sum(v[k] for k in metrics if k.startswith('second_'))
        for k,val in v.items():totals[rep,a,k]+=val
    assert len(slots)==1920 and set(slots.values())=={1}
    metrics+=['first_ns','post_root_ns','second_total_ns']
    pairs=[('fold','named_a'),('fold','named_b'),('fold','recognize'),('recognize','named_a'),('recognize','named_b'),('named_b','named_a')]
    with (root/'clusters.csv').open('w') as f:
        wr=csv.writer(f,lineterminator='\n');wr.writerow(('cluster','arm','reference','metric','mean_log_ratio','ratio'))
        for a,b in pairs:
            for k in metrics:
                vals=[(totals[rep,a,k],totals[rep,b,k]) for rep in range(48)]
                if any(x==0 or y==0 for x,y in vals):continue
                ratios=[log(x/y) for x,y in vals]
                clusters=[mean(ratios[i:i+4]) for i in range(0,48,4)]
                for i,v in enumerate(clusters):wr.writerow((i,a,b,k,v,exp(v)))
                center=mean(clusters);e=2.200985160082949*stdev(clusters)/sqrt(12)
                print(f'{a}/{b} {k} {exp(center):.5f} [{exp(center-e):.5f},{exp(center+e):.5f}]')
    for a in arm_order:
        print(a,'ten-block arithmetic mean ms',{k:round(mean(totals[i,a,k] for i in range(48))*1e-6,4) for k in metrics})
    for src,n in [('ten.csv',10),('admissions.csv',228)]:
        rr=rows(root/src);assert len(rr)==n*len(arm_order)
        assert len({(r['arm'],r['stream']) for r in rr})==len(rr)
        for a in arm_order:
            arm=[r for r in rr if r['arm']==a];assert len(arm)==n
            print(src,a,'sums',{k:sum(int(r[k]) for r in arm) for k in arm[0] if k not in
                ['stream','arm','capacity','second']},'stock second',sum(r['second']=='true' for r in arm))
    rr=rows(root/'rss.csv');assert len(rr)==228*4
    for a in arm_order:
        samples=[r for r in rr if r['arm']==a];assert len({r['stream'] for r in samples})==228
        print(a,'max retained bytes',max(int(r['retained_bytes']) for r in samples),'HWM KiB',max(int(r['hwm_kib']) for r in samples))

if __name__=='__main__':report(Path(sys.argv[1]))
