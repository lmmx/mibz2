"""Aggregate pinned, previously recorded counts; no new decoder run.
Fetch the three JSONL inputs from 25481a03d4fae5aa6405ffebb8a5b343efe9b8bc
at the paths named in report.md, and place them beside this script.
"""
import json
from pathlib import Path
base=Path(__file__).parent
def read(f):return [json.loads(s) for s in (base/f).read_text().splitlines() if s.strip()]
a=read('beta_1.5_wikidata.jsonl');stopped=[x for x in a if x['beta_stop'][0]]
r=sum(x['r'] for x in a);w=sum(x['alloc'] for x in a)
saved=sum(x['alloc']-x['beta_stop'][2] for x in stopped)
out={'stop':dict(blocks=len(a),stopped=len(stopped),r=r,word_bytes=w,saved=saved,bytes_per_r=saved/r,fraction=saved/w),
 'points_per_r_stopped':sum(x['beta_stop'][0] for x in stopped)/sum(x['r'] for x in stopped)}
a=[x for x in read('tail_wikidata.jsonl') if x['switch']=='1.2'];r=sum(x['r'] for x in a);w=sum(x['alloc'] for x in a)
saved=sum(x['alloc']-x['made_at_switch']-(4 if x['tailc'][3]<=65536 else 6)*x['tailc'][0]-8*x['tailc'][1] for x in a)
out['tail']=dict(blocks=len(a),saved=saved,bytes_per_r=saved/r,fraction=saved/w,wide_id_blocks=sum(x['tailc'][3]>65536 for x in a))
a=[x for x in read('certificates_wikidata_every8.jsonl') if x['beta']=='1.5'];m=sum(x['m'] for x in a)
out['certificates']=dict(blocks=len(a),start_weighted_share=sum(x['m']*x['certificates'][2][2] for x in a)/m,scope='64-step cap, no end wrap; rounded per-block inputs')
out['original_census']=dict(S=27439723,r=15141602,n=209697662,S_per_r=27439723/15141602,S_per_n=27439723/209697662)
print(json.dumps(out,indent=2))
