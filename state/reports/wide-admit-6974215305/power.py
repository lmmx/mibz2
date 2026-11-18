"""Prospective power from the matched #535 logs. Requires scipy."""
import json
import math
from pathlib import Path
from statistics import mean, stdev
import sys
from scipy.stats import nct, t


def power(ratio, sd, rounds):
    """Single-cell chance to meet Holm's first threshold in a two-cell family."""
    cutoff = -t.ppf(.975, rounds - 1)
    shift = math.log(ratio) * math.sqrt(rounds) / sd
    return float(nct.cdf(cutoff, rounds - 1, shift))


def rows(path):
    data = [json.loads(line) for line in Path(path).read_text().splitlines()]
    assert len(data) == 80
    out = {}
    for threads in (10, 12):
        paired = {}
        for row in data:
            if row["threads"] == threads and row["corpus"] == "wikidata-x4":
                arm = row["arm"].split("@")[0]
                paired.setdefault(row["round"], {})[arm] = row["wall"]
        assert set(paired) == set(range(10))
        logs = [math.log(x["mibz2-fused"] / x["mibz2"]) for x in paired.values()]
        mu, sd = mean(logs), stdev(logs)
        half = t.ppf(.975, 9) * sd / math.sqrt(10)
        out[str(threads)] = dict(log_sd=sd, ratios=[math.exp(mu), math.exp(mu - half),
                                                  math.exp(mu + half)],
                                 p=float(t.cdf(mu * math.sqrt(10) / sd, 9)))
    return out


fused = rows(sys.argv[1])
scenarios = {f"fused {key}t": item["log_sd"] for key, item in fused.items()}
scenarios.update({str(sd): sd for sd in (.04, .05, .06, .08)})
out = {}
for label, sd in scenarios.items():
    out[label] = {
        "sd": sd,
        "power10": {str(r): power(r, sd, 10) for r in (.985, .97, .96)},
        "rounds80_at_0.97": next(n for n in range(3, 1000) if power(.97, sd, n) >= .8),
    }
print(json.dumps({"fused": fused, "scenarios": out,
                  "sd_max_at_10_for_observed_0.97": -math.log(.97) * math.sqrt(10) / t.ppf(.975, 9)},
                 indent=2))
