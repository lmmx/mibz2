"""Exercise concurrent pools in one address space and validate both worker rosters."""
from collections import Counter
import bz2
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory

with TemporaryDirectory() as tmp:
    d=Path(tmp)
    raw=(b'{"a":true,"b":1141,"c":"en","d":"hello"}\n'*2331+b"xyz"*517+b"\x00"*789)*13
    (d/"raw").write_bytes(raw)
    (d/"data.bz2").write_bytes(bz2.compress(raw,compresslevel=1))
    proc=subprocess.run([sys.argv[1],d/"data.bz2",d/"raw"],capture_output=True,text=True,check=True)
    assert set(proc.stdout.splitlines())=={f"W7 exact {len(raw)} bytes",f"W10 exact {len(raw)} bytes"}
    roster=Counter()
    for line in proc.stderr.splitlines():
        if line.startswith("dose-init "):
            _,worker,role=line.split()
            i=int(worker.removeprefix("Some(").removesuffix(")"))
            assert role==("eager" if i%4==0 else "edge")
            roster[i]+=1
    assert roster==Counter([*range(7),*range(10)])
    print(f"W7 and W10 concurrent pools: {sum(roster.values())} matched instances, {len(raw)} exact bytes per pool")
