import sys
sys.path.insert(0, ".")
from des import Sim, calib, RUNNERS

def calib_m2(alone, mixed, delta, bdelta):
    """M for both the walk and the build: each costs `delta` (`bdelta`) more beside a
    memory-heavy sibling (W or B) than beside a light one, the mixture kept"""
    f, p = calib(alone, mixed, "M", delta)
    m = mixed["B"]
    c = m - (p["W"] + p["H"]) * bdelta
    f["B"].update(L=c, H=c + bdelta, W=c + bdelta)
    return f

N = 600
seeds = (1, 2, 3)
def avg(alone, f, pol, **kw):
    rs = [Sim(alone, f, pol, N, seed=s, **kw).run() for s in seeds]
    return {k: sum(r[k] for r in rs) / len(rs) for k in rs[0]}

for name in ("7763", "9V74"):
    alone, mixed = RUNNERS[name]
    print("==", name)
    for d, bd in ((0.6, 0.0), (0.6, 0.4), (1.0, 0.4), (1.0, 0.8)):
        f = calib_m2(alone, mixed, d, bd)
        b = avg(alone, f, "today")
        t = avg(alone, f, "token", wcap=2, ccap=1, keep=False)
        m = avg(alone, f, "token", wcap=2, ccap=1, keep=False, buildtok=True)
        print(f"  M walk d={d} build d={bd}: f(W|L)={f['W']['L']:.2f} f(B|L)={f['B']['L']:.2f} "
              f"| token+pool {100*(t['T']/b['T']-1):+5.1f}% | memory token {100*(m['T']/b['T']-1):+5.1f}%"
              f" (tok_busy {m['tok_busy']:.2f})")
    # pool + simple release (keep=0) against keep=1, and window 8 vs 16, under P d=0.6
    f, _ = calib(alone, mixed, "P", 0.6)
    b16 = avg(alone, f, "today")
    for win in (8, 16, 32):
        b = avg(alone, f, "today", win=win)
        t = avg(alone, f, "token", wcap=2, ccap=1, keep=False, win=win)
        print(f"  P d=0.6 window {win}: today {4*b['per_block']:.2f} ({100*(b['T']/b16['T']-1):+.1f}% vs w16)"
              f" token+pool keep0 {100*(t['T']/b['T']-1):+5.1f}% jobwait {t['jobwait']:.3f} park {t['park']:.3f}")
    for jit in (0.0, 0.1, 0.5):
        b = avg(alone, f, "today", jit=jit)
        t = avg(alone, f, "token", wcap=2, ccap=1, keep=False, jit=jit)
        print(f"  P d=0.6 block-size jitter {jit}: token+pool {100*(t['T']/b['T']-1):+5.1f}% (today ww {b['ww']:.2f})")
