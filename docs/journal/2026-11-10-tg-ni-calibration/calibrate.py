"""Prospective calibration of the target-gated NI batch-means bound (registration §2.9, DRAFT).
Inputs: the pilot's mean-free menu only: menu.json, run 42748681243, as archived in
state/reports/pilot-final-design-6758725018/ at e4f1d8be (SHA-256 a74a3f9e…0ce6). No candidate
mean, ratio or bound is read: the per-cell inputs are LRV_m estimates (variances) at m 10, 20,
30, 48 (first 576 rounds) and 60, all order-balanced (even lengths from even starts).

The class: round-level residual X = sqrt(1 - f) W + sqrt(f) S, unit variance, W white, S a slow
unit-variance Gaussian component of one shape: AR(1) with time constant tau rounds (10 … 3000),
or fractional Gaussian noise of Hurst H (0.6 … 0.9). Per cell and shape, f is fitted to the
menu's LRV growth relative to m 10 (weighted least squares on log ratios, the estimator's exact
finite-sample expectation, so centring's bias is modelled), with a one-sided 95% upper bound by
parametric bootstrap (4000 series of 600 rounds). Each class member's boundary size at
R2880/B24 is then simulated from the exact covariance of the 24 batch means of 120 rounds.

    python -I calibrate.py menu.json fit-and-size.json
    python -I calib2.py menu.json fit-and-size.json calibrated.json
(Python 3.13, numpy, scipy; seeds 6759371719 and 6759371720.)
"""
import json, math, sys
import numpy as np
from scipy import stats

SEED = 6759371719
MENU = ((10, 600), (20, 600), (30, 600), (48, 576), (60, 600))
R, B = 2880, 24
M = R // B
DELTA = math.log(1.02 / 1.005)
CUT = stats.t.ppf(1 - 0.01 / 84, 22)
TQ = stats.t.ppf(0.95, B - 1)
FGRID = np.round(np.linspace(0, 1, 201), 4)
SHAPES = [("ar", t) for t in (10, 30, 100, 300, 1000, 3000)] + [("fgn", h) for h in (0.6, 0.7, 0.8, 0.9)]


def gamma(shape, k):
    k = np.abs(np.asarray(k, dtype=float))
    kind, p = shape
    if kind == "ar":
        return np.exp(-k / p)
    h2 = 2 * p
    return 0.5 * (np.abs(k + 1) ** h2 - 2 * k ** h2 + np.abs(k - 1) ** h2)


def white(k):
    return (np.abs(np.asarray(k)) == 0).astype(float)


def batch_cov(g, m, k):
    """covariance of k consecutive batch means of length m, from autocovariance function g"""
    lags = np.arange(-(m - 1), m)
    w = (m - np.abs(lags)) / m ** 2
    row = np.array([np.sum(w * g(h * m + lags)) for h in range(k)])
    idx = np.abs(np.subtract.outer(np.arange(k), np.arange(k)))
    return row[idx]


def expected_lrv(cov, m):
    k = cov.shape[0]
    return m * (np.trace(cov) - cov.sum() / k) / (k - 1)


def model_tables():
    """per shape: E[LRV_m] of the white part and of the slow part (unit variance each)"""
    t = {}
    for s in SHAPES:
        t[s] = {m: (expected_lrv(batch_cov(white, m, n // m), m),
                    expected_lrv(batch_cov(lambda k: gamma(s, k), m, n // m), m)) for m, n in MENU}
    return t


def ratios(tab, f):
    """E[LRV_m] / E[LRV_10], m 20, 30, 48, 60, at slow share f (array)"""
    e = {m: (1 - f) * w + f * sl for m, (w, sl) in tab.items()}
    return np.stack([e[m] / e[10] for m, _ in MENU[1:]], axis=-1), e[10]


DF = np.array([600 // m - 1 if n == 600 else n // m - 1 for m, n in MENU], float)
W = 1 / (2 / DF[1:] + 2 / DF[0])


def fit(obs_logr, tab):
    """f-hat by weighted least squares on log ratios; obs_logr shape (..., 4)"""
    rho, _ = ratios(tab, FGRID)  # (F, 4)
    lr = np.log(rho)
    d = obs_logr[..., None, :] - lr  # (..., F, 4)
    loss = (W * d ** 2).sum(-1)
    return FGRID[np.argmin(loss, axis=-1)]


def menu_logr(x):
    """x (N, 600): the menu's LRV ratios on simulated series, logged"""
    out = []
    for m, n in MENU:
        bm = x[:, :n].reshape(len(x), n // m, m).mean(-1)
        out.append(m * bm.var(axis=1, ddof=1))
    l = np.stack(out, -1)
    return np.log(l[:, 1:] / l[:, :1])


def size_power(cov, crit, n, rng, delta=None, veto=False):
    L = np.linalg.cholesky(cov + 1e-15 * np.eye(B))
    y = rng.standard_normal((n, B)) @ L.T
    mu = y.mean(1)
    se = y.std(1, ddof=1) / math.sqrt(B)
    ok = mu + crit * se <= (0.0 if delta is None else delta)
    if veto:
        x = np.arange(B) - (B - 1) / 2
        b = (y - mu[:, None]) @ x / (x @ x)
        res = y - mu[:, None] - b[:, None] * x
        tt = b / np.sqrt((res ** 2).sum(1) / (B - 2) / (x @ x))
        ok &= np.abs(tt) <= CUT
    return ok.mean()


def main():
    menu = json.load(open(sys.argv[1]))["cells"]
    rng = np.random.default_rng(SEED)
    tab = model_tables()
    obs = {}
    for c, v in menu.items():
        l = {b["m"]: b["lrv"] for b in v["batches"]}
        obs[c] = (np.log(np.array([l[m] / l[10] for m, _ in MENU[1:]])), l[10])
    out = {"seed": SEED, "cut": CUT, "tq": TQ, "growth": {c: {str(m): float(math.exp(o[0][i])) for i, (m, _) in enumerate(MENU[1:])} for c, o in obs.items()}}
    # null check: how often does white noise alone produce growth like the pilot's?
    N = 4000
    wsim = rng.standard_normal((N, 600))
    wl = menu_logr(wsim)
    out["white_null"] = {"p_growth60_ge_obs": {c: float((wl[:, 3] >= o[0][3]).mean()) for c, o in obs.items()}}
    # per shape: slow draws, f-hat sampling distribution on the f grid, per-cell fit and 95% upper bound
    res = {}
    FB = FGRID[::5]  # bootstrap grid, step 0.025
    for s in SHAPES:
        g = gamma(s, np.subtract.outer(np.arange(600), np.arange(600)))
        Ls = np.linalg.cholesky(g + 1e-10 * np.eye(600))
        ssim = rng.standard_normal((N, 600)) @ Ls.T
        wsim = rng.standard_normal((N, 600))
        fh = {}
        for f in FB:
            x = math.sqrt(1 - f) * wsim + math.sqrt(f) * ssim
            fh[float(f)] = fit(menu_logr(x), tab[s])
        cells = {}
        for c, (lr, l10) in obs.items():
            f_hat = float(fit(lr[None, :], tab[s])[0])
            # one-sided 95% upper bound: the largest f whose bootstrap P(f-hat* <= f_hat_obs) >= 0.05
            fu = max([f for f in fh if (fh[f] <= f_hat).mean() >= 0.05] or [0.0])
            cells[c] = {"f_hat": f_hat, "f_upper95": fu}
        # size at R2880/B24 across the f grid for this shape
        g120 = lambda k, f: (1 - f) * white(k) + f * gamma(s, k)
        sz = {}
        for f in FB:
            cov = batch_cov(lambda k: g120(k, f), M, B)
            sz[float(f)] = float(size_power(cov, TQ, 100000, rng))
        res[f"{s[0]}{s[1]}"] = {"cells": cells, "size_by_f": sz}
        print(s, "max f_hat", max(v["f_hat"] for v in cells.values()), "max f_U", max(v["f_upper95"] for v in cells.values()),
              "size at max f_U", sz[max(v["f_upper95"] for v in cells.values())], flush=True)
    out["shapes"] = res
    json.dump(out, open(sys.argv[2], "w"), indent=1)


if __name__ == "__main__":
    main()
