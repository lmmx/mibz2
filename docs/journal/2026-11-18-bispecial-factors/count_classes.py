"""Check finite cyclic type classes independently, and measure the frozen block."""

import csv
from collections import Counter
from functools import cache, reduce
from itertools import product
from math import comb, factorial, gcd, lgamma, log, log2, prod
from pathlib import Path
import sys


def circular_spectrum(text, length):
    """Count length-many consecutive bytes at each position, wrapping as often as needed."""
    rows = text * (1 + (length + len(text) - 2) // len(text))
    return Counter(rows[i : i + length] for i in range(len(text)))


def column_spectrum(last, length):
    """Read the prefix of every rotation directly from stable first-to-last."""
    psi = sorted(range(len(last)), key=last.__getitem__)
    counts = Counter()
    for p in range(len(last)):
        word = bytearray()
        for _ in range(length):
            p = psi[p]
            word.append(last[p])
        counts[bytes(word)] += 1
    return counts


def primitive_column(last):
    """Test whether the standard permutation visits every row from row zero."""
    psi = sorted(range(len(last)), key=last.__getitem__)
    p, steps = psi[0], 1
    while p:
        p = psi[p]
        steps += 1
    return steps == len(last)


def determinant(matrix):
    """Compute an exact integer determinant by fraction-free elimination."""
    rows = [row.copy() for row in matrix]
    if not rows:
        return 1
    previous, sign = 1, 1
    for j in range(len(rows) - 1):
        if rows[j][j] == 0:
            swap = next((i for i in range(j + 1, len(rows)) if rows[i][j]), None)
            if swap is None:
                return 0
            rows[j], rows[swap] = rows[swap], rows[j]
            sign = -sign
        pivot = rows[j][j]
        for i in range(j + 1, len(rows)):
            for h in range(j + 1, len(rows)):
                num = pivot * rows[i][h] - rows[i][j] * rows[j][h]
                rows[i][h] = num // previous
            rows[i][j] = 0
        previous = pivot
    return sign * rows[-1][-1]


def graph(spectrum):
    """Return vertex outdegrees and the number of in-arborescences at one vertex."""
    degrees = Counter()
    for word, count in spectrum.items():
        degrees[word[:-1]] += count
    vertices = sorted(degrees)
    rank = {s: i for i, s in enumerate(vertices)}
    laplacian = [[0] * len(vertices) for _ in vertices]
    for word, count in spectrum.items():
        i, j = rank[word[:-1]], rank[word[1:]]
        laplacian[i][i] += count
        laplacian[i][j] -= count
    trees = determinant([r[1:] for r in laplacian[1:]])
    return degrees, trees


def key(spectrum):
    """Use a canonical immutable key for spectra."""
    return tuple(sorted(spectrum.items()))


@cache
def counts_for(profile):
    """Count arbitrary columns, indexed texts, and primitive one-cycle columns."""
    spectrum = dict(profile)
    degrees, trees = graph(spectrum)
    n = sum(spectrum.values())
    omega = prod(map(factorial, degrees.values())) // prod(map(factorial, spectrum.values()))
    numerator = n * omega * trees
    denominator = prod(degrees.values())
    assert numerator % denominator == 0
    indexed = numerator // denominator
    primitive = indexed
    common = reduce(gcd, spectrum.values())
    for d in range(2, common + 1):
        if common % d == 0:
            divided = tuple((w, v // d) for w, v in profile)
            primitive -= (n // d) * counts_for(divided)[2]
    assert primitive % n == 0
    return omega, indexed, primitive // n


def bwt(text):
    """Compute the cyclic last column directly by sorting all rotations."""
    starts = sorted(range(len(text)), key=lambda i: text[i:] + text[:i])
    return tuple(text[i - 1] for i in starts)


def words_at(text, marker):
    """Return the complete cyclic phrases after a particular delimiter occurrence."""
    d = bytes([marker])
    first = text.index(d) + 1
    rotated = text[first:] + text[:first]
    return [part + d for part in rotated.split(d)[:-1]]


def check_markers():
    """Fix the marker and verify both extremes on the same 70 indexed blocks."""
    rare_books, rare_orders, common_books, common_orders = set(), set(), set(), set()
    for bits in product(b"01", repeat=8):
        if bits.count(ord("1")) != 4:
            continue
        text = b"#|" + b"".join(bytes([x]) + b"|" for x in bits)
        for c, books, orders in [(35, rare_books, rare_orders), (124, common_books, common_orders)]:
            words = words_at(text, c)
            hist = key(Counter(words))
            rank = {w: i for i, (w, _) in enumerate(hist)}
            books.add(hist)
            orders.add(bwt(tuple(rank[w] for w in words)))
    assert (len(rare_books), len(rare_orders)) == (comb(8, 4), 1)
    assert (len(common_books), len(common_orders)) == (1, comb(8, 4))
    print("70 blocks, 70 spelling multisets and one order at #, one multiset and 70 orders at |")


def necklace_set(last):
    """Forget the multiplicities of the necklaces in the stable permutation."""
    psi = sorted(range(len(last)), key=last.__getitem__)
    seen, necklaces = set(), set()
    for i in range(len(last)):
        if i in seen:
            continue
        p, word = i, bytearray()
        while p not in seen:
            seen.add(p)
            p = psi[p]
            word.append(last[p])
        w = bytes(word)
        necklaces.add(min(w[j:] + w[:j] for j in range(len(w))))
    return tuple(sorted(necklaces))


def multiplicities(lengths, n):
    """Count the positive integer multiplicities with fixed total length."""
    remaining = n - sum(lengths)
    ways = [1] + [0] * remaining
    for length in lengths:
        for j in range(length, remaining + 1):
            ways[j] += ways[j - length]
    return ways[remaining]


def exact_checks():
    """Enumerate entire classes, including columns containing incompatible cycles."""
    checks = 0
    support_classes = 0
    for sigma, limit in [(2, 9), (3, 6)]:
        for n in range(1, limit + 1):
            texts = [bytes(v) for v in product(range(sigma), repeat=n)]
            by_support = Counter(necklace_set(v) for v in texts)
            for necklaces, count in by_support.items():
                assert multiplicities([len(w) for w in necklaces], n) == count
                support_classes += 1
            for k in range(4):
                columns, indexed, primitive = Counter(), Counter(), Counter()
                for v in texts:
                    profile = key(column_spectrum(v, k + 1))
                    columns[profile] += 1
                    primitive[profile] += int(primitive_column(v))
                    indexed[key(circular_spectrum(v, k + 1))] += 1
                assert set(indexed) <= set(columns)
                for profile, cnt in columns.items():
                    assert counts_for(profile) == (cnt, indexed[profile], primitive[profile])
                    checks += 1
    print(checks, "complete spectrum classes agree with enumeration")
    print(support_classes, "unweighted support classes have exactly the predicted multiplicities")
    examples = [b"00010111", b"00011101"]
    profiles = [key(circular_spectrum(v, 3)) for v in examples]
    assert profiles[0] == profiles[1] and counts_for(profiles[0]) == (16, 16, 2)
    print(
        "binary order-three de Bruijn spectrum: 16 columns, 16 indexed texts, 2 primitive necklaces"
    )
    check_markers()


def frozen(last, raw, destination):
    """Measure admissible markers and exact one-cycle corrections, using independent inputs."""
    last, raw = Path(last).read_bytes(), Path(raw).read_bytes()
    assert Counter(last) == Counter(raw)
    n = len(last)
    h = Counter(last)
    byte_order = sorted(h)
    destination = Path(destination)
    with (destination / "frozen-markers.csv").open("w", newline="") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(
            (
                "byte",
                "frequency",
                "rle_cut",
                "types",
                "distinct_spelling_bytes",
                "log2_indexed_words_given_phrase_multiset",
            )
        )
        extended = raw + raw[:3]
        for c in byte_order:
            m = h[c]
            counts = Counter(words_at(raw, c))
            log_orders = (lgamma(m + 1) - sum(lgamma(v + 1) for v in counts.values())) / log(2)
            writer.writerow(
                (
                    c,
                    m,
                    int(bytes([c]) * 4 not in extended),
                    len(counts),
                    sum(map(len, counts)),
                    f"{log2(n / m) + log_orders:.6f}",
                )
            )
    one = Counter({bytes([a]): count for a, count in h.items()})
    two = Counter()
    start = 0
    for b in byte_order:
        end = start + h[b]
        for a, count in Counter(last[start:end]).items():
            two[bytes([a, b])] = count
        start = end
    assert two == circular_spectrum(raw, 2)
    print("order,vertices,gcd,trees,log2(all_columns / primitive_columns)")
    for k, spec in enumerate([one, two]):
        degrees, trees = graph(spec)
        assert trees > 0 and reduce(gcd, spec.values()) == 1
        correction = sum(log2(v) for v in degrees.values()) - log2(trees)
        print(k, len(degrees), 1, trees, f"{correction:.9f}", sep=",")


if __name__ == "__main__":
    if len(sys.argv) == 4:
        frozen(*sys.argv[1:])
    elif len(sys.argv) == 1:
        exact_checks()
    else:
        raise SystemExit("count_classes.py [COLUMN PRE_RLE EVIDENCE_DIRECTORY]")
