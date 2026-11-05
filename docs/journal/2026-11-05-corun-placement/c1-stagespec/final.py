import sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sim import MODELS, calibrate, run_today, run_staged
print("model  outcome  N=flat   N=prefers_walk  N=dislikes_walk   (staged core time per block vs today; TT2 OUT1, walker pulls no jobs)")
for name, m in MODELS.items():
    for oc in ["C", "M", "P", "P25"]:
        cells = []
        for npat in ["flat", "prefers_walk", "dislikes_walk"]:
            fac, mixed = calibrate(m, oc, npat)
            today, _ = run_today(m, fac, blocks=4000, seed=5)
            st, info = run_staged(m, fac, blocks=4000, seed=5, TT=2, OUT=1, walker_sm=False)
            cells.append(f"{(st/today-1)*100:+6.1f}% [fWN {fac.f('W','M'):.2f} fWW {fac.f('W','W'):.2f} cal {mixed['W']:.2f}]")
        print(f"{name:6} {oc:4} " + "  ".join(cells))
