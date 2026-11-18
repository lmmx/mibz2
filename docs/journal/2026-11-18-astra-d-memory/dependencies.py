"""Count address-producing loads separately from line transfers or hardware stalls."""
import csv
import sys
from array import array
from collections import Counter
from itertools import accumulate, groupby
from pathlib import Path
from struct import iter_unpack
study=Path(__file__).resolve().parent
sys.path.insert(0,str(study.parents[2]/"bench"))
import irec
from tiles import price

def slot_counts(rec, run):
    h = rec.head
    v = rec.a
    n,r = h["n"],h["r"]
    assert h["period"]==n and h["closures"]==1
    widths=v["l_lam"]
    ends=(set(accumulate(widths)),set(accumulate(widths[j] for j in v["l_bottom"])))
    fresh=[True,True]
    clock=n
    seen=Counter()
    chain=[0,0]
    longest=[0,0]
    for i in range(rec.steps+1):
        # Full-period records have their only closure after the last cut.
        bits=v["s_fresh"][i] if i<rec.steps else 4*fresh[0]+8*fresh[1]
        for side in (0,1):
            if not fresh[side]: continue
            header=clock in ends[side]
            slot=bool(bits & (4<<side))
            assert slot != bool(bits & (1<<side))
            if header:
                seen[side,"headers"]+=1
                seen[side,"tagged"]+=int(slot)
                chain[side]=0
            else:
                assert slot
                seen[side,"linked"]+=1
                chain[side]+=1
                longest[side]=max(longest[side],chain[side])
            seen[side,"slots"]+=int(slot)
        if i<rec.steps:
            a,b=v["s_ra"][i],v["s_rb"][i]
            fresh=[a<=b,b<=a]
            clock-=min(a,b)
    assert seen[0,"headers"]==seen[1,"headers"]==r
    assert seen[0,"tagged"]+seen[1,"tagged"]==r
    x=rec.final
    spine=0
    while v["w_kind"][x]&3 == irec.RULE:
        a,b=v["s_a"][x-r],v["s_b"][x-r]
        x=a if v["w_kind"][a]&irec.MARKED else b
        spine+=1
    assert x==rec.row_letter
    rule=lambda j: v["w_kind"][j]&3==irec.RULE
    edges=sum(rule(a)+rule(b) for a,b in zip(v["s_a"],v["s_b"]) if rule(a) or rule(b))
    rules=sum(rule(j) for j in range(len(v["w_kind"])))
    commands=edges-(rules-1)
    meta=next(s for s in (run/"ranges").read_text().splitlines() if s.startswith("info cells")).split()
    m=dict(zip(meta[1::2],map(int,meta[2::2])))
    assert rules==m["rules"]
    return [h["index"],n,rec.steps,r,*[seen[side,k] for side in (0,1) for k in ("headers","tagged","linked","slots")],
            *longest,r-m["peak_pairs"],spine,rules,commands]

def segments(path,n,p):
    starts={p}
    x=0x2545f4914f6cdd1d ^ n
    mask=(1<<64)-1
    for _ in range(511):
        x ^= (x<<13)&mask
        x ^= x>>7
        x ^= (x<<17)&mask
        starts.add(x%n)
    col=bytearray()
    for (rec,) in iter_unpack("<H",path.read_bytes()):
        col.extend(bytes([rec>>8])*(rec&255))
    assert len(col)==n
    hist=Counter(col)
    cur=list(accumulate([0]+[hist[c] for c in range(255)]))
    lf=array("I")
    for c in col: lf.append(cur[c]); cur[c]+=1
    lengths=[]
    for a in starts:
        q=lf[a]
        d=1
        while q not in starts:
            q=lf[q]
            d+=1
        lengths.append(d)
    assert sum(lengths)==n
    period = 1
    row = lf[p]
    while row != p:
        row = lf[row]
        period += 1
    assert period == n
    # Stable F order from the captured L, independently of the native tile builder.
    widths_by_symbol = [[] for _ in range(256)]
    for symbol, same in groupby(col):
        widths_by_symbol[symbol].append(sum(1 for _ in same))
    widths = [w for bucket in widths_by_symbol for w in bucket]
    cuts = list(accumulate(widths, initial=0))
    pieces, probes = price(cuts, 128)
    navigation_bytes = 4 * (pieces + (n + 127) // 128 + 256)
    return (len(starts), sum(lengths), max(lengths), 32, period, len(widths),
            pieces, navigation_bytes, probes)

if __name__=="__main__":
    tapes,runs,validation,evidence=map(Path,sys.argv[1:])
    with (evidence/"dependencies.csv").open("w") as f:
        w=csv.writer(f,lineterminator="\n")
        w.writerow("ordinal,rows,cuts,letters,top_headers,top_tagged,top_linked,top_slots,bottom_headers,bottom_tagged,bottom_linked,bottom_slots,max_top_linked_stretch,max_bottom_linked_stretch,free_pops,marked_spine,rules,commands".split(","))
        for p in sorted(tapes.glob("*.irec.xz")):
            rec=irec.load(p)
            run=runs/f'{rec.head["index"]:03}'
            if "info cells 1 " in (run/"ranges").read_text(): w.writerow(slot_counts(rec,run))
    with (evidence/"segments.csv").open("w") as f:
        w=csv.writer(f,lineterminator="\n")
        w.writerow(("ordinal","segments","row_links","longest_segment","lanes","primary_period",
                    "maximal_runs","tile_pieces","navigation_bytes","expected_probes"))
        for x in csv.DictReader((validation/"stock"/"rows.csv").open()):
            p=validation/f'{int(x["ordinal"]):03}.u16'
            if p.exists():
                vals=segments(p,int(x["rows"]),int(x["primary"]))
                w.writerow((x["ordinal"],*vals))
