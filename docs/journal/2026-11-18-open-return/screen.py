"""Dense, independent structural controls. The timed decoder never calls these."""
import argparse
from collections import Counter, deque
from itertools import accumulate
from pathlib import Path


def lf(column):
    counts = Counter(column)
    cursor = [0, *accumulate(counts[b] for b in range(256))]
    result = []
    for b in column:
        result.append(cursor[b])
        cursor[b] += 1
    return result


def copy_screen(column, primary):
    n = len(column)
    permutation = lf(column)
    phase = [-1] * n
    text = bytearray(n)
    row = primary
    for pos in reversed(range(n)):
        assert phase[row] < 0, "this census uses a primitive column"
        phase[row] = pos
        text[pos] = column[row]
        row = permutation[row]
    assert row == primary
    heads = [i for i in range(n) if not i or column[i] != column[i - 1]]
    # Phi takes the row to its lexicographic predecessor.
    pairs = sorted((phase[s], phase[(s - 1) % n]) for s in heads)
    pieces = []
    arcs = []
    for (a, b), (stop, _) in zip(pairs, pairs[1:] + pairs[:1]):
        length = (stop - a) % n - 1
        if length <= 0:
            continue
        node = len(pieces)
        pieces.append(length)
        a, b = (a + 1) % n, (b + 1) % n
        while length:
            size = min(length, n - a, n - b)
            assert text[a:a + size] == text[b:b + size]
            arcs.append((a, b, size, node))
            length -= size
            a, b = (a + size) % n, (b + size) % n
    dest = sorted(arcs)
    source = sorted((b, a, size, node) for a, b, size, node in arcs)
    dependencies = [set() for _ in pieces]
    j = 0
    for start, _, size, target in source:
        while j < len(dest) and dest[j][0] + dest[j][2] <= start:
            j += 1
        q = j
        while q < len(dest) and dest[q][0] < start + size:
            other = dest[q][3]
            if other != target:
                dependencies[target].add(other)
            q += 1
    users = [[] for _ in pieces]
    for target, sources in enumerate(dependencies):
        for src in sources:
            users[src].append(target)
    ready = deque(i for i, ds in enumerate(dependencies) if not ds)
    settled = []
    while ready:
        src = ready.popleft()
        settled.append(src)
        for target in users[src]:
            dependencies[target].remove(src)
            if not dependencies[target]:
                ready.append(target)
    print(f"rows={n} heads={len(heads)} copy_spans={len(pieces)} copy_bytes={sum(pieces)}")
    print(f"settled_spans={len(settled)} settled_bytes={sum(pieces[i] for i in settled)}")


def twins():
    widths = [2, 2, 1, 1, 1, 1, 3, 1]
    for labels in ["20210210", "10102021"]:
        column = bytes(int(c) for c, w in zip(labels, widths) for _ in range(w))
        f = lf(column)
        x, orbit = 0, []
        while x not in orbit:
            orbit.append(x)
            x = f[x]
        assert x == 0 and len(orbit) == len(column)
        longest = sum(widths[:6])
        # The three thick rows are the only ones outside the section.
        gap = abs(f[longest] - longest)
        assert len(set(f[i] - i for i in range(longest, longest + 3))) == 1
        entering = min(3, gap)
        walks = []
        for v in range(longest, longest + 3):
            if f.index(v) in range(longest, longest + 3):
                continue
            z = v
            height = 0
            while longest <= z < longest + 3:
                z = f[z]
                height += 1
            walks.append(height)
        print(f"L={''.join(map(str,column))} primary=0 displacement={f[longest]-longest} "
              f"entries={entering} heights={walks} literal_work_per_parallel_band={walks[0]}")
        assert len(walks) == entering and 3 == sum(walks) and len(set(walks)) == 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("column", nargs="?")
    parser.add_argument("primary", nargs="?", type=int)
    args = parser.parse_args()
    twins()
    if args.column:
        copy_screen(Path(args.column).read_bytes(), args.primary)
