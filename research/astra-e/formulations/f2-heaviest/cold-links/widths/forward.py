"""Independently resolve missing cold arcs forward, by affine Psi subintervals."""
from bisect import bisect_right
from collections import Counter,deque
import heapq
from itertools import groupby

def components(points):
    groups=[]
    for x in sorted(points):
        if groups and groups[-1][1]==x:groups[-1][1]+=1
        else:groups.append([x,x+1])
    return groups

def compile_psi(last,lf,stop):
    # Reverse each original-run translation. Split its *source* at cutoff
    # boundaries first, so membership after Psi is present in the descriptor.
    cells=[];n=len(last)
    bounds={y for item in components(stop) for y in item if 0<y<n}
    start=0
    for i in range(1,n+1):
        if i<n and last[i]==last[i-1] and i not in bounds:continue
        image=lf[start]
        cells.append((image,image+i-start,start-image,last[start],start in stop))
        start=i
    cells.sort()
    assert cells[0][0]==0 and cells[-1][1]==n
    assert all(a[1]==b[0] for a,b in zip(cells,cells[1:]))
    return cells,[c[0] for c in cells]

def prefix_count(strings):
    # Number of distinct nonterminal prefixes, including empty.
    total=1;prev=b""
    for w in sorted(set(s[:-1] for s in strings)):
        i=0;n=min(len(w),len(prev))
        while i<n and w[i]==prev[i]: i+=1
        total+=len(w)-i;prev=w
    return total if strings else 0

def schedule(children,roots,weights):
    if not roots: return 0,0
    clock=0;ready=deque(range(roots));free=list(range(8));working=[]
    longest=weights.copy()
    for i,kids in enumerate(children):
        for kid in kids: longest[kid]=longest[i]+weights[kid]
    while ready or working:
        while ready and free:
            lane=heapq.heappop(free);v=ready.popleft()
            heapq.heappush(working,(clock+weights[v],lane,v))
        clock=working[0][0]
        while working and working[0][0]==clock:
            _,lane,v=heapq.heappop(working);heapq.heappush(free,lane)
            ready.extend(children[v])
    return clock,max(longest)

def advance(last,q,lf,seeds,src,labels,cut,hot,full):
    m=len(cut);E=set(range(m))-{v for v,_ in hot.values()}
    seed_to_dest={v[0]:i for i,v in enumerate(cut) if v is not None}
    assert set(seed_to_dest)==set(seeds)
    stop=set(range(q,q+m)) if full else set(seeds)
    cells,starts=compile_psi(last,lf,stop)
    st=Counter();st["table"]=len(cells)
    stack=[(a+q,b+q,a,b"",j) for j,(a,b) in enumerate(components(E))]
    children=[[] for _ in stack];weights=[0]*len(stack)
    st["entrances"]=len(stack)
    tokens=[];visited=0
    while stack:
        left,right,base,prefix,node=stack.pop()
        j=bisect_right(starts,left)-1
        st["lookups"]+=1
        while left<right:
            lo,hi,shift,byte,done=cells[j]
            end=min(right,hi);width=end-left
            spell=prefix+bytes([byte])
            st["pieces"]+=1;weights[node]+=1
            if done:
                st["terminal_spans"]+=1;visited+=width
                for t in range(width):
                    dest=left+shift+t-q if full else seed_to_dest[left+shift+t]
                    assert cut[dest] is not None and src[dest]==base+t
                    suffix=cut[dest][1]
                    want=labels[dest] if full or not suffix else labels[dest][:-len(suffix)]
                    assert want==spell
                    tokens.append(want)
                st["spell_bytes_by_span"]+=len(spell)
            else:
                nid=len(children);children[node].append(nid);children.append([]);weights.append(0)
                stack.append((left+shift,end+shift,base,spell,nid))
            base+=width;left=end;j+=1
    assert visited==len(E)
    st["rank_waves8"],st["rank_depth"]=schedule(children,st["entrances"],[1]*len(children))
    st["piece_waves8"],st["piece_depth"]=schedule(children,st["entrances"],weights)
    st["row_queries"]=sum(map(len,tokens))
    st["ideal_prefix_queries"]=prefix_count(tokens)
    return st
