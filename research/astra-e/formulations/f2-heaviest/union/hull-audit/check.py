"""Dense oracle and occupied-run grouping on arbitrary marked stable LF permutations."""

from collections import Counter, defaultdict
from itertools import groupby, product
import json


def geometry(last):
    first = sorted(last)
    psi = sorted(range(len(last)), key=last.__getitem__)
    lf = [0] * len(last)
    for x, y in enumerate(psi):
        lf[y] = x
    runs = []
    end = [0] * len(last)
    for _, group in groupby(range(len(last)), key=last.__getitem__):
        rows = list(group)
        runs.append(rows[0])
        for x in rows:
            end[x] = rows[-1] + 1
    return first, psi, lf, end, len(runs)


def prefix(x, steps, letters, count):
    chars = []
    for _ in range(count):
        chars.append(letters[x])
        x = steps[x]
    return bytes(chars)


def first_return(x, section, psi, first):
    y = x
    word = []
    while True:
        word.append(first[y])
        y = psi[y]
        if y in section:
            return bytes(word), y


def ranges(rows):
    return [
        (g[0], g[-1] + 1, j)
        for j, (_, it) in enumerate(groupby(enumerate(rows), key=lambda v: v[1] - v[0]))
        for g in [[x for _, x in it]]
    ]


def hit(word, marker, chosen):
    trailer = word + marker
    if not trailer.startswith(marker):
        return False
    i = trailer.index(marker, 1)
    assert i <= len(word)
    return word[:i] in chosen


def group_children(last, lf, end, sections):
    # Visit the next occupied row directly, including when an entire gap is skipped.
    child = defaultdict(list)
    selector = []
    probes = 0
    overlays = 0
    run_end = -1
    b = None
    displacement = None
    for lo, hi, c in sections:
        while lo < hi:
            if lo >= run_end:
                b, run_end, displacement = last[lo], end[lo], lf[lo] - lo
                probes += 1
            stop = min(run_end, hi)
            overlays += 1
            if selector and selector[-1][0] == b:
                selector[-1][1] += stop - lo
            else:
                selector.append([b, stop - lo])
            x = lo + displacement
            y = x + stop - lo
            out = child[b]
            if out and out[-1][2] == c:
                assert out[-1][1] == x
                out[-1] = (out[-1][0], y, c)
            else:
                assert not out or (out[-1][1] <= x and out[-1][2] < c)
                out.append((x, y, c))
            lo = stop
    return child, selector, probes, overlays


def discover(last, marker, chosen, roots, utility=None):
    first, psi, lf, end, nruns = geometry(last)
    selected = {x for lo, hi, c in roots for x in range(lo, hi)}
    colour = {x: c for lo, hi, c in roots for x in range(lo, hi)}

    def witness(name, word, sections, extra):
        if utility is not None and name not in utility:
            utility[name] = dict(
                last=list(last),
                marker=list(marker),
                selected=[list(x) for x in sorted(chosen)],
                word=list(word),
                occupied=sections,
                **extra,
            )

    seen = set()
    nodes = {}
    term = {}
    buckets = {}
    stats = Counter()

    def holes(pieces):
        return pieces[-1][1] - pieces[0][0] - sum(hi - lo for lo, hi, _ in pieces)

    initial_holes = holes(roots)
    stack = [(b"", roots)]
    while stack:
        suffix, sections = stack.pop()
        live = [x for lo, hi, _ in sections for x in range(lo, hi)]
        assert not seen.intersection(live)
        seen.update(live)
        child, selector, j, m = group_children(last, lf, end, sections)
        stats.update(D=1, C=len(sections), J=j, M=m, S=len(selector))
        lost_holes = holes(sections) - sum(holes(pieces) for pieces in child.values())
        assert lost_holes >= 0
        if len(child) == 1:
            assert j - 1 <= lost_holes
            stats["unary_hidden"] += j - 1
        elif j - len(selector) > lost_holes:
            witness(
                "branch_hidden_exceeds_local_hole_loss",
                suffix,
                sections,
                dict(full_run_lookups=j, packed_runs=len(selector), hole_loss=lost_holes),
            )
        stats["total_hidden"] += j - len(selector)
        stats["hull_runs"] += 1 + sum(
            a != b for a, b in zip(last[live[0] : live[-1]], last[live[0] + 1 : live[-1] + 1])
        )
        hull = range(live[0], live[-1] + 1)
        if utility is not None:
            for x in hull:
                assert prefix(x, psi, first, len(suffix) + len(marker)) == suffix + marker
            if len(live) > 1 and len({last[i] for i in live}) == 1 and len({last[i] for i in hull}) > 1:
                witness(
                    "same_occupied_byte_distinct_displacements",
                    suffix,
                    sections,
                    dict(displacements=[lf[i] - i for i in live], hull_last=[last[i] for i in hull]),
                )
            for x in set(hull) - set(live):
                y = x
                while y not in selected:
                    y = lf[y]
                    if y == x:
                        break
                if y not in selected:
                    witness("hull_contains_disjoint_primary", suffix, sections, dict(mark=x))
                    if suffix:
                        witness("nonempty_hull_contains_disjoint_primary", suffix, sections, dict(mark=x))
        assert sorted(child) == sorted({last[x] for x in live})
        nodes[suffix] = selector
        for b, pieces in child.items():
            u = bytes([b]) + suffix
            target = hit(u, marker, chosen)
            ordered = []
            for x, y, c in pieces:
                src = [t for lo, hi, col in sections if col == c for t in range(lo, hi) if last[t] == b]
                assert [lf[t] for t in src] == list(range(x, y))
                if utility is not None and len(src) > 1 and src[-1] - src[0] + 1 > len(src):
                    witness(
                        "dense_ordinal_is_not_destination_rank",
                        suffix,
                        sections,
                        dict(byte=b, colour=c, old_rows=src, new_interval=[x, y]),
                    )
                assert all((i in selected) == target for i in range(x, y))
                ordered.extend([(u, c)] * (y - x))
                if target:
                    assert (u, c) not in buckets
                    buckets[u, c] = (x, y)
            if target:
                term[u] = ordered
            else:
                stack.append((u, pieces))
    stats["root_holes"] = initial_holes
    assert stats["unary_hidden"] <= initial_holes
    if stats["total_hidden"] > initial_holes:
        witness(
            "branch_hidden_exceeds_global_hole_mass",
            b"",
            roots,
            dict(total_hidden=stats["total_hidden"], root_holes=initial_holes),
        )
    assert stats["D"] <= stats["J"] <= stats["M"] <= stats["C"] + nruns - 1
    assert stats["S"] <= stats["J"] and stats["M"] <= len(roots) * stats["J"]
    # Stable bottom-up merge. For a fixed byte its child is exactly the
    # matching subsequence of the packed parent, even when positions have holes.
    incoming = term
    for suffix, selector in reversed(list(nodes.items())):
        pointers = Counter()
        recovered = []
        for b, n in selector:
            u = bytes([b]) + suffix
            i = pointers[b]
            recovered.extend(incoming[u][i : i + n])
            pointers[b] += n
        assert all(i == len(incoming[bytes([b]) + suffix]) for b, i in pointers.items())
        incoming[suffix] = recovered
    counts = Counter()
    forward = {}
    for y, key in zip(sorted(selected), incoming[b""], strict=True):
        assert key[1] == colour[y]
        x = buckets[key][0] + counts[key]
        counts[key] += 1
        assert x not in forward
        forward[x] = (key[0], y)
    assert all(forward[x] == first_return(x, selected, psi, first) for x in selected)
    return stats, forward, seen


def audit():
    stats = Counter()
    examples = {}
    bounds = [(2, 11), (3, 8)]
    for alphabet, limit in bounds:
        for n in range(1, limit + 1):
            for last in product(range(alphabet), repeat=n):
                if alphabet == 3 and 2 not in last:
                    continue
                first, psi, lf, _, _ = geometry(last)
                for length in range(1, 5):
                    for marker in sorted({prefix(x, psi, first, length) for x in range(n)}):
                        section = {x for x in range(n) if prefix(x, psi, first, length) == marker}
                        words = {x: first_return(x, section, psi, first)[0] for x in section}
                        types = sorted(set(words.values()))
                        for mask in range(1, 1 << len(types)):
                            chosen = {w for i, w in enumerate(types) if mask >> i & 1}
                            roots = ranges(sorted(x for x in section if words[x] in chosen))
                            if len(roots) < 2:
                                continue
                            g, forward, seen = discover(last, marker, chosen, roots, examples)
                            stats.update(g)
                            stats["selections"] += 1
                            for primary in range(n):
                                x = primary
                                skip = 0
                                while x not in forward:
                                    x = lf[x]
                                    skip += 1
                                    if x == primary:
                                        break
                                assert (x in forward) == (primary in seen)
                                if x not in forward:
                                    stats["disjoint_marks"] += 1
                                    continue
                                decoded = b""
                                while len(decoded) < n:
                                    spelling, x = forward[x]
                                    assert skip < len(spelling)
                                    decoded += spelling[skip:]
                                    skip = 0
                                assert decoded[:n] == prefix(primary, psi, first, n)
                                orbit = 1
                                y = psi[primary]
                                while y != primary:
                                    orbit += 1
                                    y = psi[y]
                                stats["nondividing_mark_period"] += int(n % orbit != 0)
                                stats["marked_outputs"] += 1
    print(
        json.dumps(
            dict(
                alphabet_and_max_length=bounds,
                parent_lengths=[1, 2, 3, 4],
                counts=stats,
                counterexamples=examples,
            ),
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    audit()
