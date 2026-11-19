"""Explicit sharp examples with a ternary byte lift of an arbitrary token last column."""
import json
from audit import inverse, outgoing, runs

def lift(e):
    # P=0, X_j=0(2^j)1. F rows are P, then 1P, then 21P, 221P, ...
    return tuple([1]*len(e)+[0 if j==d else 2 for d in range(max(e)+1) for j in e if j>=d])

def inspect(e,selected):
    l=lift(e); psi=sorted(range(len(l)),key=l.__getitem__); f=tuple(sorted(l))
    s=list(range(len(e))); w,t=outgoing(s,psi,f)
    labels={j:(0,)+(2,)*j+(1,) for j in set(e)}
    lf_token=inverse([t[i] for i in s])
    assert [w[lf_token[i]] for i in s]==[labels[j] for j in e]
    u=[i for i in s if w[i] in {labels[j] for j in selected}]
    uw,ut=outgoing(u,psi,f)
    back={target:source for source,target in ut.items()}
    component_by_row={}
    components=0
    for i,x in enumerate(u):
        if i==0 or x!=u[i-1]+1: components+=1
        component_by_row[x]=components-1
    ordered=[uw[back[x]] for x in u]
    zero_parts=sum((uw[x]==labels[0]) and (i==0 or uw[u[i-1]]!=labels[0]) for i,x in enumerate(u))
    return dict(tokens=len(e),original_rows=len(l),h=components,r=runs(e),t=runs(ordered),
                colored=runs((uw[back[x]],component_by_row[x]) for x in u),zero_parts=zero_parts)

results={}
for kind,base in [('run_bound',(1,1,0,0,2,2,0)),('color_cost',(1,0,2,0))]:
    rows=[]
    for m in range(1,11):
        e=tuple(3*i+c for i in range(m) for c in base)
        result=inspect(e,{3*i+a for i in range(m) for a in (0,2)})
        assert result['h']==m+1
        if kind=='run_bound': assert result['t']==result['colored']==result['r']+m
        else: assert result['colored']-result['t']==m
        rows.append(result)
    results[kind]=rows
results['fragmentation']=[]
for h in range(2,13):
    e=tuple(range(2*h-1))+(0,)*(2*h-2)
    result=inspect(e,set(range(0,2*h-1,2)))
    assert result['h']==h==result['zero_parts']
    results['fragmentation'].append(result)
print(json.dumps(results,indent=2))
