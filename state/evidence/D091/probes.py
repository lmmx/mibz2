import json, subprocess, sys
from pathlib import Path
import walk_stats_check as wc
import test_walk_stats_check as fixture

root=Path(__file__).resolve().parent
out={}
for name, amounts in [('balanced',(100,100,100)),('balance_only',(100,1,99)),('zero_helpers',(100,0,0))]:
    rows=[]
    for h in (1,2): rows.append(dict(kind='share',team=0,block=0,thread=h))
    for t,n in enumerate(amounts):
        rows.append(dict(kind='call',team=0,block=0,thread=t,role='leader' if t==0 else 'helper',walker='traverse',takes=int(n>0),exhausts=1,bytes=n,tail=0,fragments=0,copied=0))
    rows.append(dict(kind='summary',team=0,block=0,rows=sum(amounts),starts=sum(n>0 for n in amounts),seg_bytes=sum(amounts),entered=7,joined=True,built=6))
    lines=fixture.lines(rows)
    by,bad=wc.validate(lines,{0:'T'},2)
    assert not bad,bad
    preds=wc.predicates(by,'traverse',2)
    (root/(name+'.jsonl')).write_text('\n'.join(lines)+'\n')
    (root/'classes.txt').write_text('T\n')
    p=subprocess.run([sys.executable,str(root/'walk_stats_check.py'),str(root/(name+'.jsonl')),'--walker','traverse','--helpers','2','--classes',str(root/'classes.txt')],capture_output=True,text=True)
    accepted=all(preds[k][0] for k in ['a','c','d'])
    out[name]={'structural_valid':not bad,'predicates':preds,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr,'section8_accepts':accepted,'section3_accepts_exit':p.returncode==0}
assert out['balanced']['exit']==0
assert out['balance_only']['exit']==1 and out['balance_only']['section8_accepts']
assert out['zero_helpers']['exit']==1 and out['zero_helpers']['section8_accepts']
(root/'probes.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
