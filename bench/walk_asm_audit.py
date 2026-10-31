"""Structural audit of the inverse-BWT room walk's hot step (no timing, no execution).

For each instantiation of a walk function (default: `bwt::room_segments`) in BINARY, the
hot path is traced, not read off an address span (LLVM may lay a loop out in pieces
around its cold blocks): for each backward branch, the path from its target that falls
through every other conditional branch (the walk's `START` tests are forward and not
taken in an ordinary step) and follows every unconditional jump, until it comes back to
that branch. A full-lane pass's path holds, for each 4-byte indexed load `[base+idx*4]`
(the T-vector load), one software prefetch and one byte store at a base plus the pass's
offset (`BYTE PTR [r+r*1]`); that excludes the setup's take loop (whose `starts` load also
matches `[r+r*4]`) and the tail (whose byte store is at `[p]`). Of those paths, the hot
path is the one with the fewest instructions per T-vector load. On it, counted per T-vector load (one per lane-step, so an unrolled
round normalises to the same unit):

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
BYTE_AT_OFFSET = re.compile(r"BYTE PTR \[r\w+\+r\w+\*1\],")
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


def trace(ins, back, limit=8000):
    """The path from `back`'s target, falling through other conditional branches and
    following unconditional jumps, until it returns to `back`; None if it doesn't"""
    at = {a: i for i, (a, _, _) in enumerate(ins)}
    t = target(ins[back][2])
    if t not in at:
        return None
    i, path = at[t], []
    while len(path) < limit:
        addr, mn, op = ins[i]
        path.append(ins[i])
        if i == back:
            return path
        if mn == "jmp":
            j = target(op)
            if j not in at:
                return None
            i = at[j]
            continue
        if mn.startswith("ret") or mn == "ud2":
            return None
        i += 1
        if i >= len(ins):
            return None
    return None


def hot_loop(ins):
    best, best_rate = None, None
    for i, (addr, mn, op) in enumerate(ins):
        if not (COND.match(mn) or mn == "jmp"):
            continue
        t = target(op)
        if t is None or t > addr:
            continue
        path = trace(ins, i)
        if not path:
            continue
        tl = sum(1 for _, m, o in path if m == "mov" and TLOAD.search(o))
        pf = sum(1 for _, m, _ in path if m.startswith("prefetch"))
        bs = sum(1 for _, m, o in path if m == "mov" and BYTE_AT_OFFSET.match(o))
        if tl == 0 or pf != tl or bs != tl:
            continue
        rate = len(path) / tl
        if best is None or rate < best_rate:
            best, best_rate = path, rate
    return best


def census(body):
    real = [x for x in body if not x[1].startswith("nop") and "nop" not in x[1] and x[1] not in ("data16", "cs")]
    real = [x for x in real if not (x[1] in ("data16", "cs") or "nop" in x[2].split()[:1])]
    c = dict(instructions=0, macro_ops=0, loads=0, stores=0, prefetches=0, stack_loads=0,
             stack_stores=0, conditional_branches=0, unconditional_jumps=0, calls=0, t_loads=0)
    prev_fusable = False
    for addr, mn, op in real:
        c["instructions"] += 1
        fused = prev_fusable and COND.match(mn)
        if not fused:
            c["macro_ops"] += 1
        prev_fusable = mn in FUSABLE and not MEM.search(op.split(",")[0] if "," in op else op)
        if COND.match(mn):
            c["conditional_branches"] += 1
        if mn == "jmp":
            c["unconditional_jumps"] += 1
        if mn.startswith("call"):
            c["calls"] += 1
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
            report[name] = dict(hot_path=None)
            continue
        c, per = census(body)
        report[name] = dict(hot_path=[hex(body[0][0]), hex(body[-1][0])], totals=c,
                            per_lane_step=per, unrolled=c["t_loads"] > 1)
        print(f"{name}: hot path {hex(body[0][0])} .. {hex(body[-1][0])}, {c['instructions']} instructions, {c['t_loads']} T load(s); per lane-step: "
              + ", ".join(f"{k} {v}" for k, v in per.items()))
    if a.out:
        json.dump(report, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
