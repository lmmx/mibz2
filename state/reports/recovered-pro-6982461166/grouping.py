from itertools import product


def mapping(word):
    n=len(word)
    sa=sorted(range(n),key=lambda i:word[i:]+word[:i])
    if len(set(word[i:]+word[:i] for i in sa))!=n:
        return None
    phase=[(i-1)%n for i in sa]
    col=[word[i] for i in phase]
    # Text-coordinate source relation Phi, broken at run heads.
    parent={phase[r]:phase[r-1] for r in range(1,n) if col[r-1]==col[r]}
    spans=[]
    for i,p in sorted(parent.items()):
        if not spans or spans[-1][1]!=i or spans[-1][2]!=p-i:
            spans.append([i,i+1,p-i])
        else:
            spans[-1][1]+=1
    owner={x:k for k,(a,b,_) in enumerate(spans) for x in range(a,b)}
    edges={k:{owner[x+d] for x in range(a,b) if x+d in owner and owner[x+d]!=k}
           for k,(a,b,d) in enumerate(spans)}
    return "".join(col),sa,parent,spans,edges


for n in range(3,12):
    for w in product("ab",repeat=n):
        w="".join(w)
        m=mapping(w)
        if m is None:
            continue
        col,sa,parent,spans,edges=m
        for a, links in edges.items():
            for b in links:
                if a in edges[b] and spans[a][1]-spans[a][0]>=2 and spans[b][1]-spans[b][0]>=2:
                    print("whole-span cycle",w,col,sa,parent,spans,a,b)
                    raise SystemExit
