#!/usr/bin/env python3
"""Archive a function's listing in the components' format (`# SYMBOL`, `# size 0x…`, then
objdump -d -C --no-show-raw-insn over its nm -C -S extent).

    python3 listing.py BINARY OUT_FILE SYMBOL
"""
import subprocess, sys
binary, out, sym = sys.argv[1:4]
nm = subprocess.run(['nm', '-C', '-S', binary], capture_output=True, text=True, check=True).stdout
hits = [(int(p[0], 16), int(p[1], 16)) for p in (l.split(None, 3) for l in nm.splitlines())
        if len(p) == 4 and p[2] in 'tTwW' and p[3] == sym]
assert len(hits) == 1, (sym, hits)
start, size = hits[0]
text = subprocess.run(['objdump', '-d', '-C', '--no-show-raw-insn', f'--start-address=0x{start:x}',
                       f'--stop-address=0x{start + size:x}', binary], capture_output=True, text=True, check=True).stdout
open(out, 'w').write(f'# {sym}\n# size 0x{size:x}\n' + text)
n = sum(1 for l in text.splitlines() if l.startswith(' ') and ':\t' in l)
print(f'{sym}: 0x{start:x} size 0x{size:x}, {n} instructions -> {out}')
