"""Continue the pinned Study 7 induction trace into the pinned full-period W16 writer."""
import argparse
from collections import Counter
from pathlib import Path
import sys

from lines import Trace

JOURNAL = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(JOURNAL / "2026-11-18-cells-endpoint-sweep"))
import traffic
from sweep import ARENA, RULES, MARK, LOW, Words

BUF, DST, COUNTERS = (i << 32 for i in (5, 6, 7))


def owned(trace, words, n, arena, rules):
    """The scan in src/cells_owned.rs, including the marked spine and the position fill."""
    pairs = words.pairs
    nr = len(pairs)
    def rnum(w):
        return ((w >> 32) - RULES) // 16
    def length(w):
        return w & (MARK-1)
    def pair(j):
        trace.at(rules + 16*j, 16)
        return pairs[j]
    def pos_addr(j):
        return BUF+4096+4*j
    trace.at(pos_addr(0), 4*nr, True)
    j, mark = nr-1, 0
    while True:
        a, b = pair(j)
        if a & MARK:
            w = a
        else:
            mark += length(a)
            w = b
        if w >> 32 < RULES:
            break
        j = rnum(w)
    pos = [-1] * nr
    pos[-1] = (-(mark+1)) % n
    trace.at(pos_addr(nr-1), 4, True)
    later, queue = (-1, -1), []
    for j in reversed(range(nr)):
        a, b = pair(j)
        trace.at(pos_addr(j), 4)
        assert pos[j] >= 0
        for w, old, d in ((a,later[0],pos[j]), (b,later[1],(pos[j]+length(a))%n)):
            if w >> 32 < RULES:
                trace.at(arena + (w >> 32) - ARENA, length(w))
                trace.at(DST+d, length(w), True)
            elif w == old:
                queue.append((w, d))
                trace.at(rules+16*(nr-len(queue)), 16, True)
            else:
                pos[rnum(w)] = d
                trace.at(pos_addr(rnum(w)), 4, True)
        later = a, b
    for j, (w, d) in enumerate(reversed(queue), start=nr-len(queue)):
        trace.at(rules+16*j, 16)
        trace.at(pos_addr(rnum(w)), 4)
        trace.at(DST+pos[rnum(w)], length(w))
        trace.at(DST+d, length(w), True)
    return nr, len(queue)


def emit(rec, lazy, trace):
    budget = rec.n*(8 if lazy else 4)//25
    arena = BUF+4096+20*rec.r
    rules = arena+16+256+16+16*budget+32
    tby = 128+4*rec.r
    # The historical induction tracer uses independent logical bases. Map to the
    # production release offsets, including reuse of bottom storage as output.
    origins = {traffic.TOP: BUF, traffic.BOT: DST+(tby+rec.r-128 if lazy else 0),
               traffic.ENTRY: DST+128, traffic.TBYTE: DST+tby, traffic.COUNTER: COUNTERS,
               traffic.BYTE: arena, traffic.RULE: rules}
    class Memory:
        def __init__(self):
            self.count = Counter()
        def at(self, address, size=4, write=False, group="core"):
            base = (address >> 32) << 32
            trace.at(origins[base]+address-base, size, write)
        def report(self):
            return {}
    saved = []
    class Capture(Words):
        def __init__(self):
            super().__init__()
            saved.append(self)
    # Stock trace cancels common capped scans. Leave those free for Cells. Both
    # implementations get initialized reusable storage; all bytes written by
    # the word writer and the candidate are charged.
    original = traffic.Memory, traffic.Words
    try:
        traffic.Memory, traffic.Words = Memory, Capture
        trace.at(arena+16, 256, True)
        traffic.baseline(rec, lazy)
    finally:
        traffic.Memory, traffic.Words = original
    assert 4*len(saved[0].pairs) <= 20*rec.r  # pos indeed reuses the top slots
    trace.next_phase()
    return owned(trace, saved[0], rec.n, arena, rules)


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("records", type=Path)
    ap.add_argument("traces", type=Path)
    args = ap.parse_args()
    args.traces.mkdir(exist_ok=True, parents=True)
    for path in sorted(args.records.glob("*.irec*")):
        rec = traffic.irec.load(str(path))
        for mode in ("eager", "lazy"):
            with Trace(args.traces / f"{rec.block:04}-{mode}.bin") as trace:
                rules, copies = emit(rec, mode=="lazy", trace)
            print(rec.block, mode, rules, copies, flush=True)
