"""Freeze J token objects and independent directly walked H oracles from the original byte BWT."""
from array import array
import csv
from hashlib import sha256
from itertools import groupby
from pathlib import Path
from struct import Struct
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from holes import fixed
from origin import Column, RunIndex, heavy_set, paths, read_input
from sources import read as read_pack

ONE,TWO,FOUR,TEN=(Struct('<'+fmt) for fmt in ('I','II','IIII','IIIIIIIIII'))
CASES=[('frequent',0,8),('quote',1,12)]
FIXED=b'":"' + b'value","property'
assert len(FIXED)==19

def direct(last,psi,section):
    """Return destinations and outgoing bytes from each rank, without a token BWT."""
    targets={x:i for i,x in enumerate(section)}
    succ=[]
    words=[]
    for x in section:
        w=bytearray()
        while True:
            x=psi[x]
            w.append(last[x])
            if x in targets: break
        succ.append(targets[x]);words.append(bytes(w))
    assert sorted(succ)==list(range(len(section)))
    return succ,words

def marked(lf,section,mark):
    x=mark
    lookup={v:i for i,v in enumerate(section)}
    for skip in range(len(lf)):
        if x in lookup: return lookup[x],skip
        x=lf[x]
    return 0xffffffff,0

def cycle(succ,words,at):
    if at==0xffffffff:return 0,0
    x=at
    tokens=byte_count=0
    while True:
        tokens+=1;byte_count+=len(words[x]);x=succ[x]
        if x==at:return tokens,byte_count

def output(succ,words,at,skip,n):
    assert 0<=skip<len(words[at])
    data=bytearray(words[at][skip:])
    at=succ[at]
    while len(data)<n:
        data.extend(words[at]);at=succ[at]
    return bytes(data[:n])

def cut_from_original(last,psi,u,core,h):
    # This oracle examines every U-return arc starting at the old source, independently
    # of the run-index backward search. path[d] is the actual original physical cut.
    selected=[]
    for x in range(*u):
        y=x
        w=bytearray()
        prefix=[x]
        while True:
            y=psi[y]
            w.append(last[y])
            if u[0]<=y<u[1]:break
            if len(prefix)<=h:prefix.append(y)
        at=min(h,len(w)-1)
        if w[at:]==core:selected.append(prefix[at])
    return sorted(selected)

def encode_j(succ,words):
    # Give source types their physical base order rather than Python lexical order.
    types=[]
    bases=[]
    incoming=[None]*len(words)
    k=0
    for word,same in groupby(words):
        q=sum(1 for _ in same)
        t=len(types);types.append(word);bases.append(k)
        for i in range(k,k+q):incoming[succ[i]]=t
        k+=q
    assert len(set(types))==len(types)
    runs=[(t,sum(1 for _ in items)) for t,items in groupby(incoming)]
    # The stable inverse of this incoming column must recover every J successor.
    cursor=bases.copy()
    for row,t in enumerate(incoming):
        assert succ[cursor[t]]==row
        cursor[t]+=1
    return types,runs

def case(ordinal,name,policy,h,last,psi,lf,u,idx,mark,plain,dest,period):
    core=b',' if name=='quote' else FIXED[h:]
    j,parts,_,_=fixed(idx,u,core,h)
    jrows=range(*j)
    hrows=cut_from_original(last,psi,u,core,h)
    assert hrows==[x for a,b in parts for x in range(a,b)]
    owners=set(hrows)
    holes=[i for i,x in enumerate(jrows) if x not in owners]
    js,jw=direct(last,psi,jrows)
    hs,hw=direct(last,psi,hrows)
    jm,jskip=marked(lf,jrows,mark)
    hm,hskip=marked(lf,hrows,mark)
    jt,jbytes=cycle(js,jw,jm)
    ht,hbytes=cycle(hs,hw,hm)
    if jm!=0xffffffff:assert jbytes==period and output(js,jw,jm,jskip,len(last))==plain
    if hm!=0xffffffff:assert hbytes==period and output(hs,hw,hm,hskip,len(last))==plain
    types,runs=encode_j(js,jw)
    stem=f'{ordinal:04}-{name}-h{h}'
    jpath=dest/(stem+'.jh');hpath=dest/(stem+'.oracle')
    with jpath.open('wb') as f:
        f.write(b'JHSPv1\0\0')
        f.write(TEN.pack(ordinal,policy,h,len(last),len(jrows),jm,jskip,len(types),len(runs),len(holes)))
        for w in types: f.write(ONE.pack(len(w)));f.write(w)
        for t,q in runs: f.write(TWO.pack(t,q))
        for rank in holes: f.write(ONE.pack(rank))
    topology=sha256()
    with hpath.open('wb') as f:
        f.write(b'JHORv1\0\0')
        f.write(FOUR.pack(len(hrows),hm,hskip,len(last)))
        for y,w in zip(hs,hw):
            encoded=TWO.pack(y,len(w))+w
            topology.update(encoded);f.write(encoded)
        f.write(plain)
    return dict(case=stem,ordinal=ordinal,section=name,h=h,core=core.hex(),n=len(last),primary=mark,
        j_lo=j[0],j_hi=j[1],j_mass=len(jrows),j_mark=jm,j_skip=jskip,types=len(types),runs=len(runs),
        dict_bytes=sum(map(len,types)),holes=len(holes),h_mass=len(hrows),h_mark=hm,h_skip=hskip,
        j_sha256=sha256(jpath.read_bytes()).hexdigest(),h_sha256=sha256(hpath.read_bytes()).hexdigest(),
        h_map_words_sha256=topology.hexdigest(),marked_sha256=sha256(plain).hexdigest(),
        original_cycle_bytes=period,j_cycle_tokens=jt,j_cycle_bytes=jbytes,h_cycle_tokens=ht,h_cycle_bytes=hbytes)

def main():
    dest=Path(sys.argv[2]);dest.mkdir(parents=True,exist_ok=True)
    cases=CASES if len(sys.argv)<4 else [('frequent',0,h) for h in (4,8,12)]+[('quote',1,h) for h in (4,8,12)]
    writer=None
    src=Path(sys.argv[1])
    if src.is_dir() or src.name.endswith('.irec.xz'):
        inputs=((int(t.name.split('.')[0]),*read_input(t),sha256(t.read_bytes()).hexdigest()) for t in paths(src))
    else:
        inputs=read_pack(src)
    for ordinal,runs,mark,archived_digest in inputs:
        last=b''.join(bytes([a])*q for a,q in runs)
        idx=RunIndex(runs);col=Column(runs)
        psi=array('I',sorted(range(len(last)),key=last.__getitem__))
        lf=array('I',[0])*len(last)
        for i,y in enumerate(psi):lf[y]=i
        x=mark;plain=bytearray();period=0
        for step in range(1,len(last)+1):
            x=psi[x];plain.append(last[x])
            if x==mark and not period:period=step
        assert period>0
        quote=col.interval(b'"')
        frequent=heavy_set(col,quote,1)[0][0]
        for name,policy,h in cases:
            row=case(ordinal,name,policy,h,last,psi,lf,frequent if policy==0 else quote,idx,mark,plain,dest,period)
            row['input_tape_sha256']=archived_digest
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n');writer.writeheader()
            writer.writerow(row);sys.stdout.flush()
if __name__=='__main__':
    main()
