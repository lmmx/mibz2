"""Size of the batch-means t bound for the window-conditioned target (registration §2.10, DRAFT)
when the residuals are independent but heteroskedastic and heavy-tailed as the pilot's are.
Mean-free: each cell's series is centred, cycle-aggregated and block-demeaned (residual.py's
construction) before use; no mean, ratio or bound of the candidate is computed or printed.

The worst case for the conditional target is a flat path (g = 0: no conservatism from the slow
path). Each simulated qualification window: 24 batches; batch b takes one of the pilot's 10
blocks at random (its variance and tail profile) and 30 cycle residuals resampled with
replacement from that block, rescaled for the block's demeaning. Boundary size: the share of
windows with mean + t(0.95, 23) * sd/sqrt(24) <= 0.

    python -I hetero.py ARTIFACTS_DIR out.json
"""
import json, math, os, sys
import numpy as np
from scipy import stats as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import residual as R

N, B, CYC_B = 20000, 24, 30


def main():
    root, out = sys.argv[1], sys.argv[2]
    rng = np.random.default_rng(6759863882)
    tq = st.t.ppf(0.95, B - 1)
    res = {}
    for sid, (corpus, threads) in R.SHARDS.items():
        rows = [json.loads(l) for l in open(os.path.join(root, f"tg-ni-{sid}-a1", sid, "runs.jsonl"))]
        for t in threads:
            for metric in ("wall", "cpu"):
                c = R.series(rows, corpus, t, metric).reshape(-1, R.CYC).mean(1)
                blocks = c.reshape(-1, R.BLK)
                resid = (blocks - blocks.mean(1, keepdims=True)) * math.sqrt(R.BLK / (R.BLK - 1))
                v = resid.var(1, ddof=0)
                pick = rng.integers(0, len(resid), (N, B))
                draw = rng.integers(0, R.BLK, (N, B, CYC_B))
                y = resid[pick[..., None], draw].mean(-1)
                mu, se = y.mean(1), y.std(1, ddof=1) / math.sqrt(B)
                size = float((mu + tq * se <= 0).mean())
                res[f"{corpus}/{t}/{metric}"] = {"size": size, "block_var_max_over_min": float(v.max() / v.min())}
    json.dump(res, open(out, "w"), indent=1)
    s = sorted((x["size"], k) for k, x in res.items())
    vr = sorted(x["block_var_max_over_min"] for x in res.values())
    print("size: median %.4f, max %.4f (%s), min %.4f; MC SE ~%.4f" % (s[21][0], s[-1][0], s[-1][1], s[0][0], math.sqrt(.05 * .95 / N)))
    print("cells with size > 6%:", [k for v_, k in s if v_ > 0.06])
    print("block variance max/min: median %.1f, max %.1f" % (vr[21], vr[-1]))


if __name__ == "__main__":
    main()
