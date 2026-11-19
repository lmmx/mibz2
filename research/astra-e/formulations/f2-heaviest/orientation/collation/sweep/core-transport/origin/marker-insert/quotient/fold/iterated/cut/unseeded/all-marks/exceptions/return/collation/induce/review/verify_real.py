"""Reconcile the independent order counts with the original linked-list checker."""
import csv
import json
from pathlib import Path
import subprocess as sp
import sys

d=Path(sys.argv[1]).resolve()
ref="cd35d808"
root=Path(sp.check_output(["git","-C",d,"rev-parse","--show-toplevel"],text=True).strip())
path=(d.parent/"blocks.csv").relative_to(root)
old={(q["block"],q["policy"]):q for q in csv.DictReader(sp.check_output(["git","show",f"{ref}:{path}"],cwd=root,text=True).splitlines())}
rows=list(csv.DictReader((d/"orders.csv").open()))
def val(q,k):return int(q[k])
count=0;groups={}
for r in rows:
    if r["merged"]=="0":
        key=(r["block"],r["section"])
        a,b=val(r,"initial_atoms"),val(r,"final_bands")
        assert (a,b)==groups.setdefault(key,(a,b))
        assert val(r,"clipped")==0
        assert val(r,"steps")>=max(a-b,val(r,"stabbing_step_floor"),val(r,"distinct_touching_bigrams"))
        if r["policy"]=="stable":
            f=old[key]
            for a,b in [("kept","marked_section"),("coordinate_seams","section_seams"),
                        ("initial_atoms","initial_arcs"),("final_bands","final_arcs")]:
                assert val(r,a)==val(f,b)
            assert val(r,"steps")==sum(val(f,k) for k in ("strict","equal","self","terminal_partial"))
            count+=1
assert len(rows)==300 and count==len(groups)==30
sort=list(csv.DictReader((d/"sort-cost.csv").open()))
for q in sort:
    assert val(q,"kept")+val(q,"rows_in_complement")==val(old[q["block"],q["section"]],"n")
audit=dict(rows=len(rows),stable_models_identical_to_pinned_parent=count,paired_layout_sets=len(groups),
           atom_only_stabbing_equals_dense_models=300,
           hist_source=sp.check_output(["git","rev-parse",ref],cwd=root,text=True).strip())
(d/"audit.json").write_text(json.dumps(audit,indent=2)+"\n")
