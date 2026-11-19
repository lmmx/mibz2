"""Summarize exact queue geometry and simulated cold-per-block cache misses."""
import csv
import sys
from collections import Counter, defaultdict
from pathlib import Path
from statistics import median

base = Path(sys.argv[1])


def read(name):
    with (base / name).open() as f:
        return list(csv.DictReader(f))


blocks = read("blocks.csv")
alphabets = defaultdict(list)
for row in read("alphabet.csv"):
    alphabets[row["ordinal"]].append(int(row["pieces"]))

totals = Counter()
for block in blocks:
    totals.update({k: int(v) for k, v in block.items() if k != "ordinal"})

size = max(int(b["pieces"]) for b in blocks)
print(f"blocks={len(blocks)} rows={totals['n']} queue_rows={totals['queued_rows']} "
      f"pieces={totals['pieces']} singletons={totals['singletons']} "
      f"width1_to_4={sum(totals[k] for k in ('singletons','short2','short3','short4'))}")
print(f"descriptor_bytes_written={4*totals['pieces']} descriptor_bytes_read={4*totals['pieces']} "
      f"max_block_live_descriptor_bytes={4*size}")
for k in ["active_bytes", "active_nibbles"]:
    values = [int(b[k]) for b in blocks]
    print(f"{k}_min_median_max={min(values)} {median(values)} {max(values)}")
for k in [4, 8, 16, 32]:
    hottest = sum(sum(sorted(freq, reverse=True)[:k]) for freq in alphabets.values())
    print(f"per_block_hottest_{k}_piece_fraction={hottest/totals['pieces']:.6f}")

measurements = read("locality.csv")
keys = list(measurements[0])[3:]
sums = defaultdict(Counter)
blocks_by_mode = {}
for row in measurements:
    counts = Counter({k: int(row[k]) for k in keys})
    sums[row["layout"], row["mode"]].update(counts)
    blocks_by_mode[row["ordinal"], row["layout"], row["mode"]] = counts
print("layout mode", *keys)
for (layout, mode), counts in sums.items():
    print(layout, mode, *(counts[k] for k in keys))
print("blocks with fewer complete staged misses than direct")
print("layout", *keys[2:])
for layout in ("0", "1", "2"):
    wins = Counter()
    for b in blocks:
        direct, staged = (blocks_by_mode[b["ordinal"], layout, m] for m in ("direct", "staged"))
        for k in keys[2:]:
            wins[k] += staged[k] < direct[k]
    print(layout, *(wins[k] for k in keys[2:]))
