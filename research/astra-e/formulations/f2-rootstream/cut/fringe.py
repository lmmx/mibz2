"""A constant-radius selector-only certificate at parents of terminal leaves."""
from fractions import Fraction
import csv,sys
from frontier import load

def choose(m,penalty):
    cuts = set()
    lower = 0
    for v in range(1,m.n):
        kids = m.children[v]
        if not kids or any(m.children[c] for c in kids):
            continue
        build = m.sel[v] + len(kids)
        # Every selection of v from its parent produces at least one root run.
        # At most terminal_width such runs can be self-terminal occurrences.
        guaranteed = m.rows[v]["incoming_pieces"]-m.rows[v]["terminal_width"]-build
        if guaranteed > penalty*build:
            cuts.add(v)
            lower += guaranteed
    return cuts,lower

if __name__ == "__main__":
    w = csv.writer(sys.stdout,lineterminator="\n")
    w.writerow(["block","penalty","cuts","extra_pieces","work","saved","certified_saving"])
    for b,m in load(sys.argv[1]).items():
        for penalty in (Fraction(0),Fraction(1),Fraction(4)):
            cuts,lower = choose(m,penalty)
            a,c,writes,mem,_ = m.count(cuts)
            work = a+c+writes
            assert sum(m.g)-work >= lower
            w.writerow([b,penalty,len(cuts),mem,work,sum(m.g)-work,lower])
