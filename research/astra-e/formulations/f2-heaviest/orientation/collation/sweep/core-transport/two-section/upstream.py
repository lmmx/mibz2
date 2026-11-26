"""Two frozen integrations with the run-native core discovery, independent of dense fixture names."""
import csv
import hashlib
from pathlib import Path
import sys

from check import Atlas, Column, Lift, fixture, geometry, heavy_set, plain, read_input, source_mark
from stock import Stock
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from return_map import incoming


def main(tape, library):
    runs,mark=read_input(Path(tape))
    col=Column(runs)
    last=b''.join(bytes([c])*w for c,w in runs)
    psi,_=geometry(last)
    expected=plain(last,psi,mark)
    q=col.interval(b'"')
    stock=Stock(library)
    writer=None
    for name,u in [('frequent',heavy_set(col,q,1)[0][0]),('quote',q)]:
        atlas=Atlas(col,u,4)
        root,words,minima,mass,cost=incoming(col,u,atlas)
        lift=Lift(u,atlas.leaves,root,words,minima)
        row,skip=source_mark(col,mark,u,atlas.leaves)
        regular,used,_=stock(lift,row,skip,col.rows)
        capped,forced,_=stock(lift.cap_prefixes(),row,skip,col.rows)
        assert regular==capped==expected
        dense,_,dense_min,_=fixture(last,psi,u,4)
        assert [(words[c],w) for c,w in root]==dense
        assert {words[c]:v for c,v in minima.items()}==dense_min
        record=dict(section=name,h=4,core_types=len(words),core_runs=len(root),**cost,
                    stock_second=used,capped_second=forced,mark=row,skip=skip,
                    marked_bytes=col.rows,sha256=hashlib.sha256(expected).hexdigest())
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=record,lineterminator='\n')
            writer.writeheader()
        writer.writerow(record)
        sys.stdout.flush()


if __name__=='__main__':
    main(*sys.argv[1:])
