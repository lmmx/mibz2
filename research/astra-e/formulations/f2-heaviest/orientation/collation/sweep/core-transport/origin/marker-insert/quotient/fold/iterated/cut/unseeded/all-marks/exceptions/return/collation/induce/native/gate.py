"""Check all native outputs and original sparse selectors on authenticated real cap files."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
here=Path(__file__).resolve()
exe, data=(Path(x).resolve() for x in sys.argv[1:])
packet=next(p/'powers/xor/capped/receipt.json' for p in here.parents if (p/'powers/xor/capped/receipt.json').is_file())
print('block,arm,n,primary,kept,spans,arcs,joins,lap,reused_vector_bytes,live_with_result,orbit_returns,visited_joins')
for case in json.loads(packet.read_text())['inputs']:
    b=case['block']
    for ext, key in [('caps','caps_le_sha256'),('raw','raw_sha256')]:
        assert hashlib.sha256((data/f'block-{b}.{ext}').read_bytes()).hexdigest()==case[key]
    print(subprocess.check_output([exe,data,str(b),str(case['mark'])],text=True),end='')
