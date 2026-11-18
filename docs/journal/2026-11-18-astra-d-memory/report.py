"""Publish portable denominators, an exact frozen ledger and the marginal prices."""
import csv
import sys
from collections import Counter,defaultdict
from pathlib import Path

here=Path(__file__).resolve().parent
e=Path(sys.argv[1])
def table(path):
    """Read a CSV file as a list of rows."""
    with path.open() as f: return list(csv.DictReader(f))
def write(path,rows):
    """Write rows to a CSV file."""
    with path.open("w") as f:
        w=csv.DictWriter(f,fieldnames=rows[0],lineterminator="\n")
        w.writeheader();w.writerows(rows)
def group(x):
    """The ledger group of a region."""
    n=x["region"]
    return {"mtf_steps":"MTF and counts","mtf_bits":"MTF and counts","mtf_rows":"MTF and counts","mtf_counts":"MTF and counts",
            "run_counts":"MTF and counts","scratch_counts":"MTF and counts","entries":"entry, byte, bottom",
            "front_bytes":"entry, byte, bottom","bottom":"entry, byte, bottom","tops":"top slots",
            "final":"final output","inverse":"inverse output","cellbuf":"other","rle":"inverse output"}.get(n,n)
f=defaultdict(Counter)
for x in table(e/"stock-regions.csv"):
    if x["ordinal"]=="1":
        f[group(x)].update({k:int(x[k]) for k in x if k not in ("ordinal","llc_KiB","region","phase")})
def lines(c): return c["dram_read"]+c["dram_rfo"]+c["dram_wb"]
def frozen(g,c): return dict(region=g,**c,model_bytes_per_output=f'{64*lines(c)/900000:.5f}')
out=[frozen(g,c) for g,c in sorted(f.items())]
out.append(frozen("TOTAL",sum(f.values(),Counter())))
write(e/"frozen-stock.csv",out)

inputs={int(x["ordinal"]):x for x in table(here/"inputs.csv")}
arms=("stock","direct","tiles")
blocks={}
for mode in arms:
    blocks[mode]={}
    for x in table(e/(mode+"-blocks.csv")):
        y={k:int(v) for k,v in x.items()}
        blocks[mode][y["ordinal"],y["llc_KiB"]]=y
out=[];prices=[]
samples={"frozen":[1],"systematic":[k for k in inputs if k!=270],
         "Cells":[k for k in inputs if blocks["stock"][k,2048]["cells"]],
         "slot eligible":[k for k in inputs if blocks["direct"][k,2048]["direct_scan"]],
         "systematic T":[41],"T control":[270]}
for name,ids in samples.items():
    for kib in (1024,2048,4096):
        total={}
        for mode in arms:
            c=sum((Counter(blocks[mode][i,kib]) for i in ids),Counter())
            total[mode]=c; n=c["output"]
            out.append(dict(sample=name,blocks=len(ids),llc_KiB=kib,arm=mode,output=n,
                input_plus_output=f'{1+sum(int(inputs[i]["envelope_bytes"]) for i in ids)/n:.7f}',
                dram_B_per_B=f'{64*lines(c)/n:.7f}',l2_requests_per_B=f'{c["l2_req"]/n:.7f}',
                l2_misses_per_B=f'{c["l2_miss"]/n:.7f}',
                l1_l2_B_per_B=f'{64*(c["l2_req"]+c["l1_wb"])/n:.7f}',
                l2_llc_B_per_B=f'{64*(c["llc_req"]+c["l2_wb"])/n:.7f}',
                guest_I_per_B=f'{c["instructions"]/n:.7f}'))
        for mode in arms[1:]:
            b,c=total["stock"],total[mode]
            saved=64*(lines(b)-lines(c)); added=c["instructions"]-b["instructions"]
            hits=lambda t:t["llc_req"]-t["llc_miss"]
            nh=hits(c)-hits(b)
            if kib!=2048:continue
            def price(w,bw):
                """The net modelled saving at a bandwidth."""
                return w*4.8/bw*saved - 0.5*added
            prices.append(dict(sample=name,arm=mode,saved_bytes=saved,extra_guest_I=added,
                extra_LLC_hits=nh,extra_I_per_saved_B=f'{added/saved:.5f}' if saved>0 else "",
                seven_margin_cycles=f'{price(7,24.8):.1f}',ten_margin_cycles=f'{price(10,28.6):.1f}',
                seven_max_extra_cycles_per_net_hit=f'{price(7,24.8)/nh:.3f}' if nh>0 and saved>0 and price(7,24.8)>0 else ""))
write(e/"summary.csv",out);write(e/"prices.csv",prices)

# These independent counts test the native site attribution, including aliasing.
dep={int(x["ordinal"]):x for x in table(e/"dependencies.csv")}
tile={int(x["ordinal"]):x for x in table(here/"tile-screen.csv")
      if x["tile"]=="128" and x["direction"]=="psi"}
for s in table(e/"segments.csv"):
    i = int(s["ordinal"])
    control = {"header_loads": s["primary_period"], "descriptor_probes": s["expected_probes"]}
    if i in tile:
        assert all(tile[i][key] == value for key, value in control.items())
    else:
        tile[i] = control

def lookup(mode):
    """Per-region load counts for a mode."""
    return {(int(x["ordinal"]),int(x["phase"]),x["region"]):int(x["loads"])
            for x in table(e/(mode+"-regions.csv"))}
stock=lookup("stock"); tiled=lookup("tiles")
proof=[]
for i,x in tile.items():
    row=dict(ordinal=i,header_reads=tiled[i,17,"headers"],expected_headers=int(x["header_loads"]),
             piece_reads=tiled[i,17,"pieces"],expected_pieces=int(x["descriptor_probes"]))
    assert row["header_reads"]==row["expected_headers"] and row["piece_reads"]==row["expected_pieces"]
    for name,phase,field in [("rule",6,"rules"),("copy",7,"commands")]:
        row[name+"_position_reads"]=stock.get((i,phase,"positions"),0)
        row["expected_"+field]=int(dep[i][field]) if i in dep else 0
        assert row[name+"_position_reads"]==row["expected_"+field]
    proof.append(row)
write(e/"crosschecks.csv",proof)
