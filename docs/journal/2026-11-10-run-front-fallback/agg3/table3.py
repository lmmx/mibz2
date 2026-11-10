import re,sys,os
D=sys.argv[1]
builds=[('sc-master','master 27ed84df'),('sc-head','#324 head e88c0631'),('sc-boxed','boxed f72b2a3c'),('sc-cand','candidate e8a417c6')]
runs=[('rand-0','random, pad 0'),('rand-336','random, pad 336'),('rec-0','records, pad 0')]
def parse(f):
    out={}; fn=None
    for line in open(f):
        m=re.match(r'(\S+) loads',line)
        if m: fn=m.group(1); out[fn]={}; continue
        m=re.match(r'\s+mod (4K|1K) \(load<-store\):(.*)',line)
        if m and fn:
            for k,v in re.findall(r'(\w<-\w)=(\d+)',m.group(2)): out[fn][(m.group(1),k)]=int(v)
    return out
data={}
for b,_ in builds:
    for r,_ in runs:
        f=f'{D}/{b}-{r}.txt'
        if os.path.exists(f) and os.path.getsize(f): data[(b,r)]=parse(f)
rows=[('mtf.mtf_avx2','4K','D<-H','`STEPS` load ← heap store (the named pair\'s class)'),
      ('mtf.mtf_avx2','1K','D<-H',''),
      ('mtf.mtf_avx2','4K','H<-H',''),('mtf.mtf_avx2','1K','H<-H',''),
      ('mtf.mtf_avx2','4K','S<-H',''),('mtf.mtf_avx2','4K','H<-S',''),('mtf.mtf_avx2','4K','D<-S',''),
      ('block.read_item','1K','S<-S','stack ← stack'),('block.read_item','4K','S<-S',''),('block.read_item','4K','H<-H',''),('block.read_item','1K','H<-H',''),
      ('bwt.build','4K','H<-H',''),('bwt.build','1K','S<-S',''),('bwt.build','1K','H<-H',''),
      ('bwt.room_segments','4K','H<-H',''),('bwt.room_segments','4K','S<-S',''),('bwt.room_segments','1K','S<-S','')]
for r,rn in runs:
    print(f"\n**{rn}**\n")
    print("| function | mod | load ← store | "+" | ".join(n for _,n in builds)+" |")
    print("|---|---|---|"+"--:|"*len(builds))
    for fn,m,k,_ in rows:
        cells=[]
        base=data.get(('sc-master',r),{}).get(fn,{}).get((m,k))
        for b,_ in builds:
            v=data.get((b,r),{}).get(fn,{}).get((m,k))
            if v is None: cells.append('–' if (b,r) in data else 'n/a'); continue
            rel='' if b=='sc-master' or not base else f" ({v/base:.2f}×)"
            cells.append(f"{v:,}{rel}")
        print(f"| `{fn.split('.')[1]}` | {m} | {k} | "+" | ".join(cells)+" |")
