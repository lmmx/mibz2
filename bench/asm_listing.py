#!/usr/bin/env python3
"""Whether a binary's `C30::room::<false>` is the reviewed listing (gate G3 of
docs/journal/2026-11-02-compact30-traversal-protocol.md).

    python3 bench/asm_listing.py BINARY REFERENCE.asm OUT_DIR

Disassembles the function from BINARY (nm and objdump, Intel syntax, no raw bytes) into
OUT_DIR/room-false.asm, then compares it with REFERENCE.asm, both normalised:
- the addresses are dropped;
- a branch into the function becomes its offset from the function's start;
- a call or branch out of it becomes the callee's symbol;
- RIP-relative displacements and objdump's comments are masked.

So the comparison is every instruction, in order, with its operands, independent of where
the linker placed the function and its constants. It writes OUT_DIR/room-false.diff (empty
when identical) and prints `yes` (identical) or `no`; the exit status is 0 either way, and
non-zero only if the function cannot be found.
"""

import re
import subprocess
import sys

SYMBOL = "C304roomKb0_"  # the mangled <c30walk::C30>::room::<false>


def normalise(text):
    lines = [l for l in text.splitlines() if re.match(r"^ *[0-9a-f]+:\t", l)]
    base = int(lines[0].split(":")[0], 16)
    end = int(lines[-1].split(":")[0], 16)

    def target(m):
        a = int(m.group(1), 16)
        return "T%+d" % (a - base) if base <= a <= end else "SYM<%s>" % m.group(2)

    out = []
    for l in lines:
        ins = l.split("\t", 1)[1].rstrip()
        ins = re.sub(r"#.*$", "", ins).rstrip()
        ins = re.sub(r"\b([0-9a-f]+) <([^>+]*)(\+0x[0-9a-f]+)?>", target, ins)
        ins = re.sub(r"rip\+0x[0-9a-f]+", "rip+DISP", ins)
        out.append(ins)
    return out


def main():
    binary, reference, out_dir = sys.argv[1:4]
    syms = subprocess.run(["nm", "-n", binary], capture_output=True, text=True, check=True)
    rows = [l.split() for l in syms.stdout.splitlines() if len(l.split()) == 3]
    at = [i for i, r in enumerate(rows) if SYMBOL in r[2]]
    if len(at) != 1:
        print(f"asm_listing: {len(at)} symbols match {SYMBOL}", file=sys.stderr)
        return 2
    start = rows[at[0]][0]
    stop = next(r[0] for r in rows[at[0] + 1 :] if r[0] != start)
    asm = subprocess.run(
        ["objdump", "-d", "--no-show-raw-insn", "-M", "intel",
         f"--start-address=0x{start}", f"--stop-address=0x{stop}", binary],
        capture_output=True, text=True, check=True).stdout  # fmt: skip
    with open(f"{out_dir}/room-false.asm", "w") as f:
        f.write(asm)
    got, want = normalise(asm), normalise(open(reference).read())
    diff = [
        f"{i}: reference {a!r} / this binary {b!r}"
        for i, (a, b) in enumerate(zip(want, got))
        if a != b
    ]
    if len(got) != len(want):
        diff.append(f"length: reference {len(want)} / this binary {len(got)} instructions")
    with open(f"{out_dir}/room-false.diff", "w") as f:
        f.write("".join(d + "\n" for d in diff))
    print("yes" if not diff else "no")
    return 0


if __name__ == "__main__":
    sys.exit(main())
