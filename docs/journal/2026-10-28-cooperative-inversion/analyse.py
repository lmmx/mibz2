"""The cooperative grid's rounds, paired against p4 (journal §7, §8.1).

    python3 analyse.py coop-<time>.jsonl

For each arm: the geometric mean of its ratio to p4 in the same round, the t-interval, a
block bootstrap (blocks of 4 rounds), the lag-1 autocorrelation and an AR(1)-widened
interval, the halves, the trend per 10 rounds, the position effect, and the §5 rule
(mean >= 1.10, lower bound > 1.0, every round above p4).
"""
import json,collections,statistics as st,math,random,sys
R=[json.loads(l) for l in open(sys.argv[1])]
by=collections.defaultdict(dict)
arms=[]
for r in R:
    by[r['arm']][r['round']]=r
    if r['arm'] not in arms: arms.append(r['arm'])
rounds=sorted(by['p4'])
na=len(arms)
def order(r):
    o=list(range(na)); o=o[r%na:]+o[:r%na]
    if (r//na)%2==1: o.reverse()
    return [arms[i] for i in o]
pos={(a,r):order(r).index(a) for r in rounds for a in arms}
def lr(a,b='p4',key='mb_s'): return [math.log(by[a][k][key]/by[b][k][key]) for k in rounds]
def tq(df):
    # two-sided 97.5% t quantile, small table
    T={5:2.571,8:2.306,9:2.262,10:2.228,12:2.179,15:2.131,18:2.101,19:2.093,20:2.086,24:2.064,30:2.042,37:2.026,40:2.021}
    return T.get(df, 2.0 if df>40 else T[min(T,key=lambda d:abs(d-df))])
random.seed(1)
def boot_block(x,L=4,B=4000):
    n=len(x); ms=[]
    for _ in range(B):
        s=[]
        while len(s)<n:
            i=random.randrange(n-L+1); s+=x[i:i+L]
        ms.append(st.mean(s[:n]))
    ms.sort(); return ms[int(.025*B)],ms[int(.975*B)]
def ac1(x):
    m=st.mean(x); d=[v-m for v in x]
    return sum(d[i]*d[i+1] for i in range(len(d)-1))/sum(v*v for v in d)
def slope(x):
    n=len(x); t=list(range(n)); mt=st.mean(t); mx=st.mean(x)
    b=sum((ti-mt)*(xi-mx) for ti,xi in zip(t,x))/sum((ti-mt)**2 for ti in t)
    res=[xi-mx-b*(ti-mt) for ti,xi in zip(t,x)]
    se=math.sqrt(sum(e*e for e in res)/(n-2)/sum((ti-mt)**2 for ti in t))
    return b,se
print(f"{len(R)} records, {na} arms, {len(rounds)} rounds; misplaced {sum(not r['placement_ok'] for r in R)}")
print("arm        geo   t95-iv          block-boot(L=4)   ac1    halves(1st,2nd)   trend/10rnds  (t)  pos-effect(log/slot)")
for a in arms:
    if a=='p4': continue
    x=lr(a); n=len(x); m=st.mean(x); se=st.stdev(x)/math.sqrt(n); t=tq(n-1)
    lo,hi=boot_block(x)
    h1=math.exp(st.mean(x[:n//2])); h2=math.exp(st.mean(x[n//2:]))
    b,sb=slope(x)
    # position effect: regress ratio on (pos[a]-pos[p4])
    dp=[pos[(a,k)]-pos[('p4',k)] for k in rounds]; md=st.mean(dp)
    pb=sum((d-md)*(v-m) for d,v in zip(dp,x))/max(1e-12,sum((d-md)**2 for d in dp))
    # AR(1)-inflated interval
    rho=ac1(x); infl=math.sqrt(max(1,(1+rho)/(1-rho)))
    rule=math.exp(m)>=1.10 and math.exp(m-t*se)>1.0 and min(x)>0
    print(f"{a:9} wins {sum(v>0 for v in x)}/{n} worst {math.exp(min(x)):.3f} rule {'PASS' if rule else 'fail'}")
    print(f"{a:9} {math.exp(m):.3f} [{math.exp(m-t*se):.3f},{math.exp(m+t*se):.3f}] [{math.exp(lo):.3f},{math.exp(hi):.3f}] {rho:+.2f} AR1-iv [{math.exp(m-t*se*infl):.3f},{math.exp(m+t*se*infl):.3f}] {h1:.3f},{h2:.3f} {math.exp(10*b)-1:+.3%} ({b/sb:+.1f}) {pb:+.4f}")
print()
print("absolute MB/s by half (median):")
for a in arms:
    v=[by[a][k]['mb_s'] for k in rounds]; n=len(v)
    print(f"  {a:9} {st.median(v[:n//2]):7.0f} {st.median(v[n//2:]):7.0f}  ({st.median(v[n//2:])/st.median(v[:n//2])-1:+.1%})")
