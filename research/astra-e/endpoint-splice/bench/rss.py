"""Fresh cap-only inverse high-water, with the authenticated raw reference loaded last."""
from pathlib import Path
import csv
import hashlib
import json
import subprocess as sp
import sys
here=Path(__file__).resolve().parent
repo=here.parents[3]
d,donor=map(lambda x:Path(x).resolve(),sys.argv[1:3])
receipt=next((repo/"research/astra-e/formulations").rglob("powers/xor/capped/receipt.json"))
inputs=json.loads(receipt.read_text())["inputs"]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for q in inputs:
    b=q["block"]
    assert sha(donor/f"block-{b}.caps")==q["caps_le_sha256"] and sha(donor/f"block-{b}.raw")==q["raw_sha256"]
arms=["control","replica","batched"]
with (d/"rss.csv").open("w") as f:
    wr=csv.writer(f,lineterminator="\n");wr.writerow(["round","block","arm","rows","pre_hwm_kib","post_hwm_kib"])
    for lap in range(3):
        for q in inputs:
            b=q["block"]
            for slot in range(3):
                a=arms[(b+lap+slot)%3]
                result=sp.check_output([d/"panel","rss",donor/f"block-{b}.caps",str(q["rows"]),str(q["mark"]),donor/f"block-{b}.raw",a],text=True)
                pre,post=map(int,result.strip().split(","))
                assert pre<=post;wr.writerow([lap,b,a,q["rows"],pre,post])
