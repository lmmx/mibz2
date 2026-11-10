import sys,tempfile,json,random
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import test_service_tg as f
import service_tg as t
out={}
def shuffled(rows):
 groups=[rows[i:i+6] for i in range(0,len(rows),6)]
 random.Random(64).shuffle(groups)
 return sum(groups,[])
for name,change in [('clean',lambda r:r),('shuffle_rounds',shuffled),('reverse_rounds',lambda r:sum([r[i:i+6] for i in reversed(range(0,len(r),6))],[])),('threads_reversed',lambda r:sum([r[i+4:i+6]+r[i+2:i+4]+r[i:i+2] for i in range(0,len(r),6)],[]))]:
 with tempfile.TemporaryDirectory() as root:
  rows=change(f.shard_rows('runs'));m=f.build(root,runs=[{'rows':rows}])
  v=t.ni_judge(root,m,f.HEAD,reg=f.REG,check_git=False,rounds=f.R,batches=f.B)
  out[name]={'valid':v['valid'],'problems':v['problems'],'condition2':v['condition2'],'first_rows':[(r['round'],r['threads'],r['arm']) for r in rows[:6]]}
# Also exercise the production default R/B, with consistent synthetic observation times.
f.R=t.NI_ROUNDS
with tempfile.TemporaryDirectory() as root:
 m=f.build(root,runs=[{'rows':shuffled(f.shard_rows('runs'))}])
 for p in Path(root).glob('*/*/row-times.txt'):
  n=len(p.read_text().splitlines())-1
  p.write_text(f'# start {f.T0 + .5:.3f}\n'+''.join(f'{i} {f.T0+1+i*.01:.3f}\n' for i in range(n)))
 m=f.remanifest(root)
 v=t.ni_judge(root,m,f.HEAD,reg=f.REG,check_git=False)
 out['shuffle_rounds_R2880_default']={'valid':v['valid'],'problems':v['problems'],'condition2':v['condition2'],'batches':v['batches']}
print(json.dumps(out,indent=2));Path(__file__).with_name('probe-results.json').write_text(json.dumps(out,indent=2)+'\n')
