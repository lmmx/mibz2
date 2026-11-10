import re,json,collections
from pathlib import Path
ROOT = Path(__file__).resolve().parent / 'raw'
def parse(path):
 names={}; own=collections.Counter(); arcs=collections.Counter(); calls=collections.Counter()
 fn=None; callee=None; pending=None
 for line in open(path):
  line=line.strip()
  m=re.match(r'(c?fn)=\((\d+)\)(?: (.*))?$',line)
  if m:
   tag,idx,name=m.groups()
   if name:names[idx]=name
   if tag=='fn':fn=idx
   else:callee=idx
   continue
  if line.startswith('calls='):
   pending=int(line.split()[0][6:]);continue
  if re.match(r'^(?:\d+|[+-]\d+|\*)\s+\d+',line):
   cost=int(line.split()[1])
   if pending is not None:
    arcs[fn,callee]+=cost;calls[fn,callee]+=pending;pending=None
   else:own[fn]+=cost
 out=[]
 for idx,cost in own.items():
  name=names.get(idx,idx)
  if any(s in name for s in ['room_segments','mtf_avx2','measured','memset','front_avx2']):
   out.append(dict(name=name,self_ir=cost,inclusive_ir=cost+sum(v for (a,b),v in arcs.items() if a==idx),
    callees=[dict(name=names.get(b,b),ir=v,calls=calls[a,b]) for (a,b),v in arcs.items() if a==idx]))
 return out
print(json.dumps({p:parse(ROOT / p) for p in ["rescue-base-random.txt","rescue-head-random.txt","rescue-base-records.txt","rescue-head-records.txt"]},indent=2))
