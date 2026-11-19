"""Run four complete marked inverses on identical systematic runs and expected text."""
import argparse
import csv
from pathlib import Path
import subprocess
import sys
import tempfile

from real import check_frozen, dense_audit, paths, read_input

HERE = Path(__file__).resolve().parent
HEAD = """ordinal,section,rep,arm,total_cpu_s,construct_cpu_s,indexed_emit_cpu_s,process_hwm_kib,construct_requested_bytes,full_requested_bytes,states,heap_pops,published_bands,summary_reads,mask_writes,intrapage_hops,source_page_increments,tree_compares"""
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary",type=Path)
    parser.add_argument("tapes",type=Path)
    parser.add_argument("dest",type=Path)
    parser.add_argument("--reps",type=int,default=8)
    a=parser.parse_args()
    a.dest.mkdir(exist_ok=True,parents=True)
    laps=a.dest/"paired.csv"; rss=a.dest/"fresh.csv"
    for target in laps,rss: target.write_text(HEAD+"\n")
    manifest={int(row["ordinal"]):row for row in csv.DictReader((HERE/"inputs.csv").open())}
    with tempfile.TemporaryDirectory() as tmp:
        for path in paths(a.tapes):
            ordinal=int(path.name.split(".")[0]);m=manifest[ordinal]
            runs,mark=read_input(path);raw, expected, _=dense_audit(runs,mark); check_frozen(ordinal,raw,expected,mark)
            fixture=Path(tmp,"runs.txt");gold=Path(tmp,"gold");gold.write_bytes(expected)
            with fixture.open("w") as txt:
                print(len(raw),mark,len(runs),2,file=txt)
                txt.writelines(f"{b} {w}\n" for b,w in runs)
                for sec in "quote","frequent": print(sec,1,m[f"{sec}_lo"],m[f"{sec}_hi"],file=txt)
            base=[a.binary.resolve(),fixture,gold,str(ordinal)]
            subprocess.run([*base,str(a.reps),laps],check=True)
            for sec in "quote","frequent":
                for arm in "tree","paged","direct","compact":
                    subprocess.run([*base,"1",rss,arm,sec],check=True)
            print(f"paired {ordinal:04}",file=sys.stderr,flush=True)
if __name__=="__main__":
    main()
