import json,re,sys,numpy as np
D=sys.argv[1]; man=json.load(open(D+'/manifest.json'))
CL=['guid','sha1','entity-id','number','json-key','text','structure']
pat=[('guid',rb'[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}'),
     ('sha1',rb'(?<![0-9a-f])[0-9a-f]{40}(?![0-9a-f])'),
     ('entity-id',rb'(?<![A-Za-z0-9])[QPL][0-9]+(?:-[SF][0-9]+)?'),
     ('number',rb'[+-]?[0-9][0-9.:TZ+-]*'),
     ('json-key',rb'"[A-Za-z_-]+"(?=:)')]
print('block | share of thin-run rows (width<16) by output token class; then width-1 share; then share of all bytes')
for b in man['blocks']:
    L=np.fromfile(D+'/'+b['file'],dtype=np.uint8); n=len(L); p=b['primary_index']
    cnt=np.bincount(L,minlength=256); C=np.concatenate([[0],np.cumsum(cnt)[:-1]])
    # stable rank
    order=np.argsort(L,kind='stable'); LF=np.empty(n,np.int64); LF[order]=np.arange(n)
    # run widths
    brk=np.flatnonzero(np.diff(L))+1; starts=np.concatenate([[0],brk]); ends=np.concatenate([brk,[n]])
    w=np.repeat(ends-starts,ends-starts)
    # walk: step k visits row r, byte at text pos n-1-k
    rows=np.empty(n,np.int64); r=p
    LFl=LF.tolist(); 
    for k in range(n): rows[k]=r; r=LFl[r]
    pos=np.empty(n,np.int64); pos[rows]=n-1-np.arange(n)
    T=L[rows][::-1].tobytes()
    cls=np.full(n,6,np.int8)
    # text: letters inside strings that aren't keys -> approximate: alpha bytes not otherwise classed
    a=np.frombuffer(T,np.uint8); alpha=((a|32)>=97)&((a|32)<=122)|(a>=128); cls[alpha]=5
    for ci,(name,rx) in enumerate(reversed(pat)):
        idx=CL.index(name)
        for m in re.finditer(rx,T): cls[m.start():m.end()]=idx
    c_row=cls[pos]
    thin=w<16; one=w==1
    def sh(mask): h=np.bincount(c_row[mask],minlength=7); return h/h.sum()
    f=lambda v:' '.join(f'{x*100:4.1f}' for x in v)
    print(f"{b['file']:12} thin {thin.mean()*100:4.1f}% of rows")
    print('   classes     ',' '.join(f'{c[:4]:>4}' for c in CL))
    print('   thin rows   ',f(sh(thin)))
    print('   width-1 rows',f(sh(one)))
    print('   all bytes   ',f(sh(np.ones(n,bool))))
