import csv
from pathlib import Path
import subprocess
import sys

def run(exe,wires):
    out=csv.writer(sys.stdout,lineterminator="\n")
    out.writerow("block q level n kept input_bands residue_pieces fragments row_visits joins output_bands binary_probes max_pending max_macro_depth tree_joins terminal_visits cached_bytes".split())
    for block in sorted(int(p.stem) for p in wires.glob("*.mask8")):
        data=subprocess.check_output([exe,wires/f"{block}.mask8"],text=True)
        out.writerows((block,*r.split(",")) for r in data.splitlines())

if __name__=="__main__":
    run(*map(Path,sys.argv[1:]))
