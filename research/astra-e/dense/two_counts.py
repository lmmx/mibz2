"""Form the entire two-step partition and compare its traced maximal bands independently."""
import bisect
import csv
import sys
from screen import paths, prepare, read_input


def census(runs, primary):
    hist, holes, starts, ends, syms, delta, n, retained = prepare(runs, primary)
    stops, tags, jumps = [], [], []
    x = advances = 0
    # The Python construction uses binary search per source for independence.
    # The Rust builder starts each byte's cursor at its F bucket and advances
    # monotonically. Count advances from those exact initial cursors separately.
    cursors = [bisect.bisect_right(ends, base) for base in starts[:-1]]
    for end, c, d in zip(ends, syms, delta, strict=True):
        if c is None:
            stops.append(end)
            tags.append(0)
            jumps.append(d)
            x = end
            continue
        y = x + d
        j = bisect.bisect_right(ends, y)
        advances += j - cursors[c]
        while x < end:
            width = min(end - x, ends[j] - y)
            x += width
            y += width
            stops.append(x)
            tags.append(1 if syms[j] is None else 2)
            jumps.append(d + delta[j])
            if y == ends[j]:
                advances += 1
                j += 1
        cursors[c] = j
    assert len(stops) <= 2 * len(ends) - 1

    visited = bands = steps = pair_steps = singles_mixed = pairs_mixed = 0
    mixed0 = {(end - 1) // 32 for end in ends[:-1] if end % 32}
    mixed2 = {(end - 1) // 32 for end in stops[:-1] if end % 32}
    for c, wide in enumerate(holes):
        left = starts[c]
        for start, end in wide + [(hist[c], hist[c])]:
            stop = starts[c] + start
            while left < stop:
                row = left
                w = stop - left
                while True:
                    steps += 1
                    singles_mixed += row // 32 in mixed0
                    j = bisect.bisect_right(ends, row)
                    w = min(w, ends[j] - row)
                    row += delta[j]
                    if syms[j] is None:
                        dest = row
                        break
                row = left
                w2 = stop - left
                while True:
                    pair_steps += 1
                    pairs_mixed += row // 32 in mixed2
                    j = bisect.bisect_right(stops, row)
                    w2 = min(w2, stops[j] - row)
                    row += jumps[j]
                    if tags[j] < 2:
                        break
                assert w == w2 and row == dest
                left += w
                visited += w
                bands += 1
            left = starts[c] + end
    assert visited == retained
    return n, retained, len(ends), len(stops), advances, bands, steps, pair_steps, singles_mixed, pairs_mixed


if __name__ == "__main__":
    out = csv.writer(sys.stdout, lineterminator="\n")
    out.writerow("block,rows,retained,one_tiles,two_tiles,refinement_cursor_advances,bands,one_probes,two_probes,one_mixed,two_mixed".split(","))
    for path in paths(sys.argv[1]):
        out.writerow((path.name.split(".")[0], *census(*read_input(path))))
