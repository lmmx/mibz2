import csv
from pathlib import Path
import subprocess
import sys

def run(exe, wires):
    out = csv.writer(sys.stdout,lineterminator="\n")
    out.writerow("block policy n original_runs marked_section section_seams initial_arcs strict equal self terminal_partial joins live_joins final_arcs canonical_literal_id_arcs unique_arc_tree_visits union_lap_bytes marked_bytes".split())
    for block in sorted(int(p.stem) for p in wires.glob("*.mask8")):
        data = subprocess.check_output([exe,wires/f"{block}.mask8"],text=True)
        out.writerows((block,*r.split(",")) for r in data.splitlines())

if __name__=="__main__":
    run(*map(Path,sys.argv[1:]))
