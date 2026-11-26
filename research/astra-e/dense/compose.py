"""Bound the cost and actual benefit of decoding up to two steps in one descriptor."""
import bisect
import csv
import sys
from screen import paths, prepare, read_input


def census(runs, primary, pages):
    hist, holes, starts, ends, syms, delta, n, retained = prepare(runs, primary)
    for page in pages:
        # A page may compile only on homogeneous wide source and destination.
        # Its full slack is measured relative to the first row of the page.
        lookup = []
        setup = pairs = 0
        for left in range(0, n, page):
            hi = min(left + page, n)
            i = bisect.bisect_right(ends, left)
            if ends[i] < hi or syms[i] is None:
                lookup.append(None)
                continue
            setup += 1
            y = left + delta[i]
            j = bisect.bisect_right(ends, y)
            if ends[j] < y + hi - left:
                lookup.append(None)
                continue
            pairs += 1
            end = min(ends[i], left + ends[j] - y)
            lookup.append((end, delta[i] + delta[j], syms[j] is None))

        accesses = old = composed = 0
        terminals = weighted = 0
        for c, wide in enumerate(holes):
            left = starts[c]
            for start, end in wide + [(hist[c], hist[c])]:
                stop = starts[c] + start
                while left < stop:
                    row = left
                    width = stop - left
                    height = 0
                    while True:
                        accesses += 1
                        pair = lookup[row // page]
                        if pair is None:
                            i = bisect.bisect_right(ends, row)
                            width = min(width, ends[i] - row)
                            terminal = syms[i] is None
                            row += delta[i]
                            old += 1
                            height += not terminal
                        else:
                            bound, advance, terminal = pair
                            width = min(width, bound - row)
                            row += advance
                            old += 2
                            composed += 1
                            height += 2 - terminal
                        if terminal:
                            break
                    terminals += width
                    weighted += height * width
                    left += width
                left = starts[c] + end
        assert retained == terminals and weighted + terminals == n
        assert old == accesses + composed
        yield n, page, len(lookup), setup, pairs, old, accesses, composed


if __name__ == "__main__":
    out = csv.writer(sys.stdout, lineterminator="\n")
    out.writerow("block,rows,page,pages,setup_target_lookups,composed_pages,old_probes,new_probes,removed_probes".split(","))
    for path in paths(sys.argv[1]):
        for row in census(*read_input(path), (16, 32, 64)):
            out.writerow((path.name.split(".")[0], *row))
