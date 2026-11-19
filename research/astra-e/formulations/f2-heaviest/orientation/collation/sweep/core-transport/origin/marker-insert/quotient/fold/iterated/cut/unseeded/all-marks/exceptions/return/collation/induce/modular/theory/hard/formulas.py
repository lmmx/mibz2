"""Check closed-form predictions against the donor's raw records."""
import csv
import json
from math import gcd

with open('counts.csv') as src:
    rows=list(csv.DictReader(src))
single_layers=whole_towers=expanded=0
for raw in rows:
    family=raw.pop('family')
    d={k:int(v) for k,v in raw.items()}
    q,m,n,b=[d[k] for k in ('q','m','n','offset')]
    c=q-1
    assert d['lap']==n//gcd(n,b)
    assert d['first_partial']==0 and d['first_final_bands']==2
    if family in ('primitive_q','periodic_q','square_periodic'):
        primitive=family=='primitive_q'
        assert d['first_pieces']==2*q and d['first_strict']==c*(m-2)
        assert d['first_equal']==c*(1+int(primitive)) and d['first_self']==c*int(not primitive)
    else:
        assert d['first_pieces']==q+1 and d['first_strict']==d['first_equal']==c
        assert d['first_self']==0
    single_layers+=1
    if d['lap']==n:assert d['live_joins']==d['all_joins']==d['total_contractions']
    s=0;a=m
    while a%q==0:a//=q;s+=1
    if a==1:
        whole_towers+=1
        if family=='primitive_q':
            contracts=c*(m+s)+1;strict=c*(m-2);all_joins=live=contracts
        if family=='primitive_one':
            contracts=c*(2*s+1);strict=c*s;all_joins=live=contracts
        if family in ('periodic_q','square_periodic'):
            stages=1 if family=='periodic_q' else s
            live=c*(2*s-1);contracts=stages*c*m+live
            strict=stages*c*(m-2)+c*(s-1);all_joins=stages*c*(m-1)+live
        assert (contracts,strict,all_joins,live)==tuple(d[k] for k in
            ('total_contractions','total_strict','all_joins','live_joins')),d
    expanded+=n
print(json.dumps(dict(exact_first_layers=single_layers,exact_whole_towers=whole_towers,
                      directly_checked_emitted_bytes=expanded),indent=2))
