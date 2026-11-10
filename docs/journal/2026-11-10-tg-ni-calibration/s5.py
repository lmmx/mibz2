"""The frozen residual-dependence check proposed for §2.10 (DRAFT): S5, the sum of the lag 1..5
autocorrelations of the cycle residuals, pairs within a batch only, after each batch's own mean
is removed (the conditioned slow path at batch resolution). Positive residual dependence raises
it. Its threshold is the iid-Gaussian null's 1 - 0.01/42 quantile at the qualification shape
(720 cycles, 24 batches of 30), by simulation. Also: the pilot's own S5 (150 cycles, 10 blocks
of 15) against the same-shape null. Mean-free (residual.py's construction).

    python -I s5.py ARTIFACTS_DIR out.json
"""
import json, os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import residual as R


def s5(c, blk):
    n = c.shape[-1]
    b = c.reshape(*c.shape[:-1], n // blk, blk)
    r = b - b.mean(-1, keepdims=True)
    v = (r ** 2).sum((-1, -2))
    return sum((r[..., L:] * r[..., :-L]).sum((-1, -2)) / v for L in range(1, 6))


def null(n, blk, total, rng, chunk=50000):
    out = []
    while sum(len(x) for x in out) < total:
        out.append(s5(rng.standard_normal((chunk, n)), blk))
    return np.concatenate(out)[:total]


def main():
    rng = np.random.default_rng(6759863883)
    q = 1 - 0.01 / 42
    nq = null(720, 30, 2_000_000, rng)
    thr = float(np.quantile(nq, q))
    npl = null(150, 15, 400_000, rng)
    thr_p, thr_p05 = float(np.quantile(npl, q)), float(np.quantile(npl, 0.95))
    res = {"threshold_720_30": thr, "null_720_30_mean": float(nq.mean()), "null_720_30_sd": float(nq.std()),
           "pilot_null_150_15": {"q_family": thr_p, "q95": thr_p05, "mean": float(npl.mean())}, "cells": {}}
    root = sys.argv[1]
    for sid, (corpus, threads) in R.SHARDS.items():
        rows = [json.loads(l) for l in open(os.path.join(root, f"tg-ni-{sid}-a1", sid, "runs.jsonl"))]
        for t in threads:
            for metric in ("wall", "cpu"):
                c = R.series(rows, corpus, t, metric).reshape(-1, R.CYC).mean(1)
                v = float(s5(c[None, :], 15)[0])
                res["cells"][f"{corpus}/{t}/{metric}"] = {"s5": v, "p": float((npl >= v).mean())}
    json.dump(res, open(sys.argv[2], "w"), indent=1)
    cs = sorted(res["cells"].items(), key=lambda kv: -kv[1]["s5"])
    print("threshold (720/30, 1-0.01/42): %.4f; null mean %.4f sd %.4f" % (thr, res["null_720_30_mean"], res["null_720_30_sd"]))
    print("pilot-shape null: mean %.4f, q95 %.4f, family q %.4f" % (npl.mean(), thr_p05, thr_p))
    print("pilot cells over q95:", sum(x["s5"] > thr_p05 for _, x in cs), " over family q:", sum(x["s5"] > thr_p for _, x in cs))
    print("pilot S5 median %.4f, largest:" % np.median([x["s5"] for _, x in cs]), [(k, round(x["s5"], 3), x["p"]) for k, x in cs[:4]])


if __name__ == "__main__":
    main()
