"""Check phased whole-kernel identities and report the 24-pair geometric CPU ratios."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics as st
import struct

p=argparse.ArgumentParser()
p.add_argument("output",type=Path)
p.add_argument("corpus",type=Path)
p.add_argument("donor",type=Path)
a=p.parse_args()
here=Path(__file__).resolve().parent
sha=lambda data:hashlib.sha256(data).hexdigest()
old=[json.loads(line) for line in (here.parent.parent/"gated/python.jsonl").read_text().splitlines()][1:]
historical=json.loads(next(v for v in here.parents if (v/"research").is_dir()).joinpath("research/astra-e/results/cap-counter-route-pricing/manifest.json").read_text())
pin=historical["arms"]["scalar"]
b=json.loads((a.output/"build.json").read_text())
assert b["donor"]==pin["pin"] and b["src_tree"]==pin["src_tree"]
for rel,digest in pin["research_source_sha256"].items():assert sha((a.donor/rel).read_bytes())==digest
digest=sha(a.corpus.read_bytes())
inputs=[]
for ref in old:
    i=ref["original_block_index"]
    raw=(a.output/f"block-{i}.raw").read_bytes();assert sha(raw)==ref["original_inverse_sha256"]
    cap=(a.output/f"block-{i}.caps").read_bytes()
    records=[c for (c,) in struct.iter_unpack("<H",cap)]
    runs=1+sum(x>>8!=y>>8 for x,y in zip(records,records[1:]))
    assert len(raw)==sum(x&255 for x in records)==ref["source_bytes"] and runs==ref["source_runs"]
    assert ref["corpus_sha256"]==digest and ref["libbzip2_match"]
    inputs.append({"block":i,"mark":ref["mark"],"rows":len(raw),"records":len(records),
                   "maximal_runs":runs,"caps_le_sha256":sha(cap),"raw_sha256":sha(raw),
                   "decoded_sha256":ref["decoded_sha256"],"decoded_bytes":ref["decoded_bytes"],
                   "block_crc":ref["block_crc"]})
timing={}
slots={}
for t in csv.DictReader((a.output/"times.csv").open()):
    i=int(t["block"]);arm=t["arm"];r=int(t["round"]);s=int(t["slot"])
    key=(arm,i,r)
    assert key not in timing and 0<=s<6
    assert not (slots.setdefault((i,r),0)&(1<<s));slots[i,r]|=1<<s
    timing[key]=int(t["thread_ns"])
    ref=next(v for v in inputs if v["block"]==i)
    assert (int(t["mark"]),int(t["rows"]),int(t["cap_records"]),int(t["maximal_runs"]))==(ref["mark"],ref["rows"],ref["records"],ref["maximal_runs"])
assert len(timing)==1440 and len(slots)==240 and set(slots.values())=={63}
arms=["stream","streamgaps","dense","scalar","lazy","opend"]
def quantity(ids,arm,r):return sum(timing[arm,i,r] for i in ids)
def stats(ids,x,y):
    logs=[math.log(quantity(ids,x,r)/quantity(ids,y,r)) for r in range(1,25)]
    avg=st.mean(logs);radius=2.068657610419*st.stdev(logs)/math.sqrt(len(logs))
    return [round(math.exp(p),6) for p in (avg,avg-radius,avg+radius)]
results={}
for key,ids in [("all",[v["block"] for v in inputs])]+[(str(v["block"]),[v["block"]]) for v in inputs]:
    results[key]={"median_ms":{arm:round(st.median(quantity(ids,arm,r) for r in range(1,25))/1e6,5) for arm in arms},
                  "ratios":{f"{x}/{y}":stats(ids,x,y) for x in ("stream","streamgaps") for y in arms if y not in (x,"streamgaps")}}
summary={"preregistration":"923f215","corpus_sha256":digest,"checked_reference_sources":pin["research_source_sha256"],
         "inputs":inputs,"timings":results}
(a.output/"receipt.json").write_text(json.dumps(summary,indent=2)+"\n")
print(results["all"])
