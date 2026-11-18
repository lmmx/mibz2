"""Ideal recognition screen. Dense orbit and output here are measuring instruments."""
import argparse
import bisect
import csv
import sys

from inputs import check_frozen, dense_audit, paths, pieces, read_input


def spans(output, policy):
    if isinstance(policy, int):
        for i in range(0, len(output), policy):
            yield i, min(i + policy, len(output))
    else:
        # A quote and a comma are JSON punctuation. 64 bounds a long delimiter-free path.
        delim = policy.encode()
        i = 0
        while i < len(output):
            end = min(i + 64, len(output))
            stop = output.find(delim, i, end)
            j = end if stop < 0 else stop + 1
            yield i, j
            i = j


def screen(output, orbit, policy):
    unique_words, seen = set(), set()
    chunks = walk = first_bytes = hit_bytes = 0
    for i, j in spans(output, policy):
        word = output[i:j]
        key = word, orbit[j] - orbit[i]
        chunks += 1
        if word not in unique_words:
            unique_words.add(word)
            first_bytes += j - i
        if key in seen:
            hit_bytes += j - i
        else:
            seen.add(key)
            walk += j - i
    assert walk + hit_bytes == len(output)
    return chunks, len(unique_words), len(seen), first_bytes, walk, hit_bytes


def census(path):
    runs, p = read_input(path)
    block = int(path.name.split(".")[0])
    col, output, orbit = dense_audit(runs, p)
    check_frozen(block, col, output, p)
    # Input-only test of deterministic next-run transitions. On a full orbit each
    # rank row is used precisely once. The weighted count then equals actual visits.
    entries = pieces(runs)
    stops = [a for a, *_ in entries]
    splits = span = 0
    for a, b, delta, _ in entries:
        after = bisect.bisect_right(stops, a + delta)
        if after < len(stops) and stops[after] < b + delta:
            splits += 1
            span += b - a
    assert orbit.index(p, 1) == len(col)
    for policy in [2, 4, 8, 16, 32, 64, '"', ',']:
        yield [block, len(col), p, len(runs), sum((w+126)//127 for _, w in runs),
               splits, span, str(policy), *screen(output, orbit, policy)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input")
    args = parser.parse_args()
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow("block n p runs capped split_pieces split_visits policy chunks words "
                    "certificates word_first_bytes direct_steps copy_bytes".split())
    for path in paths(args.input):
        writer.writerows(census(path))


if __name__ == "__main__":
    main()
