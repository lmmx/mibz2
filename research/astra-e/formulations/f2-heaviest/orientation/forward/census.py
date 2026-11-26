"""Price prefix discovery against the contiguous backward FM frontier and Direct band oracle."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[5]
sys.path.insert(0,str(HERE.parents[1]/"union"))
from direct import Column, Direct, heavy_set
sys.path.insert(0,str(ROOT/"docs/journal/2026-11-18-astra-c-output-first"))
from inputs import paths, read_input


def normalize(bands):
    out=[]
    for src,dest,w,word,color in sorted(bands):
        if out:
            a,b,n,old,k=out[-1]
            if (a+n,b+n,old,k)==(src,dest,word,color):
                out[-1]=(a,b,n+w,old,k)
                continue
        out.append((src,dest,w,word,color))
    return out


def backward_cost(col,section):
    # The same backward frontier recurrence used by the interval dictionary builder.
    # Terminal gaps in arbitrary row subsets need splitting; this audit uses cylinders.
    start,end=section
    live=[(start,end)]
    nodes=pieces=child_intervals=levels=0
    while live:
        levels+=1
        nxt=[]
        for l,h in live:
            nodes+=1
            children,n=col.children(l,h)
            pieces+=n
            child_intervals+=len(children)
            for a,b in children.values():
                if start<=a<end:
                    assert b<=end
                else:
                    assert b<=start or a>=end
                    nxt.append((a,b))
        live=nxt
    return nodes,pieces,child_intervals,levels


def one(binary,runs,n,mark,sections):
    header=f"{n} {mark} {len(runs)} {len(sections)}\n"
    payload=header+"".join(f"{a} {b}\n" for a,b in [*runs,*sections])
    return json.loads(subprocess.check_output([binary,"BANDS"],input=payload.encode()))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary",type=Path);p.add_argument("tapes",type=Path);p.add_argument("output",type=Path)
    a=p.parse_args()
    table={}
    with (HERE.parent/"wikidata.csv").open() as f:
        for r in csv.DictReader(f):
            if r["subset"]=="all": table[int(r["ordinal"]),r["section"]]=r
    writer=None
    with a.output.open("w") as f:
        for tape in paths(a.tapes):
            ordinal=int(tape.name.split(".")[0])
            runs,mark=read_input(tape);col=Column(runs);parent=col.interval(b'"')
            frequent,pops,search_pieces=heavy_set(col,parent,1)
            for name,sections in (("quote",[parent]),("frequent",frequent)):
                assert len(sections)==1
                direct=Direct(col,sections)
                res=one(a.binary.resolve(),runs,col.rows,mark,sections)
                incoming=[(src,d,w,bytes.fromhex(word),c) for src,d,w,word,c in res.pop("bands")]
                assert normalize(direct.bands)==normalize(incoming)
                output=direct.expand(mark)
                assert output is not None and len(output)==col.rows
                assert bytes.fromhex(res.pop("expanded_hex"))==output
                nf,parts,children,levels=backward_cost(col,sections[0])
                oracle=table[ordinal,name]
                assert res["word_count"]==int(oracle["words"])==len(direct.word_counts)
                assert res["spelling_bytes"]==int(oracle["bytes"])
                assert res["groups"]==int(oracle["forward"])
                assert nf==int(oracle["backward"]) and levels==res["max_depth"]
                assert res["live_merges"]==0
                row=dict(ordinal=ordinal,section=name,section_lo=sections[0][0],section_hi=sections[0][1],
                         mark=mark,runs=len(runs),tape_sha256=hashlib.sha256(tape.read_bytes()).hexdigest(),
                         output_sha256=hashlib.sha256(output).hexdigest(),**res,
                         backward_nodes=nf,backward_original_run_intersections=parts,backward_child_intervals=children,
                         direct_hops=direct.visits,direct_raw_bands=len(direct.bands),matched_canonical_bands=len(normalize(incoming)),
                         selection_heap_pops=pops if name=="frequent" else 0,
                         selection_intersections=search_pieces if name=="frequent" else 0)
                if writer is None:
                    writer=csv.DictWriter(f,fieldnames=row,lineterminator="\n");writer.writeheader()
                writer.writerow(row);f.flush()
                print(ordinal,name,"prefix nodes",res["groups"],"packets",res["packet_inputs"],
                      "pieces",res["select_translations"],"backward nodes/pieces",nf,parts,flush=True)

if __name__=="__main__":
    main()
