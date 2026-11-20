import json,sys
from array import array
import os
d=os.environ.get('BLOCKS','admitted-bwt/'); m=json.load(open(d+'manifest.json'))
for b in m['blocks']:
    L=open(d+b['file'],'rb').read(); n=len(L)
    cnt=[0]*256
    for c in L: cnt[c]+=1
    C=[0]*256;a=0
    for c in range(256): C[c]=a;a+=cnt[c]
    occ=C[:]; LF=array('I',[0])*n
    for i,c in enumerate(L): LF[i]=occ[c]; occ[c]+=1
    inv=array('I',[0])*n
    for i in range(n): inv[LF[i]]=i
    # commutator k = r u r^-1 u^-1 with r=LF, u=+1 mod n
    seen=bytearray(n); fixed=0; cyc=[]
    def k(x):
        y=(x-1)%n; y=inv[y]; y=(y+1)%n; return LF[y]
    for x in range(n):
        if seen[x]: continue
        l=0;y=x
        while not seen[y]: seen[y]=1; y=k(y); l+=1
        if l==1: fixed+=1
        else: cyc.append(l)
    s=sum(c-1 for c in cyc); g=(s+2)//2
    from collections import Counter
    # LF single cycle?
    x=b['primary_index'];L1=0
    while True:
        x=LF[x];L1+=1
        if x==b['primary_index']: break
    print(f"{b['file']:12s} n={n} r={b['r']} | squares moved by commutator={n-fixed} ({(n-fixed)/b['r']:.2f}r) cone points={len(cyc)} ({len(cyc)/b['r']:.2f}r) genus={g} (2g-2={s}) | cone angles (2π·len) top={Counter(cyc).most_common(4)} | LF cycle={L1==n}")
