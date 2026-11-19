"""Induce an ordinary token column on the moved section, directly from the core RLE."""
import csv
import json
import sys
from collections import Counter
from itertools import groupby, product
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from return_map import Atlas, Column, Oracle, append_run, heavy_set, incoming, truth


def physical(u, atlas, stream, cores, minima, counts):
    # Prefix bands in physical order, with each old source lane's scatter destination.
    lanes = []
    buffers = []
    physical_base = 0
    for origin, _, width, prefix in atlas.leaves:
        lanes.append((origin, origin + width, physical_base, prefix, len(buffers)))
        buffers.append([])
        physical_base += width
    lanes.sort()
    assert lanes[0][0] == u[0] and lanes[-1][1] == u[1]
    assert all(a[1] == b[0] for a, b in zip(lanes, lanes[1:]))

    cursors = {}
    source = 0
    for core in sorted(cores, key=minima.get):
        cursors[core] = source
        source += counts[core]
    assert source == physical_base
    last = {}
    labels = {}
    stable_types = len(cores)
    j = fragments = 0
    dest = u[0]
    for core, width in stream:
        while width:
            start, stop, base, prefix, lane = lanes[j]
            take = min(width, stop - dest)
            assert take > 0
            moved = base + dest - start
            prev = last.get(core)
            if prev is not None:
                old_prefix, label, last_end = prev
                if moved < last_end:
                    stable_types += 1
                    assert old_prefix != prefix
            if prev is None or old_prefix != prefix:
                label = cursors[core]
                labels[label] = (core, prefix)
            last[core] = (prefix, label, moved + take)
            append_run(buffers[lane], label, take)
            cursors[core] += take
            fragments += 1
            width -= take
            dest += take
            if dest == stop:
                j += 1

    # Label IDs are source bases; no comparison of byte spellings determines their order.
    column = []
    for buffer in buffers:
        for label, width in buffer:
            append_run(column, label, width)
    assert dest == u[1] and len(column) <= fragments <= len(stream) + len(lanes) - 1
    return column, labels, fragments, stable_types


def verify(col, oracle, u, h, marks):
    atlas = Atlas(col, u, h)
    stream, cores, minima, counts, cost = incoming(col, u, atlas)
    column, labels, fragments, stable = physical(u, atlas, stream, cores, minima, counts)
    old, original_in = truth(oracle, u)
    # Dense independent verification only. Stable F slots and physical byte partners.
    head = {label: label for label in labels}
    rows = [r for _, lo, w, _ in atlas.leaves for r in range(lo, lo + w)]
    ordinal = {r: i for i, r in enumerate(rows)}
    spells = {k: cores[c] + p for k, (c, p) in labels.items()}
    by_source = [None] * len(rows)
    dest = 0
    for label, width in column:
        for _ in range(width):
            s = head[label]
            head[label] += 1
            assert by_source[s] is None
            x = rows[s]
            word = bytearray()
            while True:
                word.append(oracle.first[x])
                x = oracle.forward[x]
                if x in ordinal:
                    break
            assert ordinal[x] == dest and word == spells[label]
            by_source[s] = (dest, label)
            dest += 1
    assert all(entry is not None for entry in by_source)
    for mark in marks:
        x = mark
        expected = bytearray()
        for _ in range(col.rows):
            expected.append(oracle.first[x])
            x = oracle.forward[x]
        # Transport from the mark to the first old source, then append that source's
        # prefix before handing off to the physical-column decoder.
        x = mark
        out = bytearray()
        while not u[0] <= x < u[1] and len(out) < col.rows:
            out.append(oracle.first[x])
            x = oracle.forward[x]
        if len(out) < col.rows:
            for origin, lo, width, prefix in atlas.leaves:
                if origin <= x < origin + width:
                    out.extend(prefix[:col.rows-len(out)])
                    v = ordinal[lo + x - origin]
                    break
            while len(out) < col.rows:
                v, label = by_source[v]
                out.extend(spells[label][:col.rows-len(out)])
        assert out == expected

    # Even dropping core identity, every contiguous fixed-spelling alphabet must
    # break wherever the physical spelling changes in the F order. Equal spellings
    # transport monotonically, so these groups attain the lower bound.
    canonical = {}
    previous = None
    for label in sorted(labels):
        spell = spells[label]
        if spell != previous:
            representative = label
        canonical[label] = representative
        previous = spell
    minimal = []
    for label, width in column:
        append_run(minimal, canonical[label], width)
    minimal_heads = {k: k for k in set(canonical.values())}
    d = 0
    for label, width in minimal:
        for _ in range(width):
            s = minimal_heads[label]
            minimal_heads[label] += 1
            assert by_source[s][0] == d and spells[by_source[s][1]] == spells[label]
            d += 1
    original_runs = sum(1 for _ in groupby(original_in))
    return dict(mass=len(rows), original_K=len(old), core_K=len(cores), stability_K=stable,
                refined_K=len(labels), minimal_K=len(minimal_heads), original_R=original_runs,
                core_R=len(stream), refined_R=len(column), minimal_R=len(minimal),
                max_label_mass=max(head[k]-k for k in labels), prefix_bands=len(atlas.leaves),
                overlay_fragments=fragments, original_D=sum(map(len,old)),
                refined_D=sum(map(len,spells.values())), minimal_D=sum(len(spells[k]) for k in minimal_heads),
                unique_D=sum(map(len,set(spells.values()))), unique_K=len(set(spells.values())),
                factored_D=sum(map(len, cores.values()))+sum(map(len,{p for _,p in labels.values()})),
                lookup_states=cost['live_states'], marked=len(marks))


def small():
    counts = Counter()
    witnesses = {}
    for n in range(1, 8):
        for x in product(range(2), repeat=n):
            col = Column([(c, sum(1 for _ in g)) for c,g in groupby(x)])
            o = Oracle(bytes(x))
            for a in range(n):
                for b in range(a+1, n+1):
                    for h in (0, 1, 3, n):
                        row = verify(col, o, (a,b), h, range(n))
                        counts.update(objects=1, marks=n, partners=b-a)
                        for name, condition in [('descent',row['stability_K']>row['core_K']),
                             ('payload_split',row['refined_K']>row['stability_K']),
                             ('cross_core_merge',row['minimal_K']<row['refined_K']),
                             ('recurring_equal_spelling',row['minimal_K']>row['unique_K'])]:
                            if condition:
                                counts[name] += 1
                                witnesses.setdefault(name,dict(column=bytes(x).hex(),u=[a,b],h=h,**row))
    print(json.dumps(dict(counts=counts,witnesses=witnesses),indent=2))


def real(path, section):
    sys.path.insert(0,str(HERE.parents[8]/'docs/journal/2026-11-18-astra-c-output-first'))
    from inputs import paths, read_input
    writer = None
    for tape in paths(path):
        runs, mark = read_input(tape)
        col = Column(runs)
        oracle = Oracle(b''.join(bytes([c])*w for c,w in runs))
        q = col.interval(b'"')
        u = q if section=='quote' else heavy_set(col,q,1)[0][0]
        for h in (0,4,8,12):
            row = dict(ordinal=int(tape.name.split('.')[0]),section=section,h=h,**verify(col,oracle,u,h,[mark]))
            if writer is None:
                writer = csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__=='__main__':
    real(sys.argv[1],sys.argv[2]) if len(sys.argv)>1 else small()
