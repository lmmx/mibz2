"""Count actual component/size-table setup on the same supplied marked sections."""
import csv
import itertools
from pathlib import Path
import re
import struct
import sys
wires,dest=map(Path,sys.argv[1:])
rows=[]
for f in sorted(wires.glob("*.mask8"),key=lambda p:int(p.stem)):
    blob=f.read_bytes();assert blob[:8]==b"CLKMASK1"
    n,mark=struct.unpack_from("<2I",blob,8);last=blob[16:16+n];error=blob[16+n:];assert len(error)==n
    # Shift supplied true event flags by stable LF, using only their byte buckets.
    bucket=[bytearray() for _ in range(256)]
    for a,e in zip(last,error,strict=True):bucket[a].append(e)
    flags=b"".join(bucket)
    for section,selected in [("seed8",bytearray(flags)),("heads",bytearray([1])+bytearray(a!=b for a,b in zip(last[1:],last))),
                  ("prefix",bytearray([1])*sum(error)+bytearray(n-sum(error)))]:
        selected[mark]=1
        widths=[m.end()-m.start() for m in re.finditer(b"\x00+",selected)]
        rows.append(dict(block=int(f.stem),section=section,kept=sum(selected),complement_components=len(widths),
                distinct_widths=len(set(widths)),maximum_width=max(widths,default=0),
                rows_in_complement=sum(widths)))
with dest.open("w") as f:
    w=csv.DictWriter(f,list(rows[0]),lineterminator="\n");w.writeheader();w.writerows(rows)
