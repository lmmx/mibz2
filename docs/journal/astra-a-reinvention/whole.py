"""Whole original domains only. A failed containment test retires its source permanently."""
from collections import Counter

from shortcut import HEADER, NODE, OUT, ignore, initialize, root_at


def reconstruct(records, n, primary, at=ignore, audit=None):
    masks, before, nodes, headers, roots = initialize(records, n, primary, at, whole=True)
    stats = Counter(roots=roots, nodes=len(nodes), tiles=len(masks))
    out, row, pos, previous = bytearray(n), primary, 0, None
    while pos < n:
        root = root_at(row, masks, before, at)
        at(HEADER + 4*root, 4)
        k = headers[root]
        if k < 0:
            target, value = divmod(-1-k, 256)
            low, high, shift, length, src, retired = row, row+1, target-row, 1, value, True
            stats["unit_visits"] += 1
        else:
            at(NODE + 16*k, 16)
            low, high, shift, length, src, retired = nodes[k]
            stats["wide_visits"] += 1
        take = min(n-pos, length)
        if length == 1:
            out[pos] = src
            stats["literals"] += 1
        else:
            assert src + length <= pos
            at(OUT+src, take)
            out[pos:pos+take] = out[src:src+take]
            stats["copies"] += 1
            stats["copy_bytes"] += take
        at(OUT+pos, take, True)
        if audit is not None:
            audit(pos, row, (low, high, shift, length, src), take)
        pos += take
        if pos == n:
            stats["clipped"] = int(take != length)
            break

        invalid = False
        if previous is not None:
            old_key, a, b, e, m, ref = previous
            if low <= a+e and b+e <= high:
                nodes[old_key] = (a, b, e+shift, m+length, pos-m-length, False)
                stats["installs"] += 1
            else:
                nodes[old_key] = (a, b, e, m, ref, True)
                stats["retired"] += 1
            at(NODE+16*old_key, 16, True)
            invalid = old_key == k
            stats["aliases"] += int(invalid)
        previous = None if retired or invalid else (k, low, high, shift, length, src)
        row += shift
    stats["phrases"] = stats["literals"] + stats["copies"]
    return out, stats
