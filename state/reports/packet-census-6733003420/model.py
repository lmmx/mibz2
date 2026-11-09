"""Independent finite checks; no decoder execution or timing."""
import itertools,json
from pathlib import Path

def model(l,sb):
 n=len(l);psi=sorted(range(n),key=lambda q:(l[q],q));sig=[psi[psi[q]] if sb==2 else psi[q] for q in range(n)]
 keys=[(sig[q]-q, (l[q],l[psi[q]]) if sb==2 else (l[q],)) for q in range(n)]
 cuts=[0]+[q for q in range(1,n) if keys[q]!=keys[q-1]]+[n]
 cells=[(a,b,*keys[a]) for a,b in zip(cuts,cuts[1:])];ids=[0]*n
 for i,(a,b,*_) in enumerate(cells):ids[a:b]=[i]*(b-a)
 return psi,sig,cells,ids

def paths(cells,ids,q,rem,sb,kmax):
 lo,hi=-10**9,10**9;s=0;y=q;word=[];path=[]
 for k in range(1,kmax+1):
  cid=ids[y];a,b,d,w=cells[cid];path.append(cid);word+=w
  lo=max(lo,a-s);hi=min(hi,b-s);s+=d;y+=d
  reps=(hi-1-q)//s+1 if s>0 else (q-lo)//(-s)+1 if s<0 else 10**9
  m=min(reps,(rem+sb*k-1)//(sb*k));length=min(sb*k*m,rem)
  yield dict(k=k,reps=reps,m=m,length=length,delta=s,word=word[:],path=path[:],domain=[lo,hi])
  if sb*k>=rem:break

counts={'columns':0,'packets':0,'lce_certificates':0,'lf_offset_certificates':0};equal_delta=None;cap=None;lf_nonadj=None
for n in range(1,8):
 for ll in itertools.product(range(3),repeat=n):
  l=list(ll);counts['columns']+=1;r=1+sum(a!=b for a,b in zip(l,l[1:]))
  psi=sorted(range(n),key=lambda q:(l[q],q));lf=[0]*n
  for q,p in enumerate(psi):lf[p]=q
  for sb in [1,2]:
   _,sig,cells,ids=model(l,sb);assert len(cells)<=(2*r-1 if sb==1 else 3*r-2)
   for q in range(n):
    x=[q]
    for _ in range(2*n+16):x.append(psi[x[-1]])
    out=[l[v] for v in x]
    if equal_delta is None and psi[q]-q==psi[psi[q]]-psi[q] and l[q]!=l[psi[q]]:
     equal_delta={'L':l,'q':q,'states':x[:3],'labels':out[:3],'delta':psi[q]-q}
    for rem in sorted(set([1,n,max(1,n//2)])):
     for p in paths(cells,ids,q,rem,sb,4):
      per=p['k']*sb
      assert out[:p['length']]==(p['word']*((p['length']+per-1)//per))[:p['length']]
      if p['length']==p['m']*per:assert x[p['length']]==q+p['m']*p['delta']
      # Independent repeated cell-label walk, not interval arithmetic.
      whole=0
      while whole*per<rem:
       if any(ids[x[whole*per+j*sb]]!=cid for j,cid in enumerate(p['path'])):break
       whole+=1
      assert p['length']==min(whole*per,rem)
      counts['packets']+=1
      if cap is None and p['k']>=2 and p['reps']>=2 and p['length']<=per and len(set(p['word']))>1:
       cap={'L':l,'q':q,'remaining':rem,'step_bytes':sb,'packet':p,'output':out[:rem]}
    if sb==1:
     for period in range(1,min(4,n)+1):
      q2=x[period];y=q2;K=0
      while K<n and out[K]==l[y]:K+=1;y=psi[y]
      length=min(n,period+K)
      assert all(out[i]==out[i-period] for i in range(period,length))
      counts['lce_certificates']+=1
  # LF-domain s-run certificates: identical inverse-path L labels imply the same
  # offsets only within a common contiguous itinerary interval. Test adjacent rows.
  for a in range(n-1):
   x,y=a,a+1
   for s in range(1,5):
    if l[x]!=l[y] or y!=x+1:break
    x,y=lf[x],lf[y];assert y-x==1;counts['lf_offset_certificates']+=1

# Explicit cap witness in a periodic column: pair k=3, six-byte word, four bytes left.
l=[2]*60+[0]*60+[1]*60;psi,sig,cells,ids=model(l,2)
periodic=list(paths(cells,ids,0,4,2,8))
# A cleaner two-step single-word witness at a cap also suffices (printed above).
out={'counts':counts,'equal_displacement_is_not_equal_output':equal_delta,'uncapped_repeat_but_no_two_copies_before_marker':cap}
Path(__file__).with_suffix('.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
