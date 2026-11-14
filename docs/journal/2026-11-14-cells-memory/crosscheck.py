"""memtrace's counts against cachegrind's, per kernel phase (the README's cross-check).

    valgrind --tool=cachegrind --cache-sim=yes --I1=32768,8,64 --D1=49152,12,64 \
      --LL=2097152,16,64 --cachegrind-out-file=CG cells_memory wikidata wikidata.bz2 cells
    python3 crosscheck.py CG trace-cells.jsonl

Cachegrind's self Dr/Dw of Kernel::slots, ::induce_cells, ::write_cells (memset and memcpy
are separate functions in cachegrind, inside write_cells's phases in memtrace), and its
two-level set-associative misses, beside memtrace's loads and stores over all Cells attempts.
"""
import collections
import json
import sys

cg = collections.defaultdict(lambda: [0] * 9)
fn = None
for line in open(sys.argv[1]):
    if line.startswith("fn="):
        fn = line[3:].strip()
    elif line[:1].isdigit() and fn:
        v = list(map(int, line.split()[1:]))
        v += [0] * (9 - len(v))
        for i in range(9):
            cg[fn][i] += v[i]
names = {"slots": "<mibz2::cells::Kernel>::slots",
         "induce": "<mibz2::cells::Kernel>::induce_cells",
         "write": "<mibz2::cells::Kernel>::write_cells"}
group = {10: "slots", 11: "slots", 12: "slots", 20: "induce"}
for p in range(30, 37):
    group[p] = "write"
mine = collections.defaultdict(lambda: [0, 0])
for line in open(sys.argv[2]):
    a = json.loads(line)
    for s in a["stats"]:
        g = group.get(s["phase"])
        if g:
            mine[g][0 if s["rw"] == "r" else 1] += s["accesses"]
print("events: Dr D1mr DLmr Dw D1mw DLmw (cachegrind self) | loads stores (memtrace)")
for g, f in names.items():
    v = cg[f]
    print(f"{g}: cachegrind Dr={v[3]:,} D1mr={v[4]:,} DLmr={v[5]:,} Dw={v[6]:,} D1mw={v[7]:,} "
          f"DLmw={v[8]:,} | memtrace loads={mine[g][0]:,} stores={mine[g][1]:,}")
for f, v in sorted(cg.items()):
    if "mem" in f and v[3] + v[6] > 1e6:
        print(f"{f} (whole program): Dr={v[3]:,} Dw={v[6]:,} DLmr={v[5]:,} DLmw={v[8]:,}")
