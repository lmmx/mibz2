"""Count compatible primitive necklaces from a cyclic k-mer spectrum on one primitive block."""

from collections import Counter
from functools import reduce
from hashlib import sha256
from itertools import permutations, product
from math import factorial, gcd, prod
from pathlib import Path
import sys

from count_classes import bwt, circular_spectrum, counts_for, determinant, key


class CyclicIndex:
    """Build an independent cyclic suffix array and its adjacent longest common prefixes."""

    def __init__(self, last, raw, primary):
        self.last = last
        n = len(last)
        assert n == len(raw) and reduce(gcd, Counter(last).values()) == 1
        psi = sorted(range(n), key=last.__getitem__)
        sa, isa = [-1] * n, [-1] * n
        row = primary
        for i in range(n):
            assert sa[row] == -1
            sa[row], isa[i] = i, row
            row = psi[row]
            assert last[row] == raw[i]
        assert row == primary
        self.sa = sa
        doubled = raw + raw
        lcp, h = [0] * n, 0
        for i, rank in enumerate(isa):
            if rank == 0:
                h = 0
                continue
            j = sa[rank - 1]
            while h < n and doubled[i + h] == doubled[j + h]:
                h += 1
            assert h < n
            lcp[rank] = h
            h = max(h - 1, 0)
        self.lcp = lcp
        self.run_ends = [i for i in range(1, n) if last[i] != last[i - 1]]

    def mixed(self, k):
        """Find all k-prefix intervals containing at least two preceding symbols."""
        groups = []
        end = 0
        for i in self.run_ends:
            if i < end or self.lcp[i] < k:
                continue
            lo, end = i - 1, i + 1
            while lo and self.lcp[lo] >= k:
                lo -= 1
            while end < len(self.last) and self.lcp[end] >= k:
                end += 1
            groups.append((lo, end))
        return groups

    def classes(self, k):
        """Contract deterministic paths and evaluate the directed tree count exactly."""
        groups = self.mixed(k)
        if not groups:
            return 0, 0, 1, 1
        visits = []
        omega, product_degrees = 1, 1
        for group, (lo, hi) in enumerate(groups):
            hist = Counter(self.last[lo:hi])
            omega *= factorial(hi - lo) // prod(map(factorial, hist.values()))
            product_degrees *= hi - lo
            visits.extend((self.sa[row], group) for row in range(lo, hi))
        # LF moves one position backwards in this primitive text.
        visits.sort(reverse=True)
        graph = [[0] * len(groups) for _ in groups]
        for i, (_, u) in enumerate(visits):
            v = visits[(i + 1) % len(visits)][1]
            graph[u][u] += 1
            graph[u][v] -= 1
        trees = determinant([r[1:] for r in graph[1:]])
        numerator = omega * trees
        assert numerator % product_degrees == 0
        return len(groups), len(visits), omega, numerator // product_degrees


def small_check():
    """Check contraction and cyclic LCP against direct k-mer counts and enumeration."""
    tested = 0
    for n in range(1, 9):
        for bits in product(range(2), repeat=n):
            raw = bytes(bits)
            if reduce(gcd, Counter(raw).values()) != 1:
                continue
            rotations = sorted(range(n), key=lambda i: raw[i:] + raw[:i])
            index = CyclicIndex(bytes(bwt(raw)), raw, rotations.index(0))
            assert rotations == index.sa
            for k in range(n + 1):
                expected, _, primitive = counts_for(key(circular_spectrum(raw, k + 1)))
                assert index.classes(k)[2:] == (expected, primitive)
                tested += 1
    print(tested, "contracted graph counts agree with the uncontracted exact count")


def measure(last, raw, primary):
    """Find the first spectrum that identifies the primitive necklace on the supplied block."""
    index = CyclicIndex(Path(last).read_bytes(), Path(raw).read_bytes(), int(primary))
    last_split = max(index.lcp)
    samples = {}

    def counts(k):
        if k not in samples:
            samples[k] = index.classes(k)
        return samples[k]

    # Check order zero analytically: a primitive composition with several possible necklaces.
    hist = Counter(index.last)
    assert len(hist) >= 3 and max(hist.values()) > 1
    lo, hi = 0, last_split + 1
    while hi - lo > 1:
        mid = (hi + lo) // 2
        if counts(mid)[-1] == 1:
            hi = mid
        else:
            lo = mid
    for k in (lo, hi, last_split, last_split + 1):
        counts(k)
    assert counts(lo)[-1] > 1 and counts(hi)[-1] == 1
    print("k,spectrum_length,branch_vertices,visits,all_columns,primitive_necklaces")
    for k, (vertices, visits, omega, necklaces) in sorted(samples.items()):
        print(k, k + 1, vertices, visits, omega, necklaces, sep=",")


def witnesses(last_path, raw_path, primary, k, alternate_path):
    """Enumerate both sides of the frozen block's one-cycle identifiability threshold."""
    last, raw, primary, k = (
        Path(last_path).read_bytes(),
        Path(raw_path).read_bytes(),
        int(primary),
        int(k),
    )
    index = CyclicIndex(last, raw, primary)
    found = 0
    print(
        "k,spectrum_length,interval_start,ordered_predecessors_hex,cycle_lengths,marked_raw_sha256"
    )
    for order in (k, k + 1):
        [(lo, hi)] = index.mixed(order)
        assert hi - lo <= 3
        for values in sorted(set(permutations(last[lo:hi]))):
            piece = bytes(values)
            other = last[:lo] + piece + last[hi:]
            psi = sorted(range(len(other)), key=other.__getitem__)
            seen, lengths = bytearray(len(other)), []
            for i in range(len(other)):
                if seen[i]:
                    continue
                p, d = i, 0
                while not seen[p]:
                    seen[p] = 1
                    p, d = psi[p], d + 1
                lengths.append(d)
            digest = ""
            if len(lengths) == 1:
                p, output = primary, bytearray()
                for _ in other:
                    p = psi[p]
                    output.append(other[p])
                digest = sha256(output).hexdigest()
                if other == last:
                    assert output == raw
                else:
                    assert order == k
                    Path(alternate_path).write_bytes(output)
                    found += 1
            print(
                order,
                order + 1,
                lo,
                piece.hex(),
                "+".join(map(str, sorted(lengths))),
                digest,
                sep=",",
            )
    assert found == 1


if __name__ == "__main__":
    if len(sys.argv) == 1:
        small_check()
    elif sys.argv[1] == "witness":
        witnesses(*sys.argv[2:])
    else:
        measure(*sys.argv[1:])
