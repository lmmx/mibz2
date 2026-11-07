"""Rule A's plans on a grid, from plan_tables.py's `rule_a`, for the Rust implementation's
test (src/parallel/general.rs, `rule_a_matches_plan_tables`), which reads the file this
writes:

    python3 rule_a_grid.py > ../../../tests/data/rule_a_grid.txt

The grid: physical cores 1-64; the L3 unread, 2-128 MiB in 1 MiB steps, and the edge
values 19.25 MiB and 20 MiB +/- 1 byte; the L3 shared by every usable core and of one
kind, or not; threads asked for 1 to 2 x cores.

The whole table is one line a topology, its plans over the threads as runs,
`PHYS L3 SHARED  N0-N1:PLAN ...`, where L3 is bytes or `-` and PLAN is `w` (as many
workers as threads asked for), `TxK` (plain teams) or `TxKp` (pipelined, with park and
small). It is 1.7 MB, so the file holds its FNV-1a 64 digest and line count, which the
Rust test recomputes from its own rule A, and, readable, every topology where the fast plan
fires: `PHYS L3 N0 T`, rule A running T x 3 pipelined from N0 threads to 2 x PHYS (the fast
plan, once it fires, holds as the threads rise). Pure arithmetic, as plan_tables.py is.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from plan_tables import MIB, rule_a  # noqa: E402


def code(plan):
    if plan[0] == "workers":
        return "w"  # (its count is the threads asked for, always)
    _, t, k, piped = plan
    return f"{t}x{k}" + ("p" if piped else "")


L3S = [None] + [m * MIB for m in range(2, 129)] + [77 * MIB // 4, 20 * MIB - 1, 20 * MIB + 1]


def fnv1a64(data):
    h = 0xCBF29CE484222325
    for b in data:
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def full():
    """the whole table, its lines"""
    out = []
    for phys in range(1, 65):
        for l3 in L3S:
            for shared in (True, False):
                runs = []
                for n in range(1, 2 * phys + 1):
                    c = code(rule_a(n, phys, l3, shared=shared))
                    if runs and runs[-1][2] == c:
                        runs[-1][1] = n
                    else:
                        runs.append([n, n, c])
                cells = " ".join(f"{a}-{b}:{c}" for a, b, c in runs)
                out.append(f"{phys} {'-' if l3 is None else l3} {int(shared)}  {cells}")
                if shared and runs[-1][2].endswith("p"):
                    a, b, c = runs[-1]
                    assert b == 2 * phys and not any(r[2].endswith("p") for r in runs[:-1])
                    FAST.append(f"{phys} {l3} {a} {c[:-3]}")
    return out


FAST = []


def main():
    lines = full()
    text = "".join(line + "\n" for line in lines).encode()
    print("# rule A (plan_tables.py rule_a) on a grid; written by rule_a_grid.py")
    print(f"lines {len(lines)}")
    print(f"fnv1a64 {fnv1a64(text):016x}")
    print("# where the fast plan fires: PHYS L3 N0 T (T x 3 pipelined from N0 threads to 2 x PHYS)")
    for f in FAST:
        print(f)


if __name__ == "__main__":
    main()
