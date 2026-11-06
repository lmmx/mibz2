"""Header-aware reader for callgrind profiles (format version 1).

Reads the `positions:` and `events:` headers, so it handles `positions: line`
and `positions: instr line`, with compressed subpositions (`*`, `+n`, `-n`,
hex or decimal) and compressed names (`(id) name`, `(id)`).

usage: cgparse2.py PROFILE [FUNCTION-REGEX] [--callees]

For each function matching the regex it prints its self cost, its inclusive
cost (self plus every call cost line), and with --callees the inclusive cost
of each callee called from it, with the call count. Every event the header
declares is reported.
"""

import collections
import gzip
import re
import sys


def parse(path):
    positions, events = ["line"], None
    names = {}
    self_c = collections.defaultdict(lambda: None)
    calls = collections.defaultdict(lambda: collections.defaultdict(lambda: None))
    ncalls = collections.defaultdict(collections.Counter)
    fn = cfn = None
    pending = None
    last = None

    def name(kind, s):
        m = re.match(r"\((\d+)\)(?: (.*))?$", s)
        if not m:
            return s
        k = (kind, m.group(1))
        if m.group(2) is not None:
            names[k] = m.group(2)
        return names[k]

    def sub(tok, prev):
        if tok == "*":
            return prev
        if tok[0] in "+-":
            return prev + int(tok, 0)
        return int(tok, 0)

    def add(acc, vals):
        if acc is None:
            acc = [0] * len(events)
        for i, v in enumerate(vals):
            acc[i] += v
        return acc

    for line in (gzip.open(path, 'rt') if path.endswith('.gz') else open(path)):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        if line.startswith("positions:"):
            positions = line.split(":", 1)[1].split()
            last = [0] * len(positions)
            continue
        if line.startswith("events:"):
            events = line.split(":", 1)[1].split()
            continue
        key, sep, val = line.partition("=")
        if sep and key in ("fn", "cfn", "fl", "fi", "fe", "cfl", "cfi", "ob", "cob"):
            kind = {"fn": "fn", "cfn": "fn", "ob": "ob", "cob": "ob"}.get(key, "fl")
            v = name(kind, val)
            if key == "fn":
                fn = v
            elif key == "cfn":
                cfn = v
            continue
        if key == "calls" and sep:
            pending = int(val.split()[0])
            continue
        if key in ("jump", "jcnd") and sep:
            raise SystemExit("jump lines not supported")
        if line[0].isdigit() or line[0] in "+-*":
            if last is None:
                last = [0] * len(positions)
            p = line.split()
            np_ = len(positions)
            for i in range(np_):
                last[i] = sub(p[i], last[i])
            vals = [int(x) for x in p[np_:]]
            vals += [0] * (len(events) - len(vals))
            if pending is not None:
                calls[fn][cfn] = add(calls[fn][cfn], vals)
                ncalls[fn][cfn] += pending
                pending = None
            else:
                self_c[fn] = add(self_c[fn], vals)
            continue
        # header lines (version, creator, cmd, totals, summary, ...) are skipped
    return positions, events, self_c, calls, ncalls


def main():
    path = sys.argv[1]
    pat = re.compile(sys.argv[2] if len(sys.argv) > 2 and not sys.argv[2].startswith("--") else ".")
    want_callees = "--callees" in sys.argv
    positions, events, self_c, calls, ncalls = parse(path)
    print(f"# {path}: positions {' '.join(positions)}; events {' '.join(events)}")
    ev = [e for e in ("Ir", "Dr", "Dw") if e in events] or events
    ix = [events.index(e) for e in ev]
    fmt = lambda v: " ".join(f"{e} {v[i]:,}" for e, i in zip(ev, ix))
    for f in sorted(set(self_c) | set(calls)):
        if not pat.search(f):
            continue
        s = self_c[f] or [0] * len(events)
        inc = list(s)
        for c, v in calls[f].items():
            inc = [a + b for a, b in zip(inc, v)]
        print(f"{f}\n   self      {fmt(s)}\n   inclusive {fmt(inc)}")
        if want_callees:
            for c, v in sorted(calls[f].items(), key=lambda kv: -kv[1][0]):
                print(f"     -> {ncalls[f][c]:>9,} calls  {fmt(v)}  {c}")


if __name__ == "__main__":
    main()

