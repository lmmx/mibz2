"""Small true modal section where sorting predecessor branches changes the child column."""
import itertools
def example(last,k):
 n=len(last);f=sorted(last);psi=sorted(range(n),key=lambda x:last[x])
 def prefix(x,k):
  s=[]
  for _ in range(k):s.append(f[x]);x=psi[x]
  return tuple(s)
 contexts=[prefix(x,k)for x in range(n)]
 sources={}
 for x,w in enumerate(contexts):sources.setdefault(w,[]).append(x)
 modes={w:min(range(3),key=lambda a:(-sum(last[x]==a for x in r),a))for w,r in sources.items()}
 lf=[None]*n
 for x,y in enumerate(psi):lf[y]=x
 section=sorted(lf[x] for x,w in enumerate(contexts)if last[x]!=modes[w])
 if not section:return
 rank={x:j for j,x in enumerate(section)};incoming=[None]*len(section)
 words=[];starts={};target=[]
 for j,x in enumerate(section):
  w=[];y=x
  while True:
   w.append(f[y]);y=psi[y]
   if y in rank:break
  key=(tuple(w),prefix(y,k+1));starts.setdefault(key,j);words.append(tuple(w));target.append(rank[y])
  incoming[rank[y]]=starts[key]
 naive=incoming[:]
 for _,positions in itertools.groupby(range(len(section)),key=lambda j:prefix(section[j],k+1)):
  pos=list(positions);ordered=sorted((incoming[j] for j in pos),key=lambda code:words[code][::-1])
  for j,label in zip(pos,ordered):naive[j]=label
 if naive==incoming:return
 print('k',k,'original L',''.join(map(str,last)),'shifted physical section',section)
 print('source spellings',[''.join(map(str,w)) for w in words],'actual targets',target)
 print('actual incoming class bases',incoming,'branch-sorted class bases',naive)
 return True
for n in range(1,11):
 for last in itertools.product(range(3),repeat=n):
  for k in [1,2]:
   if example(last,k):raise SystemExit
