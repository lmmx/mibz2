"""Prove accounting identities by directly scanning original run/context intersections."""
import argparse
import csv
import hashlib
import itertools
import json
import struct
import sys
from collections import Counter
from itertools import accumulate, groupby
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from clock import Clock


def geometry(runs, end, modal, mark):
    pos = state = middle_modal = nonmodal = synthetic = common = 0
    active = set()
    has_modal = False
    for byte, width in runs:
        stop = pos + width
        while pos < stop:
            hi = min(stop, end[state])
            if byte == modal[state]:
                middle_modal += has_modal
                has_modal = True
                if pos <= mark < hi:
                    synthetic = 1
                    active.add(state)
            else:
                nonmodal += 1
                active.add(state)
            pos = hi
            if hi == end[state]:
                assert has_modal
                common += hi == stop and hi < end[-1]
                state += 1
                has_modal = False
    r, p = len(runs), nonmodal + synthetic
    assert pos == end[-1] and state == len(end)
    assert nonmodal == r - 1 - common - middle_modal
    assert len(active) <= p <= r - common
    return dict(r=r, k_states=len(end), shared=common, repeated_modal=middle_modal,
                nonmodal=nonmodal, synthetic=synthetic, elementary=p, active=len(active))


def describe(column, k):
    clock = Clock(column, k)
    counts = Counter(clock.state)
    end = list(accumulate(counts[c] for c in range(len(clock.modal))))
    runs = [(byte, len(list(g))) for byte, g in groupby(column)]
    return clock, runs, end


def small():
    total = equal = nontrivial = 0
    for alphabet, maximum in ((2, 7), (3, 4)):
        for n in range(1, maximum + 1):
            for column in itertools.product(range(alphabet), repeat=n):
                for k in sorted({0, 1, 2, 3, n+1}):
                    c, runs, end = describe(column, k)
                    for mark in range(n):
                        z = geometry(runs, end, c.modal, mark)
                        tight = z['elementary'] == z['r'] - z['shared']
                        total += 1
                        equal += tight
                        nontrivial += tight and z['elementary'] >= 3 and k > 0
    witnesses = []
    for k, m in itertools.product((0, 1, 4, 16), (2, 3, 8, 19, 48)):
        text = b''.join(b'b' + b'a'*(k+i) for i in range(1, m+1))
        n = len(text)
        rotations = sorted(range(n), key=lambda i: text[i:] + text[:i])
        column = bytes(text[i-1] for i in rotations)
        c, runs, end = describe(column, k)
        events = {x for x in range(n) if column[x] != c.modal[c.state[x]]}
        assert events == {x for x in range(n) if column[x] == ord('b')}
        mark = rotations.index(1)
        assert mark in events and c.lap(mark)[0] == n
        z = geometry(runs, end, c.modal, mark)
        weights = []
        for x in sorted(events):
            y, h = c.lf[x], 1
            while y not in events:
                y, h = c.lf[y], h + 1
            weights.append(h)
        assert sorted(weights) == list(range(k+2, k+m+2))
        assert z['elementary'] == m-1 and z['r'] == 2*m-2 and z['active'] == 1
        witnesses.append(dict(k=k, m=m, n=n, canonical=len(weights), **z))
    return dict(marked_models=total, sharp=equal, positive_depth_sharp_p_at_least_3=nontrivial,
                distinct_weight_witnesses=witnesses)


def real(original, models):
    pinned = {int(row['index']): row for row in
              csv.DictReader((HERE.parents[2]/'contexts'/'inputs.csv').open())}
    for row in csv.DictReader((HERE.parent/'real.csv').open()):
        i, k = int(row['block']), int(row['k'])
        pin = pinned[i]
        raw = (original/pin['tape'].split('.')[0]/'input.txt').read_bytes()
        assert hashlib.sha256(raw).hexdigest() == pin['input_sha256']
        lines = raw.splitlines()
        mark = int(lines[0])
        runs = [tuple(map(int, line.split())) for line in lines[1:]]
        wire = (models/str(i)/f'{k}.ctx').read_bytes()
        assert hashlib.sha256(wire).hexdigest() == row['context_sha256']
        assert wire[:8] == b'CTXMOD01'
        n, p, depth, states, _, _, _ = struct.unpack_from('<7I', wire, 8)
        assert (p, depth, n) == (mark, k, int(row['n']))
        end, _, modal = zip(*struct.iter_unpack('<IIB', wire[36:36+9*states]))
        z = geometry(runs, end, modal, mark)
        assert z['active'] == int(row['active_states'])
        assert z['elementary'] >= int(row['birth_intervals'])
        yield dict(block=i, k=k, **z, canonical=int(row['canonical_return_pieces']))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('paths', type=Path, nargs='*', metavar='original models')
    args = parser.parse_args()
    if args.paths:
        records = list(real(*args.paths))
        output = csv.DictWriter(sys.stdout, records[0], lineterminator='\n')
        output.writeheader()
        output.writerows(records)
    else:
        print(json.dumps(small(), indent=2))
