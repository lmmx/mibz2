import itertools,json
from pathlib import Path

def scan(t,pmax):
 covered=set()
 for p in range(1,min(pmax,len(t)//2)+1):
  run=0;i0=0
  for i in range(len(t)-p+1):
   if i<len(t)-p and t[i]==t[i+p]:
    if run==0:i0=i
    run+=1
   else:
    if run>=p:covered.update(range(i0,i0+run+p))
    run=0
 return covered

def brute(t,pmax):
 covered=set()
 for a in range(len(t)):
  for b in range(a+1,len(t)+1):
   for p in range(1,min(pmax,(b-a)//2)+1):
    if all(t[i]==t[i-p] for i in range(a+p,b)):covered.update(range(a,b))
 return covered

checks=0
for n in range(1,8):
 for t in itertools.product(range(3),repeat=n):
  for p in [1,2,4,64]:assert scan(t,p)==brute(t,p);checks+=1
t='abcdXabcdY';n=len(t);rot=sorted(range(n),key=lambda i:t[i:]+t[:i]);l=''.join(t[(i-1)%n] for i in rot);psi=sorted(range(n),key=lambda q:(l[q],q));x=[];q=psi[rot.index(0)]
for _ in range(n):x.append(q);q=psi[q]
assert not scan(t,64)
a,b=x[0],x[5];u,v=a,b;word=[]
for _ in range(3):assert l[u]==l[v];word.append(l[u]);u,v=psi[u],psi[v];assert v-u==b-a
out={'tandem_scan_bruteforce_checks':checks,'non_tandem_copy':{'text':t,'L':l,'psi':psi,'states':x,'period_le64_covered':0,'source_position':0,'destination_position':5,'source_row':a,'destination_row':b,'word':''.join(word),'source_end':u,'destination_end':v,'certificate':'same three-step affine itinerary on rows [4,6), endpoint q-4'}}
Path(__file__).with_suffix('.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
