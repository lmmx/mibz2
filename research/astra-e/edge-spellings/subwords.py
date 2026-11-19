"""Exact substring and adjacent-overlap census for marked width-16 dictionary words."""
import argparse
from collections import deque
import csv
from itertools import combinations, permutations, product
from pathlib import Path
import struct
import sys


def overlap(left, right):
    """Length of the longest suffix of left that is also a prefix of right."""
    n = min(len(left), len(right))
    if not n or left[-1] not in right[:n]:
        return 0
    pattern = right[:n]
    prefix = [0] * n
    k = 0
    for i in range(1, n):
        while k and pattern[k] != pattern[i]:
            k = prefix[k - 1]
        if pattern[k] == pattern[i]:
            k += 1
        prefix[i] = k
    k = 0
    for c in left[-n:]:
        while k and pattern[k] != c:
            k = prefix[k - 1]
        if c == pattern[k]:
            k += 1
    return k


class PrefixIndex:
    def __init__(self, words):
        self.children = [{}]
        self.failure = [0]
        self.owner = [-1]
        self.max_length = [0]
        self.end = []
        # An owner of -2 means that more than one word starts with this prefix.
        for i, word in enumerate(words):
            node = 0
            for c in word:
                child = self.children[node].get(c)
                if child is None:
                    child = len(self.children)
                    self.children[node][c] = child
                    self.children.append({})
                    self.failure.append(0)
                    self.owner.append(i)
                    self.max_length.append(len(word))
                else:
                    self.owner[child] = -2
                    self.max_length[child] = max(self.max_length[child], len(word))
                node = child
            self.end.append(node)

        queue = deque(self.children[0].values())
        self.order = []
        while queue:
            parent = queue.popleft()
            self.order.append(parent)
            for c, child in self.children[parent].items():
                node = self.failure[parent]
                while node and c not in self.children[node]:
                    node = self.failure[node]
                self.failure[child] = self.children[node].get(c, 0)
                queue.append(child)

    def maximal_indices(self, words):
        """Every substring ends at a prefix of a word; propagate its witness length."""
        witness = self.max_length[:]
        for child in reversed(self.order):
            parent = self.failure[child]
            witness[parent] = max(witness[parent], witness[child])
        return [i for i, end in enumerate(self.end) if witness[end] == len(words[i])]

    def outgoing_bound(self):
        """Maximum total overlap after relaxing distinct destinations and cycles."""
        depth = [0] * len(self.children)
        for node in [0, *self.order]:
            for child in self.children[node].values():
                depth[child] = depth[node] + 1
        largest = []
        for i, end in enumerate(self.end):
            node = self.failure[end]
            while node and self.owner[node] == i:
                node = self.failure[node]
            largest.append(depth[node])
        return sum(largest) - min(largest, default=0)


def analyze(words):
    if not words:
        return dict(unique=0, unique_bytes=0, maximal=0, maximal_bytes=0, subword_bytes=0,
                    adjacent_overlap=0, adjacent_reachable_arena=0, maximal_order_overlap=0,
                    maximal_order_arena=0, incoming_floor=0, outgoing_floor=0, qgram_floor=0, arena_floor=0)
    unique = list(dict.fromkeys(words))
    index = PrefixIndex(unique)
    maximal = [unique[i] for i in index.maximal_indices(unique)]
    del index
    unique_bytes = sum(map(len, unique))
    maximal_bytes = sum(map(len, maximal))
    overlap_original = sum(overlap(a, b) for a, b in zip(words, words[1:]))
    overlap_maximal = sum(overlap(a, b) for a, b in zip(maximal, maximal[1:]))
    outgoing_floor = maximal_bytes - PrefixIndex(maximal).outgoing_bound()
    incoming_floor = maximal_bytes - PrefixIndex([w[::-1] for w in maximal]).outgoing_bound()
    qgram_floor = 0
    for k in (1, 2, 3, 4, 8, 16):
        grams = {word[i:i+k] for word in maximal for i in range(len(word) + 1 - k)}
        if grams:
            qgram_floor = max(qgram_floor, len(grams) + k - 1)
    floor = max(max(map(len, maximal)), outgoing_floor, incoming_floor, qgram_floor)
    assert floor <= maximal_bytes - overlap_maximal
    return dict(unique=len(unique), unique_bytes=unique_bytes, maximal=len(maximal),
                maximal_bytes=maximal_bytes, subword_bytes=unique_bytes-maximal_bytes,
                adjacent_overlap=overlap_original,
                adjacent_reachable_arena=sum(map(len, words))-overlap_original,
                maximal_order_overlap=overlap_maximal,
                maximal_order_arena=maximal_bytes-overlap_maximal,
                incoming_floor=incoming_floor, outgoing_floor=outgoing_floor,
                qgram_floor=qgram_floor, arena_floor=floor)


def blocks(path):
    header, entry = struct.Struct('<6I'), struct.Struct('<3I')
    with path.open('rb') as f:
        while raw := f.read(header.size):
            b, stream, n, m, edge, count = header.unpack(raw)
            words, visits = [], []
            for _ in range(count):
                _, v, length = entry.unpack(f.read(entry.size))
                word = f.read(length)
                assert len(word) == length and length >= 2
                words.append(word)
                visits.append(v)
            yield dict(block=b, stream=stream, n=n, m=m, edge_bytes=edge, bands=count,
                       raw_bytes=sum(map(len, words)), expansions=sum(visits),
                       bytes_spelled=sum(v*len(w) for v,w in zip(visits,words))), words


def check():
    alphabet = b'ab'
    universe = [bytes(w) for n in range(1,4) for w in product(alphabet, repeat=n)]
    for a,b in product(universe,repeat=2):
        brute = max(k for k in range(min(len(a),len(b))+1) if a.endswith(b[:k]))
        assert overlap(a,b) == brute
    trials = 0
    for n in range(1,5):
        for words in combinations(universe,n):
            values = analyze(words)
            maximal = [w for w in words if not any(w != u and w in u for u in words)]
            assert values['maximal_bytes'] == sum(map(len,maximal))
            exact = min(sum(map(len,maximal))-sum(overlap(a,b) for a,b in zip(p,p[1:]))
                        for p in permutations(maximal))
            assert values['arena_floor'] <= exact <= values['maximal_order_arena']
            trials += 1
    assert analyze([b'aaaa',b'aaa',b'aa',b'aa'])['maximal_bytes']==4
    print(f"{trials} binary word sets and {len(universe)**2} pair overlaps checked",file=sys.stderr)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('path', type=Path, nargs='?')
    ap.add_argument('--check', action='store_true')
    args = ap.parse_args()
    if args.check:
        check()
    if args.path:
        writer = None
        for base, words in blocks(args.path):
            record = {**base, **analyze(words)}
            if writer is None:
                writer = csv.DictWriter(sys.stdout,record,lineterminator='\n')
                writer.writeheader()
            writer.writerow(record)


if __name__ == '__main__':
    main()
