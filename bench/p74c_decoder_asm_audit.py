"""Structural assembly audit of the integrated P74-C decoder binary (no timing, no execution).

bench/p74c_asm_audit.py (8b6caf1) adapted to the integrated binary
(docs/journal/2026-10-31-p74c-decoder-protocol.md §9). Gate 1's Free kernel is no longer
linked, so the ordinary low-rank cycle of each `p74_c::combined` instantiation (plain:
`::<0>`, checkpoints: `::<16384>`) is compared with FROZEN REFERENCE counts: the 8b6caf1
audits (identical on the three CI hosts of run 41190706676 and the i9-10900K, rustc
1.90.0). The cycle-finding and feature code below is the original's, unchanged.

- the ordinary low-rank cycle: the shortest CFG cycle, in instructions (NOPs excluded),
  through a basic block that holds the head update (`vpermq ymmH,ymmH,0x4e`, two
  `vpshufb`, `vpblendvb`): rank 1..31, no run to flush, no limit crossed.
- G1 conditional branches on the cycle <= Free's reference (no extra low-rank branch).
- G2 no YMM register stored to or loaded from the stack on the cycle.
- G3 stack slots both loaded and stored on the cycle (memory RMW chains) <= Free's
  reference, and the head-chain multiset equal to the reference (no new loop-carried
  dependency on MTF state).
- G4 the monitor is off the cycle: no `add ...,0x7b` (b = 123) and no `shr ...,0x5`.

Instruction counts are reported against the pin's combined counts (the +1/+1 over Free
was waived by the stage protocol's §4; not a gate). Exits 1 if any gate fails.
Usage: p74c_decoder_asm_audit.py BINARY [--out report.json]
"""
import argparse
import heapq
import json
import re
import subprocess
import sys
from collections import Counter

# The frozen references (8b6caf1, rustc 1.90.0; docs/journal/2026-10-31-p74c-ci-41190706676
# r1-r3 and 2026-10-31-p74c-i9-10900K asm-audit.json, identical)
HEAD_CHAIN = {"vpermq": 1, "vpshufb": 2, "vmovdqa": 1, "vpblendvb": 1}
REFERENCE = {
    "plain": dict(free=dict(instructions=37, conditional_branches=6, stack_rmw_slots=0),
                  combined_at_pin=dict(instructions=38, conditional_branches=6, stack_rmw_slots=0)),
    "checkpoints": dict(free=dict(instructions=38, conditional_branches=6, stack_rmw_slots=2),
                        combined_at_pin=dict(instructions=39, conditional_branches=6, stack_rmw_slots=0)),
}


LINE = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")
FUNC = re.compile(r"^([0-9a-f]+) <(.*)>:$")
COND = re.compile(r"^j(?!mp)[a-z]+$")


def functions(binary):
    text = subprocess.check_output(
        ["objdump", "-d", "-C", "-Mintel", "--no-show-raw-insn", "-w", binary], text=True)
    # Keyed by "name@address": legacy demangling gives both instantiations one name.
    funcs, cur = {}, None
    for line in text.splitlines():
        m = FUNC.match(line)
        if m:
            cur = funcs.setdefault(f"{m.group(2)}@{m.group(1).lstrip('0')}", [])
            continue
        m = LINE.match(line)
        if m and cur is not None:
            cur.append((int(m.group(1), 16), m.group(2), m.group(3).split("#")[0].strip()))
    return funcs


def pick(funcs):
    """The two combined instantiations, told apart by the checkpoint interval (0x4000), and
    the shipped kernels' symbols (present, unaudited: their code is master's)."""
    def named(pattern):
        return [k for k in funcs if re.search(pattern, k)]
    comb = named(r"mtf::p74_c::combined")
    assert len(comb) == 2, comb
    has_cp = [any("0x4000" in ops for _, _, ops in funcs[k]) for k in comb]
    assert sorted(has_cp) == [False, True], "cannot tell the CP instantiation apart"
    c_cp, c_plain = (comb[0], comb[1]) if has_cp[0] else (comb[1], comb[0])
    shipped = {name: named(rf"mtf::{name}@") for name in ("mtf_avx2", "mtf_checkpoints_avx2", "mtf_scalar", "mtf_checkpoints_scalar")}
    assert all(len(v) == 1 for v in shipped.values()), shipped
    # Gate 1's own kernels must be gone from the library
    assert not named(r"mtf::p74::avx2::mtf"), "Gate 1's kernels are linked"
    return {"plain": c_plain, "checkpoints": c_cp}, {k: v[0] for k, v in shipped.items()}


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
    kernels, shipped = pick(funcs)
    report, ok = dict(shipped_symbols=shipped, reference=REFERENCE), True
    for variant, comb in kernels.items():
        c = features(cycle_of(funcs[comb]))
        ref = REFERENCE[variant]
        gates = {
            "G1_branches": c["conditional_branches"] <= ref["free"]["conditional_branches"],
            "G2_no_ymm_stack": not c["ymm_stack_accesses"],
            "G3_rmw": len(c["stack_rmw_slots"]) <= ref["free"]["stack_rmw_slots"],
            "G3_head_chain": c["head_chain"] == HEAD_CHAIN,
            "G4_monitor_off_cycle": not c["monitor_on_cycle"],
        }
        ok &= all(gates.values())
        report[variant] = dict(combined=dict(symbol=comb, **c), gates=gates,
                               instructions_vs_pin=c["instructions"] - ref["combined_at_pin"]["instructions"],
                               instructions_vs_free_reference=c["instructions"] - ref["free"]["instructions"],
                               same_as_pin=dict(instructions=c["instructions"] == ref["combined_at_pin"]["instructions"],
                                                branches=c["conditional_branches"] == ref["combined_at_pin"]["conditional_branches"],
                                                rmw=len(c["stack_rmw_slots"]) == ref["combined_at_pin"]["stack_rmw_slots"]))
        print(f"{variant}: combined {c['instructions']} instr/{c['conditional_branches']} jcc/"
              f"{len(c['stack_rmw_slots'])} rmw (pin {ref['combined_at_pin']['instructions']}/"
              f"{ref['combined_at_pin']['conditional_branches']}/{ref['combined_at_pin']['stack_rmw_slots']}; "
              f"Free reference {ref['free']['instructions']}/{ref['free']['conditional_branches']}/"
              f"{ref['free']['stack_rmw_slots']}); gates {gates}")
    report["pass"] = ok
    if a.out:
        with open(a.out, "w") as fh:
            json.dump(report, fh, indent=1)
    print("STRUCTURAL AUDIT", "PASS" if ok else "FAIL")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
