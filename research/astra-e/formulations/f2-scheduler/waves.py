"""Exactly simulate compulsory lookups and parent publication for immutable contracted trie shapes."""
import csv
import heapq
import lzma
import sys
from collections import Counter, defaultdict, deque
from itertools import groupby

def measure(tree,lanes):
    children=[[] for _ in tree]
    cost=[int(n["ranks"]) for n in tree]
    span=[0]*len(tree)
    event=Counter()
    for i,n in enumerate(tree):
        p=int(n["parent"])
        assert i==int(n["node"]) and cost[i]>0
        if i:
            assert p<i
            children[p].append(i)
        begin=span[p] if i else 0
        span[i]=begin+cost[i]
        event[begin]+=1
        event[span[i]]-=1
    active=0; peak=0
    for t,change in sorted(event.items()):
        active+=change
        peak=max(peak,active)

    # Baseline waits for the longest path of the batch before *any* child publication.
    ready=deque([0])
    batched=0
    while ready:
        group=[ready.popleft() for _ in range(min(lanes,len(ready)))]
        batched+=max(cost[i] for i in group)
        for i in group: ready.extend(children[i])

    # One lookup per active lane per wave. A finished parent first publishes a
    # complete path + selectors + dictionary entry; all children are ready for
    # the next wave. Tie-order matches the native ascending lane publications.
    ready=deque([0])
    free=list(range(lanes)); working=[]
    now=0; queue_peak=0
    while ready or working:
        queue_peak=max(queue_peak,len(ready))
        while free and ready:
            lane=heapq.heappop(free)
            i=ready.popleft()
            heapq.heappush(working,(now+cost[i],lane,i))
        now=working[0][0]
        while working and working[0][0]==now:
            _,lane,i=heapq.heappop(working)
            heapq.heappush(free,lane)
            ready.extend(children[i])
    work=sum(cost)
    return [lanes,len(tree),work,max(span),peak,sum(int(n["selectors"]) for n in tree),
            sum(int(n["dict_bytes"]) for n in tree),max(map(len,children)),queue_peak,
            max(max(span),(work+lanes-1)//lanes),batched,now,
            f"{work/(lanes*batched):.6f}",f"{work/(lanes*now):.6f}"]

with lzma.open(sys.argv[1],"rt") as data:
    rows=csv.DictReader(data)
    output=csv.writer(sys.stdout,lineterminator="\n")
    output.writerow("stream lanes nodes lookups critical_path infinite_peak selectors dict_bytes max_fanout queue_peak lower_waves batch_waves refill_waves batch_occupancy refill_occupancy".split())
    for stream,group in groupby(rows,key=lambda r:r["stream"]):
        tree=list(group)
        for lanes in (1,4,8,16):
            output.writerow([stream,*measure(tree,lanes)])
