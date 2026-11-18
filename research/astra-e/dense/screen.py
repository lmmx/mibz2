"""Count the tile descriptors actually reached in queued, width-16 straight tracing."""
import bisect
import csv
import sys
from pathlib import Path

STUDY = Path(__file__).resolve().parents[3] / "docs/journal/2026-11-18-astra-c-output-first"
sys.path.insert(0, str(STUDY))
from inputs import read_input, paths


def count(runs, primary, pages):
    xs, endpoints, symbols, delta, by_symbol = [], [], [], [], [[] for _ in range(256)]
    hist = [0] * 256
    retained = cursor = n = 0
    for c, w in runs:
        parts = [(w, w <= 16)]
        if w > 16 and n <= primary < n + w:
            parts = [(primary - n, False), (1, True), (n + w - primary - 1, False)]
        for width, keep in parts:
            if not width:
                continue
            if not keep:
                if cursor < n:
                    xs.append((cursor, n, None, retained - (n - cursor) - cursor))
                xs.append((n, n + width, c, hist[c] - n))
                by_symbol[c].append((hist[c], hist[c] + width))
                cursor = n + width
            else:
                retained += width
            n += width
            hist[c] += width
    if cursor < n:
        xs.append((cursor, n, None, retained - (n - cursor) - cursor))
    starts = [0]
    for width in hist:
        starts.append(starts[-1] + width)
    for _, end, byte, d in xs:
        endpoints.append(end)
        symbols.append(byte)
        delta.append(d if byte is None else d + starts[byte])

    visits = [[] for _ in pages]
    mixed = []
    for i, page in enumerate(pages):
        mixed.append({(end - 1) // page for end in endpoints[:-1] if end % page})
        visits[i] = [0, 0, 0]
    steps = bands = terminals = weighted = 0
    for c, holes in enumerate(by_symbol):
        old = starts[c]
        for start, end in holes + [(hist[c], hist[c])]:
            stop = starts[c] + start
            while old < stop:
                row = old
                width = stop - old
                height = 0
                while True:
                    q = bisect.bisect_right(endpoints, row)
                    for i, page in enumerate(pages):
                        visits[i][0] += row // page in mixed[i]
                        q0 = bisect.bisect_right(endpoints, row // page * page)
                        visits[i][1] += (q != q0)
                        visits[i][2] += q - q0
                    width = min(width, endpoints[q] - row)
                    if symbols[q] is None:
                        break
                    row += delta[q]
                    height += 1
                old += width
                terminals += width
                steps += height
                weighted += height * width
                bands += 1
            old = starts[c] + end
    assert terminals == retained and terminals + weighted == n, "expected full-orbit archive"
    for page, mix, (indirect, after_first, scans) in zip(pages, mixed, visits):
        yield n, retained, len(xs), bands, steps, page, (n + page - 1) // page, len(mix), indirect, after_first, scans


def main():
    out = csv.writer(sys.stdout, lineterminator="\n")
    out.writerow("block,rows,retained,tiles,bands,wide_steps,page,pages,mixed_pages,mixed_lookups,after_first_tile,old_scan_advances".split(","))
    for path in paths(sys.argv[1]):
        runs, primary = read_input(path)
        for row in count(runs, primary, (8, 16, 32, 64)):
            out.writerow((path.name.split(".")[0], *row))


if __name__ == "__main__":
    main()
