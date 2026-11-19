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
    last_split = False
    for a, b, delta, _ in entries:
        after = bisect.bisect_right(stops, a + delta)
        if after < len(stops) and stops[after] < b + delta:
            splits += 1
            span += b - a
            last_split |= a <= orbit[-2] < b
    assert orbit.index(p, 1) == len(col)
    for policy in [2, 4, 8, 16, 32, 64, '"', ',']:
        yield [block, len(col), p, len(runs), sum((w+126)//127 for _, w in runs),
               splits, span, span-int(last_split), str(policy), *screen(output, orbit, policy)]



def section_gate(path):
    """Word-only and joint costs on complete comma-to-comma gaps, with ends separate."""
    runs, p = read_input(path)
    block = int(path.name.split(".")[0])
    col, output, orbit = dense_audit(runs,p)
    check_frozen(block,col,output,p)
    assert orbit.index(p,1)==len(col)
    anchor = 44
    positions = [i for i,c in enumerate(output) if c==anchor]
    a = sum(w for c,w in runs if c<anchor)
    f = sum(w for c,w in runs if c==anchor)
    assert len(positions)==f and f>0
    words, joint = set(), set()
    first_words = first_joint = longest = 0
    for i,j in zip(positions,positions[1:]):
        w = output[i:j]
        key = w,orbit[j]-orbit[i]
        longest = max(longest,j-i)
        if w not in words:
            first_words += j-i
            words.add(w)
        if key not in joint:
            first_joint += j-i
            joint.add(key)
    return [block,len(col),a,f,positions[0],len(col)-positions[-1],f-1,len(words),
            len(joint),first_words,first_joint,longest]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input")
    parser.add_argument("--section",action="store_true")
    args = parser.parse_args()
    writer = csv.writer(sys.stdout, lineterminator="\n")
    if args.section:
        writer.writerow("block n start size prefix tail complete words certificates word_first_bytes "
                        "joint_first_bytes max_complete_length".split())
        for path in paths(args.input):
            writer.writerow(section_gate(path))
        return
    writer.writerow("block n p runs capped split_pieces split_visits needed_split_exits policy chunks words "
                    "certificates word_first_bytes direct_steps copy_bytes".split())
    for path in paths(args.input):
        writer.writerows(census(path))


if __name__ == "__main__":
    main()
