"""Separate offline oracle and input identities; never feeds paths to the candidate."""

import argparse
import bz2
import hashlib
import json
import sys
from array import array
from collections import Counter
from pathlib import Path


def sha(data):
    """Return a content identity."""
    return hashlib.sha256(data).hexdigest()


def main():
    """Check actual exported seam against independent LF and libbzip2 output."""
    parser = argparse.ArgumentParser()
    parser.add_argument("export", type=Path)
    parser.add_argument("corpus", type=Path)
    args = parser.parse_args()
    meta = json.loads((args.export / "native.json").read_text())
    column = (args.export / "column.bin").read_bytes()
    baseline = (args.export / "baseline-rle1.bin").read_bytes()
    n, primary = meta["n"], meta["primary"]
    assert len(column) == len(baseline) == n
    forward, inverse = array("I", [0]) * n, array("I", [0]) * n
    counts, cursor, total = Counter(column), {}, 0
    for byte in sorted(counts):
        cursor[byte] = total
        total += counts[byte]
    for x, byte in enumerate(column):
        y = cursor[byte]
        cursor[byte] += 1
        forward[x], inverse[y] = y, x
    seen, cycles, selected = bytearray(n), [], None
    for start in range(n):
        if seen[start]:
            continue
        x, length, contains_primary = start, 0, False
        while not seen[x]:
            seen[x] = 1
            contains_primary |= x == primary
            length += 1
            x = forward[x]
        assert x == start
        cycles.append(length)
        if contains_primary:
            selected = length
    decoded, x = bytearray(), primary
    for _ in range(n):
        x = inverse[x]
        decoded.append(column[x])
    assert decoded == baseline
    corpus = args.corpus.read_bytes()
    assert len(corpus) == 11854933
    assert sha(corpus) == "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
    needed = meta["preceding_output_bytes"] + meta["output_bytes"]
    prefix, decoder, data = bytearray(), bz2.BZ2Decompressor(), corpus
    while len(prefix) < needed:
        prefix.extend(decoder.decompress(data, max_length=needed - len(prefix)))
        data = b""
        if decoder.eof:
            data = decoder.unused_data
            decoder = bz2.BZ2Decompressor()
        else:
            assert not decoder.needs_input or len(prefix) == needed
    expected_output = (args.export / "baseline-output.bin").read_bytes()
    assert prefix[meta["preceding_output_bytes"] :] == expected_output
    meta.update(
        baseline_revision="57f51bbc0ca62ac3ee1cc148a1010842eea6222a",
        prototype_revision="af3327d8aee9fffe26d2d6de76818b6ac69cc145",
        corpus_revision="a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a",
        corpus_sha256=sha(corpus),
        corpus_bytes=len(corpus),
        delimiter=34,
        artifacts={
            name: {
                "bytes": len((args.export / name).read_bytes()),
                "sha256": sha((args.export / name).read_bytes()),
            }
            for name in ("runs.u16le", "column.bin", "baseline-rle1.bin", "baseline-output.bin")
        },
        oracle={
            "method": "independent row LF + Python/libbzip2 prefix",
            "cycles": len(cycles),
            "selected_cycle_length": selected,
            "min_cycle_length": min(cycles),
            "max_cycle_length": max(cycles),
            "row_lf_writes": n,
            "inverse_writes": n,
            "cycle_visits": n,
            "inverse_output_visits": n,
            "column_lf_inverse_seen_decoded_baseline_bytes": sum(
                sys.getsizeof(x) for x in (column, forward, inverse, seen, decoded, baseline)
            ),
            "libbzip2_prefix_bytes": len(prefix),
            "compressed_input_bytes": len(corpus),
            "native_baseline_private_scratch_peak": "not measured",
            "candidate_received_oracle_tables_or_words": False,
        },
    )
    (args.export / "identities.json").write_text(json.dumps(meta, indent=2, sort_keys=True) + "\n")
    print(json.dumps(meta, sort_keys=True))


if __name__ == "__main__":
    main()
