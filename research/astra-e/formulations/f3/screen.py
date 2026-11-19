"""Count projection errors from three pinned scorer symbol tapes without constructing LF."""
import bisect
import csv
from hashlib import sha256
from pathlib import Path
import struct
import sys


def columns(path):
    data = memoryview(Path(path).read_bytes())
    while data:
        crc, p, seq, eob, limit, count = struct.unpack_from('<II256sHQQ', data)
        header = struct.calcsize('<II256sHQQ')
        front = list(range(256))
        pieces = []
        run = 0
        weight = 1
        def add(c, length):
            if pieces and pieces[-1][0] == c:
                pieces[-1] = (c, pieces[-1][1] + length)
            else:
                pieces.append((c, length))
        for (symbol,) in struct.iter_unpack('<H', data[header:header + 2 * count]):
            if symbol <= 1:
                run += (symbol + 1) * weight
                weight *= 2
                continue
            if run:
                add(seq[front[0]], run)
                run, weight = 0, 1
            if symbol == eob:
                break
            c = front.pop(symbol - 1)
            front.insert(0, c)
            add(seq[c], 1)
        assert sum(w for _, w in pieces) <= limit
        yield p, pieces
        data = data[header + 2 * count:]


def receipts(runs):
    packed = bytearray()
    for c, w in runs:
        while w:
            take = min(w, 127)
            packed.extend(struct.pack('<H', (c << 8) | take))
            w -= take
    return sha256(packed).hexdigest()


def measure(p, runs):
    freq = [0] * 256
    for c, w in runs:
        freq[c] += w
    ends = []
    n = 0
    for f in freq:
        n += f
        ends.append(n)
    # Fixed rule: choose a byte boundary closest to half the rows, smallest on a tie.
    cut = min(range(1, 256), key=lambda c: abs(2 * ends[c - 1] - n))
    m = ends[cut - 1]
    binary_ends = [m, n]
    prefix = [0] * 256
    used = [0] * 256
    used_group = [0, 0]
    ranks = [[] for _ in range(256)]
    group_ranks = [[], []]
    last_delta = [None, None]
    row = displaced = bands = 0
    for c, w in runs:
        g = int(c >= cut)
        prefix[c] += max(0, min(w, m - row))
        byte_start = ends[c] - freq[c]
        local_start = byte_start - (m if g else 0)
        delta = local_start + used[c] - used_group[g]
        displaced += w * (delta != 0)
        bands += int(delta != last_delta[g])
        last_delta[g] = delta
        used[c] += w
        used_group[g] += w
        ranks[c].append((used[c], row))
        group_ranks[g].append((used_group[g], row))
        row += w
    mistakes = 0
    for group in (range(cut), range(cut, 256)):
        end_zero = sum(prefix[c] for c in group)
        start = common = 0
        for c in group:
            common += max(0, min(end_zero, start + prefix[c]) - start)
            start += freq[c]
        mistakes += 2 * (end_zero - common)
    def advance(index, sym_ends, lists):
        c = bisect.bisect_right(sym_ends, index)
        k = index - (0 if c == 0 else sym_ends[c - 1])
        j = bisect.bisect_right(lists[c], (k, n))
        before = 0 if j == 0 else lists[c][j - 1][0]
        return lists[c][j][1] + k - before
    original = projected = p
    divergence = None
    for step in range(128):
        if (original >= m) != (projected >= m):
            divergence = step
            break
        original = advance(original, ends, ranks)
        projected = advance(projected, binary_ends, group_ranks)
    return n, cut, m, displaced, bands, mistakes, divergence


if __name__ == '__main__':
    journal = Path(__file__).resolve().parents[4] / 'docs/journal/2026-11-18-astra-b-collective/verified.csv'
    with journal.open() as f:
        known = {int(r['index']): r for r in csv.DictReader(f)}
    samples = [part for path in sys.argv[1:] for part in columns(path)]
    if len(samples) != 3:
        raise ValueError('provide scorer-extracted symbol tapes for ordinals 1,41,85 in that order')
    out = csv.writer(sys.stdout, lineterminator='\n')
    out.writerow(['stream_block', 'primary', 'capped_run_sha256', 'n', 'cut_byte', 'rows_in_group0',
                  'changed_successor_edges', 'local_correction_bands', 'wrong_next_colours', 'first_marked_difference'])
    for idx, (p, runs) in zip((1, 41, 85), samples):
        digest = receipts(runs)
        assert known[idx]['records_sha256'] == digest and int(known[idx]['primary']) == p
        out.writerow([idx, p, digest, *measure(p, runs)])
