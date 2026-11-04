#!/usr/bin/env python3
"""S62-F04: the harness call site of build::<false> in `c30walk::main`, with each instruction's
own execution count in the two archived profiles (v2 alone, candidate alone).

    python3 callsite.py BINARY V2_PROFILE CAND_PROFILE CAND_BUILD_SYMBOL

BINARY is the profiled binary. Prints the listing of `main` from the first instruction whose
count differs between the profiles (less 16 instructions of context) through the instruction
after both `build::<false>` calls, every line annotated `v2-count cand-count`, then the
per-instruction delta total (it must equal main's own-cost delta).
"""
import re, subprocess, sys
from f01_replay import profile

binary, pv, pcand, sym = sys.argv[1:5]
a, b = profile(pv), profile(pcand)
A, B = a['addresses']['c30walk::main'], b['addresses']['c30walk::main']
diff = {x: B.get(x, {}).get('Ir', 0) - A.get(x, {}).get('Ir', 0) for x in set(A) | set(B)}
diff = {x: v for x, v in diff.items() if v}
sites = [s['address'] for s in a['sites']['c30walk::main'] if s['callee'].endswith('::Ctor2>::build::<false>')]
sites += [s['address'] for s in b['sites']['c30walk::main'] if s['callee'] == sym]
lo, hi = min(min(diff), min(sites)), max(max(diff), max(sites))
nm = subprocess.run(['nm', '-C', '-S', binary], capture_output=True, text=True, check=True).stdout
start, size = next((int(p[0], 16), int(p[1], 16)) for p in (l.split(None, 3) for l in nm.splitlines())
                   if len(p) == 4 and p[3] == 'c30walk::main')
text = subprocess.run(['objdump', '-d', '-C', '--no-show-raw-insn', f'--start-address=0x{start:x}',
                       f'--stop-address=0x{start + size:x}', binary], capture_output=True, text=True, check=True).stdout
text = '\n'.join(l for l in text.splitlines()
                 if (m := re.match(r'^ *([0-9a-f]+):', l)) and lo - 0x48 <= int(m.group(1), 16) <= hi + 0x10)
print(f'# c30walk::main, harness call site of build::<false>; columns: address, own count (v2 alone), own count ({sym} alone), instruction')
for line in text.splitlines():
    m = re.match(r'^ *([0-9a-f]+):\t(.*)$', line)
    if m:
        x = int(m.group(1), 16)
        print(f"{x:8x}  {A.get(x, {}).get('Ir', 0):>6} {B.get(x, {}).get('Ir', 0):>6}  {m.group(2)}")
own = (a['own']['c30walk::main']['Ir'], b['own']['c30walk::main']['Ir'])
print(f'# main own Ir {own[0]} -> {own[1]} ({own[1] - own[0]:+}); per-instruction deltas sum {sum(diff.values()):+} '
      f'over {len(diff)} instructions; every other instruction of main equal')
assert sum(diff.values()) == own[1] - own[0]
