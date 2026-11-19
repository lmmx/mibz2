"""Find a non-LF colored exchange defeating byte semantics of the strip certificate."""
import itertools
for n in range(3,9):
 for perm in itertools.permutations(range(n)):
  for labels in itertools.product(range(2),repeat=n):
   if tuple(sorted(range(n),key=lambda z:(labels[z],z)))==tuple(perm.index(x) for x in range(n)):continue
   for q in range(2,min(4,n)+1):
    for x in range(n-q):
     def route(y):
      w=[];v=y
      while True:
       w.append(labels[v]);v=perm[v]
       if v%q==y%q:return tuple(w),v
     w,a=route(x);v,b=route(x+q)
     if w!=v or b!=a+q:continue
     y=x
     for i in range(len(w)):
      good=all(labels[z]==labels[y] and perm[z]==perm[y]+z-y for z in range(y,y+q+1))
      if not good:
       print('arbitrary permutation',perm,'colors',labels,'q',q,'start',x,
             'equal return word',w,'targets',a,b,'first failed translation',i)
       raise SystemExit
      y=perm[y]
