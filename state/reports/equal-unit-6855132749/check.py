#!/usr/bin/env python3
"""Finite semantic falsifier, not a native kernel or a cost/timing benchmark.

Run with Python 3.11+: python3 check.py > results.json
The direct LF oracle never uses the induction's orders or grammar.
"""

import itertools
import json
from collections import Counter
from dataclasses import dataclass


@dataclass(frozen=True)
class Word:
    """Symbolic reference: ordered bytes/mark, with optional deferred children."""

    value: tuple
    left: int = -1
    right: int = -1
    deferred: bool = False


def lf_map(column):
    """Stable sorted positions give LF and its independently inverted map."""
    inverse = sorted(range(len(column)), key=lambda x: column[x])
    lf = [0] * len(column)
    for dest, src in enumerate(inverse):
        lf[src] = dest
    return lf, inverse


def run(column, origin, budget=None):
    """Check every induced state and return exact integer work counts."""
    n = len(column)
    lf, inverse = lf_map(column)
    words = []
    top, width, label, symbols = [], {}, {}, {}
    for row, byte in enumerate(column):
        marked = row == origin
        if top and byte == symbols[top[-1]] and not marked and row != origin + 1:
            width[top[-1]] += 1
        else:
            k = len(top)
            top.append(k)
            width[k] = 1
            symbols[k] = byte
            label[k] = len(words)
            words.append(Word(((byte, marked),)))
    bottom = sorted(top, key=lambda k: symbols[k])
    m = n
    counts = Counter()
    witnesses = {}
    uses = Counter()

    def validate():
        assert set(top) == set(bottom)
        assert sum(width[k] for k in top) == m
        dest, pos = {}, 0
        for k in bottom:
            dest[k] = pos
            pos += width[k]
        pos = 0
        for k in top:
            word = words[label[k]].value
            for x in range(pos, pos + width[k]):
                y, itinerary = x, []
                while True:
                    itinerary.append((column[y], y == origin))
                    y = lf[y]
                    assert len(itinerary) <= n
                    if y < m:
                        break
                assert y == dest[k] + x - pos
                assert tuple(reversed(itinerary)) == word
                counts["oracle_return_visits"] += len(itinerary)
            pos += width[k]

    validate()
    while True:
        a, b = top[-1], bottom[-1]
        wa, wb = words[label[a]], words[label[b]]
        if a == b:
            if any(mark for _, mark in wa.value):
                root = label[a]
                break
            m -= width[a]
            top.pop()
            bottom.pop()
            counts["dropped_orbits_groups"] += 1
            validate()
            continue
        if budget is not None and counts["steps"] == budget:
            counts["refused"] += 1
            return counts, witnesses
        ra, rb = width[a], width[b]
        eligible = ra == rb == 1
        w = Word(wa.value + wb.value, label[a], label[b], eligible)
        marked = any(mark for _, mark in w.value)
        assert sum(mark for _, mark in w.value) <= 1
        counts["steps"] += 1
        counts["eligibility_checks"] += 1
        counts["unit_width"] += min(ra, rb) == 1
        counts["eligible"] += eligible
        counts["retained_full"] += not eligible
        counts["original_rules"] += marked or len(w.value) > 16
        counts["original_arena_bytes"] += 0 if marked or len(w.value) > 16 else len(w.value)
        if eligible:
            counts["eligible_marked"] += marked
            counts["eligible_long"] += len(w.value) > 16
            counts["eligible_short_plain"] += not marked and len(w.value) <= 16
            counts["deferred_records"] += 1
            counts["deferred_child_refs"] += 2
            for kind, condition in [
                ("eligible", True),
                ("marked", marked),
                ("long", len(w.value) > 16),
                ("word_not_unit", len(wa.value) > 1 or len(wb.value) > 1),
            ]:
                if condition and kind not in witnesses:
                    witnesses[kind] = dict(
                        column=list(column),
                        origin=origin,
                        ra=ra,
                        rb=rb,
                        left=list(wa.value),
                        right=list(wb.value),
                    )
        elif min(ra, rb) == 1 and "unit_not_equal" not in witnesses:
            witnesses["unit_not_equal"] = dict(column=list(column), origin=origin, ra=ra, rb=rb)
        for child in (w.left, w.right):
            if words[child].deferred:
                uses[child] += 1
                # Width-one results have one live occurrence: no fan-out at induction.
                assert uses[child] == 1
                counts["deferred_consumed_equal" if eligible else "deferred_consumed_full"] += 1
                if not eligible and "deferred_consumed_full" not in witnesses:
                    witnesses["deferred_consumed_full"] = dict(
                        column=list(column),
                        origin=origin,
                        ra=ra,
                        rb=rb,
                        child_word_length=len(words[child].value),
                    )
        if eligible and words[w.left].deferred and words[w.right].deferred:
            counts["two_deferred_children"] += 1
            witnesses.setdefault(
                "two_deferred_children", dict(column=list(column), origin=origin, ra=ra, rb=rb)
            )
        new = len(words)
        words.append(w)
        if ra > rb:
            width[a] -= rb
            label[b] = new
            bottom.pop()
            bottom.insert(bottom.index(a) + 1, b)
        elif ra < rb:
            width[b] -= ra
            label[a] = new
            top.pop()
            top.insert(top.index(b) + 1, a)
        else:
            label[b] = new
            top.pop()
            bottom.pop()
            bottom[bottom.index(a)] = b
        m -= min(ra, rb)
        validate()

    # A topological materialisation preserves the unchanged writer's representation,
    # but must perform every original composition, including deferred ones.
    materialized = []
    for w in words:
        value = w.value if w.left < 0 else materialized[w.left] + materialized[w.right]
        assert value == w.value
        materialized.append(value)
        if w.left >= 0:
            counts["materialized_compositions"] += 1
            counts["materialized_copy16_bytes"] += 32
            counts["materialized_rule_write_bytes"] += 16
            counts["materialized_deferred"] += w.deferred
    assert counts["materialized_compositions"] == counts["steps"]
    assert counts["materialized_deferred"] == counts["eligible"]

    # Direct deferred expansion is a DIFFERENT writer, only a semantic oracle here.
    pending, expanded = [root], []
    while pending:
        k = pending.pop()
        w = words[k]
        counts["reference_expansion_visits"] += 1
        counts["deferred_traversals"] += w.deferred
        if w.left < 0:
            expanded.extend(w.value)
        else:
            pending.extend((w.right, w.left))
    assert tuple(expanded) == materialized[root]
    j = next(i for i, (_, mark) in enumerate(expanded) if mark)
    p = len(expanded)
    start = (j + 1) % p
    # Explicit hypothetical four-lane start discovery, not a native implementation.
    # Length-indexed tree descent charges each node read; no free evenly-spaced starts.
    lane_offsets = sorted({k * p // 4 for k in range(4)})
    counts["potential_lanes"] += len(lane_offsets)
    for lane in lane_offsets[1:]:
        rank, k = (start + lane) % p, root
        while words[k].left >= 0:
            counts["lane_start_node_visits"] += 1
            left, right = words[k].left, words[k].right
            length = len(words[left].value)
            if rank < length:
                k = left
            else:
                rank -= length
                k = right
        counts["lane_start_leaf_visits"] += 1
        assert words[k].value[rank] == expanded[(start + lane) % p]
    rotated = expanded[start:] + expanded[:start]
    decoded = bytes(rotated[i % p][0] for i in range(n))
    x, direct = origin, bytearray()
    for _ in range(n):
        x = inverse[x]
        direct.append(column[x])
    assert decoded == direct
    counts["decoded_bytes"] += n
    counts["short_cycle"] += p < n
    counts["period_not_divisor"] += n % p != 0
    counts["naive_drop_left_mismatch"] += any(
        w.deferred and w.value != words[w.right].value for w in words
    )
    return counts, witnesses


def main():
    """Execute the frozen finite inventory and print reproducible counts."""
    totals, witnesses = Counter(), {}
    inventories = []
    for alphabet, limit in [(2, 8), (3, 6)]:
        cases = 0
        for n in range(1, limit + 1):
            for column in itertools.product(range(alphabet), repeat=n):
                for origin in range(n):
                    c, w = run(column, origin)
                    totals.update(c)
                    for key, value in w.items():
                        witnesses.setdefault(key, value)
                    cases += 1
        inventories.append(dict(alphabet=alphabet, max_n=limit, cases=cases))
    # Fixed larger columns; these are arbitrary LF columns, not named corpus files.
    special = [
        bytes([1] + [0] * 17),
        bytes([1] * 33 + [0] * 32),
        bytes([0, 1] * 32),
        bytes([2, 1, 0] * 21),
        bytes([0] * 128),
    ]
    special_cases = 0
    for column in special:
        for origin in sorted({0, 1, len(column) // 2, len(column) - 1}):
            c, w = run(column, origin)
            totals.update(c)
            for key, value in w.items():
                witnesses.setdefault(key, value)
            special_cases += 1
    # Logical budget still charges every elided step: preserving refusal is mandatory.
    budget_checks = []
    for budget in range(3):
        c, _ = run(bytes([1, 0, 0]), 0, budget)
        assert c["refused"] == (budget < 2)
        budget_checks.append(dict(budget=budget, counts=dict(c)))
    assert totals["eligible"] > 0 and totals["deferred_consumed_full"] > 0
    assert totals["eligible_long"] > 0 and totals["period_not_divisor"] > 0
    print(
        json.dumps(
            dict(
                inventory=inventories,
                special_cases=special_cases,
                counts=dict(totals),
                witnesses=witnesses,
                budget_checks=budget_checks,
                verdict="semantic reference only; no native feasibility PASS",
            ),
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
