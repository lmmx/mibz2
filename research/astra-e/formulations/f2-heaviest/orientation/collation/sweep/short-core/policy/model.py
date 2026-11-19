"""Short-safe original-return prefix policies, exact per-node charges, and paid application counts."""
from array import array
from bisect import bisect_right
from collections import Counter, defaultdict
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from induce import Atlas, Column, Oracle, heavy_set, suffixes, truth

sys.path.insert(0, str(HERE.parents[8] / "docs/journal/2026-11-18-astra-c-output-first"))
from inputs import read_input


def get_input(path):
    runs, mark = read_input(path)
    col = Column(runs)
    oracle = Oracle(b"".join(bytes([c]) * width for c, width in runs))
    quote = col.interval(b'"')
    sections, pops, pieces = heavy_set(col, quote, 1)
    (sec,) = sections
    words, _ = truth(oracle, sec)
    return col, oracle, sec, words, dict(mark=mark, selector_pops=pops, selector_pieces=pieces)


class Trie:
    """Sparse sorted byte edges. Edge i leads to state i+1 in breadth-first order."""
    def __init__(self, enabled):
        edges = defaultdict(list)
        for key in enabled:
            if key:
                assert key[:-1] in enabled
                edges[key[:-1]].append(key[-1])
        self.starts = array("I", [0])
        self.labels = bytearray()
        self.root = 0 if b"" in enabled else -1
        queue = [b""] if self.root == 0 else []
        for key in queue:
            chars = sorted(edges[key])
            self.labels.extend(chars)
            self.starts.append(len(self.labels))
            queue.extend(key + bytes([c]) for c in chars)
        assert self.starts.itemsize == 4 and len(queue) == len(enabled)
        self.lookup = self.compare = self.forward_segments = 0

    def child(self, state, symbol):
        self.forward_segments += 1
        lo, end = self.starts[state:state+2]
        if lo == end:
            return -1
        self.lookup += 1
        hi = end
        while lo < hi:
            mid = (lo + hi) // 2
            self.compare += 1
            if self.labels[mid] < symbol:
                lo = mid + 1
            else:
                hi = mid
        if lo == end:
            return -1
        self.compare += 1
        return lo + 1 if self.labels[lo] == symbol else -1


def geometry(col, sec, maximum, trie=None):
    """Emit the actual leaves with source ownership. The string field is solely for oracle receipts."""
    a, b = sec
    strips = sorted((left+t, right+t, -t, sym) for left, right, t, sym in
                    zip(col.starts, col.starts[1:], col.maps, col.symbols))
    ends = [right for _, right, _, _ in strips]
    costs = defaultdict(Counter)
    active = [(a, a, b-a, b"", 0 if trie is None else trie.root)]
    leaves = []
    for _ in range(maximum):
        next_active = []
        for base, row, width, prefix, state in active:
            if state == -1:
                leaves.append((base, row, width, prefix))
                continue
            cost = costs[prefix]
            cost["in"] += 1
            x = row
            ix = bisect_right(ends, x)
            while x < row + width:
                _, edge, shift, c = strips[ix]
                stop = min(row+width, edge)
                first, last = x+shift, stop+shift
                boundaries = [first] + [v for v in sec if first < v < last] + [last]
                for low, high in zip(boundaries, boundaries[1:]):
                    off = low-shift-row
                    if a <= low < b:
                        leaves.append((base+off, row+off, high-low, prefix))
                        cost["short"] += 1
                    else:
                        child = trie.child(state, c) if trie is not None else 0
                        next_active.append((base+off, low, high-low, prefix+bytes([c]), child))
                        cost["out"] += 1
                cost["F"] += 1
                x = stop
                ix += 1
        active = next_active
        if not active:
            break
    leaves.extend((base, row, width, prefix) for base, row, width, prefix, _ in active)
    for cost in costs.values():
        cost["dB"] = cost["short"] + cost["out"] - cost["in"]
        assert cost["dB"] >= 0
    assert len(leaves) == 1 + sum(v["dB"] for v in costs.values())
    return costs, leaves


def depths(words, enabled):
    cuts = {}
    for word in words:
        depth = 0
        while depth < len(word)-1 and word[:depth] in enabled:
            depth += 1
        cuts[word] = depth
    return cuts


def audit_leaves(leaves, oracle, sec, words, cuts):
    """Every source row and every positive return, including tied/multiple marked cycles."""
    sources = {}
    physical = set()
    for base, row, width, prefix in leaves:
        for j in range(width):
            assert base+j not in sources and row+j not in physical
            sources[base+j] = row+j, prefix
            physical.add(row+j)
    assert len(sources) == sec[1]-sec[0]
    for word, arcs in words.items():
        d = cuts[word]
        for source, dest in arcs:
            row = source
            for i, c in enumerate(word):
                assert oracle.first[row] == c
                if i == d:
                    assert sources[source] == (row, word[:d])
                row = oracle.forward[row]
            assert row == dest


def application(col, oracle, sec, words, maximum, literal=None):
    t = None if literal is None else Trie(literal)
    costs, leaves = geometry(col, sec, maximum, t)
    cuts = ({w: min(maximum, len(w)-1) for w in words} if t is None else depths(words, literal))
    audit_leaves(leaves, oracle, sec, words, cuts)
    if t is None:
        reference = Atlas(col, sec, maximum)
        assert sorted(leaves) == sorted(reference.leaves)
    cores = {w[d:] for w, d in cuts.items()}
    stats = dict(Q=suffixes(cores), F=sum(v["F"] for v in costs.values()), B=len(leaves),
                 cores=len(cores), core_bytes=sum(map(len, cores)),
                 reached_states=len(costs), in_packets=sum(v["in"] for v in costs.values()))
    if t is not None:
        stats.update(child_segments=t.forward_segments, lookups=t.lookup,
                     symbol_tests=t.compare, trie_bytes=4*len(t.starts)+len(t.labels))
    return costs, stats


def obligations(words, cap):
    """Group removable nonempty proper suffixes by their deepest distinct mandatory prefixes."""
    # Store suffix strings only at potentially removable depths. A second pass rejects
    # these strings anywhere a longer removed prefix would be required.
    needed = defaultdict(set)
    for word in words:
        for d in range(1, min(cap, len(word)-1)+1):
            needed[word[d:]].add(word[:d-1])
    for word in words:
        for d in range(cap+1, len(word)):
            needed.pop(word[d:], None)
    grouped = Counter()
    for reqs in needed.values():
        implied = set()
        for key in reqs:
            implied.update(key[:i] for i in range(len(key)))
        grouped[tuple(sorted(reqs - implied))] += 1
    return grouped


if __name__ == "__main__":
    import argparse
    import csv
    import hashlib
    import json
    import lzma
    p = argparse.ArgumentParser()
    p.add_argument("tape", type=Path)
    p.add_argument("directory", type=Path)
    args = p.parse_args()
    assert args.tape.name == "0000.irec.xz"
    col, oracle, sec, words, info = get_input(args.tape)
    groups = obligations(words, 16)
    costs16, full = application(col, oracle, sec, words, 16)
    _, ref = application(col, oracle, sec, words, 8)
    q0 = suffixes(words)
    cap8 = {key for key in costs16 if len(key)<8}
    def price(keys):
        return (q0-sum(w for req, w in groups.items() if set(req)<=keys),
                sum(costs16[key]["F"] for key in keys),1+sum(costs16[key]["dB"] for key in keys))
    assert price(cap8) == tuple(ref[k] for k in ("Q","F","B"))
    assert price(set(costs16)) == tuple(full[k] for k in ("Q","F","B"))
    args.directory.mkdir(parents=True, exist_ok=True)
    with lzma.open(args.directory/"frozen-costs.csv.xz","wt") as f:
        w = csv.writer(f,lineterminator="\n")
        w.writerow(["prefix_hex","intersections","delta_bands","input_packets","continued_packets","short_packets"])
        for key, v in sorted(costs16.items()):
            w.writerow([key.hex(),v["F"],v["dB"],v["in"],v["out"],v["short"]])
    with lzma.open(args.directory/"frozen-goals.jsonl.xz","wt") as f:
        for req, weight in sorted(groups.items()):
            print(json.dumps([weight,[k.hex() for k in req]],separators=(",",":")),file=f)
    summary = dict(ordinal=0, tape_sha256=hashlib.sha256(args.tape.read_bytes()).hexdigest(),sec=sec,
                   words=len(words),original_Q=q0,cap8=ref,cap16=full,goals=len(groups),variables=len(costs16),**info)
    (args.directory/"frozen.json").write_text(json.dumps(summary,indent=2)+"\n")
    print(json.dumps(summary))
