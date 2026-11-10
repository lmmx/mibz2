"""The target-gated noninferiority pilot's analysis (registration §2.8): one fixed, nonqualifying
R = 600 dispatch of all nine shards, labelled `pilot`, read for what the final design needs:
noise, dependence, order, trend and time, never for whether the candidate passes. It reports
no cell's ratio or bound: every series is centred on its own mean before anything is printed.

    python3 bench/tg_ni_pilot.py SHARDS_DIR --manifest MANIFEST.json --head SHA --base SHA \\
        --rustc "rustc 1.99.1 (...)" --corpus-text SHA256 --out PREFIX

**Admission:** the judge's own (bench/service_tg.py): the run's whole history reconciled
fail-closed, and every shard's files, runner, read-back, corpus and rows. Two differences:
- the label must be `pilot`;
- the frozen values come from the command line, since the pilot precedes the freeze.

**The analysis menu, fixed before the pilot runs.** For each of the 42 cells (wall and CPU
alike), with X_i = log(head / base), i = 0 … R − 1, in round order, centred:
- **Dependence:**
  - the marginal sd;
  - the autocorrelation at lags 1, 2, 5, 10, 30 and 60.
- **Batch means:** for each batch length m in 10, 20, 30, 48 and 60 that divides R:
  - the number of batches;
  - the batch-means standard error, and its ratio to the iid s/sqrt(R) (the long-run-variance
    sensitivity);
  - the lag-1 correlation of the batch means.
- **Order:** the mean of X with the head run first minus with the base run first, and that
  difference's iid standard error, from the rows' order within each round.
- **Trend:**
  - the OLS slope of X on the round index, as the change over the whole run, with its iid t;
  - the CUSUM maximum |S_k| / (sd sqrt(R)) and where it falls (a change-point screen).
- **Time:** from the row timestamps (bench/tg_ni_stamp.py):
  - each round's duration (median, p95, max);
  - each batch length's physical duration (median).
- **Per shard:**
  - setup (the job's start to the measurement's start);
  - measurement (start to end);
  - the runner's model.

Nothing here is a verdict. The pilot's numbers set the final design's R, batch length,
dependence model and any frozen veto, which are then frozen against fresh qualification data.
"""

import argparse
import datetime as dt
import json
import math
import os
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import service_tg as tg  # noqa: E402

LAGS = (1, 2, 5, 10, 30, 60)
BATCH_LENGTHS = (10, 20, 30, 48, 60)


def acf(x, lag):
    n = len(x)
    if lag >= n:
        return None
    m = statistics.fmean(x)
    v = sum((a - m) ** 2 for a in x)
    return sum((x[i] - m) * (x[i + lag] - m) for i in range(n - lag)) / v if v else 0.0


def batches(x, m):
    k = len(x) // m
    means = [statistics.fmean(x[i * m:(i + 1) * m]) for i in range(k)]
    se = statistics.stdev(means) / math.sqrt(k) if k > 1 else None
    iid = statistics.stdev(x) / math.sqrt(len(x))
    return {"m": m, "batches": k, "se": se, "se_over_iid": se / iid if se and iid else None,
            "lag1_of_means": acf(means, 1) if k > 2 else None}


def trend(x):
    n = len(x)
    t = list(range(n))
    tm, xm = statistics.fmean(t), statistics.fmean(x)
    sxx = sum((a - tm) ** 2 for a in t)
    b = sum((t[i] - tm) * (x[i] - xm) for i in range(n)) / sxx
    resid = [x[i] - xm - b * (t[i] - tm) for i in range(n)]
    s = math.sqrt(sum(r * r for r in resid) / (n - 2))
    se = s / math.sqrt(sxx)
    return {"change_over_run": math.exp(b * (n - 1)), "t": b / se if se else None}


def cusum(x):
    n, m, sd = len(x), statistics.fmean(x), statistics.stdev(x)
    s, best, at = 0.0, 0.0, 0
    for i, a in enumerate(x):
        s += a - m
        if abs(s) > best:
            best, at = abs(s), i
    return {"max": best / (sd * math.sqrt(n)) if sd else None, "at_round": at}


def cell_series(rows, corpus, t, metric):
    """X in round order, and for each round whether the head ran first"""
    by = {}
    for i, r in enumerate(rows):
        if r["corpus"] == corpus and r["threads"] == t:
            by.setdefault(r["round"], []).append((i, r))
    x, head_first = [], []
    for rnd in sorted(by):
        pair = sorted(by[rnd])
        arms = {r["arm"]: r for _, r in pair}
        h, b = arms[tg.CI_ARMS[0]], arms[tg.CI_ARMS[1]]
        x.append(math.log(h[metric] / b[metric]))
        head_first.append(pair[0][1]["arm"] == tg.CI_ARMS[0])
    return x, head_first


def order_contrast(x, head_first):
    a = [v for v, h in zip(x, head_first) if h]
    b = [v for v, h in zip(x, head_first) if not h]
    if len(a) < 2 or len(b) < 2:
        return None
    d = statistics.fmean(a) - statistics.fmean(b)
    se = math.sqrt(statistics.variance(a) / len(a) + statistics.variance(b) / len(b))
    return {"difference": d, "se": se}


def round_times(times, rows):
    """Each round's duration from its last row's time to the previous round's"""
    ends = {}
    for i, r in enumerate(rows):
        if i in times:
            ends[r["round"]] = max(ends.get(r["round"], 0.0), times[i])
    rs = sorted(ends)
    return [ends[rs[k]] - ends[rs[k - 1]] for k in range(1, len(rs))]


def read_times(path):
    out = {}
    try:
        with open(path) as f:
            for line in f:
                if line.startswith("#"):
                    out = {}
                    continue
                i, t = line.split()
                out[int(i)] = float(t)
    except (OSError, ValueError):
        return {}
    return out


def ts(s):
    return dt.datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp() if s else None


def analyse(root, man, head, base, rustc, corpus_text, rounds=tg.NI_ROUNDS):
    reg = {"frozen": {"base": base, "ni_rustc": rustc, "corpus_text": corpus_text}, "bench": None}
    out = {"schema": "mibz2.tg_ni_pilot/1", "run": man.get("run"), "head": head, "valid": False,
           "problems": [], "cells": {}, "shards": {}}
    p = out["problems"]
    if man.get("label") != "pilot":
        p.append(f"labelled {man.get('label')!r}, not pilot")
    if man.get("schema") != "mibz2.tg_ni_manifest/2":
        p.append("the manifest records no history")
    jobs = man.get("jobs") or []
    for sid in tg.SHARDS:
        q, a = tg.history_problems(sid, jobs, [x for x in man.get("artifacts", []) if x["shard"] == sid],
                                   head=head, run=man.get("run"))
        if q:
            p += q
            continue
        d = os.path.join(root, a["name"], sid)
        q, _ = tg.shard_problems(sid, d, reg, head, man.get("run"), rounds, attempt=a["attempt"], label="pilot")
        if q:
            p += q
            continue
        rows, _ = tg.read_jsonl(os.path.join(d, "runs.jsonl"))
        times = read_times(os.path.join(d, "row-times.txt"))
        job = [j for j in tg.shard_jobs(jobs, sid) if j.get("run_attempt") == a["attempt"]][0]
        started = tg.read_text(d, "measurement-started.txt").strip()
        ended = tg.read_text(d, "measurement-ended.txt").strip()
        model = next((l.split(":", 1)[1].strip() for l in tg.read_text(d, "lscpu.txt").splitlines()
                      if l.startswith("Model name:")), None)
        rt = round_times(times, rows)
        out["shards"][sid] = {
            "model": model,
            "setup_s": ts(started) - ts(job.get("started_at")) if job.get("started_at") else None,
            "measurement_s": ts(ended) - ts(started),
            "round_s": {"median": statistics.median(rt), "p95": sorted(rt)[int(0.95 * (len(rt) - 1))],
                        "max": max(rt)} if rt else None,
            "timestamps": len(times),
        }
        corpus, threads = tg.SHARDS[sid]
        for t in threads:
            for metric in tg.CI_METRICS:
                x, hf = cell_series(rows, corpus, t, metric)
                c = {"shard": sid, "sd": statistics.stdev(x),
                     "acf": {str(k): acf(x, k) for k in LAGS},
                     "batches": [batches(x, m) for m in BATCH_LENGTHS if len(x) % m == 0 and m % 2 == 0],
                     "order": order_contrast(x, hf), "trend": trend(x), "cusum": cusum(x)}
                if rt:
                    med = statistics.median(rt)
                    c["batch_seconds"] = {str(m): m * med for m in BATCH_LENGTHS}
                # (the series centred: no ratio or bound of the candidate's is kept)
                c["trend"] = {"t": c["trend"]["t"], "change_over_run": c["trend"]["change_over_run"]}
                out["cells"][f"{corpus}/{t}/{metric}"] = c
    out["valid"] = not p
    return out


def markdown(v):
    o = ["# The target-gated noninferiority pilot (not evidence: no cell's ratio or bound is shown)", ""]
    if not v["valid"]:
        return "\n".join(o + ["**Not admitted:**"] + [f"- {x}" for x in v["problems"]]) + "\n"
    o += ["| cell | sd | acf 1 / 5 / 30 | SE ratio at m 10 / 30 / 60 | order diff / se | trend t | CUSUM |",
          "|---|--:|---|---|--:|--:|--:|"]
    for k, c in v["cells"].items():
        a = c["acf"]
        br = {b["m"]: b["se_over_iid"] for b in c["batches"]}
        f = lambda z: "–" if z is None else f"{z:+.2f}"  # noqa: E731
        g = lambda z: "–" if z is None else f"{z:.2f}"  # noqa: E731
        od = c["order"]
        order = "–" if not od else "%+.4f / %.4f" % (od["difference"], od["se"])
        o.append(f"| {k} | {c['sd']:.4f} | {f(a['1'])} / {f(a['5'])} / {f(a['30'])} | "
                 f"{g(br.get(10))} / {g(br.get(30))} / {g(br.get(60))} | "
                 f"{order} | {f(c['trend']['t'])} | {g(c['cusum']['max'])} |")
    o += ["", "| shard | model | setup s | measurement s | round s median / p95 / max |", "|---|---|--:|--:|---|"]
    for sid, s in v["shards"].items():
        r = s["round_s"] or {}
        o.append(f"| {sid} | {s['model']} | {s['setup_s'] or 0:.0f} | {s['measurement_s']:.0f} | "
                 f"{r.get('median', 0):.2f} / {r.get('p95', 0):.2f} / {r.get('max', 0):.2f} |")
    return "\n".join(o) + "\n"


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("root")
    p.add_argument("--manifest", required=True)
    p.add_argument("--head", required=True)
    p.add_argument("--base", required=True)
    p.add_argument("--rustc", required=True)
    p.add_argument("--corpus-text", required=True)
    p.add_argument("--out", required=True)
    a = p.parse_args(argv)
    with open(a.manifest) as f:
        man = json.load(f)
    v = analyse(a.root, man, a.head, a.base, a.rustc, a.corpus_text)
    tg.write(a.out, v, markdown(v))
    return 0 if v["valid"] else 2


if __name__ == "__main__":
    sys.exit(main())
