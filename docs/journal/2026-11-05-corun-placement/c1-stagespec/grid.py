import sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import sim
from sim import MODELS, Factors, run_today, run_staged

def pattern(k, npat):
    def g(x, y):
        if y is None: return 0.0
        if x == "W": return 1.0 if y == "W" else k
        if npat == "flat": return 1.0
        if npat == "prefers_walk": return 0.6 if y == "W" else 1.0
        if npat == "dislikes_walk": return 1.0 if y == "W" else 0.6
    return g

def calib(m, g, iters=25):
    a = {s: m["mixed"][s] - 1.0 for s in "SMBWF"}
    for _ in range(iters):
        _, mixed = run_today(m, Factors(a, g), blocks=1500)
        for s in "SMBWF":
            a[s] *= (m["mixed"][s] - 1.0) / max(mixed[s] - 1.0, 1e-6)
    fac = Factors(a, g)
    _, mixed = run_today(m, fac, blocks=3000, seed=7)
    return fac, mixed

for name, m in MODELS.items():
    for npat in ["flat", "prefers_walk", "dislikes_walk"]:
        cells = []
        for k in [1.0, 0.75, 0.5, 0.35, 0.25, 0.1]:
            fac, mixed = calib(m, pattern(k, npat))
            today, _ = run_today(m, fac, blocks=4000, seed=3)
            st, _ = run_staged(m, fac, blocks=4000, seed=3, TT=2, walker_sm=False)
            ok = abs(mixed["W"] - m["mixed"]["W"]) < 0.05
            cells.append(f"fWN {fac.f('W','M'):.2f}/fWW {fac.f('W','W'):.2f}{'' if ok else '!'} {(st/today-1)*100:+5.1f}%")
        print(f"{name:6} {npat:13} " + " | ".join(cells))
