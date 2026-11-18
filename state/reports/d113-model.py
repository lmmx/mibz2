"""Guardian model: explicit two-order induction; byte-addressed destructive writer.
Not Rust execution or Miri. No timing. Candidate source cd7e5de2.
"""
import itertools, random, json
from collections import Counter
M=1<<31; RO=1<<24
C=Counter()
def induction(col, primary):
    arena=bytearray(range(256)); words=[]; widths=[]
    for i,c in enumerate(col):
        if i in (primary,primary+1) or not words or c != col[i-1]:
            words.append((c<<32)|1|(M if i==primary else 0)); widths.append(0)
        widths[-1]+=1
    top=list(range(len(words))); bot=sorted(top,key=lambda i: words[i]>>32)
    records=[]
    while True:
        a,b=top[-1],bot[-1]
        if a==b:
            if words[a]&M: root=words[a];break
            top.pop();bot.pop();continue
        x=len(words); width=min(widths[a],widths[b]); wa,wb=words[a],words[b]
        length=(wa&0xffffffff)+(wb&0xffffffff)
        if length<=16:
            r=len(arena); arena+=arena[wa>>32:(wa>>32)+(wa&~M&0xffffffff)]+arena[wb>>32:(wb>>32)+(wb&~M&0xffffffff)]
        else:
            r=RO+16*len(records);records.append((wa,wb))
        words.append((r<<32)|length);widths.append(width)
        top.insert(top.index(b)+1,x);bot.insert(bot.index(a)+1,x)
        for i in (a,b):
            widths[i]-=width
            if widths[i]==0:top.remove(i);bot.remove(i)
    inv=sorted(range(len(col)),key=col.__getitem__); row=primary; cycle=[]
    while True:
        row=inv[row];cycle.append(col[row])
        if row==primary:break
    return arena,records,root,bytes(cycle)
def writer(col, primary, mutant=None):
    arena,original,root,want=induction(col,primary);p=root&0x7fffffff;nr=len(original)
    assert p==len(want)
    if p!=len(col) or nr==0 or root>>32!=RO+16*(nr-1):
        C['fallback']+=1;return
    C['owned']+=1; records=original.copy(); pos=[0xffffffff]*nr
    last={}
    for i,(a,b) in enumerate(original):
        for w in (a,b):
            if w>>32>=RO:last[w]=i
    w=root;j=0
    while w>>32>=RO:
        a,b=original[((w>>32)-RO)//16]
        if a&M:w=a
        else:j+=a&0xffffffff;w=b
    pos[-1]=(-(j+1))%p;tail=nr;na=nb=(1<<64)-1;out=bytearray(b^255 for b in want);filled=bytearray(p)
    def put(d,data):
        assert 0<=d and d+len(data)<=p,'destination bounds'
        assert not any(filled[d:d+len(data)]),'double write'
        out[d:d+len(data)]=data;filled[d:d+len(data)]=b'\1'*len(data)
    for j in reversed(range(nr)):
        assert j<tail,'unread rule overwritten'
        a,b=records[j];assert (a,b)==original[j]
        q=pos[j];assert q<p,'owner unassigned'
        right=q+(a&0x7fffffff)
        if right>=p:right-=p;C['wraps']+=1
        if mutant=='reread' and j+1<nr:na,nb=records[j+1]
        count=0
        for w,later,d in ((a,na,q),(b,nb,right)):
            r=w>>32;length=w&0x7fffffff
            if r<RO:
                assert length<=16;put(d,arena[r:r+length])
            elif w==later:
                assert last[w]>j,'false survival';assert not w&M
                assert tail>j;tail-=1;records[tail]=(w,d);count+=1
                if tail==j:C['overwrite_current']+=1
                if tail==j+1:C['overwrite_next']+=1
            else:
                assert last[w]==j,'false owner'
                k=(r-RO)//16;assert k<j and pos[k]==0xffffffff
                pos[k]=d
        assert count<=1;na,nb=a,b
    for i in (reversed(range(tail,nr)) if mutant=='reverse' else range(tail,nr)):
        w,d=records[i];f=pos[((w>>32)-RO)//16];l=w&0xffffffff
        assert f<=p and l<=p-f and d<=p and l<=p-d
        assert all(filled[f:f+l]),'unfinished source'
        assert f+l<=d or d+l<=f,'source overlaps destination'
        C['copies']+=1;C['later_address']+=f>d
        put(d,out[f:f+l])
    assert all(filled) and out==want,'output differs'
def main():
    total=0
    for alphabet,bound in ((2,10),(3,7)):
        for n in range(1,bound+1):
            for col in itertools.product(range(alphabet),repeat=n):
                for p in range(n):writer(col,p);total+=1
    for n in (35,36):
        col=[1,1]+[0]*(n-2)
        for p in range(n):writer(col,p);total+=1
    rng=random.Random(113)
    for _ in range(1500):
        n=rng.randrange(17,180);col=[rng.randrange(rng.choice([2,3,5])) for _ in range(n)]
        writer(col,rng.randrange(n));total+=1
    witnesses={};mutation_cases=[]
    # Text-derived BWTs induce nested copies; test each with independent LF output.
    for name in ('cells.rs','cells_owned.rs'):
        text=open(name,'rb').read()[:12000];n=len(text)
        order=sorted(range(n),key=lambda i:text[i:]+text[:i])
        col=[text[(i-1)%n] for i in order];p=order.index(0)
        writer(col,p);total+=1
        mutation_cases.append((name,n,col,p))
    baseline=dict(C)
    for name,n,col,p in mutation_cases:
        for mutant in ('reread','reverse'):
            try:writer(col,p,mutant)
            except AssertionError as e:witnesses[mutant]={'file':name,'prefix_bytes':n,'failure':str(e)}
    print(json.dumps({'cases':total,'baseline_counters':baseline,'mutants':witnesses},indent=2))
if __name__=='__main__':main()
