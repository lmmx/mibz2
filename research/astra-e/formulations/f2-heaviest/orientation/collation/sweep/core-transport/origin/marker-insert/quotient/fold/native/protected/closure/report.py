import csv,sys
from collections import defaultdict,Counter
from pathlib import Path
from statistics import mean,stdev
from math import exp,log,sqrt

def data(p):
    with p.open() as f:return list(csv.DictReader(f))

def main(d):
    price=data(d/'census-price.csv');ss=data(d/'solver.csv');act=data(d/'apply.csv');fresh=data(d/'fresh.csv')
    assert len(price)==120 and len(ss)==len(act)==10 and len(fresh)==160
    canon={int(r['stream']):r for r in act};assert len(canon)==10
    ids={int(r['stream']) for r in ss};assert ids==canon.keys()
    for s in ss:
        a=canon[int(s['stream'])]
        assert int(a['old_q'])-int(a['new_q'])==int(s['optimal_saved_q'])
        assert int(s['minimum_appended'])==int(a['appended'])
        assert a['selected']==s['selected'] and a['removed']==s['removed_rows']
        assert a['old_dictionary']==a['new_dictionary']
    assert len({(r['rep'],r['stream']) for r in price})==120
    census=defaultdict(int)
    for r in price:
        assert 0<=int(r['rep'])<12 and int(r['stream']) in ids
        for k in r:
            if k not in ('rep','stream'):census[int(r['rep']),k]+=int(r[k])
    for k in price[0]:
        if k not in ('rep','stream'):
            x=mean(census[i,k] for i in range(12)); print('census',k,round(x/1e6,5) if k.endswith('_ns') else x)
    print('solver totals',{k:sum(int(r[k]) for r in ss) for k in ss[0] if k not in ('stream','solve_cpu_ns')})
    print('application totals',{k:sum(int(r[k]) for r in act) for k in act[0] if k!='stream'})
    cells=set();orders=Counter();tot=defaultdict(int)
    for r in fresh:
        a=r['arm'];s=int(r['stream']);i=int(r['rep']);p=int(r['position'])
        assert 0<=i<8 and s in ids and a in ['named','fresh'] and p in (0,1)
        assert (i,s,a) not in cells;cells.add((i,s,a));orders[s,a,p]+=1
        if a=='fresh':
            z=canon[s];assert int(r['saved_q'])==int(z['old_q'])-int(z['new_q']) and r['appended']==z['appended']
        for k in r:
            if k.endswith('_ns'):tot[i,a,k]+=int(r[k])
        tot[i,a,'both_cpu_ns']+=int(r['caller_cpu_ns'])+int(r['solver_cpu_ns'])
    assert set(orders.values())=={4} and len(orders)==40
    for a in ('named','fresh'):
        print(a,'mean ten-block ms',{k:round(mean(tot[i,a,k] for i in range(8))/1e6,5) for k in
            [k for k in fresh[0] if k.endswith('_ns')]+['both_cpu_ns']})
    for k in ('elapsed_ns','both_cpu_ns','caller_cpu_ns'):
        ratios=[log(tot[i,'fresh',k]/tot[i,'named',k]) for i in range(8)]
        center=mean(ratios);e=2.3646242515927844*stdev(ratios)/sqrt(8)
        print(k,'paired ratio',round(exp(center),4),round(exp(center-e),4),round(exp(center+e),4))
if __name__=='__main__':main(Path(sys.argv[1]))
