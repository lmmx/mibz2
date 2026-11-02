#!/usr/bin/env python3
"""Whether a binary's `C30::room::<false>` is the reviewed listing (gate G3 of
docs/journal/2026-11-02-compact30-traversal-protocol.md).

    python3 bench/asm_listing.py BINARY REFERENCE.asm OUT_DIR

Disassembles BINARY (nm and objdump, Intel syntax, no raw bytes) from the function's
symbol to the next symbol into OUT_DIR/room-false.asm, then compares it with
REFERENCE.asm, both normalised:
- the addresses are dropped;
- a branch into the function becomes its offset from the function's start;
- a call or branch out of it becomes the callee's symbol;
- RIP-relative displacements and objdump's comments are masked.

The function is the symbol's own extent, [start, start + size), with the size from the
symbol table (`nm -S`, the ELF st_size the compiler emits as the function's `.size`). Every
instruction in that extent is compared, in order, with its operands, the function's own
alignment NOPs included. The size must equal the reviewed function's, REFERENCE_SIZE.

The bytes from the function's end to the next symbol are not the function: they are the
linker's alignment fill, and differ between linkers (`int3` from one, a multi-byte `nop`
from another; report 182). They are not compared. Each must be an alignment instruction
(`int3`, a `nop` of any width or prefix, `xchg ax,ax`); anything else there fails G3, as
does a missing instruction boundary at the function's end. The reference's own tail is
held to the same rule.

It writes OUT_DIR/room-false.diff (empty when identical) and OUT_DIR/room-false.bounds
(the extent and the tails), and prints `yes` (identical) or `no`; the exit status is 0
either way, and non-zero only if the function cannot be found.
"""

import re
import subprocess
import sys

SYMBOL = "C304roomKb0_"  # the mangled <c30walk::C30>::room::<false>

# The reviewed function's st_size. The reference listing records addresses, not the symbol
# table, so its size is recorded here: the VM's rustc 1.97.0 build of the pin (binary
# sha256 82c3a685...), whose listing is the reference instruction for instruction, has
# `nm -S` size 0x20aa, and the reference's function starts at 0x14ed0 and its fill at
# 0x16f7a = 0x14ed0 + 0x20aa.
REFERENCE_SIZE = 0x20AA

INSN = re.compile(r"^ *([0-9a-f]+):\t(.*)$")
FILL = re.compile(r"^(int3|(data16 )*(cs )?nop( .*)?|xchg ax,ax)$")


def instructions(text):
    """(address, instruction text) for each instruction line; objdump's comments dropped.
    A line that is neither an instruction nor part of objdump's header is kept as
    (None, line), so an elided run (`...`) or anything unexpected is not lost."""
    out, started = [], False
    for line in text.splitlines():
        m = INSN.match(line)
        if m:
            started = True
            out.append((int(m.group(1), 16), re.sub(r"#.*$", "", m.group(2)).rstrip()))
        elif started and line.strip() and not line.endswith(">:"):
            out.append((None, line.strip()))
    return out


def split(text, size):
    """The function's instructions, [start, start + size), and the rest (the tail), with
    the problems that make the split untrustworthy."""
    ins = instructions(text)
    problems = []
    if not ins or ins[0][0] is None:
        return [], [], ["no instructions"], 0
    start = ins[0][0]
    end = start + size
    cut = next((i for i, (a, _) in enumerate(ins) if a is not None and a >= end), len(ins))
    body, tail = ins[:cut], ins[cut:]
    for a, t in body:
        if a is None:
            problems.append(f"an unrecognised line inside the function: {t!r}")
    body = [(a, t) for a, t in body if a is not None]
    if tail and tail[0][0] != end:
        problems.append(f"no instruction starts at the function's end, 0x{end:x}")
    for a, t in tail:
        if a is None or not FILL.match(" ".join(t.split())):
            where = "?" if a is None else f"0x{a:x}"
            problems.append(f"not alignment fill after the function's end: {where} {t!r}")
    return body, tail, problems, start


def normalise(body, start, size):
    def target(m):
        a = int(m.group(1), 16)
        return "T%+d" % (a - start) if start <= a < start + size else "SYM<%s>" % m.group(2)

    out = []
    for _, ins in body:
        ins = re.sub(r"\b([0-9a-f]+) <([^>+]*)(\+0x[0-9a-f]+)?>", target, ins)
        ins = re.sub(r"rip\+0x[0-9a-f]+", "rip+DISP", ins)
        out.append(ins)
    return out


def compare(got_text, got_size, ref_text, ref_size=REFERENCE_SIZE):
    """(diff lines, bounds lines). G3 passes when the diff is empty."""
    diff = []
    if got_size != ref_size:
        diff.append(f"size: reference 0x{ref_size:x} / this binary 0x{got_size:x} bytes")
    g_body, g_tail, g_problems, g_start = split(got_text, got_size)
    r_body, r_tail, r_problems, r_start = split(ref_text, ref_size)
    diff += [f"this binary: {p}" for p in g_problems]
    diff += [f"reference: {p}" for p in r_problems]
    got, want = normalise(g_body, g_start, got_size), normalise(r_body, r_start, ref_size)
    diff += [
        f"{i}: reference {a!r} / this binary {b!r}"
        for i, (a, b) in enumerate(zip(want, got))
        if a != b
    ]
    if len(got) != len(want):
        diff.append(f"length: reference {len(want)} / this binary {len(got)} instructions")

    def tail(name, start, size, body, t):
        kinds = [" ".join(x.split()) for _, x in t]
        fill = ", ".join(f"{k} x{kinds.count(k)}" for k in sorted(set(kinds))) or "nothing"
        return [
            f"{name}: function 0x{start:x} + 0x{size:x} = 0x{start + size:x}, "
            f"{len(body)} instructions compared",
            f"{name}: after the function, not compared: {fill}",
        ]

    bounds = tail("this binary", g_start, got_size, g_body, g_tail)
    bounds += tail("reference", r_start, ref_size, r_body, r_tail)
    return diff, bounds


def main():
    binary, reference, out_dir = sys.argv[1:4]
    syms = subprocess.run(["nm", "-S", "-n", binary], capture_output=True, text=True, check=True)
    rows = [l.split() for l in syms.stdout.splitlines()]
    at = [i for i, r in enumerate(rows) if len(r) >= 3 and SYMBOL in r[-1]]
    if len(at) != 1 or len(rows[at[0]]) != 4:
        print(f"asm_listing: {len(at)} sized symbols match {SYMBOL}", file=sys.stderr)
        return 2
    start, size = int(rows[at[0]][0], 16), int(rows[at[0]][1], 16)
    stop = next(int(r[0], 16) for r in rows[at[0] + 1 :] if len(r) >= 3 and int(r[0], 16) > start)
    asm = subprocess.run(
        ["objdump", "-d", "--no-show-raw-insn", "-M", "intel",
         f"--start-address=0x{start:x}", f"--stop-address=0x{stop:x}", binary],
        capture_output=True, text=True, check=True).stdout  # fmt: skip
    with open(f"{out_dir}/room-false.asm", "w") as f:
        f.write(asm)
    diff, bounds = compare(asm, size, open(reference).read())
    bounds.insert(0, f"this binary: next symbol at 0x{stop:x} (nm -S -n)")
    with open(f"{out_dir}/room-false.diff", "w") as f:
        f.write("".join(d + "\n" for d in diff))
    with open(f"{out_dir}/room-false.bounds", "w") as f:
        f.write("".join(b + "\n" for b in bounds))
    print("yes" if not diff else "no")
    return 0


if __name__ == "__main__":
    sys.exit(main())
