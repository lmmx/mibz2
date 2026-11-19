"""Exhaustive marked outputs and all retained arrows against an independent row oracle."""

from collections import Counter
from itertools import groupby, permutations, product
from random import Random
import json

from diagonal import census
from factor import choose, factor_decode, phases, pure_rotation
from queue_ranges import contract, decode
from packets import contract as grouped_tiles
from residual import induce, original, quotient
from screen import bands_from_records, tiles


def records(column):
    tape = bytearray()
    for c, run in groupby(column):
        w = sum(1 for _ in run)
        while w:
            take = min(w, 127)
            tape.extend((take, c))
            w -= take
    return tape


def oracle(column, p):
    # Independently sort row numbers by their last bytes to build Psi, not LF runs.
    psi = sorted(range(len(column)), key=column.__getitem__)
    lf = [0] * len(psi)
    for i, x in enumerate(psi):
        lf[x] = i
    row = p
    word = bytearray()
    period = 0
    for _ in column:
        row = psi[row]
        word.append(column[row])
        if not period and row == p:
            period = len(word)
    return bytes(word), period, lf


def check_arrows(arcs, kept, lf, column, p, marked, offset):
    ids = {x: i for i, x in enumerate(kept)}
    covered = 0
    for src, dst, width, word in arcs:
        assert src == covered
        covered += width
        for i in range(width):
            y = kept[src + i]
            trace = []
            while True:
                trace.append(y)
                y = lf[y]
                if y in ids:
                    break
            assert ids[y] == dst + i and bytes(column[j] for j in trace) == word
            if src + i == marked:
                assert trace[offset] == p
    assert covered == len(kept)


def tile_kept(column, lf, p, b):
    n = len(column)
    kept = []
    for x in range(0, n, b):
        rows = range(x, min(n, x + b))
        clean = (
            len(rows) == b
            and lf[x] % b == 0
            and p not in rows
            and all(lf[j] == lf[x] + j - x and column[j] == column[x] for j in rows)
        )
        if not clean:
            kept.extend(rows)
    return kept


def check_phase_order():
    # Sparse exceptions, two tile cycles, nonzero lap shifts and nontrivial gcd.
    perm, shifts, b = [2, 3, 0, 1], [2, 5, 2, 3], 6
    q = lambda x: perm[x // b] * b + (x % b + shifts[x // b]) % b
    pairs = 0
    for e in ({0, 1, 5, 8, 11, 14, 17, 20}, {2}, {0, 3, 6, 9, 12, 15, 18, 21}, set(range(24))):
        succ, _ = phases(perm, shifts, b, e)
        for x in e:
            y, steps = q(x), 1
            while y not in e:
                y, steps = q(y), steps + 1
            assert succ[x] == (y, steps)
            pairs += 1
    return pairs


def check_bounds():
    cases = loose_arbitrary = loose_shift = 0
    for n in range(1, 8):
        for column in product(range(2), repeat=n):
            _, _, lf = oracle(column, 0)
            bands = bands_from_records(records(column))
            for width in [2, 3, 4]:
                k = (n + width - 1) // width
                any_best = shift_best = 0
                for perm in permutations(range(k)):
                    lanes = [[0] * width for _ in perm]
                    for x, y in enumerate(lf):
                        if perm[x // width] == y // width:
                            lanes[x // width][(y - x) % width] += 1
                    any_best = max(any_best, sum(sum(row) for row in lanes))
                    shift_best = max(shift_best, sum(max(row) for row in lanes))
                bound = tiles(bands, n, width)
                assert n - any_best >= bound["arbitrary_min_errors"]
                assert n - shift_best >= bound["shift_min_errors"]
                loose_arbitrary += n - any_best > bound["arbitrary_min_errors"]
                loose_shift += n - shift_best > bound["shift_min_errors"]
                cases += 1
    return dict(cases=cases, loose_arbitrary=loose_arbitrary, loose_shift=loose_shift)


def stress():
    rng = Random(19831)
    tested = 0
    for i in range(120):
        n = rng.randrange(17, 150)
        alphabet = [0, 7, 127, 128, 255][: 2 + i % 4]
        column = [rng.choice(alphabet) for _ in range(n)]
        if i % 3 == 1:
            # BWT of a real cyclic word, tie-stable even for a repeated word.
            rotations = sorted(range(n), key=lambda j: column[j:] + column[:j])
            column = [column[j - 1] for j in rotations]
        elif i % 3 == 2:
            column = sorted(column, reverse=True)
            for _ in range(i % 7):
                a, b = rng.randrange(n), rng.randrange(n)
                column[a], column[b] = column[b], column[a]
        bands = bands_from_records(records(column))
        size = rng.choice([3, 5, 7, 16, 32])
        perm, shifts = choose(bands, n, size)
        for p in {0, n - 1, rng.randrange(n)}:
            want, period, lf = oracle(column, p)
            got, row = decode(bands, n, p)
            assert got == want and row["period"] == period
            s, mp, arcs, _ = grouped_tiles(bands, n, p, size)
            assert (s, mp, arcs) == quotient(bands, n, p, size)
            check_arrows(arcs, tile_kept(column, lf, p, size), lf, column, p, mp, 0)
            assert induce(arcs, s, mp, n)[0] == want
            assert factor_decode(bands, n, p, size, perm, shifts) == (want, period)
            tested += 1
    return tested


def main():
    hits = Counter()
    witnesses = {}

    def witness(key, column, p, **facts):
        witnesses.setdefault(key, dict(column="".join(map(str, column)), primary=p, **facts))

    for alphabet, limit in [(2, 9), (3, 6)]:
        for n in range(1, limit + 1):
            for column in product(range(alphabet), repeat=n):
                bands = bands_from_records(records(column))
                candidates = {b: choose(bands, n, b) for b in (2, 3, 4)}
                for p in range(n):
                    hits["marked_cases"] += 1
                    want, period, lf = oracle(column, p)
                    baseline, base = induce(original(bands), n, p, n)
                    assert want == baseline and period == base["period"]
                    hits["short_periods"] += period < n
                    hits["nondividing_periods"] += n % period != 0
                    hits["equal_cuts"] += base["equal"]
                    hits["unmarked_closures"] += base["closures"] - 1
                    if n % period:
                        witness("nondividing_period", column, p, period=period, output=list(want))
                    if base["closures"] > 1 and base["equal"]:
                        witness(
                            "unmarked_closure_then_mark_and_equality",
                            column,
                            p,
                            period=period,
                            closures=base["closures"],
                            equal=base["equal"],
                        )
                    rot = pure_rotation(bands, n, p)
                    if rot is not None:
                        assert rot == want
                        hits["pure_rotations"] += 1
                    (s, mark, off, arcs), _ = contract(bands, n, p)
                    kept = [x for x in range(n) if column[x] != sorted(column)[x]]
                    if s:
                        assert s == len(kept)
                        check_arrows(arcs, kept, lf, column, p, mark, off)
                        if off:
                            hits["interior_marks"] += 1
                            witness(
                                "mark_inside_queue_word", column, p, lf_offset=off, period=period
                            )
                    else:
                        assert lf[p] == p
                    got, qr = decode(bands, n, p)
                    assert got == want and qr["period"] == period
                    for b, (perm, shift) in candidates.items():
                        s, mark, arcs = quotient(bands, n, p, b)
                        gs, gm, ga, _ = grouped_tiles(bands, n, p, b)
                        assert (gs, gm, ga) == (s, mark, arcs)
                        kept = tile_kept(column, lf, p, b)
                        assert s == len(kept)
                        check_arrows(arcs, kept, lf, column, p, mark, 0)
                        got, row = induce(arcs, s, mark, n)
                        assert got == want and row["period"] == period
                        if s < n and row["cuts"] > base["cuts"]:
                            witness(
                                "smaller_support_more_cuts",
                                column,
                                p,
                                B=b,
                                support=s,
                                before=base["cuts"],
                                after=row["cuts"],
                            )
                        assert factor_decode(bands, n, p, b, perm, shift) == (want, period)
                        k = len(perm)
                        assert factor_decode(
                            bands,
                            n,
                            p,
                            b,
                            [(j + 1) % k for j in range(k)],
                            [(2 * j + 1) % b for j in range(k)],
                        ) == (want, period)
                        hits["tile_quotients"] += 1
                        hits["reference_factors"] += 2
    col = [1, 1, 0, 0, 0]
    _, _, lf = oracle(col, 0)
    first = sorted(col)
    enter = sorted(lf[x] for x in range(len(col)) if col[x] == 0 and first[x] != 0)
    exits = []
    for x in enter:
        while col[x] == 0:
            x = lf[x]
        exits.append(x)
    assert enter == [1, 2] and exits == [1, 0]
    witness("entrance_order_is_not_exit_order", col, 0, byte=0, enters=enter, exits=exits)
    n = 4096
    col = [1] * (n - 1) + [0]
    p = n - 2
    bands = bands_from_records(records(col))
    want, _, _ = oracle(col, p)
    assert pure_rotation(bands, n, p) == want == bytes([1]) * (n - 2) + bytes([0, 1])
    _, base = induce(original(bands), n, p, n)
    got, qr = decode(bands, n, p)
    assert got == want and base["cuts"] == n - 1 and qr["cuts"] == 1
    family = dict(
        n=n,
        primary=p,
        records=len(records(col)) // 2,
        cuts=base["cuts"],
        queue_residual=qr["cuts"],
        period=qr["period"],
        **census(bands),
    )
    print(
        json.dumps(
            dict(
                counts=dict(hits),
                phase_successors=check_phase_order(),
                stress_marks=stress(),
                tile_oracle_bounds=check_bounds(),
                witnesses=witnesses,
                comb=family,
            ),
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
