"""Compare both actual constructors to indexed phases of primitive necklace multisets."""
from collections import Counter, defaultdict
from functools import cmp_to_key
from itertools import product
from pathlib import Path
import hashlib
import json
import random
import subprocess
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "f4"))
from phrases import compress, transform, spelling


def canonical(t):
    return min(t[i:] + t[:i] for i in range(len(t)))


def primitive(t):
    return all(t != t[i:] + t[:i] for i in range(1, len(t)))


def multisets(sigma, limit):
    pool = []
    for n in range(1, limit + 1):
        pool.extend(w for w in product(range(sigma), repeat=n) if w == canonical(w) and primitive(w))
    def visit(start, budget, chosen):
        if chosen:
            yield tuple(chosen)
        for idx in range(start, len(pool)):
            w = pool[idx]
            if len(w) > budget:
                break
            yield from visit(idx, budget-len(w), chosen + [w])
    return visit(0, limit, [])


def as_text(word):
    return "".join(chr(65+x) for x in word)


def cmp(a, b):
    return (a > b) - (a < b)


class Wheels:
    def __init__(self, wheels):
        self.wheels = wheels
        # For two periodic streams u^infinity and v^infinity the first difference
        # has the sign of uv vs vu. Ties occur at the same phase of identical
        # primitive wheels and retain increasing copy ID. No LF is used here.
        items = [(w[i:] + w[:i], j, i) for j,w in enumerate(wheels) for i in range(len(w))]
        def compare(a,b):
            u,v = a[0],b[0]
            return cmp(u+v,v+u) or cmp(a[1:],b[1:])
        items.sort(key=cmp_to_key(compare))
        self.rows = [(j,i) for _,j,i in items]
        self.n = len(items)
        self.last = tuple(wheels[j][i-1] for j,i in self.rows)
        self.primary_output = [tuple(wheels[j][(i+k)%len(wheels[j])] for k in range(self.n))
                               for j,i in self.rows]
        self.flags = set()
        if len(set(wheels)) < len(wheels):
            self.flags.add("equal_rotations")
        if any(self.n % len(w) for w in wheels):
            self.flags.add("nondividing_period")

    def section(self, boundary):
        chosen = [r for r in self.rows if boundary(*r)]
        rank = {r:k for k,r in enumerate(chosen)}
        words, targets, token_last = [], [], [None] * len(chosen)
        for k,(j,i) in enumerate(chosen):
            w = self.wheels[j]
            phrase = [w[i]]
            end = (i+1)%len(w)
            while (j,end) not in rank:
                phrase.append(w[end])
                end = (end+1)%len(w)
            words.append(tuple(phrase))
            targets.append(rank[j,end])
            token_last[targets[-1]] = words[-1]
        anchors = []
        for j,i in self.rows:
            w = self.wheels[j]
            for offset in range(len(w)):
                start = (j,(i-offset)%len(w))
                if start in rank:
                    anchors.append((rank[start], offset))
                    break
            else:
                anchors.append(None)
        return words, targets, token_last, anchors


def check_f2(wheels, mask, line, stats):
    wh = wheels.wheels
    selected = lambda x: (mask >> x) & 1
    words, targets, previous, marks = wheels.section(lambda j,i: selected(wh[j][i-1]))
    geom, arcs, actual = (field.strip().split() for field in line.split("|"))
    assert arcs == [f"{k}:{as_text(w)}" for k,w in zip(targets,words)]
    assert actual == ["-" if a is None else f"{a[1]}:{as_text(out)}"
                      for a,out in zip(marks, wheels.primary_output)]

    # Obtain each first-delimiter prefix directly from its wheel. Establish
    # disjoint FM intervals and count their exact L-run intersections.
    visiting = defaultdict(list)
    for r,(j,i) in enumerate(wheels.rows):
        w = wh[j]
        for extent in range(1,len(w)+1):
            if selected(w[(i+extent-1)%len(w)]):
                key = tuple(w[(i+k)%len(w)] for k in range(extent))
                visiting[key].append(r)
                break
    intersections = 0
    for key, indices in visiting.items():
        assert indices == list(range(indices[0], indices[-1]+1))
        intersections += 1 + sum(wheels.last[r] != wheels.last[r-1] for r in indices[1:])
    runs = 1 + sum(a != b for a,b in zip(wheels.last,wheels.last[1:]))
    assert intersections <= len(visiting)+runs-1
    assert list(map(int,geom)) == [len(visiting), intersections]
    assert sorted(range(len(previous)),key=previous.__getitem__) == targets
    stats["factorizations"] += 1
    stats["arcs"] += len(targets)
    stats["marks"] += wheels.n
    stats["mixed_covered_uncovered"] += int(any(a is None for a in marks) and any(a for a in marks))
    stats["delimiter_free_marks"] += sum(a is None for a in marks)
    stats["swallowed_marks"] += sum(a is not None and a[1] != 0 for a in marks)


def rotate(w, offset):
    return w[offset:] + w[:offset]


def token_period(t, p):
    # Re-derive token psi from repeated occurrence ranks, with no call to the
    # author's inverse or marked expansion. The wheel remains the authority.
    by_token = defaultdict(list)
    for i,x in enumerate(t):
        by_token[x].append(i)
    psi = [i for x in sorted(by_token) for i in by_token[x]]
    q = p
    period = []
    while True:
        q = psi[q]
        period.append(t[q])
        if q == p:
            return tuple(period)


def unpack(result):
    runs, p, d, parents, dictionary, _ = result
    return tuple(c for c,w in runs for _ in range(w)), p, d, [tuple(spelling(node, parents)) for node in dictionary]


def check_f4(w, asc, levels, stats):
    wh = w.wheels
    words, targets, previous, marks = w.section(lambda j,i: wh[j][i-1] >= wh[j][i] if asc
                                                  else wh[j][i-1] <= wh[j][i])
    assert all(a is not None for a in marks)
    def ordered(a,b):
        for x,y in zip(a,b):
            if x != y:
                return cmp(x,y)
        return cmp(len(a),len(b)) if asc else cmp(len(b),len(a))
    dic = sorted(set(words),key=cmp_to_key(ordered))
    codes = {word:i for i,word in enumerate(dic)}
    t = tuple(codes[word] for word in previous)
    assert sorted(range(len(t)),key=t.__getitem__) == targets
    stats["marks"] += w.n
    stats["arcs"] += len(targets)
    stats["swallowed_marks"] += sum(offset > 0 for _,offset in marks)
    stats["proper_prefix_word_pairs"] += sum(any(a[:i] in codes for i in range(1,len(a))) for a in codes)
    for p,expected in enumerate(w.primary_output):
        built, rp, skip, dictionary = unpack(transform(compress(w.last),p,asc))
        assert built == t and dictionary == dic and (rp,skip) == marks[p]
        assert skip <= len(set(w.last))-1
        first = token_period(built,rp)
        accum = tuple(c for name in first for c in dictionary[name])
        period = rotate(accum,skip)
        assert tuple(period[i%len(period)] for i in range(w.n)) == expected
        if levels:
            for direction in (False,True):
                top, high, offset, names = unpack(transform(compress(t),rp,direction))
                tokens = token_period(top,high)
                middle = rotate(tuple(c for x in tokens for c in names[x]),offset)
                period2 = rotate(tuple(c for x in middle for c in dic[x]),skip)
                assert period2 == period
                stats["two_level_mark_lifts"] += 1


def main():
    rng = random.Random(20777042)
    f2, f4, inputs = Counter(), Counter(), Counter()
    cases = []
    for sigma,n in [(2,9),(3,6),(4,4)]:
        cols = set()
        for collection in multisets(sigma,n):
            w = Wheels(collection)
            assert w.last not in cols
            cols.add(w.last)
            cases.append((sigma,w,len(w.last)<=5))
        # A cardinality test confirms every arbitrary L column over this alphabet
        # in the stated size range was exercised through the distinct wheel oracle.
        assert len(cols) == sum(sigma**k for k in range(1,n+1))
    for _ in range(35):
        sigma = rng.randrange(2,6)
        pool = []
        for j in range(rng.randrange(1,6)):
            while True:
                raw = tuple(rng.randrange(sigma) for _ in range(rng.randrange(1,17)))
                if primitive(raw):
                    break
            pool.extend([canonical(raw)]*rng.randrange(1,4))
        cases.append((sigma,Wheels(tuple(pool)),False))

    queries = []
    lines = []
    for sigma,w,levels in cases:
        inputs["columns"] += 1
        inputs.update(w.flags)
        for asc in (False,True):
            check_f4(w,asc,levels,f4)
        for mask in range(1 << sigma):
            queries.append((w,mask))
            lines.append(f"{as_text(w.last)} {mask}")
    data = "\n".join(lines)+"\n"
    proc = subprocess.run([sys.argv[1]],input=data,text=True,capture_output=True,check=True)
    got = proc.stdout.splitlines()
    assert len(got) == len(queries)
    for (w,mask),row in zip(queries,got):
        check_f2(w,mask,row,f2)
    print(json.dumps(dict(inputs=dict(inputs), f2=dict(f2), f4=dict(f4),
                          f2_query_sha256=hashlib.sha256(data.encode()).hexdigest(),
                          f2_reply_sha256=hashlib.sha256(proc.stdout.encode()).hexdigest()),
                     indent=2,sort_keys=True))


if __name__ == "__main__":
    main()
