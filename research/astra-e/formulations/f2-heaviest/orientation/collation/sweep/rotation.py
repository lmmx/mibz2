"""Independent dense word checks for the orbit shift c·v -> v·c."""
import argparse
import itertools
import json
from pathlib import Path


def words(psi, label, roots):
    root = set(roots)
    result = set()
    for x in root:
        w = bytearray()
        while True:
            w.append(label[x])
            x = psi[x]
            if x in root:
                break
        result.add(bytes(w))
    return result


def nodes(prefixes):
    ordered = sorted(set(prefixes))
    size = int(bool(ordered))
    prev = b""
    for current in ordered:
        common = 0
        while common < min(len(current),len(prev)) and current[common] == prev[common]:
            common += 1
        size += len(current)-common
        prev = current
    return size


def price(dictionary):
    return dict(words=len(dictionary), distinct_bytes=sum(map(len,dictionary)),
                proper_suffixes=nodes(w[:0:-1] for w in dictionary))


def audit(psi,label,section):
    section = sorted(section)
    c = label[section[0]]
    if any(label[x] != c for x in section):
        return
    image = sorted(psi[x] for x in section)
    source = words(psi,label,section)
    target = words(psi,label,image)
    assert {w[1:]+w[:1] for w in source} == target
    old,new = price(source),price(target)
    assert (old["distinct_bytes"],old["words"]) == (new["distinct_bytes"],new["words"])
    # In reverse order, a terminal is maximal under proper-suffix inclusion iff
    # the next distinct key does not extend it. Count maximal nonempty tails.
    tails = sorted(w[:0:-1] for w in source)
    maximal = sum(bool(w) and (i+1 == len(tails) or not tails[i+1].startswith(w))
                  for i,w in enumerate(tails))
    improvement = old["proper_suffixes"]-new["proper_suffixes"]
    assert improvement == max(0,maximal-1)
    assert 0 <= improvement <= len(source)-1
    return image, old, new, maximal, improvement


def incoming_table(psi,label,section):
    # The domain and codomain are ordered *compressed* section positions.
    pos={row:i for i,row in enumerate(section)}
    targets=[]
    spellings=[]
    for x in section:
        w=bytearray()
        while True:
            w.append(label[x]);x=psi[x]
            if x in pos:
                targets.append(pos[x]);spellings.append(bytes(w));break
    return targets,spellings


def small():
    counts = dict(monochromatic=0, contiguous_images=0, holed_images=0,
                  positive=0, zero_multiword=0, strictly_between=0, full_gain_multiword=0)
    examples = {}
    def process(psi,label,stable):
        n=len(label)
        for lo in range(n):
            for hi in range(lo+1,n+1):
                result=audit(psi,label,range(lo,hi))
                if result is None:
                    continue
                image,old,new,maximal,improvement=result
                contiguous=image[-1]-image[0]+1==len(image)
                counts["monochromatic"]+=1
                counts["contiguous_images" if contiguous else "holed_images"]+=1
                counts["positive"]+=improvement>0
                if old["words"]>1:
                    key=("zero_multiword" if improvement==0 else
                         "full_gain_multiword" if improvement==old["words"]-1 else "strictly_between")
                    counts[key]+=1
                    if stable and contiguous and key not in examples:
                        examples[key]=dict(psi=psi,label=list(label),source=[lo,hi],image=image,
                                           original=old,shifted=new,maximal_tails=maximal)
    for n in range(1,6):
        for perm in itertools.permutations(range(n)):
            for label in itertools.product(range(2),repeat=n):
                process(perm,label,False)
    for n in range(1,9):
        for col in itertools.product(range(2),repeat=n):
            psi=sorted(range(n),key=col.__getitem__)
            process(psi,[col[i] for i in psi],True)
    return dict(counts=counts,stable_contiguous_examples=examples)


def frozen(column,summary):
    col=column.read_bytes()
    psi=sorted(range(len(col)),key=col.__getitem__)
    label=bytes(col[i] for i in psi)
    report=json.loads(summary.read_text())
    m=report["mass"]
    targets={report["named"]["start"], report["min_D_then_Q_then_start"]["start"],
             report["min_Q_then_D_then_start"]["start"],
             *(x["start"] for x in report["jointly_better_pareto_points"]),
             *(x["start"] for x in report["frozen_rules"].values())}
    expected={x["start"]:x for x in [report["named"],report["min_D_then_Q_then_start"],
              report["min_Q_then_D_then_start"],*report["jointly_better_pareto_points"],
              *report["frozen_rules"].values()]}
    checked={}
    for a in sorted(targets):
        metric=price(words(psi,label,range(a,a+m)))
        assert all(expected[a][k]==v for k,v in metric.items())
        checked[a]=metric
    a=report["named"]["start"]
    image,old,new,maximal,improvement=audit(psi,label,range(a,a+m))
    first=image[0];assert image==list(range(first,first+m))
    second_image=[psi[x] for x in image]
    chain=[]
    section=list(range(a,a+m))
    original_targets,_=incoming_table(psi,label,section)
    while True:
        targets,names=incoming_table(psi,label,section)
        assert targets==original_targets
        letters=set(label[x] for x in section)
        chain.append(dict(depth=len(chain),intervals=1+sum(y!=x+1 for x,y in zip(section,section[1:])),
                          first=section[0],last=section[-1],outgoing_bytes=len(letters),
                          common_byte=next(iter(letters)) if len(letters)==1 else None,
                          **price(set(names))))
        if len(letters)!=1:break
        advanced=[psi[x] for x in section]
        assert advanced==sorted(advanced)
        next_targets,next_names=incoming_table(psi,label,advanced)
        assert next_targets==targets and all(w[1:]+w[:1]==v for w,v in zip(names,next_names))
        if advanced==section:break
        section=advanced
        assert len(chain)<len(col)
    return dict(explicit_dense_prices=checked, successive_common_byte_shifts=chain,
                named_image=dict(start=first,end=first+m,common_byte=label[a],original=old,shifted=new,
                                 maximal_nonempty_tails=maximal,suffix_nodes_saved=improvement,
                                 next_image_min=min(second_image),next_image_max=max(second_image),
                                 next_image_runs=1+sum(y!=x+1 for x,y in zip(second_image,second_image[1:]))))


if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("column",type=Path,nargs="?");p.add_argument("summary",type=Path,nargs="?")
    args=p.parse_args()
    print(json.dumps(frozen(args.column,args.summary) if args.column else small(),indent=2))
