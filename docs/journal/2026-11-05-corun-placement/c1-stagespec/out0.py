import sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sim import MODELS, calibrate, run_today, run_staged
for name, m in MODELS.items():
    for oc in ["C", "M", "P"]:
        fac, _ = calibrate(m, oc, "flat")
        today, _ = run_today(m, fac, blocks=4000, seed=3)
        r = []
        for OUT in (0, 1, 2):
            st, info = run_staged(m, fac, blocks=4000, seed=3, TT=2, OUT=OUT, walker_sm=False)
            r.append(f"OUT{OUT} {(st/today-1)*100:+5.1f}%")
        # seeds spread at TT2 OUT1
        sp = [ (run_staged(m, fac, blocks=4000, seed=s, TT=2, OUT=1, walker_sm=False)[0] / run_today(m, fac, blocks=4000, seed=s)[0] - 1)*100 for s in (11, 12, 13)]
        print(f"{name:6} {oc}: " + " | ".join(r) + f" | seeds {min(sp):+.1f}..{max(sp):+.1f}")
