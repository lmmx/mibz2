import re,collections,sys
names={}
def nm(kind,s):
    m=re.match(r'\((\d+)\)(?: (.*))?',s)
    if m:
        k=(kind,m.group(1))
        if m.group(2) is not None: names[k]=m.group(2)
        return names[k]
    return s
self_c=collections.defaultdict(lambda:[0]*9); incl_calls=collections.defaultdict(lambda:[0]*9)
fn=None; pending=False
for line in open(sys.argv[1]):
    line=line.rstrip('\n')
    if line.startswith('events:'): events=line.split()[1:]; continue
    for pre,kind in (('fn=','fn'),('cfn=','fn'),('fl=','fl'),('fi=','fl'),('fe=','fl'),('cfl=','fl'),('cfi=','fl'),('ob=','ob'),('cob=','ob')):
        if line.startswith(pre):
            v=nm(kind,line[len(pre):])
            if pre=='fn=': fn=v
            break
    else:
        if line.startswith('calls='): pending=True; continue
        if line and (line[0].isdigit() or line[0] in '+-*'):
            parts=line.split()
            tgt=incl_calls if pending else self_c
            for i,x in enumerate(parts[1:]): tgt[fn][i]+=int(x)
            pending=False
print(events)
for f in sorted(self_c):
    if re.search(r'Ctor2s?>::build::<false>|ctor2s?::W<false>>::(finish|refill)',f):
        inc=[a+b for a,b in zip(self_c[f],incl_calls[f])]
        print(f'{f}\n   self Ir {self_c[f][0]:,} Dr {self_c[f][1]:,} Dw {self_c[f][2]:,}\n   inclusive Ir {inc[0]:,} Dr {inc[1]:,} Dw {inc[2]:,}')
