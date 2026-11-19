from collections import Counter,defaultdict
import csv
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from census import ROOT,make_index,prune,describe,marked,read_input
from truncated import construct
from barriers import descriptors,decode_barriers,verify_partial_intervals
from forward import components,advance,prefix_count

def inspect(last,p,t,q,m,lf,original,expected,forward=True):
    seeds,paid,hot_sites=prune(last,q,m,lf,t)
    src,labels,cut,rem,hot=describe(last,q,m,lf,seeds)
    assert (hot,cut)==construct(last,q,m,lf,t)
    R=marked(last,p,q,m,lf,cut,hot)
    assert (R is None)==(expected is None)
    if R is not None:assert R[0]==expected
    groups=defaultdict(list)
    for i,c in enumerate(cut):
        if c is not None: groups[c[1]].append(i)
    # Each group is one source interval of at most threshold rows, stable in
    # destination order. Original partial suffixes are the same in the group.
    for inds in groups.values():
        start=cut[inds[0]][0]
        assert len(inds)<=t and [cut[i][0] for i in inds]==list(range(start,start+len(inds)))
    cold=[i for inds in groups.values() for i in inds]
    cold_types={labels[i] for i in cold}
    post=0;homogeneous=0;ambiguous=0;affine=0
    for suffix,inds in groups.items():
        shadows=[labels[i][::-1][len(suffix):] for i in inds]
        post+=prefix_count(shadows)
        if len(set(shadows))==1: homogeneous+=1
        else:ambiguous+=1
        affine+=all(src[v]==src[inds[0]]+j for j,v in enumerate(inds)) and len(set(shadows))==1
    assert original==paid+post
    h=descriptors(hot,cut)
    if h:
        B,macros,met=h
        check=decode_barriers(last,p,q,lf,hot,cut,B,macros)
        assert (check is None)==(expected is None)
        if check is not None:assert check[0]==expected
        actual,parts=verify_partial_intervals(B,hot,cut,macros)
        assert actual==met["regularity_queries"] and len(B)==len(cold)+met["second_rows"]
    else:
        met={};parts=0
    r=dict(threshold=t,rows=len(last),section_rows=m,cold_groups=len(groups),cold_rows=len(cold),
           hidden_word_types=len(cold_types),homogeneous_groups=homogeneous,mixed_groups=ambiguous,
           fully_affine_groups=affine,first_original_queries=original,first_hot_queries=paid,
           eager_saved_queries=post,backward_row_queries=sum(rem),weighted_cold_bytes=sum(len(labels[i]) for i in cold),
           elided_distinct_dict_bytes=sum(map(len,cold_types)),true_cold_prefix_steps=sum(rem)-len(rem),
           retained_suffix_bytes=sum(len(cut[i][1]) for i in cold),partial_fm_intersections=parts,
           marked_cold_queries=(R[3] if R else 0),**met)
    for full in ((False,True) if forward else ()):
        metrics=advance(last,q,lf,seeds,src,labels,cut,hot,full)
        if not full:assert metrics["row_queries"]==sum(rem)
        r.update({("full_" if full else "tail_")+k:v for k,v in metrics.items()})
    return r

if __name__=="__main__":
    export=Path(sys.argv[1]);w=None
    files=sorted((ROOT/"research/astra-e/dense/target/wikidata").glob("*.irec.xz"))
    for path,target in zip(files,sorted(export.glob("*.expected"),key=lambda p:int(p.stem)),strict=True):
        runs,p=read_input(path);last=b"".join(bytes([c])*n for c,n in runs);q,m,lf=make_index(last)
        expected=target.read_bytes()
        original=prune(last,q,m,lf,0)[1]
        for threshold in (1,2,4,8):
            r=inspect(last,p,threshold,q,m,lf,original,expected)
            row=dict(stream=int(target.stem),source_runs=len(runs),**r)
            if w is None:
                w=csv.DictWriter(sys.stdout,fieldnames=list(row),lineterminator="\n");w.writeheader()
            w.writerow(row);sys.stdout.flush()
