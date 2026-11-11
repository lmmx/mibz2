"""Exhaustive mathematical sanity checks, not an inverse-BWT performance test.

For every permutation on 1..6 points, check non-adjacent deletion surgery,
its forward-travel word labels, and conjugacy under coordinate compaction.
For permutations on 1..5 points, also check nested first-return composition.
No third-party packages, network access, or repository changes.
"""
from itertools import permutations
import json


def induce(f, labels, keep):
    """Exact first-return oracle on a nonempty subset of a finite permutation."""
    out, words = {}, {}
    for x in keep:
        y, word = f[x], labels[x]
        for _ in range(len(f)):
            if y in keep:
                break
            word = word + labels[y]
            y = f[y]
        else:
            raise AssertionError('A finite permutation must return to its start subset')
        out[x], words[x] = y, word
    return out, words


def subsets(domain):
    domain = tuple(domain)
    for mask in range(1 << len(domain)):
        yield {x for i, x in enumerate(domain) if (mask >> i) & 1}


def main():
    stats = dict(permutations_for_splice=0, valid_deletion_sets=0,
                 nonempty_valid_deletion_sets=0, checked_splice_transitions=0,
                 nested_subset_cases=0, mismatches=0)
    for n in range(1, 7):
        universe = set(range(n))
        for perm in permutations(range(n)):
            f = dict(enumerate(perm))
            # Unique symbol at every point checks full itineraries, not merely counts.
            labels = {x: (x,) for x in universe}
            stats['permutations_for_splice'] += 1
            for removed in subsets(range(n)):
                if removed.intersection(f[x] for x in removed):
                    continue
                keep = universe - removed
                if not keep:
                    continue
                expected_f, expected_w = induce(f, labels, keep)
                actual_f = {x: f[f[x]] if f[x] in removed else f[x] for x in keep}
                actual_w = {x: labels[x] + labels[f[x]] if f[x] in removed else labels[x]
                            for x in keep}
                assert (actual_f, actual_w) == (expected_f, expected_w)
                stats['valid_deletion_sets'] += 1
                stats['nonempty_valid_deletion_sets'] += bool(removed)
                stats['checked_splice_transitions'] += len(keep)
                rank = {x: j for j, x in enumerate(sorted(keep))}
                packed = {rank[x]: rank[actual_f[x]] for x in keep}
                packed_words = {rank[x]: actual_w[x] for x in keep}
                assert sorted(packed.values()) == list(range(len(keep)))
                for origin in keep:
                    x, y = origin, rank[origin]
                    sparse_word, packed_word = (), ()
                    while True:
                        sparse_word += actual_w[x]
                        packed_word += packed_words[y]
                        x, y = actual_f[x], packed[y]
                        assert y == rank[x]
                        if x == origin:
                            break
                    assert sparse_word == packed_word
            if n <= 5:
                for large in subsets(range(n)):
                    if not large:
                        continue
                    g, gw = induce(f, labels, large)
                    for small in subsets(sorted(large)):
                        if not small:
                            continue
                        assert induce(g, gw, small) == induce(f, labels, small)
                        stats['nested_subset_cases'] += 1
    stats['scope'] = ('All permutations n=1..6 for local splice/compaction; '
                      'all nonempty nested subsets for permutations n=1..5. '
                      'Finite-state correctness only; no BWT run representation, '
                      'decoder integration, memory accounting or performance measured.')
    print(json.dumps(stats, indent=2))

if __name__ == '__main__':
    main()
