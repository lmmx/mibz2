from collections import Counter
import csv
import hashlib
from itertools import groupby, product
from pathlib import Path
import sys
from direct import Column, Direct, heavy_set
from audit import Oracle


def tiny():
    matches = refused = splits = 0
    for n in range(1,9):
        for last in product(range(2), repeat=n):
            raw = bytes(last)
            col = Column([(b,len(list(g))) for b,g in groupby(last)])
            oracle = Oracle(raw)
            for p in (b"\0", b"\0\0", b"\0\1", b"\1\0"):
                base = col.interval(p)
                if base[0] == base[1]:
                    continue
                parent = Direct(col, [base])
                for k in (1,2,3):
                    chosen,_,_ = heavy_set(col,base,k)
                    obj = Direct(col, chosen)
                    h = len(obj.components)
                    assert obj.plain_runs <= parent.plain_runs+h-1
                    assert len(obj.bands) <= obj.plain_runs+h-1
                    splits += bool(obj.split_plain)
                    for mark in range(n):
                        got = obj.expand(mark)
                        in_section = any(oracle.mark(mark,s)>=0 for s in obj.components)
                        assert (got is not None) == in_section
                        if got is None:
                            refused += 1
                        else:
                            row = mark
                            want = bytearray()
                            for _ in range(n):
                                want.append(oracle.first[row]); row=oracle.forward[row]
                            assert want == got
                            matches += 1
    print(f"{matches} exact marked inverses, {refused} disjoint-cycle refusals, {splits} selected unions split an uncolored source label")


def real(folder):
    root=Path(__file__).resolve().parents[5]
    old=root/"docs/journal/2026-11-18-astra-c-output-first"
    sys.path.insert(0,str(old))
    from inputs import paths, read_input
    with (old/"section.csv").open() as h:
        expected={int(r["block"]):r["marked_sha256"] for r in csv.DictReader(h)}
    writer=None
    for path in paths(folder):
        ordinal=int(path.name.split(".")[0])
        runs,p=read_input(path)
        col=Column(runs)
        base=col.interval(b'"')
        parent=Direct(col,[base])
        for k in (1,2,4,8):
            selected,pops,pieces=heavy_set(col,base,k)
            obj=Direct(col,selected)
            digest=hashlib.sha256(obj.expand(p)).hexdigest()
            assert digest == expected[ordinal]
            h=len(obj.components)
            assert obj.plain_runs <= parent.plain_runs+h-1
            assert len(obj.bands) <= obj.plain_runs+h-1
            row=dict(ordinal=ordinal,k=k,rows=col.rows,source_runs=len(runs),parent_runs=parent.plain_runs,
                     components=h,section_rows=sum(b-a for a,b in selected),plain_runs=obj.plain_runs,
                     colored_runs=len(obj.bands),split_plain_words=obj.split_plain,words=len(obj.word_counts),
                     distinct_spelling=sum(map(len,obj.word_counts)),lf_probes=obj.visits,heap_pops=pops,
                     select_pieces=pieces,sha256=digest)
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
        sys.stdout.flush()


if __name__ == "__main__":
    tiny() if len(sys.argv)==1 else real(sys.argv[1])
