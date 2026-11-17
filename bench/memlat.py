#!/usr/bin/env python3
"""Load latency by data source, sampled with instruction pointers: where the slow loads are
served from and which instructions wait on them (`just bench-miss-cost`; MONODROMY's request with
record 47 on PR #497). A diagnostic: it records and decides nothing. Standard library only.

    memlat.py run --ab AB --arm A --arm B --threads 10 --reps 3 --file F --out DIR [--period 10007] [--ldlat 30]
    memlat.py report DIR --arm A --arm B --threads 10 --reps 3 [--top 10]

**How it samples.** Each rep runs each arm (`ab child ARM T FILE`, pinned as `ab run --spread`
pins: the first CPU of each of the first T cores) under `perf record` with the event `perf mem
record` uses on Intel cores, `cpu/mem-loads,ldlat=L/P`: precise samples of retired loads slower
than L core cycles, one every `--period` of them, each with its latency (`-W`, the weight, in core
cycles), its data source and its address (`-d`). As this user: per-process sampling needs
`kernel.perf_event_paranoid` at 2 or less, and the workstation's is 1. The arms alternate in
order from rep to rep. `perf script -F data_src,weight,ip,sym` writes each sample as text, kept
gzipped in DIR (`ARM-repN.txt.gz`); perf.data is deleted. The first rep's `perf mem report
--stdio --sort=mem` is kept beside it as perf's own reading, for a check by hand.

**The data source** is read from perf's decoded `|LVL ...|` field: RAM (any `RAM`), L3, L2,
LFB/MAB (a load that found its line already in flight in the fill buffer), L1, else other.

**VOID** (exit 1, nothing reported): a (arm, rep) missing or duplicated; a run whose perf, child
or `perf script` failed; an arm with no sample parsed; the arms' outputs differing.

**What it reports**, for each arm, the reps pooled: by data source, the share of samples, the
share of the summed latency, and the median and mean latency in cycles; then the top IPs by
summed latency, each with its symbol, its share, its samples, its median latency and the source
most of its samples came from. Lines the parser could not read are counted and printed.
"""
import argparse
import gzip
import json
import os
import re
import statistics
import subprocess
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from counters import harness, spread  # noqa: E402

# perf script's sample line for -F data_src,weight,ip,sym (tools/perf/builtin-script.c, 6.8):
# the data source as hex and decoded, the weight right-aligned in 16, then the IP and symbol
LINE = re.compile(r"^\s*(?P<src>[0-9a-f]+)\s+(?P<txt>\|OP.*?)\s+(?P<w>\d+)\s+(?P<ip>[0-9a-f]+)(?:\s+(?P<sym>.*?))?\s*$")
LVL = re.compile(r"\|LVL ([^|]*)")
SOURCES = ["L1", "LFB/MAB", "L2", "L3", "RAM", "other"]


def source(lvl):
    """The data source of perf's decoded level"""
    for key, name in (("RAM", "RAM"), ("L3", "L3"), ("L2", "L2"), ("LFB", "LFB/MAB"), ("L1", "L1")):
        if key in lvl:
            return name
    return "other"


def parse(lines):
    """Samples (source, weight, ip, sym) and the count of lines not read"""
    samples, bad = [], 0
    for line in lines:
        if not line.strip():
            continue
        m = LINE.match(line.rstrip("\n"))
        lv = LVL.search(line)
        if not m or not lv:
            bad += 1
            continue
        samples.append((source(lv.group(1)), int(m.group("w")), m.group("ip"), (m.group("sym") or "[unknown]").strip()))
    return samples, bad


def stem(out, arm, rep):
    return os.path.join(out, f"{arm.replace('/', '_').replace('@', '_at_')}-rep{rep}")


def run(a):
    os.makedirs(a.out, exist_ok=True)
    t = a.threads
    with open(os.path.join(a.out, "memlat.jsonl"), "w") as log:
        for rep in range(a.reps):
            for arm in (a.arm if rep % 2 == 0 else list(reversed(a.arm))):
                s = stem(a.out, arm, rep)
                data = s + ".data"
                rec = {"arm": arm, "threads": t, "rep": rep, "period": a.period, "ldlat": a.ldlat}
                cpus = spread(t)
                cmd = ["perf", "record", "-q", "-e", f"cpu/mem-loads,ldlat={a.ldlat}/P", "-W", "-d",
                       "-c", str(a.period), "-o", data, "--"]
                if cpus:
                    cmd += ["taskset", "-c", cpus]
                exe, name, env = harness(a.ab, arm)
                cmd += [exe, "child", name, str(t), a.file]
                rec["cpus"] = cpus or "unpinned"
                try:
                    p = subprocess.run(cmd, capture_output=True, text=True,
                                       env={**os.environ, **env} if env else None)
                except OSError as e:
                    rec["err"] = f"perf did not start: {e}"
                    log.write(json.dumps(rec) + "\n")
                    continue
                rec["rc"] = p.returncode
                if p.returncode != 0:
                    rec["err"] = (p.stderr.strip().splitlines() or ["failed"])[-1][:300]
                else:
                    try:
                        child = json.loads(p.stdout.strip().splitlines()[-1])
                        rec.update(out=child.get("out"), hash=child.get("sha256") or child.get("hash"),
                                   wall=child.get("wall"), child_err=child.get("err"))
                    except (ValueError, IndexError):
                        rec["err"] = "no child record"
                    q = subprocess.run(["perf", "script", "-i", data, "-F", "data_src,weight,ip,sym"],
                                       capture_output=True, text=True)
                    rec["script_rc"] = q.returncode
                    if q.returncode != 0:
                        rec["err"] = "perf script: " + (q.stderr.strip().splitlines() or ["failed"])[-1][:300]
                    with gzip.open(s + ".txt.gz", "wt") as f:
                        f.write(q.stdout)
                    rec["script"] = os.path.basename(s + ".txt.gz")
                    if rep == 0:
                        r = subprocess.run(["perf", "mem", "report", "-i", data, "--stdio", "--sort=mem"],
                                           capture_output=True, text=True)
                        with open(s + "-mem-report.txt", "w") as f:
                            f.write(r.stdout or r.stderr)
                try:
                    os.remove(data)
                except OSError:
                    pass
                log.write(json.dumps(rec) + "\n")
                print(f"  memlat rep {rep} t={t} {arm}: {rec.get('err') or 'sampled'}", flush=True)
    return 0


def report(a):
    roster = {(arm, r) for arm in a.arm for r in range(a.reps)}
    rows, errors, outputs = {}, [], set()
    path = os.path.join(a.dir, "memlat.jsonl")
    if not os.path.exists(path):
        errors.append(f"{path}: missing")
    else:
        for ln, line in enumerate(open(path), 1):
            if not line.strip():
                continue
            try:
                r = json.loads(line)
                key = (r["arm"], int(r["rep"]))
            except (ValueError, KeyError, TypeError):
                errors.append(f"line {ln}: not a record")
                continue
            if key in rows:
                errors.append(f"{key} duplicated")
                continue
            if key not in roster or int(r.get("threads", -1)) != a.threads:
                errors.append(f"{key} not in the roster")
                continue
            rows[key] = r
            if r.get("err") or r.get("rc") != 0 or r.get("child_err") or r.get("script_rc") != 0:
                errors.append(f"{key}: the run failed: {r.get('err') or r.get('child_err') or r.get('rc')}")
                continue
            outputs.add((r.get("out"), r.get("hash")))
    missing = sorted(roster - set(rows))
    if missing:
        errors.append(f"{len(missing)} of {len(roster)} runs missing, e.g. {missing[:3]}")
    if len(outputs) > 1:
        errors.append(f"the arms' outputs differ: {sorted(outputs, key=str)[:3]}")
    pooled, bad = defaultdict(list), Counter()
    if not errors:
        for (arm, rep), r in sorted(rows.items()):
            try:
                with gzip.open(os.path.join(a.dir, r["script"]), "rt") as f:
                    s, b = parse(f)
            except (OSError, KeyError, EOFError) as e:
                errors.append(f"{(arm, rep)}: the samples unreadable: {e}")
                continue
            pooled[arm] += s
            bad[arm] += b
        for arm in a.arm:
            if not pooled[arm]:
                errors.append(f"{arm}: no sample parsed ({bad[arm]} lines not read)")
    if errors:
        print(f"VOID: the load-latency samples do not make the registered run ({len(errors)} problems); nothing reported")
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    print(f"\nload latency at {a.threads} threads (core cycles), {a.reps} reps pooled an arm, "
          "loads slower than the threshold sampled")
    for arm in a.arm:
        s = pooled[arm]
        total_w = sum(w for _, w, _, _ in s)
        print(f"\n  {arm}: {len(s)} samples, {bad[arm]} lines not read")
        print(f"  {'source':10s} {'samples':>9s} {'latency share':>14s} {'median':>8s} {'mean':>8s}")
        for src in SOURCES:
            ws = [w for so, w, _, _ in s if so == src]
            if ws:
                print(f"  {src:10s} {len(ws) / len(s):9.1%} {sum(ws) / total_w:14.1%} "
                      f"{statistics.median(ws):8.0f} {statistics.mean(ws):8.1f}")
        by_ip = defaultdict(list)
        for so, w, ip, sym in s:
            by_ip[(ip, sym)].append((so, w))
        top = sorted(by_ip.items(), key=lambda kv: -sum(w for _, w in kv[1]))[:a.top]
        print(f"  the top {len(top)} IPs by summed latency:")
        print(f"    {'ip':>16s} {'share':>7s} {'samples':>8s} {'median':>7s} {'mostly':>8s}  symbol")
        for (ip, sym), v in top:
            src = Counter(so for so, _ in v).most_common(1)[0][0]
            print(f"    {ip:>16s} {sum(w for _, w in v) / total_w:7.1%} {len(v):8d} "
                  f"{statistics.median(w for _, w in v):7.0f} {src:>8s}  {sym[:60]}")
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("--ab", required=True)
    r.add_argument("--arm", action="append", required=True)
    r.add_argument("--threads", type=int, required=True)
    r.add_argument("--reps", type=int, default=3)
    r.add_argument("--file", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--period", type=int, default=10007, help="one sample every PERIOD qualifying loads")
    r.add_argument("--ldlat", type=int, default=30, help="the latency threshold, core cycles")
    p = sub.add_parser("report")
    p.add_argument("dir")
    p.add_argument("--arm", action="append", required=True)
    p.add_argument("--threads", type=int, required=True)
    p.add_argument("--reps", type=int, default=3)
    p.add_argument("--top", type=int, default=10)
    a = ap.parse_args(argv[1:])
    return run(a) if a.cmd == "run" else report(a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
