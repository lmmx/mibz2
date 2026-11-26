"""Fresh-process VmHWM for each paid kernel on one actual recovered block."""
import argparse
import csv
import json
import os
from pathlib import Path
import subprocess as sp
import tempfile

p=argparse.ArgumentParser()
p.add_argument("corpus",type=Path)
p.add_argument("target",type=Path)
p.add_argument("stage",type=Path)
a=p.parse_args()
stage=a.stage.resolve()
target=a.target.resolve()
here=Path(__file__).resolve().parent
receipt=json.loads((stage/"receipt.json").read_text())
manifest=json.loads((stage/"build.json").read_text())
source=(stage/"main.rs").read_text().split("fn main() {")[0]+(here/"rss.rs").read_text()
main=stage/"rss-main.rs";main.write_text(source)
sp.run(["rustc","-A","unused",*manifest["rust_flags"],main,"-L",f"dependency={target/'deps'}",
        "--extern",f"mibz2={target/'deps/libmibz2.rlib'}","-L",f"native={stage}","-l","dylib=grammar",
        "-C",f"link-arg=-Wl,-rpath,{stage}","-o",stage/"rss"],check=True)
arms=["stream","streamgaps","dense","scalar","lazy","opend"]
with tempfile.TemporaryDirectory(dir=stage) as tmp, (stage/"rss.csv").open("w") as f:
    folder=Path(tmp);(folder/"sample.bz2").symlink_to(a.corpus.resolve())
    sp.run(["bzip2recover","sample.bz2"],cwd=folder,stdout=sp.DEVNULL,stderr=sp.DEVNULL,check=True)
    out=csv.writer(f,lineterminator="\n");out.writerow(["round","block","arm","mark","rows","caps","pre_kernel_hwm_kib","post_kernel_hwm_kib"])
    for rnd in range(3):
        for src in receipt["inputs"]:
            i=src["block"]
            for shift in range(len(arms)):
                arm=arms[(shift+rnd+i)%len(arms)]
                cmd=[stage/"rss",folder/f"rec{i+1:05d}sample.bz2",arm,stage/f"block-{i}.raw"]
                mark,rows,caps,before,peak=map(int,sp.check_output(cmd,text=True).strip().split(","))
                assert (mark,rows,caps)==(src["mark"],src["rows"],src["records"])
                out.writerow([rnd,i,arm,mark,rows,caps,before,peak])
