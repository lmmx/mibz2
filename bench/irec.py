#!/usr/bin/env python3
"""Read the induction recorder's records (`irec/1`, `examples/induction_trace.rs`), and
replay the two orders of letters to any step. Standard library only.

    irec.py FILE.irec|DIR [--check]      a line a record; --check replays every step
                                         (also .irec.xz and .irec.gz)

In Python:

    import irec
    rec = irec.load("0000.irec")         # rec.head (dict), rec.a["s_a"] (array of u32) ...
    o = irec.Orders(rec)
    o.run_to(1000)                       # the state after 1000 steps
    top, bottom = o.top(), o.bottom()    # the live letters, left to right
    o.lam[x]                             # what is left of letter x, in rows

**The objects** (`src/itrace.rs` has them in full).
- **Letters:** 0..r-1 are the block's letters in top order. r + i is the cell of step i.
- **A step:** A is last on top and B last on the bottom (`s_a`, `s_b`, with `s_ra`, `s_rb`
  their rows left). The cell is r + i, of width min(ra, rb), and its word is A's then B's.
  - The cell lies right after B on top and right after A on the bottom.
  - A and B lose its width, and a letter at 0 leaves both orders.
  - `s_type`: 0 both end, 1 A wins (B ends), 2 B wins (A ends).
  - `s_heir`: the letter the cell continues, as classical Rauzy-Veech keeps it: the loser,
    or B at an equal step. Rename the cell by it to get the Rauzy-class permutation.
  - `s_fresh`: how the step's fronts were read for it. Bit 0 (1): A from its stream, a
    block's letter no front had reached. Bit 1 (2): B from its stream. Bit 2 (4): A from a
    slot. Bit 3 (8): B from a slot. A letter that goes on from the step before is not read.
- **The walk's row:** `row_letter` is the one-row letter cut out of the walk's run.
- **Closures:** at the start of a run of steps (the first, or one after an equal step or a
  closure), a letter last in both orders closes and leaves them (`c_step`, `c_letter`).
  The last to close is the marked one, `final`, whose word is the cycle's.
- **Words** (`w_*`): a letter's word.
  - Its length; its kind (0 byte, 1 arena, 2 rule, | 4 marked).
  - The cells it is a child of.
  - The step its letter was made at and the step it ended or closed at (`NONE` for neither).
"""

import array
import gzip
import json
import lzma
import os
import sys

NONE = 0xFFFFFFFF
EQUAL, TOP_WINS, BOTTOM_WINS = 0, 1, 2
FRESH_A, FRESH_B, SLOT_A, SLOT_B = 1, 2, 4, 8
BYTE, ARENA, RULE, MARKED = 0, 1, 2, 4


class Record:
    def __init__(self, head, arrays):
        self.head = head
        self.a = arrays

    def __getattr__(self, name):
        if name in ("head", "a"):
            raise AttributeError(name)
        try:
            return self.head[name]
        except KeyError:
            raise AttributeError(name) from None

    def winner_loser(self, i):
        """Step i's (winner, loser): None, None at an equal step"""
        t = self.a["s_type"][i]
        a, b = self.a["s_a"][i], self.a["s_b"][i]
        return {EQUAL: (None, None), TOP_WINS: (a, b), BOTTOM_WINS: (b, a)}[t]

    def children(self, x):
        """A cell's two letters (A, B), or None for a block's own letter"""
        r = self.head["r"]
        return None if x < r else (self.a["s_a"][x - r], self.a["s_b"][x - r])

    def word(self, x):
        """Letter x's word, as bytes (expanded: up to the block's length)"""
        r, out, stack = self.head["r"], bytearray(), [x]
        sa, sb, byte = self.a["s_a"], self.a["s_b"], self.a["l_byte"]
        while stack:
            y = stack.pop()
            if y < r:
                out.append(byte[y])
            else:
                stack.append(sb[y - r])
                stack.append(sa[y - r])
        return bytes(out)


def load(path):
    """A record, from `NNNN.irec`, or compressed as `.irec.xz` or `.irec.gz`"""
    opener = {".xz": lzma.open, ".gz": gzip.open}.get(os.path.splitext(path)[1], open)
    with opener(path, "rb") as f:
        data = f.read()
    nl = data.index(b"\n")
    head = json.loads(data[:nl])
    if head.get("format") != "irec/1":
        raise ValueError(f"{path}: not irec/1: {head.get('format')!r}")
    body = memoryview(data)[nl + 1 :]
    arrays = {}
    for name, (at, count) in head["arrays"].items():
        a = array.array("I")
        a.frombytes(body[at : at + 4 * count])
        if sys.byteorder != "little":
            a.byteswap()
        arrays[name] = a
    return Record(head, arrays)


class Orders:
    """The two orders and each letter's rows left, replayed from the record's steps"""

    def __init__(self, rec):
        self.rec = rec
        r, steps = rec.head["r"], rec.head["steps"]
        cap = r + steps
        self.r = r
        self.lam = array.array("I", rec.a["l_lam"]) + array.array("I", [0]) * steps
        self.tp, self.tn = [NONE] * cap, [NONE] * cap
        self.bp, self.bn = [NONE] * cap, [NONE] * cap
        top = range(r)
        bottom = rec.a["l_bottom"]
        for seq, prev, nxt in ((top, self.tp, self.tn), (bottom, self.bp, self.bn)):
            for x, y in zip(seq, list(seq)[1:]):
                nxt[x], prev[y] = y, x
        self.ttail, self.btail = (r - 1, bottom[-1]) if r else (NONE, NONE)
        self.thead, self.bhead = (0, bottom[0]) if r else (NONE, NONE)
        self.steps = 0  # steps taken
        self.closed = 0  # closures taken
        self.whole = True

    def _remove(self, x, prev, nxt, which):
        p, n = prev[x], nxt[x]
        if p != NONE:
            nxt[p] = n
        elif which == "t":
            self.thead = n
        else:
            self.bhead = n
        if n != NONE:
            prev[n] = p
        elif which == "t":
            self.ttail = p
        else:
            self.btail = p
        prev[x] = nxt[x] = NONE

    def _after(self, at, x, prev, nxt, which):
        n = nxt[at]
        nxt[at], prev[x], nxt[x] = x, at, n
        if n == NONE:
            if which == "t":
                self.ttail = x
            else:
                self.btail = x
        else:
            prev[n] = x

    def _close(self):
        """The closure due before the next step, if any, as the record has it"""
        c = self.rec.a
        while (
            self.closed < len(c["c_step"])
            and c["c_step"][self.closed] == self.steps
            and self.whole
            and self.ttail == self.btail
        ):
            x = self.ttail
            if c["c_letter"][self.closed] != x:
                raise AssertionError(
                    f"closure {self.closed}: replay {x}, record {c['c_letter'][self.closed]}"
                )
            self._remove(x, self.tp, self.tn, "t")
            self._remove(x, self.bp, self.bn, "b")
            self.closed += 1

    def step(self):
        """One step, checked against the record: its two letters must be the replay's fronts"""
        self._close()
        i = self.steps
        a, b = self.rec.a["s_a"][i], self.rec.a["s_b"][i]
        if (a, b) != (self.ttail, self.btail):
            raise AssertionError(
                f"step {i}: record ({a}, {b}), replay fronts ({self.ttail}, {self.btail})"
            )
        ra, rb = self.lam[a], self.lam[b]
        if (ra, rb) != (self.rec.a["s_ra"][i], self.rec.a["s_rb"][i]):
            raise AssertionError(f"step {i}: rows ({ra}, {rb}) against the record's")
        heir = a if ra < rb else b
        if self.rec.a["s_heir"][i] != heir:
            raise AssertionError(f"step {i}: heir {self.rec.a['s_heir'][i]}, replay {heir}")
        w = min(ra, rb)
        c = self.r + i
        self.lam[c] = w
        self._after(b, c, self.tp, self.tn, "t")
        self._after(a, c, self.bp, self.bn, "b")
        self.lam[a] -= w
        self.lam[b] -= w
        for x in (a, b):
            if self.lam[x] == 0:
                self._remove(x, self.tp, self.tn, "t")
                self._remove(x, self.bp, self.bn, "b")
        self.whole = ra == rb
        self.steps += 1

    def run_to(self, k):
        """The state after k steps (from here; replay is forward only)"""
        if k < self.steps:
            raise ValueError("the replay goes forward only: make a new Orders")
        while self.steps < k:
            self.step()

    def finish(self):
        """Every step, then the closures after the last, which must end with `final`"""
        self.run_to(self.rec.head["steps"])
        self._close()
        if self.closed != len(self.rec.a["c_step"]):
            raise AssertionError(
                f"{self.closed} closures replayed, {len(self.rec.a['c_step'])} recorded"
            )

    def _walk(self, head, nxt):
        out, x = [], head
        while x != NONE:
            out.append(x)
            x = nxt[x]
        return out

    def top(self):
        return self._walk(self.thead, self.tn)

    def bottom(self):
        return self._walk(self.bhead, self.bn)


def paths(arg):
    if os.path.isdir(arg):
        ends = (".irec", ".irec.xz", ".irec.gz")
        return sorted(os.path.join(arg, f) for f in os.listdir(arg) if f.endswith(ends))
    return [arg]


def main(argv):
    check = "--check" in argv
    args = [a for a in argv[1:] if a != "--check"]
    if not args:
        print(__doc__.split("\n\n")[1])
        return 2
    for p in paths(args[0]):
        rec = load(p)
        h = rec.head
        line = {
            k: h[k]
            for k in ("corpus", "block", "n", "r", "steps", "closures", "period", "admitted")
        }
        if check:
            o = Orders(rec)
            o.finish()
            line["checked"] = True
            if rec.word(h["final"]).__len__() != h["period"]:
                raise AssertionError("the final word's length")
        print(json.dumps(line))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
