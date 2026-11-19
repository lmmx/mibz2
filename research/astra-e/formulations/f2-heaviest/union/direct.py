"""Direct first-return bands for a union of the most frequent parent return types."""
from bisect import bisect_right
from heapq import heappush, heappop
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from search import Column


def heavy_set(col, section, k):
    start, end = section
    if start == end:
        return [], 0, 0
    heap = [(start-end, start)]
    winners = []  # (frequency, negative left endpoint)
    pops = pieces = 0
    while heap:
        negwidth, left = heappop(heap)
        if len(winners) == k and -negwidth < winners[0][0]:
            break
        children, cost = col.children(left, left-negwidth)
        pieces += cost
        pops += 1
        for a, b in children.values():
            width = b-a
            if len(winners) == k and width < winners[0][0]:
                continue
            if start <= a < end:
                assert b <= end
                heappush(winners, (width,-a))
                if len(winners) > k:
                    heappop(winners)
            else:
                assert b <= start or a >= end
                heappush(heap, (-width,a))
    result = sorted((-neg, -neg+width) for width, neg in winners)
    assert len(winners) < k or pops * winners[0][0] <= col.rows
    return result, pops, pieces


def components(intervals):
    result = []
    for a,b in intervals:
        if result and result[-1][1] == a:
            result[-1] = (result[-1][0], b)
        else:
            assert not result or result[-1][1] < a
            result.append((a,b))
    return result


class Direct:
    def __init__(self, col, intervals):
        self.col = col
        self.components = components(intervals)
        self.starts = [a for a,_ in self.components]
        self.bands = []
        self.word_counts = {}
        self.visits = self.plain_runs = 0
        previous = None
        for color,(start,end) in enumerate(self.components):
            dest = start
            while dest < end:
                row = dest
                width = end-dest
                reversed_word = bytearray()
                while True:
                    r = bisect_right(col.starts, row)-1
                    width = min(width, col.starts[r+1]-row)
                    reversed_word.append(col.symbols[r])
                    row += col.maps[r]
                    i = bisect_right(self.starts, row)-1
                    if i >= 0 and row < self.components[i][1]:
                        assert row+width <= self.components[i][1]
                        break
                    assert i+1 == len(self.starts) or row+width <= self.starts[i+1]
                word = bytes(reversed(reversed_word))
                self.visits += len(word)
                self.plain_runs += int(previous != word)
                previous = word
                self.word_counts[word] = self.word_counts.get(word,0) + width
                self.bands.append((row, dest, width, word, color))
                dest += width
        self.bands.sort()
        self.sources = [r[0] for r in self.bands]
        # The induced map is a permutation of U. Sorted affine sources tile its components.
        cursor = 0
        for start,end in self.components:
            row = start
            while row < end:
                base, _, width, _, _ = self.bands[cursor]
                assert base == row
                row += width
                cursor += 1
            assert row == end
        assert cursor == len(self.bands)
        assert self.visits <= self.col.rows
        self.split_plain = self._check_buckets()

    def _check_buckets(self):
        grouped = {}
        for start, _, width, word, color in self.bands:
            key = word,color
            if key in grouped:
                assert grouped[key] == start
            grouped[key] = start + width
        closed = set()
        previous = None
        split = set()
        for _,_,_,word,_ in self.bands:
            if previous != word:
                if word in closed:
                    split.add(word)
                closed.add(word)
                previous = word
        return len(split)

    def expand(self, mark):
        col = self.col
        if not self.bands:
            return None
        row = mark
        skip = 0
        while True:
            i = bisect_right(self.starts, row)-1
            if i >= 0 and row < self.components[i][1]:
                break
            r = bisect_right(col.starts, row)-1
            row += col.maps[r]
            skip += 1
            if row == mark:
                return None
        out = bytearray()
        while len(out) < col.rows:
            i = bisect_right(self.sources, row)-1
            base, dest, width, word, _ = self.bands[i]
            assert base <= row < base+width and skip < len(word)
            out.extend(word[skip:skip+col.rows-len(out)])
            row = dest+row-base
            skip = 0
        return bytes(out)
