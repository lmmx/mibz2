"""Stop the quote suffix trie at first unit width and replay missing LF arcs."""
from array import array
from bisect import bisect_right
from collections import Counter
import csv
from pathlib import Path
from hashlib import sha256
import sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
sys.path.insert(0,str(ROOT/"docs/journal/2026-11-18-astra-c-output-first"))
from inputs import read_input
sys.path.insert(0,str(HERE.parents[1]/"f2c"))
from second import read
from truncated import construct
from barriers import descriptors,decode_barriers,verify_partial_intervals

def make_index(last):
    freq=Counter(last); starts=[];base=0
    for c in range(256):
        starts.append(base);base+=freq[c]
    q=starts[34];m=freq[34]
    lf=array("I")
    for ch in last:
        lf.append(starts[ch]);starts[ch]+=1
    return q,m,lf

def prune(last,q,m,lf,threshold=1):
    if not m:return {},0,0
    # The state carries the original row interval, and its already-known reverse suffix.
    stack=[(q,q+m,b"")]; seeds={};queries=hot_sites=0
    while stack:
        a,b,suffix=stack.pop()
        if b-a<=threshold:
            for x in range(a,b):
                assert x not in seeds
                seeds[x]=suffix
            continue
        queries+=1
        bins={}
        for row in range(a,b):
            c=last[row]
            if c==34: continue
            at=lf[row]
            if c in bins: assert at==bins[c][1];bins[c][1]+=1
            else: bins[c]=[at,at+1]
        if 34 in last[a:b]: hot_sites+=1
        for c,(lo,hi) in bins.items():
            stack.append((lo,hi,suffix+bytes([c])))
    return seeds,queries,hot_sites

def describe(last,q,m,lf,seeds):
    source=array("I");labels=[];cut=[None]*m;remaining=[];hot={}
    for dest in range(m):
        x=q+dest;back=bytearray()
        while True:
            if x in seeds:
                assert cut[dest] is None and back==seeds[x]
                cut[dest]=(x,seeds[x])
            b=last[x];back.append(b);x=lf[x]
            if b==34:break
        y=bytes(reversed(back))
        source.append(x-q);labels.append(y)
        if cut[dest] is not None:
            remaining.append(len(y)-len(cut[dest][1]))
        else:
            hot[dest]=(x-q,y)
    assert len(set(source))==m and len(set(seeds))==len(remaining)
    return source,labels,cut,remaining,hot

def marked(last,p,q,m,lf,cut,hot):
    mark=p;offset=0
    while not q<=mark<q+m:
        mark=lf[mark];offset+=1
        if mark==p:return None
    anchor=mark-q;x=anchor;rev=bytearray();visits=cold_queries=cold_visits=0
    while True:
        visits+=1
        if x in hot:
            x,forward=hot[x]
            rev.extend(reversed(forward))
        else:
            x,suffix=cut[x];rev.extend(suffix)
            cold_visits+=1
            while True:
                byte=last[x];cold_queries+=1;rev.append(byte);x=lf[x]
                if byte==34:break
            x-=q
        if x==anchor: break
    period=bytes(reversed(rev))
    off=offset%len(period)
    lap=period[off:]+period[:off]
    n=len(last)
    return (lap*(n//len(lap))+lap[:n%len(lap)],len(lap),visits,cold_queries,cold_visits)

def sizes(values):
    s=sorted(values)
    if not s: return (0,0,0,0)
    return len(s),s[(len(s)-1)//2],s[(len(s)-1)*95//100],s[-1]

def gap_hist(source,labels, cut,hot):
    cold={i for i,x in enumerate(cut) if x is not None}
    slots=set(source)-{x[0] for x in hot.values()}
    assert slots=={source[x] for x in cold}
    lengths=[];sizes_b=[];seen=set()
    for x in slots:
        length=width=0
        while x not in cold:
            assert x not in seen;seen.add(x)
            width+=len(labels[x]);x=source[x];length+=1
        lengths.append(length);sizes_b.append(width)
    # Remaining nodes belong to hot-only cycles.
    return cold,slots,lengths,sizes_b,len(source)-len(seen)-len(cold)

def second_cost(source,labels,cold,slots,base):
    # Heaviest word's contiguous token-F bucket is supplied by a hot terminal.
    freq=Counter(labels)
    heavy=min(freq,key=lambda w:(-freq[w],base[w]))
    D=set(range(base[heavy],base[heavy]+freq[heavy]))
    extra=D|cold|slots
    bad=good=bad_bytes=good_bytes=0
    for start in D:
        p=start;tainted=False;width=0
        while True:
            tainted|=p in cold;width+=len(labels[p]);p=source[p]
            if p in D: break
        if tainted: bad+=1;bad_bytes+=width
        else: good+=1;good_bytes+=width
    return len(D),len(extra),bad,good,bad_bytes,good_bytes

def analyse(path,token,distributions):
    runs,p=read_input(path)
    last=b"".join(bytes([c])*n for c,n in runs)
    q,m,lf=make_index(last)
    seeds,queries,hot_sites=prune(last,q,m,lf)
    source,labels,cut,rem,hot=describe(last,q,m,lf,seeds)
    n,_,_,bases,words,col=read(token)
    # Exported words own the ending quote. Our raw LF arcs own the starting one.
    move=lambda s:s[-1:]+s[:-1]
    base={move(w):b for w,b in zip(words,bases)}
    count=Counter();i=0
    for wid,f in col:
        w=move(words[wid])
        for j in range(f):
            assert (labels[i+j],source[i+j])==(w,bases[wid]+count[wid]+j)
        count[wid]+=f;i+=f
    assert i==m and n==len(last)
    produced_hot,produced_cut=construct(last,q,m,lf)
    assert (produced_hot,produced_cut)==(hot,cut)
    B,macros,metrics=descriptors(produced_hot,produced_cut)
    paid,pieces=verify_partial_intervals(B,produced_hot,produced_cut,macros)
    assert paid==metrics["regularity_queries"]
    resolved,barrier_reads,barrier_visits=decode_barriers(last,p,q,lf,produced_hot,produced_cut,B,macros)
    marked_result=marked(last,p,q,m,lf,produced_cut,produced_hot)
    out,period,visits,cold_queries,cold_visits=marked_result
    assert resolved==out==token.with_suffix(".expected").read_bytes()
    cold,slots,gaps,byte_gaps,other=gap_hist(source,labels,cut,hot)
    assert len({labels[i] for i in cold})==len(cold)
    root_runs=sum(1 for j in range(m) if j==0 or labels[j]!=labels[j-1])
    d,new,bad,good,badb,goodb=second_cost(source,labels,cold,slots,base)
    freq=Counter(labels)
    ell=[len(labels[i]) for i in cold]
    for measure,sample in (("cold_post_cut_lookups",rem),("cold_complete_bytes",ell),
                           ("hot_gap_tokens",gaps),("hot_gap_bytes",byte_gaps),
                           ("cut_known_nonquote_bytes",[len(v[1]) for v in cut if v is not None])):
        for val,f in sorted(Counter(sample).items()):
            distributions.append([int(token.stem),measure,val,f])
    _,r50,r95,rmax=sizes(rem)
    _,g50,g95,gmax=sizes(gaps)
    _,b50,b95,bmax=sizes(byte_gaps)
    return dict(stream=int(token.stem),rows=n,marked_period=period,token_rows=m,root_runs=root_runs,words=len(words),
                singleton_types=sum(f==1 for f in freq.values()),cutoff_nodes=len(cold),known_terminal_sites=hot_sites,
                retained_queries=queries,deferred_queries=sum(rem),total_queries=queries+sum(rem),
                cold_equal_quote_at_cut=sum(v==1 for v in rem),tail_p50=r50,tail_p95=r95,tail_max=rmax,
                known_nonquote_bytes=sum(len(x[1]) for x in cut if x is not None),
                cold_raw_bytes=sum(ell),total_dict_bytes=sum(map(len,words)),suppressed_edge_bytes=sum(rem)-len(rem),
                marked_visits=visits,marked_cold_visits=cold_visits,marked_serial_cold_queries=cold_queries,
                missing_sources=len(slots),hot_gap_zero=sum(g==0 for g in gaps),hot_gap_p50=g50,hot_gap_p95=g95,hot_gap_max=gmax,
                hot_gap_bytes_p50=b50,hot_gap_bytes_p95=b95,hot_gap_bytes_max=bmax,hot_cycle_rows=other,
                second_rows=d,both_endpoint_barrier_rows=new,tainted_second_arcs=bad,hot_second_arcs=good,
                tainted_second_bytes=badb,hot_second_bytes=goodb,output_sha256=sha256(out).hexdigest(),barrier_original_queries=barrier_reads,barrier_visits=barrier_visits,partial_fm_intersections=pieces,
                **{"partial_"+k:v for k,v in metrics.items()})

if __name__=="__main__":
    folder=Path(sys.argv[1])
    w=None;distributions=[]
    for p,t in zip(sorted((ROOT/"research/astra-e/dense/target/wikidata").glob("*.irec.xz")),
                   sorted(folder.glob("*.token"),key=lambda t:int(t.stem)),strict=True):
        row=analyse(p,t,distributions)
        if w is None:
            w=csv.DictWriter(sys.stdout,fieldnames=list(row),lineterminator="\n");w.writeheader()
        w.writerow(row);sys.stdout.flush()
    with Path(sys.argv[2]).open("w") as f:
        h=csv.writer(f,lineterminator="\n");h.writerow(["stream","measure","value","frequency"]);h.writerows(distributions)
