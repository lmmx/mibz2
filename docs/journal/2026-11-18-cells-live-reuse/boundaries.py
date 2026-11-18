#!/usr/bin/env python3
"""Check forwarding the previous-link field during a same-winner run in the lazy slot model."""
import argparse
import csv
import sys
from pathlib import Path

previous_study = Path(__file__).resolve().parent.parent / "2026-11-18-cells-mixed-steps"
sys.path.insert(0, str(previous_study))
import verify as base

irec = base.irec


class Forward(base.Machine):
    previous = None
    saved = None
    site = None
    switch_witness = None

    def ordinary(self, a, b, kind, word, x):
        if kind == base.E:
            self.previous = None
            return super().ordinary(a, b, kind, word, x)
        is_top = kind == base.T
        old = a.other if is_top else b.other
        slots = self.bots if is_top else self.tops
        if kind == self.previous:
            h = self.saved
            assert old == self.site and h == slots[old].prev  # audit-only field read
            self.cost["forwarded"] += 1
        else:
            h = slots[old].prev
            if self.previous is not None and self.saved != h and self.switch_witness is None:
                self.switch_witness = (x - self.r, self.saved, h)
        if is_top:
            self.cell(b.other, old, b.width, word, x)
            self.bots[b.slot].prev = h
            self.bots[old].prev, self.bots[old].other = b.slot, b.other
            a.other = self.site = b.slot
        else:
            self.bots[a.other].other = old
            self.tops[a.slot].prev = h
            self.tops[old].prev = a.slot
            self.cell(old, a.other, a.width, word, x)
            b.other = self.site = a.slot
        self.saved, self.previous = h, kind


def check(rec):
    v = rec.a
    state = Forward(v["l_lam"], v["l_byte"], v["l_bottom"], rec.row_letter, joint=False)
    audit = base.Audit(rec)
    final, (word, mark) = state.run(audit)
    assert audit.closed == rec.closures and final == rec.final and word == rec.word(final)
    x, q = final, 0
    while x >= rec.r:
        left, right = rec.children(x)
        if v["w_kind"][left] & irec.MARKED:
            x = left
        else:
            q += v["w_len"][left]
            x = right
    assert x == rec.row_letter and q == mark
    steps = v["s_type"]
    assert state.cost["forwarded"] == sum(a == b != base.E for a, b in zip(steps, steps[1:]))
    return (rec.corpus, rec.block, state.cost["forwarded"], state.switch_witness)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("records", nargs="+")
    args = parser.parse_args()
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(("corpus", "block", "forwarded", "wrong_side_example"))
    for group in args.records:
        for record in irec.paths(group):
            writer.writerow(check(irec.load(record)))


if __name__ == "__main__":
    main()
