"""Exhaustive marked semantics and the word/exit interval theorem, with witnesses."""
from collections import Counter
from itertools import groupby, product
import json

from cursor import emit
from inputs import dense_audit, pieces


def runs_of(column):
    return [(c, len(list(g))) for c, g in groupby(column)]


def forward_bwt(word):
    order = sorted(range(len(word)), key=lambda i: word[i:] + word[:i])
    return bytes(word[i-1] for i in order), order.index(0)


def check_intervals(column, runs, total, witnesses):
    n = len(column)
    successor = sorted(range(n), key=column.__getitem__)
    base = pieces(runs)
    ids = [j for j, (a, b, _, _) in enumerate(base) for _ in range(b-a)]
    labels = [b"" for _ in column]
    routes = [() for _ in column]
    rows = list(range(n))
    full = len(set(cycle(successor, 0))) == n
    for k in range(1, n+3):
        for i, row in enumerate(rows):
            j = ids[row]
            a, b, delta, c = base[j]
            assert a <= row < b and row + delta == successor[row]
            labels[i] += bytes([c])
            routes[i] += (j,)
            rows[i] = successor[row]
        assert labels == sorted(labels)
        total["row_depths"] += n
        for i in range(n):
            for j in range(i+1, n):
                if labels[i] != labels[j]:
                    continue
                total["equal_word_pairs"] += 1
                gap = rows[j] - rows[i]
                assert gap >= j-i
                same = gap == j-i
                assert same == (routes[i] == routes[j])
                if same:
                    assert all(routes[z] == routes[i] for z in range(i, j+1))
                elif full and k == 2 and "same_word_distinct_exit" not in witnesses:
                    witnesses["same_word_distinct_exit"] = dict(
                        L=list(column), rows=[i,j], word=list(labels[i]),
                        exits=[rows[i], rows[j]], displacements=[rows[i]-i, rows[j]-j])
        if full and k == 2 and "different_row_copy" not in witnesses:
            # Tiles encountered in actual primary-0 order, with disjoint destinations.
            used = {}
            x = 0
            for d in range(0, n-1, 2):
                key = (labels[x], rows[x]-x)
                if key in used and used[key][0] != x:
                    y, source = used[key]
                    witnesses["different_row_copy"] = dict(
                        L=list(column), p=0, prior_row=y, row=x, source=source, dest=d,
                        pair=list(labels[x]), prior_exit=rows[y], exit=rows[x],
                        displacement=key[1])
                    break
                used[key] = x, d
                x = rows[x]


def cycle(mapping, p):
    at = p
    while True:
        yield at
        at = mapping[at]
        if at == p:
            return


def main():
    totals, witnesses = Counter(), {}
    for sigma, limit in ((2, 9), (3, 6)):
        for n in range(1, limit+1):
            for t in product(range(sigma), repeat=n):
                col = bytes(t)
                runs = runs_of(col)
                check_intervals(col, runs, totals, witnesses)
                # Construct forward LF independently by stable occurrence numbering.
                counters = [sum(v < c for v in col) for c in range(sigma)]
                lf = []
                for c in col:
                    lf.append(counters[c])
                    counters[c] += 1
                for p in range(n):
                    _, want, _ = dense_audit(runs, p)
                    backward = bytes(col[i] for i in cycle(lf, p))
                    period = len(backward)
                    assert want == (backward[::-1] * ((n+period-1)//period))[:n]
                    totals["marks"] += 1
                    totals["short_cycle"] += period < n
                    totals["partial_last_cycle"] += n % period != 0
                    for delimiter in (0, -1):
                        out, stats = emit(runs, p, delimiter, cap=3)
                        assert out == want
                        totals["cursor_checks"] += 1
                        totals["partial_hits"] += stats["partial_hits"]
                        totals["hits"] += stats["hits"]

    for n in range(1, 9):
        for t in product(range(2), repeat=n):
            word = bytes(t)
            col, p = forward_bwt(word)
            assert emit(runs_of(col), p, 0, cap=3)[0] == word
            totals["text_round_trips"] += 1

    # Several pages, capped records, repeated rotations and a final partial tile.
    for col in (b"x"*260, b"b"*130+b"a"*130, bytes(range(128))*3):
        runs = runs_of(col)
        for p in (0, 63, 64, len(col)-1):
            _, want, _ = dense_audit(runs, p)
            for delim in (-1, ord("b")):
                assert emit(runs,p,delim)[0] == want
                totals["wide_cursor_checks"] += 1
    witnesses["phase_if_writing_back_from_n"] = dict(
        L=[0,1,0], p=1, period=2, backward_from_end=[1,0,1], marked=[0,1,0])
    assert set(witnesses) == {"phase_if_writing_back_from_n","same_word_distinct_exit",
                              "different_row_copy"}
    print(json.dumps(dict(counts=dict(totals), witnesses=witnesses), indent=2))


if __name__ == "__main__":
    main()
