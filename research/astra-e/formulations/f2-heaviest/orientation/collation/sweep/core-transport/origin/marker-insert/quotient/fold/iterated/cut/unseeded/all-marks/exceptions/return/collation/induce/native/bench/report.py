"""Verify original clocks, arm order, all stamps and pre/post cap-only process HWM."""
import math
from collections import Counter
from pathlib import Path
import statistics as st
import sys
from run import OLD,NEW,PHASE,SHAPE,dump,read

def ratio(a,b):
    logs=[math.log(x/y) for x,y in zip(a,b,strict=True)]
    assert len(logs)==24
    means=[st.mean(logs[i:i+6])for i in range(0,24,6)]
    center=st.mean(means);half=3.1824463052843*st.stdev(means)/2
    return [round(math.exp(center+f*half),6)for f in (0,-1,1)]

def main(d):
    data,raw,calls,rss=[read(d/s)for s in ('paired.csv','times.csv','stamps.csv','rss.csv')]
    assert len(data)==len(raw)==1440 and len(calls)==750
    assert len(read(d/'gate-stamps.csv'))==30
    assert len({(r['round'],r['block'],r['arm'])for r in data})==1440
    assert set(Counter((r['slot'],r['block'],r['arm'])for r in data).values())=={4}
    block_ids=sorted({int(r['block'])for r in data});assert len(block_ids)==10
    order=[]
    for rnd in range(1,25):
        for j in range(10):
            b=block_ids[(j+rnd)%10]
            for slot in range(6):
                offset=slot if (rnd//6)%2==0 else 5-slot
                order.append((str(rnd),str(b),str(slot),NEW[(rnd+b+offset)%6]))
    assert order==[(v['round'],v['block'],v['slot'],v['arm']) for v in data]
    it=iter(calls[30:])
    for r,old in zip(data,raw,strict=True):
        a=OLD.index(old['arm'])
        assert r['arm']==NEW[a] and all(r[k]==old[k]for k in old if k!='arm')
        if a<3:
            s=next(it);assert int(s['arm'])==a
            assert [s[k]for k in ('n','primary','input')]==[r[k]for k in ('rows','mark','cap_records')]
            assert all(r[k]==s[k]for k in PHASE+SHAPE)
    assert not list(it)
    result={};dimensions={}
    for where in ['all',*map(str,block_ids)]:
        tally=Counter()
        for r in data:
            if where!='all' and where!=r['block']:continue
            a=r['arm'];round_id=int(r['round'])
            for k in ['thread_ns',*PHASE]:
                if r.get(k):tally[a,round_id,k]+=int(r[k])
            if a in NEW[:3]:
                tally[a,round_id,'other_ns']+=int(r['thread_ns'])-sum(int(r[k])for k in PHASE)
        sample={a:[tally[a,i,'thread_ns']for i in range(1,25)]for a in NEW}
        result[where]=dict(median_ms={a:round(st.median(seq)/1e6,6)for a,seq in sample.items()},
            paired_ratios={f'{a}/{b}':ratio(sample[a],sample[b])for a in NEW[:3]for b in NEW[3:]},
            phase_median_ms={a:{k:round(st.median(tally[a,i,k]for i in range(1,25))/1e6,6)for k in [*PHASE,'other_ns']}for a in NEW[:3]})
        if where!='all':
            dimensions[where]={}
            for a in NEW[:3]:
                vs=[r for r in data if r['block']==where and r['arm']==a]
                counts={}
                for k in SHAPE[:-2]:
                    v={int(r[k])for r in vs};assert len(v)==1;counts[k]=v.pop()
                for k in SHAPE[-2:]:counts[k+'_max']=max(int(r[k])for r in vs)
                dimensions[where][a]=counts
    assert len(rss)==len({(r['round'],r['block'],r['arm'])for r in rss})==180
    hwm={}
    for a in NEW:
        rows=[r for r in rss if r['arm']==a]
        before=[int(r['pre_hwm_kib'])for r in rows];after=[int(r['post_hwm_kib'])for r in rows]
        growth=[hi-lo for lo,hi in zip(before,after,strict=True)]
        assert len(rows)==30 and min(growth)>=0
        hwm[a]=dict(median_kib=st.median(after),max_kib=max(after),median_growth_kib=st.median(growth),max_growth_kib=max(growth))
    dump(d/'summary.json',dict(stats=result,dimensions=dimensions,cap_only_process_hwm=hwm))

if __name__=='__main__':main(Path(sys.argv[1]))
