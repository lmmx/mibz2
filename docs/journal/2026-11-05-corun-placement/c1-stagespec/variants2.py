import sys; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sim import MODELS, calibrate, run_today, run_staged
for name, m in MODELS.items():
    for oc in ["C", "M", "P"]:
        fac, mixed = calibrate(m, oc, "flat")
        today, _ = run_today(m, fac, blocks=4000, seed=3)
        out = []
        for label, kw in [("base", {}), ("walker-builds-only", dict(wb_only=True)), ("walker-no-SM", dict(walker_sm=False)),
                          ("TT2-noSM", dict(TT=2, walker_sm=False))]:
            st, info = run_staged(m, fac, blocks=4000, seed=3, **kw)
            p = info["pairs"]
            wn = sum(v for k, v in p.items() if k[0] == "W" and k[1] not in ("W", None))
            wi = sum(v for k, v in p.items() if k[0] == "W" and k[1] is None)
            out.append(f"{label} {(st/today-1)*100:+5.1f}% (W|N {wn:.2f} W|idle {wi:.2f} idle c {info['idle_compute']:.2f})")
        print(f"{name:6} {oc}: " + " | ".join(out))
