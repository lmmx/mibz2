import os
# Throwaway: measure the emergent sibling-stage share during walks in stagespec's 'today' sim
import sys, random
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "../c1-stagespec"))
import sim
from sim import MODELS, calibrate, jit

def today_shares(m, fac, blocks=4000, seed=5):
    rng = random.Random(seed); alone = m["alone"]
    def nb(): return [(s, jit(rng, alone[s])) for s in "SMBWF"]
    th = [nb(), nb()]
    off = rng.uniform(0, sum(alone.values()))
    while off > 0 and th[1]:
        s, r = th[1][0]
        if r <= off: off -= r; th[1].pop(0)
        else: th[1][0] = (s, r - off); off = 0
    if not th[1]: th[1] = nb()
    pair = {}; done = 0; warm = blocks // 10
    while done < blocks:
        st = [th[i][0][0] for i in (0, 1)]
        rates = [1.0 / fac.f(st[0], st[1]), 1.0 / fac.f(st[1], st[0])]
        dt = min(th[i][0][1] / rates[i] for i in (0, 1))
        if done >= warm:
            for i in (0, 1):
                k = (st[i], st[1 - i]); pair[k] = pair.get(k, 0) + dt
        for i in (0, 1):
            s, r = th[i][0]; th[i][0] = (s, r - dt * rates[i])
        for i in (0, 1):
            if th[i][0][1] <= 1e-12:
                th[i].pop(0)
                if not th[i]: done += 1; th[i] = nb()
    return pair

for name in ["7763", "9V74", "8370C"]:
    m = MODELS[name]
    for oc in ["C", "M", "P"]:
        fac, mixed = calibrate(m, oc, "flat")
        pr = today_shares(m, fac)
        wt = sum(v for (x, y), v in pr.items() if x == "W")
        sh = {y: sum(v for (x, yy), v in pr.items() if x == "W" and yy == y) / wt for y in "SMBWF"}
        tot = sum(pr.values()) / 2
        wshare = wt / (2 * tot)
        print(f"{name} {oc}: fWW {fac.f('W','W'):.2f} fWB {fac.f('W','B'):.2f} fWM {fac.f('W','M'):.2f} cal {mixed['W']:.2f} | "
              f"walk time share {wshare:.3f} | sibling during my walk: " + " ".join(f"{y}:{sh[y]:.2f}" for y in "SMBWF"))
