"""Emit a marked prefix and build only the first-return cylinders it discovers."""
import argparse
import csv
import sys
from collections import Counter

from cursor import locate, setup
from inputs import check_frozen, dense_audit, digest, paths, read_input
from lines import BASE, HEAD, OUT, Traffic

MASK = (1 << 20) - 1
BIAS = 1 << 20


def emit_section(runs, row, anchor=44, memory=None, audit=False, out=None):
    assert 0 <= anchor <= 255
    at = memory.at if memory else lambda *args: None
    base, directory, (a, b) = setup(runs, at, anchor)
    n = base[-1][0]
    assert 0 <= row < n <= 900_000
    slots = [0] * (b-a)
    if slots:
        at(HEAD, 8*len(slots), True)
    c = Counter(section_start=a, section_size=b-a)
    c["first_copy_dest"] = -1
    if out is None:
        out = bytearray(n)
    assert len(out) == n
    view = memoryview(out)
    q = 0
    # The anchor may be absent from this particular marked cycle.
    while q < n and not a <= row < b:
        _, _, delta, byte = locate(row, base, directory, at)
        at(OUT + q, 1, True)
        out[q] = byte
        q += 1
        row += delta
    c["prefix"] = q
    while q < n:
        assert a <= row < b
        dest = q
        c["queries"] += 1
        at(HEAD + 8*(row-a), 8)
        slot = slots[row-a]
        if slot:
            source, length = slot & MASK, (slot >> 20) & MASK
            take = min(length, n-dest)
            assert length > 0 and source+length <= dest
            at(OUT+source, take)
            at(OUT+dest, take, True)
            view[q:q+take] = view[source:source+take]
            q += take
            if take == length:
                row += (slot >> 40) - BIAS
            else:
                c["partial_hits"] += 1
            if c["hits"] == 0:
                c["first_copy_dest"] = dest
            c["hits"] += 1
            c["copy_bytes"] += take
            continue

        start, lo, hi = row, a, b
        while True:
            idx, begin, delta, byte = locate(row, base, directory, at)
            at(BASE + 8*(idx+1), 4)
            shift = start-row
            lo = max(lo, begin+shift)
            hi = min(hi, base[idx+1][0]+shift)
            at(OUT+q, 1, True)
            out[q] = byte
            q += 1
            row += delta
            if a <= row < b or q == n:
                break
        length = q-dest
        c["walked"] += length
        if q == n:
            break
        delta = row-start
        # The exit must actually be in A for every published starting row.
        lo, hi = max(lo, a-delta), min(hi, b-delta)
        assert a <= lo <= start < hi <= b
        if audit:
            assert not any(slots[lo-a:hi-a])
        slot = source_slot(dest, length, delta)
        at(HEAD + 8*(lo-a), 8*(hi-lo), True)
        for place in range(lo-a, hi-a):
            slots[place] = slot
        c["certificates"] += 1
        c["posted_rows"] += hi-lo
        c["single_row"] += hi-lo == 1
        c["last_publish_dest"] = dest
    c["direct_steps"] = c["prefix"] + c["walked"]
    assert c["posted_rows"] <= b-a
    assert c["direct_steps"] + c["copy_bytes"] == q == n
    return out, c


def source_slot(position, length, delta):
    assert 0 <= position <= MASK and 0 < length <= MASK and -BIAS <= delta < BIAS
    return ((delta+BIAS) << 40) | (length << 20) | position


def unrle(data):
    """The common bzip2 RLE1 pass, for evidence after collecting inverse-stage counts."""
    out = bytearray()
    i, run = 0, 0
    prev = -1
    while i < len(data):
        c = data[i]
        i += 1
        out.append(c)
        run = run+1 if c == prev else 1
        prev = c
        if run == 4:
            assert i < len(data)
            out.extend(bytes([c])*data[i])
            i += 1
            run, prev = 0, -1
    return out


def measure(path):
    block = int(path.name.split(".")[0])
    runs, p = read_input(path)
    mem = Traffic()
    storage = bytearray([0xa5]) * sum(w for _, w in runs)
    out, c = emit_section(runs, p, memory=mem, audit=True, out=storage)
    col, expected, _ = dense_audit(runs, p)
    assert out == expected
    check_frozen(block, col, out, p)
    raw = unrle(out)
    if block == 0:
        assert len(raw) == 900000 and digest(raw) == \
            "d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210"
    keys = ("section_start section_size prefix queries hits partial_hits walked direct_steps "
            "copy_bytes certificates posted_rows single_row first_copy_dest last_publish_dest").split()
    return dict(block=block, n=len(out), p=p, runs=len(runs), **{k:c[k] for k in keys},
                **mem.report(), marked_sha256=digest(out), raw_bytes=len(raw), raw_sha256=digest(raw))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input")
    args = parser.parse_args()
    output = None
    for path in paths(args.input):
        r = measure(path)
        if output is None:
            output = csv.DictWriter(sys.stdout, fieldnames=r, lineterminator="\n")
            output.writeheader()
        output.writerow(r)
        sys.stdout.flush()
