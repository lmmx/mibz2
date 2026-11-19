"""Audit every return cylinder to a byte section and each marked spelling on small inputs."""
import json
from collections import Counter
from itertools import product

from inputs import dense_audit, pieces
from section import emit_section
from small import forward_bwt, runs_of


def all_returns(runs, byte, total):
    base = pieces(runs)
    atoms = [(a,b,d,c) for a,b,d,c in base for _ in range(b-a)]
    A = [x for x, v in enumerate(atoms) if v[3] == byte]
    if not A:
        return
    a, b = A[0], A[-1]+1
    answers = {}
    predicted = {}
    for s in A:
        row, lo, hi, word = s, a, b, bytearray()
        while True:
            left, right, d, c = atoms[row]
            lo = max(lo, left+s-row)
            hi = min(hi, right+s-row)
            word.append(c)
            row += d
            if a <= row < b:
                break
            assert len(word) <= len(atoms)
        shift = row-s
        answers[s] = bytes(word), shift
        clipped = max(lo, a-shift), min(hi, b-shift)
        total["strict_exit_clips"] += clipped != (lo, hi)
        predicted[s] = clipped
    # Independently group origins by actual spelling AND exit; require a maximal interval.
    groups = {}
    for s, key in answers.items():
        groups.setdefault(key, []).append(s)
    for s, key in answers.items():
        rows = groups[key]
        assert rows == list(range(*predicted[s]))
    total["anchor_rows"] += len(A)
    total["cylinders"] += len(groups)


def main():
    counts = Counter()
    for sigma, limit in ((2,9),(3,6)):
        for n in range(1,limit+1):
            for t in product(range(sigma), repeat=n):
                runs = runs_of(bytes(t))
                # Zero and the highest symbol exercise both ends of the alphabet.
                for anchor in (0,sigma-1):
                    all_returns(runs, anchor, counts)
                    for p in range(n):
                        _, want, _ = dense_audit(runs,p)
                        out,c = emit_section(runs,p,anchor,audit=True)
                        assert out == want
                        counts["marked_checks"] += 1
                        counts["misses_only"] += c["hits"] == 0
                        counts["partial_hits"] += c["partial_hits"]
                        counts["anchor_present_but_unreached"] += c["section_size"] > 0 and c["queries"] == 0
    # Destination not in A until explicitly intersecting the final range.
    col = b"\x01\x00\x00"
    assert emit_section(runs_of(col),0,0,audit=True)[0] == b"\x00\x00\x01"
    # All-equal, missing byte, repeated text, and capped input records > 127.
    for col,anchor in ((b"x"*300,120),(b"x"*300,42),(b"b"*140+b"a"*140,97)):
        for p in (0,len(col)//2,len(col)-1):
            runs=runs_of(col)
            assert emit_section(runs,p,anchor,audit=True)[0] == dense_audit(runs,p)[1]
            counts["wide_marks"] += 1
    for m in (1,2,3,8,32):
        word = b"".join(b"a"+b"b"*k for k in range(1,m+1))
        col, p = forward_bwt(word)
        out,c = emit_section(runs_of(col),p,97,audit=True)
        assert out == word and c["hits"] == 0 and c["direct_steps"] == len(word)
        counts["distinct_gap_family"] += 1
    print(json.dumps(dict(counts=dict(counts), exit_clip_witness={
        "L":[1,0,0],"primary":0,"anchor":0,"A":[0,2],"source_row":0,
        "piece_before_exit_clip_half_open":[0,2],"certified_origins_half_open":[0,1],
        "false_origin_if_unclipped":1,"false_exit_outside_A":2,
    }), indent=2))


if __name__ == "__main__":
    main()
