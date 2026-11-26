"""Fixed-service bounded ordered pipeline, using integer nanoseconds throughout."""
import heapq
from dataclasses import dataclass

@dataclass(frozen=True)
class Jobs:
    scan: list
    work: list
    copy: list
    symbols: list

def bounds(a,w,cap):
    """Lower bound every enqueue and ordered take by induction, scaled by w."""
    enq=read=work=release_minus_prefix=0
    takes=[]
    for i,(s,d) in enumerate(zip(a.scan,a.work)):
        enq=max(enq,takes[i-cap] if i>=cap else 0)+w*s
        release_minus_prefix=max(release_minus_prefix,enq-work)
        work+=d
        read=max(read+(w*a.copy[i-1] if i else 0),enq+w*d,release_minus_prefix+work)
        takes.append(read)
    return (read+w*a.copy[-1]+w-1)//w

def choose(q,policy,a):
    if policy=='fifo': return q[0]
    if policy=='lpt': return max(q,key=lambda i:(a.work[i],-i))
    if policy=='symbols': return max(q,key=lambda i:(a.symbols[i],-i))
    if policy=='old4-lpt': return max(q[:4],key=lambda i:(a.work[i],-i))
    if policy=='old4-symbols': return max(q[:4],key=lambda i:(a.symbols[i],-i))
    return policy(q)

def simulate(a,w,cap,policy,retain=False):
    assert 1<=w<=cap and len(a.scan)==len(a.work)==len(a.copy)==len(a.symbols)
    assert min(a.scan+a.work+a.copy)>0
    n=len(a.work); finished=[False]*n; ready=[]; timeline=[]
    if policy in ('slack-symbols','slack-oracle'):
        estimates=a.symbols if policy=='slack-symbols' else a.work
        from itertools import accumulate
        # The first h entries are enqueued; all later prefix values stay inaccessible.
        prefix=list(accumulate(estimates,initial=0))
    else:
        estimates=None
    events=[]; t=0; next_job=0; taken=0; free=w; scanning=reading=False
    alt=changed=maxjobs=0
    while True:
        if not reading and taken<n and finished[taken]:
            reading=True
            heapq.heappush(events,(t+a.copy[taken],'copy',taken))
            taken+=1
        if not scanning and next_job<n and next_job-taken<cap:
            scanning=True
            heapq.heappush(events,(t+a.scan[next_job],'scan',next_job))
            next_job+=1
        while free and ready:
            if estimates is None:
                i=choose(ready,policy,a)
            else:
                released=next_job-int(scanning)
                # Integer comparison of i - w * cost_i / prefix_mean.
                i=min(ready,key=lambda j:(j*prefix[released]-w*estimates[j]*released,j))
            alt+=len(ready)>1
            changed+=i!=ready[0]
            maxjobs=max(maxjobs,len(ready))
            free-=1;ready.remove(i)
            heapq.heappush(events,(t+a.work[i],'work',i))
            if retain: timeline.append((t,i))
        if not events:
            assert free==w and taken==n
            break
        t=events[0][0]
        while events and events[0][0]==t:
            _,kind,i=heapq.heappop(events)
            if kind=='copy': reading=False
            elif kind=='scan': scanning=False;ready.append(i)
            else: free+=1;finished[i]=True
    return t,alt,changed,maxjobs,timeline

def fifo_independent(a,w,cap):
    free=[0]*w
    enq=0;takes=[]
    for i,s in enumerate(a.scan):
        enq=max(enq,takes[i-cap] if i>=cap else 0)+s
        began=max(enq,heapq.heappop(free))
        ended=began+a.work[i];heapq.heappush(free,ended)
        takes.append(max(ended,takes[-1]+a.copy[i-1] if i else 0))
    return takes[-1]+a.copy[-1]
