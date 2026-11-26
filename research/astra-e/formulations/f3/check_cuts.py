"""Independently verify the fast Fenwick and FM queries by explicit permutation walks."""
from itertools import groupby, product
from cuts import screen
from exact import successor, word

cases = 0
for n in range(1, 5):
    for l in product(range(3), repeat=n):
        runs = [(c, sum(1 for _ in it)) for c, it in groupby(l)]
        for p in range(n):
            for q, m, rows, errors, difference, any_mark_matches in screen(p, runs):
                labels = [int(c >= q) for c in sorted(l)]
                full = successor(l)
                binary = successor([int(c >= q) for c in l])
                assert errors == sum(labels[a] != labels[b] for a, b in zip(full, binary))
                wanted = word(labels, full, p, 128)
                outputs = [word(labels, binary, i, 128) for i in range(n)]
                first = next((i for i in range(128) if wanted[i] != outputs[p][i]), None)
                assert first == difference
                assert any_mark_matches == outputs.count(wanted)
                cases += 1
print(f'{cases} marked cut tests passed for the successor-colour count and the actual/any-primary 128-byte tests.')
