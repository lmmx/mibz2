"""Positive first-hit intervals on the original run tape and an exact top-core selector."""
from bisect import bisect_right
from collections import defaultdict
from heapq import heappop, heappush
from itertools import accumulate


def clip(interval, section):
    """Subtract the section. Every suffix interval has width at most the section width."""
    lo, hi = interval
    a, b = section
    assert hi - lo <= b - a
    if hi <= a or b <= lo:
        return interval
    if lo < a:
        assert hi <= b
        return lo, a
    return min(hi, b), hi


class RunIndex:
    def __init__(self, runs):
        self.tape = list(runs)
        self.starts = [0]
        self.positions = defaultdict(list)
        self.endcounts = defaultdict(list)
        for byte, width in runs:
            self.positions[byte].append(self.starts[-1])
            counts = self.endcounts[byte]
            counts.append((counts[-1] if counts else 0) + width)
            self.starts.append(self.starts[-1] + width)
        self.base = [0] + list(accumulate(
            self.endcounts[b][-1] if b in self.endcounts else 0 for b in range(256)))

    def endpoint(self, byte, pos):
        i = bisect_right(self.positions.get(byte, ()), pos) - 1
        if i < 0:
            return self.base[byte]
        counts = self.endcounts[byte]
        prior = counts[i-1] if i else 0
        return self.base[byte] + min(counts[i], prior + pos - self.positions[byte][i])

    def interval(self, word, section):
        assert word
        lo, hi = section
        for depth, byte in enumerate(reversed(word)):
            if depth:
                lo, hi = clip((lo, hi), section)
            lo, hi = self.endpoint(byte, lo), self.endpoint(byte, hi)
        return lo, hi

    def children(self, interval):
        lo, hi = interval
        i = bisect_right(self.starts, lo) - 1
        seen = {}
        pieces = 0
        while lo < hi:
            byte = self.tape[i][0]
            end = min(hi, self.starts[i+1])
            if byte not in seen:
                left = self.endpoint(byte, lo)
                seen[byte] = [left, left]
            seen[byte][1] += end - lo
            pieces += 1
            lo = end
            i += 1
        return seen, pieces


def consecutive(values):
    intervals = []
    for x in sorted(values):
        if intervals and x == intervals[-1][1]:
            intervals[-1] = intervals[-1][0], x+1
        else:
            intervals.append((x, x+1))
    return intervals


class Mask:
    def __init__(self, pieces):
        unions = []
        for lo, hi in sorted(pieces):
            assert not unions or lo >= unions[-1][1]
            if unions and lo == unions[-1][1]:
                unions[-1] = unions[-1][0], hi
            else:
                unions.append((lo, hi))
        self.intervals = unions
        self.starts = [lo for lo, _ in unions]
        self.ends = [hi for _, hi in unions]
        self.bases = [0] + list(accumulate(hi-lo for lo, hi in unions))

    def rank(self, x):
        i = bisect_right(self.starts, x) - 1
        return self.bases[i] + min(x, self.ends[i]) - self.starts[i] if i >= 0 else 0

    def components(self, lo, hi):
        j = bisect_right(self.ends, lo)
        out = []
        while j < len(self.starts) and self.starts[j] < hi:
            out.append((max(lo, self.starts[j]), min(hi, self.ends[j])))
            j += 1
        return out


def heaviest(index, section, mask):
    # A queued source interval carries a common first-hit word read from right to left.
    # Cardinality before restriction to V bounds all descendants, even if this node misses V.
    a, b = section
    queue = [(a-b, b'', a, b)]
    best = 0
    ties = []
    stats = dict(expanded=0, run_pieces=0, children=0, mask_rank_queries=0, peak_heap=1,
                 max_frontier_bound=0, clipped_empty=0)
    while queue and -queue[0][0] >= best:
        _, rev, lo, hi = heappop(queue)
        if rev:
            lo, hi = clip((lo, hi), section)
        if lo == hi:
            stats['clipped_empty'] += 1
            continue
        children, cost = index.children((lo, hi))
        stats['expanded'] += 1
        stats['run_pieces'] += cost
        stats['children'] += len(children)
        for byte, (left, right) in children.items():
            if right - left < best:
                continue
            spelling = rev + bytes((byte,))
            mass = mask.rank(right) - mask.rank(left)
            stats['mask_rank_queries'] += 2
            if mass > best:
                best, ties = mass, []
            if mass and mass == best:
                ties.append((spelling[::-1], left, right))
            heappush(queue, (left-right, spelling, left, right))
        stats['peak_heap'] = max(stats['peak_heap'], len(queue))
    if queue:
        stats['max_frontier_bound'] = -queue[0][0]
        assert stats['max_frontier_bound'] < best
    return best, sorted(ties), stats
