"""Compute clean-tile transfers directly on disjoint runs of tiles. No tile catalogue."""

import argparse
from bisect import bisect_right
from collections import Counter
import csv
from heapq import merge
from pathlib import Path
import sys

from residual import append_arc, induce, original
from screen import inputs


def groups_of(bands, b, p):
    groups = []
    images = [[] for _ in range(256)]
    marked = p // b
    for x, y, w, c in bands:
        if (y - x) % b:
            continue
        lo, hi, delta = (x + b - 1) // b, (x + w) // b, (y - x) // b
        for a, z in ((lo, min(hi, marked)), (max(lo, marked + 1), hi)):
            if a < z:
                groups.append((a, z, delta, c))
                images[c].append((a + delta, z + delta))
    return groups, [span for bucket in images for span in bucket]


def heads(groups, images):
    j = 0
    for a, z, _, _ in groups:
        while j < len(images) and images[j][1] <= a:
            j += 1
        while a < z:
            if j == len(images) or images[j][0] >= z:
                yield a, z
                break
            l, h = images[j]
            if a < l:
                yield a, l
            a = max(a, h)
            j += h <= z


def contract(bands, n, p, b, visit_start=None):
    groups, images = groups_of(bands, b, p)
    starts = [g[0] for g in groups]
    count = Counter(
        groups=len(groups),
        head_spans=0,
        bundles=0,
        inside_visits=0,
        probes=0,
        rank_queries=0,
        route_advances=0,
        route_uses=0,
        external_pieces=0,
        initial_seeks=0,
    )

    def pred(x):
        lo, hi = 0, len(starts)
        while lo < hi:
            mid = (lo + hi) // 2
            count["probes"] += 1
            if visit_start is not None:
                visit_start(mid)
            if starts[mid] <= x:
                lo = mid + 1
            else:
                hi = mid
        return lo - 1

    direct = []
    prefix_removed = []
    left = removed = 0
    for a, z, _, _ in groups:
        if left < a * b:
            direct.append((left, a * b, left - removed, b""))
        prefix_removed.append(removed)
        removed += (z - a) * b
        left = z * b
    if left < n:
        direct.append((left, n, left - removed, b""))

    def compact(x):
        count["rank_queries"] += 1
        j = pred(x // b)
        if j < 0:
            return x
        a, z, _, _ = groups[j]
        assert x >= z * b
        return x - prefix_removed[j] - (z - a) * b

    transfers = []
    for h, limit in heads(groups, images):
        count["head_spans"] += 1
        while h < limit:
            y, width = h, limit - h
            word = bytearray()
            while True:
                j = pred(y)
                if j >= 0 and y < groups[j][1]:
                    _, stop, d, c = groups[j]
                    width = min(width, stop - y)
                    word.append(c)
                    count["inside_visits"] += 1
                    y += d
                else:
                    if j + 1 < len(starts):
                        width = min(width, starts[j + 1] - y)
                    break
            assert width > 0 and word
            transfers.append((h * b, (h + width) * b, compact(y * b), bytes(word)))
            count["bundles"] += 1
            h += width

    # Two already sorted streams. The image of the retained part of each byte
    # advances monotonically through its own subrange of this list.
    routes = list(merge(direct, transfers))
    route_starts = [route[0] for route in routes]
    positions = [None] * 256
    arcs = []
    i = 0
    for a, z, rank, _ in direct:
        while i < len(bands) and bands[i][0] + bands[i][2] <= a:
            i += 1
        while a < z:
            x, y, w, c = bands[i]
            take = min(z, x + w) - a
            target = y + a - x
            count["external_pieces"] += 1
            j = positions[c]
            if j is None:
                j = bisect_right(route_starts, target) - 1
                count["initial_seeks"] += 1
            stop = target + take
            while target < stop:
                while routes[j][1] <= target:
                    j += 1
                    count["route_advances"] += 1
                lo, hi, dst, rest = routes[j]
                assert lo <= target < hi
                part = min(hi, stop) - target
                append_arc(arcs, rank, dst + target - lo, part, bytes([c]) + rest)
                rank += part
                target += part
                count["route_uses"] += 1
            positions[c] = j
            a += take
            i += a == x + w
    return n - removed, compact(p), arcs, dict(count)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("--blocks", nargs="+", type=int)
    parser.add_argument("--size", type=int, default=2)
    args = parser.parse_args()
    writer = None
    for block, n, p, _, bands in inputs(args.folder, args.blocks):
        s, mark, arcs, c = contract(bands, n, p, args.size)
        got, row = induce(arcs, s, mark, n)
        want, base = induce(original(bands), n, p, n)
        assert got == want
        result = dict(
            block=block,
            B=args.size,
            base_cuts=base["cuts"],
            cuts=row["cuts"],
            rules=row["rules"],
            raw_bands=len(arcs),
            **c,
        )
        if writer is None:
            writer = csv.DictWriter(sys.stdout, fieldnames=result, lineterminator="\n")
            writer.writeheader()
        writer.writerow(result)


if __name__ == "__main__":
    main()
