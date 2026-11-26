"""Verify the in-place terminal representation against the original indexed representation."""
import argparse
import csv
import hashlib
import json
import subprocess
from pathlib import Path
from census import Column,heavy_set,paths,read_input

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary",type=Path);p.add_argument("tapes",type=Path);p.add_argument("output",type=Path)
    a=p.parse_args(); records=[]
    for path in paths(a.tapes):
        ordinal=int(path.name.split(".")[0]);runs,mark=read_input(path)
        col=Column(runs);parent=col.interval(b'"')
        for section,ints in (("quote",[parent]),("frequent",heavy_set(col,parent,1)[0])):
            data=(f"{col.rows} {mark} {len(runs)} {len(ints)}\n"+"".join(f"{x} {y}\n" for x,y in [*runs,*ints])).encode()
            ordinary,recycled=(json.loads(subprocess.check_output([a.binary.resolve(),mode],input=data))
                               for mode in ("INDEXED_BANDS","RECYCLED_BANDS"))
            assert recycled["word_index_capacity_bytes"]==0 and ordinary["word_index_capacity_bytes"]>0
            size_keys=[k for k in recycled if k.endswith("capacity_bytes")]
            assert all(ordinary[k]==recycled[k] for k in recycled if
                       not k.endswith("_ms") and k!="word_index_capacity_bytes")
            assert all(ordinary[k]==recycled[k] for k in size_keys if k!="word_index_capacity_bytes")
            records.append(dict(ordinal=ordinal,section=section,tagged_bands=recycled["bands_count"],
                                output_sha256=hashlib.sha256(bytes.fromhex(recycled["expanded_hex"])).hexdigest(),
                                old_retained_bytes=sum(ordinary[k] for k in size_keys),
                                recycled_retained_bytes=sum(recycled[k] for k in size_keys),
                                saved_bytes=ordinary["word_index_capacity_bytes"]))
    with a.output.open("w") as f:
        w=csv.DictWriter(f,fieldnames=records[0],lineterminator="\n");w.writeheader();w.writerows(records)

if __name__=="__main__":
    main()
