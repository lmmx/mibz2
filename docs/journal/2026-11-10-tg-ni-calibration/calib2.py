"""Step 2: boundary-calibrated critical values and power at 1.005 for chosen class members."""
import json, math, sys
import numpy as np
sys.path.insert(0, ".")
import calibrate as C

menu = json.load(open(sys.argv[1]))["cells"]
d = json.load(open(sys.argv[2]))
rng = np.random.default_rng(C.SEED + 1)
tab = C.model_tables()
l10 = {c: {b["m"]: b["lrv"] for b in v["batches"]}[10] for c, v in menu.items()}
N = 200000

def unit_cov(s, f):
    return C.batch_cov(lambda k: (1 - f) * C.white(k) + f * C.gamma(s, k), C.M, C.B)

def draws(cov, n):
    return rng.standard_normal((n, C.B)) @ np.linalg.cholesky(cov + 1e-15 * np.eye(C.B)).T

def stat(y):
    return y.mean(1), y.std(1, ddof=1) / math.sqrt(C.B)

def crit_for(cov, size=0.05):
    mu, se = stat(draws(cov, N))
    # pass iff mu + c*se <= 0, i.e. -mu/se >= c: c* is the (1 - size) quantile of -mu/se
    return float(np.quantile(-mu / se, 1 - size))

def power(s, f, crit, n=40000):
    out = {}
    for c in menu:
        e10 = (1 - f) * tab[s][10][0] + f * tab[s][10][1]
        var = l10[c] / e10
        mu, se = stat(draws(var * unit_cov(s, f), n))
        out[c] = float((mu + crit * se <= C.DELTA).mean())
    miss = sum(1 - p for p in out.values())
    return {"lower": max(0.0, 1 - miss), "min": min(out.values()), "limiting": min(out, key=out.get)}

members = {"white (f 0)": (("ar", 10), 0.0)}
for name, s in (("ar30", ("ar", 30)), ("ar100", ("ar", 100)), ("ar300", ("ar", 300)), ("ar1000", ("ar", 1000)),
                ("ar3000", ("ar", 3000)), ("fgn0.7", ("fgn", 0.7)), ("fgn0.9", ("fgn", 0.9))):
    fh = max(v["f_hat"] for v in d["shapes"][name]["cells"].values())
    members[f"{name}, max point f_hat {fh}"] = (s, fh)
res = {}
for name, (s, f) in members.items():
    cov = unit_cov(s, f)
    mu, se = stat(draws(cov, N))
    size = float((mu + C.TQ * se <= 0).mean())
    cs = crit_for(cov)
    res[name] = {"size_nominal": size, "crit_5pct": cs, "power_nominal": power(s, f, C.TQ), "power_calibrated": power(s, f, cs)}
    print(name, json.dumps(res[name]), flush=True)
json.dump(res, open(sys.argv[3], "w"), indent=1)

# per-cell point estimates: each cell under its own f-hat for the shape, with the nominal t and
# with the shape's class-worst calibrated critical value (the max point f-hat member's c*)
def power_own(name, s, crit, n=40000):
    out = {}
    for c in menu:
        f = d["shapes"][name]["cells"][c]["f_hat"]
        e10 = (1 - f) * tab[s][10][0] + f * tab[s][10][1]
        mu, se = stat(draws(l10[c] / e10 * unit_cov(s, f), n))
        out[c] = float((mu + crit * se <= C.DELTA).mean())
    miss = sum(1 - p for p in out.values())
    return {"lower": max(0.0, 1 - miss), "min": min(out.values()), "limiting": min(out, key=out.get)}

own = {}
for key, (s, f) in members.items():
    if key.startswith("white"):
        continue
    name = key.split(",")[0]
    own[name] = {"nominal": power_own(name, s, C.TQ), "calibrated": power_own(name, s, res[key]["crit_5pct"])}
    print(name, json.dumps(own[name]), flush=True)
res["own_f_hat_power"] = own
json.dump(res, open(sys.argv[3], "w"), indent=1)
