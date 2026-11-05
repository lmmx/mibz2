"""Parse a callgrind.out file: self cost by function and by (file, line), every event.

usage: python3 -I cgparse.py OUT [--lines FILESUBSTR] [--top N]
"""
import re
import sys
from collections import defaultdict


def parse(path):
    events = None
    names = {"fl": {}, "fn": {}, "cfn": {}, "cfl": {}}
    fl = fi = fn = None
    by_fn = defaultdict(lambda: None)
    by_line = defaultdict(lambda: None)
    global by_fnline
    by_fnline = defaultdict(lambda: None)
    totals = None
    skip_next = False
    sub = {"fl": "fl", "fi": "fl", "fe": "fl", "fn": "fn", "cfn": "fn", "cfl": "fl", "cfi": "fl"}

    def name(kind, rest):
        m = re.match(r"\((\d+)\)(?: (.*))?", rest)
        if not m:
            return rest
        k = sub[kind]
        if m.group(2) is not None:
            names[k][m.group(1)] = m.group(2)
        return names[k].get(m.group(1), "?")

    def add(d, key, vals):
        cur = d[key]
        if cur is None:
            d[key] = list(vals)
        else:
            for i, v in enumerate(vals):
                cur[i] += v

    last_line = 0
    for raw in open(path):
        line = raw.rstrip("\n")
        if line.startswith("events:"):
            events = line.split()[1:]
            totals = [0] * len(events)
            continue
        m = re.match(r"^(fl|fi|fe|fn|cfn|cfl|cfi)=(.*)$", line)
        if m:
            kind, rest = m.groups()
            nm = name(kind, rest)
            if kind == "fl":
                fl = fi = nm
            elif kind in ("fi", "fe"):
                fi = nm
            elif kind == "fn":
                fn = nm
                fi = fl
            continue
        if line.startswith("calls="):
            skip_next = True
            continue
        if not line or not (line[0].isdigit() or line[0] in "+-*"):
            continue
        parts = line.split()
        p0 = parts[0]
        if p0.startswith("+"):
            ln = last_line + int(p0[1:])
        elif p0.startswith("-"):
            ln = last_line - int(p0[1:])
        elif p0 == "*":
            ln = last_line
        else:
            ln = int(p0)
        last_line = ln
        vals = [int(x) for x in parts[1:]] + [0] * (len(events) - len(parts[1:]))
        if skip_next:
            skip_next = False
            continue
        add(by_fn, fn, vals)
        add(by_line, (fi, ln), vals)
        add(by_fnline, (fn, fi, ln), vals)
        for i, v in enumerate(vals):
            totals[i] += v
    return events, totals, by_fn, by_line


def main():
    path = sys.argv[1]
    sub = None
    top = 25
    if "--lines" in sys.argv:
        sub = sys.argv[sys.argv.index("--lines") + 1]
    if "--top" in sys.argv:
        top = int(sys.argv[sys.argv.index("--top") + 1])
    events, totals, by_fn, by_line = parse(path)
    print("events", " ".join(events))
    print("TOTAL", " ".join(str(x) for x in totals))
    ir = events.index("Ir")
    for f, v in sorted(by_fn.items(), key=lambda kv: -kv[1][ir])[:top]:
        print("FN", " ".join(str(x) for x in v), "|", f)
    if sub:
        for (f, ln), v in sorted(by_line.items(), key=lambda kv: (str(kv[0][0]), kv[0][1])):
            if f and sub in f and any(v):
                print("LN", f.split("/")[-1], ln, " ".join(str(x) for x in v))


if __name__ == "__main__":
    main()
