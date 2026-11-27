"""Quote-delimited dictionary edits on the large Wikidata blocks, measured on oracle bytes."""

from collections import Counter
from pathlib import Path
from struct import unpack
import csv
import re
import sys

from verify_phrase_bwt import rle1


def rows(source):
    """Count phrase payloads changed by an independent RLE1 pass, per input block."""
    four = re.compile(rb"(.)\1{3}", re.DOTALL)
    while header := source.read(8):
        i, n = unpack("<II", header)
        raw = source.read(n)
        parts = raw.split(b'"')
        words = Counter(s + b'"' for s in parts[1:-1])
        words[parts[-1] + parts[0] + b'"'] += 1
        changed = [w for w in words if four.search(w) and rle1(w) != w]
        yield [
            i,
            n,
            len(words),
            sum(words.values()),
            len(changed),
            sum(words[w] for w in changed),
            sum(len(w) for w in changed),
            sum(len(w) * words[w] for w in changed),
        ]


if __name__ == "__main__":
    wr = csv.writer(sys.stdout)
    wr.writerow(
        "block,rows,types,occurrences,changed_types,changed_occurrences,changed_dict_input_bytes,changed_occurrence_input_bytes".split(
            ","
        )
    )
    with Path(sys.argv[1]).open("rb") as f:
        wr.writerows(rows(f))
