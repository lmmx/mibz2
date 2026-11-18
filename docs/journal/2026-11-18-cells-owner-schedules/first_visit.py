#!/usr/bin/env python3
"""Trace the first-output-occurrence walk. Reads a grammar and audits spans, without emitting a block."""
import argparse
import csv
import sys
from collections import Counter

from schedules import LineCache, irec, measure, path_cost, recorded, study5


class Locality:
    """One shared line cache and two restricted views. All region starts are line aligned."""
    def __init__(self, g):
        self.rule = LineCache()
        self.output = LineCache()
        self.shared = LineCache()
        self.fills = Counter()
        self.fields = 0
        self.rule_base = (g.period + 63) // 64 * 64
        self.arena = self.rule_base + ((16 * len(g.rules) + 63) // 64) * 64
        self.scratch = self.arena + ((len(g.arena) + 63) // 64) * 64

    def combined(self, region, start, length, write=False):
        previous = self.shared.misses
        self.shared.touch(start, length, write)
        self.fills[region] += self.shared.misses - previous

    def record(self, j, offset, length, write=False):
        self.fields += 2 if length > 8 else 1
        self.rule.touch(16 * j + offset, length, write)
        self.combined("rule", self.rule_base + 16 * j + offset, length, write)

    def out(self, q, length, write=False):
        self.output.touch(q, length, write)
        self.combined("output", q, length, write)

    def literal(self, w, q):
        self.combined("arena", self.arena + study5.ref(w), study5.size(w))
        self.out(q, study5.size(w), write=True)

    def tmp(self, offset, length, write=False):
        self.fields += 1
        self.combined("scratch", self.scratch + offset, length, write)

    def counts(self):
        assert sum(self.fills.values()) == self.shared.misses
        return dict(fields=self.fields, rule_misses=self.rule.misses, output_misses=self.output.misses,
                    combined_misses=self.shared.misses, combined_rule=self.fills["rule"],
                    combined_output=self.fills["output"], combined_arena=self.fills["arena"],
                    combined_scratch=self.fills["scratch"], writebacks=self.shared.writebacks())


def last_locality(g):
    """Same trace convention, charging the Study 5 position array and in-place queue."""
    cache = Locality(g)
    nr = len(g.rules)
    pos = [-1] * nr
    queue = []
    wraps = 0
    if nr:
        pos[-1] = g.origin
        cache.tmp(4 * (nr - 1), 4, write=True)
    else:
        cache.literal(g.root, 0)
    later = (-1, -1)
    for j in reversed(range(nr)):
        cache.record(j, 0, 16)
        cache.tmp(4 * j, 4)
        wraps += pos[j] + study5.size(g.rules[j][0]) >= g.period
        for side, (w, d) in enumerate(g.offers(j, pos[j])):
            k = study5.ref(w) - study5.BASE
            if k < 0:
                cache.literal(w, d)
            elif w != later[side]:
                pos[k] = d
                cache.tmp(4 * k, 4, write=True)
            else:
                queue.append((k, d))
                cache.record(nr - len(queue), 0, 12, write=True)
        later = g.rules[j]
    later_in_word = 0
    for j, (k, d) in enumerate(reversed(queue), nr - len(queue)):
        later_in_word += (pos[k] - g.origin) % g.period > (d - g.origin) % g.period
        cache.record(j, 0, 12)
        cache.tmp(4 * k, 4)
        cache.out(pos[k], g.length[k])
        cache.out(d, g.length[k], write=True)
    assert cache.fields == 4 * nr + 5 * len(queue)
    return dict(last_wraps=wraps, last_later_in_word=later_in_word,
                **{"last_" + k: v for k, v in cache.counts().items()})


def walk(g, expected=None):
    # A real operand has positive low 32-bit length (the mark bit may also be set).
    # A visited record's first operand instead has low half zero and source in the high half.
    work = list(g.rules)
    memo_owner = [-1] * len(work)  # audit only
    pending = []
    pushes = high = copies = amount = literals = 0
    visited = later = runs = 0
    prior_source = -1
    cache = Locality(g)
    audit = study5.Audit(expected) if expected is not None else None
    q = g.origin
    emitted = 0

    def literal(w):
        nonlocal q, emitted, literals
        length = study5.size(w)
        assert q + length <= g.period
        if audit is not None:
            audit.literal(g.arena, w, q)
        cache.literal(w, q)
        emitted += length
        q = (q + length) % g.period
        literals += 1

    w, edge = g.root, -1
    while True:
        j = study5.ref(w) - study5.BASE
        if j < 0:
            literal(w)
        else:
            cache.record(j, 0, 8)
            a = work[j][0]
            if a & 0xffffffff:
                cache.record(j, 8, 8)
                b = work[j][1]
                cache.record(j, 0, 8, write=True)
                work[j] = (q << 32, b)
                visited += 1
                memo_owner[j] = edge
                if study5.ref(a) < study5.BASE:
                    literal(a)
                    w, edge = b, 2 * j + 1
                else:
                    cache.tmp(8 * len(pending), 8, write=True)
                    pending.append((b, 2 * j + 1))  # edge numbers are audit only
                    high = max(high, len(pending))
                    pushes += 1
                    w, edge = a, 2 * j
                continue
            source, length = a >> 32, study5.size(w)
            assert source + length <= g.period and q + length <= g.period
            assert (source - g.origin) % g.period + length <= emitted
            assert source + length <= q or q + length <= source
            if audit is not None:
                audit.copy(w, q, source)
            cache.out(source, length)
            cache.out(q, length, write=True)
            later += source > q
            emitted += length
            q = (q + length) % g.period
            copies += 1
            amount += length
            runs += prior_source != j
            prior_source = j
        if not pending:
            break
        w, edge = pending.pop()
        cache.tmp(8 * len(pending), 8)
    assert emitted == g.period and q == g.origin and visited == len(work)
    if audit is not None:
        audit.finish()
    assert cache.fields == 3 * visited + copies + 2 * pushes
    changed = sum(bool(edges) and memo_owner[k] != edges[-1] for k, edges in enumerate(g.incoming))
    assert copies == sum(len(edges) - 1 for edges in g.incoming if edges)
    row = dict(rules=visited, copies=copies, copy_bytes=amount, literals=literals,
               pushes=pushes, stack_peak=high, rounds=path_cost(g, memo_owner, False),
               span_bytes=path_cost(g, memo_owner, True), changed=changed,
               source_runs=runs, from_later=later, **cache.counts())
    return row, memo_owner


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("records", nargs="+")
    p.add_argument("--check", action="store_true")
    args = p.parse_args()
    writer = None
    for arg in args.records:
        for path in irec.paths(arg):
            rec = irec.load(path)
            g, expected = recorded(rec, args.check)
            visit, owner = walk(g, expected)
            # Hold the chosen occurrences fixed and put the original two phases back.
            birth = measure(g, "text", expected, owner)
            assert visit["copies"] == birth["copies"] and visit["copy_bytes"] == birth["copy_bytes"]
            row = dict(block=rec.block, **visit, **last_locality(g),
                       text_birth_output_misses=birth["output_misses"])
            if writer is None:
                writer = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)


if __name__ == "__main__":
    main()
