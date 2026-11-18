"""Reference transducer. Its only inputs are capped run records, n and the primary row.

A leaf is (low, high, displacement, length, literal byte or earlier output offset).
An internal node has two boundaries and two or three adjacent children.
"""
from collections import Counter
from itertools import groupby


# Distinct line-aligned regions of the cost model. The probe uses Python objects.
RECORD, HIST, TILE, NODE, OUT = (i << 32 for i in (0, 1, 2, 3, 4))


def capped(column):
    result = []
    for byte, group in groupby(column):
        q, r = divmod(sum(1 for _ in group), 127)
        result.extend([(byte << 8) | 127] * q)
        if r:
            result.append((byte << 8) | r)
    return result


def runs(records):
    for c, group in groupby(records, lambda z: z >> 8):
        width = 0
        for record in group:
            count = record & 255
            assert 0 <= c < 256 and 1 <= count <= 127
            width += count
        yield c, width


def ignore(addr, size, write=False):
    pass


HEADER = 5 << 32


def initialize(records, n, primary, at, whole=False):
    if not 0 <= primary < n <= 900000:
        raise ValueError("expected a nonempty bzip2 block and a valid primary")
    hist = [[0, 0] for _ in range(256)]
    at(HIST, 8 * 256, True)
    # Both input scans are sequential. Issue each read at its position in the scan.
    def scan():
        def reads():
            for i, record in enumerate(records):
                at(RECORD + 2 * i, 2)
                yield record
        return runs(reads())

    for c, width in scan():
        at(HIST + 8*c, 8)
        hist[c][0] += width
        hist[c][1] += 1
        at(HIST + 8*c, 8, True)
    rows = roots = 0
    for c, (width, count) in enumerate(hist):
        at(HIST + 8*c, 8)
        hist[c] = [rows, roots]
        rows += width
        roots += count
        at(HIST + 8*c, 8, True)
    assert rows == n
    nt = (n + 63) // 64
    masks, before = [0] * nt, [0] * nt
    nodes = [] if whole else [None] * roots
    headers = [0] * roots if whole else None
    at(TILE, 16 * nt, True)
    top = 0
    for c, width in scan():
        at(HIST + 8*c, 8)
        first, root = hist[c]
        hist[c] = [first + width, root + 1]
        at(HIST + 8*c, 8, True)
        leaf = (first, first + width, top - first, 1, c)
        if whole:
            at(HEADER + 4*root, 4, True)
            if width == 1:
                headers[root] = -1 - (top * 256 + c)
            else:
                headers[root] = len(nodes)
                at(NODE + 16*len(nodes), 16, True)
                nodes.append((*leaf, False))
        else:
            nodes[root] = leaf
            at(NODE + 16*root, 16, True)
        tile, bit = divmod(first, 64)
        at(TILE + 16*tile, 8)
        masks[tile] |= 1 << bit
        at(TILE + 16*tile, 8, True)
        top += width
    seen = 0
    for t, mask in enumerate(masks):
        at(TILE + 16*t, 8)
        before[t] = seen
        seen += mask.bit_count()
        at(TILE + 16*t + 8, 4, True)
    assert seen == roots
    return masks, before, nodes, headers, roots


def root_at(row, masks, before, at):
    t, bit = divmod(row, 64)
    at(TILE + 16*t, 16)
    return before[t] + (masks[t] & ((2 << bit)-1)).bit_count() - 1


def reconstruct(records, n, primary, at=ignore, audit=None):
    masks, before, nodes, _, roots = initialize(records, n, primary, at)
    stats = Counter()
    row, pos, previous = primary, 0, None
    out = bytearray(n)  # target model writes the initialized prefix once
    while pos < n:
        k = root_at(row, masks, before, at)
        depth = 0
        while True:
            at(NODE + 16*k, 16)
            node = nodes[k]
            if len(node) == 5:
                break
            a, b, children = node
            k = children + (row >= a) + (row >= b)
            depth += 1
        stats["internal_reads"] += depth
        stats["max_depth"] = max(stats["max_depth"], depth)
        low, high, shift, length, src = node
        assert low <= row < high
        take = min(n - pos, length)
        if length == 1:
            stats["literals"] += 1
            out[pos] = src
        else:
            assert 0 <= src and src + length <= pos
            at(OUT + src, take)
            out[pos:pos+take] = out[src:src+take]
            stats["copies"] += 1
            stats["copy_bytes"] += take
        at(OUT + pos, take, True)
        if audit is not None:
            audit(pos, row, node, take)
        pos += take
        if pos == n:
            stats["clipped"] = int(take != length)
            break
        invalid = False
        if previous is not None:
            key, old, start = previous
            a, b, e, m, ref = old
            lo, hi = max(a, low-e), min(b, high-e)
            assert lo <= start < hi
            if hi - lo == 1:
                stats["unit_offers"] += 1
            else:
                shortcut = (lo, hi, e+shift, m+length, pos-m-length)
                if lo == a and hi == b:
                    nodes[key] = shortcut
                    stats["whole"] += 1
                else:
                    children = len(nodes)
                    if a < lo:
                        nodes.append((a, lo, e, m, ref))
                    nodes.append(shortcut)
                    if hi < b:
                        nodes.append((hi, b, e, m, ref))
                    splits = [v for v in (lo, hi) if a < v < b]
                    stats["new_bounds"] += len(splits)
                    stats["splits"] += 1
                    # n is a valid upper sentinel. No query falls at n.
                    nodes[key] = (splits[0], splits[1] if len(splits)==2 else n, children)
                    at(NODE + 16*children, 16*(len(splits)+1), True)
                at(NODE + 16*key, 16, True)
                invalid = key == k
                stats["aliases"] += int(invalid)
        previous = None if invalid else (k, node, row)
        row += shift

    stats["phrases"] = stats["literals"] + stats["copies"]
    stats["installs"] = stats["splits"] + stats["whole"]
    stats["nodes"] = len(nodes)
    stats["roots"] = roots
    stats["tiles"] = len(masks)
    return out, stats
