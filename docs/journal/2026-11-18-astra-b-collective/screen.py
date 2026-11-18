"""Exact screens from capped records. Python 3.10+, no event or grammar input."""
import argparse
import csv
from collections import Counter
from pathlib import Path
from struct import iter_unpack
import sys


def bands_from_records(tape):
    runs = []
    sizes = [0] * 256
    for width, byte in iter_unpack("BB", tape):
        assert 1 <= width <= 127
        sizes[byte] += width
        if runs and runs[-1][0] == byte:
            runs[-1] = (byte, width + runs[-1][1])
        else:
            runs.append((byte, width))
    cursor = []
    total = 0
    for size in sizes:
        cursor.append(total)
        total += size
    bands = []
    x = 0
    for byte, width in runs:
        bands.append((x, cursor[byte], width, byte))
        x += width
        cursor[byte] += width
    return bands


def rotations(bands, n):
    mass = Counter()
    for x, y, w, _ in bands:
        mass[(y - x) % n] += w
    step = min(mass, key=lambda d: (-mass[d], d))
    return dict(rotation=step, shifts=len(mass), rotation_errors=n - mass[step])


def tiles(bands, n, size, primary):
    # Only real rows vote. The oracle may route a partial last tile as freely
    # as a full one and permute lanes arbitrarily.
    count = (n + size - 1) // size
    col_best = [0] * count
    rigid_col_best = [0] * count
    piece_count = pair_count = row_best = rigid_best = 0
    pairs = Counter()
    votes = Counter()
    for x, y, width, _ in bands:
        while width:
            take = min(width, size - x % size, size - y % size)
            dest = y // size
            pairs[dest] += take
            votes[dest, (y - x) % size] += take
            piece_count += 1
            x += take
            y += take
            width -= take
            if x % size == 0 or x == n:
                pair_count += len(pairs)
                row_best += max(pairs.values())
                rigid_best += max(votes.values())
                for j, mass in pairs.items():
                    col_best[j] = max(col_best[j], mass)
                for (j, _), mass in votes.items():
                    rigid_col_best[j] = max(rigid_col_best[j], mass)
                pairs.clear()
                votes.clear()
    return dict(tiles=count, fragments=piece_count, tile_pairs=pair_count,
                arbitrary_min_errors=n-min(row_best, sum(col_best)),
                shift_min_errors=n-min(rigid_best, sum(rigid_col_best)))


def clean_links(bands, size, primary):
    # Series of full aligned tiles can be recognized directly on each run.
    links = {}
    groups = 0
    marked_tile = primary // size
    for x, y, w, byte in bands:
        if (y-x) % size:
            continue
        first, stop = (x+size-1)//size, (x+w)//size
        delta = (y-x)//size
        groups += int(first < min(stop, marked_tile)) + int(max(first, marked_tile+1) < stop)
        for tile in range(first, stop):
            if tile != marked_tile:
                links[tile] = (tile+delta, byte)
    return links, groups


def chains(links):
    incoming = {dest for dest, _ in links.values()}
    assert len(incoming) == len(links)
    heads = links.keys() - incoming
    paths = {}
    steps = 0
    longest = 0
    for head in heads:
        tile = head
        word = bytearray()
        while tile in links:
            tile, byte = links[tile]
            word.append(byte)
        steps += len(word)
        longest = max(longest, len(word))
        paths[head] = (tile, bytes(word))
    return paths, dict(clean_tiles=len(links), paths=len(paths), path_links=steps,
                       cycle_tiles=len(links)-steps, longest=longest)


def inputs(folder, only):
    with (folder/"inputs.csv").open() as f:
        for row in csv.DictReader(f):
            block = int(row["block"])
            if only is None or block in only:
                tape = (folder/f"{block:04}.recs").read_bytes()
                assert len(tape) == 2 * int(row["records"])
                bands = bands_from_records(tape)
                n, primary = int(row["n"]), int(row["primary"])
                assert sum(w for _, _, w, _ in bands) == n and 0 <= primary < n
                yield block, n, primary, len(tape)//2, bands


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("--blocks", type=int, nargs="+")
    parser.add_argument("--sizes", type=int, nargs="+", default=[2,4,8,16,32,64])
    args = parser.parse_args()
    writer = None
    for block, n, p, m, bands in inputs(args.folder, args.blocks):
        rotation = rotations(bands, n)
        for size in args.sizes:
            assert size > 1
            links, groups = clean_links(bands, size, p)
            _, path_counts = chains(links)
            row = dict(block=block, n=n, primary=p, records=m, runs=len(bands),
                       **rotation, B=size, **tiles(bands, n, size, p),
                       groups=groups, **path_counts)
            if writer is None:
                writer = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)


if __name__ == "__main__":
    main()
