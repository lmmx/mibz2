"""Untimed replay of the #576 adjudicator with synthetic counter records."""
from collections import Counter
import copy
import json
import math
from pathlib import Path
import tempfile

from scipy.optimize import brentq
from scipy.stats import nct, t

import sustain
from test_sustain import judge


def record(phase, seq, corpus="wikidata-x16", index=None):
    counts = {field: [1000, 100.0, 1] for field in sustain.FIELD.values()}
    entry = dict(phase=phase, seq=seq, placement="unpinned", cell="u10", threads=10,
                 round=0, arm="mibz2-occ-7-28", corpus=corpus, rc=0, out=100,
                 hash=corpus, counts=counts, cpu=4.0, wall=1.0, rss_kb=100,
                 cpus="0", thread_cpu=" ".join(f"mibz2-{i}=0.1" for i in range(7)))
    if index is not None:
        entry["index"] = index
    return entry


def set_clock(row, x):
    row["counts"]["cycles"][0] = 1000 * x


def fixture(mutate):
    rows = [record("prewarm", 0), record("warm", 1)]
    for i, corpus in enumerate(("wikidata-x16", "text-x16", "random-x8", "wikidata-x16")):
        rows.append(record("measure", i+2, corpus, i))
    mutate(rows)
    with tempfile.TemporaryDirectory() as scratch:
        path = Path(scratch) / "runs.jsonl"
        path.write_text("".join(json.dumps(row) + "\n" for row in rows))
        rc, out, judged = judge(path, "unpinned", ["--clamped", "u10"])
        return dict(rc=rc, text=out.strip(), judged_written=judged.exists())


def straddle(rows, lo, hi):
    set_clock(rows[2], lo)
    set_clock(rows[5], hi)


def bad_prewarm(rows):
    rows[0]["rc"] = 1
    rows[0]["err"] = "failed prewarm"
    rows[0]["counts"] = {}


def placement_power(m, n, sd, ratio):
    alpha = .025 / m
    q = t.isf(alpha, n - 1)
    effect = -math.log(ratio)
    return float(nct.sf(q, n - 1, math.sqrt(n) * effect / sd))


def power_table():
    rows = []
    for name, m, n, ratio in (("unpinned", 3, 8, .97),
                              ("pinned10/8", 10, 8, .975),
                              ("pinned6/4", 10, 6, .98)):
        f = lambda log_effect: placement_power(m, n, 1, math.exp(-log_effect)) - .8
        k = brentq(f, 0, 4)
        sds = (.005, .01, .02, .04)
        rows.append(dict(name=name, m=m, n=n, ratio=ratio, threshold=float(t.isf(.025/m,n-1)),
                         mde_percent=[100*(-math.expm1(-sd*k)) for sd in sds],
                         power=[placement_power(m,n,sd,ratio) for sd in sds],
                         sd_max_at_ratio=-math.log(ratio)/k))
    return rows


def schedule(k):
    names = list("ABCD"[:k])
    rounds = [sustain.order(names, r) for r in range(2*k)]
    flat = sum(rounds, [])
    positions = Counter((name, pos) for rnd in rounds for pos, name in enumerate(rnd))
    assert set(positions.values()) == {2}
    counts = Counter(a+b for a,b in zip(flat,flat[1:]))
    return dict(rounds=["".join(r) for r in rounds], predecessors=dict(sorted(counts.items())))


if __name__ == "__main__":
    checks = dict(
        ordinary_pass=fixture(lambda _: None),
        cutoff_exact_void=fixture(lambda r: straddle(r, .80, 1.20)),
        balanced_clock_excursions_pass=fixture(lambda r: straddle(r, .96, 1.04)),
        failed_prewarm_passes=fixture(bad_prewarm),
    )
    assert [checks[k]["rc"] for k in checks] == [0, 1, 0, 0]
    print(json.dumps(dict(checks=checks, power=power_table(),
                          order_4=schedule(4), order_3=schedule(3)), indent=2))
