"""Compile the fixed free-index probe beside the identical scalar library."""
import json
from pathlib import Path
import subprocess as sp,sys
base,target,out=[Path(x).resolve() for x in sys.argv[1:]]
p=Path(__file__).resolve().parent;out.mkdir(parents=True,exist_ok=True)
main=out/'main.rs';main.write_text((base/'main.rs').read_text().split('fn main() {')[0]+(p/'main.rs').read_text())
sp.run(['g++','-O3','-std=c++20','-shared','-fPIC',p/'writer.cpp','-o',out/'libprobe.so'],check=True)
flags=json.loads((base/'build.json').read_text())['rust_flags']
sp.run(['rustc','-A','unused',*flags,main,'-L',f'dependency={target/"deps"}','--extern',f'mibz2={target/"deps/libmibz2.rlib"}','-L',f'native={out}','-l','dylib=probe','-C',f'link-arg=-Wl,-rpath,{out}','-o',out/'probe'],check=True)
