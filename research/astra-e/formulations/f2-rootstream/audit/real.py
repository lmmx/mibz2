"""Build the actual delimiter trie by dense per-row spelling and reverse projection."""
import csv
import hashlib
import sys
from array import array
from itertools import accumulate
from pathlib import Path

from tree import audit

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / "docs/journal/2026-11-18-astra-c-output-first"))
from inputs import read_input

def spell(column, delimiter):
    freq = [column.count(c) for c in range(256)]
    base = list(accumulate([0] + freq[:-1]))
    first = base[delimiter]
    lf = array("I")
    for c in column:
        lf.append(base[c])
        base[c] += 1
    # Read every first-return word by independent, dense LF, in section rank order.
    unique = {}
    labels = []
    covered = 0
    for p in range(first, first + freq[delimiter]):
        word = bytearray()
        while column[p] != delimiter:
            word.append(column[p])
            p = lf[p]
        covered += len(word) + 1
        key = bytes(word)
        labels.append(unique.setdefault(key, len(unique)))
    return unique, labels, covered

def compact(words):
    children = [{}]
    terminal = {}
    for word, label in words.items():
        v = 0
        for c in word:
            if c not in children[v]:
                children[v][c] = len(children)
                children.append({})
            v = children[v][c]
        terminal[v] = label
    parent = []
    terminal_node = {}
    stack = [(0, -1)]
    while stack:
        u, p = stack.pop()
        if u in terminal or len(children[u]) != 1:
            pid = len(parent)
            parent.append(p)
            if u in terminal:
                terminal_node[terminal[u]] = pid
            p = pid
        stack.extend((v, p) for v in children[u].values())
    return parent, terminal_node

def row(path):
    runs, primary = read_input(path)
    column = b"".join(bytes([c]) * width for c, width in runs)
    unique, labels, covered = spell(column, ord('"'))
    parent, terminal = compact(unique)
    counts = audit([terminal[v] for v in labels], parent)
    assert counts["visits"] <= covered <= len(column)
    out = dict(block=int(path.name.split(".")[0]), rows=len(column), primary=primary,
               column_sha256=hashlib.sha256(column).hexdigest(), covered=covered,
               words=len(unique), token_rows=len(labels))
    for k in ("nodes","selectors","materialized_runs","root_runs","visits"):
        out[k] = counts[k]
    return out

if __name__ == "__main__":
    geom = HERE.parent / "geometry.csv"
    with geom.open() as f:
        reference = {(int(r["rows"]), int(r["primary"])): r for r in csv.DictReader(f)}
    inputs = sorted((ROOT / "research/astra-e/dense/target/wikidata").glob("*.irec.xz"))
    assert len(inputs) == 10
    w = None
    for path in inputs:
        r = row(path)
        ref = reference[(r["rows"],r["primary"])]
        r["candidate_admitted"] = int(ref["took"])
        if r["candidate_admitted"]:
            for k in ("words","token_rows","nodes","selectors","root_runs"):
                assert r[k] == int(ref[k]), (k,r,ref)
            assert r["visits"] == int(ref["merge_steps"])
        if w is None:
            w = csv.DictWriter(sys.stdout, fieldnames=list(r),lineterminator="\n")
            w.writeheader()
        w.writerow(r)
