"""Build and verify labelled half-frontiers on projected Wikidata inputs."""
import argparse
import csv
import hashlib
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
sys.path.insert(0, str(ROOT / "docs/journal/2026-11-18-astra-c-output-first"))
sys.path.insert(0, str(ROOT / "research/astra-e/formulations/f2-heaviest/union"))
from direct import Column, heavy_set
from inputs import check_frozen, dense_audit, paths, read_input

COLUMNS = """ordinal section mode lap runs visits phases edge_steps advance_pieces
indexed_seeks seek_cmps sort_keys sort_cmps join_steps owner_steps survivor_writes
hit_spans final_bands forward_nodes back_nodes words dictionary_bytes peak_packet_slots
build_cpu_s output_cpu_s""".split()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("inputs", type=Path)
    parser.add_argument("out", type=Path)
    parser.add_argument("--laps", type=int, default=6)
    parser.add_argument("--match", action="store_true", help="Run the complete reference matcher and fresh-process memory samples.")
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    with (tempfile.TemporaryDirectory() as tmp,
          (args.out / "inputs.csv").open("w") as manifest,
          (args.out / "laps.csv").open("w") as laps):
        fields = ("ordinal tape_sha256 column_sha256 marked_inverse_sha256 rows mark runs "
                  "quote_start quote_end compound_start compound_end heavy_pops heavy_pieces "
                  "selection_cpu_s").split()
        writer = csv.writer(manifest, lineterminator="\n")
        writer.writerow(fields)
        columns = ["ordinal", "section", "mode", "lap", "total_cpu_s", "process_hwm_kib"] if args.match else COLUMNS
        print(",".join(columns), file=laps)
        if args.match:
            (args.out / "memory.csv").write_text(",".join(columns) + "\n")
        tapes = paths(args.inputs)
        assert tapes, "no irec inputs"
        for tape in tapes:
            ordinal = int(tape.name.split(".")[0])
            runs, mark = read_input(tape)
            started = time.process_time()
            col = Column(runs)
            quote = col.interval(b'"')
            compound, pops, pieces = heavy_set(col, quote, 1)
            select = time.process_time() - started
            column, marked, _ = dense_audit(runs, mark)
            check_frozen(ordinal, column, marked, mark)
            binary_input = Path(tmp, "runs.txt")
            expected = Path(tmp, "expected.bin")
            expected.write_bytes(marked)
            with binary_input.open("w") as stream:
                print(col.rows, mark, len(runs), 2, file=stream)
                stream.writelines(f"{b} {n}\n" for b, n in runs)
                for name, (lo, hi) in [("quote", quote), ("compound", compound[0])]:
                    print(name, 1, lo, hi, file=stream)
            writer.writerow([ordinal, digest(tape.read_bytes()), digest(column), digest(marked),
                             col.rows, mark, len(runs), *quote, *compound[0], pops, pieces,
                             f"{select:.6f}"])
            manifest.flush()
            laps.flush()
            subprocess.run([args.binary.resolve(), binary_input, str(args.laps), str(ordinal),
                            expected], check=True, stdout=laps)
            if args.match:
                with (args.out / "memory.csv").open("a") as memory:
                    for section in ["quote", "compound"]:
                        for method in ["fixed_sort", "fixed_radix", "indexed_direct"]:
                            subprocess.run([args.binary.resolve(), binary_input, "1", str(ordinal),
                                            expected, method, section], stdout=memory, check=True)
            print(f"verified tape {ordinal:04}", file=sys.stderr, flush=True)


if __name__ == "__main__":
    main()
