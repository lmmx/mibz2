"""Construct the incomplete first-level destination table without reading a singleton tail."""
def construct(last,q,m,lf,threshold=1):
    if not m:return {},[]
    nodes=[(q,q+m,b"")]
    children=[]
    for a,b,suffix in nodes:
        by_byte={}
        if b-a>threshold:
            for x in range(a,b):
                c=last[x]
                if c==34:continue
                if c not in by_byte: by_byte[c]=[lf[x],lf[x]]
                assert by_byte[c][1]==lf[x]
                by_byte[c][1]+=1
            for c,(lo,hi) in list(by_byte.items()):
                by_byte[c]=len(nodes)
                nodes.append((lo,hi,suffix+bytes([c])))
        children.append(by_byte)
    outputs=[None]*len(nodes)
    for i in reversed(range(len(nodes))):
        a,b,suffix=nodes[i]
        if b-a<=threshold:
            outputs[i]=[(False,x,suffix) for x in range(a,b)];continue
        seen={};out=[]
        terminal=b'"'+suffix[::-1]
        for x in range(a,b):
            c=last[x]
            if c==34:
                out.append((True,lf[x]-q,terminal))
            else:
                j=seen.get(c,0);out.append(outputs[children[i][c]][j]);seen[c]=j+1
        outputs[i]=out
        for child in children[i].values():outputs[child]=None
    hot={};cut=[None]*m
    for t,(known,value,content) in enumerate(outputs[0]):
        if known: hot[t]=(value,content)
        else: cut[t]=(value,content)
    return hot,cut
