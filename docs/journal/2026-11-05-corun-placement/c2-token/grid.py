import sys
sys.path.insert(0, ".")
from des import Sim, calib, RUNNERS
N, seeds = 600, (1, 2, 3, 4)
def avg(alone, f, pol, **kw):
    rs = [Sim(alone, f, pol, N, seed=s, **kw).run() for s in seeds]
    return {k: sum(r[k] for r in rs) / len(rs) for k in rs[0]}
for name, (alone, mixed) in RUNNERS.items():
    print(f"== {name}  (token + per-core table pool of 2, release to a parked sibling, else free)")
    print("   delta  f(W|C)  f(W|W) |  cw=0.0          cw=0.1          cw=0.2   | live today/token  park  w|idle")
    for d in (-0.15, 0.0, 0.15, 0.3, 0.45, 0.6, 0.8, 1.0):
        row, extra = [], ""
        for cw in (0.0, 0.1, 0.2):
            f, _ = calib(alone, mixed, "P" if d else "C", d, cw)
            b = avg(alone, f, "today")
            t = avg(alone, f, "token", wcap=2, ccap=1, keep=False)
            row.append(f"{100*(t['T']/b['T']-1):+5.1f}% (tb {t['tok_busy']:.2f})")
            if cw == 0.0:
                extra = (f"{b['live_mean']:.2f}/{t['live_mean']:.2f}  {t['park']:.3f}  {t['w_idle']:.2f}")
                fw = (f['W']['L'], f['W']['W'])
        print(f"   {d:+.2f}  {fw[0]:.2f}    {fw[1]:.2f}  | " + "  ".join(row) + " | " + extra)
