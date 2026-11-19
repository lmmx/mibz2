"""Build the signed phase graphs only from source caps and verify their frozen identity."""
from pathlib import Path
import csv,hashlib,json,subprocess,sys
exe,data=map(Path,sys.argv[1:])
receipt=next(p/'powers/xor/capped/receipt.json' for p in Path(__file__).resolve().parents
             if (p/'powers/xor/capped/receipt.json').is_file())
print('block,n,runs,overlay,levels,signed_pairs,fixed,equal_level,guarantee_numerator_of_four,greedy_removed,exact_removed,separable_crossing_upper,greedy_phases,exact_phases,independent_tile_crossing_upper,unaggregated_bias_upper')
print('block,lo,hi,mass,raw_different',file=sys.stderr)
for q in json.loads(receipt.read_text())['inputs']:
    caps=data/f"block-{q['block']}.caps"
    assert hashlib.sha256(caps.read_bytes()).hexdigest()==q['caps_le_sha256']
    a=subprocess.run([exe,caps,str(q['rows']),str(q['mark']),str(q['block'])],check=True,capture_output=True,text=True)
    print(a.stdout,end='');print(a.stderr,end='',file=sys.stderr)
