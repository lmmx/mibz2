"""Test every distinct contiguous split of the present symbols on the pinned tapes."""
import bisect
import csv
import sys
from screen import columns, receipts


def screen(primary, runs):
    counts = [0] * 256
    by_byte = [[] for _ in counts]
    starts = []
    row = 0
    for i, (c, w) in enumerate(runs):
        starts.append(row)
        row += w
        counts[c] += w
        by_byte[c].append((counts[c], i))
    n = row
    ends = []
    start = 0
    for w in counts:
        start += w
        ends.append(start)
    # Reuse the true first 128 forward ranks for all colourings.
    actual = [primary]
    for _ in range(127):
        p = actual[-1]
        c = bisect.bisect_right(ends, p)
        k = p - (ends[c] - counts[c])
        j = bisect.bisect_right(by_byte[c], (k, len(runs)))
        before = by_byte[c][j-1][0] if j else 0
        actual.append(starts[by_byte[c][j][1]] + k - before)
    # Fenwick tree over L runs for binary select as the split moves. The
    # complement's node weight is its full input width minus its low-group width.
    size = len(runs)
    low = [0] * (size + 1)
    total = [0] + [runs[i][1] for i in range(size)]
    for i in range(1, size + 1):
        j = i + (i & -i)
        if j <= size:
            total[j] += total[i]
    def select(k, high):
        i = 0
        bit = 1 << (size.bit_length() - 1)
        while bit:
            j = i + bit
            if j <= size:
                mass = total[j] - low[j] if high else low[j]
                if mass <= k:
                    k -= mass
                    i = j
            bit >>= 1
        return starts[i] + k
    prefix = [0] * 256
    pos = run_idx = 0
    for c in range(255):
        if not counts[c]:
            continue
        m = ends[c]
        if m == n:
            break
        for _, idx in by_byte[c]:
            i = idx + 1
            while i <= size:
                low[i] += runs[idx][1]
                i += i & -i
        while pos < m:
            sym, w = runs[run_idx]
            take = min(starts[run_idx] + w, m) - pos
            prefix[sym] += take
            pos += take
            if pos == starts[run_idx] + w:
                run_idx += 1
        errors = 0
        for group in (range(c + 1), range(c + 1, 256)):
            t = sum(prefix[j] for j in group)
            init = 0
            for j in group:
                common = max(0, min(t, init + prefix[j]) - init)
                errors += 2 * (prefix[j] - common)
                init += counts[j]
        rank = primary
        diff = None
        for i, wanted in enumerate(actual):
            high = rank >= m
            if high != (wanted >= m):
                diff = i
                break
            rank = select(rank - m if high else rank, high)
        yield c + 1, m, n, errors, diff


if __name__ == '__main__':
    samples = [part for path in sys.argv[1:] for part in columns(path)]
    if len(samples) != 3:
        raise ValueError('provide scorer-extracted symbol tapes for ordinals 1,41,85')
    from pathlib import Path
    with Path(__file__).with_name('wikidata.csv').open() as f:
        known = list(csv.DictReader(f))
    out = csv.writer(sys.stdout, lineterminator='\n')
    out.writerow(['ordinal', 'cut', 'group0_rows', 'n', 'wrong_next_colours', 'first_marked_difference_in_128'])
    for row, (p, runs) in zip(known, samples):
        assert receipts(runs) == row['capped_run_sha256']
        for values in screen(p, runs):
            out.writerow([row['stream_block'], *values])
