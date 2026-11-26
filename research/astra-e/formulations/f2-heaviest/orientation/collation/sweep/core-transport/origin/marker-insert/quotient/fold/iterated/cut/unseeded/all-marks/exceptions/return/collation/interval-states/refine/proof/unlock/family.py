"""Check the unbounded simultaneous-optimum family at small and large depths."""
from collections import Counter
from fractions import Fraction
import json
from oracle import example


def check(k, exhaustive):
    n = 3*k
    word = '1011'+'0'*(n-4)
    bound = n-3
    psi = sorted(range(n), key=word.__getitem__)
    orbit = [0]
    while psi[orbit[-1]] != 0:
        orbit.append(psi[orbit[-1]])
    assert len(orbit) == n
    cuts = frozenset(range(1, n, 3))
    old = {bound}
    universe = set(range(1, n)) - old
    supports = {t: set(range(psi[t-1]+1, psi[t]+1)) for t in universe}
    assert supports[1] == {2, 3, 4}
    assert all(supports[t] == {t+3} for t in range(2, bound))
    assert supports[bound+1] == {1, 2} and supports[bound+2] == {3}
    assert all((old|cuts)&supports[t] == {1 if t==bound+1 else t+3} for t in cuts)
    forced_two = set(range(2, n, 3)) | set(range(3, bound, 3))
    assert len(forced_two) == 2*k-2 and forced_two | cuts == universe
    def mass(seams):
        b = [0, *sorted(old|seams), n]
        return sum(max(Counter(word[l:r]).values()) for l,r in zip(b,b[1:]))
    assert (mass(set()), mass(cuts), mass(universe)) == (n-3, n-1, n)
    if exhaustive:
        verified = example(word, Fraction(1,k))
        assert verified['best']['neutral'] == {'gain':'1','choices':[sorted(cuts)]}
        return {'k':k,'single_best_gain':verified['best']['one_at_a_time']['gain'],
                'unique_simultaneous_optimum':sorted(cuts)}
    return None


if __name__ == '__main__':
    small = [check(k, True) for k in range(3,7)]
    for k in [*range(7,129), 1000, 10000]:
        check(k,False)
    print(json.dumps({'exhaustive':small,'structural_regular_depths':[3,128],
                      'structural_large_depths':[1000,10000]}, indent=2))
