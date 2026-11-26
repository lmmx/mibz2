"""Independent partial-map audit of each adjacent gap through full interval-tip clipping."""
import itertools
def gaps(width,top,bottom):
 dest={};n=0
 for i in bottom:dest[i]=n;n+=width[i]
 g=[None]*max(0,n-1);x=0
 for i in top:
  for k in range(width[i]-1):g[x+k]=dest[i]+k
  x+=width[i]
 return g,n
def check(old,bottom,remove):
 size=list(old);top=list(range(len(size)));bottom=list(bottom)
 g,n=gaps(size,top,bottom)
 a,b=top[-1],bottom[-1];wa,wb=size[a],size[b];w=remove
 if a==b:
  if wa==w:top.pop();bottom.pop()
  else:size[a]-=w
 elif w<wa and w<wb:
  c=len(size);size.append(w);size[a]-=w;size[b]-=w
  top.insert(top.index(b)+1,c);bottom.insert(bottom.index(a)+1,c)
 elif wa==wb:
  top.remove(a);bottom.remove(b);bottom[bottom.index(a)]=b
 elif wa>wb:
  size[a]-=w;bottom.remove(b);bottom.insert(bottom.index(a)+1,b)
 else:
  size[b]-=w;top.remove(a);top.insert(top.index(b)+1,a)
 got,h=gaps(size,top,bottom);assert h==n-w
 expected=[]
 for x in range(max(0,h-1)):
  y=g[x]
  while y is not None and y>=h-1:y=g[y]
  expected.append(y)
 return got,expected
full=0;partial=0;failed=0;witness=None
for k in range(1,6):
 for widths in itertools.product(range(1,4),repeat=k):
  for b in itertools.permutations(range(k)):
   w=min(widths[-1],widths[b[-1]])
   got,want=check(widths,b,w);assert got==want;full+=1
   for take in range(1,w):
    got,want=check(widths,b,take);partial+=1
    if got!=want:
     failed+=1
     if witness is None or sum(widths)<witness[0]:witness=(sum(widths),widths,b,take,got,want)
print('full tip surgeries',full,'partial controls',partial,'partial differences',failed)
print('minimal observed arbitrary partial difference',witness)
