import os
# Throwaway: C1's staged arm against a 'today' whose pairs are re-drawn at block ends (4 threads on
# 2 cores, as the migrating decoder), with the SAME pair factors C1 calibrated on its pinned today.
import sys, random
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "../c1-stagespec"))
from sim import MODELS, calibrate, run_today, run_staged, jit

def today_repaired(m, fac, blocks=8000, seed=5, swap=1.0):
    rng = random.Random(seed); alone = m["alone"]
    def nb(): return [(s, jit(rng, alone[s])) for s in "SMBWF"]
    th = [nb() for _ in range(4)]
    for i in range(1, 4):  # random phase offsets
        off = rng.uniform(0, sum(alone.values()))
        while off > 0 and th[i]:
            s, r = th[i][0]
            if r <= off: off -= r; th[i].pop(0)
            else: th[i][0] = (s, r - off); off = 0
        if not th[i]: th[i] = nb()
    cpu = [0, 1, 2, 3]  # cpu[t] = cpu of thread t; siblings (0,1), (2,3)
    t = 0.0; done = 0; warm = blocks // 10; t0 = None
    ww = wt = 0.0; wtime = wwork = 0.0
    while done < blocks:
        at = {cpu[i]: i for i in range(4)}
        sib = [at[cpu[i] ^ 1] for i in range(4)]
        st = [th[i][0][0] for i in range(4)]
        rates = [1.0 / fac.f(st[i], st[sib[i]]) for i in range(4)]
        dt = min(th[i][0][1] / rates[i] for i in range(4))
        if done >= warm:
            for i in range(4):
                if st[i] == "W":
                    wt += dt; wtime += dt; wwork += dt * rates[i]
                    if st[sib[i]] == "W": ww += dt
        for i in range(4):
            s, r = th[i][0]; th[i][0] = (s, r - dt * rates[i])
        t += dt
        for i in range(4):
            if th[i][0][1] <= 1e-12:
                th[i].pop(0)
                if not th[i]:
                    done += 1; th[i] = nb()
                    if done == warm: t0 = t
                    if rng.random() < swap:  # migrate: swap with a thread on the other core
                        j = rng.choice([k for k in range(4) if cpu[k] // 2 != cpu[i] // 2])
                        cpu[i], cpu[j] = cpu[j], cpu[i]
    per_block_core = (t - t0) / (blocks - warm) * 2 / 2  # 4 threads, 2 cores: core time per block = span*2/blocks
    return (t - t0) * 2 / (blocks - warm), ww / wt, wtime / wwork

for name in ["7763", "9V74", "8370C"]:
    m = MODELS[name]
    for oc in ["C", "M", "P"]:
        fac, mixed = calibrate(m, oc, "flat")
        pinned, _ = run_today(m, fac, blocks=4000, seed=5)
        rep, wwshare, fw = today_repaired(m, fac)
        st, info = run_staged(m, fac, blocks=4000, seed=5, TT=2, OUT=1, walker_sm=False)
        print(f"{name} {oc}: today pinned {pinned:.2f}  today re-paired {rep:.2f} (walk|walk {wwshare:.2f}, walk factor {fw:.2f})"
              f"  staged {st:.2f}: vs pinned {100*(st/pinned-1):+.1f}%  vs re-paired {100*(st/rep-1):+.1f}%")
