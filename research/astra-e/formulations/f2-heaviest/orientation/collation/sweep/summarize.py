"""Summarize the exact spectrum and test fixed inexpensive frozen-only rules."""
import argparse
from array import array
from collections import Counter
import csv
import hashlib
from itertools import accumulate
import json
import lzma
from pathlib import Path


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("csv_xz", type=Path)
    p.add_argument("column", type=Path)
    p.add_argument("mass", type=int)
    p.add_argument("named", type=int)
    p.add_argument("out", type=Path)
    a = p.parse_args()
    column = a.column.read_bytes()
    end = len(column) - a.mass
    ks, ds, qs = (array("I") for _ in range(3))
    by_bytes = {}
    with lzma.open(a.csv_xz, "rt") as f:
        rows = csv.reader(f)
        assert next(rows) == ["start", "words", "distinct_bytes", "proper_suffixes"]
        for i, row in enumerate(rows):
            start, k, d, q = map(int, row)
            assert start == i and 0 < k <= a.mass and 1 <= q <= d <= len(column)
            for col, val in zip((ks, ds, qs), (k, d, q)):
                col.append(val)
            prev = by_bytes.get(d)
            if prev is None or q < prev[0]:
                by_bytes[d] = (q, start, 1)
            elif q == prev[0]:
                by_bytes[d] = (q, prev[1], prev[2]+1)
    assert len(qs) == end+1
    def metrics(i):
        return dict(start=i, words=ks[i], distinct_bytes=ds[i], proper_suffixes=qs[i])

    ref_d, ref_q = ds[a.named], qs[a.named]
    best_q = len(column)+1
    frontier = []
    for d, (q,i,count) in sorted(by_bytes.items()):
        if q < best_q:
            best_q = q
            frontier.append(dict(**metrics(i), starts_at_pair=count))
    with (a.out/"pareto.csv").open("w") as f:
        w = csv.DictWriter(f, fieldnames=list(frontier[0]), lineterminator="\n")
        w.writeheader()
        w.writerows(frontier)
    runs = []
    for i in range(end+1):
        if ds[i] < ref_d and qs[i] < ref_q:
            if runs and i == runs[-1][1]:
                runs[-1][1] = i+1
            else:
                runs.append([i,i+1])
    count = sum(hi-lo for lo,hi in runs)
    # Exact integer ordering by max(D/D_ref,Q/Q_ref), then Q,D,start.
    def minmax(i):
        return max(ds[i]*ref_q, qs[i]*ref_d), qs[i], ds[i], i

    frontier_joint = [x for x in frontier if x["distinct_bytes"] < ref_d and x["proper_suffixes"] < ref_q]
    freq = Counter(column)
    base = list(accumulate((freq[c] for c in range(256)), initial=0))
    byte_edges = sorted({x for x in base if 0 <= x <= end}
                        | {x-a.mass for x in base if 0 <= x-a.mass <= end})
    dominated_edges = [i for i in byte_edges if ds[i] < ref_d and qs[i] < ref_q]
    r = array("I", [0])
    for i, c in enumerate(column):
        r.append(r[-1]+int(i > 0 and column[i-1] != c))
    def run_count(i):
        return 1+r[i+a.mass]-r[i+1]
    lo = min(range(end+1), key=run_count)
    hi = max(range(end+1), key=run_count)
    named_runs = run_count(a.named)
    equal_runs = [i for i in range(end+1) if run_count(i) == named_runs]
    candidates = {f"F_base_byte_{c}":v for c,v in enumerate(base[:-1])
                  if freq[c] and 0 <= v <= end}
    candidates.update({f"window_ending_at_F_base_byte_{c}":v-a.mass for c,v in enumerate(base[:-1])
                       if freq[c] and 0 <= v-a.mass <= end})
    rules = {
        "named":a.named, "min_window_L_run_count_earliest":lo, "max_window_L_run_count_earliest":hi,
        "colon_F_base":base[58], "colon_F_base_minus_mass_div_10":base[58]-a.mass//10,
        "best_minmax_over_all_F_edges_given_oracle_prices":min(byte_edges,key=minmax)
    }
    summary = dict(
        spectrum_xz_sha256=hashlib.sha256(a.csv_xz.read_bytes()).hexdigest(),
        n=len(column), mass=a.mass, windows=end+1, named=metrics(a.named),
        min_D_then_Q_then_start=frontier[0], min_Q_then_D_then_start=frontier[-1],
        min_max_normalized_D_Q_then_Q_D_start=metrics(min(range(end+1),key=minmax)),
        strict_improvements=count, strict_ranges_half_open=runs,
        pareto_pairs=len(frontier), starts_on_frontier=sum(x["starts_at_pair"] for x in frontier),
        jointly_better_pareto_points=frontier_joint,
        F_boundary_windows=len(byte_edges), F_boundary_joint_improvements=dominated_edges,
        F_boundary_rules_hitting_improvement={k:v for k,v in candidates.items() if v in dominated_edges},
        named_L_runs=named_runs, windows_with_same_L_runs=len(equal_runs),
        same_L_runs_min_D=min(ds[i] for i in equal_runs),same_L_runs_max_D=max(ds[i] for i in equal_runs),
        same_L_runs_min_Q=min(qs[i] for i in equal_runs), same_L_runs_max_Q=max(qs[i] for i in equal_runs),
        frozen_rules={name:{**metrics(i), "last_column_runs_in_window":run_count(i)}
                      for name,i in rules.items()})
    (a.out/"summary.json").write_text(json.dumps(summary, indent=2)+"\n")


if __name__ == "__main__":
    main()
