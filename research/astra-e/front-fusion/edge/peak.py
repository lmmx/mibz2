"""Build a fresh one-arm process without allocating the diagnostic stock reserve vectors."""
from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent;p=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"memory.py"),str(p)],check=True)
s=(p/"main.rs").read_text()
s=s.replace("let mut caps=Vec::<u16>::new();let mut one=Vec::<u64>::new();let mut lit=Vec::<u64>::new();","let mut caps=0usize;let mut bits=0usize;")
s=s.replace("caps.reserve(m+symbols.max_len/127+3);one.reserve(m/64+3);lit.reserve(m/64+3);","let need=m+symbols.max_len/127+3;if caps<need{caps=need.max(2*caps).max(4);}let need=m/64+3;if bits<need{bits=need.max(2*bits).max(4);}")
s=s.replace("2*caps.capacity()+8*(one.capacity()+lit.capacity())","2*caps+16*bits")
(p/"main.rs").write_text(s)
