#!/usr/bin/env python3
"""Finite, untimed reduced-LF experiment; Python 3.10+, standard library only.

Run without -O: python3 experiment.py > results.json
Half-open intervals use original L coordinates, never packed coordinates.
Words store run identities (bytes are their projection), not just byte labels.
"""

import json
from collections import Counter
from dataclasses import dataclass
from itertools import product


@dataclass(frozen=True)
class Piece:
    lo: int
    hi: int
    shift: int
    word: tuple


class LF:
    def __init__(self, column):
        self.column = column
        self.n = len(column)
        assert self.n
        # Independent row oracle: stable sort of individual row indices.
        self.inverse = sorted(range(self.n), key=lambda x: column[x])
        self.forward = [None] * self.n
        for y, x in enumerate(self.inverse):
            self.forward[x] = y
        self.runs = []  # (lo, hi, byte)
        for x, byte in enumerate(column):
            if x == 0 or byte != column[x - 1]:
                self.runs.append([x, x + 1, byte])
            else:
                self.runs[-1][1] = x + 1
        self.top = tuple(range(len(self.runs)))
        self.bottom = tuple(sorted(self.top, key=lambda i: self.runs[i][2]))
        start = 0
        image = {}
        for i in self.bottom:
            image[i] = start
            lo, hi, _ = self.runs[i]
            start += hi - lo
        self.pieces = [Piece(lo, hi, image[i] - lo, (i,))
                       for i, (lo, hi, _) in enumerate(self.runs)]
        self.row_run = [i for i, (lo, hi, _) in enumerate(self.runs)
                        for _ in range(lo, hi)]
        assert all(x + p.shift == self.forward[x]
                   for p in self.pieces for x in range(p.lo, p.hi))

    def target(self, black):
        return [(self.runs[i][0], self.runs[i][1]) for i in sorted(black)]

    def decode(self, origin):
        out = []
        for _ in range(self.n):
            origin = self.inverse[origin]
            out.append(self.column[origin])
        return ''.join(out)


def reduction(model, black):
    """Lossless unweighted deletion + paired gray endpoint insertion labels.

    Each endpoint is (number of preceding black endpoints, ordinal in gap).
    This symmetric convention differs from de Mourgues's left/right convention
    but stores the same insertion information, including exterior gaps.
    """
    orders, placements = [], {}
    for order in (model.top, model.bottom):
        reduced, gray = [], {}
        ordinal = 0
        for i in order:
            if i in black:
                reduced.append(i)
                ordinal = 0
            else:
                gray[i] = (len(reduced), ordinal)
                ordinal += 1
        rebuilt = []
        for gap in range(len(reduced) + 1):
            rebuilt.extend(i for _, i in sorted((pos[1], i)
                           for i, pos in gray.items() if pos[0] == gap))
            if gap < len(reduced):
                rebuilt.append(reduced[gap])
        assert tuple(rebuilt) == order
        orders.append(tuple(reduced))
        for i, endpoint in gray.items():
            placements.setdefault(i, []).append(endpoint)
    assert all(len(endpoints) == 2 for endpoints in placements.values())
    return orders, placements


def naive_deleted(model, black):
    """Weighted IET obtained by deleting gray edges in both orders."""
    orders, _ = reduction(model, black)
    packed = [x for i in orders[0]
              for x in range(model.runs[i][0], model.runs[i][1])]
    image = [x for i in orders[1]
             for x in range(model.runs[i][0], model.runs[i][1])]
    # Rank of the row's labelled offset in the packed bottom order.
    rank = {x: y for y, x in enumerate(image)}
    return {x: packed[rank[x]] for x in packed}


def row_returns(model, black):
    answer = {}
    for x in range(model.n):
        if model.row_run[x] not in black:
            continue
        y, word = x, []
        while True:
            word.append(model.row_run[y])
            y = model.forward[y]
            if model.row_run[y] in black:
                break
            assert len(word) < model.n
        answer[x] = (y, tuple(word))
    assert set(y for y, _ in answer.values()) == set(answer)
    return answer


def locate(pieces, x, counts):
    lo, hi = 0, len(pieces)
    while lo < hi:
        counts['base_search_comparisons'] += 1
        mid = (lo + hi) // 2
        if pieces[mid].lo <= x:
            lo = mid + 1
        else:
            hi = mid
    i = lo - 1
    assert i >= 0 and pieces[i].lo <= x < pieces[i].hi
    return i


def induce(base, target):
    """Exact translation/itinerary refinement, without row-wise construction.

    Each work item maps its original source interval to its current position.
    Split on the next base-piece boundary and then on target boundaries.
    Count every target test, binary-search comparison, composed piece and
    copied run label. Flat tuples deliberately make label costs visible.
    """
    counts = Counter()
    pending = [Piece(lo, hi, 0, ()) for lo, hi in target]
    result = []
    while pending:
        p = pending.pop()
        counts['work_items'] += 1
        left = p.lo
        while left < p.hi:
            q = base[locate(base, left + p.shift, counts)]
            right = min(p.hi, q.hi - p.shift)
            counts['base_piece_visits'] += 1
            word = p.word + q.word
            counts['label_symbols_copied'] += len(word)
            shift = p.shift + q.shift
            cuts = {left, right}
            for lo, hi in target:
                counts['target_endpoint_tests'] += 2
                for endpoint in (lo, hi):
                    cut = endpoint - shift
                    if left < cut < right:
                        cuts.add(cut)
            cuts = sorted(cuts)
            for a, b in zip(cuts, cuts[1:]):
                counts['composed_fragments'] += 1
                returns = False
                for lo, hi in target:
                    counts['target_membership_tests'] += 1
                    if lo <= a + shift < hi:
                        returns = True
                        break
                item = Piece(a, b, shift, word)
                (result if returns else pending).append(item)
            left = right
    merged = []
    for p in sorted(result, key=lambda p: p.lo):
        if (merged and merged[-1].hi == p.lo and
                (merged[-1].shift, merged[-1].word) == (p.shift, p.word)):
            prev = merged.pop()
            p = Piece(prev.lo, p.hi, p.shift, p.word)
        merged.append(p)
    counts['final_fragments'] = len(merged)
    counts['stored_word_symbols'] = sum(len(p.word) for p in merged)
    return merged, counts


def verify_map(pieces, oracle):
    actual = {x: (x + p.shift, p.word)
              for p in pieces for x in range(p.lo, p.hi)}
    assert sum(p.hi - p.lo for p in pieces) == len(actual)
    assert actual == oracle  # destination, itinerary, hence time and bytes


def reconstruct(model, pieces, origin, counts):
    """Lift a chosen origin, then traverse the INVERSE induced map.

    Retain full LF for origin search and unanchored-cycle fallback: charged,
    not silently attributed to the sparse representation. No row return table.
    """
    images = sorted([Piece(p.lo + p.shift, p.hi + p.shift,
                           -p.shift, tuple(reversed(p.word))) for p in pieces],
                    key=lambda p: p.lo)
    # Origin search scans descriptors so its cost cannot hide behind an oracle.
    anchor, phase = origin, 0
    while True:
        retained = False
        for p in pieces:
            counts['origin_membership_tests'] += 1
            if p.lo <= anchor < p.hi:
                retained = True
                break
        if retained:
            break
        anchor = model.forward[anchor]
        counts['origin_lf_visits'] += 1
        phase += 1
        if anchor == origin:
            counts['unanchored_origins'] += 1
            out, x = [], origin
            for _ in range(model.n):
                x = model.inverse[x]
                counts['fallback_inverse_visits'] += 1
                out.append(model.column[x])
            return ''.join(out)
    out = []
    # The first `phase` bytes precede the requested origin, so discard them.
    while len(out) < model.n + phase:
        p = images[locate(images, anchor, counts)]
        counts['inverse_fragment_visits'] += 1
        out.extend(model.runs[i][2] for i in p.word)
        counts['expanded_word_symbols'] += len(p.word)
        anchor += p.shift
    counts['discarded_phase_symbols'] += phase
    counts['discarded_tail_symbols'] += len(out) - phase - model.n
    return ''.join(out[phase:phase + model.n])


def bwt(word):
    starts = sorted(range(len(word)), key=lambda i: word[i:] + word[:i])
    return ''.join(word[(i - 1) % len(word)] for i in starts), starts.index(0)


def witness(model, black, oracle, kind):
    return dict(column=model.column, black=sorted(black), kind=kind,
                top=list(model.top), bottom=list(model.bottom),
                lengths=[hi - lo for lo, hi, _ in model.runs],
                returns={str(x): dict(destination=y, time=len(w), itinerary=w,
                                     bytes=''.join(model.runs[i][2] for i in w))
                         for x, (y, w) in oracle.items()})


def check_column(column, totals, first, expected=None):
    model = LF(column)
    totals['models'] += 1
    totals['input_rows'] += model.n
    totals['input_runs'] += len(model.runs)
    if expected is not None:
        word, origin = expected
        assert model.decode(origin) == word
    # Empty reduction must expose every cycle as unanchored.
    reduction(model, set())
    empty_counts = Counter()
    for origin in range(model.n):
        assert reconstruct(model, [], origin, empty_counts) == model.decode(origin)
    assert empty_counts['unanchored_origins'] == model.n
    totals['empty_retention_origins'] += model.n
    cache = {}
    for mask in range(1, 1 << len(model.runs)):
        black = {i for i in model.top if mask & (1 << i)}
        orders, placement = reduction(model, black)
        oracle = row_returns(model, black)
        pieces, build = induce(model.pieces, model.target(black))
        verify_map(pieces, oracle)
        cache[mask] = (pieces, oracle)
        totals['colorings'] += 1
        totals['retained_runs'] += len(orders[0])
        totals['retained_boundaries'] += len({v for interval in model.target(black)
                                            for v in interval})
        totals['deferred_edge_records'] += len(placement)
        totals['deferred_endpoint_records'] += 2 * len(placement)
        totals['oracle_forward_visits'] += sum(len(w) for _, w in oracle.values())
        totals['checked_return_rows'] += len(oracle)
        totals.update({'build_' + k: v for k, v in build.items()})
        totals['max_fragments_per_coloring'] = max(totals['max_fragments_per_coloring'], len(pieces))
        totals['max_fragments_minus_retained'] = max(totals['max_fragments_minus_retained'], len(pieces) - len(black))
        naive = naive_deleted(model, black)
        mismatches = {
            'deletion_time_or_word': any(w != (model.row_run[x],) for x, (_, w) in oracle.items()),
            'deletion_destination': any(naive[x] != y for x, (y, _) in oracle.items()),
            'positional_lift': any((y - x, w) != (oracle[model.runs[model.row_run[x]][0]][0]
                                                  - model.runs[model.row_run[x]][0],
                                                  oracle[model.runs[model.row_run[x]][0]][1])
                                   for x, (y, w) in oracle.items()),
        }
        for kind, fails in mismatches.items():
            if fails:
                totals['failed_' + kind] += 1
                # Global minimum under (length, column lexicographic, mask).
                key = (model.n, column, mask)
                if kind not in first or key < first[kind][0]:
                    first[kind] = (key, witness(model, black, oracle, kind))
        decode_counts = Counter()
        for origin in range(model.n):
            assert reconstruct(model, pieces, origin, decode_counts) == model.decode(origin)
            totals['checked_origins'] += 1
            totals['oracle_inverse_visits'] += model.n
        totals.update({'decode_' + k: v for k, v in decode_counts.items()})
    # All single-run deletions; subsets are still unions of original runs.
    # Cached lower subset gives independently rebuilt LF result, NOT update code.
    for mask, (pieces, _) in cache.items():
        for i in model.top:
            if not mask & (1 << i) or mask == 1 << i:
                continue
            smaller = mask ^ (1 << i)
            target = model.target({j for j in model.top if smaller & (1 << j)})
            lifted, cost = induce(pieces, target)
            fresh, oracle = cache[smaller]
            verify_map(lifted, oracle)
            assert lifted == fresh
            totals['nested_updates'] += 1
            totals.update({'update_' + k: v for k, v in cost.items()})
            # No access to discarded original runs inside induce(pieces,target).
            totals['update_original_gray_record_lookups'] += 0


def families():
    result = []
    for m in (1, 2, 4, 8, 16, 32, 64):
        for kind, column, black in (
                ('aligned', 'b' * m + 'a' * m, {1}),
                ('long_excursion', 'b' + 'a' * m, {0})):
            model = LF(column)
            pieces, cost = induce(model.pieces, model.target(black))
            oracle = row_returns(model, black)
            verify_map(pieces, oracle)
            decode_counts = Counter()
            for origin in range(model.n):
                assert reconstruct(model, pieces, origin, decode_counts) == model.decode(origin)
            if kind == 'aligned':
                assert cost['base_piece_visits'] == 2
                assert cost['label_symbols_copied'] == 3
            else:
                assert cost['base_piece_visits'] == m + 1
                assert cost['label_symbols_copied'] == (m + 1) * (m + 2) // 2
            result.append(dict(family=kind, m=m, rows=model.n, runs=2,
                               retained_boundaries=2, deferred_edge_records=1,
                               deferred_endpoint_records=2,
                               oracle_forward_visits=sum(len(w) for _, w in oracle.values()),
                               construction=dict(cost), decoding=dict(decode_counts)))
    return result


def main():
    assert __debug__, 'Assertions must be enabled; do not use python -O.'
    suites, first = {}, {}
    for alphabet, bound in (('ab', 7), ('abc', 5)):
        for mode in ('column', 'cyclic_bwt'):
            totals = Counter()
            for n in range(1, bound + 1):
                for letters in product(alphabet, repeat=n):
                    word = ''.join(letters)
                    column, origin = bwt(word) if mode == 'cyclic_bwt' else (word, 0)
                    check_column(column, totals, first,
                                 (word, origin) if mode == 'cyclic_bwt' else None)
            suites[f'{alphabet}_{mode}_through_{bound}'] = dict(totals)
    totals = Counter()
    for base in ('a', 'ab', 'aba', 'abc', 'aab'):
        for power in (2, 3, 5, 8):
            word = base * power
            column, origin = bwt(word)
            check_column(column, totals, first, (word, origin))
    suites['explicit_periodic_bwt'] = dict(totals)
    # Minimality is exhaustive in both alphabets through the witness length.
    assert first['deletion_time_or_word'][0][:2] == (2, 'ba')
    assert first['deletion_destination'][0][:2] == (3, 'baa')
    assert first['positional_lift'][0][:2] == (3, 'baa')
    print(json.dumps(dict(schema=1, suites=suites,
                          smallest_counterexamples={k: v[1] for k, v in first.items()},
                          count_families=families()), indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
