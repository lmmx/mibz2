"""Compare the C ABI result to the previously byte-exact independent canonical hashes."""
import argparse,csv,ctypes as ct,hashlib,itertools,json,struct,sys,tempfile
from collections import Counter
from itertools import accumulate,groupby
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[3]))
from clock import Clock

def interface(path):
    lib=ct.CDLL(str(path));ptr=ct.c_void_p;u=ct.c_uint32
    for n,args,ret in [('range_state',[],ptr),('range_state_free',[ptr],None),
       ('range_input',[ct.c_char_p,ct.c_char_p],ptr),('range_input_free',[ptr],None),
       ('range_construct',[ptr,ptr,ct.POINTER(u)],ptr)]:
        op=getattr(lib,n);op.argtypes=args;op.restype=ret
    return lib

def bytes_for(lib,state,raw,ctx):
    inp=lib.range_input(str(raw).encode(),str(ctx).encode());n=ct.c_uint32()
    try:p=lib.range_construct(state,inp,ct.byref(n));return ct.string_at(p,12*n.value)
    finally:lib.range_input_free(inp)

def real(lib,state,original,models,src,receipts):
    mapping={int(r['index']):r['tape'].split('.')[0] for r in csv.DictReader(src.open())}
    cells=records=0
    for r in csv.DictReader(receipts.open()):
        i=int(r['block']);k=int(r['k'])
        blob=bytes_for(lib,state,original/mapping[i]/'input.txt',models/str(i)/f'{k}.ctx')
        assert hashlib.sha256(blob).hexdigest()==r['independent_band_sha256']
        assert len(blob)==12*int(r['canonical_return_pieces']);cells+=1;records+=len(blob)//12
    return dict(cells=cells,canonical_triples=records)

def tiny(lib,state):
    total=Counter()
    with tempfile.TemporaryDirectory()as d:
        raw,ctx=Path(d)/'input',Path(d)/'ctx'
        for a,nmax in [(2,7),(3,4)]:
            for n in range(1,nmax+1):
                for L in itertools.product(range(a),repeat=n):
                    runs=[(v,sum(1 for _ in g))for v,g in groupby(L)]
                    for k in sorted({0,1,2,3,n+1}):
                        c=Clock(L,k);f=Counter(c.state);end=list(accumulate(f[x]for x in range(len(f))))
                        defs=b''.join(struct.pack('<IIB',v,j,b) for v,j,b in zip(end,c.default,c.modal))
                        E={r for r in range(n) if L[r]!=c.modal[c.state[r]]}
                        for p in range(n):
                            raw.write_text(str(p)+'\n'+''.join(f'{v} {w}\n'for v,w in runs))
                            ctx.write_bytes(b'CTXMOD01'+struct.pack('<7I',n,p,k,len(f),0,len(E),c.state[p])+defs)
                            data=bytes_for(lib,state,raw,ctx);domain=sorted(E|{p});rank={x:i for i,x in enumerate(domain)}
                            prev=None;want=[]
                            for j,x in enumerate(domain):
                                y=c.lf[x];steps=1
                                while y not in rank:y=c.lf[y];steps+=1
                                t=rank[y]
                                if prev!=(t-1,steps):want.append((j,t,steps))
                                prev=(t,steps)
                            assert list(struct.iter_unpack('<III',data))==want
                            total["marked_models"]+=1;total["physical_returns"]+=len(domain)
    return total
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument("lib",type=Path);p.add_argument("mode",choices=('real','tiny'))
    p.add_argument('files',type=Path,nargs='*');a=p.parse_args();lib=interface(a.lib);s=lib.range_state()
    try:print(json.dumps(real(lib,s,*a.files) if a.mode=='real' else tiny(lib,s),indent=2))
    finally:lib.range_state_free(s)
