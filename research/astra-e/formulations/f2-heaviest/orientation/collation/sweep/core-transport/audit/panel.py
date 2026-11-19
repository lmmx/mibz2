"""Census on the ten original run tapes. The dense verifier never supplies C or Z to the selector."""
from array import array
from collections import Counter
from pathlib import Path
import csv
import hashlib
import sys

from intervals import Mask, RunIndex, consecutive, heaviest

HERE = Path(__file__).resolve().parent
REPO = next(p for p in HERE.parents if (p / 'Cargo.toml').exists())
sys.path.insert(0, str(HERE.parents[1] / 'short-core'))
sys.path.insert(0, str(REPO / 'docs/journal/2026-11-18-astra-c-output-first'))
from induce import Atlas, Column, heavy_set
from inputs import paths, read_input


def arcs(last, forward, start, stop):
    rows = []
    for source in range(start, stop):
        word = bytearray()
        x = source
        while True:
            x = forward[x]
            word.append(last[x])
            if start <= x < stop:
                break
        rows.append(bytes(word))
    return rows


def check(winner, left, right, mass, geom, words, section, h, forward, winners):
    # Every positive-hit source is in exactly one original arc ending in U.
    # Its length-|C| suffix, if it exists, uniquely supplies a row of Z.
    found = []
    selected = []
    counts = Counter()
    full_steps = selected_steps = 0
    for source, word in enumerate(words, section[0]):
        t = min(h, len(word)-1)
        core = word[t:]
        counts[core] += 1
        if word.endswith(winner):
            x = source
            depth = len(word) - len(winner)
            for _ in range(depth):
                x = forward[x]
            full_steps += depth
            found.append(x)
            if core == winner:
                selected.append(x)
                selected_steps += depth
    assert mass == max(counts.values())
    assert winners == sorted(k for k in counts if counts[k] == mass)
    assert winner == winners[0]
    assert sorted(found) == list(range(left, right))
    assert consecutive(selected) == geom.components(left, right)
    return len(counts), len(found), len(selected), full_steps, selected_steps


def bridge(mark, forward, pieces):
    mask = Mask(pieces)
    x = mark
    for distance in range(len(forward)):
        if mask.rank(x+1) > mask.rank(x):
            return distance, x
        x = forward[x]
        if x == mark:
            break
    return -1, -1


def main(tapes):
    writer = None
    for path in paths(tapes):
        runs, mark = read_input(path)
        last = b''.join(bytes((byte,)) * width for byte, width in runs)
        forward = array('I', sorted(range(len(last)), key=last.__getitem__))
        index = RunIndex(runs)
        column = Column(runs)
        quote = index.base[34], index.base[35]
        picked, pops, pieces = heavy_set(column, quote, 1)
        for name, section in [('frequent', picked[0]), ('quote', quote)]:
            words = arcs(last, forward, *section)
            for h in (4, 8, 12):
                atlas = Atlas(column, section, h)
                mask = Mask((lo, lo+w) for _, lo, w, _ in atlas.leaves)
                mass, ties, costs = heaviest(index, section, mask)
                core, left, right = ties[0]
                assert index.interval(core, section) == (left, right)
                types, full, nsel, fsteps, ssteps = check(
                    core, left, right, mass, mask, words, section, h, forward, [c for c, _, _ in ties])
                assert full == right-left and nsel == mass
                zdist, zrow = bridge(mark, forward, [(left, right)])
                vdist, vrow = bridge(mark, forward, mask.components(left, right))
                row = dict(ordinal=int(path.name.split('.')[0]), section=name, h=h, rows=len(last), runs=len(runs),
                           input_sha256=hashlib.sha256(path.read_bytes()).hexdigest(), mark=mark,
                           u_lo=section[0], u_hi=section[1], u_mass=section[1]-section[0],
                           section_select_pops=pops if name=='frequent' else 0,
                           section_select_run_pieces=pieces if name=='frequent' else 0,
                           atlas_forward_intersections=atlas.census['forward_intersections'],
                           atlas_bands=len(atlas.leaves), v_components=len(mask.starts), **costs,
                           n_core_types=types, n_top_ties=len(ties), top_core_hex=core.hex(), z_lo=left, z_hi=right,
                           z_mass=full, selected_mass=mass, extra_rows=full-mass,
                           selected_components=len(mask.components(left,right)),
                           mark_to_z=zdist, mark_first_z_row=zrow,
                           mark_to_selected=vdist, mark_first_selected_row=vrow,
                           full_components=1, full_interval_endpoint_queries=2*len(core),
                           verifier_old_arc_steps=sum(map(len,words)), verifier_full_offset_steps=fsteps,
                           verifier_selected_offset_steps=ssteps)
                if writer is None:
                    writer = csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                    writer.writeheader()
                writer.writerow(row)
                sys.stdout.flush()


if __name__ == '__main__':
    main(Path(sys.argv[1]))
