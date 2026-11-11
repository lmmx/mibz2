"""Freeze identical eligible deletion schedules using only the original code."""

import json
from pathlib import Path

from common import chosen_color, initial, inputs, original, ranges


def native_contract(pieces, grammar, marker, seed):
    """Run unchanged batch source, with the registered special coloring if needed."""
    old_color = original.color
    if seed == -1:
        original.color = lambda word, _: int(word == ord("b"))
    try:
        return original.contract(pieces, grammar, marker, seed)
    finally:
        original.color = old_color


def one(name, column, origin, special=False):
    """Record original-row identities for every deletion, before new comparisons."""
    pieces, grammar, kept = initial(column, special)
    marker = kept.index(origin)
    record = dict(
        family=name,
        column=column.hex(),
        origin=origin,
        initial_keep=ranges(kept),
        rounds=[],
        special=special,
    )
    for seed in [-1] if special else [1, 2, 3, 4]:
        forward, words = {}, {}
        for a in pieces:
            for x in range(a.s, a.s + a.n):
                forward[x], words[x] = a.t + x - a.s, a.w
        removed = {
            y
            for x, y in forward.items()
            if y != marker
            and chosen_color(words[x], seed) == 0
            and chosen_color(words[y], seed) == 1
        }
        assert not removed.intersection(forward[x] for x in removed)
        record["rounds"].append(
            dict(
                seed=seed,
                kept_before=ranges(kept),
                deleted_packed=ranges(removed),
                deleted_original=ranges(kept[x] for x in removed),
            )
        )
        pieces, marker, stats = native_contract(pieces, grammar, marker, seed)
        assert stats["removed"] == len(removed)
        kept = [x for i, x in enumerate(kept) if i not in removed]
        assert kept[marker] == origin
    return record


def main():
    """Emit one immutable, deterministic schedule file with no candidate calls."""
    records = [one(name, col, p) for name, col, _ in inputs() for p in range(len(col))]
    records.append(one("pr369_cached_regression", b"babaaa", 1, True))
    Path(__file__).with_name("schedules.json").write_text(
        json.dumps(dict(schema=1, cases=records), separators=(",", ":")) + "\n"
    )
    print(json.dumps(dict(cases=len(records), rounds=sum(len(x["rounds"]) for x in records))))


if __name__ == "__main__":
    main()
