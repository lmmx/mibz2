"""Exhaust tiny marked columns; price every syntax node in the quadratic two-run powers."""
from collections import Counter
from itertools import product
from pathlib import Path
import json
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[5]))
from clock import index
from trajectory import cascade,induce,runs,spell


def small():
    count=Counter()
    for alphabet,maximum in ((2,7),(3,4)):
        for n in range(1,maximum+1):
            for L in product(range(alphabet),repeat=n):
                F,psi,lf=index(L)
                arcs=runs(L)
                for p in range(n):
                    raw=[]
                    z=p
                    while True:
                        raw.append(L[z]);z=lf[z]
                        if z==p:break
                    for q in (2,3,4,8):
                        logs,gr,live,length,root=cascade(arcs,p,q)
                        assert spell(gr,root)==bytes(raw)
                        expected=[];z=p
                        for _ in range(n):
                            expected.append(F[z]);z=psi[z]
                        forward=spell(gr,root)[::-1]
                        assert (forward*((n+len(forward)-1)//len(forward)))[:n]==bytes(expected)
                        assert len(live)<=length-1 and length==len(raw)
                        count['all_marked_radices']+=1
                        count['short_laps']+=length<n
                        count['live_joins']+=len(live)
                        count['dead_joins']+=len(gr)-len(live)
                        m=n
                        for j,r in enumerate(logs):
                            assert r['partial']==0
                            assert r['survivors']==r['progressions']-r['equal']-r['fixed']
                            assert r['progressions']<=min(m,q**(j+1)*len(arcs))
                            assert r['n']==m;m=r['kept']
    return dict(count)


def powers():
    records=[]
    for q in (2,3,4,8):
        for v in range(1,9):
            g=q**v;t=g+1;n=t*g
            if t>8193:break
            # The first arc maps forward q in the base and is multiplied by g in each axis.
            base=[(0,q,t-q,1),(t-q,0,q,0)]
            full=[(g*x,g*y,g*w,b) for x,y,w,b in base]
            discard=[]
            _,one=induce(base,0,discard)
            mark=g//2
            stages,gr,live,lap,root=cascade(full,mark,q)
            bst,bgr,blive,blap,broot=cascade(base,0,q)
            assert lap==blap==t
            assert spell(gr,root)==spell(bgr,broot)
            assert all(r['joins']==(q-1)*len(discard) for r in stages[:v])
            assert sum(r['joins'] for r in stages[v:])==len(bgr)
            first_base=sum(r['joins'] for r in stages[:v])
            assert all(id>=256+first_base for id in live)
            assert len(live)==len(blive)==len(bgr)>=max(0,(t-1)//q-1)
            assert stages[v]['n']==t
            assert len(discard)>=max(0,(t-q-1)//q)
            records.append(dict(q=q,v=v,base_t=t,g=g,n=n,base_to_zero_rules=len(discard),
                early_dead_rules=first_base,remaining_live_rules=len(live),created=len(gr),
                progressions=sum(z['progressions'] for z in stages),depth=len(stages)))
    return records


if __name__=='__main__':print(json.dumps(dict(tiny=small(),quadratic_powers=powers()),indent=2))
