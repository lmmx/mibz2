"""Construct a numerical inverse with two kinds of affine level, storing no grammar node."""
import json
from pathlib import Path
import struct
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from atlas import atlas,real
from decode import Fold,append_run,choose,estimate,extras,geometry,overlay


class Length(int):
    """Adapter for the original Fold constructor. Its only payload is an integer."""
    def __len__(self):return int(self)
    def __add__(self,other):return self if other==b'' else Length(int(self)+int(other))


def physical(slices,words,b,target):
    """Survivors use compact predecessor indices. Deleted rows point to physical survivors."""
    out=[];old=removed=0;occ=[0]*len(words)
    for k,j,n in slices:
        if k not in target and j in target:
            act=(b[k]+occ[k]-old,int(words[k]));removed+=n
        else:act=(-removed,0)
        if not out or out[-1][1:]!=act:out.append((old,*act))
        old+=n;occ[k]+=n
    return out


def build(raw,weights):
    levels=[];trace=[]
    while True:
        f,b=geometry(raw,len(weights))
        if b[-1]==1:
            return levels,trace,'complete'
        s,e=overlay(raw,b)
        T=choose(e,0)
        q=sum(n for a,z in enumerate(e) if a not in T for j,n in z.items() if j in T)
        if 4*q<b[-1]:
            return [],trace,'insufficient ordinary cut'
        other=set(range(len(weights)))-T
        seams=extras(s)
        if estimate(other,e,seams,len(raw))<estimate(T,e,seams,len(raw)):T=other
        rec=atlas(raw,weights,b,s,T) if not levels else physical(s,weights,b,T)
        ids={};chosen=set();new=[]
        for k,z in enumerate(e):
            for j in (tuple(z) if k not in T else (None,)):
                typ=len(new);ids[k,j]=typ;new.append(weights[k])
                if j is not None and j in T:chosen.add(typ)
        col=[]
        for k,j,n in s:append_run(col,ids[k,None if k in T else j],n)
        nfs,_=geometry(col,len(new))
        folded=Fold(col,dict(enumerate(new)),list(range(len(new))),dict(enumerate(nfs)),chosen=chosen)
        assert folded.m==b[-1]-q and all(isinstance(v,Length) for v in folded.weights)
        levels.append((b[-1],folded.m,rec))
        trace.append(dict(rows=b[-1],after=folded.m,old_runs=len(raw),pieces=len(rec),old_types=len(weights),
                          joins_avoided=len(chosen),numeric_types=len(folded.weights)))
        raw,weights=folded.root,folded.weights


def serialize_parts(raw, words, mark, dest):
    lv,tr,state=build(raw,[Length(1) for _ in words])
    if state!='complete':
        return None
    # Original alphabet counts suffice to obtain F on final output; L is not needed again.
    freq,_=geometry(raw,len(words))
    with dest.open('xb') as f:
        f.write(b'POSONLY1');f.write(struct.pack('<IIII',sum(freq),mark,len(freq),len(lv)))
        for n,w in zip(freq,words):f.write(struct.pack('<IB',n,w[0]))
        for old,new,records in lv:
            f.write(struct.pack('<III',old,new,len(records)))
            for rec in records:f.write(struct.pack('<IiI',*rec))
    return tr


def serialize(tape,dest):
    _,_,mark,raw,words=real.source(tape)
    return serialize_parts(raw,words,mark,dest)


if __name__=='__main__':
    print(json.dumps(serialize(Path(sys.argv[1]),Path(sys.argv[2]))))
