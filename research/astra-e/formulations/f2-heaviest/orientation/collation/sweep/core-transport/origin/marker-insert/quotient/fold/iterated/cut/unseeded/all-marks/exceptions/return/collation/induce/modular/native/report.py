"""Replay the exact 24 six-way rotations and reconcile per-call copies against the modular gate."""
import csv
from collections import Counter
from pathlib import Path
import statistics as st
import math
import sys
from run import OLD,NEW,PHASE,SHAPE,read,dump,D

def ratio(a,b):
    logs=[math.log(x/y)for x,y in zip(a,b,strict=True)]
    assert len(logs)==24
    means=[st.mean(logs[i:i+6])for i in range(0,24,6)];mid=st.mean(means);half=3.1824463052843*st.stdev(means)/2
    return [round(math.exp(mid+k*half),6)for k in (0,-1,1)]
def main(path):
    raw,combined,stamps,rss=[read(path/n)for n in ('times.csv','paired.csv','stamps.csv','rss.csv')]
    assert len(raw)==len(combined)==1440 and len(stamps)==750 and len(read(path/'gate-stamps.csv'))==30
    assert set(Counter((r['arm'],r['slot'],r['block'])for r in combined).values())=={4}
    assert len(rss)==len({(r['round'],r['block'],r['arm'])for r in rss})==180
    ids=sorted({int(v['block'])for v in raw});assert len(ids)==10
    expected=[]
    for rnd in range(1,25):
        for j in range(10):
            b=ids[(j+rnd)%10]
            for slot in range(6):
                offset=slot if (rnd//6)%2==0 else 5-slot
                expected.append((rnd,b,slot,(rnd+b+offset)%6))
    geometry={}
    for row in read(D.parent/'levels.csv'):geometry[int(row['block']),int(row['q'])]=row
    cs=iter(stamps[30:])
    for r,v,key in zip(raw,combined,expected,strict=True):
        a=OLD.index(r['arm']);assert (int(r['round']),int(r['block']),int(r['slot']),a)==key
        assert v['arm']==NEW[a] and all(v[k]==r[k] for k in r if k!='arm')
        if a>=3:continue
        s=next(cs);assert int(s['arm'])==a and all(s[k]==v[k]for k in PHASE+SHAPE)
        assert [s[k]for k in ('n','mark','records')]==[v[k]for k in ('rows','mark','cap_records')]
        g=geometry[key[1],2<<a]
        for x,y in [('joins','tree_joins'),('copies','cached_copies'),('aliases','cached_bytes'),('visits','terminal_visits'),('lap','marked_lap')]:
            assert s[x]==g[y]
    assert not list(cs)
    result={}
    for scope in ('all',*map(str,ids)):
        rows=[v for v in combined if scope=='all' or v['block']==scope]
        sums=Counter()
        for v in rows:
            rnd=int(v['round']);a=v['arm']
            for k in ['thread_ns',*PHASE]:
                if v.get(k):sums[a,rnd,k]+=int(v[k])
            if a in NEW[:3]:
                sums[a,rnd,'other_ns']+=int(v['thread_ns'])-sum(int(v[k])for k in PHASE)
        samples={a:[sums[a,i,'thread_ns']for i in range(1,25)]for a in NEW}
        result[scope]=dict(times_ms={a:round(st.median(v)/1e6,6)for a,v in samples.items()},
             paired={f'{a}/{b}':ratio(samples[a],samples[b])for a in NEW[:3]for b in NEW[3:]},
             phase_ms={a:{k:round(st.median(sums[a,i,k]for i in range(1,25))/1e6,6)for k in [*PHASE,'other_ns']}for a in NEW[:3]})
    sharing={}
    for a in NEW[:3]:
        r=[x for x in combined if x['round']=='1'and x['arm']==a]
        sharing[a]={k:sum(int(x[k])for x in r) for k in ('n','runs','joins','visits','copies','aliases','repeats','pieces','steps') if k!='n'}
        sharing[a]['rows']=sum(int(x['rows']) for x in r)
    memory={}
    for a in NEW:
        r=[v for v in rss if v['arm']==a];assert len(r)==30
        h=[int(v['post_hwm_kib'])for v in r];growth=[v-int(x['pre_hwm_kib'])for v,x in zip(h,r)]
        memory[a]=dict(median_kib=st.median(h),max_kib=max(h),max_growth_kib=max(growth))
    dump(path/'summary.json',dict(results=result,sharing=sharing,fresh_process_hwm=memory))
if __name__=='__main__':main(Path(sys.argv[1]))
