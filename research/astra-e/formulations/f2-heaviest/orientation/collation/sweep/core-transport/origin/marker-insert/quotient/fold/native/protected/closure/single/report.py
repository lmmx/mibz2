import csv,sys
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev

ARMS=('named_a','named_b','recognize','literal')
def load(f):
    with f.open() as s:return list(csv.DictReader(s))

def full(d):
    split=load(d/'split.csv')
    assert len(split)==238
    test={int(r['stream']) for r in split if r['split']=='heldout'}
    timed={int(r['stream']) for r in split if r['timed']=='1'}
    assert len(timed)==12 and timed<=test and len(test)==219
    for name,ids in [('training',None),('heldout',test),('panel',timed)]:
        rows=load(d/(name+'.csv'));assert len(rows)==(10 if ids is None else len(ids))
        if ids:assert {int(r['stream']) for r in rows}==ids
        for r in rows:
            assert r['old_dictionary']==r['new_dictionary'] and r['old_links']==r['new_links']
            assert int(r['new_q'])<=int(r['old_q'])
        sums={k:sum(int(r[k]) for r in rows) for k in ('rows','old_quote','removed','old_runs','new_runs','old_q','new_q','old_dictionary','old_links')}
        saved=sums['old_q']-sums['new_q']
        if name=='training':assert saved==3808
        print(name,'blocks',len(rows),'reasons',dict(Counter(r['reason'] for r in rows)),'moved',sum(r['moved']=='true' for r in rows),sums,'saved',saved)
    phase=load(d/'phase.csv');assert len(phase)==48*12*4
    totals=defaultdict(int);slots=Counter();cells=set()
    metrics=[k for k in phase[0] if k.endswith('_ns')]
    for r in phase:
        a=r['arm'];rep,cl,i,pos=map(int,(r['rep'],r['cluster'],r['stream'],r['position']))
        assert a in ARMS and rep in range(48) and cl==rep//4 and i in timed and pos in range(4)
        cell=(rep,i,a);assert cell not in cells;cells.add(cell);slots[cl,i,a,pos]+=1
        v={k:int(r[k]) for k in metrics}
        v['first_ns']=v['setup_ns']+v['discover_ns']+v['merge_ns']
        v['post_root_ns']=v['total_ns']-v['first_ns']
        v['index_shift_ns']=v['direct_index_ns']+v['stock_index_mark_ns']
        v['second_ns']=v['index_shift_ns']+sum(v[k] for k in ('second_discover_ns','second_merge_ns','second_emit_ns'))
        for k,vv in v.items():totals[rep,a,k]+=vv
    assert len(slots)==48*12*4 and set(slots.values())=={1}
    metrics+=['first_ns','post_root_ns','index_shift_ns','second_ns']
    with (d/'clusters.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(('cluster','arm','reference','metric','mean_log_ratio','ratio'))
        for a,b in [('literal','named_a'),('literal','named_b'),('literal','recognize'),('recognize','named_a'),('recognize','named_b'),('named_b','named_a')]:
            for k in metrics:
                vals=[(totals[i,a,k],totals[i,b,k]) for i in range(48)]
                if any(min(v)==0 for v in vals):continue
                logs=[log(x/y) for x,y in vals]
                clusters=[mean(logs[i:i+4]) for i in range(0,48,4)]
                for idx,x in enumerate(clusters):w.writerow((idx,a,b,k,x,exp(x)))
                center=mean(clusters);e=2.200985160082949*stdev(clusters)/sqrt(12)
                print(a+'/'+b,k,f'{exp(center):.5f} [{exp(center-e):.5f},{exp(center+e):.5f}]')
    for a in ARMS:
        print(a,'mean twelve-block ms',{k:round(mean(totals[i,a,k] for i in range(48))*1e-6,4) for k in metrics})
    rss=load(d/'rss.csv');assert len(rss)==228*4
    for a in ARMS:
        r=[x for x in rss if x['arm']==a];assert len({x['stream'] for x in r})==228
        print(a,'max retained bytes',max(int(x['retained_bytes']) for x in r),'HWM KiB',max(int(x['hwm_kib']) for x in r))
if __name__=='__main__':full(Path(sys.argv[1]))
