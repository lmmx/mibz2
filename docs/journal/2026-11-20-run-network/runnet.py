import json,os
from collections import Counter
d=os.environ.get('BLOCKS','admitted-bwt/'); m=json.load(open(d+'manifest.json'))
for b in m['blocks']:
    L=open(d+b['file'],'rb').read(); n=len(L)
    runs=[];i=0
    while i<n:
        c=L[i];j=i+1
        while j<n and L[j]==c: j+=1
        runs.append((c,i,j-i)); i=j
    r=len(runs); cnt=[0]*256
    for c,_,w in runs: cnt[c]+=w
    st=[0]*256;a=0
    for c in range(256): st[c]=a;a+=cnt[c]
    cur=st[:]; img=[]
    for k,(c,s,w) in enumerate(runs): img.append((cur[c],cur[c]+w,k)); cur[c]+=w
    img.sort()
    # arcs: run k (source) -> run v (target domain) where LF image of k overlaps v
    out=[0]*r; inn=[0]*r; arcs=[]; vi=0
    for a0,a1,k in img:
        x=a0
        while x<a1:
            while runs[vi][1]+runs[vi][2]<=x: vi+=1
            e=min(a1,runs[vi][1]+runs[vi][2])
            arcs.append((k,vi,e-x)); out[k]+=1; inn[vi]+=1; x=e
    A=len(arcs)
    deg=Counter((inn[k],out[k]) for k in range(r))
    series=[k for k in range(r) if inn[k]==1 and out[k]==1]
    core=r-len(series)
    # series chains: follow out-arc from core vertex through series vertices
    succ={}
    for k,v,w in arcs:
        if out[k]==1: succ[k]=v
    issr=bytearray(r)
    for k in series: issr[k]=1
    chain=[]; 
    for k,v,w in arcs:
        if not issr[k]:
            l=0;x=v
            while issr[x]: l+=1;x=succ[x]
            chain.append(l)
    widths=[w for _,_,w in arcs]
    print(f"{b['file']:12s} r={r} arcs={A} ({A/r:.3f}r) cycle-rank={A-r+1} ({(A-r+1)/r:.3f}r) | in=out=1 runs={len(series)} ({len(series)/r:.2f}r) core={core} ({core/r:.3f}r) | core arcs after series reduction={len(chain)} ({len(chain)/r:.3f}r), mean chain {sum(chain)/max(1,len(chain)):.2f}, max {max(chain)} | deg (in,out) top {deg.most_common(5)} | arc width median {sorted(widths)[A//2]}")

print("--- rows in core runs (a section: every row of a non-series run) vs Open's retained thin rows")
for b in m['blocks']:
    L=open(d+b['file'],'rb').read(); n=len(L)
    runs=[];i=0
    while i<n:
        c=L[i];j=i+1
        while j<n and L[j]==c: j+=1
        runs.append((c,i,j-i)); i=j
    r=len(runs); cnt=[0]*256
    for c,_,w in runs: cnt[c]+=w
    st=[0]*256;a=0
    for c in range(256): st[c]=a;a+=cnt[c]
    cur=st[:]; img=[]
    for k,(c,s,w) in enumerate(runs): img.append((cur[c],cur[c]+w,k)); cur[c]+=w
    img.sort(); out=[0]*r; inn=[0]*r; vi=0
    for a0,a1,k in img:
        x=a0
        while x<a1:
            while runs[vi][1]+runs[vi][2]<=x: vi+=1
            e=min(a1,runs[vi][1]+runs[vi][2]); out[k]+=1; inn[vi]+=1; x=e
    corerows=sum(runs[k][2] for k in range(r) if not(inn[k]==1 and out[k]==1))
    thin=sum(w for _,_,w in runs if w<=16)
    print(f"{b['file']:12s} core-run rows={corerows} ({corerows/n:.3f}n)  Open thin rows={thin} ({thin/n:.3f}n)  series-run rows={(n-corerows)/n:.3f}n")
