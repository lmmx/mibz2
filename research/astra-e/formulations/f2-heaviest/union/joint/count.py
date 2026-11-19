"""Construct a shared return trie while preserving every component's tagged source interval."""
from bisect import bisect_right
from collections import defaultdict
from pathlib import Path
import argparse
import csv
import hashlib
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from direct import Column, Direct, components, heavy_set


def discover(col, chosen):
    starts = [a for a, _ in chosen]
    stack = [(0, [(color, a, b) for color, (a, b) in enumerate(chosen)])]
    # Every path node is shared across colors. Terminal paths are reconstructed at the end.
    paths = [(None, None)]
    terminals = []
    colors = nodes = pieces = probes = same_hull = affine_colored = 0
    while stack:
        path, intervals = stack.pop()
        nodes += 1
        colors += len(intervals)
        branches = defaultdict(list)
        last_run = -1
        first_run = bisect_right(col.starts, intervals[0][1]) - 1
        if intervals[-1][2] <= col.starts[first_run + 1]:
            same_hull += 1
            affine_colored += len(intervals)
        for color, a, b in intervals:
            if last_run >= 0 and a < col.starts[last_run + 1]:
                run = last_run
            else:
                # Reuse the already located first run when visiting the first interval.
                run = first_run if last_run == -1 else bisect_right(col.starts, a) - 1
                probes += 1
            assert col.starts[run] <= a < b
            while a < b:
                stop = min(b, col.starts[run + 1])
                byte = col.symbols[run]
                left, right = a + col.maps[run], stop + col.maps[run]
                entries = branches[byte]
                if entries and entries[-1][0] == color:
                    assert entries[-1][2] == left
                    entries[-1] = (color, entries[-1][1], right)
                else:
                    assert not entries or entries[-1][2] <= left
                    entries.append((color, left, right))
                pieces += 1
                last_run = run
                a = stop
                run += 1
        for byte, entries in branches.items():
            child = len(paths)
            paths.append((path, byte))
            hits = []
            for _, a, b in entries:
                i = bisect_right(starts, a) - 1
                hit = i >= 0 and a < chosen[i][1]
                assert (b <= chosen[i][1]) if hit else (i + 1 == len(starts) or b <= starts[i+1])
                hits.append(hit)
            assert len(set(hits)) == 1
            if hits[0]:
                terminals.append((child, entries))
            else:
                stack.append((child, entries))
    dictionary = {}
    for path, entries in terminals:
        word = bytearray()
        while path:
            path, byte = paths[path]
            word.append(byte)
        key = bytes(word)
        for color, a, b in entries:
            assert (key, color) not in dictionary
            dictionary[key, color] = (a, b)
    assert nodes <= probes <= colors <= pieces
    return dictionary, dict(nodes=nodes, colored_nodes=colors, predecessor_searches=probes,
                            pieces=pieces, homogeneous_nodes=same_hull, homogeneous_colors=affine_colored,
                            words=len(terminals), tagged_words=len(dictionary),
                            unique_bytes=sum(len(word) for word in set(w for w,_ in dictionary)),
                            tagged_bytes=sum(len(word) for word,_ in dictionary))


def actual(col, selected):
    comp = components(selected)
    observed, cost = discover(col, comp)
    direct = Direct(col, selected)
    expected = {}
    for a, _, width, word, color in direct.bands:
        if (word, color) in expected:
            assert expected[word, color][1] == a
            expected[word, color] = expected[word, color][0], a + width
        else:
            expected[word, color] = a, a + width
    assert observed == expected
    return dict(components=len(comp), selected=sum(b-a for a,b in comp), **cost)


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("tapes", type=Path)
    p.add_argument("--blocks", type=int, nargs="+")
    a = p.parse_args()
    root = HERE.parents[5]
    sys.path.insert(0, str(root / "docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths, read_input
    out = None
    for tape in paths(a.tapes):
        ordinal = int(tape.name.split(".")[0])
        if a.blocks and ordinal not in a.blocks:
            continue
        runs, _ = read_input(tape)
        col = Column(runs)
        section = col.interval(b'"')
        for k in (1, 2, 4, 8):
            row = dict(ordinal=ordinal, tape_sha256=hashlib.sha256(tape.read_bytes()).hexdigest(),
                       k=k, **actual(col, heavy_set(col, section, k)[0]))
            if out is None:
                out = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                out.writeheader()
            out.writerow(row)
            sys.stdout.flush()
