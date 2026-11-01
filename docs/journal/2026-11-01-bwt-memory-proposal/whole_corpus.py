"""Read-only: each phase's CPU time summed over every block of each run (all 434, short
blocks included), divided by the decoded bytes (207,000,624): CPU-s per GB decoded, the
median over the 10 runs at 1 and at 4 workers. The full-block figures (phases.py) are per
BWT row; these are per decoded byte, the decoder-level denominator."""
import collections, json, statistics, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from phases import load, blocks  # noqa: E402

DECODED = 207_000_624
PH = [("mtf", "DecodeStart", "InvertStart"), ("build", "InvertStart", "WalkStart"), ("walk_and_assembly", "WalkStart", "InvertEnd"), ("rle1_crc", "InvertEnd", "DecodeEnd")]


def main():
    d = Path(sys.argv[1])
    out = {}
    for t in (1, 4):
        per_run = collections.defaultdict(list)
        nblocks = []
        for f in sorted(d.glob(f"r??-t{t}-full.tsv")):
            bl = [b for b in blocks(load(f)) if all(k in b["m"] and b["m"][k][1] is not None for k in ("DecodeStart", "InvertStart", "WalkStart", "InvertEnd", "DecodeEnd"))]
            nblocks.append(len(bl))
            for name, a, z in PH:
                per_run[name].append(sum(b["m"][z][1] - b["m"][a][1] for b in bl) / DECODED)  # ns per byte = CPU-s per GB
        out[t] = {name: round(statistics.median(v), 4) for name, v in per_run.items()} | dict(blocks_per_run=sorted(set(nblocks)))
    out["increase_cpu_s_per_gb"] = {name: round(out[4][name] - out[1][name], 4) for name, _, _ in PH}
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
