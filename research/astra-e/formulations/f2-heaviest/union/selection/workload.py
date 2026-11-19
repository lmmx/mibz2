"""Price colored cyclic return spellings without FM queries or the native constructor."""
import csv
import hashlib
import json
from collections import Counter
from itertools import combinations
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"f2c"))
from second import read

def load_block(token_file):
    n,_,_,bases,words,col=read(token_file)
    raw=token_file.with_suffix(".expected").read_bytes()
    doubled=raw+raw
    pos=[i for i,c in enumerate(raw) if c==34]
    assert len(raw)==n and len(pos)==sum(w for _,w in col) and len(set(words))==len(words)
    word_id={v:i for i,v in enumerate(words)}
    labels=[word_id[doubled[a+1:b+1]] for a,b in zip(pos,pos[1:]+[pos[0]+n])]
    counts=Counter(labels)
    recorded=Counter()
    for i,w in col:
        recorded[i]+=w
    assert counts==recorded
    ordered=sorted(range(len(words)),key=lambda i:bases[i])
    assert [bases[i] for i in ordered]==list(exclusive_sums(counts[i] for i in ordered))
    return n,doubled,pos,labels,counts,ordered,words

def exclusive_sums(values):
    total=0
    for x in values:
        yield total
        total+=x

def query_count(dictionary):
    """All proper suffixes, including empty, by the LCPs of reversed strings."""
    prev=b""
    total=1
    for key in sorted(w[:0:-1] for w in dictionary):
        n=min(len(key),len(prev))
        i=0
        while i<n and key[i]==prev[i]:
            i+=1
        total+=len(key)-i
        prev=key
    return total

def workload(block,selected):
    n,raw,pos,labels,freq,order,words=block
    colors={}
    previous=False
    count=0
    for i in order:
        active=words[i] in selected
        if active:
            if not previous:
                count+=1
            colors[i]=count-1
        previous=active
    if not colors: return dict(components=0,residual_rows=0,color_tags=0,spelling_bytes=0,lookup_steps=0)
    hits=[(p,colors[t]) for p,t in zip(pos,labels) if t in colors]
    dictionaries=[set() for _ in range(count)]
    last, _=hits[-1]
    first,dest=hits[0]
    dictionaries[dest].add(raw[last:first+n])
    for (a,_),(b,dest) in zip(hits,hits[1:]):
        dictionaries[dest].add(raw[a:b])
    assert len(hits)==sum(freq[i] for i in colors)
    return dict(components=count,residual_rows=len(hits),color_tags=sum(len(w) for w in dictionaries),
                spelling_bytes=sum(len(y) for ds in dictionaries for y in ds),
                lookup_steps=sum(query_count(ds) for ds in dictionaries))

def main(folder,out):
    frozen=json.loads((HERE/"frozen.json").read_text())
    names=[bytes.fromhex(r["hex"]) for r in frozen["ordered_tokens"]]
    ref=bytes.fromhex(frozen["reference_token_hex"])
    assert ref in names
    with (HERE.parent/"native/geometry.csv").open() as f:
        native={(int(r["stream"]),int(r["k"])):r for r in csv.DictReader(f)}
    verify=[]; raw=[]; files=[]
    for path in sorted(folder.glob("*.token"),key=lambda x:int(x.stem)):
        stream=int(path.stem)
        block=load_block(path)
        n,doubled,pos,labels,freq,order,words=block
        rank={w:i for i,w in enumerate(order)}
        top=sorted(order,key=lambda i:(-freq[i],rank[i]))
        files.append(dict(stream=stream,token_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                          expected_sha256=hashlib.sha256(doubled[:n]).hexdigest()))
        for k in (1,4,8):
            if (stream,k) not in native: continue
            cand=[words[i] for i in top[:k]]
            res=workload(block,cand)
            r=native[stream,k]
            for field,native_field in (("components","components"),("residual_rows","token_rows"),
                                      ("color_tags","distinct_tags"),("spelling_bytes","spelling_bytes"),
                                      ("lookup_steps","lookup_steps")):
                assert res[field]==int(r[native_field]),(stream,k,field,res,r)
            verify.append(dict(stream=stream,k=k,**res))
        for size in (1,2):
            for selected in combinations(names,size):
                tag="+".join(w.hex() for w in selected)
                raw.append(dict(stream=stream,role="development" if stream==1 else "control" if stream==41 else "heldout",
                                marker=tag,**workload(block,selected)))
    for filename,rows in (("verified.csv",verify),("fixed.csv",raw),("inputs.csv",files)):
        with (out/filename).open("w") as f:
            w=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator="\n")
            w.writeheader();w.writerows(rows)
    assert len(verify)==27 and len(raw)==360 and len(files)==10

if __name__=="__main__":
    main(Path(sys.argv[1]),Path(sys.argv[2]))
