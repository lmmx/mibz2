"""Structural audit of the inverse-BWT room walk's hot step (no timing, no execution).

For each instantiation of a walk function (default: `bwt::room_segments`) in BINARY, the
hot loop is the smallest backward branch's span (a jump to an address at or below its own)
that holds a software prefetch and a 4-byte indexed load `[base+idx*4]` (the T-vector
load). Within it, counted per T-vector load (one per lane-step, so an unrolled step
normalises to the same unit):

- instructions (NOPs excluded) and macro-ops (a `test`/`cmp`/`and`/`add`/`sub`/`inc`/`dec`
  directly followed by a conditional jump counted once, as Zen and Intel cores fuse them);
- loads (memory source operands, excluding `lea` and prefetches), stores (memory
  destinations), prefetches, and the stack accesses among them (`rsp`/`rbp` based);
- conditional branches, and whether the loop body holds more than one T-vector load
  (an unrolled lane loop).

A heuristic reader, not a proof: its output is a signature to compare with an expected
one, stated before timing. Usage: walk_asm_audit.py BINARY [--symbol SUBSTR] [--out FILE]
"""
import argparse
import json
import re
import subprocess

LINE = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")
FUNC = re.compile(r"^([0-9a-f]+) <(.*)>:$")
COND = re.compile(r"^j(?!mp)[a-z]+$")
FUSABLE = {"test", "cmp", "and", "add", "sub", "inc", "dec"}
TLOAD = re.compile(r"DWORD PTR \[(r\w+)\+(r\w+)\*4\]")
MEM = re.compile(r"(BYTE|WORD|DWORD|QWORD|XMMWORD|YMMWORD) PTR \[([^\]]+)\]")


def functions(binary, symbol):
    text = subprocess.check_output(
        ["objdump", "-d", "-C", "-Mintel", "--no-show-raw-insn", "-w", binary], text=True)
    funcs, cur = {}, None
    for line in text.splitlines():
        m = FUNC.match(line)
        if m:
            cur = funcs.setdefault(f"{m.group(2)}@{m.group(1).lstrip('0')}", []) if symbol in m.group(2) else None
            continue
        m = LINE.match(line)
        if m and cur is not None:
            cur.append((int(m.group(1), 16), m.group(2), m.group(3).split("#")[0].strip()))
    return funcs


def target(op):
    m = re.match(r"([0-9a-f]+)\b", op)
    return int(m.group(1), 16) if m else None


def hot_loop(ins):
    best = None
    for i, (addr, mn, op) in enumerate(ins):
        if not (COND.match(mn) or mn == "jmp"):
            continue
        t = target(op)
        if t is None or t > addr:
            continue
        body = [x for x in ins if t <= x[0] <= addr]
        if any(m.startswith("prefetch") for _, m, _ in body) and any(TLOAD.search(o) for _, m, o in body if m == "mov"):
            if best is None or len(body) < len(best):
                best = body
    return best


def census(body):
    real = [x for x in body if not x[1].startswith("nop") and "nop" not in x[1] and x[1] not in ("data16", "cs")]
    real = [x for x in real if not (x[1] in ("data16", "cs") or "nop" in x[2].split()[:1])]
    c = dict(instructions=0, macro_ops=0, loads=0, stores=0, prefetches=0, stack_loads=0,
             stack_stores=0, conditional_branches=0, t_loads=0)
    prev_fusable = False
    for addr, mn, op in real:
        c["instructions"] += 1
        fused = prev_fusable and COND.match(mn)
        if not fused:
            c["macro_ops"] += 1
        prev_fusable = mn in FUSABLE and not MEM.search(op.split(",")[0] if "," in op else op)
        if COND.match(mn):
            c["conditional_branches"] += 1
        if mn.startswith("prefetch"):
            c["prefetches"] += 1
            continue
        if mn == "lea" or mn.startswith("j") or mn in ("nop",):
            continue
        parts = [p.strip() for p in op.split(",")]
        if mn == "mov" and TLOAD.search(op) and len(parts) == 2 and not MEM.search(parts[0]):
            c["t_loads"] += 1
        if parts and MEM.search(parts[0]) and mn not in ("cmp", "test"):
            c["stores"] += 1
            c["stack_stores"] += bool(re.search(r"\[(rsp|rbp)", parts[0]))
            if mn not in ("mov", "movzx") and not mn.startswith("vmov"):
                c["loads"] += 1  # read-modify-write
                c["stack_loads"] += bool(re.search(r"\[(rsp|rbp)", parts[0]))
        for p in parts[1:] if len(parts) > 1 else []:
            if MEM.search(p):
                c["loads"] += 1
                c["stack_loads"] += bool(re.search(r"\[(rsp|rbp)", p))
        if len(parts) == 1 and MEM.search(parts[0]) and mn in ("cmp", "test"):
            c["loads"] += 1
        if len(parts) == 2 and mn in ("cmp", "test") and MEM.search(parts[0]):
            c["loads"] += 1
    n = max(c["t_loads"], 1)
    per = {k: round(v / n, 3) for k, v in c.items() if k != "t_loads"}
    return c, per


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("binary")
    ap.add_argument("--symbol", default="bwt::room_segments")
    ap.add_argument("--out")
    a = ap.parse_args()
    report = {}
    for name, ins in sorted(functions(a.binary, a.symbol).items()):
        body = hot_loop(ins)
        if body is None:
            report[name] = dict(hot_loop=None)
            continue
        c, per = census(body)
        report[name] = dict(hot_loop=[hex(body[0][0]), hex(body[-1][0])], totals=c,
                            per_lane_step=per, unrolled=c["t_loads"] > 1)
        print(f"{name}: loop {hex(body[0][0])}-{hex(body[-1][0])}, {c['t_loads']} T load(s); per lane-step: "
              + ", ".join(f"{k} {v}" for k, v in per.items()))
    if a.out:
        json.dump(report, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
