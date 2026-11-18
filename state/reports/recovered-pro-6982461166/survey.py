from collections import Counter
from csv import DictReader, writer
from hashlib import sha256
from json import loads
from pathlib import Path
from statistics import median
from struct import unpack
import sys

from structure import endpoint, isolated, strip_table


spec=loads(Path("wikidata.json").read_text())
for window in spec["windows"]:
    path=Path(window["name"]+".bin")
    payload=path.read_bytes()
    assert len(payload)==window["length"] and sha256(payload).hexdigest()==window["sha256"]
expected=[row for row in DictReader(Path("source-blocks.csv").open()) if row["corpus"]=="wikidata"]
assert len(expected)==sum(w["blocks"] for w in spec["windows"])==431

out=writer(sys.stdout)
out.writerow(["ordinal","n","mark","isolated","old","new","relative_reduction"])
count=0
values=[]
while header:=sys.stdin.buffer.read(8):
    n,p=unpack("<II",header)
    column=sys.stdin.buffer.read(n)
    assert len(column)==n==int(expected[count]["n"])
    if count==1:
        assert p==264045 and sha256(column).hexdigest()=="db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67"
    if n>=65536:
        strips=strip_table(column,p,16)
        dead,revised=isolated(n,strips)
        old_nodes,old=endpoint(n,strips)
        new_nodes,new=endpoint(n-dead,revised)
        assert (len(strips),old_nodes,old)==tuple(int(expected[count][k]) for k in ("H","open_nodes","steps"))
        v=(old-new)/old
        out.writerow([count,n,p,dead,old,new,v])
        values.append(v)
    count+=1
assert count==431 and len(values)==233
print("large blocks",len(values),"min/median/max",min(values),median(values),max(values),
      "signs",Counter((r>0)-(r<0) for r in values),file=sys.stderr)
