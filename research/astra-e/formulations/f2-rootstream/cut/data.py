"""Per-node statistics from independently spelled quote returns in root order."""
import csv
import lzma
from itertools import groupby
from pathlib import Path
import sys
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "audit"))
from real import ROOT, compact, read_input, spell
from tree import routes

def statistics(labels, parent):
    paths = routes(parent)
    nodes = len(parent)
    local = [0] * nodes
    root = [0] * nodes
    selectors = [0] * nodes
    last_label = [None] * nodes
    last_choice = [None] * nodes
    incoming = [0] * nodes
    term_width = [0] * nodes
    for t, group in groupby(labels):
        term_width[t] += sum(1 for _ in group)
        path = paths[t]
        for i,v in enumerate(path):
            choice = path[i+1] if i+1 < len(path) else v
            root[v] += 1
            local[v] += int(last_label[v] != t)
            if last_choice[v] != choice:
                selectors[v] += 1
                if choice != v:
                    incoming[choice] += 1
            last_label[v] = t
            last_choice[v] = choice
    assert local[0] == root[0]
    assert all(g >= r >= s > 0 for g, r, s in zip(root, local, selectors))
    return [(v, p, len(paths[v])-1, selectors[v], local[v], root[v], incoming[v], term_width[v])
            for v, p in enumerate(parent)]

if __name__ == "__main__":
    with open(HERE.parent/"audit/real.csv") as f:
        expected = {int(r["block"]):r for r in csv.DictReader(f)}
    target = sys.argv[1]
    with lzma.open(target, "wt", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["block","node","parent","depth","S","R","G","incoming_pieces","terminal_width"])
        for path in sorted((ROOT/"research/astra-e/dense/target/wikidata").glob("*.irec.xz")):
            runs, _ = read_input(path)
            column = b"".join(bytes([c])*width for c,width in runs)
            unique, labels, _ = spell(column, ord('"'))
            parent, term = compact(unique)
            node_rows = statistics((term[v] for v in labels),parent)
            b = int(path.name.split(".")[0])
            ref = expected[b]
            assert len(node_rows) == int(ref["nodes"])
            assert [sum(x[i] for x in node_rows) for i in (3,4,5)] == [
                int(ref[f]) for f in ("selectors","materialized_runs","visits")]
            w.writerows((b,*r) for r in node_rows)
