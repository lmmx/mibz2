"""The target-gated noninferiority pilot's analysis (registration §2.8): one fixed, nonqualifying
R = 600 dispatch of all nine shards, labelled `pilot`, read for what the final design needs:
noise, dependence, order, trend and time, never for whether the candidate passes. It reports
no cell's ratio or bound: every series is centred on its own mean before anything is printed.

    python3 bench/tg_ni_pilot.py SHARDS_DIR --manifest MANIFEST.json --head SHA --base SHA \\
        --rustc "rustc 1.99.1 (...)" --corpus-text SHA256 --out PREFIX

**Admission:** the judge's own structural admission (`service_tg.structural`):
- the manifest's schema;
- nothing downloaded that GitHub didn't list;
- the run's whole history reconciled fail-closed;
- every file rehashed against the manifest;
- every shard's files, runner, read-back, corpus, rows and timing.

Two differences:
- the label must be `pilot`;
- the frozen values come from the command line, since the pilot precedes the freeze.

A setup time the API didn't record is a problem, not a zero.

**The analysis menu, fixed before the pilot runs.** For each of the 42 cells (wall and CPU
alike), with X_i = log(head / base), i = 0 … R − 1, in round order, centred:
- **Dependence:**
  - the marginal sd;
  - the autocorrelation at lags 1, 2, 5, 10, 30 and 60.
- **Batch means:** for m = 10, 20, 30 and 60 over all 600 rounds, and for m = 48 over the
  first 576 (twelve complete batches; the last 24 rounds are left out of that one diagnostic and
  kept in every other analysis):
  - the number of batches;
  - the long-run-variance estimate LRV_m = m · var(batch means) = n_m · SE_m², and its ratio to
    the marginal variance. The largest over this fixed menu is a sensitivity choice, not a
    confidence bound on the long-run variance, nor a guarantee of power;
  - the lag-1 correlation of the batch means.
- **Order:** the mean of X with the head run first minus with the base run first, and that
  difference's iid standard error, from the rows' order within each round.
- **Trend:**
  - the OLS slope of X on the round index, as the change over the whole run, with its iid t;
  - the CUSUM maximum |S_k| / (sd sqrt(R)) and where it falls (a change-point screen).
- **Time:** from the row timestamps (bench/tg_ni_stamp.py: CLOCK_REALTIME, to the
  millisecond, observation times of a 0.2 s poller), which must be complete, finite, ordered,
  within the measurement's markers and from a stamper that exited 0 (`service_tg.timing_problems`):
  - each round's duration (median, p95, max);
  - each batch partition's actual elapsed span (median, p95, max). That runs from the previous
    batch's last round's end (the first batch's from the stamper's start) to its own last round's
    end, settle and polling included. It is not m times a median round.
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
# The batch-length menu at R = 600, fixed before the pilot (registration §2.8): 10, 20, 30 and 60
# over all 600 rounds; 48 (the R960 candidate's batch length) over the first 576 rounds only,
# twelve complete batches, the last 24 rounds left out of that one diagnostic and kept in every
# other analysis. Each is (m, the rounds it uses)
BATCH_MENU = ((10, None), (20, None), (30, None), (48, 576), (60, None))


def acf(x, lag):
    n = len(x)
    if lag >= n:
        return None
    m = statistics.fmean(x)
    v = sum((a - m) ** 2 for a in x)
    return sum((x[i] - m) * (x[i + lag] - m) for i in range(n - lag)) / v if v else 0.0


def batches(x, m, n=None):
    """Batch means of length m over the first n rounds (all of them if n is None; n must be a
    multiple of m). The long-run-variance estimate is normalised as LRV_m = m * var(batch
    means) = n * SE_m^2, against the marginal variance (their ratio, 1 for iid rounds)"""
    n = len(x) if n is None else n
    if n > len(x) or n % m:
        raise ValueError(f"{n} rounds don't split into batches of {m} (of {len(x)})")
    y = x[:n]
    k = n // m
    means = [statistics.fmean(y[i * m:(i + 1) * m]) for i in range(k)]
    var_means = statistics.variance(means) if k > 1 else None
    lrv = m * var_means if var_means is not None else None
    marginal = statistics.variance(y)
    return {"m": m, "rounds": n, "batches": k, "lrv": lrv,
            "se": math.sqrt(lrv / n) if lrv is not None else None,
            "lrv_over_marginal": lrv / marginal if lrv is not None and marginal else None,
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


def round_ends(times, rows):
    """Each round's end: its last row's time (rows in file order, which is run order)"""
    ends = {}
    for i, r in enumerate(rows):
        ends[r["round"]] = max(ends.get(r["round"], -math.inf), times[i])
    return [ends[k] for k in sorted(ends)]


def spans(start, ends, m, n=None):
    """The actual elapsed span of each batch of m rounds over the first n: from the previous
    batch's last round's end (for the first batch, the stamper's start) to this batch's last
    round's end. It includes each run's settle and the poller's observation delay"""
    n = len(ends) if n is None else n
    out, prev = [], start
    for b in range(n // m):
        end = ends[(b + 1) * m - 1]
        out.append(end - prev)
        prev = end
    return out


def tail(xs):
    if not xs:
        return None
    s = sorted(xs)
    return {"median": statistics.median(s), "p95": s[min(len(s) - 1, math.ceil(0.95 * len(s)) - 1)],
            "max": s[-1], "n": len(s)}


def ts(s):
    return dt.datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp() if s else None


def analyse(root, man, head, base, rustc, corpus_text, rounds=tg.NI_ROUNDS, menu=BATCH_MENU):
    """The pilot's analysis, after the judge's own structural admission (`service_tg.structural`,
    label pilot: the manifest, unlisted artifacts, the history, every file rehashed, every
    shard's files, runner, read-back, corpus, rows and timing)"""
    reg = {"frozen": {"base": base, "ni_rustc": rustc, "corpus_text": corpus_text}, "bench": None}
    out = {"schema": "mibz2.tg_ni_pilot/2", "run": man.get("run"), "head": head, "valid": False,
           "problems": [], "cells": {}, "shards": {}}
    p = out["problems"]
    q, admitted = tg.structural(root, man, head, reg, rounds, "pilot", check_git=False)
    p += q
    jobs = man.get("jobs") or []
    for sid, (a, _) in admitted.items():
        d = os.path.join(root, a["name"], sid)
        rows, _ = tg.read_jsonl(os.path.join(d, "runs.jsonl"))
        start, times = tg.read_row_times(d)
        job = [j for j in tg.shard_jobs(jobs, sid) if j.get("run_attempt") == a["attempt"]][0]
        started = tg.read_text(d, "measurement-started.txt").strip()
        ended = tg.read_text(d, "measurement-ended.txt").strip()
        if not job.get("started_at"):
            p.append(f"{sid}: the job's start time is missing: setup unknown")
        model = next((l.split(":", 1)[1].strip() for l in tg.read_text(d, "lscpu.txt").splitlines()
                      if l.startswith("Model name:")), None)
        ends = round_ends(times, rows)
        durations = [b - a_ for a_, b in zip([start] + ends[:-1], ends)]
        out["shards"][sid] = {
            "model": model,
            "setup_s": ts(started) - ts(job["started_at"]) if job.get("started_at") else None,
            "measurement_s": ts(ended) - ts(started),
            "round_s": tail(durations),
            "batch_s": {str(m): tail(spans(start, ends, m, n)) for m, n in menu},
        }
        corpus, threads = tg.SHARDS[sid]
        for t in threads:
            for metric in tg.CI_METRICS:
                x, hf = cell_series(rows, corpus, t, metric)
                c = {"shard": sid, "sd": statistics.stdev(x),
                     "acf": {str(k): acf(x, k) for k in LAGS},
                     "batches": [batches(x, m, n) for m, n in menu],
                     "order": order_contrast(x, hf), "trend": trend(x), "cusum": cusum(x)}
                # (the series centred: no ratio or bound of the candidate's is kept)
                c["trend"] = {"t": c["trend"]["t"], "change_over_run": c["trend"]["change_over_run"]}
                out["cells"][f"{corpus}/{t}/{metric}"] = c
    out["valid"] = not p
    if p:
        out["cells"], out["shards"] = {}, {}
    return out


def markdown(v):
    o = ["# The target-gated noninferiority pilot (not evidence: no cell's ratio or bound is shown)", ""]
    if not v["valid"]:
        return "\n".join(o + ["**Not admitted:**"] + [f"- {x}" for x in v["problems"]]) + "\n"
    f = lambda z: "–" if z is None else f"{z:+.2f}"  # noqa: E731
    g = lambda z: "–" if z is None else f"{z:.2f}"  # noqa: E731
    o += ["LRV_m = m · var(batch means) = n_m · SE_m², shown over the marginal variance (1 for iid rounds). "
          "The largest over the fixed menu is a sensitivity choice, not a confidence bound on the long-run "
          "variance, nor a guarantee of any power.", "",
          "| cell | sd | acf 1 / 5 / 30 | LRV / marginal at m 10 / 30 / 48* / 60 | order diff / se | trend t | CUSUM |",
          "|---|--:|---|---|--:|--:|--:|"]
    for k, c in v["cells"].items():
        a = c["acf"]
        br = {b["m"]: b["lrv_over_marginal"] for b in c["batches"]}
        od = c["order"]
        order = "–" if not od else "%+.4f / %.4f" % (od["difference"], od["se"])
        o.append(f"| {k} | {c['sd']:.4f} | {f(a['1'])} / {f(a['5'])} / {f(a['30'])} | "
                 f"{g(br.get(10))} / {g(br.get(30))} / {g(br.get(48))} / {g(br.get(60))} | "
                 f"{order} | {f(c['trend']['t'])} | {g(c['cusum']['max'])} |")
    o += ["", "*m 48 over the first 576 rounds only (twelve batches; the last 24 rounds are in every other column).", "",
          "| shard | model | setup s | measurement s | round s median / p95 / max | batch s at m 30: median / p95 / max |",
          "|---|---|--:|--:|---|---|"]
    t3 = lambda r: "–" if not r else f"{r['median']:.2f} / {r['p95']:.2f} / {r['max']:.2f}"  # noqa: E731
    for sid, s in v["shards"].items():
        setup = "–" if s["setup_s"] is None else f"{s['setup_s']:.0f}"
        o.append(f"| {sid} | {s['model']} | {setup} | {s['measurement_s']:.0f} | {t3(s['round_s'])} | "
                 f"{t3(s['batch_s'].get('30'))} |")
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
