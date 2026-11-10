"""The conditional estimand's residual checks (registration §2.10, DRAFT), on the pilot's rows.
Mean-free: every cell's paired log series is centred on its own mean before anything else, and
no mean, ratio or bound is computed or printed.

    python -I residual.py ARTIFACTS_DIR out.json

Per cell (42): X_i = log(head/base) at round i, centred; cycle means c_j over complete
four-round order cycles (order's constant and linear terms cancel); the conditioned slow path
taken at batch resolution: c_j minus its block's mean, blocks of 15 cycles (60 rounds, the
pilot's longest menu batch; 10 blocks). On those within-block residuals:
- lag 1, 2, 3, 5 autocorrelation, pairs within a block only;
- Ljung-Box over lags 1..5 (within-block pairs);
- Bartlett's statistic for equal residual variance across the 10 blocks;
- excess kurtosis.
p-values are simulated under the working null (iid Gaussian cycle noise around an arbitrary
block-level path), with the same structure, so demeaning bias is accounted for.
"""
import json, math, os, sys
import numpy as np

SHARDS = {c: (c, (1, 2, 4)) for c in ("onebyte", "pattern", "random", "records", "runs", "text")}
SHARDS.update({f"wikidata-{t}": ("wikidata", (t,)) for t in (1, 2, 4)})
ARMS = ("mibz2", "mibz2@base/ab")
CYC, BLK = 4, 15
LAGS = (1, 2, 3, 5)


def series(rows, corpus, t, metric, R=600):
    by = {}
    for r in rows:
        if r["corpus"] == corpus and r["threads"] == t:
            by[(r["round"], r["arm"])] = r[metric]
    x = np.array([math.log(by[(i, ARMS[0])] / by[(i, ARMS[1])]) for i in range(R)])
    return x - x.mean()


def stats(c):
    """c: (..., n) cycle means; returns dict of arrays over the leading axes"""
    n = c.shape[-1]
    k = n // BLK
    b = c[..., :k * BLK].reshape(*c.shape[:-1], k, BLK)
    r = b - b.mean(-1, keepdims=True)
    v = (r ** 2).sum((-1, -2))
    acf = {}
    for L in LAGS:
        acf[L] = (r[..., L:] * r[..., :-L]).sum((-1, -2)) / v
    nb = k * BLK
    # Ljung-Box over lags 1..5 (within-block pairs)
    q = 0
    for L in range(1, 6):
        a = (r[..., L:] * r[..., :-L]).sum((-1, -2)) / v
        q = q + a ** 2 / (nb - k * L)
    q = nb * (nb + 2) * q
    s2 = (r ** 2).sum(-1) / (BLK - 1)
    pooled = s2.mean(-1)
    bart = (k * (BLK - 1) * np.log(pooled) - (BLK - 1) * np.log(s2).sum(-1))
    flat = r.reshape(*r.shape[:-2], -1)
    kurt = (flat ** 4).mean(-1) / ((flat ** 2).mean(-1)) ** 2 - 3
    return {"acf": acf, "lb": q, "bartlett": bart, "kurt": kurt}


def main():
    root, out = sys.argv[1], sys.argv[2]
    rng = np.random.default_rng(6759863881)
    null = stats(rng.standard_normal((20000, 150)))
    res = {}
    for sid, (corpus, threads) in SHARDS.items():
        rows = [json.loads(l) for l in open(os.path.join(root, f"tg-ni-{sid}-a1", sid, "runs.jsonl"))]
        for t in threads:
            for metric in ("wall", "cpu"):
                x = series(rows, corpus, t, metric)
                c = x.reshape(-1, CYC).mean(1)
                s = stats(c[None, :])
                p = lambda k: float((null[k] >= s[k][0]).mean())
                res[f"{corpus}/{t}/{metric}"] = {
                    "acf": {str(L): float(s["acf"][L][0]) for L in LAGS},
                    "acf_null_mean": {str(L): float(null["acf"][L].mean()) for L in LAGS},
                    "lb": float(s["lb"][0]), "p_lb": p("lb"),
                    "bartlett": float(s["bartlett"][0]), "p_bartlett": p("bartlett"),
                    "kurt": float(s["kurt"][0]), "p_kurt": p("kurt")}
    json.dump(res, open(out, "w"), indent=1)
    v = list(res.values())
    for k in ("p_lb", "p_bartlett", "p_kurt"):
        ps = sorted(x[k] for x in v)
        print(k, "cells p<0.05:", sum(p < 0.05 for p in ps), " p<0.005/42:", sum(p < 0.005 / 42 for p in ps), " smallest:", ps[:4])
    a1 = sorted(x["acf"]["1"] for x in v)
    print("acf1 within-block residual: median %.3f, range %.3f..%.3f, null mean %.3f" % (a1[21], a1[0], a1[-1], v[0]["acf_null_mean"]["1"]))
    for c, x in sorted(res.items(), key=lambda kv: kv[1]["p_lb"])[:6]:
        print(c, "acf1 %.3f acf2 %.3f p_lb %.4f p_bart %.4f kurt %.2f p_kurt %.4f" % (x["acf"]["1"], x["acf"]["2"], x["p_lb"], x["p_bartlett"], x["kurt"], x["p_kurt"]))


if __name__ == "__main__":
    main()
