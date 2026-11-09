"""Finite proof checks for forward affine itinerary classes and non-tandem reuse."""
import itertools,json
from pathlib import Path
total=0;example=None
for n in range(1,8):
 for l in itertools.product(range(3),repeat=n):
  psi=sorted(range(n),key=lambda q:(l[q],q));r=1+sum(a!=b for a,b in zip(l,l[1:]))
  for s in range(1,5):
   keys=[];words=[];ends=[]
   for q in range(n):
    y=q;word=[];offset=[]
    for j in range(s):word.append(l[y]);y=psi[y];offset.append(y-q)
    keys.append((tuple(word),tuple(offset)));words.append(word);ends.append(y)
   cuts=[0]+[i for i in range(1,n) if keys[i]!=keys[i-1]]+[n]
   assert len(cuts)-1<=1+2*s*(r-1)
   for a,b in zip(cuts,cuts[1:]):
    for q in range(a,b):
     assert words[q]==words[a] and ends[q]==ends[a]+q-a;total+=1
   if s>=3 and example is None:
    ids=[0]*n
    for i,(a,b) in enumerate(zip(cuts,cuts[1:])):ids[a:b]=[i]*(b-a)
    for origin in range(n):
     x=[origin]
     for _ in range(n):x.append(psi[x[-1]])
     out=[l[q] for q in x[:-1]]
     for i in range(n-s+1):
      for j in range(i+s,n-s+1):
       if x[i]!=x[j] and ids[x[i]]==ids[x[j]] and j>i+s and len(set(out[i:i+s]))>1:
        example={'L':l,'origin':origin,'s':s,'source_offset':i,'destination_offset':j,'source_row':x[i],'destination_row':x[j],'word':out[i:i+s],'output':out,'source_end_row':x[i+s],'destination_end_row':x[j+s]};break
      if example:break
     if example:break
out={'affine_class_row_checks':total,'bound':'cells <= 1+2*s*(r-1), s=1..4; ternary length1..7','nonadjacent_reuse_example':example}
Path(__file__).with_suffix('.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
