"""The forced-team screen's checker (DRAFT stub, not yet run or adopted).

docs/journal/2026-11-10-team-screen-design-draft.md (the Guardian's D083). From one arm's
callgrind profiles (one file a thread, `--dump-instr=yes --separate-threads=yes`), it
asserts the screen's executed kernels (A2) and cooperative engagement (A3), and writes
the matched per-thread table (A4); from the harness's child records, the arms' exact
outputs and errors (A1). Every failed assertion is named, and the exit status is 1.

    python3 bench/team_screen.py check PREFIX --binary AB \\
        --require 'coop::traverse_staged=2' --forbid 'coop::traverse(?!_staged)' \\
        --require 'FusedBuild>::share=2' --require 'mtf_checkpoints_avx2=1' \\
        --require 'Relay>::walk_and_leave=1' --coop-blocks 233 --walked 209697662 \\
        [--table OUT.json]
    python3 bench/team_screen.py outputs A.json B.json ...

No timing is read, and nothing here is an acceptance figure.
"""

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import team_gap_count as tg  # noqa: E402


def calls_to(profile, rx):
    return sum(
        c for (caller, callee), c in profile.calls.items() if callee and re.search(rx, callee)
    )


def check(args):
    profs = tg.load(args.prefix)
    if not profs:
        sys.exit(f"no profiles at {args.prefix}")
    dis = tg.disasm(args.binary, r"coop::traverse")
    lines = tg.coop_lines(args.coop_src)
    failed = []
    # A2: the executed kernels
    for spec in args.require:
        rx, _, k = spec.rpartition("=")
        threads = [p.thread for p in profs if calls_to(p, rx) > 0]
        if len(threads) < int(k):
            failed.append(f"A2: {rx} executed on {len(threads)} threads, needs {k}")
    for rx in args.forbid:
        threads = [p.thread for p in profs if calls_to(p, rx) > 0]
        if threads:
            failed.append(f"A2: {rx} executed (threads {threads}), must not be")
    # A3: engagement, per thread
    rows = []
    walked = claims = takes = calls = 0
    for p in profs:
        st, _ = tg.stages(p)
        w = tg.walker_counts(p, dis, lines)
        shares = calls_to(p, r"FusedBuild>::share")
        row = dict(thread=p.thread, total_ir=p.total, shares=shares)
        row.update({s: st[s] for s in ("MTF", "build", "walk", "coordination")})
        for v, c in w.items():
            if c["calls"]:
                row[v] = dict(
                    calls=c["calls"],
                    claims=c["claims"],
                    fragments=c["fragments"],
                    flush_ir=c["flush_ir"],
                    **{k: c["walked"][k] for k in ("take", "full", "tail")},
                )
                walked += sum(c["walked"][k] for k in ("take", "full", "tail"))
                claims += c["claims"]
                takes += c["walked"]["take"]
                calls += c["calls"]
        rows.append(row)
        if shares:  # a helper
            wcalls = sum(c["calls"] for c in w.values())
            if not (shares == wcalls == args.coop_blocks):
                failed.append(
                    f"A3: helper thread {p.thread}: shares {shares}, walker calls {wcalls}, blocks {args.coop_blocks}"
                )
            if not any(c["walked"]["take"] for c in w.values()):
                failed.append(f"A3: helper thread {p.thread} walked no segment")
    if args.walked is not None and walked != args.walked:
        failed.append(f"A3: walked bytes {walked:,} != {args.walked:,}")
    if claims != takes + calls:
        failed.append(f"A3: claims {claims:,} != takes {takes:,} + walker calls {calls:,}")
    if args.table:
        with open(args.table, "w") as f:
            json.dump(
                dict(prefix=args.prefix, threads=rows, walked=walked, claims=claims), f, indent=1
            )
    for msg in failed:
        print("FAIL", msg)
    print("team screen:", "FAIL" if failed else "ok", f"({len(profs)} thread profiles)")
    return 1 if failed else 0


def outputs(args):
    recs = []
    for path in args.records:
        with open(path) as f:
            j = json.load(f)
        recs.append((path, j.get("out"), j.get("hash"), j.get("err")))
    first = recs[0][1:]
    bad = [r for r in recs if r[1:] != first]
    for r in recs:
        print(*r)
    if bad:
        print("FAIL A1: the arms differ")
        return 1
    print("A1 ok")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check")
    c.add_argument("prefix")
    c.add_argument("--binary", required=True)
    c.add_argument("--coop-src", default="src/coop.rs")
    c.add_argument("--require", action="append", default=[], help="REGEX=MIN_THREADS")
    c.add_argument("--forbid", action="append", default=[], help="REGEX")
    c.add_argument("--coop-blocks", type=int, required=True)
    c.add_argument("--walked", type=int)
    c.add_argument("--table")
    o = sub.add_parser("outputs")
    o.add_argument("records", nargs="+")
    args = ap.parse_args()
    return check(args) if args.cmd == "check" else outputs(args)


if __name__ == "__main__":
    sys.exit(main())
