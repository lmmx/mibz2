"""Reproduce history prices using only original runs, selected intervals and marked oracle output."""
import argparse
import csv
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"native"))
from real import read_input, paths, Column, heavy_set, dense_audit, check_frozen

FIELDS="""ordinal section page states heap_pops stale_repairs edge_row_steps live_fragments clipped_edges
cohorts published_fragments fresh_interior_rows interior_meeting_rows root_meeting_rows archive_fragments
history_lookups descriptors_examined bitmap_word_reads bitmap_word_writes section_tests lazy_skip_ranges
lazy_skipped_rows mass_zero_packet_drops packet_pool_peak packet_reuses archive_capacity_bytes
page_heads_capacity_bytes bitmap_capacity_bytes constructor_peak_bytes marked_peak_bytes constructor_cpu_s
output_cpu_s""".split()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary",type=Path);p.add_argument("tapes",type=Path);p.add_argument("out",type=Path)
    a=p.parse_args()
    with (HERE.parent/"native/wikidata.csv").open() as f:
        old={(r["ordinal"],r["section"]):r for r in csv.DictReader(f)}
    with a.out.open("w") as f,tempfile.TemporaryDirectory() as tmp:
        writer=csv.DictWriter(f,fieldnames=[*FIELDS,"tape_sha256","marked_sha256"],lineterminator="\n");writer.writeheader()
        for path in paths(a.tapes):
            ordinal=int(path.name.split(".")[0]);runs,mark=read_input(path);col=Column(runs)
            column,marked,_=dense_audit(runs,mark);check_frozen(ordinal,column,marked,mark)
            quote=col.interval(b'"');frequent=heavy_set(col,quote,1)[0]
            head=f"{col.rows} {mark} {len(runs)} 2\n"
            raw=Path(tmp)/"input";gold=Path(tmp)/"gold"
            raw.write_text(head+"".join(f"{x} {y}\n" for x,y in runs)+
                           f"quote 1 {quote[0]} {quote[1]}\nfrequent 1 {frequent[0][0]} {frequent[0][1]}\n")
            gold.write_bytes(marked)
            lines=subprocess.check_output([a.binary.resolve(),raw,gold,str(ordinal)]).decode().splitlines()
            assert len(lines)==6
            for row in csv.DictReader(lines,fieldnames=FIELDS):
                base=old[row["ordinal"],row["section"]]
                for key in ("states","heap_pops","stale_repairs","edge_row_steps"):
                    assert int(row[key])==int(base[key]),(ordinal,row["section"],key)
                assert int(row["fresh_interior_rows"])+int(base["endpoints"])==int(row["edge_row_steps"])
                assert int(row["interior_meeting_rows"])+int(row["root_meeting_rows"])==int(base["endpoints"])
                writer.writerow(dict(row,tape_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                                     marked_sha256=hashlib.sha256(marked).hexdigest()))
            f.flush();print(ordinal,flush=True)
if __name__=="__main__":
    main()
