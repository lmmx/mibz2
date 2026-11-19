"""Extract only the original L runs and reproduce the named compound by bounded search."""
from pathlib import Path
import csv
import hashlib
import sys

HERE=Path(__file__).resolve().parent
root=HERE.parents[7]
sys.path.insert(0,str(root/"docs/journal/2026-11-18-astra-c-output-first"))
sys.path.insert(0,str(HERE.parents[3]/"union"))
from inputs import paths, read_input, dense_audit
from direct import Column, heavy_set

def main():
    directory, out = map(Path,sys.argv[1:])
    with (HERE.parents[2]/"seams/async/native/inputs.csv").open() as f:
        frozen={int(row["ordinal"]):row for row in csv.DictReader(f)}
    with out.open("w") as f:
        print("ordinal,tape_sha256,marked_sha256,n,mark,lo,hi,selection_pops,selection_run_pieces")
        for path in paths(directory):
            runs,p=read_input(path);col=Column(runs)
            win,pops,pieces=heavy_set(col,col.interval(b'"'),1);(a,b),=win
            idx=int(path.name.split(".")[0]);rec=frozen[idx]
            digest=hashlib.sha256(path.read_bytes()).hexdigest()
            assert (digest,a,b)==(rec["tape_sha256"],int(rec["frequent_lo"]),int(rec["frequent_hi"]))
            _,gold,_=dense_audit(runs,p);actual=hashlib.sha256(gold).hexdigest()
            assert actual==rec["marked_sha256"]
            print(f"{idx},{digest},{actual},{col.rows},{p},{a},{b},{pops},{pieces}")
            f.write(f"{idx} {col.rows} {p} {a} {b} {len(runs)}\n"+"".join(f"{c} {w}\n" for c,w in runs))

if __name__=="__main__":
    main()
