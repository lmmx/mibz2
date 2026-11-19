"""Exhaustive check of Neon's Theorem 1 (the discontinuity budget) on small BWTs.

For every ternary column L with n <= 7 and every section X' that meets every LF cycle,
count the maximal pieces of the first-return map on X' (by map and by return word together)
and compare them with r + (c - 1) + |boundaries of X' outside D(T) and C(T)|. Also replays the
response's 7-cycle tightness example (L = ccabbbb). Run: python3 check_theorem1.py
"""

import itertools


def lf(col):
    """Return the stable-rank LF permutation of a BWT last column."""
    first = {c: sum(1 for x in col if x < c) for c in set(col)}
    seen = {}
    t = []
    for c in col:
        t.append(first[c] + seen.get(c, 0))
        seen[c] = seen.get(c, 0) + 1
    return t


def covered(t, section):
    """Return the rows of every LF cycle that meets the section."""
    seen = [False] * len(t)
    rows = set()
    for start in range(len(t)):
        if seen[start]:
            continue
        cycle, x = [], start
        while not seen[x]:
            seen[x] = True
            cycle.append(x)
            x = t[x]
        if any(y in section for y in cycle):
            rows |= set(cycle)
    return rows


def induced(t, col, section):
    """Return the first-return map on the section (compressed coordinates) and its words."""
    rows = sorted(section)
    index = {x: i for i, x in enumerate(rows)}
    targets, words = [], []
    for x in rows:
        y, word = x, []
        while True:
            word.append(col[y])
            y = t[y]
            if y in section:
                break
        targets.append(index[y])
        words.append(tuple(word))
    return targets, words


def pieces(targets, words):
    """Count maximal intervals on which the map is a translation with one return word."""
    breaks = sum(
        1
        for i in range(1, len(targets))
        if not (targets[i] == targets[i - 1] + 1 and words[i] == words[i - 1])
    )
    return 1 + breaks


def bound(t, col, section):
    """Return Theorem 1's bound r + (c - 1) + |internal boundaries outside D and C|."""
    n = len(t)
    domain_cuts = {i for i in range(1, n) if col[i] != col[i - 1]}
    inverse = [0] * n
    for i, j in enumerate(t):
        inverse[j] = i
    codomain_cuts = {j for j in range(1, n) if inverse[j] != inverse[j - 1] + 1}
    rows = sorted(section)
    components = 1 + sum(1 for i in range(1, len(rows)) if rows[i] != rows[i - 1] + 1)
    boundaries = {i for i in range(1, n) if (i - 1 in section) != (i in section)}
    runs = 1 + len(domain_cuts)
    return runs + (components - 1) + len(boundaries - (domain_cuts | codomain_cuts))


def main():
    """Replay the 7-cycle example, then check every small case."""
    col = list("ccabbbb")
    t = lf(col)
    section = {0, 1, 3, 4, 5, 6}
    targets, words = induced(t, col, section)
    print("7-cycle example: LF", t, "induced", targets)
    print("  pieces", pieces(targets, words), "bound", bound(t, col, section))

    checked = violations = 0
    for n in range(2, 8):
        for col in itertools.product("abc", repeat=n):
            t = lf(col)
            for mask in range(1, 2**n - 1):
                section = {i for i in range(n) if mask >> i & 1}
                if len(covered(t, section)) != n:
                    continue
                targets, words = induced(t, col, section)
                checked += 1
                if pieces(targets, words) > bound(t, col, section):
                    violations += 1
    print("checked", checked, "violations", violations)


if __name__ == "__main__":
    main()
