import subprocess,glob,re,sys
def prof(f):
    out=subprocess.run(['callgrind_annotate','--inclusive=yes',f],capture_output=True,text=True).stdout
    d={}
    for line in out.splitlines():
        m=re.match(r'\s*([\d,]+) \(.*?\)\s+([^:\s]+):(.*?)( \[.*)?$',line)
        if not m: continue
        v=int(m.group(1).replace(',','')); fn=m.group(3)
        d.setdefault(fn,v)
        d[fn]=max(d[fn],v)
    tot=max(v for k,v in d.items() if 'measured' in k)
    g=lambda pat: max([v for k,v in d.items() if re.search(pat,k)] or [0])
    front=g(r'mtf::mtf_avx2$')+g(r'runs::front_avx2$')
    table=g(r'runs::fill_avx2$')
    walk=g(r'room_segments')
    fin=g(r'rle::unrle_avx2$')+g(r'crc::clmul_update$')
    return tot,front,table,walk,fin
blocks=sys.argv[1:]
print("| block | output bytes | base total | head total | change | front: MTF → emitter | fill (head) | walk: base → head | build + join (base) → join (head) |")
print("|---|--:|--:|--:|--:|--:|--:|--:|--:|")
for b in blocks:
    B=prof(f'cg/base-{b}.out'); H=prof(f'cg/head-{b}.out')
    so=open(f'cg/head-{b}.stdout').read(); sb=open(f'cg/base-{b}.stdout').read()
    assert so.split(', syms')[0]==sb.split(', syms')[0], (so,sb)
    nb=int(re.search(r': (\d+) bytes',so).group(1)); takes='takes true' in so
    rb=B[0]-B[1]-B[3]-B[4]; rh=H[0]-H[1]-H[2]-H[3]-H[4]
    f=lambda x:f"{x:,}"
    print(f"| {b}{'' if takes else ' (column)'} | {nb:,} | {f(B[0])} | {f(H[0])} | {100*(H[0]/B[0]-1):+.2f}% | {f(B[1])} → {f(H[1])} | {f(H[2])} | {f(B[3])} → {f(H[3])} | {f(rb)} → {f(rh)} |")
