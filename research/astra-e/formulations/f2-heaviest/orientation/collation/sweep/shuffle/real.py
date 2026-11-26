"""Fixed policy geometry screens and a separate first-return byte audit from physical V."""
from collections import Counter
import csv
import hashlib
from itertools import accumulate,groupby
from pathlib import Path
import sys
from census import Oracle,Column,heavy_set,labels,follower_classes,read_input,paths,HERE
from interface import geometry,shuffle,mask_runs,reconstruct,incoming_by_lf,emit
sys.path.insert(0,str(HERE.parent/"context"))
from shifts import suffixes


def strict_policies(cls,src):
    counts=Counter(cls)
    visit=Counter(cls[i] for i in src)
    biggest=max(counts,key=lambda c:(visit[c],-c))
    return [("stay",set()),("shift",set(counts)),("largest",{biggest}),
            ("except_largest",set(counts)-{biggest}),
            ("singleton",{c for c,n in counts.items() if n==1}),
            ("multiple",{c for c,n in counts.items() if n>1})]


def main():
    out=None
    for p in paths(sys.argv[1]):
        runs,mark=read_input(p)
        col=Column(runs);last=b"".join(bytes([x])*n for x,n in runs)
        ora=Oracle(last)
        from array import array
        lf=array("I",[0])*len(last)
        for i,x in enumerate(ora.forward):lf[x]=i
        quote=col.interval(b'"')
        row=mark;expected=bytearray()
        for _ in range(len(last)):
            expected.append(ora.first[row]);row=ora.forward[row]
        for name,sec in [("quote",quote),("frequent",heavy_set(col,quote,1)[0][0])]:
            a,b=sec
            words,counts,src,inc=labels(ora,sec)
            cls,after,followers=follower_classes(words,src,inc)
            fixed=strict_policies(cls,src)
            # Independent original source/destination reference. All geometry calls
            # take only on-step Ψ, U and the known source-type move sequence.
            starts=list(accumulate([0]+counts[:-1]))
            tau=[None]*len(src)
            for j,t in enumerate(inc):
                tau[starts[t]]=j;starts[t]+=1
            audit_done=False
            for policy,selected in fixed:
                bits=[int(c in selected) for c in cls]
                after_bit=[int(c in selected) for c in after]
                mask=[bits[x] for x in src]
                transformed=[w[bits[i]:]+(w[:1] if after_bit[i] else b"") for i,w in enumerate(words)]
                info=dict(ordinal=int(p.name.split(".")[0]),section=name,policy=policy,m=len(src),
                          moved=sum(mask),classes=len(set(cls)),old_types=len(words),
                          distinct_new=len(set(transformed)),empty=sum(not word for word in transformed),
                          D=None,Q=None,fragments=None,source_mask_runs=mask_runs(mask),physical_mask_runs=None,
                          physical_in_runs=None,old_in_runs=mask_runs(inc),nonidentity=None,
                          dense_LF_audit_steps=None,marked_sha256=None)
                g=geometry(ora.forward,sec,mask)
                if g is not None:
                    phys,t=g
                    assert not info["empty"]
                    sig=shuffle(mask,t)
                    pwords=[None]*len(src)
                    for j,x in enumerate(sig):pwords[x]=transformed[inc[j]]
                    sol=reconstruct(words,counts,mask,t,pwords,words[0][:1])
                    assert sol==tau
                    eligible=info["distinct_new"]==len(words)
                    info.update(D=sum(map(len,set(transformed))),Q=suffixes(set(transformed)),
                                fragments=1+sum(x+1!=y for x,y in zip(phys,phys[1:])),
                                physical_mask_runs=mask_runs(t),physical_in_runs=mask_runs(pwords),
                                nonidentity=sum(i!=x for i,x in enumerate(sig)))
                    # First fixed nonuniform, also on cases whose dictionary coalesces.
                    if not audit_done and 0<sum(mask)<len(mask):
                        actual,steps=incoming_by_lf(phys,last,lf)
                        assert actual==pwords
                        derived=reconstruct(words,counts,mask,t,actual,words[0][:1])
                        assert derived==tau and emit(ora,sec,mark,src,words,derived)==expected
                        info.update(dense_LF_audit_steps=steps,marked_sha256=hashlib.sha256(expected).hexdigest())
                        audit_done=True
                else:
                    assert info["empty"]
                if out is None:
                    out=csv.DictWriter(sys.stdout,fieldnames=info,lineterminator="\n");out.writeheader()
                out.writerow(info)
            assert audit_done
            sys.stdout.flush()


if __name__=="__main__":
    main()
