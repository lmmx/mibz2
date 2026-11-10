import json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import test_two_modes as f
import two_modes as tm
r=f.Recheck()
out={}
for name,edit in [('baseline',None),('extra_C',lambda rows:rows[1].update(condition='C')),('all_treatments_C',lambda rows:[x.update(condition='C') for x in rows[1:]])]:
    rc,j=r.replay(edit)
    out[name]={'rc':rc,'label':j['label'],'problems':j['problems'],'gap_outcome':j['gaps']['outcome'],'G_n':j['gaps']['conditions']['G']['n'],'Z_n':j['gaps']['conditions']['Z']['n']}
assert out['baseline']['label']=='VALID'
assert out['extra_C']['label']=='VALID' and out['extra_C']['G_n']==29
assert out['all_treatments_C']['label']=='VALID' and out['all_treatments_C']['G_n']==0
print(json.dumps(out,indent=2))
