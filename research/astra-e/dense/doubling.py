"""Price fully compiled 2^j-step open maps including every partition build.

A stopping interval stores its word, true stopping rank, and maximal inherited
right bound. For a continuing tile the LF image at depth k is disjoint from the
images of other continuing tiles. Intersect those images with the k-step map.
"""
import bisect
import csv
import sys
from screen import paths, prepare, read_input


def profile(runs, primary):
    hist, holes, starts, ends, symbols, delta, n, retained = prepare(runs, primary)
    k = 1
    # Source words are optional for the count, but compare all packed results with
    # the scalar spelling, exit and band width in every case.
    slab = [(e, d, b"" if c is None else bytes([c])) for e, c, d in zip(ends, symbols, delta)]
    setup = 0
    sources = 0
    baseline = []
    while k <= 64:
        ends = [t[0] for t in slab]
        mixed = {(e - 1) // 32 for e in ends[:-1] if e % 32}
        checks, probes, miss = [], 0, 0
        longest = 0
        for c, choices in enumerate(holes):
            left = starts[c]
            for first, last in choices + [(hist[c], hist[c])]:
                end = starts[c] + first
                while left < end:
                    x = left
                    width = end - left
                    spelling = bytearray()
                    while True:
                        probes += 1
                        miss += x // 32 in mixed
                        j = bisect.bisect_right(ends, x)
                        b, d, w = slab[j]
                        width = min(width, b - x)
                        x += d
                        spelling += w
                        if len(w) < k:
                            break
                    longest = max(longest, len(spelling))
                    checks.append((width, x, spelling))
                    left += width
                left = starts[c] + last
        if k == 1:
            baseline = checks
        else:
            assert checks == baseline
        assert sum(w for w, _, _ in checks) == retained
        yield n, k, len(slab), setup, sources, probes, miss, len(checks), longest
        if k == 64:
            break
        new = []
        left = 0
        for end, advance, word in slab:
            if len(word) != k:
                new.append((end, advance, word))
                left = end
                continue
            sources += 1
            y = left + advance
            j = bisect.bisect_right(ends, y)
            while left < end:
                b, d, w = slab[j]
                length = min(end - left, b - y)
                left += length
                y += length
                new.append((left, advance + d, word + w))
                j += 1
        assert len(new) <= 2 * len(slab) - 1
        setup += len(new)
        slab = new
        k *= 2


if __name__ == "__main__":
    out = csv.writer(sys.stdout, lineterminator="\n")
    out.writerow("block,rows,k,tiles,compiled_descriptor_writes,compiled_source_searches,probes,mixed_probes,bands,max_word_height".split(","))
    for path in paths(sys.argv[1]):
        for row in profile(*read_input(path)):
            out.writerow((path.name.split(".")[0], *row))
