"""Census real physical smooth endpoints and shifted sparse-exception source patterns."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

exe, data = (Path(a).resolve() for a in sys.argv[1:])
receipt = next(p/'powers/xor/capped/receipt.json' for p in Path(__file__).resolve().parents
               if (p/'powers/xor/capped/receipt.json').is_file())
print('block,k,kind,calls,bytes,short_calls,short_bytes,leading_bytes,all_destination_keys,short_keys,short_pool_bytes,prior_copies,prior_short_copies')
for case in json.loads(receipt.read_text())['inputs']:
    block = case['block']
    for ext, key in [('raw','raw_sha256'), ('caps','caps_le_sha256')]:
        assert hashlib.sha256((data/f'block-{block}.{ext}').read_bytes()).hexdigest() == case[key]
    for k in (4,8,16):
        print(subprocess.check_output([exe,data,str(block),str(case['mark']),str(k)],text=True),end='')
