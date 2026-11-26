"""Affine certificate over all integer t >= 20 for the six-type gap-coalesced rotation."""
from fractions import Fraction
from itertools import groupby
import csv
import sys
from witness import test

T=20
# Each coordinate is a*t+b. Comparisons hold on the entire ray.
def plus(x,y): return x[0]+y[0],x[1]+y[1]
def scale(c,x):return c*x[0],c*x[1]
def minus(x,y):return plus(x,scale(-1,y))
def at(x,t=T): return x[0]*t+x[1]
def nonnegative(x):return x[0]>=0 and at(x)>=0
def positive(x):return x[0]>=0 and at(x)>0
def le(x,y):return nonnegative(minus(y,x))
def lt(x,y):return positive(minus(y,x))

F=[(0,0)]
for width in [(1,2),(1,0),(1,0),(0,1),(1,0),(1,0)]:
    F.append(plus(F[-1],width))
n=F[-1]
step=(1,2)

def wrap(x):
    mult=at(x)//at(n)
    z=minus(x,scale(mult,n))
    assert nonnegative(z) and lt(z,n)
    return z

points={wrap(minus(bound,scale(shift,step))) for bound in F[:-1] for shift in (0,1,2)}
p=sorted(points,key=at)+[n]
atoms=[]
for l,h in zip(p,p[1:]):
    width=minus(h,l)
    assert positive(width)
    labels=[]
    for steps in (0,1,2):
        z=wrap(plus(l,scale(steps,step)))
        j=next(i for i in range(6) if at(F[i])<=at(z)<at(F[i+1]))
        assert le(F[j],z) and le(plus(z,width),F[j+1])
        labels.append(j)
    atoms.append((width,*labels))
assert all(a!=b and b!=c for _,a,b,c in atoms)
neighbors=[sorted({a for _,a,b,_ in atoms if b==k}) for k in range(6)]
records=[]
for mask in range(64):
    inside=lambda k:bool(mask>>k&1)
    group={}
    for i,targets in enumerate(neighbors):
        previous_selected=True
        for j in targets:
            if inside(j) or previous_selected:part=j
            group[i,j]=part
            previous_selected=inside(j)
    ingress=(0,0);labels=[]
    for width,a,b,c in atoms:
        if inside(a) and not inside(b):
            ingress=plus(ingress,width)
            continue
        pred,following=(c,b) if inside(b) and not inside(c) else (b,a)
        labels.append((pred,-1 if inside(pred) else group[pred,following]))
    runs=sum(1 for _ in groupby(labels))
    if runs<7:
        assert positive(minus(n,scale(4,ingress)))
    records.append((mask,*ingress,runs))
# Compare the affine census against independent pointwise first returns.
for t in (T,22,30,100,101):
    s='1'*t+'2'*t+'3'+'4'*t+'5'*t+'0'*(t+2)
    _,actual=test(s,True)
    assert actual==[(i,a*t+b,r) for i,a,b,r in records]
w=csv.writer(sys.stdout,lineterminator='\n')
w.writerow(('mask','q_slope','q_intercept','new_runs'))
w.writerows(records)
