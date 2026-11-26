"""Check every real packed and reclaimed seam against the pinned dynamic control."""
import argparse
import csv
import hashlib
import subprocess
import tempfile
from pathlib import Path
from price import HERE, dense_audit, paths, read_input


def main():
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("properties", type=Path)
    args.add_argument("tapes", type=Path)
    opts = args.parse_args()
    with (HERE.parent / "native/inputs.csv").open() as f:
        manifest = {int(r["ordinal"]): r for r in csv.DictReader(f)}
    print("ordinal,section,rows,seams_checked_per_variant,states,heap_pops,stale_repairs,edge_rows")
    with tempfile.TemporaryDirectory() as tmp:
        inp, gold = Path(tmp) / "input", Path(tmp) / "gold"
        for tape in paths(opts.tapes):
            ordinal = int(tape.name.split(".")[0])
            m = manifest[ordinal]
            runs, mark = read_input(tape)
            _, expected, _ = dense_audit(runs, mark)
            assert hashlib.sha256(tape.read_bytes()).hexdigest() == m["tape_sha256"]
            assert hashlib.sha256(expected).hexdigest() == m["marked_sha256"]
            gold.write_bytes(expected)
            inp.write_text(f"{len(expected)} {mark} {len(runs)} 2\n"
                           + "".join(f"{byte} {width}\n" for byte, width in runs)
                           + "".join(f"{sec} 1 {m[sec+'_lo']} {m[sec+'_hi']}\n"
                                     for sec in ("quote", "frequent")))
            result = subprocess.check_output([opts.properties.resolve(), inp, gold, str(ordinal)])
            print(result.decode(), end="")


if __name__ == "__main__":
    main()
