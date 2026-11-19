"""Rebuild against the identical frozen Rust library and research arm as the capped stage."""
import argparse
import json
from pathlib import Path
import subprocess as sp

p=argparse.ArgumentParser()
p.add_argument("capped",type=Path);p.add_argument("target",type=Path);p.add_argument("out",type=Path)
a=p.parse_args();base=a.capped.resolve();out=a.out.resolve();target=a.target.resolve()
here=Path(__file__).resolve().parent;out.mkdir(parents=True,exist_ok=True)
common=(base/"main.rs").read_text().split("fn main() {")[0]
main=out/"main.rs";main.write_text(common+(here/"main.rs").read_text())
sp.run(["g++","-O3","-std=c++20","-shared","-fPIC",here/"prefix.cpp","-o",out/"libprefix.so"],check=True)
flags=json.loads((base/"build.json").read_text())["rust_flags"]
sp.run(["rustc","-A","unused",*flags,main,"-L",f"dependency={target/'deps'}","--extern",f"mibz2={target/'deps/libmibz2.rlib'}",
        "-L",f"native={out}","-l","dylib=prefix","-C",f"link-arg=-Wl,-rpath,{out}","-o",out/"panel"],check=True)
