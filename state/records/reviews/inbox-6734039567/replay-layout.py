import hashlib,json,math,statistics
from pathlib import Path
assert hashlib.sha256(Path('direct.zip').read_bytes()).hexdigest()=='00b0e008bdd8a90108ccaa87e21467dd35d70df35e1b71b86e687078a64a3e94'
x=json.loads(Path('direct-evidence/run-fed-1.json').read_text())
f=x['files'][0]
assert x['rounds']==15 and len(f['entry'])==15
gm=lambda xs:math.exp(statistics.mean(map(math.log,xs)))
out={'B_A':gm([sum(r[1])/sum(r[0]) for r in f['entry']]),
'counterfactual_B_A_equal_fallback':gm([(r[1][0]+r[0][1])/sum(r[0]) for r in f['entry']]),
'A_fallback_share_geomean':gm([r[0][1]/sum(r[0]) for r in f['entry']]),
'fallback_B_A':gm([r[1][1]/r[0][1] for r in f['entry']]),
'interpretation':'Arithmetic counterfactual replacing B fallback seconds with same-round A fallback seconds; not causal correction, new timing or measured gain.'}
print(json.dumps(out,indent=2))
