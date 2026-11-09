import json
from collections import deque
def explore(T,K,N):
    L=T; F=T+L+K; B=2*T
    # jobs, preparing, prepared, teams, walked, finishing, fronts, backs, retired
    start=(N,0,0,0,0,0,F,B,0)
    seen={start}; q=deque([start]); peak=0; terminal=0; dead=[]
    while q:
        s=q.popleft()
        j,a,p,t,w,f,ff,bb,r=s
        assert ff+a+p+t==F
        assert bb+t+w+f==B
        assert j+a+p+t+w+f+r==N
        assert a+f<=K and t<=T
        assert p<=L+K-1
        peak=max(peak,p)
        nxt=[]
        def add(*v): nxt.append(v)
        if K>a+f:
            can=j>0 and ff>0
            if can and p==0:
                add(j-1,a+1,p,t,w,f,ff-1,bb,r)
            elif w:
                add(j,a,p,t,w-1,f+1,ff,bb,r)
            elif can and p<L:
                add(j-1,a+1,p,t,w,f,ff-1,bb,r)
        if p and bb and t<T:
            add(j,a,p-1,t+1,w,f,ff,bb-1,r)
        if a:
            add(j,a-1,p+1,t,w,f,ff,bb,r)
            add(j,a-1,p,t,w,f,ff+1,bb,r+1) # prepare error/cancel
        if t:
            add(j,a,p,t-1,w+1,f,ff+1,bb,r)
            add(j,a,p,t-1,w,f,ff+1,bb+1,r+1) # team error/cancel
        if f:
            add(j,a,p,t,w,f-1,ff,bb+1,r+1) # finish success/error/cancel
        if not nxt:
            if r==N: terminal+=1
            else: dead.append(s)
        for v in nxt:
            if v not in seen: seen.add(v);q.append(v)
    return dict(teams=T,services=K,jobs=N,states=len(seen),peak_prepared=peak,bound=L+K-1,deadlocks=dead,terminal_states=terminal)
rows=[explore(t,k,n) for t in range(1,4) for k in range(1,4) for n in range(1,9)]
assert not any(x["deadlocks"] for x in rows)
# T=L=2,K=2, no teams scheduled:
# two starts at p=0, one completion p=1, third start at p=1<L,
# two completions give p=3>L.
witness=[dict(action="initial",preparing=0,prepared=0),
dict(action="service A starts",preparing=1,prepared=0),
dict(action="service B starts",preparing=2,prepared=0),
dict(action="A completes",preparing=1,prepared=1),
dict(action="A starts third",preparing=2,prepared=1),
dict(action="B completes",preparing=1,prepared=2),
dict(action="A completes",preparing=0,prepared=3)]
out=dict(model="Count-state abstraction of hub transitions; all jobs fed/closed; atomic mutex actions, finite stage completion; not Rust, weak-memory, Condvar, ordering or fairness verification",
states=sum(x["states"] for x in rows),shapes=rows,lookahead_counterexample=witness,
wake_test_escape="With notify removed, service worker not scheduled until after team's pop: jobs.len()==1 before pop still holds; worker subsequently observes p=0 and processes the job without ever waiting. Finite yields do not assert parked state. Source-derived schedule, not an executed Rust mutant.",
cpu_saving_percent=100*(1-1/1.146),cpu_saving_interval_percent=[100*(1-1/x) for x in [1.123,1.170]])
print(json.dumps(out,indent=2))
