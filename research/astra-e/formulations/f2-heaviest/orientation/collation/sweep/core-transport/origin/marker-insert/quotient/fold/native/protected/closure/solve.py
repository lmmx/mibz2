"""Sparse maximum closure with arbitrary-precision integer capacities and a canonical colored reduction."""
from collections import Counter,defaultdict,deque
from itertools import product
from pathlib import Path
from time import thread_time_ns
import csv,gzip,sys

NONE=4294967295

class Network:
    def __init__(self,n):self.adj=[[] for _ in range(n)]
    def edge(self,a,b,c):
        i,j=len(self.adj[a]),len(self.adj[b])
        self.adj[a].append([b,c,j]);self.adj[b].append([a,0,i])
    def reachable(self,s):
        seen={s};queue=[s]
        for a in queue:
            for b,c,_ in self.adj[a]:
                if c and b not in seen:seen.add(b);queue.append(b)
        return seen
    def source_set(self,s,t):
        n=len(self.adj)
        while True:
            level=[-1]*n;level[s]=0;q=[s]
            for a in q:
                if a==t:continue
                for b,c,_ in self.adj[a]:
                    if c and level[b]<0:level[b]=level[a]+1;q.append(b)
            if level[t]<0:return set(q)
            cursor=[0]*n
            while True:
                trail=[s]
                while trail and trail[-1]!=t:
                    a=trail[-1];es=self.adj[a]
                    while cursor[a]<len(es):
                        b,c,_=es[cursor[a]]
                        if c and level[b]==level[a]+1:trail.append(b);break
                        cursor[a]+=1
                    else:
                        level[a]=-1;trail.pop()
                        if trail:cursor[trail[-1]]+=1
                if not trail:break
                delta=min(self.adj[a][cursor[a]][1] for a in trail[:-1])
                for a in trail[:-1]:
                    e=self.adj[a][cursor[a]];e[1]-=delta;self.adj[e[0]][e[2]][1]+=delta


def colors(adj,first,omit=None):
    color={first:0};q=[first];odd=None
    for x in q:
        for y in adj[x]:
            if omit and ((x,y)==omit or (y,x)==omit):continue
            if y not in color:color[y]=1-color[x];q.append(y)
            elif color[y]==color[x]:odd=x,y
    return color,odd


def conditional(color,adj,g,cost,reward,pinned):
    used={i for e in reward for i in e}
    fixed={i:0 for i in color if i not in used}
    if pinned is not None:
        a,value=pinned
        if a in fixed and value:return None
        fixed[a]=value
        if value:
            for j in adj[a]:fixed[j]=0
    variables=[i for i in color if i not in fixed]
    ids={k:i for i,k in enumerate(variables)}
    scale=sum(cost.get(k,0) for k in color)+1
    unary=[cost[i] if color[i] else -cost[i] for i in variables]
    implies=[]
    for a in variables:
        b=g[a]
        if b not in ids:continue
        assert color[a]!=color[b]
        implies.append((ids[b],ids[a]) if color[a] else (ids[a],ids[b]))
    for e,mult in reward.items():
        if any(fixed.get(i)==0 for i in e):continue
        e=[i for i in e if i not in fixed]
        if not e:continue
        flip=color[e[0]];assert all(color[i]==flip for i in e)
        value=scale*mult*(-1 if flip else 1)
        if len(e)==1:unary[ids[e[0]]]+=value;continue
        aux=len(unary);unary.append(value)
        implies.extend((ids[i],aux) if flip else (aux,ids[i]) for i in e)
    s,t=len(unary),len(unary)+1;flow=Network(t+1);inf=sum(max(0,v) for v in unary)+1
    for i,v in enumerate(unary):
        if v>0:flow.edge(s,i,v)
        elif v<0:flow.edge(i,t,-v)
    for i,j in implies:flow.edge(i,j,inf)
    picked=flow.source_set(s,t)
    selected={k for k,value in fixed.items() if value}
    selected.update(k for k in variables if (ids[k] in picked)!=bool(color[k]))
    assert all(g[i] not in selected for i in selected)
    score=sum(w for e,w in reward.items() if all(i in selected for i in e))
    return (score,-sum(cost[i] for i in selected)),selected,(len(unary),len(implies))


def solve(g,cost,reward):
    adj=defaultdict(set)
    for i,j in g.items():adj[i].add(j);adj[j].add(i)
    for k in adj:adj[k]=tuple(sorted(adj[k]))
    local=defaultdict(Counter)
    for e,n in reward.items():
        assert e and all(k in g for k in e)
        j=g[e[0]];assert all(g[k]==j for k in e)
        local[j][e]+=n
    selected=set();visited=set();total=[0,0];stats=Counter()
    for seed in sorted(adj):
        if seed in visited:continue
        color,odd=colors(adj,seed);visited.update(color)
        parts=Counter()
        for i in color:parts.update(local.get(i,{}))
        if not parts:continue
        stats['components']+=1
        if odd:
            a,b=odd
            if g.get(a)!=b:a,b=b,a
            assert g[a]==b
            color,remaining=colors(adj,seed,(a,b));assert remaining is None
            opts=[conditional(color,adj,g,cost,parts,(a,v)) for v in (0,1)]
            stats['odd']+=1
        else:opts=[conditional(color,adj,g,cost,parts,None)]
        opts=[o for o in opts if o is not None]
        for _,_,(n,m) in opts:stats['flow_vars']+=n;stats['implications']+=m
        val,take,_=max(opts,key=lambda o:o[0]);selected.update(take)
        total[0]+=val[0];total[1]+=val[1]
    assert tuple(total)==(sum(w for e,w in reward.items() if all(i in selected for i in e)),-sum(cost[i] for i in selected))
    assert all(g[k] not in selected for k in selected)
    return tuple(total),selected,stats


def small():
    cases=0;odds=0
    for n in range(1,6):
        for arcs in product(range(n),repeat=n):
            g={i:a for i,a in enumerate(arcs) if a!=i}
            c={i:1+(i*3+a*7)%13 for i,a in g.items()}
            groups=defaultdict(list)
            for i,j in g.items():groups[j].append(i)
            for mode in range(2):
                reward=Counter()
                for j,kids in groups.items():
                    for i in kids:
                        if mode or i%2:reward[(i,)]+=1+(i+j)%3
                    if len(kids)>1:reward[tuple(kids)]+=2+mode
                best=(0,0)
                for mask in range(1<<n):
                    x={i for i in range(n) if mask>>i&1}
                    if not x<=g.keys() or any(g[i] in x for i in x):continue
                    v=(sum(w for e,w in reward.items() if set(e)<=x),-sum(c[i] for i in x))
                    best=max(best,v)
                val,_,ss=solve(g,c,reward);assert val==best,(arcs,mode,val,best)
                cases+=1;odds+=ss['odd']
    print(f'{cases} graph/reward instances matched both brute objectives; {odds} rewarded odd components checked.')


def run(src,folder):
    f=gzip.open(src,'rt') if src.suffix=='.gz' else src.open()
    blocks={}
    with f:
        rd=csv.reader(f);next(rd)
        for kind,s,*vals in rd:
            idx=int(s);types,hyper=blocks.setdefault(idx,({},Counter()))
            vs=tuple(map(int,vals))
            if kind=='T':types[vs[0]]=vs[1:]
            else:assert kind=='S';hyper[vs]+=1
    with (folder/'policy.csv').open('w') as p,(folder/'solver.csv').open('w') as out:
        writer=csv.writer(p,lineterminator='\n');writer.writerow(('stream','selected_source_base'))
        w=csv.writer(out,lineterminator='\n');w.writerow(('stream','old_types','eligible','eligible_states','unique_supports','unique_incidence','state_incidence',
          'selected','removed_rows','optimal_saved_q','minimum_appended','components','odd_components','flow_variables','implications','solve_cpu_ns'))
        for idx,(ts,hs) in blocks.items():
            g={i:a[1] for i,a in ts.items() if a[1]!=NONE};cost={i:ts[i][2] for i in g}
            start=thread_time_ns();val,chosen,stats=solve(g,cost,hs);elapsed=thread_time_ns()-start
            for i in sorted(chosen):writer.writerow((idx,i))
            w.writerow((idx,len(ts),len(g),sum(hs.values()),len(hs),sum(map(len,hs)),sum(len(e)*v for e,v in hs.items()),
                        len(chosen),sum(ts[k][0] for k in chosen),val[0],-val[1],stats['components'],stats['odd'],
                        stats['flow_vars'],stats['implications'],elapsed))
if __name__=='__main__':
    if len(sys.argv)==1:small()
    else:run(Path(sys.argv[1]),Path(sys.argv[2]))
