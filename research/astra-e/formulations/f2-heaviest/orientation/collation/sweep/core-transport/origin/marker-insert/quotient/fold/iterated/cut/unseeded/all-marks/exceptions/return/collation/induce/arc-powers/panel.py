from pathlib import Path
import csv
import subprocess
import sys

def run(exe,wires):
    out=csv.writer(sys.stdout,lineterminator="\n")
    out.writerow("block stage mass bands fixed_rows self_removed power_rows power_bands pairs independent_cut triples output_bands radix_visits grammar_nodes marked_lap".split())
    for b in sorted(int(p.stem) for p in wires.glob("*.mask8")):
        data=subprocess.check_output([exe,wires/f"{b}.mask8"],text=True)
        out.writerows((b,*r.split(",")) for r in data.splitlines())

if __name__=="__main__":
    run(*map(Path,sys.argv[1:]))
