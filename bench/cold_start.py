"""W1, the cold start: the gap block's driver with its arms, and the judge
(docs/journal/2026-11-10-cold-start.md, the registration; the driver is #318's `run_gaps`,
bench/two_modes.py on bench/two-modes-discriminator, copied and extended).

    python3 bench/cold_start.py layout
    python3 bench/cold_start.py run AB SPIN CORPUS CORPUS_X4 OUT [--each N]
    python3 bench/cold_start.py judge DIR

`layout` prints P's CPUs (the lowest-numbered thread of each physical core) and B's CPU (the
last online CPU). `run` writes one JSON line a run to OUT. `judge` reads DIR/runs.jsonl and
writes DIR/judgement.json and DIR/outcome.txt, the decision on its first line.
"""

import argparse
import glob
import json
import os
import random
import statistics as st
import subprocess
import sys
import threading
import time

SCHEMA = "cold_start/run/1"
ARM = "mibz2-default33"  # the default plan, as the run-fed leader's bundle ran it
THREADS = 10
GAP_S = 0.25
EACH = 30
SEED = 20261112
ARMS = "GZPWBL"
JUDGED = "PWB"
THRESHOLD_S = 0.21  # fast: the child's own wall below it (as registered for #318)
FAST_NEEDED = 25
W_SPIN_MS = 20
SAMPLE_S = 0.02
WINDOW_S = 0.1
DECODER_PREFIX = "mibz2-"


def order(each, seed=SEED):
    slots = [a for a in ARMS for _ in range(each)]
    random.Random(seed).shuffle(slots)
    return "".join(slots)


def read(path):
    try:
        with open(path) as f:
            return f.read().strip()
    except OSError:
        return None


def online_cpus():
    out = []
    for part in (read("/sys/devices/system/cpu/online") or "0").split(","):
        a, _, b = part.partition("-")
        out += range(int(a), int(b or a) + 1)
    return out


def layout():
    """P: the lowest-numbered thread of each physical core; B: the last online CPU"""
    seen, firsts = set(), []
    for c in online_cpus():
        sib = read(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list") or str(c)
        if sib not in seen:
            seen.add(sib)
            firsts.append(c)
    return ",".join(map(str, firsts)), online_cpus()[-1]


def freqs():
    return {
        c: (int(v) if (v := read(f"/sys/devices/system/cpu/cpu{c}/cpufreq/scaling_cur_freq")) else None)
        for c in online_cpus()
    }


def sampler(pid, stop, out):
    """Passive: every SAMPLE_S, each thread's name and last CPU, and the process's rchar"""
    t0 = time.monotonic()
    while not stop.is_set():
        for d in glob.glob(f"/proc/{pid}/task/*"):
            s = read(d + "/stat")
            if not s:
                continue
            name = s[s.index("(") + 1 : s.rindex(")")]
            out["threads"][os.path.basename(d)] = [name, int(s[s.rindex(")") + 2 :].split()[36])]
        io = read(f"/proc/{pid}/io")
        if io:
            for line in io.splitlines():
                if line.startswith("rchar:"):
                    out["rchar"].append([round(time.monotonic() - t0, 4), int(line.split()[1])])
        stop.wait(SAMPLE_S)


def run(ab, spin, corpus, corpus_x4, out, each):
    p_cpus, b_cpu = layout()
    slots = order(each)
    print(f"cold-start: P on CPUs {p_cpus}, B on CPU {b_cpu}; {len(slots)} runs, order {slots}")
    ok = True
    prev_reap = None
    with open(out, "a") as f:
        for i, arm in [(None, "C")] + list(enumerate(slots)):
            holder = None
            if arm == "B":
                holder = subprocess.Popen([spin, "hold", str(b_cpu)])
            if arm not in "CZ":
                left = GAP_S - (time.monotonic_ns() - prev_reap) / 1e9
                if left > 0:
                    time.sleep(left)
            cmd = [ab, "child", ARM, str(THREADS), corpus_x4 if arm == "L" else corpus]
            if arm == "P":
                cmd = ["taskset", "-c", p_cpus] + cmd
            elif arm == "W":
                cmd = [spin, "burst", str(W_SPIN_MS), "--"] + cmd
            launch = time.monotonic_ns()
            p = subprocess.Popen(cmd, stdout=subprocess.PIPE)
            seen = {"threads": {}, "rchar": []}
            stop = threading.Event()
            th = threading.Thread(target=sampler, args=(p.pid, stop, seen), daemon=True)
            th.start()
            with p.stdout:
                line = p.stdout.read()
            fq = freqs()
            stop.set()
            th.join()
            _, status, ru = os.wait4(p.pid, 0)
            reap = time.monotonic_ns()
            if holder:
                holder.kill()
                holder.wait()
            rc = os.waitstatus_to_exitcode(status)
            try:
                child = json.loads(line)
            except ValueError:
                child = None
            rec = {
                "schema": SCHEMA,
                "index": i,
                "arm": arm,
                "cmd": cmd,
                "gap_s": None if prev_reap is None else (launch - prev_reap) / 1e9,
                "span_s": (reap - launch) / 1e9,
                "exit": rc,
                "child": child,
                "rusage": {"utime_s": ru.ru_utime, "stime_s": ru.ru_stime, "maxrss_kb": ru.ru_maxrss},
                "freq_khz_at_exit": fq,
                "threads_last_cpu": seen["threads"],
                "rchar": seen["rchar"] if arm == "L" else None,
            }
            # (B's spinner is stopped after the reap: the next gap starts when it is gone)
            prev_reap = time.monotonic_ns() if holder else reap
            ok &= rc == 0 and isinstance(child, dict) and not child.get("err")
            f.write(json.dumps(rec) + "\n")
            f.flush()
    return 0 if ok else 1


def wall(r):
    c = r.get("child")
    w = c.get("wall") if isinstance(c, dict) else None
    return w if r.get("exit") == 0 and isinstance(w, (int, float)) and w > 0 else None


def med(xs):
    xs = [x for x in xs if x is not None]
    return st.median(xs) if xs else None


def windows(rchar):
    """Input bytes consumed in each 100 ms window from the first read"""
    if not rchar or len(rchar) < 2:
        return []

    def at(t):
        v = rchar[0][1]
        for x, y in rchar:
            if x > t:
                break
            v = y
        return v

    out, t = [], 0.0
    while t + WINDOW_S <= rchar[-1][0] + 1e-9:
        out.append(at(t + WINDOW_S) - at(t))
        t += WINDOW_S
    return out


def judge(d):
    recs = [json.loads(x) for x in open(os.path.join(d, "runs.jsonl")) if x.strip()]
    recs = [r for r in recs if r.get("arm") != "C"]
    each = max((sum(r["arm"] == a for r in recs) for a in ARMS), default=0)
    dry = each < EACH
    arms = {}
    for a in ARMS:
        rs = [r for r in recs if r["arm"] == a]
        ws = [wall(r) for r in rs]
        timed = [w for w in ws if w is not None]
        cpus = [
            c
            for r in rs
            for name, c in r.get("threads_last_cpu", {}).values()
            if name.startswith(DECODER_PREFIX)
        ]
        arms[a] = {
            "runs": len(rs),
            "timed": len(timed),
            "fast": sum(w < THRESHOLD_S for w in timed),
            "wall_median_s": med(timed),
            "wall_fast_median_s": med(w for w in timed if w < THRESHOLD_S),
            "wall_slow_median_s": med(w for w in timed if w >= THRESHOLD_S),
            "cpu_median_s": med(r["rusage"]["utime_s"] + r["rusage"]["stime_s"] for r in rs),
            "freq_mhz_median_at_exit": med(
                v / 1e3 for r in rs for v in r.get("freq_khz_at_exit", {}).values() if v
            ),
            "decoder_last_cpus": sorted(set(cpus)),
        }
        if a == "L":
            ws100 = [windows(r.get("rchar")) for r in rs]
            n = max(map(len, ws100), default=0)
            arms[a]["input_bytes_per_100ms_median"] = [
                med(w[k] for w in ws100 if len(w) > k) for k in range(n)
            ]
    need = EACH if not dry else each
    incomplete = [a for a in JUDGED if arms[a]["timed"] < need]
    passed = [a for a in JUDGED if arms[a]["fast"] >= FAST_NEEDED]
    if incomplete:
        decision = f"INCOMPLETE: {', '.join(incomplete)} not timed in {need} runs; no decision"
    elif "P" in passed or "W" in passed:
        parts = []
        if "P" in passed:
            parts.append("P: pin a thread a core on the target-gated path")
        if "W" in passed:
            parts.append("W: a start-up spin in the pool, target-gated")
        decision = "INTEGRATE " + "; ".join(parts)
    elif "B" in passed:
        decision = "B ALONE: nothing for the library; the bench's settle changes; record it"
    else:
        decision = "NONE: no arm fast in 25 of 30; nothing integrates"
    if dry:
        decision = f"DRY (not evidence, {each} runs an arm): {decision}"
    tally = " ".join(f"{a} {arms[a]['fast']}/{arms[a]['timed']}" for a in ARMS if a != "L")
    lines = [decision, f"fast (wall < {THRESHOLD_S} s) / timed: {tally} (judged: P W B; L is x4 input, not classed)"]
    for a in ARMS:
        x = arms[a]
        lines.append(
            f"{a}: wall median {x['wall_median_s']}, CPU median {x['cpu_median_s']}, "
            f"clock at exit {x['freq_mhz_median_at_exit']} MHz, decoder last CPUs {x['decoder_last_cpus']}"
        )
    if arms["L"].get("input_bytes_per_100ms_median"):
        lines.append(f"L input bytes per 100 ms (median over runs): {arms['L']['input_bytes_per_100ms_median']}")
    with open(os.path.join(d, "judgement.json"), "w") as f:
        json.dump({"decision": decision, "dry": dry, "arms": arms}, f, indent=1)
    with open(os.path.join(d, "outcome.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("layout")
    s = sub.add_parser("run")
    for k in ("ab", "spin", "corpus", "corpus_x4", "out"):
        s.add_argument(k)
    s.add_argument("--each", type=int, default=EACH)
    s = sub.add_parser("judge")
    s.add_argument("dir")
    a = ap.parse_args(argv)
    if a.cmd == "layout":
        print(*layout())
        return 0
    if a.cmd == "run":
        return run(a.ab, a.spin, a.corpus, a.corpus_x4, a.out, a.each)
    return judge(a.dir)


if __name__ == "__main__":
    sys.exit(main())
