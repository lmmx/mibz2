#!/usr/bin/env python3
"""Structural audit of representative choices on a finished Cells grammar."""
import argparse
import csv
import sys
from collections import Counter, OrderedDict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "2026-11-18-cells-live-reuse"))
import positions as study5
from census import irec


class Grammar:
    """Reachable, birth-ordered W16 rules for one full period and its marked rotation."""
    def __init__(self, arena, rules, root):
        self.arena, self.rules, self.root = arena, rules, root
        self.period = study5.size(root)
        self.origin = study5.origin(rules, root)
        self.incoming = [[] for _ in rules]
        self.length = []
        for j, pair in enumerate(rules):
            self.length.append(sum(map(study5.size, pair)))
            for side, word in enumerate(pair):
                child = study5.ref(word) - study5.BASE
                if child >= 0:
                    assert child < j
                    self.incoming[child].append(2 * j + side)
        if rules:
            assert study5.ref(root) - study5.BASE == len(rules) - 1
            assert all(self.incoming[:-1]) and not self.incoming[-1]

    def offers(self, j, q):
        a, b = self.rules[j]
        return ((a, q), (b, (q + study5.size(a)) % self.period))


def path_cost(g, owners, weighted):
    """Maximum cumulative unselected-edge weight from the root to each rule."""
    cost = [0] * len(g.rules)
    for k in reversed(range(len(cost) - 1)):
        weight = g.length[k] if weighted else 1
        cost[k] = max(cost[e // 2] + (0 if e == owners[k] else weight)
                      for e in g.incoming[k])
    return max(cost, default=0)


def optimal(g, weighted):
    """Tie-break towards last use among parents with maximum incoming distance."""
    owner, cost = [-1] * len(g.rules), [0] * len(g.rules)
    for k in reversed(range(len(cost) - 1)):
        edges = g.incoming[k]
        anchor = max(reversed(edges), key=lambda e: cost[e // 2])
        owner[k] = anchor
        weight = g.length[k] if weighted else 1
        cost[k] = max(cost[e // 2] + (0 if e == anchor else weight) for e in edges)
    return owner, max(cost, default=0)


def representatives(g, name):
    if name in ("rounds", "bytes"):
        return optimal(g, name == "bytes")[0]
    index = 0 if name == "first" else -1
    return [edges[index] if edges else -1 for edges in g.incoming]


class LineCache:
    """Fixed, fully associative 32 KiB LRU of 64-byte lines, cold and write-allocate."""
    def __init__(self):
        self.lines = OrderedDict()
        self.misses = 0
        self.evicted_dirty = 0

    def touch(self, start, length, write=False):
        for line in range(start // 64, (start + length - 1) // 64 + 1):
            if line in self.lines:
                if write:
                    self.lines[line] = True
                self.lines.move_to_end(line)
            else:
                self.misses += 1
                self.lines[line] = write
                if len(self.lines) > 512:
                    _, dirty = self.lines.popitem(last=False)
                    self.evicted_dirty += dirty

    def writebacks(self):
        return self.evicted_dirty + sum(self.lines.values())


def locality_limits(g):
    """Relax the coherence of parent positions, keeping each incoming edge distinct."""
    # Only store sets for nontrivial fanouts. Each edge may occur many times in the unfolding.
    pages, lines = {}, {}
    stack = [(len(g.rules) - 1, g.origin)] if g.rules else []
    visits = 0
    while stack:
        j, q = stack.pop()
        visits += 1
        for side, (w, d) in enumerate(g.offers(j, q)):
            k = study5.ref(w) - study5.BASE
            if k >= 0:
                stack.append((k, d))
                if len(g.incoming[k]) > 1:
                    e = 2 * j + side
                    pages.setdefault(e, set()).add(d // 4096)
                    lines.setdefault(e, set()).add(d // 64)
    bounds = []
    for cells in (lines, pages):
        limit = 0
        for edges in g.incoming:
            if len(edges) > 1:
                counts = Counter(b for e in edges for b in cells[e])
                limit += max(counts.values()) - 1
        bounds.append(limit)
    return dict(unfolded=visits, line_bound=bounds[0], page_bound=bounds[1],
                many=sum(len(edges) >= 3 for edges in g.incoming),
                many_offers=sum(len(edges) for edges in g.incoming if len(edges) >= 3),
                max_offers=max(map(len, g.incoming), default=0))


def wave_control(g, owner, short, commands, expected):
    """An offline order control. Commands in the same earliest round are independent."""
    height = [0] * len(g.rules)
    for j, pair in enumerate(g.rules):
        for side, w in enumerate(pair):
            k = study5.ref(w) - study5.BASE
            if k >= 0:
                height[j] = max(height[j], height[k] + (2 * j + side != owner[k]))
    ordered = sorted(commands, key=lambda item: (height[item[0]], item[2]))
    cache = LineCache()
    for w, d in short:
        cache.touch(d, study5.size(w))
    audit = study5.Audit(expected) if expected is not None else None
    if audit is not None:
        for w, d in short:
            audit.literal(g.arena, w, d)
    previous = None
    adjacent = 0
    for k, src, dst in ordered:
        if audit is not None:
            audit.copy(((study5.BASE + k) << 32) | g.length[k], dst, src)
        adjacent += previous == (src, dst)
        previous = src + g.length[k], dst + g.length[k]
        cache.touch(src, g.length[k])
        cache.touch(dst, g.length[k])
    if audit is not None:
        audit.finish()
    return max(height, default=0), cache.misses, adjacent


def independent_span(g, commands):
    """Tiny oracle. Intersect source spans with all destination holes directly."""
    rounds, weights = [], []
    for j, (k, src, _) in enumerate(commands):
        end = src + g.length[k]
        before = [i for i, (c, _, d) in enumerate(commands)
                  if d < end and src < d + g.length[c]]
        assert all(i < j for i in before)
        rounds.append(1 + max((rounds[i] for i in before), default=0))
        weights.append(g.length[k] + max((weights[i] for i in before), default=0))
    return max(rounds, default=0), max(weights, default=0)


def measure(g, name, expected=None, prescribed=None):
    owner = representatives(g, name) if prescribed is None else prescribed
    n = len(g.rules)
    pos = [-1] * n
    if n:
        pos[-1] = g.origin
    commands, short = [], []
    pending = [] if name != "median" else [[] for _ in range(n)]
    unknown = double = 0
    for j in reversed(range(n)):
        if name == "median" and j != n - 1:
            offers = pending[j]
            assert len(offers) == len(g.incoming[j])
            if len(offers) > 2:
                ordered = sorted(offers)
                # All samples between the two middle order statistics minimise L1.
                lo, hi = ordered[(len(offers) - 1) // 2][0], ordered[len(offers) // 2][0]
                pos[j], owner[j] = next((q, e) for q, e in offers if lo <= q <= hi)
            else:
                pos[j], owner[j] = offers[0]  # last use breaks the tie
        assert pos[j] >= 0
        for side, (w, d) in enumerate(g.offers(j, pos[j])):
            k = study5.ref(w) - study5.BASE
            e = 2 * j + side
            if k < 0:
                short.append((w, d))
            elif name == "median":
                pending[k].append((d, e))
            elif owner[k] == e:
                assert pos[k] == -1
                pos[k] = d
    # Instrument the chosen tree. This second enumeration only measures it.
    # The streaming last/first schedules themselves do not need it.
    literal_edges = len(short)
    if not n:
        short.append((g.root, 0))
        literal_edges = 1
    for j in range(n):
        count = 0
        for side, (w, d) in enumerate(g.offers(j, pos[j])):
            k = study5.ref(w) - study5.BASE
            e = 2 * j + side
            if k >= 0 and owner[k] != e:
                count += 1
                unknown += e > owner[k]  # deferred before its owner in reverse order
                commands.append((k, pos[k], d))
        double += count == 2
    moved = sum(bool(edges) and owner[k] != edges[-1] for k, edges in enumerate(g.incoming))
    amount = line_near = page_near = distance = spans = 0
    source_runs = 0
    previous = -1
    for k, s, d in commands:
        length = g.length[k]
        assert abs(s - d) >= length
        amount += length
        distance += abs(s - d)
        line_near += s // 64 == d // 64
        page_near += s // 4096 == d // 4096
        spans += (s % 64 + length + 63) // 64 + (d % 64 + length + 63) // 64
        source_runs += previous != k
        previous = k
    cache = LineCache()
    for w, d in short:
        cache.touch(d, study5.size(w))
    short_misses = cache.misses
    adjacent = 0
    for i, (k, src, dst) in enumerate(commands):
        cache.touch(src, g.length[k])
        cache.touch(dst, g.length[k])
        if i:
            c, s, d = commands[i - 1]
            adjacent += ((s + g.length[c], d + g.length[c]) == (src, dst)
                         or (src + g.length[k], dst + g.length[k]) == (s, d))
    ends = {(s + g.length[k], d + g.length[k]) for k, s, d in commands}
    aligned = sum((s, d) in ends for k, s, d in commands)
    copies = sum(len(edges) - 1 for edges in g.incoming if edges)
    distinct = sum(len(edges) > 1 for edges in g.incoming)
    assert len(commands) == copies
    assert amount == sum((len(edges) - 1) * length for edges, length in zip(g.incoming, g.length) if edges)
    assert amount + sum(study5.size(w) for w, _ in short) == g.period
    assert len(ends) == len(commands)
    # Every reverse-scan suffix has enough spent slots even with a double defer.
    deferred_in_suffix = 0
    for t, j in enumerate(reversed(range(n)), 1):
        for side, w in enumerate(g.rules[j]):
            k = study5.ref(w) - study5.BASE
            deferred_in_suffix += k >= 0 and owner[k] != 2 * j + side
        assert deferred_in_suffix <= t
    if name == "last":
        assert unknown == double == 0 and source_runs == distinct
    if name == "first":
        assert double == 0 and source_runs == distinct and unknown == copies
    debt = path_cost(g, owner, False)
    weighted_debt = path_cost(g, owner, True)
    waves, wave_misses, wave_adjacent = wave_control(g, owner, short, commands, expected)
    assert waves == debt
    if name == "given":
        assert (debt, weighted_debt) == independent_span(g, commands)
    if expected is not None:
        audit = study5.Audit(expected)
        for w, d in short:
            audit.literal(g.arena, w, d)
        for k, s, d in commands:
            audit.copy(((study5.BASE + k) << 32) | g.length[k], d, s)
        audit.finish()
    return dict(policy=name, rules=n, copies=copies, copy_bytes=amount,
                literals=literal_edges, sources=distinct, rounds=debt,
                span_bytes=weighted_debt, changed=moved, early=unknown,
                double=double, source_runs=source_runs, distance=distance, same_line=line_near,
                same_page=page_near, line_spans=spans, aligned=aligned,
                adjacent=adjacent, short_misses=short_misses, output_misses=cache.misses,
                wave_misses=wave_misses, wave_adjacent=wave_adjacent)


def recorded(rec, check):
    a = rec.a
    assert rec.n == rec.period
    ar, ru, root, _ = study5.grammar(a["l_byte"], rec.row_letter, a["s_a"], a["s_b"], a["s_type"], rec.final)
    g = Grammar(ar, ru, root)
    expected = None
    if check:
        word = rec.word(rec.final)
        q = (-g.origin) % len(word)
        expected = word[q:] + word[:q]
        study5.check_lf(rec, expected)
    return g, expected


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("records", nargs="+")
    p.add_argument("--check", action="store_true", help="validate coverage, copied bytes and the independent dense LF")
    opts = p.parse_args()
    writer = None
    for arg in opts.records:
        for path in irec.paths(arg):
            rec = irec.load(path)
            g, expected = recorded(rec, opts.check)
            bounds = locality_limits(g)
            for policy in ("last", "first", "rounds", "bytes", "median"):
                row = dict(block=rec.block, **measure(g, policy, expected), **bounds)
                assert row['same_line'] <= bounds['line_bound']
                assert row['same_page'] <= bounds['page_bound']
                if writer is None:
                    writer = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                    writer.writeheader()
                writer.writerow(row)


if __name__ == "__main__":
    main()
