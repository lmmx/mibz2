"""On-demand marked cursor. A structural transducer and a packed-layout address model."""
import argparse
import csv
import sys
from collections import Counter

from inputs import check_frozen, dense_audit, digest, paths, read_input
from lines import BASE, DIRECTORY, HEAD, HIST, NODE, OUT, RAW, Traffic


def setup(runs, at, anchor=None):
    # A run is delivered by coalescing adjacent capped records in registers on
    # each pass. The trace aggregates their consecutive 2-byte reads into a span.
    histogram = [[0, 0] for _ in range(256)]
    for c in range(256):
        at(HIST + 8*c, 8, True)
    physical = 0
    for c, w in runs:
        capped = (w + 126) // 127
        at(RAW + 2*physical, 2*capped)
        physical += capped
        at(HIST + 8*c, 8)
        histogram[c][0] += w
        histogram[c][1] += 1
        at(HIST + 8*c, 8, True)

    anchor_range = None
    n = count = 0
    for c, (rows, entries) in enumerate(histogram):
        at(HIST + 8*c, 8)
        if c == anchor:
            anchor_range = (n, n + rows)
        histogram[c] = [n, count]
        n += rows
        count += entries
        at(HIST + 8*c, 8, True)
    base = [None] * count
    physical = top = 0
    for c, w in runs:
        capped = (w + 126) // 127
        at(RAW + 2*physical, 2*capped)
        physical += capped
        at(HIST + 8*c, 8)
        left, slot = histogram[c]
        histogram[c] = [left + w, slot + 1]
        at(HIST + 8*c, 8, True)
        at(BASE + 8*slot, 8, True)
        base[slot] = (left, top - left, c)
        top += w
    base.append((n, 0, 0))
    at(BASE + 8*count, 4, True)  # sentinel start n, for the last right bound

    pages = (n + 63) // 64
    directory = []
    i = 0
    for page in range(pages):
        before, mask = i, 0
        while i < count and base[i][0] // 64 == page:
            at(BASE + 8*i, 4)  # sorted streaming lookahead held across pages
            mask |= 1 << (base[i][0] % 64)
            i += 1
        directory.append((before, mask))
        at(DIRECTORY + 16*page, 16, True)
        if anchor is None:
            at(HEAD + 4*page, 4, True)
    assert i == count
    return base, directory, anchor_range


def locate(row, base, directory, at):
    page, offset = divmod(row, 64)
    at(DIRECTORY + 16*page, 16)
    before, mask = directory[page]
    idx = before + (mask & ((2 << offset)-1)).bit_count() - 1
    at(BASE + 8*idx, 8)
    a, d, c = base[idx]
    return idx, a, d, c


class Directory:
    """Unsorted disjoint clipped intervals, newest first. Zero is an empty head."""
    def __init__(self, pages, at, counts):
        self.heads = [0] * pages
        self.nodes = []
        self.capacity = 64
        self.origin = NODE
        self.at = at
        self.counts = counts

    def find(self, x):
        page, offset = divmod(x, 64)
        self.at(HEAD + 4*page, 4)
        node = self.heads[page]
        depth = 0
        while node:
            self.at(self.origin + 16*(node-1), 16)
            left, right, suffix, node = self.nodes[node-1]
            depth += 1
            if left <= offset < right:
                break
        else:
            suffix = None
        self.counts["node_tests"] += depth
        self.counts["max_chain_read"] = max(self.counts["max_chain_read"], depth)
        return suffix

    def publish(self, lo, hi, source, length, delta):
        # suffix is the packed source/length/exit information, repeated in each node.
        suffix = source, length, delta
        for page in range(lo // 64, (hi-1) // 64 + 1):
            if len(self.nodes) == self.capacity:
                # Charge an out-of-place doubling. Stored links are indices.
                new = self.origin + (1 << 30)
                for offset in range(0, 16*self.capacity, 64):
                    self.at(self.origin + offset, 64)
                    self.at(new + offset, 64, True)
                self.counts["relocated_nodes"] += self.capacity
                self.counts["growths"] += 1
                self.capacity *= 2
                self.origin = new
            left = max(lo - 64*page, 0)
            right = min(hi - 64*page, 64)
            self.at(HEAD + 4*page, 4)
            self.nodes.append((left, right, suffix, self.heads[page]))
            self.at(self.origin + 16*(len(self.nodes)-1), 16, True)
            self.heads[page] = len(self.nodes)
            self.at(HEAD + 4*page, 4, True)


def emit(runs, row, delim=44, cap=64, memory=None, out=None):
    """Use a negative delimiter for fixed cap. Source bytes always end before the cursor."""
    assert 1 <= cap <= 64 and -1 <= delim <= 255
    at = memory.at if memory else lambda *args: None
    counts = Counter()
    base, directory, _ = setup(runs, at)
    n = base[-1][0]
    assert 0 <= row < n < 1 << 24
    memo = Directory(len(directory), at, counts)
    if out is None:
        out = bytearray(n)
    assert len(out) == n
    view = memoryview(out)
    q = 0
    while q < n:
        dest = q
        counts["queries"] += 1
        found = memo.find(row)
        if found is not None:
            source, length, delta = found
            take = min(length, n-dest)
            assert 0 <= source and source + take <= dest
            at(OUT + source, take)
            at(OUT + dest, take, True)
            view[q:q+take] = view[source:source+take]
            q += take
            counts["hits"] += 1
            counts["copy_bytes"] += take
            if take == length:
                row += delta
            else:
                counts["partial_hits"] += 1  # last bytes need no successor
            continue

        start = row
        lo, hi = 0, n
        stop = min(n, dest + cap)
        while q < stop:
            idx, a, d, c = locate(row, base, directory, at)
            at(BASE + 8*(idx+1), 4)
            b = base[idx+1][0]
            assert a <= row < b
            shift = start - row
            lo, hi = max(lo, a+shift), min(hi, b+shift)
            at(OUT + q, 1, True)
            out[q] = c
            q += 1
            row += d
            if c == delim:
                break
        length = q - dest
        counts["direct_steps"] += length
        assert lo <= start < hi
        if q < n:
            memo.publish(lo, hi, dest, length, row-start)
            counts["posted"] += 1
            counts["covered_rows"] += hi-lo
    assert q == n and counts["direct_steps"] + counts["copy_bytes"] == n
    counts["nodes"] = len(memo.nodes)
    counts["capacity"] = memo.capacity
    return out, counts


def measure(path, delim):
    runs, primary = read_input(path)
    block = int(path.name.split(".")[0])
    trace = Traffic()
    storage = bytearray([0xa5]) * sum(w for _, w in runs)
    out, c = emit(runs, primary, delim, memory=trace, out=storage)
    column, want, _ = dense_audit(runs, primary)
    assert out == want
    check_frozen(block, column, out, primary)
    keys = ("queries hits partial_hits direct_steps copy_bytes posted covered_rows nodes "
            "node_tests max_chain_read capacity growths relocated_nodes").split()
    return {"block": block, "n": len(out), "p": primary, "runs": len(runs),
            "capped": sum((w+126)//127 for _, w in runs), "delimiter": delim,
            **{k: c[k] for k in keys}, **trace.report(), "output_sha256": digest(out)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input")
    parser.add_argument("--delimiter", type=int, default=44)
    args = parser.parse_args()
    writer = None
    for path in paths(args.input):
        r = measure(path, args.delimiter)
        if writer is None:
            writer = csv.DictWriter(sys.stdout, fieldnames=r, lineterminator="\n")
            writer.writeheader()
        writer.writerow(r)
        sys.stdout.flush()
