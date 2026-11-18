"""Only the comparator may read recorded cuts. Join its setup, induction and writer."""
import argparse
import csv
import sys
from collections import Counter
from pathlib import Path
from unittest.mock import patch

from inputs import paths
from lines import Traffic

HERE = Path(__file__).resolve().parent
JOURNAL = HERE.parent
SWEEP = JOURNAL / "2026-11-18-cells-endpoint-sweep"
sys.path.insert(0, str(SWEEP))
import traffic as old
from sweep import ARENA, MARK, RULES, Words

# Aligned allocation bases for capped input, kernel Vec, output Vec and counters.
RAW, KERNEL, OUTPUT, COUNTERS = (i << 30 for i in range(4))


class Layout:
    def __init__(self, rec, lazy):
        self.rec, self.lazy = rec, lazy
        self.r = rec.r
        self.arena = KERNEL + 4096 + 20*rec.r
        budget = (8 if lazy else 4) * rec.n // 25
        self.rule = self.arena + 16 + 256 + 16 + 16*budget + 32
        self.bases = {
            old.TOP: KERNEL, old.BOT: OUTPUT + (5*rec.r if lazy else 0),
            old.ENTRY: OUTPUT+128, old.TBYTE: OUTPUT+128+4*rec.r,
            old.COUNTER: COUNTERS, old.BYTE: self.arena, old.RULE: self.rule,
        }
        self.memory = Traffic()
        self.count = Counter()
        self.scatter = []
        self.scanned = 0

    def at(self, addr, size=4, write=False, group="core"):
        # In the second input pass, scatter's first counter read is issued per
        # initial letter. The raw span is charged once even if p splits the run.
        if group == "setup" and not write and self.scanned < len(self.scatter):
            expected, raw, length = self.scatter[self.scanned]
            assert addr == old.COUNTER + 4*expected
            if length:
                self.memory.at(RAW + 2*raw, 2*length)
            self.scanned += 1
        base = addr & ~0xffffffff
        self.memory.at(self.bases[base] + addr-base, size, write)
        self.count[group + ("_w" if write else "_r")] += 1

    def histogram(self, rec, same, extra):
        assert same is self and not extra
        # Raw caps aggregate in registers, counter and prefix traffic as in slots/census.
        mem = self.memory.at
        mem(COUNTERS, 257*4, True)
        tuples = list(zip(rec.a["l_byte"], rec.a["l_lam"]))
        raw = row = i = 0
        marks = []
        scatter = []
        while i < len(tuples):
            c, width = tuples[i]
            j = i+1
            while j < len(tuples) and tuples[j][0] == c:
                width += tuples[j][1]
                j += 1
            size = (width+126)//127
            mem(RAW+2*raw, 2*size)
            mem(COUNTERS+4*c, 4)
            mem(COUNTERS+4*c, 4, True)
            scatter.append((c, raw, size))
            scatter.extend((c, raw, 0) for _ in range(i+1, j))
            if row <= rec.p < row+width:
                marks = [raw, size, c]
            i, raw, row = j, raw+size, row+width
        assert len(scatter) == rec.r and row == rec.n and marks
        first, length, c = marks
        # Test neighbouring caps to delimit the marked run, then sum it.
        start, stop = max(0, first-1), min(raw, first+length+1)
        mem(RAW+2*start, 2*(stop-start))
        mem(RAW+2*first, 2*length)
        mem(COUNTERS+4*c, 4)
        mem(COUNTERS+4*c, 4, True)
        mem(self.arena+16, 256, True)
        for c in range(256):
            mem(COUNTERS+4*c, 4)
            mem(COUNTERS+4*c, 4, True)
        self.scatter = scatter

    def report(self):
        return {}


def writer(layout, words):
    """src/cells_owned.rs, full-period guard. Positions reuse the released top slots."""
    r, n, pairs = layout.r, layout.rec.n, words.pairs
    nr = len(pairs)
    assert 0 < nr <= 5*r
    at = layout.memory.at
    pos = [-1] * nr
    positions = KERNEL + 4096
    at(positions, 4*nr, True)
    ref = lambda w: w >> 32
    child = lambda w: (ref(w)-RULES)//16
    length = lambda w: w & (MARK-1)
    j, offset, spine = nr-1, 0, 0
    while j >= 0:
        a, b = pairs[j]
        at(layout.rule+16*j, 16)
        spine += 1
        if a & MARK:
            j = child(a)
        else:
            offset += length(a)
            j = child(b)
    pos[-1] = (-offset-1) % n
    at(positions+4*(nr-1), 4, True)
    later = (-1, -1)
    queue = []
    literal_bytes = copied = 0
    for j in reversed(range(nr)):
        a, b = pairs[j]
        at(layout.rule+16*j, 16)
        at(positions+4*j, 4)
        q = pos[j]
        assert 0 <= q < n
        for side, (w, d) in enumerate(((a, q), (b, (q+length(a))%n))):
            k, size = child(w), length(w)
            if k < 0:
                at(layout.arena + ref(w)-ARENA, size)
                at(OUTPUT+d, size, True)
                literal_bytes += size
            elif later[side] != w:
                pos[k] = d
                at(positions+4*k, 4, True)
            else:
                queue.append((k, d, size))
                at(layout.rule+16*(nr-len(queue)), 16, True)
        later = a, b
    for j, (k, d, size) in enumerate(reversed(queue), nr-len(queue)):
        at(layout.rule+16*j, 16)
        at(positions+4*k, 4)
        assert 0 <= pos[k] <= n-size and d <= n-size
        at(OUTPUT+pos[k], size)
        at(OUTPUT+d, size, True)
        copied += size
    assert literal_bytes + copied == n
    return nr, spine, len(queue), copied


def load_table(directory, file):
    with (directory / file).open() as f:
        return list(csv.DictReader(f))


def measure(path, gates, fields):
    rec = old.irec.load(path)
    assert rec.period == rec.n and rec.closures == 1
    counted = gates[rec.block]
    for kind in ("eager", "lazy"):
        if int(counted[kind]) == 0:
            continue
        lazy = kind == "lazy"
        m, words = Layout(rec, lazy), Words()
        with patch.object(old, "Memory", return_value=m), patch.object(old, "Words", return_value=words), \
             patch.object(old, "histogram", m.histogram):
            old.baseline(rec, lazy)
        assert m.scanned == rec.r
        for key in ("core_r","core_w","hint_r","word_r","word_w"):
            assert m.count[key] == int(fields[rec.block,kind][key])
        nr, spine, copies, size = writer(m, words)
        assert (nr,copies,size) == tuple(int(counted[k]) for k in ("rules","copies","copy_bytes"))
        row = dict(block=rec.block, model=kind, cuts=rec.steps, rules=nr, mark_depth=spine,
                   copies=copies, copy_bytes=size)
        measure = m.memory.report()
        for key in ("read_bytes","write_bytes","distinct_lines"):
            row[key] = measure[key]
        for k in (32,256,2048):
            for group in ("fills","read_fills","dirty"):
                row[f"{group}{k}"] = measure[f"{group}{k}"]
        yield row


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input")
    args = parser.parse_args()
    gates = {int(r["block"]):r for r in load_table(JOURNAL/"2026-11-18-cells-live-reuse",
                                                  "blocks.csv") if r["corpus"] == "wikidata.bz2"}
    fields = {(int(r["block"]),r["model"]):r for r in load_table(SWEEP, "traffic.csv")}
    output = None
    for path in paths(args.input):
        for r in measure(path, gates, fields):
            if output is None:
                output = csv.DictWriter(sys.stdout, fieldnames=r, lineterminator="\n")
                output.writeheader()
            output.writerow(r)
            sys.stdout.flush()
