from collections import Counter
from itertools import product
import json
from study import inspect,make_index,prune,construct
from writer import emit,prepare_psi

stats=Counter();witness={}
for n in range(1,7):
    for l in product((33,34,35),repeat=n):
        last=bytes(l);q,m,lf=make_index(last)
        psi=sorted(range(n),key=last.__getitem__)
        orig=prune(last,q,m,lf,0)[1]
        independent_psi=prepare_psi(last,lf)
        for p in range(n):
            x=p;out=bytearray()
            for _ in range(n):x=psi[x];out.append(last[x])
            expect=out if 34 in out else None
            for k in (1,2,4,8):
                r=inspect(last,p,k,q,m,lf,orig,expect)
                hot,cut=construct(last,q,m,lf,k)
                assert emit(last,p,q,independent_psi,cut,hot)[0]==out
                stats["forward_stream_decodes"]+=1
                stats["decodes_or_refusals"]+=1
                stats["cold_rows"]+=r["cold_rows"]
                stats["mixed_groups"]+=r["mixed_groups"]
                if r["mixed_groups"] and "different_hidden_words_in_one_cut" not in witness:
                    witness["different_hidden_words_in_one_cut"]=dict(last=l,threshold=k,mark=p,
                      cut_groups=r["cold_groups"],missing_destinations=r["cold_rows"],
                      hidden_types=r["hidden_word_types"])
        stats["columns"]+=1
print(json.dumps(dict(stats=stats,witness=witness),indent=2))
