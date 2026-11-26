"""Stream the marked forward output using partial macros and deferred raw Psi tails."""
from array import array
from collections import Counter
import csv
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from census import ROOT,make_index,read_input
from truncated import construct
from barriers import descriptors

def prepare_psi(last,lf):
    psi=array("I",[0])*len(last)
    for x,y in enumerate(lf):psi[y]=x
    return psi

def emit(last,p,q,psi,cut,hot):
    n=len(last);m=len(cut)
    enriched=descriptors(hot,cut)
    if enriched:
        B,macros,_=enriched
    else:
        B=set(range(m));macros={x:([],x,None) for x in B}
    complete={};continuations={}
    for dest,(back,cold,source) in macros.items():
        forward=b"".join(reversed(back))
        if cold is None:
            assert source not in complete
            complete[source]=(dest,forward)
        else:
            seed,reverse_suffix=cut[cold]
            assert seed not in continuations
            continuations[seed]=(dest,reverse_suffix[::-1]+forward)
    assert len(complete)+len(continuations)==len(B)
    out=bytearray();stats=Counter()
    while len(out)<n and p-q not in B:
        p=psi[p];out.append(last[p]);stats["initial_original_queries"]+=1
    rank=p-q
    while len(out)<n:
        stats["macro_visits"]+=1
        if rank in complete:
            rank,payload=complete[rank]
        else:
            stats["cold_macro_visits"]+=1
            x=q+rank
            while True:
                x=psi[x];out.append(last[x]);stats["cold_original_queries"]+=1
                if len(out)==n: return bytes(out),stats
                if x in continuations:break
            rank,payload=continuations[x]
        out.extend(payload[:n-len(out)])
    return bytes(out),stats

if __name__=="__main__":
    w=csv.writer(sys.stdout,lineterminator="\n");w.writerow(["stream","threshold","output_bytes","initial_queries","cold_queries","macro_visits","cold_macro_visits"])
    path=Path(sys.argv[1]);files=sorted((ROOT/"research/astra-e/dense/target/wikidata").glob("*.irec.xz"))
    for rec,ref in zip(files,sorted(path.glob("*.expected"),key=lambda x:int(x.stem)),strict=True):
        runs,p=read_input(rec);last=b"".join(bytes([c])*w for c,w in runs);q,m,lf=make_index(last)
        psi=prepare_psi(last,lf);expected=ref.read_bytes()
        for t in (1,2,4,8):
            hot,cut=construct(last,q,m,lf,t)
            got,stats=emit(last,p,q,psi,cut,hot)
            assert got==expected
            w.writerow([ref.stem,t,len(got),*(stats[k] for k in ("initial_original_queries","cold_original_queries","macro_visits","cold_macro_visits"))])
