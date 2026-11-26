"""Certify the most frequent first return, using only run/interval intersections."""
from bisect import bisect_right
from collections import Counter
from heapq import heappop, heappush


class Column:
    def __init__(self, runs):
        counts = Counter()
        for byte, width in runs:
            counts[byte] += width
        position = 0
        self.base = {}
        for byte, count in sorted(counts.items()):
            self.base[byte] = position
            position += count
        self.rows = position
        self.starts, self.symbols, self.maps = [], [], []
        slots = self.base.copy()
        position = 0
        for byte, width in runs:
            self.starts.append(position)
            self.symbols.append(byte)
            self.maps.append(slots[byte] - position)
            slots[byte] += width
            position += width
        self.starts.append(position)

    def children(self, left, right):
        index = bisect_right(self.starts, left) - 1
        groups = {}
        pieces = 0
        while self.starts[index] < right:
            a = max(self.starts[index], left) + self.maps[index]
            b = min(self.starts[index + 1], right) + self.maps[index]
            token = self.symbols[index]
            old = groups.get(token)
            if old is not None:
                assert old[1] == a
            groups[token] = (old[0] if old is not None else a, b)
            pieces += 1
            index += 1
        return groups, pieces

    def interval(self, word):
        left, right = 0, self.rows
        for byte in reversed(word):
            groups, _ = self.children(left, right)
            if byte not in groups:
                return 0, 0
            left, right = groups[byte]
        return left, right


def heaviest(column, section):
    start, end = section
    assert end > start
    heap = [(start - end, start, 0)]
    paths = [(-1, None)]
    best_width, best_left, best_id = 0, column.rows, None
    expanded = pieces = mass = 0
    peak = 1
    while heap and -heap[0][0] >= best_width:
        negwidth, left, parent = heappop(heap)
        children, scans = column.children(left, left - negwidth)
        expanded += 1
        pieces += scans
        mass -= negwidth
        for byte, (a, b) in children.items():
            width = b - a
            terminal = start <= a < end
            assert (start <= a and b <= end) if terminal else (b <= start or a >= end)
            if width < best_width:
                continue
            if terminal and (width, -a) <= (best_width, -best_left):
                continue
            child = len(paths)
            paths.append((parent, byte))
            if terminal:
                best_width, best_left, best_id = width, a, child
            else:
                heappush(heap, (-width, a, child))
        peak = max(peak, len(heap))
    assert best_id is not None and expanded * best_width <= mass <= column.rows
    assert pieces <= len(column.symbols) + expanded - 1
    spelling = bytearray()
    at = best_id
    while paths[at][0] != -1:
        at, byte = paths[at]
        spelling.append(byte)
    return bytes(spelling), (best_left, best_left + best_width), {
        "pops": expanded, "pieces": pieces, "expanded_mass": mass,
        "peak_heap": peak, "path_records": len(paths), "unresolved_upper": -heap[0][0] if heap else 0,
    }
