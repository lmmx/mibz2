"""Run the causal model from canonical records; check post-RLE1 SHA-256 against libbz2."""
import argparse
import csv
import hashlib
from pathlib import Path

from shortcut import capped, reconstruct as split_reconstruct
from whole import reconstruct as whole_reconstruct


def unrle(data):
    # Four equal bytes introduce an unsigned repeat count, including zero.
    out, start = bytearray(), 0
    import re
    for match in re.finditer(rb"(.)\1\1\1(.)", data, flags=re.DOTALL):
        a = match.start()
        out.extend(data[start:a])
        out.extend(match[1] * (4 + match[2][0]))
        start = match.end()
    out.extend(data[start:])
    return out


FIELDS = ("block index n primary capped roots phrases literals copies copy_bytes installs whole splits "
          "new_bounds unit_offers aliases nodes internal_reads max_depth tiles clipped pre_sha256 raw_sha256").split()


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("columns", type=Path)
    ap.add_argument("manifest", type=Path)
    ap.add_argument("checksums", type=Path)
    ap.add_argument("output", type=Path)
    ap.add_argument("--traces", type=Path)
    ap.add_argument("--plain", type=Path, help="directory of independent pinned Plain pre-RLE1 outputs")
    ap.add_argument("--whole", action="store_true")
    args = ap.parse_args()
    fields = FIELDS if not args.whole else [k for k in FIELDS if k not in {
        "whole", "splits", "new_bounds", "unit_offers", "internal_reads", "max_depth"
    }] + ["unit_visits", "wide_visits", "retired", "copies_le16", "max_copy",
           "both_longer_than_one", "certified_extension_area"]
    reconstruct = whole_reconstruct if args.whole else split_reconstruct
    mode = "whole" if args.whole else "shortcut"
    checks = {int(r["index"]): r for r in csv.DictReader(args.checksums.open())}
    if args.traces:
        args.traces.mkdir(exist_ok=True, parents=True)
        from lines import Trace
    with args.output.open("w") as f:
        w = csv.DictWriter(f, fields, lineterminator="\n")
        w.writeheader()
        for entry in csv.DictReader(args.manifest.open()):
            i, j, p = (int(entry[k]) for k in ("block", "index", "primary"))
            column = (args.columns / f"{i:04}.L").read_bytes()
            records = capped(column)
            if args.traces:
                with Trace(args.traces / f"{i:04}-{mode}.bin") as trace:
                    out, stats = reconstruct(records, len(column), p, trace.at, begin_walk=trace.next_phase)
            else:
                out, stats = reconstruct(records, len(column), p)
            if args.plain is not None:
                assert out == (args.plain / f"{i:04}.T").read_bytes()
            raw = unrle(out)
            digest = hashlib.sha256(raw).hexdigest()
            assert digest == checks[j]["raw_sha256"] and len(raw) == int(checks[j]["bytes"])
            stats.update(block=i, index=j, n=len(column), primary=p, capped=len(records))
            row = {k: stats[k] for k in fields}
            row["pre_sha256"] = hashlib.sha256(out).hexdigest()
            row["raw_sha256"] = digest
            w.writerow(row)
            f.flush()


if __name__ == "__main__":
    main()
