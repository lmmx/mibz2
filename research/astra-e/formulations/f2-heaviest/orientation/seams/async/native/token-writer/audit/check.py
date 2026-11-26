"""Independent rank-order, byte rotation, mark and second-induction audit."""
from collections import Counter, defaultdict
from itertools import permutations, product
import json


def step_inverse(psi):
    back = [0] * len(psi)
    for x, y in enumerate(psi):
        back[y] = x
    return back


def stream(psi, front, p, n):
    out = bytearray()
    for _ in range(n):
        out.append(front[p])
        p = psi[p]
    return bytes(out)


def returns(psi, front, section):
    """Actual endpoints and raw source-leading words, by independent row walking."""
    words, dest = {}, {}
    for x in section:
        w, y = bytearray(), x
        while True:
            w.append(front[y])
            y = psi[y]
            if y in section:
                break
        words[x], dest[x] = bytes(w), y
    return words, dest


def stable_buckets(section, words, dest):
    """Construct the token inverse using only destination labels and source bucket bases."""
    source = sorted(section)
    dest_column = {y: words[x] for x, y in dest.items()}
    rows = defaultdict(list)
    for i, x in enumerate(source):
        rows[words[x]].append(i)
    contiguous = all(xs[-1] - xs[0] + 1 == len(xs) for xs in rows.values())
    ordered = all(all(dest[source[a]] < dest[source[b]] for a, b in zip(xs, xs[1:]))
                  for xs in rows.values())
    if not contiguous:
        return None, contiguous, ordered
    next_index = {w: xs[0] for w, xs in rows.items()}
    induced = {}
    for y in source:
        w = dest_column[y]
        x = source[next_index[w]]
        next_index[w] += 1
        induced[x] = y
    assert (induced == dest) == ordered
    return induced if ordered else None, contiguous, ordered


def mark(back, section, p, strict):
    x, distance = p, 0
    if strict:
        x, distance = back[x], 1
    for _ in range(len(back)):
        if x in section:
            return x, distance - int(strict)
        x, distance = back[x], distance + 1
    return None


def expand(links, emissions, p, skip, length):
    out = bytearray()
    while len(out) < length:
        w = emissions[p]
        assert 0 <= skip < len(w)
        out.extend(w[skip:skip + length - len(out)])
        p, skip = links[p], 0
    return bytes(out)


def heavy_table(dest, words, emission):
    """Raw first-token identity chooses the heavy state; it can emit rotated bytes."""
    counts = Counter(words.values())
    first = {}
    for x in sorted(words):
        first.setdefault(words[x], x)
    chosen = max(counts, key=lambda w: (counts[w], -first[w]))
    heavy = {x for x, w in words.items() if w == chosen}
    prior = {y: x for x, y in dest.items()}
    hdest, hemit = {}, {}
    for x in heavy:
        y, out = x, bytearray()
        while True:
            y = dest[y]
            out.extend(emission[y])
            if y in heavy:
                break
        hdest[x], hemit[x] = y, bytes(out)
    return prior, heavy, hdest, hemit


def nested(dest, emission, p, skip, n, tables):
    prior, heavy, hdest, hemit = tables
    if len(heavy) == len(dest):
        return expand(dest, emission, p, skip, n), "homogeneous"
    x, begin = p, skip
    while True:
        x = prior[x]
        if x in heavy:
            return expand(hdest, hemit, x, begin, n), "second"
        begin += len(emission[x])
        if x == p:
            return expand(dest, emission, p, skip, n), "heavy_disjoint"


def encode(psi, front, section, words, dest):
    return {
        "psi": list(psi), "F": "".join(map(str, front)), "U": sorted(section),
        "raw": ["".join(map(str, words[x])) for x in sorted(section)],
        "dest": [dest[x] for x in sorted(section)],
    }


def stable_test(limit):
    tally, witnesses = Counter(), {}
    for n in range(1, limit + 1):
        for last in product(range(2), repeat=n):
            psi = sorted(range(n), key=last.__getitem__)
            front = [last[x] for x in psi]
            back = step_inverse(psi)
            gold = [stream(psi, front, p, n) for p in range(n)]
            period = []
            for p in range(n):
                y, d = psi[p], 1
                while y != p: y, d = psi[y], d + 1
                period.append(d)
            # Observe all present bounded cylinder patterns, including beyond one lap.
            prefixes = [stream(psi, front, p, 2*n + 1) for p in range(n)]
            patterns = defaultdict(set)
            for p, prefix in enumerate(prefixes):
                for length in range(1, n + 2):
                    patterns[prefix[:length]].add(p)
            by_section = defaultdict(list)
            for word, group in patterns.items():
                by_section[frozenset(group)].append(word)
            cases = [(frozenset(i for i in range(n) if mask >> i & 1), mask)
                     for mask in range(1, 1 << n)]
            for section, mask in cases:
                words, dest = returns(psi, front, section)
                inv, contiguous, ordered = stable_buckets(section, words, dest)
                assert ordered
                tally["stable_binary_sections"] += 1
                # Cylinder equality I(WP), evaluating arbitrarily long cyclic prefixes.
                for pattern in by_section.get(section, ()):
                    assert inv == dest
                    for w in set(words.values()):
                        wp = w + pattern
                        defined = {p for p, pref in enumerate(prefixes) if pref.startswith(wp)}
                        assert defined == {p for p in section if words[p] == w}
                        assert max(defined)-min(defined)+1 == len(defined)
                        tally["cylinder_word_classes"] += 1
                    tally["marker_cylinders"] += 1
                # A stable common-prefix Psi step never decreases rank difference.
                # Two intermediate outside rows cannot bracket a complete component
                # when their total separation is <= the smallest component width.
                components = []
                for row in sorted(section):
                    if not components or row != components[-1][-1]+1:
                        components.append([])
                    components[-1].append(row)
                color = {x:i for i,component in enumerate(components) for x in component}
                tagged = {x:(w,color[dest[x]]) for x,w in words.items()}
                grouped = defaultdict(list)
                for x in sorted(section): grouped[tagged[x]].append(x)
                ranks = {x:i for i,x in enumerate(sorted(section))}
                minwidth = min(map(len,components))
                for key,xs in grouped.items():
                    span = dest[xs[-1]] - dest[xs[0]]
                    full = ranks[xs[-1]] - ranks[xs[0]] + 1 == len(xs)
                    if span <= minwidth:
                        assert full
                        tally["diameter_certified_colored_classes"]+=1
                    elif not full:
                        tally["split_destination_colored_classes"]+=1
                        if "destination_color_insufficient" not in witnesses:
                            witnesses["destination_color_insufficient"] = dict(
                                last="".join(map(str,last)),colored_destination_component=key[1],
                                split_sources=xs,**encode(psi,front,section,words,dest))
                if len(components)==1:
                    assert contiguous and ordered
                    tally["single_rank_intervals"]+=1
                    order = list(dict.fromkeys(words[x] for x in sorted(section)))
                    if order != sorted(order):
                        kind = "cylinder_lex_order" if section in by_section else "noncylinder_lex_order"
                        if kind not in witnesses:
                            witnesses[kind] = dict(last="".join(map(str,last)),
                                base_order=["".join(map(str,w)) for w in order],**encode(psi,front,section,words,dest))
                    if section not in by_section:
                        at = defaultdict(list)
                        for x in sorted(section):
                            y = back[x]
                            at[front[y]].append((x,y,y in section))
                        for c, transitions in at.items():
                            if len({t[2] for t in transitions}) > 1:
                                tally["noncylinder_mixed_one_byte_extensions"]+=1
                                if "mixed_backward_termination" not in witnesses:
                                    witnesses["mixed_backward_termination"] = dict(last="".join(map(str,last)),
                                        preceding_byte=c,transitions=transitions,**encode(psi,front,section,words,dest))
                if max(map(len,components)) <= minwidth+1:
                    assert stable_buckets(section,tagged,dest)[0]==dest
                    tally["near_equal_width_certified_unions"]+=1
                if not contiguous:
                    tally["split_bucket_sections"] += 1
                    props = ["arbitrary_split"]
                    if max(section)-min(section)+1 == len(section):
                        props.append("interval_split")
                    if len({front[i] for i in section}) == 1:
                        props.append("uniform_leading_byte_split")
                    for prop in props:
                        if prop not in witnesses:
                            witnesses[prop] = dict(last="".join(map(str,last)), **encode(psi,front,section,words,dest))
                    continue
                assert inv == dest
                uniform = len({front[p] for p in section}) == 1
                rotated = {p: w[1:]+w[:1] for p,w in words.items()}
                ht = heavy_table(inv,words,words)
                rt = heavy_table(inv,words,rotated)
                for p in range(n):
                    a, b = mark(back,section,p,False), mark(back,section,p,True)
                    assert (a is None) == (b is None)
                    if a is None:
                        tally["marked_orbit_misses_section"] += 1
                        assert stream(psi,front,p,n) == gold[p]
                        continue
                    out, route = nested(inv,words,*a,n,ht)
                    assert out == expand(inv,words,*a,n) == gold[p]
                    tally["raw_"+route] += 1
                    if n % period[p]:tally["raw_nondividing_period_marks"]+=1
                    if route=="heavy_disjoint" and route not in witnesses:
                        witnesses[route] = dict(last="".join(map(str,last)), mark=p,**encode(psi,front,section,words,dest))
                    rotated_out, _ = nested(inv,rotated,*b,n,rt)
                    if uniform:
                        assert rotated_out == expand(inv,rotated,*b,n) == gold[p]
                        tally["uniform_rotated_exact_marks"] += 1
                    elif rotated_out != gold[p] and "mixed_rotation" not in witnesses:
                        witnesses["mixed_rotation"] = dict(last="".join(map(str,last)),mark=p,
                            obtained="".join(map(str,rotated_out)),expected="".join(map(str,gold[p])),
                            **encode(psi,front,section,words,dest))
                    if uniform and "inclusive_with_rotated" not in witnesses:
                        wrong = expand(inv,rotated,*a,n)
                        if wrong != gold[p]:
                            witnesses["inclusive_with_rotated"] = dict(last="".join(map(str,last)),mark=p,
                                obtained="".join(map(str,wrong)), expected="".join(map(str,gold[p])),
                                **encode(psi,front,section,words,dest))
    return tally, witnesses


def arbitrary_test(limit):
    counts, witness=Counter(),{}
    for n in range(1,limit+1):
        for psi in permutations(range(n)):
            for front in product(range(2),repeat=n):
                for mask in range(1,1<<n):
                    section=frozenset(i for i in range(n) if mask >> i & 1)
                    words,dest=returns(psi,front,section)
                    inv, contiguous, ordered=stable_buckets(section,words,dest)
                    # Minimum exact refinement when labels may split but not mix spellings:
                    # force a fresh id at a change of raw word or nonincreasing destination.
                    ids,ordinal={},-1
                    prev=None
                    for x in sorted(section):
                        if prev is None or words[x]!=words[prev] or dest[x]<=dest[prev]:
                            ordinal+=1
                        ids[x]=ordinal
                        prev=x
                    assert stable_buckets(section,ids,dest)[0]==dest
                    counts["arbitrary_minimum_refinement_added_types"]+=ordinal+1-len(set(words.values()))
                    counts["arbitrary_labeled_sections"]+=1
                    counts["arbitrary_certified"]+=inv is not None
                    counts["arbitrary_contiguous_unstable"]+=contiguous and not ordered
                    if (contiguous and not ordered and len(set(words.values()))==1 and
                            "single_type_unstable" not in witness):
                        witness["single_type_unstable"]=encode(psi,front,section,words,dest)
    return counts,witness


if __name__=="__main__":
    c,w=stable_test(7);c2,w2=arbitrary_test(5)
    report={"counts":dict(c+c2),"first_witnesses":w|w2,
            "limits":{"stable_binary_last_columns":7,"arbitrary_labeled_permutations":5}}
    print(json.dumps(report,indent=2,sort_keys=True))
