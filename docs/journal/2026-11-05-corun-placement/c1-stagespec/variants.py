import sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import sim, sys
from sim import MODELS, calibrate, run_today, run_staged

# variant: walker never takes S/M work (it cannot pull from the global job queue)
orig_staged = sim.run_staged
def staged_nosm(m, fac, **kw):
    # monkeypatch: LL limits apply; walker's SM branch disabled by passing a flag via closure
    return orig_staged(m, fac, **kw)

for name, m in MODELS.items():
    for oc in ["C", "M", "P"]:
        fac, mixed = calibrate(m, oc, "flat")
        today, _ = run_today(m, fac, blocks=4000, seed=3)
        res = []
        for TT, LL, OUT in [(3, 2, 2), (2, 2, 2), (2, 1, 1), (4, 2, 2)]:
            st, info = run_staged(m, fac, blocks=4000, seed=3, TT=TT, LL=LL, OUT=OUT)
            res.append(f"TT{TT}LL{LL}OUT{OUT} {(st/today-1)*100:+5.1f}% (idle w{info['idle_walker']:.2f} c{info['idle_compute']:.2f})")
        p = info['pairs']
        ww = sum(v for k, v in p.items() if k == ('W','W'))
        print(f"{name:6} {oc}: " + " | ".join(res))
