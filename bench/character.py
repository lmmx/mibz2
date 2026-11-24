#!/usr/bin/env python3
"""bench-character's report (stage 1; docs/journal/2026-11-24-character-stage1-registration.md).

A diagnostic: it decides nothing. It reads the driver's runs.jsonl (bench/character_run.py),
the block rows and the features, and reports the decoder's character across 1 to 20 workers:
- at each count: the wall, throughput, CPU, memory, faults, switches and migrations; each
  activity's demand and each wait (start-up, steady and drain apart); the buffers' occupancy;
- the operational laws, checked in every decode (Little's law per buffer, utilisation, forced
  flow, conservation, admission, each thread's timeline partitioned);
- k*, the first count whose median starvation gap is positive, with its round-bootstrap
  interval, and the near-optimal set (within 2% of the best median wall);
- each count's limit, by the registered rule (SCANNER-, WORKER-, ADMISSION-LIMITED or MIXED);
- the ledgers' cost (paired on and off at 1, 10 and 20);
- the counters at the anchors, the memory controller where counted, the public API's sweep;
- each route's blocks: their structure and their demand per row at each count.

It writes the TSVs for the journal's math-targets project into --tsv-dir.

    python3 bench/character.py RUNS.jsonl --lscpu LSCPU --features FEATURES.tsv \
        --blocks-dir DIR --tsv-dir DIR

VOID (no report but the reasons) if the machine is not the i9-10900K, a registered decode is
missing, duplicated or failed, an output is not 4 x 209,700,196 bytes, a window is not 4k, or a
decode lacks its memory and scheduling figures. INCOMPLETE (reported, labelled) if any grid
decode's identities fail. PERTURBED (reported, labelled) if the ledgers' cost passes the
registered bound at any of 1, 10 and 20.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
import re
import statistics
import sys

ACT = [
    "A0", "feed_wait", "in_wait", "in_maint", "A2", "A3", "slot_wait", "issue", "job_wait",
    "A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish", "A10", "read_wait", "resid",
]  # fmt: skip
WAITS = {"feed_wait", "in_wait", "slot_wait", "job_wait", "read_wait"}
WORKER_ACT = ["A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish", "resid"]
SCANNER_ACT = ["in_maint", "A2", "A3", "issue", "resid"]
BUFFERS = ["admitted", "queued", "executing", "ready", "ready_bytes"]
GRID = list(range(1, 21))
COST_COUNTS = [1, 10, 20]
ANCHORS = [1, 6, 8, 10, 12, 20]
OUT_BYTES = 4 * 209_700_196
PROC = ["wall_s", "bytes", "base_kb", "peak_kb", "mean_kb", "rss_kb_s", "user_s", "sys_s",
        "minflt", "majflt", "nvcsw", "nivcsw"]  # fmt: skip
ROUTES = ["open", "open_dense", "refused", "skipped", "column"]
ROUTE_CODE = {1: "open", 2: "open_dense", 3: "refused", 4: "skipped", 5: "column"}
# the registered rule's thresholds
COST_POINT, COST_UPPER = 1.01, 1.005
NEAR = 1.02
BOOT = 2000


def tcrit(df: int) -> float:
    table = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306,
             9: 2.262, 10: 2.228, 11: 2.201, 12: 2.179, 15: 2.131, 20: 2.086, 30: 2.042}  # fmt: skip
    return table.get(df, 1.96 if df > 30 else table[min(table, key=lambda d: abs(d - df))])


def load(path: str) -> list[dict]:
    return [json.loads(line) for line in open(path) if line.strip()]


def role_totals(d: dict, phase: int | None = None) -> dict[str, dict[str, float]]:
    """{role: {activity: seconds}} summed over the role's threads (all phases, or one)"""
    out: dict[str, dict[str, float]] = {}
    for t in d["threads"]:
        r = out.setdefault(t["role"], {a: 0.0 for a in ACT})
        for i, a in enumerate(ACT):
            v = sum(p[i] for p in t["states"]) if phase is None else t["states"][phase][i]
            r[a] += v / 1e9
    return out


def derived(d: dict) -> dict:
    """A ledger decode's figures"""
    k = d["k"]
    rt = role_totals(d)
    sc, wk = rt.get("Scanner", {}), rt.get("Worker", {})
    rd, fd = rt.get("Reader", {}), rt.get("Feeder", {})
    s_busy = sum(sc.get(a, 0) for a in SCANNER_ACT)
    w_busy = sum(wk.get(a, 0) for a in WORKER_ACT)
    x = {
        "k": k,
        "wall": d["wall_s"],
        "scanner_busy": s_busy,
        "worker_busy_mean": w_busy / k,
        "gap": s_busy - w_busy / k,
        "starve_mean": wk.get("job_wait", 0) / k,
        "slot_wait": sc.get("slot_wait", 0),
        "in_wait": sc.get("in_wait", 0),
        "read_wait": rd.get("read_wait", 0),
        "reader_active": rd.get("A10", 0) - rd.get("read_wait", 0),
        "feeder_A0": fd.get("A0", 0),
        "feed_wait": fd.get("feed_wait", 0),
    }
    for a in ACT:
        x[f"scanner_{a}"] = sc.get(a, 0)
        x[f"worker_{a}"] = wk.get(a, 0)
    sb = [0, 0]
    for t in d["threads"]:
        if t["role"] == "Scanner":
            sb = t.get("slot_bound", [0, 0])
    x["slot_window"], x["slot_cap"] = sb[0] / 1e9, sb[1] / 1e9
    # the steady phase's shares
    steady = max(d["t2_ns"] - d["t1_ns"], 0) / 1e9
    st = role_totals(d, 1)
    ssc, swk = st.get("Scanner", {}), st.get("Worker", {})
    if steady > 0:
        x["u_scanner"] = sum(ssc.get(a, 0) for a in SCANNER_ACT) / steady
        x["slot_share"] = ssc.get("slot_wait", 0) / steady
        x["u_worker"] = sum(swk.get(a, 0) for a in WORKER_ACT) / k / steady
        x["starve_share"] = swk.get("job_wait", 0) / k / steady
    else:
        x["u_scanner"] = x["slot_share"] = x["u_worker"] = x["starve_share"] = float("nan")
    x["steady"] = steady
    x["startup"] = d["t1_ns"] / 1e9
    x["drain"] = max(d["end_ns"] - d["t2_ns"], 0) / 1e9
    end = d["end_ns"] / 1e9 or d["wall_s"]
    for b in BUFFERS:
        o = d["occupancy"][b]
        x[f"occ_{b}"] = o["integral"] / 1e9 / end
        x[f"hwm_{b}"] = o["hwm"]
    x["migrations"] = sum(t["migrations"] or 0 for t in d["threads"])
    x["throughput"] = d["bytes"] / d["wall_s"] / 1e6
    x["blocks"] = d["blocks"]
    for r in ROUTES:
        x[f"route_{r}"] = d["routes"][r]
    return x


def limit(m: dict) -> str:
    """The registered rule, on the steady phase's medians"""
    if m["u_scanner"] >= 0.85 and m["slot_share"] <= 0.05:
        return "SCANNER-LIMITED"
    if m["u_worker"] >= 0.85 and m["starve_share"] <= 0.05:
        return "WORKER-LIMITED"
    if m["slot_share"] > 0.10 and m["u_worker"] < 0.85:
        return "ADMISSION-LIMITED"
    return "MIXED"


def cost_flag(gm: float, lo: float) -> bool:
    """The ledgers' cost past the registered bound: the paired on/off wall ratio's geometric mean
    above 1.01, or its 95% interval wholly above 1.005"""
    return gm > COST_POINT or lo > COST_UPPER


def kstar(gaps: dict[int, list[float]]) -> int | None:
    for k in sorted(gaps):
        if gaps[k] and statistics.median(gaps[k]) > 0:
            return k
    return None


def validate(recs: list[dict], a, lscpu: str) -> tuple[list[str], list[str]]:
    void, incomplete = [], []
    model = re.findall(r"Model name:\s*(.+)", lscpu)
    if not any("i9-10900K" in m for m in model):
        void.append(f"lscpu: model {model}, not the i9-10900K")
    for x in recs:
        if x.get("kind") == "failed":
            void.append(
                f"a decode failed: {x.get('section')} {x.get('cmd')} status {x.get('status')}"
            )
    dec = [x for x in recs if x.get("kind") in ("decode", "api")]
    for x in dec:
        if x.get("bytes") != a.out_bytes:
            void.append(
                f"{x.get('section')} k {x.get('k', x.get('requested'))}: {x.get('bytes')} bytes"
            )
        miss = [f for f in PROC if f not in x]
        if miss:
            void.append(f"{x.get('section')}: no {miss}")
    grid = [x for x in dec if x.get("section") == "grid"]
    seen: dict = {}
    for x in grid:
        key = (x["round"], x["k"], x["rep"])
        seen[key] = seen.get(key, 0) + 1
        if x.get("window") != 4 * x["k"]:
            void.append(f"grid k {x['k']}: window {x.get('window')}")
        if x.get("on") != 1:
            void.append(f"grid k {x['k']}: the ledgers off")
    for r in range(a.rounds):
        for k in GRID:
            for rep in range(a.reps):
                n = seen.get((r, k, rep), 0)
                if n != 1:
                    void.append(f"grid round {r} k {k} rep {rep}: {n} decodes")
        orders = sorted(x["order"] for x in grid if x["round"] == r and x["rep"] == 0)
        if orders != list(range(20)):
            void.append(f"grid round {r}: its order is not a permutation of the 20 counts")
    cost = [x for x in dec if x.get("section") == "cost"]
    pairs = a.rounds // 2
    for p in range(pairs):
        for k in COST_COUNTS:
            for arm in (0, 1):
                n = sum(1 for x in cost if x["pair"] == p and x["k"] == k and x["arm"] == arm)
                if n != a.reps:
                    void.append(f"cost pair {p} k {k} arm {arm}: {n} decodes")
    api = [x for x in dec if x.get("section") == "api"]
    for r in range(a.api_rounds):
        got = sorted(x["requested"] for x in api if x["round"] == r)
        if got != GRID:
            void.append(f"api round {r}: requested {got}")
    for x in grid:
        if not x.get("identities_ok"):
            bad = [c["name"] for c in x.get("identities", []) if not c["ok"]]
            incomplete.append(f"grid round {x['round']} k {x['k']} rep {x['rep']}: {bad}")
    return void, incomplete


def med(xs):
    xs = [x for x in xs if x is not None and not (isinstance(x, float) and math.isnan(x))]
    return statistics.median(xs) if xs else float("nan")


def write_tsv(path: str, cols: list[str], rows: list[dict]) -> None:
    with open(path, "w") as f:
        f.write("\t".join(cols) + "\n")
        for r in rows:
            f.write("\t".join(fmt(r.get(c, "")) for c in cols) + "\n")


def fmt(v) -> str:
    if isinstance(v, float):
        return "" if math.isnan(v) else f"{v:.6g}"
    return str(v)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--features", required=True)
    ap.add_argument("--blocks-dir", required=True)
    ap.add_argument("--tsv-dir", required=True)
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--api-rounds", type=int, default=3)
    ap.add_argument("--out-bytes", type=int, default=OUT_BYTES)
    ap.add_argument("--seed", type=int, default=20261124)
    a = ap.parse_args()
    recs = load(a.runs)
    void, incomplete = validate(recs, a, open(a.lscpu).read())
    if void:
        print(f"VOID: the records do not make the registered run ({len(void)} problems); no report")
        for v in void[:40]:
            print(f"  {v}")
        return 1
    report(recs, a, incomplete)
    return 0


def report(recs: list[dict], a, incomplete: list[str]) -> None:
    os.makedirs(a.tsv_dir, exist_ok=True)
    grid = [x for x in recs if x.get("kind") == "decode" and x.get("section") == "grid"]
    cost = [x for x in recs if x.get("kind") == "decode" and x.get("section") == "cost"]
    pmu = [x for x in recs if x.get("kind") == "decode" and x.get("section") == "pmu"]
    imc = [x for x in recs if x.get("kind") == "imc"]
    api = [x for x in recs if x.get("kind") == "api"]
    notes = [x for x in recs if x.get("kind") == "note"]
    print("# bench-character, stage 1 (diagnostic: decides nothing)\n")
    print(
        f"{len(grid)} grid decodes ({a.rounds} rounds x 20 counts x {a.reps}), {len(cost)} cost-check decodes, {len(pmu)} counter decodes, {len(imc)} memory-controller decodes, {len(api)} public-API decodes"
    )
    for n in notes:
        print(f"note ({n['section']}): {n['note']}")
    if incomplete:
        print(
            f"\nINCOMPLETE: {len(incomplete)} grid decodes fail an identity; their figures are reported, labelled"
        )
        for s in incomplete[:20]:
            print(f"  {s}")
    # ---- observations ----
    obs: dict[tuple[int, int], list[dict]] = {}
    rows = []
    for x in grid:
        dv = derived(x)
        obs.setdefault((x["round"], x["k"]), []).append(dv)
        row = {
            c: x.get(c)
            for c in [
                "section",
                "round",
                "order",
                "rep",
                "k",
                "on",
                "window",
                *PROC,
                "blocks",
                "identities_ok",
                "t1_ns",
                "t2_ns",
                "end_ns",
            ]
        }
        row.update(dv)
        rows.append(row)
    for x in cost:
        row = {
            c: x.get(c)
            for c in ["section", "round", "pair", "rep", "arm", "k", "on", "window", *PROC]
        }
        rows.append(row)
    dcols = list(derived(grid[0]).keys())
    write_tsv(
        os.path.join(a.tsv_dir, "observations.tsv"),
        [
            "section",
            "round",
            "order",
            "pair",
            "rep",
            "arm",
            "k",
            "on",
            "window",
            *PROC,
            "blocks",
            "identities_ok",
            "t1_ns",
            "t2_ns",
            "end_ns",
        ]
        + [c for c in dcols if c not in ("k", "blocks")],
        rows,
    )
    trows = []
    for x in grid:
        for t in x["threads"]:
            tr = {"round": x["round"], "k": x["k"], "rep": x["rep"], "name": t["name"], "role": t["role"],
                  "life_ns": t["life_ns"], "cpu_ns": t["cpu_ns"], "wait_wall_ns": t["wait_wall_ns"],
                  "wait_cpu_ns": t["wait_cpu_ns"], "events": t["events"], "migrations": t["migrations"]}  # fmt: skip
            for i, s in enumerate(
                [
                    "sched_run0",
                    "sched_wait0",
                    "sched_slices0",
                    "sched_run1",
                    "sched_wait1",
                    "sched_slices1",
                ]
            ):
                tr[s] = t["sched"][i]
            for p, ph in enumerate(["startup", "steady", "drain"]):
                for i, act in enumerate(ACT):
                    tr[f"{ph}_{act}"] = t["states"][p][i]
            trows.append(tr)
    write_tsv(os.path.join(a.tsv_dir, "threads.tsv"), list(trows[0].keys()), trows)
    # per observation: the reps' mean; per count: the rounds' median
    per_obs: dict[int, list[dict]] = {k: [] for k in GRID}
    for (r, k), xs in obs.items():
        m = {
            c: statistics.fmean(x[c] for x in xs)
            for c in xs[0]
            if isinstance(xs[0][c], (int, float))
        }
        m["round"] = r
        per_obs[k].append(m)
    procs: dict[int, list[dict]] = {k: [] for k in GRID}
    for x in grid:
        procs[x["k"]].append(x)
    summary = []
    for k in GRID:
        o = per_obs[k]
        s = {"k": k}
        for c in o[0]:
            if c not in ("round", "k"):
                s[c] = med(x[c] for x in o)
        walls = [x["wall"] for x in o]
        s["wall_hw"] = (
            tcrit(len(walls) - 1) * statistics.stdev(walls) / math.sqrt(len(walls))
            if len(walls) > 1
            else float("nan")
        )
        p = procs[k]
        s["cpu_s"] = med(x["user_s"] + x["sys_s"] for x in p)
        s["own_peak_mb"] = med((x["peak_kb"] - x["base_kb"]) / 1024 for x in p)
        s["own_mean_mb"] = med((x["mean_kb"] - x["base_kb"]) / 1024 for x in p)
        s["mb_s_rss"] = med((x["mean_kb"] - x["base_kb"]) / 1024 * x["wall_s"] for x in p)
        for c in ["minflt", "majflt", "nvcsw", "nivcsw"]:
            s[c] = med(x[c] for x in p)
        s["limit"] = limit(s)
        summary.append(s)
    best = min(s["wall"] for s in summary)
    near = [s["k"] for s in summary if s["wall"] <= NEAR * best]
    print("\n## The grid (medians over rounds of each observation's mean over its decodes)\n")
    print(
        "k   wall s  ±95%    MB/s   cpu s  peak MB mean MB  faults  vol.cs  inv.cs  migr.   limit"
    )
    for s in summary:
        print(
            f"{s['k']:<3}{s['wall']:>7.4f}{s['wall_hw']:>7.4f}{s['throughput']:>8.0f}{s['cpu_s']:>8.3f}{s['own_peak_mb']:>9.1f}{s['own_mean_mb']:>8.1f}{s['minflt']:>8.0f}{s['nvcsw']:>8.0f}{s['nivcsw']:>8.0f}{s['migrations']:>7.0f}   {s['limit']}"
        )
    print("\n## Where the time goes (seconds a decode; scanner, the mean worker, reader, feeder)\n")
    print(
        "k   scan busy  A2     A3     maint  issue  in_wait slot(win/cap)   wkr busy starve  A5     A6a    A6b    A6c    A7     A8     A9     pool+pub  rd_wait  A0"
    )
    for s in summary:
        k = s["k"]
        print(
            f"{k:<3}{s['scanner_busy']:>9.4f}{s['scanner_A2']:>7.4f}{s['scanner_A3']:>7.4f}{s['scanner_in_maint']:>7.4f}{s['scanner_issue']:>7.4f}{s['in_wait']:>8.4f} {s['slot_window']:>7.4f}/{s['slot_cap']:<7.4f}"
            f"{s['worker_busy_mean']:>9.4f}{s['starve_mean']:>7.4f}"
            + "".join(
                f"{s['worker_' + c] / k:>7.4f}"
                for c in ["A5", "A6a", "A6b", "A6c", "A7", "A8", "A9"]
            )
            + f"{(s['worker_pool'] + s['worker_publish']) / k:>9.4f}{s['read_wait']:>9.4f}{s['feeder_A0']:>7.4f}"
        )
    print("\n## Phases and the steady state's shares\n")
    print("k   startup s steady s drain s   u_scan  slot%   u_wkr  starve%")
    for s in summary:
        print(
            f"{s['k']:<3}{s['startup']:>9.4f}{s['steady']:>9.4f}{s['drain']:>8.4f}{s['u_scanner']:>9.3f}{100 * s['slot_share']:>7.1f}{s['u_worker']:>8.3f}{100 * s['starve_share']:>8.1f}"
        )
    print("\n## The buffers (time-averaged occupancy, and the high-water mark)\n")
    print("k   admitted       queued         executing      ready          ready MB")
    for s in summary:
        print(
            f"{s['k']:<3}"
            + "".join(f"{s['occ_' + b]:>7.2f}/{s['hwm_' + b]:<6.0f}" for b in BUFFERS[:4])
            + f"{s['occ_ready_bytes'] / 2**20:>7.2f}/{s['hwm_ready_bytes'] / 2**20:<6.1f}"
        )
    # ---- the laws ----
    print("\n## The operational laws, in every grid decode\n")
    names: dict[str, list[int]] = {}
    for x in grid:
        for c in x["identities"]:
            v = names.setdefault(c["name"], [0, 0])
            v[0 if c["ok"] else 1] += 1
    for n, (ok, bad) in names.items():
        print(f"- {n}: {ok} hold, {bad} fail")
    print(
        "\nThe utilisation and bottleneck laws, from the medians (X blocks a second; D a block's demand):"
    )
    print("k   X        D_scan ms  D_wkr/k ms  1/max(D) blocks/s  U_scan  U_wkr")
    for s in summary:
        X = s["blocks"] / s["wall"]
        ds = s["scanner_busy"] / s["blocks"]
        dw = s["worker_busy_mean"] * s["k"] / s["blocks"] / s["k"]
        print(
            f"{s['k']:<3}{X:>8.1f}{1e3 * ds:>11.4f}{1e3 * dw:>12.4f}{1 / max(ds, dw):>19.1f}{X * ds:>8.3f}{X * dw:>7.3f}"
        )
    demands(grid, a)
    # ---- k* ----
    gaps = {k: [x["gap"] for x in per_obs[k]] for k in GRID}
    ks = kstar(gaps)
    rnd = random.Random(a.seed)
    rounds = sorted({x["round"] for xs in per_obs.values() for x in xs})
    boot = []
    by_round = {(x["round"], k): x["gap"] for k in GRID for x in per_obs[k]}
    for _ in range(BOOT):
        pick = [rnd.choice(rounds) for _ in rounds]
        g = {k: [by_round[(r, k)] for r in pick] for k in GRID}
        boot.append(kstar(g))
    found = sorted(b for b in boot if b is not None)
    none = sum(1 for b in boot if b is None)
    print("\n## k* and the near-optimal set\n")
    print(
        "the starvation gap Δ_k (the scanner's busy time less the mean worker's), medians over rounds, s:"
    )
    print("  " + "  ".join(f"{k}:{statistics.median(gaps[k]):+.4f}" for k in GRID))
    if ks is None:
        print("k*: none in 1..20 (the median gap is never positive)")
    else:
        lo = found[int(0.025 * len(found))] if found else None
        hi = found[min(len(found) - 1, int(0.975 * len(found)))] if found else None
        print(
            f"k* = {ks}: the first count whose median gap is positive; round-bootstrap 95% interval [{lo}, {hi}] ({BOOT} resamples, {none} with none)"
        )
    print(f"the near-optimal set (median wall within 2% of the best, {best:.4f} s): {near}")
    print("each count's limit: " + ", ".join(f"{s['k']}:{s['limit']}" for s in summary))
    sw = [(s["k"], s["limit"]) for s in summary]
    moves = [
        f"{sw[i - 1][1]} -> {sw[i][1]} at {sw[i][0]}"
        for i in range(1, len(sw))
        if sw[i][1] != sw[i - 1][1]
    ]
    print("the limit's moves: " + ("; ".join(moves) if moves else "none"))
    write_tsv(os.path.join(a.tsv_dir, "per_k.tsv"), list(summary[0].keys()), summary)
    # ---- the cost ----
    print("\n## The ledgers' cost (pairs on/off, each arm's mean wall over its decodes)\n")
    perturbed = []
    crow = []
    for k in COST_COUNTS:
        ratios = []
        for p in sorted({x["pair"] for x in cost}):
            on = [x["wall_s"] for x in cost if x["pair"] == p and x["k"] == k and x["arm"] == 1]
            off = [x["wall_s"] for x in cost if x["pair"] == p and x["k"] == k and x["arm"] == 0]
            if on and off:
                ratios.append(statistics.fmean(on) / statistics.fmean(off))
                crow.append(
                    {
                        "k": k,
                        "pair": p,
                        "on_s": statistics.fmean(on),
                        "off_s": statistics.fmean(off),
                        "ratio": ratios[-1],
                    }
                )
        lg = [math.log(r) for r in ratios]
        gm = math.exp(statistics.fmean(lg))
        hw = (
            tcrit(len(lg) - 1) * statistics.stdev(lg) / math.sqrt(len(lg))
            if len(lg) > 1
            else float("nan")
        )
        lo, hi = math.exp(statistics.fmean(lg) - hw), math.exp(statistics.fmean(lg) + hw)
        flag = cost_flag(gm, lo)
        if flag:
            perturbed.append(k)
        print(
            f"k {k:<3} on/off {gm:.4f} [{lo:.4f}, {hi:.4f}] over {len(ratios)} pairs{'  PERTURBED' if flag else ''}"
        )
    write_tsv(os.path.join(a.tsv_dir, "cost.tsv"), ["k", "pair", "on_s", "off_s", "ratio"], crow)
    print(
        "PERTURBED at " + str(perturbed)
        if perturbed
        else "the ledgers' cost is within the bound at 1, 10 and 20"
    )
    # ---- the counters ----
    print(
        "\n## The counters at the anchors (medians; coverage the least time running over enabled)\n"
    )
    prow = []
    if not pmu or not any(v for x in pmu for v in x.get("pmu", {}).values()):
        print("none counted")
    else:
        for k in ANCHORS:
            for g in (1, 2):
                xs = [x for x in pmu if x["k"] == k and x.get("pmu_group") == g]
                if not xs:
                    continue
                for ev in xs[0]["pmu"]:
                    vals = [x["pmu"][ev] for x in xs if x["pmu"].get(ev)]
                    if not vals:
                        continue
                    scaled = [v[0] * v[1] / v[2] if v[2] else float("nan") for v in vals]
                    cov = min(v[2] / v[1] if v[1] else 0 for v in vals)
                    prow.append(
                        {
                            "k": k,
                            "group": g,
                            "event": ev,
                            "median": med(scaled),
                            "coverage": cov,
                            "n": len(vals),
                            "scope": xs[0].get("pmu_scope"),
                        }
                    )
        for k in ANCHORS:
            r = {p["event"]: p for p in prow if p["k"] == k}
            if "cycles" in r and "instructions" in r:
                ipc = r["instructions"]["median"] / r["cycles"]["median"]
                print(
                    f"k {k:<3} IPC {ipc:.3f}; "
                    + "; ".join(
                        f"{e} {p['median']:.4g} ({p['coverage']:.2f})" for e, p in r.items()
                    )
                )
    write_tsv(
        os.path.join(a.tsv_dir, "pmu.tsv"),
        ["k", "group", "event", "median", "coverage", "n", "scope"],
        prow,
    )
    if imc:
        print("\nThe memory controller (perf stat -a, the whole decode's process; medians):")
        irow = []
        for k in ANCHORS:
            xs = [x for x in imc if x["k"] == k]
            for ev in sorted({e for x in xs for e in x["counts"]}):
                v = med(x["counts"][ev][0] for x in xs if ev in x["counts"])
                unit = next((x["counts"][ev][1] for x in xs if ev in x["counts"]), "")
                irow.append({"k": k, "event": ev, "median": v, "unit": unit})
                print(f"k {k:<3} {ev} {v:.1f} {unit}")
        write_tsv(os.path.join(a.tsv_dir, "imc.tsv"), ["k", "event", "median", "unit"], irow)
    # ---- the public API ----
    print("\n## The public API (BzReader::parallel as shipped; medians over its rounds)\n")
    print("asked  threads window lazy  service  wall s   peak MB")
    arow = []
    for t in GRID:
        xs = [x for x in api if x["requested"] == t]
        r = {"requested": t, "threads": xs[0]["threads"], "window": xs[0]["window"], "lazy": xs[0]["lazy"], "service": xs[0]["service"],
             "wall_s": med(x["wall_s"] for x in xs), "own_peak_mb": med((x["peak_kb"] - x["base_kb"]) / 1024 for x in xs)}  # fmt: skip
        arow.append(r)
        print(
            f"{t:<7}{r['threads']:<8}{r['window']:<7}{str(r['lazy']):<6}{str(r['service']):<9}{r['wall_s']:<9.4f}{r['own_peak_mb']:.1f}"
        )
    write_tsv(os.path.join(a.tsv_dir, "api.tsv"), list(arow[0].keys()), arow)
    # ---- the routes ----
    blocks_report(a)
    print(
        "\nCLASSIFICATION: "
        + ("INCOMPLETE; " if incomplete else "")
        + ("PERTURBED; " if perturbed else "")
        + f"k* = {ks}; near-optimal {near}; limits "
        + ", ".join(f"{s['k']}:{s['limit']}" for s in summary)
    )


def moments(x: dict) -> dict[tuple[str, str], tuple[float, float, float]]:
    """{(role, activity): (visits, sum ns, sum of squares)} summed over the role's threads"""
    out: dict[tuple[str, str], list[float]] = {}
    for t in x["threads"]:
        for i, m in enumerate(t.get("moments", [])):
            if m[0]:
                v = out.setdefault((t["role"], ACT[i]), [0.0, 0.0, 0.0])
                v[0] += m[0]
                v[1] += m[1]
                v[2] += m[2]
    return {k: tuple(v) for k, v in out.items()}


# the stations of the closed network (brief §7b): a role's activities, and its servers
STATIONS = [
    ("scanner", "Scanner", ["in_maint", "A2", "A3", "issue"], lambda k: 1),
    (
        "workers",
        "Worker",
        ["A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish"],
        lambda k: k,
    ),
    ("reader", "Reader", ["A10"], lambda k: 1),
    ("feeder", "Feeder", ["A0"], lambda k: 1),
]
SHOWN = [("Scanner", "A2"), ("Scanner", "A3"), ("Scanner", "issue"), ("Scanner", "slot_wait"), ("Worker", "A5"),
         ("Worker", "pool"), ("Worker", "A6a"), ("Worker", "A6b"), ("Worker", "A6c"), ("Worker", "A7"),
         ("Worker", "A8"), ("Worker", "A9"), ("Worker", "job_wait"), ("Reader", "read_wait")]  # fmt: skip


def demands(grid: list[dict], a) -> None:
    """Brief §7b: each activity's visits a block V, service a visit S and its C², and its demand
    D = V·S, at each count (medians over the grid's decodes); the closed network's asymptotic
    bound and knee, its population the window N = 4k"""
    rows = []
    per_k: dict[int, dict[tuple[str, str], list[tuple[float, float, float]]]] = {}
    for x in grid:
        for (role, act), (n, sm, sq) in moments(x).items():
            mean = sm / n
            var = max(sq / n - mean * mean, 0.0)
            c2 = var / (mean * mean) if mean > 0 else float("nan")
            rows.append({"round": x["round"], "k": x["k"], "rep": x["rep"], "role": role, "activity": act,
                         "visits": n, "blocks": x["blocks"], "mean_ns": mean, "c2": c2})  # fmt: skip
            per_k.setdefault(x["k"], {}).setdefault((role, act), []).append(
                (n / x["blocks"], mean, c2)
            )
    write_tsv(
        os.path.join(a.tsv_dir, "service.tsv"),
        ["round", "k", "rep", "role", "activity", "visits", "blocks", "mean_ns", "c2"],
        rows,
    )
    print(
        "\n## Visits, service and demand (brief §7b; V visits a block, S µs a visit, C², D = V·S µs a block)\n"
    )
    print("k   " + "".join(f"{(r[0] + '.' + c)[:17]:>19}" for r, c in SHOWN))
    for k in GRID:
        cells = []
        for key in SHOWN:
            v = per_k.get(k, {}).get(key, [])
            if not v:
                cells.append(f"{'-':>19}")
                continue
            V, S, C = med(t[0] for t in v), med(t[1] for t in v) / 1e3, med(t[2] for t in v)
            cells.append(f"{V:>5.2f}/{S:>6.1f}/{C:>5.1f}")
        print(f"{k:<3} " + "".join(cells))
    print(
        "\nThe closed network (population N = the window 4k; D each station's demand a block, ms; X blocks a second):"
    )
    print("k   N    D_scan  D_wkr/k D_read  D_feed  ΣD      X bound  N*      X measured")
    for k in GRID:
        xs = [x for x in grid if x["k"] == k]
        ds = {}
        for name, role, acts, servers in STATIONS:
            vals = []
            for x in xs:
                m = moments(x)
                tot = sum(m.get((role, act), (0, 0.0, 0.0))[1] for act in acts)
                if role == "Reader":
                    tot -= m.get(("Reader", "read_wait"), (0, 0.0, 0.0))[1]
                vals.append(tot / x["blocks"] / 1e6)
            ds[name] = (med(vals), servers(k))
        total = sum(d for d, _ in ds.values())
        dmax = max(d / c for d, c in ds.values())
        n = 4 * k
        bound = min(n / total, 1 / dmax) * 1e3
        knee = total / dmax
        xm = med(x["blocks"] / x["wall_s"] for x in xs)
        print(
            f"{k:<3} {n:<4} {ds['scanner'][0]:>7.4f} {ds['workers'][0] / k:>7.4f} {ds['reader'][0]:>7.4f} {ds['feeder'][0]:>7.4f} {total:>7.4f} {bound:>8.1f} {knee:>6.1f} {xm:>10.1f}"
        )


def blocks_report(a) -> None:
    feats = {}
    with open(a.features) as f:
        cols = f.readline().rstrip("\n").split("\t")
        for line in f:
            r = dict(zip(cols, line.rstrip("\n").split("\t")))
            feats[int(r["place"])] = r
    nf = len(feats)
    rows = []
    for name in sorted(os.listdir(a.blocks_dir)):
        if re.fullmatch(r"blocks-r\d\d\.tsv", name):
            with open(os.path.join(a.blocks_dir, name)) as f:
                cols = f.readline().rstrip("\n").split("\t")
                for line in f:
                    rows.append(dict(zip(cols, line.rstrip("\n").split("\t"))))
    if not rows:
        print("\nno block rows")
        return
    print(
        f"\n## Each route's blocks ({len(rows)} block rows; the features joined by place mod {nf})\n"
    )
    print(
        "route       blocks  n median  r/n     h1    | ns a row at k = 1, 10, 20: A5, A6 (a+b+c), A7, A8, A9"
    )
    for code, name in ROUTE_CODE.items():
        rs = [r for r in rows if int(r["route"]) == code]
        if not rs:
            continue
        one = [r for r in rs if r["k"] == "1"]
        n = med(int(r["n"]) for r in one)
        rn = med(int(r["r"]) / int(r["n"]) for r in one if int(r["n"]))
        h1 = med(
            float(feats[int(r["place"]) % nf]["h1"]) for r in one if int(r["place"]) % nf in feats
        )
        parts = []
        for k in ("1", "10", "20"):
            kr = [r for r in rs if r["k"] == k and int(r["n"]) > 0]
            if not kr:
                parts.append("-")
                continue

            def per(cols):
                return med(sum(int(r[c]) for c in cols) / int(r["n"]) for r in kr)

            parts.append(
                "/".join(
                    f"{per(c):.1f}"
                    for c in (["w_A5"], ["w_A6a", "w_A6b", "w_A6c"], ["w_A7"], ["w_A8"], ["w_A9"])
                )
            )
        print(f"{name:<12}{len(one):>6}{n:>10.0f}{rn:>8.4f}{h1:>7.3f}  | " + "  ".join(parts))


if __name__ == "__main__":
    sys.exit(main())
