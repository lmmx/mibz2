"""Independently pair the library's original scanner, queue and worker timeline marks."""
import csv
import gzip
import json
from pathlib import Path
import statistics
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[3]/"bench/moonshot"))
from moonshot import paired_ratio
NIL=2**64-1

def marks(path,workers):
    with gzip.open(path,"rt") as f:
        frames=[dict(thread=r["thread"],kind=r["kind"],**{k:int(v) for k,v in r.items() if k not in ("thread","kind")})for r in csv.DictReader(f)]
    scanner=[m for m in frames if m["thread"]=="mibz2-scan"]
    assert scanner
    def boundaries(seq,begin,end,key):
        # Closed durations only. SlotGot before the terminal Done has no associated data block.
        opening=None
        result=[]
        for m in seq:
            if m["kind"]==begin:
                assert opening is None
                opening=m
            elif m["kind"] in end and opening is not None:
                assert m[key]!=NIL and opening[key]!=NIL and m[key]>=opening[key]
                result.append(m[key]-opening[key]);opening=None
        return result
    by_kind={}
    for m in frames:
        if m["place"]!=NIL:
            items=by_kind.setdefault(m["kind"],{})
            if m["kind"] in ("Issued","Enqueued","JobGot","DecodeStart","DecodeEnd","Ready","Taken"):
                assert m["place"] not in items
                items[m["place"]]=m
    expected=set(range(862))
    for kind in ("Issued","Enqueued","JobGot","DecodeStart","DecodeEnd","Ready","Taken"):
        assert by_kind[kind].keys()==expected, (kind,len(by_kind[kind]))
    assert sum(m["len"]for m in by_kind["DecodeEnd"].values())==419400392
    assert sum(m["len"]for m in by_kind["Issued"].values())==2*24979299*2
    worker_names={m["thread"]for m in by_kind["DecodeStart"].values()}
    assert len(worker_names)==workers
    def role_cpu(seq):
        begins=[m for m in seq if m["kind"]=="ThreadStart"]
        ends=[m for m in seq if m["kind"]=="ThreadEnd"]
        assert len(begins)==len(ends)==1 and begins[0]["cpu_ns"]!=NIL
        return ends[0]["cpu_ns"]-begins[0]["cpu_ns"]
    def pointwise(a,b,field):
        return [by_kind[b][j][field]-by_kind[a][j][field] for j in range(862)]
    # Ready is stamped after inserting and unlocking. Taken may be stamped before Ready.
    # Decoder has completed by DecodeEnd. Compute that observable interval separately.
    scan=boundaries(scanner,"SlotGot",{"Issued","ThreadEnd"},"ns")
    slot=boundaries(scanner,"SlotWait",{"SlotGot"},"ns")
    inputs=boundaries(scanner,"InputWait",{"InputGot","InputEnd"},"ns")
    handoff=pointwise("Issued","JobGot","ns")
    published=pointwise("DecodeEnd","Taken","ns")
    assert min(handoff)>=0 and min(published)>=0
    assert len(scan)==len(slot)==863
    return dict(blocks=862,scanner_role_cpu_s=role_cpu(scanner)*1e-9,scan_with_input_wall_s=sum(scan)*1e-9,
                scanner_slotwait_wall_s=sum(slot)*1e-9,scanner_inputwait_wall_s=sum(inputs)*1e-9,
                issued_to_job_mean_us=statistics.mean(handoff)*1e-3,issued_to_job_median_us=statistics.median(handoff)*1e-3,
                worker_role_cpu_sum_s=sum(role_cpu([m for m in frames if m["thread"]==t]) for t in worker_names)*1e-9,
                worker_decode_cpu_sum_s=sum(pointwise("DecodeStart","DecodeEnd","cpu_ns"))*1e-9,
                worker_decode_wall_sum_s=sum(pointwise("DecodeStart","DecodeEnd","ns"))*1e-9,
                finished_to_taken_mean_us=statistics.mean(published)*1e-3)

def main(out):
    runs=list(map(json.loads,(out/"runs.jsonl").read_text().splitlines()))
    rows=[]
    for r in runs:
        i,w,a=r["round"],r["workers"],r["source"]
        rows.append(dict(round=i,workers=w,source=a,wall_s=r["wall_s"],cpu_s=r["cpu_s"],
                         **marks(out/f"marks-{i}-{w}-{a}.csv.gz",w)))
    with (out/"roles.csv").open("w") as f:
        writer=csv.DictWriter(f,fieldnames=rows[0],lineterminator="\n");writer.writeheader();writer.writerows(rows)
    sums={}
    for w in (1,4):
        ws=[r for r in rows if r["workers"]==w];sums[w]={}
        for k in list(rows[0])[3:]:
            if k=="blocks":continue
            legs={a:[r[k] for r in ws if r["source"]==a]for a in ("phrase","stock")}
            sums[w][k]=dict(stock_median=statistics.median(legs["stock"]),phrase_median=statistics.median(legs["phrase"]),
                            ratio=paired_ratio(legs["phrase"],legs["stock"]))
    (out/"role-summary.json").write_text(json.dumps(sums,indent=2)+"\n")

if __name__=="__main__":main(Path(sys.argv[1]))
