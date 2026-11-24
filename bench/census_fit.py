"""The Wikidata block census against the open kernel's per-block cost (C1a, diagnostic).

    python3 bench/census_fit.py CENSUS.tsv BLOCKS.tsv OUTDIR

CENSUS.tsv is examples/census.rs' output on wikidata-x4.bz2 (one line an open block, in the
1-worker decode order); BLOCKS.tsv is bench-emit-cost-sq's (evidence a01d1f9c,
run-20261123T110350Z). The rule is #145 LOG 7126445058 with its amendments (the LOGs after it:
the held-out split by block identity, the contention row and the manipulability note; the
m-only baseline and the 0.05 entry bar):

- the targets: per block, the base arm's emit and construct ns per retained row at 1 worker,
  each the median over the 20 rounds; the 928 two-ended blocks (the 4 forward-emitted ones,
  m < 65,536, are left out);
- the split: blocks grouped by identity (census `id`, a hash of the block's run records, n and
  p), groups numbered by first appearance, even groups fit, odd groups scored;
- the search: every subset of 1 to 3 of the candidate features below (constant ones dropped),
  ordinary least squares with an intercept on the fit half; the model is chosen on the fit half
  alone (5-fold cross-validation by group within it), and only the chosen model's held-out R²
  is judged. The best held-out R² over every model is reported too, against a permutation
  null of that maximum;
- ADOPT AS SPEC iff a chosen model's held-out R² >= 0.7 on either target; the spec is "match
  m's distribution" if the m-only baseline (m and log m) alone clears 0.7, else m plus each
  named feature that adds >= 0.05 held-out R² over the m-only baseline.
- reported, deciding nothing: the 10-over-1-worker ratio of each cost per group, fitted the
  same way.
"""

import csv
import itertools
import json
import math
import statistics
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np

PERMS = 200
FOLDS = 5
SEED = 20261124


def load_census(path):
    rows = list(csv.DictReader(open(path), delimiter="\t"))
    for r in rows:
        for k, v in r.items():
            if k == "id":
                continue
            r[k] = float(v) if "." in v else int(v)
    return rows


def load_blocks(path):
    out = defaultdict(list)  # (workers, arm, round) -> records in seq order
    for r in csv.DictReader(open(path), delimiter="\t"):
        if r["kind"] != "blk":
            continue
        rec = {k: int(r[k]) for k in "seq n m construct_ns emit_ns bidi refused".split()}
        out[(int(r["workers"]), r["arm"], int(r["round"]))].append(rec)
    for v in out.values():
        v.sort(key=lambda x: x["seq"])
    return out


def features(r):
    n, m = r["n"], r["m"]
    runs = r["runs"]
    rows = [r[k] for k in "rows_1 rows_2_3 rows_4_16 rows_17_127 rows_128".split()]
    ent = -sum((x / n) * math.log2(x / n) for x in rows if x)
    return {
        "log_m": math.log(m),
        "m_over_n": m / n,
        "log_runs": math.log(runs),
        "log_mean_run": math.log(n / runs),
        "runs1_share": r["runs_1"] / runs,
        "rows1_share": rows[0] / n,
        "rows2_3_share": rows[1] / n,
        "rows1_3_share": (rows[0] + rows[1]) / n,
        "rows4_16_share": rows[2] / n,
        "rows17_127_share": rows[3] / n,
        "rows128_share": rows[4] / n,
        "runlen_entropy": ent,
        "log_strips": math.log(r["strips"]),
        "strips_per_m": r["strips"] / m,
        "cycles": r["cycles"],
        "marked_cycle_share": r["marked_cycle"] / m,
        "d_mean_log2": r["d_log2_sum"] / m,
        "log_d_p50": math.log1p(r["d_p50"]),
        "log_d_p90": math.log1p(r["d_p90"]),
        "d_lt16": r["d_lt16"] / m,
        "d_lt1024": r["d_lt1024"] / m,
        "d_lt8192": r["d_lt8192"] / m,
        "d_lt65536": r["d_lt65536"] / m,
        "d_mean_over_m": r["d_sum"] / m / m,
    }


def ols(X, y):
    A = np.column_stack([np.ones(len(X)), X])
    beta, *_ = np.linalg.lstsq(A, y, rcond=None)
    return beta


def predict(beta, X):
    return beta[0] + X @ beta[1:]


def r2(y, yhat):
    ss = ((y - y.mean()) ** 2).sum()
    return 1 - ((y - yhat) ** 2).sum() / ss


def cv_r2(X, y, g, folds):
    """R² of out-of-fold predictions, folds by group"""
    ug = np.unique(g)
    fold_of = {u: i % folds for i, u in enumerate(ug)}
    f = np.array([fold_of[u] for u in g])
    yhat = np.empty_like(y)
    for k in range(folds):
        tr, te = f != k, f == k
        yhat[te] = predict(ols(X[tr], y[tr]), X[te])
    return r2(y, yhat)


def search(F, y, g, fit, score, subsets):
    """every subset: (cv on the fit half, held-out R²)"""
    res = []
    for s in subsets:
        X = F[:, s]
        cv = cv_r2(X[fit], y[fit], g[fit], FOLDS)
        beta = ols(X[fit], y[fit])
        ho = r2(y[score], predict(beta, X[score]))
        res.append((s, cv, ho))
    return res


def ceiling(y, g, score):
    """the held-out R² of each scored group's own mean: what any function of the block's
    features could reach on the score half (copies of a block have the same features)"""
    ys, gs = y[score], g[score]
    means = {u: ys[gs == u].mean() for u in np.unique(gs)}
    return r2(ys, np.array([means[u] for u in gs]))


def analyse(name, F, names, y, g, fit, score, rng, base_idx, perms=PERMS):
    k = F.shape[1]
    subsets = [list(s) for r in (1, 2, 3) for s in itertools.combinations(range(k), r)]
    res = search(F, y, g, fit, score, subsets)
    chosen = max(res, key=lambda t: t[1])
    best = max(res, key=lambda t: t[2])
    # the m-only baseline: m and log m together, fitted the same way
    Xb = np.column_stack([np.exp(F[:, base_idx]), F[:, base_idx]])
    bb = ols(Xb[fit], y[fit])
    base_ho = r2(y[score], predict(bb, Xb[score]))
    single = {names[s[0]]: ho for s, cv, ho in res if len(s) == 1}
    # each feature's gain over m-only: m, log m and that feature
    gains = {}
    for j in range(k):
        if j == base_idx:
            continue
        X = np.column_stack([Xb, F[:, j]])
        gains[names[j]] = r2(y[score], predict(ols(X[fit], y[fit]), X[score])) - base_ho
    # the chosen model's features' gains, and the chosen model's own gain
    # permutation null: group-level targets reassigned among groups (each group's copies keep
    # their own cost vector), the whole search and choice repeated
    ug = np.unique(g)
    idx_of = {u: np.where(g == u)[0] for u in ug}
    null_chosen, null_max = [], []
    for _ in range(perms):
        perm = rng.permutation(len(ug))
        yp = np.empty_like(y)
        for a, b in zip(ug, ug[perm]):
            ia, ib = idx_of[a], idx_of[b]
            # copies: blocks of a group in decode order; groups have equal copy counts here
            yp[ia] = y[ib[: len(ia)]] if len(ib) >= len(ia) else np.resize(y[ib], len(ia))
        rp = search(F, yp, g, fit, score, subsets)
        null_chosen.append(max(rp, key=lambda t: t[1])[2])
        null_max.append(max(t[2] for t in rp))
    out = {
        "target": name,
        "models_tried": len(subsets),
        "features_searched": names,
        "chosen": {
            "features": [names[i] for i in chosen[0]],
            "cv_r2_fit_half": chosen[1],
            "heldout_r2": chosen[2],
            "coef": ols(F[fit][:, chosen[0]], y[fit]).tolist(),
        },
        "best_heldout_over_all_models": {
            "features": [names[i] for i in best[0]],
            "heldout_r2": best[2],
            "null_max_p95": float(np.percentile(null_max, 95)) if perms else None,
            "p_value": (1 + sum(x >= best[2] for x in null_max)) / (1 + perms) if perms else None,
        },
        "null_chosen_heldout_p95": float(np.percentile(null_chosen, 95)) if perms else None,
        "m_only_heldout_r2": base_ho,
        "chosen_gain_over_m_only": chosen[2] - base_ho,
        "gain_over_m_only_by_feature": dict(sorted(gains.items(), key=lambda t: -t[1])),
        "single_feature_heldout_r2": dict(sorted(single.items(), key=lambda t: -t[1])),
        "ceiling_group_means": ceiling(y, g, score) if len(g) > len(np.unique(g)) else None,
        "n_fit": int(fit.sum()),
        "n_score": int(score.sum()),
        "y_fit_mean": float(y[fit].mean()),
        "y_sd": float(y.std()),
    }
    return out


def main():
    census_p, blocks_p, outdir = sys.argv[1:4]
    outdir = Path(outdir)
    census = load_census(census_p)
    blocks = load_blocks(blocks_p)
    one = [blocks[(1, "base", r)] for r in range(20)]
    report = {}
    # the join check: the census's n and m equal the bundle's, block by block, and the census's
    # m equals the kernel's own
    ref = one[0]
    assert all(len(x) == len(census) for x in one), "block counts differ"
    for r in one:
        for c, b in zip(census, r):
            assert (c["n"], c["m"]) == (b["n"], b["m"]), (c["seq"], c["n"], c["m"], b)
    assert all(c["m"] == c["m_kernel"] for c in census)
    report["join"] = (
        f"{len(census)} open blocks: census (n, m) equal the bundle's 1-worker base records' "
        f"in every one of 20 rounds, block by block; census m equals the kernel's own retained "
        f"count on every block"
    )
    # identity groups
    gid, first = {}, []
    for c in census:
        if c["id"] not in gid:
            gid[c["id"]] = len(gid)
        first.append(gid[c["id"]])
    sizes = defaultdict(int)
    for x in first:
        sizes[x] += 1
    report["groups"] = {
        "distinct": len(gid),
        "copies_per_group": dict(
            sorted(
                defaultdict(
                    int, {s: list(sizes.values()).count(s) for s in set(sizes.values())}
                ).items()
            )
        ),
    }
    two = [i for i, b in enumerate(ref) if b["bidi"] == 1 and not b["refused"]]
    report["blocks_two_ended"] = len(two)
    report["blocks_forward_left_out"] = [
        {"seq": i, "n": ref[i]["n"], "m": ref[i]["m"]} for i in range(len(ref)) if i not in set(two)
    ]

    # targets: median over rounds of ns per retained row
    def med(arm_rounds, i, key):
        return statistics.median(r[i][key] / r[i]["m"] for r in arm_rounds)

    emit = np.array([med(one, i, "emit_ns") for i in two])
    cons = np.array([med(one, i, "construct_ns") for i in two])
    feats = [features(census[i]) for i in two]
    names = [k for k in feats[0] if len({f[k] for f in feats}) > 1]
    report["features_dropped_constant"] = {k: feats[0][k] for k in feats[0] if k not in names}
    F = np.array([[f[k] for k in names] for f in feats])
    g = np.array([first[i] for i in two])
    fit = g % 2 == 0
    score = ~fit
    report["split"] = (
        "groups of identical blocks numbered by first appearance in the decode; even groups "
        f"fit ({int(fit.sum())} blocks, {len(set(g[fit]))} groups), odd groups scored "
        f"({int(score.sum())} blocks, {len(set(g[score]))} groups)"
    )
    rng = np.random.default_rng(SEED)
    base_idx = names.index("log_m")
    report["emit"] = analyse(
        "emit ns/row, 1 worker, base", F, names, emit, g, fit, score, rng, base_idx
    )
    report["construct"] = analyse(
        "construct ns/row, 1 worker, base", F, names, cons, g, fit, score, rng, base_idx
    )
    # contention growth: 10 workers over 1, per group, by the (n, m) key where it is unique
    key_groups = defaultdict(set)
    for i in two:
        key_groups[(census[i]["n"], census[i]["m"])].add(first[i])
    ten = [blocks[(10, "base", r)] for r in range(20)]
    by_key = defaultdict(lambda: {"emit_ns": [], "construct_ns": []})
    for r in ten:
        for b in r:
            if b["bidi"] == 1 and not b["refused"]:
                for k in ("emit_ns", "construct_ns"):
                    by_key[(b["n"], b["m"])][k].append(b[k] / b["m"])
    by_key1 = defaultdict(lambda: {"emit_ns": [], "construct_ns": []})
    for r in one:
        for b in r:
            if b["bidi"] == 1:
                for k in ("emit_ns", "construct_ns"):
                    by_key1[(b["n"], b["m"])][k].append(b[k] / b["m"])
    ug = sorted({first[i] for i in two})
    rep = {u: next(i for i in two if first[i] == u) for u in ug}
    uniq = [u for u in ug if len(key_groups[(census[rep[u]]["n"], census[rep[u]]["m"])]) == 1]
    Fg = np.array([[features(census[rep[u]])[k] for k in names] for u in uniq])
    gg = np.array(uniq)
    fitg = gg % 2 == 0
    cont = {}
    for k in ("emit_ns", "construct_ns"):
        yr = np.array(
            [
                statistics.median(by_key[(census[rep[u]]["n"], census[rep[u]]["m"])][k])
                / statistics.median(by_key1[(census[rep[u]]["n"], census[rep[u]]["m"])][k])
                for u in uniq
            ]
        )
        cont[k] = analyse(
            f"{k} per row, 10 over 1 worker (per group)",
            Fg,
            names,
            yr,
            gg,
            fitg,
            ~fitg,
            rng,
            base_idx,
            perms=0,
        )
        cont[k]["ratio_median"] = float(np.median(yr))
        cont[k]["ratio_p10_p90"] = [float(np.percentile(yr, 10)), float(np.percentile(yr, 90))]
    report["contention"] = {"groups_joined_by_unique_n_m": len(uniq), "groups": len(ug), **cont}
    # distributions over the distinct two-ended blocks (one copy of each)
    dist = {}
    for k in list(names) + list(report["features_dropped_constant"]):
        v = np.array([features(census[rep[u]])[k] for u in ug])
        dist[k] = [float(x) for x in np.percentile(v, [0, 10, 50, 90, 100])]
    for k in ("m", "n", "runs", "strips", "d_p50", "d_p90"):
        v = np.array([census[rep[u]][k] for u in ug])
        dist[k] = [float(x) for x in np.percentile(v, [0, 10, 50, 90, 100])]
    dist["emit_ns_per_row"] = [float(x) for x in np.percentile(emit, [0, 10, 50, 90, 100])]
    dist["construct_ns_per_row"] = [float(x) for x in np.percentile(cons, [0, 10, 50, 90, 100])]
    report["distributions_min_p10_p50_p90_max"] = dist
    # the verdict
    passed = {t: report[t]["chosen"]["heldout_r2"] >= 0.7 for t in ("emit", "construct")}
    verdict = "ADOPT AS SPEC" if any(passed.values()) else "REPORT ONLY"
    spec = None
    if verdict == "ADOPT AS SPEC":
        spec = {}
        for t in ("emit", "construct"):
            if not passed[t]:
                continue
            if report[t]["m_only_heldout_r2"] >= 0.7:
                spec[t] = ["m"]
            else:
                spec[t] = ["m"] + [
                    f
                    for f in report[t]["chosen"]["features"]
                    if f != "log_m" and report[t]["gain_over_m_only_by_feature"].get(f, 0) >= 0.05
                ]
    if spec:
        # the spec's own model, reported (m, log m and the features that entered: it may hold
        # more than 3 regressors, so it is not one of the searched models)
        for t, fs in spec.items():
            y = emit if t == "emit" else cons
            cols = [np.exp(F[:, base_idx]), F[:, base_idx]] + [F[:, names.index(f)] for f in fs[1:]]
            X = np.column_stack(cols)
            report[t]["spec_model_heldout_r2"] = r2(
                y[score], predict(ols(X[fit], y[fit]), X[score])
            )
    report["verdict"] = verdict
    report["spec"] = spec
    # Added after the fit (#145, the coordinator's request from MONODROMY record 96): h1, the
    # maximal runs of width 1 (census `runs_1`; the marked row's singleton in a thick run is not
    # counted), and h1/m. Reported rows only: they are not in the registered search above (whose
    # rows1_share is h1/n), and decide nothing.
    h1 = np.array([census[i]["runs_1"] for i in two], dtype=float)
    mm = np.array([census[i]["m"] for i in two], dtype=float)
    post = {
        "h1": [float(x) for x in np.percentile(h1, [0, 10, 50, 90, 100])],
        "h1_over_m": [float(x) for x in np.percentile(h1 / mm, [0, 10, 50, 90, 100])],
        "corr_h1_m": float(np.corrcoef(h1, mm)[0, 1]),
    }
    Xb = np.column_stack([mm, np.log(mm)])
    for t, y in (("emit", emit), ("construct", cons)):
        row = {}
        for nm, v in (("h1", h1), ("log_h1", np.log(h1)), ("h1_over_m", h1 / mm)):
            X1 = v[:, None]
            X2 = np.column_stack([Xb, v])
            row[nm] = {
                "alone_heldout_r2": r2(y[score], predict(ols(X1[fit], y[fit]), X1[score])),
                "gain_over_m_only": r2(y[score], predict(ols(X2[fit], y[fit]), X2[score]))
                - report[t]["m_only_heldout_r2"],
            }
        post[t] = row
    report["post_hoc_h1"] = post
    outdir.mkdir(parents=True, exist_ok=True)
    (outdir / "fit.json").write_text(json.dumps(report, indent=1) + "\n")
    with open(outdir / "features.tsv", "w") as f:
        w = csv.writer(f, delimiter="\t", lineterminator="\n")
        w.writerow(["seq", "group", "half", "emit_ns_per_row", "construct_ns_per_row", *names])
        for j, i in enumerate(two):
            w.writerow(
                [
                    i,
                    first[i],
                    "fit" if fit[j] else "score",
                    f"{emit[j]:.4f}",
                    f"{cons[j]:.4f}",
                    *(f"{x:.6g}" for x in F[j]),
                ]
            )
    for t in ("emit", "construct"):
        a = report[t]
        print(
            f"{t}: chosen {a['chosen']['features']} held-out R² {a['chosen']['heldout_r2']:.3f} "
            f"(cv {a['chosen']['cv_r2_fit_half']:.3f}); m-only {a['m_only_heldout_r2']:.3f}; gain "
            f"{a['chosen_gain_over_m_only']:+.3f}; best of {a['models_tried']} "
            f"{a['best_heldout_over_all_models']['heldout_r2']:.3f} {a['best_heldout_over_all_models']['features']}"
            f" (null max p95 {a['best_heldout_over_all_models']['null_max_p95']}); ceiling "
            f"{a['ceiling_group_means']:.3f}"
        )
    print("verdict:", verdict, spec)


if __name__ == "__main__":
    main()
