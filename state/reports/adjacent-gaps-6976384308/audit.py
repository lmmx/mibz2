"""Dense reference for adjacent-rank contraction and archived measurements."""
from collections import Counter, defaultdict
from csv import DictReader
from hashlib import sha256
from itertools import product
import json
from pathlib import Path
from statistics import median

ROOT = Path(__file__).resolve().parent


def from_column(data, primary):
    counts = Counter(data)
    base = {}
    at = 0
    for a in sorted(counts):
        base[a] = at
        at += counts[a]
    pos = base.copy()
    lf = []
    for a in data:
        lf.append(pos[a])
        pos[a] += 1
    labels = data[primary:] + data[:primary]
    f = [(d-primary) % len(data) for d in lf[primary:]+lf[:primary]]
    original_zero = (-primary) % len(data)
    return f, labels, original_zero


def dense(f, labels, original_zero=0):
    """Unit-row oracle. Follow original edges to compute all first-return labels."""
    n = len(f)
    assert len(labels) == n and sorted(f) == list(range(n))
    starts = bytearray([1] + [int(labels[i]!=labels[i-1] or f[i]!=f[i-1]+1)
                              for i in range(1,n)])
    starts[original_zero] = 1
    R = sum(starts)
    rows = list(range(n))
    step = f
    k = R
    levels = []
    while step[0]:
        inv = [-1]*n
        for x in rows:
            inv[step[x]] = x
        kept = [x for x in rows if x < step[x] and x < inv[x]]
        assert kept[0] == 0 and len(kept)*2 <= len(rows)
        compact = [-1]*n
        for i, x in enumerate(kept):
            compact[x] = i
        result = [-1]*n
        count = 0
        prior = None
        for x in kept:
            y, height, seam = f[x], 1, starts[x]
            while compact[y] < 0:
                seam |= starts[y]
                y, height = f[y], height+1
            result[x] = y
            if prior is None or seam or prior != (x-1, compact[y]-1, height):
                count += 1
            else:
                # Translate the complete elementary trace by one and verify the certificate.
                low, high = x-1, x
                for _ in range(height):
                    assert low+1 == high and labels[low] == labels[high]
                    low, high = f[low], f[high]
                assert low+1 == high
            prior = x, compact[y], height
        assert count <= 3*R-2
        levels.append([len(rows), k, len(kept), count])
        rows, step, k = kept, result, count
    # The final label is the elementary excursion from zero to itself.
    reversed_period = [labels[0]]
    x = f[0]
    while x:
        reversed_period.append(labels[x])
        x = f[x]
    period = list(reversed(reversed_period))
    return levels, (period*((n+len(period)-1)//len(period)))[:n]


def direct(data, primary):
    seen, lf = defaultdict(int), []
    first = sorted(data)
    for a in data:
        lf.append(first.index(a)+seen[a])
        seen[a] += 1
    raw = []
    x = primary
    while True:
        raw.append(data[x])
        x = lf[x]
        if x == primary:
            break
    return (raw[::-1]*((len(data)+len(raw)-1)//len(raw)))[:len(data)]


def small():
    tested = 0
    for alphabet, maximum in ((2,8),(3,6)):
        for n in range(1,maximum+1):
            for data in product(range(alphabet),repeat=n):
                for p in range(n):
                    f, word, wrap = from_column(data,p)
                    assert dense(f,word,wrap)[1] == direct(data,p)
                    tested+=1
    return tested


def read(name):
    with (ROOT/"evidence"/name).open() as fh:
        return list(DictReader(fh))


def archived():
    med = defaultdict(list)
    for x in read("timing.csv"):
        if x["round"] != "0":
            med[x["input"],x["workers"],x["arm"]].append(float(x["seconds"]))
    select = []
    for key in sorted(med):
        if key[0].startswith(("wikidata","frozen","power-218750","edits-256","edits-1024")):
            select.append([*key,len(med[key]),median(med[key])])
    frozen = read("frozen-rounds.csv")
    support = defaultdict(list)
    for x in read("support-comparison.csv"):
        if x["round"] != "0":
            support[x["arm"]].append(float(x["seconds"]))
    cache = {x["arm"]: int(x["Dr"])+int(x["Dw"]) for x in read("cachegrind.csv")
             if x["input"]=="frozen"}
    wiki = read("wikidata.csv")
    return dict(frozen_rounds=len(frozen), frozen_visits=sum(int(x["atoms"]) for x in frozen),
                frozen_ports=sum(int(x["ports"]) for x in frozen),
                timing=select, support={k:median(v) for k,v in support.items()},
                support_per_pair=[a/b for a,b in zip(support["support"],support["extrema"])],
                cache_refs=cache,wiki_blocks=len(wiki),wiki_visits=sum(int(x["sum_atoms"]) for x in wiki),
                wiki_admitted=sum(int(x["admitted"]) for x in wiki))


if __name__=="__main__":
    last = (ROOT/"column.bin").read_bytes()
    ref = (ROOT/"baseline-rle1.bin").read_bytes()
    levels, raw = dense(*from_column(last,264045))
    assert bytes(raw)==ref
    expected = [[int(x[k]) for k in ("rows","atoms","minima")] for x in read("frozen-rounds.csv")]
    assert [x[:3] for x in levels]==expected
    growth = dense([4,6,7,8,9,10,11,5,0,1,2,3], [0]+[1]*6+[2]+[3]*4)[0]
    assert [x[3] for x in growth]==[5,2]
    print(json.dumps(dict(frozen_hash=sha256(bytes(raw)).hexdigest(), frozen=levels,
                          growth=growth, small_cases=small(), archived=archived()),indent=2))
