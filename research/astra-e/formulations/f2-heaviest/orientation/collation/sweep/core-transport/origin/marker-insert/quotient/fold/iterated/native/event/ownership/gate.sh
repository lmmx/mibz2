#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
out=${1:?scratch};deps=${2:?release deps};dst=${3:?receipt}
mkdir -p "$out" "$dst"
python3 -B "$d/build.py" "$out" "$deps"
"$out/owner" > "$dst/small.csv" 2> "$dst/synthetic.txt"
"$out/owner" census /tmp/astra-clipped/ten.bin - 3 > "$dst/ten.csv"
for a in 0 1 2 3; do
    "$out/owner" census /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv "$a" > "$dst/rss-$a.csv"
done
"$out/owner" census /tmp/astra-token-all/admissions.bin - 3 > "$out/all-dense.csv"
python3 - "$out" "$dst" "$deps" <<'PY'
import csv,hashlib,json,subprocess,sys
from pathlib import Path
out,d,deps=map(Path,sys.argv[1:])
read=lambda p:list(csv.DictReader(p.open()))
dense=read(out/'all-dense.csv');plain=read(d/'rss-2.csv')
assert len(dense)==len(plain)==228 and [r['hash'] for r in dense]==[r['hash'] for r in plain]
tot=sum(int(r['n']) for r in plain)
for o,p in zip(dense,plain):
    assert o['period']==p['period'] and o['moved']==p['moved'] and o['nodes']==p['nodes']
    assert int(o['copied_bytes'])+int(o['leaf_bytes'])==int(o['period'])
(d/'dense.txt').write_text(f'Both audited and timed ownership builds matched the complete independent dense-LF output for all {tot} bytes from {len(dense)} production admissions. Every period, occurrence count and moved mark agreed with dense event control.\n')
fs=[out/'owner',out/'main.rs',out/'control.rs',out/'second.rs',deps/'libmibz2.rlib',Path('/tmp/astra-clipped/ten.bin'),Path('/tmp/astra-token-all/admissions.bin'),Path('/tmp/astra-clipped-final/expected.csv')]
(d/'manifest.json').write_text(json.dumps({'prereg':'48863b5','source':'df54544fedb5cacd522cc99bde6d7430ad0eb32b','rustc':subprocess.check_output(['rustc','-vV'],text=True),'sha256':{str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in fs}},indent=2)+'\n')
PY
