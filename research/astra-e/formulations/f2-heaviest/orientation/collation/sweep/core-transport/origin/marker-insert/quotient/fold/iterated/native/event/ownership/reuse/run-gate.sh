#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
out=${1:?scratch}; deps=${2:?release deps}; dst=${3:?receipt directory}
mkdir -p "$out" "$dst"
python3 -B "$d/build.py" "$out" "$deps"
"$out/reuse" > "$dst/small.csv" 2> "$dst/synthetic.txt"
for a in 0 1 2 3; do
    "$out/reuse" census /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv "$a" > "$dst/rss-$a.csv"
done
"$out/reuse" census /tmp/astra-token-all/admissions.bin - 3 > "$out/all-bytes.csv"
python3 - "$out" "$dst" "$deps" <<'PY'
from pathlib import Path
import csv,hashlib,json,subprocess,sys
out,d,deps=map(Path,sys.argv[1:])
rd=lambda f:list(csv.DictReader(f.open()))
a=rd(out/'all-bytes.csv');base,side,other,reuse=[rd(d/f'rss-{i}.csv') for i in range(4)]
assert len(a)==len(base)==len(side)==len(reuse)==228
for x,b,s,r in zip(a,base,side,reuse):
    assert x['hash']==b['hash']==s['hash']==r['hash']
    for key in ['period','nodes','owner_slots','published','hits','copied_bytes','partial_hits','visits']:
        assert s[key]==r[key]
    assert r['owner_bytes']=='0' and int(r['borrowed_bytes'])>=4*int(r['owner_slots'])
    assert int(r['total_capacity'])<=int(b['total_capacity'])
(d/'gate.txt').write_text(f'All {sum(int(x["n"]) for x in a)} bytes independently checked in both code paths on 228 inputs. All 228 per-block retained capacities no higher than unowned dense, identical emitted ownership metrics vs side table and zero owner-table capacity.\n')
files=[out/'reuse',out/'main.rs',out/'control.rs',deps/'libmibz2.rlib',Path('/tmp/astra-clipped/ten.bin'),Path('/tmp/astra-token-all/admissions.bin')]
(d/'manifest.json').write_text(json.dumps({'prereg':'25ecb6c','control':'ac7763a','sha256':{str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in files},'rustc':subprocess.check_output(['rustc','-vV'],text=True)},indent=2)+'\n')
PY
