import os
# Throwaway: C2's DES 'today' arm, its emergent walk|walk share and walk factor under each outcome
import sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "../c2-token"))
from des import Sim, calib, RUNNERS, observed_T4
for name in ["7763", "9V74"]:
    alone, mixed = RUNNERS[name]
    obs = observed_T4(alone, mixed)
    for oc, d in [("C", 0.0), ("P", 0.6), ("M", 0.6), ("M", 1.0)]:
        f, p = calib(alone, mixed, oc, d, 0.0)
        rs = [Sim(alone, f, "today", 400, seed=s).run() for s in (1, 2, 3)]
        b = {k: sum(r[k] for r in rs) / 3 for k in rs[0]}
        t = [Sim(alone, f, "token", 400, seed=s, wcap=2, ccap=1, keep=False).run() for s in (1, 2, 3)]
        tk = sum(r["T"] for r in t) / 3
        print(f"{name} {oc} d={d}: pW {p['W']:.3f} f(W|L) {f['W']['L']:.2f} f(W|W) {f['W']['W']:.2f} | today 4*per_block "
              f"{4*b['per_block']:.2f} vs observed {obs:.2f} ({100*(4*b['per_block']/obs-1):+.1f}%), today ww {b['ww']:.2f} "
              f"| token {100*(tk/b['T']-1):+.1f}%")
