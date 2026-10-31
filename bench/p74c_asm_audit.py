"""Structural assembly audit of the exact timed P74-C binary (no timing, no execution).

Replaces the failed zero/+1 instruction-count screen (confirmatory protocol §4). For each
variant (plain, checkpoints) it compares the P74 loop of `p74_c::combined` (T2's prefix)
with Gate 1's `p74::avx2::mtf[_checkpoints]::<true>` (the Free arm) in the SAME binary:

- the ordinary low-rank cycle: the shortest CFG cycle, in instructions (NOPs excluded),
  through a basic block that holds the Free head update (`vpermq ymmH,ymmH,0x4e`, two
  `vpshufb`, `vpblendvb`). That cycle is rank 1..31, no run to flush, no limit crossed.
- G1 conditional branches on the cycle: combined <= Free.
- G2 no YMM register stored to or loaded from the stack ([rsp]/[rbp]) on the cycle.
- G3 no new loop-carried dependency on MTF state: stack slots both loaded and stored on
  the cycle (memory RMW chains) combined <= Free; and the vector head-chain instructions
  (vpermq, vpshufb, vpblendvb, vmovdqa mask loads) are the same multiset.
- G4 the monitor is off the cycle: no `add ...,0x7b` (b = 123) and no `shr ...,0x5`.

Exits 1 if any gate fails. Usage: p74c_asm_audit.py BINARY [--out report.json]
"""
import argparse
import heapq
import json
import re
import subprocess
import sys
from collections import Counter

LINE = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")
FUNC = re.compile(r"^([0-9a-f]+) <(.*)>:$")
COND = re.compile(r"^j(?!mp)[a-z]+$")


def functions(binary):
    text = subprocess.check_output(
        ["objdump", "-d", "-C", "-Mintel", "--no-show-raw-insn", "-w", binary], text=True)
    funcs, cur = {}, None
    for line in text.splitlines():
        m = FUNC.match(line)
        if m:
            cur = funcs.setdefault(m.group(2), [])
            continue
        m = LINE.match(line)
        if m and cur is not None:
            cur.append((int(m.group(1), 16), m.group(2), m.group(3).split("#")[0].strip()))
    return funcs


def pick(funcs):
    """The four kernels by demangled name (legacy or v0 mangling)."""
    def named(pattern):
        return [k for k in funcs if re.search(pattern, k)]
    free_plain = named(r"mtf::p74::avx2::mtf(::<true[^>]*>)?$")
    free_cp = named(r"mtf::p74::avx2::mtf_checkpoints(::<true[^>]*>)?$")
    comb = named(r"mtf::p74_c::combined")
    assert len(free_plain) == 1 and len(free_cp) == 1, (free_plain, free_cp, "Free must be linked once")
    assert len(comb) == 2, comb
    has_cp = [any("0x4000" in ops for _, _, ops in funcs[k]) for k in comb]
    assert sorted(has_cp) == [False, True], "cannot tell the CP instantiation apart"
    c_cp, c_plain = (comb[0], comb[1]) if has_cp[0] else (comb[1], comb[0])
    return {"plain": (free_plain[0], c_plain), "checkpoints": (free_cp[0], c_cp)}


def target(ops):
    m = re.match(r"^([0-9a-f]+)\b", ops)
    return int(m.group(1), 16) if m else None


def cfg(ins):
    addrs = [a for a, _, _ in ins]
    inside = set(addrs)
    leaders = {addrs[0]}
    for i, (a, mn, ops) in enumerate(ins):
        if mn.startswith("j") or mn.startswith("ret"):
            t = target(ops)
            if t in inside:
                leaders.add(t)
            if i + 1 < len(ins):
                leaders.add(ins[i + 1][0])
    blocks, cur = {}, None
    for a, mn, ops in ins:
        if a in leaders:
            cur = blocks.setdefault(a, [])
        cur.append((a, mn, ops))
    order = sorted(blocks)
    succ = {}
    for i, b in enumerate(order):
        a, mn, ops = blocks[b][-1]
        nxt = order[i + 1] if i + 1 < len(order) else None
        s = []
        if mn == "jmp":
            t = target(ops)
            s = [t] if t in blocks else []
        elif COND.match(mn):
            t = target(ops)
            s = ([t] if t in blocks else []) + ([nxt] if nxt else [])
        elif not mn.startswith("ret") and not mn.startswith("ud"):
            s = [nxt] if nxt else []
        succ[b] = s
    return blocks, succ


def real(block):
    return [x for x in block if not x[1].startswith("nop") and x[1] not in ("int3", "xchg")]


def is_head_block(block):
    mns = [mn for _, mn, _ in block]
    vp = [ops for _, mn, ops in block if mn == "vpermq"]
    return any(o.endswith("0x4e") for o in vp) and mns.count("vpshufb") >= 2 and "vpblendvb" in mns


def shortest_cycle(blocks, succ, start):
    cost = {b: len(real(blocks[b])) for b in blocks}
    dist, prev, heap = {}, {}, []
    for s in succ[start]:
        heapq.heappush(heap, (cost[s], s, start))
    while heap:
        d, b, p = heapq.heappop(heap)
        if b in dist:
            continue
        dist[b], prev[b] = d, p
        if b == start:
            break
        for s in succ[b]:
            if s not in dist:
                heapq.heappush(heap, (d + cost[s], s, b))
    if start not in dist:
        return None
    path, b = [start], prev[start]
    while b != start:
        path.append(b)
        b = prev[b]
    path.reverse()
    return path


def cycle_of(ins):
    blocks, succ = cfg(ins)
    best = None
    for b in blocks:
        if is_head_block(blocks[b]):
            p = shortest_cycle(blocks, succ, b)
            if p:
                n = sum(len(real(blocks[x])) for x in p)
                if best is None or n < best[0]:
                    best = (n, p)
    assert best, "no low-rank head-update cycle found"
    return [x for b in best[1] for x in real(blocks[b])]


STACK = re.compile(r"\[(rsp|rbp)([+-]0x[0-9a-f]+)?\]")


def features(cycle):
    loads, stores, ymm_stack, chain = Counter(), Counter(), [], Counter()
    for a, mn, ops in cycle:
        parts = [p.strip() for p in ops.split(",")]
        for i, p in enumerate(parts):
            m = STACK.search(p)
            if not m:
                continue
            slot = m.group(0)
            if mn.startswith("v") and re.search(r"[xy]mm", ops):
                ymm_stack.append(f"{a:x}: {mn} {ops}")
            # Destination first in Intel syntax; cmp/test only read.
            if i == 0 and not mn.startswith(("cmp", "test")):
                stores[slot] += 1
                if mn.startswith(("add", "sub", "inc", "dec", "and", "or", "xor")):
                    loads[slot] += 1
            else:
                loads[slot] += 1
        if mn in ("vpermq", "vpshufb", "vpblendvb") or (mn == "vmovdqa" and "[" in ops and "ymm" in ops.split(",")[0]):
            chain[mn] += 1
    rmw = sorted(s for s in stores if loads[s])
    return dict(
        instructions=len(cycle),
        conditional_branches=sum(1 for _, mn, _ in cycle if COND.match(mn)),
        ymm_stack_accesses=ymm_stack,
        stack_rmw_slots=rmw,
        stack_loads=sum(loads.values()),
        stack_stores=sum(stores.values()),
        head_chain=dict(chain),
        monitor_on_cycle=[f"{a:x}: {mn} {ops}" for a, mn, ops in cycle
                          if (mn == "add" and ops.endswith("0x7b")) or (mn == "shr" and ops.endswith("0x5"))],
        listing=[f"{a:x}: {mn} {ops}" for a, mn, ops in cycle],
    )


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("binary")
    p.add_argument("--out")
    a = p.parse_args()
    funcs = functions(a.binary)
    report, ok = {}, True
    for variant, (free, comb) in pick(funcs).items():
        f, c = features(cycle_of(funcs[free])), features(cycle_of(funcs[comb]))
        gates = {
            "G1_branches": c["conditional_branches"] <= f["conditional_branches"],
            "G2_no_ymm_stack": not c["ymm_stack_accesses"],
            "G3_rmw": len(c["stack_rmw_slots"]) <= len(f["stack_rmw_slots"]),
            "G3_head_chain": c["head_chain"] == f["head_chain"],
            "G4_monitor_off_cycle": not c["monitor_on_cycle"],
        }
        ok &= all(gates.values())
        report[variant] = dict(free=dict(symbol=free, **f), combined=dict(symbol=comb, **c), gates=gates,
                               instruction_difference=c["instructions"] - f["instructions"])
        print(f"{variant}: Free {f['instructions']} instr/{f['conditional_branches']} jcc/"
              f"{len(f['stack_rmw_slots'])} rmw; C {c['instructions']}/{c['conditional_branches']}/"
              f"{len(c['stack_rmw_slots'])}; gates {gates}")
    report["pass"] = ok
    if a.out:
        with open(a.out, "w") as fh:
            json.dump(report, fh, indent=1)
    print("STRUCTURAL AUDIT", "PASS" if ok else "FAIL")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
