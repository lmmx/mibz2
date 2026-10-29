"""Where the cooperative kernel's instructions go (lmmx/mibz2#77; research only, no kernel change).

E3 (docs/journal/2026-10-28-cooperative-inversion.md §10.4) measured the instructions per
output byte: production (p4) 23.8, the cooperative kernel at k = 1 (4x1) 31.8, in teams
(3x3) 35.3, libsais (s4x1) 21.4. This attributes them to code:

- **Three sampled events a run**, each in a `perf record` of its own on `grid FILE 11 ARM`
  (one arm alone, pinned as in the grid), at a fixed period so that samples x period counts
  the event:
  - `instructions:u`, where the retired instructions are;
  - `cycles:u`, where the time is;
  - `cycle_activity.stalls_mem_any`, where the cycles stalled on memory are. It is left out
    where the PMU lacks it.
- **Per symbol, per output byte:** each event's count over the bytes the run inverted (its 11
  timed rounds and the untimed check round). Each symbol is labelled by phase:
  - walk, build, synchronisation, copies;
  - setup, which reads the corpus and runs MTF before any round.
- **The hottest instructions** of each arm's hottest symbols, per event (`perf annotate`), with
  their source lines where the debug info has them.
- **Provenance:** the commit, the binary's SHA-256 and build id, rustc's and perf's versions.

Needs sudo for perf. Writes coop-profile-<UTC time>/ (every report and annotation in full)
and coop-profile-<UTC time>.md, the summary; then packs the directory as
coop-profile-<UTC time>.tar.gz. Upload the .md and the .tar.gz.

  just bench-coop-profile
  just bench-coop-profile --arms p4,4x1 --rounds 3   (a quick check, about a minute)
"""

import argparse
import datetime
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tarfile
import tempfile

sys.path.insert(0, os.path.dirname(__file__))
import coop_counters as cc  # noqa: E402

EVENTS = ["instructions:u", "cycles:u", "cycle_activity.stalls_mem_any"]
PERIOD = 2_000_003
# a symbol's phase, by the identifiers its (v0-mangled) name contains, first match wins
PHASES = [
    ("walk", ("room_segments", "coop_walk", "decode_chains", "walk_segments")),
    ("build", ("coop_build", "biPSI", "compute_histogram", "bigram", "transpose_bucket")),
    ("build (production's, inlined in invert)", ("invert",)),
    ("synchronisation", ("Pool", "omp_", "_omp_fn", "spin", "park", "futex", "sched_yield")),
    ("copies", ("memmove", "memcpy", "memset")),
    ("setup", ("mtf", "Huff", "grid_mode", "main", "quicksort", "ipnsort", "median3", "fnv")),
]


def demangle(sym):
    """a v0-mangled Rust name's identifiers joined by ::, enough to read it by; else as is"""
    if not sym.startswith("_R"):
        return sym
    # crate disambiguators (Cs<hash>_) carry no name; drop them before reading identifiers
    sym = re.sub(r"Cs[0-9A-Za-z]*_", "C", sym)
    out, i = [], 0
    while i < len(sym):
        m = re.match(r"(\d+)", sym[i:])
        if not m:
            i += 1
            continue
        n, j = int(m.group(1)), i + len(m.group(1))
        word = sym[j : j + n]
        if n and len(word) == n and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", word):
            out.append(word)
            i = j + n
        else:
            i = j
    return "::".join(w for w in out if not re.fullmatch(r"[A-Za-z0-9]{8,}_?", w)) or sym


def phase(sym):
    name = sym + " " + demangle(sym)
    for p, keys in PHASES:
        if any(k in name for k in keys):
            return p
    return "other"


def run(argv, **kw):
    return subprocess.run(argv, capture_output=True, text=True, **kw)


def record(perf, binary, cpus, arm, rounds, event, out_dir):
    """one `perf record` of `event` on the grid; (perf.data path, bytes inverted) or an error"""
    data = os.path.join(out_dir, f"perf-{arm}-{event.split(':')[0]}.data")
    with tempfile.TemporaryDirectory() as d:
        jl = os.path.join(d, "rounds.jsonl")
        cmd = cc.grid(cpus, binary, rounds, arm, jl)
        r = run([*perf, "record", "-q", "-e", event, "-c", str(PERIOD), "-o", data, "--", *cmd])
        if r.returncode:
            return None, 0, r.stderr.strip()[-400:]
        timed = cc.timed(jl)
    per_round = sum(t[2] for t in timed) / max(len(timed), 1)
    # the timed rounds and the untimed check round before them, which inverts every block too
    return data, per_round * (len(timed) + 1), ""


def report(perf, data):
    """[(symbol, samples)] from `perf report`, user symbols"""
    r = run([*perf, "report", "-i", data, "--stdio", "-n", "--no-children", "--sort", "sym", "-q"])
    rows = []
    for line in r.stdout.splitlines():
        m = re.match(r"\s*([\d.]+)%\s+(\d+)\s+\[(.)\]\s+(.+?)\s*$", line)
        if m:
            rows.append((m.group(4), int(m.group(2)), m.group(3)))
    return rows, r.stdout


def annotate(perf, data, sym):
    """the full annotation of `sym`, and its hottest instruction lines"""
    r = run([*perf, "annotate", "-i", data, "--stdio", "-s", sym])
    hot = []
    for line in r.stdout.splitlines():
        m = re.match(r"\s*([\d.]+)\s*:\s*([0-9a-f]+):\s*(.+)$", line)
        if m and float(m.group(1)) > 0:
            hot.append((float(m.group(1)), m.group(2), m.group(3).strip()))
    hot.sort(reverse=True)
    return r.stdout, hot


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--arms", default=None, help="default p4,4x1,3x3 (+s4x1)")
    ap.add_argument("--rounds", type=int, default=11)
    ap.add_argument("--top", type=int, default=4, help="symbols annotated per arm and event")
    ap.add_argument("--lines", type=int, default=30, help="hottest instructions kept per symbol")
    ap.add_argument("--perf", default="sudo perf")
    ap.add_argument("--note", default="")
    a = ap.parse_args()
    perf = shlex.split(a.perf)

    b = cc.build()
    binary, cpus, libsais = b["bin"], b["cpus"], b["libsais"] == "1"
    arms = a.arms.split(",") if a.arms else ["p4", "4x1", "3x3"] + (["s4x1"] if libsais else [])
    cores = len(cpus.split(","))
    if any(cc.threads(x) > cores for x in arms):
        sys.exit(f"bench-coop-profile: {arms} need more than this machine's {cores} cores")

    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    base = f"coop-profile-{stamp}"
    os.makedirs(base)
    print("bench-coop-profile: perf needs sudo", flush=True)

    events, results = [], {}
    for ev in EVENTS:
        probe = run([*perf, "stat", "-e", ev, "--", "true"])
        if probe.returncode == 0 and "<not supported>" not in probe.stderr:
            events.append(ev)
        else:
            print(f"bench-coop-profile: {ev} left out: {probe.stderr.strip()[:200]}")
    for arm in arms:
        for ev in events:
            print(f"bench-coop-profile: {arm} {ev}", flush=True)
            data, nbytes, err = record(perf, binary, cpus, arm, a.rounds, ev, base)
            if data is None:
                results[(arm, ev)] = {"error": err}
                continue
            rows, text = report(perf, data)
            open(os.path.join(base, f"report-{arm}-{ev.split(':')[0]}.txt"), "w").write(text)
            results[(arm, ev)] = {"rows": rows, "bytes": nbytes, "data": data}

    # each arm's hottest symbols by instructions and by cycles, annotated under both events
    hot = {}
    for arm in arms:
        syms = []
        for ev in events[:2]:
            for s, _, kind in results.get((arm, ev), {}).get("rows", [])[: a.top]:
                if kind != "k" and s not in syms:
                    syms.append(s)
        for s in syms:
            for ev in events:
                res = results.get((arm, ev), {})
                if "data" not in res:
                    continue
                text, lines = annotate(perf, res["data"], s)
                tag = re.sub(r"[^A-Za-z0-9_]+", "_", demangle(s))[:60]
                open(os.path.join(base, f"annotate-{arm}-{ev.split(':')[0]}-{tag}.txt"), "w").write(
                    text
                )
                hot[(arm, ev, s)] = lines[: a.lines]

    sha = hashlib.sha256(open(binary, "rb").read()).hexdigest()
    prov = {
        "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
        "binary": binary,
        "binary_sha256": sha,
        "build_id": run(["readelf", "-n", binary]).stdout.split("Build ID:")[-1].split()[0]
        if "Build ID:" in run(["readelf", "-n", binary]).stdout
        else "",
        "rustc": run(["rustc", "-V"]).stdout.strip(),
        "perf": run([*perf, "--version"]).stdout.strip(),
        "period": PERIOD,
        "rounds": a.rounds,
        "arms": arms,
        "events": events,
        "note": a.note,
    }
    json.dump(prov, open(os.path.join(base, "provenance.json"), "w"), indent=1)

    with open(f"{base}.md", "w") as md:
        w = lambda s="": print(s, file=md)  # noqa: E731
        w(f"# Where the cooperative kernel's instructions go ({stamp})")
        w()
        w(f"- **Commit:** {prov['commit'][:7]}; the kernels unchanged from c2620b8")
        w(f"- **Binary:** sha256 {sha[:16]}…, build id {prov['build_id'][:16]}…; {prov['rustc']}")
        w(f"- **perf:** {prov['perf']}; events {', '.join(events)}, each at period {PERIOD}")
        w(
            f"- **Runs:** `grid FILE {a.rounds} ARM`, each arm alone; per byte = the event's count over the bytes of its {a.rounds} timed rounds and its check round"
        )
        if a.note:
            w(f"- **Note:** {a.note}")
        w()
        for arm in arms:
            w(f"## {arm}")
            w()
            per = {}
            for ev in events:
                res = results.get((arm, ev), {})
                if "rows" not in res:
                    w(f"- {ev}: {res.get('error', 'not recorded')}")
                    continue
                for s, n, kind in res["rows"]:
                    per.setdefault(s, {})[ev] = n * PERIOD / res["bytes"]
            w("| phase | symbol | " + " | ".join(f"{e} / B" for e in events) + " |")
            w("|---|---|" + "--:|" * len(events))
            ranked = sorted(per.items(), key=lambda kv: -kv[1].get(events[0], 0))
            for s, v in ranked[:20]:
                cells = [cc.fmt(v.get(e)) for e in events]
                w(f"| {phase(s)} | `{demangle(s)[:70]}` | " + " | ".join(cells) + " |")
            w()
            w("By phase, per output byte (setup is outside the rounds and is not the kernel's):")
            w()
            w("| phase | " + " | ".join(events) + " |")
            w("|---|" + "--:|" * len(events))
            phases = {}
            for s, v in per.items():
                for e, x in v.items():
                    phases.setdefault(phase(s), {}).setdefault(e, 0.0)
                    phases[phase(s)][e] += x
            for p in [q for q, _ in PHASES] + ["other"]:
                if p in phases:
                    w(f"| {p} | " + " | ".join(cc.fmt(phases[p].get(e)) for e in events) + " |")
            w()
            for (arm2, ev, s), lines in hot.items():
                if arm2 != arm or not lines:
                    continue
                w(
                    f"**`{demangle(s)[:70]}`, its hottest instructions by {ev}** (% of the symbol's samples):"
                )
                w()
                w("```")
                for pct, addr, ins in lines:
                    w(f"{pct:6.2f}  {addr}:  {ins[:110]}")
                w("```")
                w()
    # the perf.data files stay beside the text for re-annotating, out of the archive (large)
    with tarfile.open(f"{base}.tar.gz", "w:gz") as t:
        t.add(base, filter=lambda m: None if m.name.endswith(".data") else m)
    print(f"bench-coop-profile: wrote {base}.md, {base}.tar.gz and {base}/")


if __name__ == "__main__":
    if not shutil.which("perf"):
        sys.exit("bench-coop-profile: needs perf on PATH")
    main()
