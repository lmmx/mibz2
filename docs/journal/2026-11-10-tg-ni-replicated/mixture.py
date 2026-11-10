"""Boundary-size sensitivity of the frozen 12-dispatch bound under runner-model mixtures, from
neutral evidence only (registration §2.11; the Researcher's REVIEW 6763089509 §2). Sensitivity
models, not coverage: no candidate data, no weights, critical value or k chosen from them.

Per cell and metric: the strict neutral links' 10-round mean log ratios (plan.py's set), each
tagged with its runner model (AMD 7763, AMD 9V74, Intel). A scenario is a mixture over those
links: model weights w_m, links equally weighted within a model. Its population mean is moved
to log 1.02 exactly (the null boundary), the within-model spread and between-model offsets
kept. 12 dispatch contrasts are drawn iid from the mixture; the cell passes if
exp(mean + t(0.95, 11) sd / sqrt(12)) <= 1.02. Scenarios: the links as observed; equal model
weights; each model left out; each model at 80%. Also iid normal with the mixture's variance.
These 10-round contrasts carry more within-run noise than an R240 dispatch would; that is a
stated limitation. 40,000 draws a scenario and cell; binomial SE about 0.11 points at 5%.

    python -I mixture.py links.jsonl out.json
"""
import json, math, sys
import numpy as np
from scipy import stats

HARNESS = {69, 74, 219, 290, 298, 326}
CORPORA = ("onebyte", "pattern", "random", "records", "runs", "text", "wikidata")
K, N = 12, 40000
TQ = stats.t.ppf(0.95, K - 1)
B = math.log(1.02)


def model(cpu):
    return "7763" if "7763" in cpu else "9V74" if "9V74" in cpu else "9V45" if "9V45" in cpu else "Intel" if "ntel" in cpu or "NTEL" in cpu else cpu


def main():
    links = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
    S = [x for x in links if str(x["bench"]).startswith("3.") and x["balanced"] and x["decoder"] is False
         and x["rounds"] == 10 and x["cores"] == 4 and x["pr"] not in HARNESS]
    tags = [model(x["cpu"]) for x in S]
    models = sorted(set(tags))
    counts = {m: tags.count(m) for m in models}
    scen = {"as observed": {m: counts[m] / len(S) for m in models},
            "equal model weights": {m: 1 / len(models) for m in models}}
    for m in models:
        scen[f"without {m}"] = {o: (counts[o] if o != m else 0) for o in models}
        scen[f"{m} at 80%"] = {o: (0.8 if o == m else 0.2 * counts[o] / (len(S) - counts[m])) for o in models}
    rng = np.random.default_rng(6763089509)
    out = {"links": len(S), "models": counts, "scenarios": {}}
    for name, w in scen.items():
        tot = sum(w.values())
        p = np.array([w[t] / tot / counts[t] for t in tags])
        sizes, normal = {}, {}
        for c in CORPORA:
            for th in ("1", "2", "4"):
                for metric in ("wall", "cpu"):
                    y = np.array([x["change"][c][th][metric][0] for x in S])
                    mu = (p * y).sum()
                    y = y - mu + B
                    var = (p * (y - B) ** 2).sum()
                    draw = y[rng.choice(len(y), size=(N, K), p=p)]
                    m_, sd = draw.mean(1), draw.std(1, ddof=1)
                    sizes[f"{c}/{th}/{metric}"] = float((m_ + TQ * sd / math.sqrt(K) <= B).mean())
                    z = B + math.sqrt(var) * rng.standard_normal((N, K))
                    normal[f"{c}/{th}/{metric}"] = float((z.mean(1) + TQ * z.std(1, ddof=1) / math.sqrt(K) <= B).mean())
        v = sorted(sizes.values())
        out["scenarios"][name] = {"weights": w, "size": sizes, "normal": normal,
                                  "median": v[21], "max": v[-1], "argmax": max(sizes, key=sizes.get),
                                  "cells_over_6pct": sum(x > 0.06 for x in v), "normal_max": max(normal.values())}
        print(f"{name:22s} median {v[21]:.4f} max {v[-1]:.4f} ({max(sizes, key=sizes.get)}) cells>6%: "
              f"{sum(x > 0.06 for x in v)}  [iid normal, same variance: max {max(normal.values()):.4f}]")
    json.dump(out, open(sys.argv[2], "w"), indent=1)
    print(counts)


if __name__ == "__main__":
    main()
