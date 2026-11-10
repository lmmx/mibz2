"""Independent arithmetic model, not execution of Rust or a concurrency checker."""
import itertools, random, json, hashlib, pathlib
P=12
def records(col):
    out=[]
    for c,g in itertools.groupby(col):
        n=len(list(g))
        while n:
            k=min(n,127);out.append((c,k));n-=k
    return out
def cuts(rs,counts):
    n=sum(counts);nxt=list(itertools.accumulate([0]+counts))[:-1]
    cs=[(0,0,nxt.copy())];row=work=0;total=n+8*len(rs)
    for i,(c,k) in enumerate(rs):
        while len(cs)<P and work>=(total*len(cs)+P-1)//P:
            cs.append((i,row,nxt.copy()))
        row+=k;work+=k+8;nxt[c]+=k
    cs += [(len(rs),row,nxt.copy())]*(P+1-len(cs))
    return cs
def fill(rs,a,b,table,vector):
    ri,row,nxt=a;endri,end,lim=b;nxt=nxt.copy()
    if ri>endri or endri>len(rs) or row>end or any(x>y for x,y in zip(nxt,lim)):return False
    for c,k in rs[ri:endri]:
        if not k or k>end-row or k>lim[c]-nxt[c]:return False
        e=c|(nxt[c]<<8);stop=row+k
        if vector and end-stop>=7:
            for w in range(row,stop,8):
                assert w+8<=end
                for j in range(8):table[w+j]=(e+((w-row+j)<<8))&0xffffffff
        else:
            for j in range(k):table[row+j]=(e+(j<<8))&0xffffffff
        row=stop;nxt[c]+=k
    return row==end and nxt==lim
def check(col):
    rs=records(col);counts=[col.count(c) for c in range(4)];cs=cuts(rs,counts);n=len(col)
    starts=list(itertools.accumulate([0]+counts))[:-1];seen=[0]*4;want=[]
    for c in col:want.append(c|((starts[c]+seen[c])<<8));seen[c]+=1
    for shares in [1,2,3,4,5,6,12,13,64]:
        for vector in [False,True]:
            t=[None]*(n+8)
            for j in reversed(range(shares)):
                a,b=cs[P*j//shares],cs[P*(j+1)//shares]
                before=t.copy();assert fill(rs,a,b,t,vector)
                assert t[:a[1]]==before[:a[1]] and t[b[1]:]==before[b[1]:]
            assert t[:n]==want and t[n:]==[None]*8
    return 1
count=0
for n in range(7):
    for col in itertools.product(range(3),repeat=n):count+=check(list(col))
r=random.Random(97097)
for _ in range(120):
    col=[]
    for j in range(r.randrange(1,40)):col += [r.randrange(4)]*r.randrange(1,270)
    count+=check(col)
# Exhaust all byte-sized record lengths and distances near the vector tail boundary.
tail=0
for length in range(1,256):
    for after in range(17):
        for vector in [False,True]:
            t=[None]*(length+after+8);rs=[(0,length)]+([(1,after)] if after else [])
            a=(0,0,[0,length]);b=(len(rs),length+after,[length,length+after])
            assert fill(rs,a,b,t,vector)
            assert t[:length+after]==[(i<<8) for i in range(length)]+[1|(i<<8) for i in range(length,length+after)]
            assert t[length+after:]==[None]*8;tail+=1
assert (2**24-1)*255+(2**24-1)<2**32
alignment={a:sum(o+320>4096 for o in range(0,4096,a)) for a in (64,512)}
assert alignment=={64:4,512:0}
result={'model_columns':count,'share_counts':[1,2,3,4,5,6,12,13,64],'scalar_and_vector':True,'tail_cases':tail,'alignment_straddles':alignment,'limits':'Python arithmetic model; no Rust, Miri, scheduling or timing replay'}
print(json.dumps(result,indent=2))
pathlib.Path(__file__).with_name('model-results.json').write_text(json.dumps(result,indent=2)+'\n')
