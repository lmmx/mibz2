"""Compare every physical jump against a dense oracle on all ten authenticated capped inputs."""
import csv,hashlib,json,subprocess,sys
from pathlib import Path
gate,data=Path(sys.argv[1]).resolve(),Path(sys.argv[2]).resolve()
here=Path(__file__).resolve().parent
receipt=here.parents[4]/'powers/xor/capped/receipt.json'
if not receipt.is_file():
    receipt=next(p/'powers/xor/capped/receipt.json' for p in here.parents if (p/'powers/xor/capped/receipt.json').is_file())
cases=json.loads(receipt.read_text())['inputs']
print("block,variable,rows,components,exception_rows,g8_rows,octets,units,exception_steps,index_bytes,index_build_peak,geometry_bytes,run_capacity")
for q in cases:
    for suf,key in [('caps','caps_le_sha256'),('raw','raw_sha256')]:
        assert hashlib.sha256((data/f"block-{q['block']}.{suf}").read_bytes()).hexdigest()==q[key]
    for variant in [0,1]:
        print(subprocess.check_output([gate,data,str(q['block']),str(q['mark']),str(variant)],text=True,stderr=sys.stderr).strip())
