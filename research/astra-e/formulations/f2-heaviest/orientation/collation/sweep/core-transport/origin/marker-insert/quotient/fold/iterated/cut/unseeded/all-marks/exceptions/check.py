"""Independent arbitrary-column, endpoint, precompiled-state and chunk emission checks."""
from collections import Counter
import itertools,json
from clock import Clock,seq

tot=Counter();witness={}
for s,limit in [(2,9),(3,5)]:
    for n in range(1,limit+1):
        for L in itertools.product(range(s),repeat=n):
            tot["columns"]+=1
            for k in sorted({0,1,2,3,n+1}):
                c=Clock(L,k);tot["context_tables"]+=1
                for p in range(n):
                    ell,packed=c.lap(p);period=c.emit(packed,ell)
                    assert period==seq(c.F,c.psi,p,ell)
                    if not packed[1]:tot["pure_default_laps"]+=1
                    if ell<n:tot["foreign_cycle_marks"]+=1
                    for N in sorted({0,1,ell-1,ell,ell+1,n,n+1,2*n+3}):
                        want=seq(c.F,c.psi,p,N)
                        got=c.emit(c.finite(p,N),N)
                        assert want==got==(period*((N+ell-1)//ell))[:N]
                        if N==n and n%ell and 'wrong_endpoint' not in witness and k:
                            wrong=c.emit(c.encode_from_end(p,N),N)
                            if wrong!=want:witness['wrong_endpoint']=dict(L=L,k=k,p=p,N=N,ell=ell,wrong=list(wrong),right=list(want))
                        tot["finite_mark_horizons"]+=1
                        if N%ell:tot["mid_cycle_horizons"]+=1
# Even with no future bytes, the postexception state is empty for both byte values.
c=Clock([0,0,0,1,2],0)
assert c.words==[b''] and c.modal==[0]
witness["zero_context"]=dict(physical_L=list(c.L),different_exception_bytes=[1,2],successor_state=0)
print(json.dumps(dict(totals=tot,witnesses=witness),indent=2))
