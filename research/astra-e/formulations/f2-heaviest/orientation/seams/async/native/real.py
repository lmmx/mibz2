"""Recover the two rank sections, verify native marked bytes and compare the dictionary price oracle."""
import csv
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import hashlib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[7]
sys.path.insert(0, str(ROOT / "docs/journal/2026-11-18-astra-c-output-first"))
sys.path.insert(0, str(ROOT / "research/astra-e/formulations/f2-heaviest/union"))
from direct import Column, heavy_set
from inputs import check_frozen, dense_audit, paths, read_input

FIELDS = """ordinal section rows runs endpoints words states heap_pops stale_repairs edge_row_steps
source_packets edge_comparisons clipped_translations opposite_seeks map_comparisons join_fragments
nonmeeting_fragments meeting_fragments split_remainders coalescences packet_inserts packet_erases
peak_packet_nodes peak_heap cohort_nodes published_bands requested_peak_bytes allocations
heap_comparisons publication_sort_comparisons marked_peak_bytes constructor_cpu_s marked_cpu_s""".split()


def main(binary, tapes, dest):
    dest.mkdir(exist_ok=True, parents=True)
    oracle = {(int(r["ordinal"]), r["section"]): r for r in csv.DictReader((HERE.parent / "wikidata.csv").open())}
    with tempfile.TemporaryDirectory() as tmp, (dest / "wikidata.csv").open("w") as out, (dest / "inputs.csv").open("w") as inputs:
        result = csv.DictWriter(out, fieldnames=FIELDS, lineterminator="\n")
        result.writeheader()
        manifest = csv.writer(inputs, lineterminator="\n")
        manifest.writerow("ordinal tape_sha256 marked_sha256 primary quote_lo quote_hi frequent_lo frequent_hi selection_pops selection_pieces selection_process_s".split())
        for tape in paths(tapes):
            ordinal = int(tape.name.split(".")[0])
            runs, primary = read_input(tape)
            column, expected, _ = dense_audit(runs, primary)
            check_frozen(ordinal, column, expected, primary)
            t = time.process_time()
            col = Column(runs)
            quote = col.interval(b'"')
            compound, pops, pieces = heavy_set(col, quote, 1)
            assert len(compound) == 1
            select_time = time.process_time() - t
            manifest.writerow([ordinal, hashlib.sha256(tape.read_bytes()).hexdigest(), hashlib.sha256(expected).hexdigest(),
                               primary, *quote, *compound[0], pops, pieces, f"{select_time:.6f}"])
            raw, gold = Path(tmp, "input.txt"), Path(tmp, "gold.bin")
            with raw.open("w") as f:
                print(len(expected), primary, len(runs), 2, file=f)
                f.writelines(f"{b} {w}\n" for b, w in runs)
                for name, (lo, hi) in [("quote", quote), ("frequent", compound[0])]:
                    print(name, 1, lo, hi, file=f)
            gold.write_bytes(expected)
            lines = subprocess.check_output([binary.resolve(), raw, gold, str(ordinal)], text=True)
            for r in csv.DictReader(lines.splitlines(), fieldnames=FIELDS):
                ref = oracle[ordinal, r["section"]]
                assert all(r[a] == ref[b] for a, b in [("states", "width_price"), ("heap_pops", "width_heap_pops"), ("words", "words")]), (r, ref)
                result.writerow(r)
            out.flush()
            inputs.flush()
            print(f"verified {ordinal:04}", file=sys.stderr, flush=True)


if __name__ == "__main__":
    main(*map(Path, sys.argv[1:]))
