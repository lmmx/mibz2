#!/usr/bin/env python3
"""bench-kernel-count's driver (docs/journal/2026-11-25-kernel-count-registration.md): lazy
against eager workers below 8, and the worker-count cap, through the explicit workers and the
public API. Every decode in a process of its own (`ab child`, examples/ab.rs), unpinned, in a
fixed minimal environment; what each cell builds is probed beside it (examples/kernel_count.rs).

The cells (29 a round):
- **explicit**: `BzReader::parallel_workers(r, k, 4k, detect, lazy)` (`ab child
  mibz2-workers-lazy K` and `mibz2-workers-eager K`), lazy at k = 1..10 and eager at k = 1..9;
  the cap question's cells are lazy k = 7, 8, 9 and 10 among them;
- **api**: `BzReader::parallel(r, t)` as shipped (`ab child mibz2 T`), t = 3..12 asked.

The order:
1. **The probes, untimed:** every explicit cell's build (`kernel_count workers K LAZY`), which
   must be k workers, a window of 4k and the kernel asked; and the public API's at every t.
2. **The pre-warm:** PREWARM seconds of decodes of lazy 8, discarded (section `prewarm`).
3. **ROUNDS randomised complete rounds:** in each, the 29 cells once each in an order drawn from
   SEED. An observation is SETTLE seconds of sleep, then REPS decodes of its cell back to back,
   each a fresh process. An api observation is followed by its probe (`kernel_count built T`).

Every child's line goes to runs.jsonl with this driver's fields: section, round, order (its
position in the round), rep, cell, arm, k or t, the exit status, the span the driver timed,
the machine's other busy cores over the decode (from /proc/stat, less the child's CPU) and the
load. A child that fails is kept, with `"ok": false`, and the run goes on.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import subprocess
import sys
import time

LAZY_KS = list(range(1, 11))
EAGER_KS = list(range(1, 10))
API_TS = list(range(3, 13))
PREWARM_CELL = ("explicit", "lazy", 8)
ENV = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"}
HZ = os.sysconf("SC_CLK_TCK")


def cells():
    out = [("explicit", "lazy", k) for k in LAZY_KS]
    out += [("explicit", "eager", k) for k in EAGER_KS]
    out += [("api", "api", t) for t in API_TS]
    return out


def cell_id(c):
    sec, kind, n = c
    return f"api-t{n:02d}" if sec == "api" else f"{kind}-k{n:02d}"


def arm_of(c):
    sec, kind, n = c
    if sec == "api":
        return "mibz2", n
    return ("mibz2-workers-lazy" if kind == "lazy" else "mibz2-workers-eager"), n


def stat():
    with open("/proc/stat") as f:
        v = [int(x) for x in f.readline().split()[1:]]
    # user nice system idle iowait irq softirq steal
    return v[0] + v[1] + v[2] + v[5] + v[6] + v[7]


def load1():
    with open("/proc/loadavg") as f:
        return float(f.read().split()[0])


def child(ab, c, corpus):
    arm, n = arm_of(c)
    # (one argument layout for every decode: the count two digits wide)
    cmd = [ab, "child", arm, f"{n:02d}", corpus]
    load = load1()
    b0 = stat()
    t0 = time.monotonic()
    p = subprocess.run(cmd, env=ENV, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    span = time.monotonic() - t0
    b1 = stat()
    rec = {"cmd": " ".join(cmd[1:4]), "exit": p.returncode, "span": round(span, 4), "load1": load}
    line = p.stdout.strip().splitlines()[-1] if p.stdout.strip() else ""
    try:
        r = json.loads(line)
    except ValueError:
        r = {}
    rec.update(r)
    cpu = r.get("cpu", 0.0) or 0.0
    rec["others_busy"] = round(max(0.0, (b1 - b0) / HZ - cpu) / max(span, 1e-9), 3)
    rec["ok"] = p.returncode == 0 and bool(r) and not r.get("err")
    if not rec["ok"]:
        rec["stderr"] = p.stderr[-2000:]
    return rec


def probe(kc, c):
    sec, kind, n = c
    if sec == "api":
        cmd = [kc, "built", str(n)]
    else:
        cmd = [kc, "workers", str(n), "1" if kind == "lazy" else "0"]
    p = subprocess.run(cmd, env=ENV, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        r = json.loads(p.stdout.strip().splitlines()[-1])
    except (ValueError, IndexError):
        r = {"kind": "built", "error": p.stderr[-500:]}
    r["exit"] = p.returncode
    return r


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ab", required=True)
    ap.add_argument("--probe", required=True, help="examples/kernel_count's binary")
    ap.add_argument("--corpus", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--settle", type=float, default=1.0)
    ap.add_argument("--prewarm", type=float, default=50.0)
    ap.add_argument("--seed", type=int, default=20261125)
    a = ap.parse_args()
    runs = open(os.path.join(a.out, "runs.jsonl"), "a")

    def emit(rec):
        runs.write(json.dumps(rec, sort_keys=True) + "\n")
        runs.flush()

    all_cells = cells()
    # 1. the probes, untimed
    bad = 0
    for c in all_cells:
        r = probe(a.probe, c)
        r.update(section="probe", round=-1, cell=cell_id(c))
        sec, kind, n = c
        if sec == "explicit":
            r["as_asked"] = (
                r.get("exit") == 0
                and r.get("threads") == n
                and r.get("window") == 4 * n
                and r.get("lazy") == (kind == "lazy")
                and not r.get("service")
                and not r.get("pipelined")
            )
            bad += not r["as_asked"]
        emit(r)
        print(
            f"probe {cell_id(c)}: {json.dumps({k: r.get(k) for k in ('threads', 'window', 'lazy', 'service', 'pipelined')})}",
            flush=True,
        )
    if bad:
        print(
            f"kernel_count_run: {bad} explicit cells did not build as asked (runs.jsonl)",
            flush=True,
        )
    # 2. the pre-warm, discarded
    t_end = time.monotonic() + a.prewarm
    i = 0
    while time.monotonic() < t_end:
        r = child(a.ab, PREWARM_CELL, a.corpus)
        r.update(section="prewarm", round=-1, order=i, rep=0, cell=cell_id(PREWARM_CELL))
        emit(r)
        i += 1
    print(
        f"pre-warm: {i} decodes of {cell_id(PREWARM_CELL)} in {a.prewarm:.0f} s, discarded",
        flush=True,
    )
    # 3. the rounds
    rng = random.Random(a.seed)
    prev = None
    for rnd in range(a.rounds):
        order = all_cells[:]
        rng.shuffle(order)
        t_round = time.monotonic()
        for pos, c in enumerate(order):
            if a.settle > 0:
                time.sleep(a.settle)
            sec, kind, n = c
            for rep in range(a.reps):
                r = child(a.ab, c, a.corpus)
                r.update(
                    section=sec,
                    round=rnd,
                    order=pos,
                    rep=rep,
                    cell=cell_id(c),
                    kind=kind,
                    n=n,
                    prev=prev,
                )
                emit(r)
                if not r["ok"]:
                    print(
                        f"r{rnd} {cell_id(c)} rep {rep}: FAILED (exit {r['exit']}): {r.get('err') or r.get('stderr', '')[:200]}",
                        flush=True,
                    )
            prev = cell_id(c)
            if sec == "api":
                b = probe(a.probe, c)
                b.update(section="api-built", round=rnd, cell=cell_id(c), n=n)
                emit(b)
        print(
            f"round {rnd + 1}/{a.rounds}: {len(order)} cells x {a.reps} in {time.monotonic() - t_round:.0f} s",
            flush=True,
        )
    runs.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
