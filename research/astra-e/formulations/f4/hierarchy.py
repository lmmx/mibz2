"""A full marked solver using successive ascent-token BWTs and a dense final table."""
from array import array
from bisect import bisect_right
from itertools import accumulate
from phrases import spelling, transform


def decode(runs, p, stop=32_768, max_levels=8):
    initial = sum(w for _, w in runs)
    words = [bytes([c]) for c in range(256)]
    delta = 0
    n = initial
    layers = []
    for _ in range(max_levels):
        if n <= stop:
            break
        nxt, rank, skip, parents, dictionary, stats = transform(runs, p, True)
        freqs = [0] * len(dictionary)
        for token, width in nxt:
            freqs[token] += width
        # The carrier token at new primary rank contains the old primary symbol.
        # Transport its position by the number of original bytes before that symbol.
        carrier = bisect_right(list(accumulate(freqs)), rank)
        names = [spelling(node, parents) for node in dictionary]
        delta += sum(len(words[c]) for c in names[carrier][:skip])
        new_words = [b"".join(words[c] for c in digits) for digits in names]
        stats.update(input_rows=n, dictionary_digits=sum(map(len, names)),
                     dictionary_bytes=sum(map(len, new_words)), carried_byte_offset=delta)
        layers.append(stats)
        words, runs, p = new_words, nxt, rank
        if stats["token_rows"] == n:
            break
        n = stats["token_rows"]

    count = [0] * len(words)
    for token, width in runs:
        count[token] += width
    cursor = [0] + list(accumulate(count))[:-1]
    bits = max(1, (n - 1).bit_length())
    mask = (1 << bits) - 1
    fmt = "I" if (len(words)-1).bit_length() + bits <= 32 else "Q"
    rows = array(fmt)
    for token, width in runs:
        pos = cursor[token]
        payload = token << bits
        rows.extend(payload | j for j in range(pos, pos + width))
        cursor[token] += width
    assert len(rows) == n
    backward = []
    row = p
    while True:
        cell = rows[row]
        backward.append(cell >> bits)
        row = cell & mask
        if row == p:
            break
    period = b"".join(words[token] for token in reversed(backward))
    assert 0 <= delta < len(period)
    marked = period[delta:] + period[:delta]
    output = (marked * ((initial + len(marked) - 1) // len(marked)))[:initial]
    return output, layers, (n, len(rows) * rows.itemsize, len(backward), len(period))


if __name__ == "__main__":
    import argparse
    import csv
    import hashlib
    import itertools
    import json
    import sys
    from pathlib import Path
    from phrases import compress, inverse

    cli = argparse.ArgumentParser()
    cli.add_argument("input", help="archive directory, individual archive, or small")
    args = cli.parse_args()
    if args.input == "small":
        total = levels = 0
        for alphabet, limit in [(2,7), (3,6), (4,4)]:
            for n in range(1,limit+1):
                for col in itertools.product(range(alphabet), repeat=n):
                    for p in range(n):
                        out, layers, _ = decode(compress(col), p, stop=2, max_levels=5)
                        assert list(out) == inverse(col, p)
                        total += 1
                        levels += len(layers)
        print(json.dumps({"marked_checks":total,"reduction_layers":levels}))
        sys.exit()

    root = Path(__file__).resolve().parents[4]
    study = root / "docs/journal/2026-11-18-astra-c-output-first"
    sys.path.insert(0, str(study))
    from inputs import paths, read_input
    archive = {int(x["block"]):x for x in csv.DictReader((study / "section.csv").open())}
    fields = "input_rows input_runs overlays prefix_pieces token_pieces token_rows token_runs trie_nodes alphabet dictionary_digits dictionary_bytes carried_byte_offset".split()
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(["block","layer",*fields,"final_rows","final_table_bytes","final_marked_links","expanded_period","marked_sha256"])
    for path in paths(args.input):
        index = int(path.name.split(".")[0])
        out, layers, tail = decode(*read_input(path))
        sha = hashlib.sha256(out).hexdigest()
        assert sha == archive[index]["marked_sha256"]
        for i,s in enumerate(layers):
            writer.writerow([index,i+1,*[s[k] for k in fields],*tail,sha])
        sys.stdout.flush()
