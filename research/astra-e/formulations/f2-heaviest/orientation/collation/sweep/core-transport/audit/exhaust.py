"""Independent rotation-rank truth on every binary last column through length eight."""
from collections import Counter, defaultdict
from itertools import groupby, product
import json

from intervals import Mask, RunIndex, consecutive, heaviest


def spans(x):
    return 1 + sum(abs(a-b) != 1 for a,b in zip(x,x[1:])) if x else 0


def plus_hit(forward, first, section):
    words = [None] * len(forward)
    dest = [None] * len(forward)
    for source in range(len(forward)):
        path = bytearray()
        x = source
        for _ in forward:
            path.append(first[x])
            x = forward[x]
            if x in section:
                words[source] = bytes(path)
                dest[source] = x
                break
    return words, dest


def transport(u, forward, first, original, ends, depths, marked_truth):
    visits = []
    for source, depth in zip(u, depths):
        word = original[source]
        z = source
        for _ in range(depth):
            z = forward[z]
        p, c = word[:depth], word[depth:]
        visits.append((z, source, ends[source], p, c))
    visits.sort()
    assert len({z for z, *_ in visits}) == len(u)
    bands = 0
    partitions = []
    prev = None
    for z, s, d, p, c in visits:
        if prev != (z-1, s-1, p):
            bands += 1
        partitions.append(bands-1)
        prev = z, s, p
    by_core = defaultdict(list)
    for ind, (_, s, _, _, c) in enumerate(visits):
        by_core[c].append((s, partitions[ind]))
    # Bounded real rank checks independently verify occurrence pairing.
    for c in by_core:
        physical = [index for index,v in enumerate(visits) if v[-1] == c]
        assert spans(physical) == 1
    lifted = {}
    idx = Counter()
    tags = []
    fullwords = []
    cores = []
    for _, s, dest, p, c in sorted(visits, key=lambda v:v[2]):
        old_source, band = by_core[c][idx[c]]
        assert old_source == s and ends[old_source] == dest
        idx[c] += 1
        lifted[old_source] = dest, p+c
        tags.append((band,c))
        fullwords.append(p+c)
        cores.append(c)
    groups = lambda seq: sum(1 for _ in groupby(seq))
    r, k, i = groups(fullwords), groups(cores), groups(tags)
    assert r <= i <= k+bands-1
    assert bands <= i
    for marked, expected in enumerate(marked_truth):
        out = bytearray()
        x = marked
        while x not in lifted and len(out) < len(forward):
            out.append(first[x])
            x = forward[x]
        while len(out) < len(forward):
            x, word = lifted[x]
            out.extend(word[:len(forward)-len(out)])
        assert out == expected
    return bands, r, k, i


def main():
    counts = Counter()
    witnesses = {}
    for n in range(1, 9):
        for raw in product(range(2), repeat=n):
            last = bytes(raw)
            forward = sorted(range(n), key=last.__getitem__)
            first = bytes(last[x] for x in forward)
            marked_truth = []
            periods = []
            for mark in range(n):
                x = mark
                stream = bytearray()
                period = None
                for step in range(1, n+1):
                    stream.append(first[x])
                    x = forward[x]
                    if x == mark and period is None:
                        period = step
                marked_truth.append(bytes(stream))
                periods.append(period)
            index = RunIndex([(k, sum(1 for _ in g)) for k,g in groupby(raw)])
            # Every mark appears among these starting ranks; an orbit can miss U forever.
            for a in range(n):
                for b in range(a+1, n+1):
                    u = list(range(a,b))
                    word, ends = plus_hit(forward, first, set(u))
                    counts['intervals'] += 1
                    counts['nondividing_mark_section_pairs'] += sum(n % p != 0 for p in periods)
                    counts['tied_itinerary_mark_section_pairs'] += sum(marked_truth.count(s) > 1 for s in marked_truth)
                    buckets = defaultdict(list)
                    for x,w in enumerate(word):
                        counts['rows'] += 1
                        if w is None:
                            counts['disjoint_rows'] += 1
                        else:
                            buckets[w].append(x)
                    probes = set(buckets) | {bytes(k) for ln in range(1,4) for k in product(range(2),repeat=ln)}
                    for w in sorted(probes):
                        lo, hi = index.interval(w,(a,b))
                        assert list(range(lo,hi)) == buckets.get(w,[])
                        counts['whole_interval_probes'] += 1
                        if hi > lo:
                            assert all(ends[i] < ends[i+1] for i in range(lo,hi-1))
                            counts['present_words'] += 1
                        else:
                            counts['absent_probes'] += 1
                    if n <= 5:
                        for mask_bits in range(1 << n):
                            v = [x for x in range(n) if (mask_bits >> x) & 1]
                            count = Counter(word[x] for x in v if word[x] is not None)
                            mass, ties, _ = heaviest(index,(a,b),Mask(consecutive(v)))
                            assert mass == max(count.values(),default=0)
                            assert [w for w, _, _ in ties] == sorted(w for w in count if count[w]==mass)
                            counts['all_mask_heaviest_checks_through_n5'] += 1
                    for ts in product(*(range(len(word[i])) for i in u)):
                        unique = {}
                        typewise = True
                        for i,t in zip(u,ts):
                            w = word[i]
                            if w in unique and unique[w] != t:
                                typewise = False
                            unique[w] = t
                        band, r, rc, intersections = transport(u,forward,first,word,ends,ts,marked_truth)
                        counts['marked_outputs'] += n
                        counts['all_occurrence_assignments'] += 1
                        if typewise:
                            counts['typewise_assignments'] += 1
                            assert intersections == r and band <= r
                        elif intersections != r:
                            counts['varying_split_assignments'] += 1
                            if band > r:
                                counts['varying_band_gt_full_runs'] += 1
                                name = 'minimum_occurrence_varying_overpartition'
                                if name not in witnesses:
                                    witnesses[name] = dict(n=n,L=raw,U=[a,b],psi=forward,
                                        cuts=ts,words=[word[x].hex() for x in u],dest=[ends[x] for x in u],
                                        atlas_bands=band,full_runs=r,core_runs=rc,intersections=intersections)
            # Separately falsify convexity and look for a stricter one-cycle witness.
            one_cycle = True
            x = forward[0]
            for _ in range(n-1):
                if x == 0:
                    one_cycle = False
                    break
                x = forward[x]
            for bits in range(1, 1<<n):
                u = [i for i in range(n) if bits >> i & 1]
                if spans(u) == 1:
                    continue
                words, _ = plus_hit(forward,first,set(u))
                for c in (v for v in set(words) if v):
                    z = [i for i,v in enumerate(words) if v == c]
                    if spans(z) == 1:
                        continue
                    for name, use in [('minimum_nonconvex',True),('minimum_nonconvex_one_cycle',one_cycle)]:
                        if use and name not in witnesses:
                            witnesses[name] = dict(n=n,L=raw,U=u,psi=forward,core=c.hex(),members=z,
                                all_rows=[v.hex() if v else None for v in words])
    print(json.dumps(dict(counts=counts,witnesses=witnesses),indent=2))


if __name__ == '__main__':
    main()
