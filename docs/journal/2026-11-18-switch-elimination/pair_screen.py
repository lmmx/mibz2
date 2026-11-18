"""Screen a word-based section by simultaneously contracting disjoint oriented pairs.

A structural count. The native decoders separately verify complete inversion.
"""
from collections import Counter
from itertools import accumulate, groupby
from pathlib import Path
import sys


def colour(word, generation):
    x = (word + generation * 62327) & 0xffffffff
    x = ((x ^ (x >> 16)) * 0x85ebca6b) & 0xffffffff
    x = ((x ^ (x >> 13)) * 0xc2b2ae35) & 0xffffffff
    return (x ^ (x >> 16)) & 1


def compact(pieces):
    # Both lists omit the same set of ranks. Name the remaining ranks in order.
    source_order = []
    at = 0
    for _, target, width, word in sorted(pieces):
        source_order.append((at, target, width, word))
        at += width
    image_order = []
    at = 0
    for source, _, width, word in sorted(source_order, key=lambda e: e[1]):
        image_order.append((source, at, width, word))
        at += width
    result = []
    for source, target, width, word in sorted(image_order):
        if result:
            old_source, old_target, old_width, old_word = result[-1]
            if old_word == word and old_target + old_width == target:
                result[-1] = old_source, old_target, old_width + width, word
                continue
        result.append((source, target, width, word))
    return result, at


def screen(column, mark):
    counts = Counter(column)
    slots = [0, *accumulate(counts[c] for c in range(256))]
    bands = []
    start = 0
    for byte, items in groupby(column):
        width = sum(1 for _ in items)
        bands.append((start, slots[byte], width, byte))
        slots[byte] += width
        start += width
    rules = {}
    print("round,intersections,removed,rows,bands,pair_rules_to_date")
    for generation in range(64):
        if len(bands) < 2 or sum(b[2] for b in bands) < 64:
            break
        by_image = sorted(bands, key=lambda b: b[1])
        i = j = z = removed = intersections = 0
        new_mark = None
        pieces = []
        colours = {b[3]: colour(b[3], generation) for b in bands}
        while i < len(bands):
            x, y, width_a, a = by_image[i]
            s, t, width_b, b = bands[j]
            end = min(y + width_a, s + width_b)
            intersections += 1
            cuts = [z, mark, mark + 1, end] if z <= mark < end else [z, end]
            for lo, hi in zip(cuts, cuts[1:]):
                if lo == hi:
                    continue
                if lo == mark:
                    new_mark = mark - removed
                if colours[a] == 0 and colours[b] == 1 and lo != mark:
                    new_word = rules.setdefault((b, a), 256 + len(rules))
                    pieces.append((x + lo - y, t + lo - s, hi - lo, new_word))
                    removed += hi - lo
                else:
                    if colours[a] == 0:
                        pieces.append((x + lo - y, lo, hi - lo, a))
                    if colours[b] == 1:
                        pieces.append((lo, t + lo - s, hi - lo, b))
            z = end
            i += z == y + width_a
            j += z == s + width_b
        assert new_mark is not None
        bands, rows = compact(pieces)
        print(generation, intersections, removed, rows, len(bands), len(rules), sep=",")
        mark = new_mark


if __name__ == "__main__":
    screen(Path(sys.argv[1]).read_bytes(), int(sys.argv[2]))
