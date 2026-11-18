"""Independent structural and archived-data checks. Python standard library only."""

from collections import Counter, defaultdict
from csv import DictReader, reader
from itertools import accumulate, groupby, product
from pathlib import Path
from statistics import median
import json

ROOT = Path(__file__).resolve().parent


def runs(column):
    return sum(1 for _ in groupby(column))


def lf_tables(last):
    counts = Counter(last)
    base = dict(zip(sorted(counts), accumulate((counts[a] for a in sorted(counts)), initial=0)))
    seen = Counter()
    lf = []
    for a in last:
        lf.append(base[a] + seen[a])
        seen[a] += 1
    return counts, base, lf


def direct_token_bwt(tokens):
    """Sort cyclic rotations by rank doubling. Stable ties use the start index."""
    size = len(tokens)
    rank = list(tokens)
    span = 1
    order = list(range(size))
    while span < size:
        pairs = [(rank[i], rank[(i + span) % size]) for i in range(size)]
        order = sorted(range(size), key=pairs.__getitem__)
        next_rank = [0] * size
        for a, b in zip(order, order[1:]):
            next_rank[b] = next_rank[a] + (pairs[a] != pairs[b])
        rank = next_rank
        if rank[order[-1]] == size - 1:
            break
        span *= 2
    return [tokens[(i - 1) % size] for i in order]


def frozen():
    last = (ROOT / "column.bin").read_bytes()
    inverse = (ROOT / "baseline-rle1.bin").read_bytes()
    marker = 34
    cut = inverse.index(marker) + 1
    parts = (inverse[cut:] + inverse[:cut]).split(bytes([marker]))
    assert parts.pop() == b""
    phrases = [word + bytes([marker]) for word in parts]
    counts = Counter(phrases)
    ids = {word: i for i, word in enumerate(sorted(counts))}
    words = sorted(counts)
    direct = direct_token_bwt([ids[word] for word in phrases])

    # This metadata comes from the reference inverse, before reading the column.
    sizes = Counter()
    choices = defaultdict(set)
    for word, frequency in counts.items():
        for i in range(len(word)):
            suffix = word[i:]
            sizes[suffix] += frequency
            choices[suffix].add(word[i - 1] if i else marker)
    offsets = dict(zip(sorted(sizes), accumulate((sizes[s] for s in sorted(sizes)), initial=0)))
    assert sum(sizes.values()) == len(last)

    # Bottom-up stable deinterleave. Only mixed buckets read the byte column.
    reads = [0, 0, 0]
    def recover(suffix):
        size = sizes[suffix]
        outgoing = choices[suffix]
        if len(outgoing) == 1:
            a = next(iter(outgoing))
            return [ids[suffix]] * size if a == marker else recover(bytes([a]) + suffix)
        start = offsets[suffix]
        routing = last[start:start + size]
        reads[0] += 1
        reads[1] += runs(routing)
        reads[2] += size
        streams = {
            a: iter(recover(bytes([a]) + suffix)) for a in outgoing if a != marker
        }
        ordered = [ids[suffix] if a == marker else next(streams[a]) for a in routing]
        assert all(next(stream, None) is None for stream in streams.values())
        return ordered

    recovered = recover(bytes([marker]))
    assert recovered == direct

    # Each token contributes its preceding letter to each suffix it visits.
    output = bytearray(len(last))
    tails = offsets.copy()
    for token in recovered:
        word = words[token]
        for i in range(len(word)):
            suffix = word[i:]
            output[tails[suffix]] = word[i - 1] if i else marker
            tails[suffix] += 1
    assert output == last
    assert decode_rle1(inverse) == ("ok", (ROOT / "baseline-output.bin").read_bytes())
    return dict(rows=len(last), types=len(counts), tokens=len(phrases),
                token_runs=runs(direct), suffixes=len(sizes), mixed=reads,
                byte_runs=runs(last))


def representative_edge(last, x, marker, base):
    # Emulate the first usable marker-run lookup in both Rust rank-edge variants.
    for y in range(x, len(last)):
        if last[y] == marker:
            return base + last[:y].count(marker)
    return base + last.count(marker)


def check_columns():
    tested = tight = 0
    for n in range(1, 9):
        for last in product(range(3), repeat=n):
            counts, base, lf = lf_tables(last)
            first = sorted(last)
            psi = [0] * n
            for i, target in enumerate(lf):
                psi[target] = i
            for c, m in counts.items():
                b = base[c]
                end = b + m
                letters = []
                for source in range(b, end):
                    word = []
                    row = source
                    while True:
                        a = last[row]
                        row = lf[row]
                        if a == c:
                            break
                        word.append(a)
                    letters.append(tuple(reversed(word)) + (c,))

                endings = set()
                for i in range(m - 1):
                    if letters[i] == letters[i + 1]:
                        continue
                    row = b + i
                    while last[row] == last[row + 1]:
                        assert last[row] != c
                        assert lf[row + 1] == lf[row] + 1
                        row = lf[row]
                    assert row not in endings
                    endings.add(row)
                assert len(endings) == runs(letters) - 1
                assert runs(letters) <= runs(last)
                tight += runs(letters) == runs(last)

                a, z = b, end
                for _ in range(3):
                    a = representative_edge(last, a, c, b)
                    z = representative_edge(last, z, c, b)
                actual = set()
                for row in range(n):
                    x = row
                    for _ in range(4):
                        if first[x] != c:
                            break
                        x = psi[x]
                    else:
                        actual.add(row)
                assert actual == set(range(a, z))
                tested += 1
    return dict(column_marker_pairs=tested, tight_run_bound=tight)


def decode_rle1(data):
    """Byte-state transducer with output accrued even if terminal state errors."""
    prev = None
    repeat = 0
    out = bytearray()
    for byte in data:
        if repeat == 4:
            out.extend(bytes([prev]) * byte)
            prev, repeat = None, 0
        else:
            repeat = repeat + 1 if prev == byte else 1
            prev = byte
            out.append(byte)
    return ("error" if repeat == 4 else "ok"), bytes(out)


def check_cuts():
    tested = errors = 0
    for n in range(1, 9):
        for period in product(range(3), repeat=n):
            if 0 not in period:
                continue
            cyc = bytes(period) * (2 * n + 4)
            if b"\0" * 4 in cyc:
                continue
            for length in range(1, 2 * n + 4):
                data = cyc[:length]
                parts = data.split(b"\0")
                chunks = [part + b"\0" for part in parts[:-1]] + parts[-1:]
                partial = [decode_rle1(chunk) for chunk in chunks]
                assert all(status == "ok" for status, _ in partial[:-1])
                result = decode_rle1(data)
                assert result == (partial[-1][0], b"".join(out for _, out in partial))
                errors += result[0] == "error"
                tested += 1
    return dict(linear_slice_cases=tested, terminal_unfinished_cases=errors)


def measurements():
    evidence = ROOT / "evidence"
    with (evidence / "grouped.csv").open() as handle:
        blocks = list(DictReader(handle))
    large = {row["block"] for row in blocks if int(row["n"]) >= 65536}
    with (evidence / "allocations.csv").open() as handle:
        allocations = list(DictReader(handle))
    with (evidence / "single-core.csv").open() as handle:
        rows = list(reader(handle))
    assert len(rows) == 29 and rows[0][0] == "cells"
    measurements = defaultdict(list)
    ratios = []
    for start in range(1, len(rows), 4):
        batch = rows[start:start + 4]
        assert {r[0] for r in batch} == {"cells", "lazy", "grouped", "fused2"}
        assert {int(r[2]) for r in batch} == {209700196}
        by_arm = {r[0]: float(r[1]) for r in batch}
        for name, seconds in by_arm.items():
            measurements[name].append(seconds)
        ratios.append(by_arm["grouped"] / by_arm["cells"])
    out = {}
    for name, seconds in measurements.items():
        all_blocks = [r for r in allocations if r["decoder"] == name]
        assert len(all_blocks) == 431
        assert {int(r["block"]) for r in all_blocks} == set(range(431))
        assert sum(int(r["returned_length"]) for r in all_blocks) == 209700196
        per_large = [r for r in all_blocks if r["block"] in large]
        assert len(per_large) == 233
        out[name] = dict(median_s=median(seconds),
                        peak_requested_bytes=median(int(r["additional_peak_requested_bytes"]) for r in per_large),
                        admitted=sum(int(r["fused"]) for r in all_blocks),
                        admitted_large=sum(int(r["fused"]) for r in per_large))
    return dict(blocks=431, dense_grouped_stats_blocks=len(blocks), large=len(large),
                arms=out, grouped_over_cells=ratios,
                paired_median=median(ratios))


if __name__ == "__main__":
    print(json.dumps(dict(frozen=frozen(), exhaustive=check_columns(),
                          cuts=check_cuts(), measurements=measurements()), indent=2))
