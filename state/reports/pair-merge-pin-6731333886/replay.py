"""Reproduce the synthetic-text byte census from Git blob sizes at both commits.
No decoder, compressor or benchmark runs. input-manifest.json is from recursive Git trees.
"""
import json
from pathlib import Path
x=json.loads((Path(__file__).parent/'input-manifest.json').read_text())
for k in ['old','new']:
 files=x[k+'_inputs']
 seed=sum(f['size'] for f in files)
 size=seed;i=0
 while size<3000000:
  size+=len(f'\n-- copy {i} --\n'.encode())+seed
  i+=1
 actual={'seed_bytes':seed,'extra_copies':i,'total_bytes':size,'files':len(files)}
 assert actual==x[k],(k,actual,x[k])
 print(k,actual)
assert x['old']['total_bytes']!=x['new']['total_bytes']
print('Text workload differs by',x['new']['total_bytes']-x['old']['total_bytes'],'bytes')
