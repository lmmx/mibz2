#!/usr/bin/env python3
"""Independent saved-profile aggregate cross-check; no project execution."""
import argparse
import gzip
import json
import re
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('evidence_root', type=Path)
root = parser.parse_args().evidence_root
base = root / 'docs/journal/2026-11-04-compact30-scanner-prototype'
lines = gzip.open(base / 'callgrind/per-instruction.callgrind.gz', 'rt').read().splitlines()
names = dict(re.findall(r'^c?fn=\((\d+)\) (.+)$', '\n'.join(lines), re.M))
selected = {
    '<c30walk::ctor2::Ctor2>::build::<false>': 'v2',
    '<c30walk::ctor2s::Ctor2s>::build::<false>': 'scanner',
}
out = {n: {'self': 0, 'call_edges': 0} for n in selected.values()}
function = None
edge = False
for line in lines:
    if line.startswith('fn='):
        match = re.match(r'fn=\((\d+)\)', line)
        function = names[match[1]] if match else line[3:]
    elif line.startswith('calls='):
        edge = True
    elif re.match(r'^(?:0x[0-9a-f]+|\*|[+-]?\d+)\s', line):
        # Profile declares two positions and a single Ir event. Aggregate
        # checking needs no compressed-address decoder or listing classifier.
        fields = line.split()
        assert len(fields) == 3
        if function in selected:
            out[selected[function]]['call_edges' if edge else 'self'] += int(fields[2])
        edge = False
for row in out.values():
    row['inclusive'] = row['self'] + row['call_edges']
out['saved'] = {k: out['v2'][k] - out['scanner'][k] for k in out['v2']}
print(json.dumps(out, indent=2, sort_keys=True))
