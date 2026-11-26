import csv
from pathlib import Path
import subprocess
import sys

def run(exe,wires):
    out=csv.writer(sys.stdout,lineterminator="\n")
    out.writerow("block q level n kept old_bands residue_pieces strict equal self partial final_bands marked_lap terminal_visits cached_copies cached_bytes tree_joins".split())
    for block in sorted(int(p.stem) for p in wires.glob("*.mask8")):
        data=subprocess.check_output([exe,wires/f"{block}.mask8"],text=True)
        out.writerows((block,*r.split(",")) for r in data.splitlines())

if __name__=="__main__":
    run(*map(Path,sys.argv[1:]))
