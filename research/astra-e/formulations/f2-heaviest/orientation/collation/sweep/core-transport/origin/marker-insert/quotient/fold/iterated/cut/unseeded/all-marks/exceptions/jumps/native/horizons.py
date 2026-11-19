"""Count exact schedule actions from true marked streaks; crosscheck the existing kernel."""
import csv
import hashlib
import json
import subprocess
import sys
from pathlib import Path

here = Path(__file__).resolve().parent
exe, data = map(Path, sys.argv[1:])
aggregate = {(int(r['block']), int(r['variable'])): r
             for r in csv.DictReader((here / 'real.csv').open())}
receipt = next(p / 'powers/xor/capped/receipt.json' for p in here.parents
               if (p / 'powers/xor/capped/receipt.json').is_file())
print('block,variable,n,exceptions,positive_streaks,longest,octets,original,g4,binary,ideal8,ideal16,ideal32,unbounded')
for case in json.loads(receipt.read_text())['inputs']:
    block, mark = case['block'], case['mark']
    caps = data / f'block-{block}.caps'
    assert hashlib.sha256(caps.read_bytes()).hexdigest() == case['caps_le_sha256']
    for v in (0, 1):
        truth = aggregate[block, v]
        line = subprocess.check_output([exe, caps, truth['rows'], str(mark), str(v), str(block)], text=True).strip()
        counts = [int(k) for k in line.split(',')]
        assert counts[3] == int(truth['exception_steps'])
        assert counts[6] == int(truth['octets'])
        assert counts[7] == int(truth['octets']) + int(truth['units'])
        assert all(b <= a for a, b in zip(counts[7:], counts[8:]))
        print(line)
