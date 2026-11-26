#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
out=${1:?fresh scratch directory}
deps=${2:?release deps}
dst=${3:?receipt directory}
mkdir -p "$out" "$dst"
python3 -B "$d/build.py" "$out" "$deps"
"$out/ribbons" > "$dst/small.csv"
"$out/ribbons" census /tmp/astra-clipped/ten.bin - 4 > "$dst/ten-flat.csv"
"$out/ribbons" census /tmp/astra-clipped/ten.bin - 5 > "$dst/ten-tree.csv"
for a in 0 1 2 3 4 5; do
    "$out/ribbons" census /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv "$a" > "$dst/rss-$a.csv"
done
for a in 4 5; do
    "$out/ribbons" census /tmp/astra-token-all/admissions.bin - "$a" > "$out/dense-$a.csv"
done
python3 - "$out" "$dst" "$deps" <<'PY'
import csv,hashlib,json,sys,subprocess
from pathlib import Path
t,d,deps=map(Path,sys.argv[1:])
r=lambda f:list(csv.DictReader(f.open()))
total=0
for arm in [4,5]:
    inp=r(t/f'dense-{arm}.csv');original=r(d/'rss-3.csv')
    assert len(inp)==len(original)==228
    for aa,bb in zip(inp,original):
        for k in ['stream','n','hash','live','events','edits','moved','queue_pushes','pops','peak_queue','nodes','period']:
            assert aa[k]==bb[k],(arm,k,aa,bb)
    total=sum(int(a['n']) for a in inp)
(d/'dense.txt').write_text(f'{total} output bytes independently compared in full on all 228 retained blocks in both the audit and the paid specialization for each sparse backend. Terminal, edits, event count, immutable node count, queue sequence aggregate counts, transferred marks, marked period and output digest agree with dense FIFO on every block.\n')
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
paths=[deps/'libmibz2.rlib',t/'ribbons',t/'control.rs',t/'main.rs',t/'second.rs',Path('/tmp/astra-clipped/ten.bin'),Path('/tmp/astra-token-all/admissions.bin'),Path('/tmp/astra-clipped-final/expected.csv')]
(d/'manifest.json').write_text(json.dumps({'prereg':'68d260e','dense_control':'df54544fedb5cacd522cc99bde6d7430ad0eb32b','rustc':subprocess.check_output(['rustc','-vV'],text=True),'sources_and_inputs':{str(p):h(p) for p in paths}},indent=2)+'\n')
PY
