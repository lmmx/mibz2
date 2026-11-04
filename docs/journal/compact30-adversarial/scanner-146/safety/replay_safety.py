#!/usr/bin/env python3
"""Static evidence replay and finite scanner model. Never builds or runs a constructor."""
import argparse
import hashlib
import itertools
import json
import re
from pathlib import Path


def blob_sha(p):
    b = p.read_bytes()
    return hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()


def scan(tokens, tail, epoch_tiles):
    # 0/1 are uniform full tiles, 2/3 are nonuniform full tiles.  A token carries
    # only the observations consumed by scanner control flow, not output fields.
    out = []
    A = J = H = Q = S = R = 0
    for epoch_start in range(0, len(tokens), epoch_tiles):
        end = min(len(tokens), epoch_start + epoch_tiles)
        start, held = epoch_start, None
        while start < end:
            if held is None:
                a = tokens[start]
                A += 1
            else:
                a = held
                assert a == tokens[start]
                held = None
                H += 1
            if a < 2:
                j = start + 1
                while j < end:
                    a2 = tokens[j]
                    J += 1
                    if a2 == a:
                        j += 1
                    else:
                        held = a2
                        break
                Q += held is None
                out.append(('run', start, j, a))
                R += 1
                start = j
            else:
                out.append(('tile', start, start + 1, a))
                S += 1
                start += 1
    assert A + J == len(tokens)
    assert R == H + Q
    assert A == S + Q
    expanded = [k for _, lo, hi, _ in out for k in range(lo, hi)]
    assert expanded == list(range(len(tokens)))
    assert 0 <= tail <= 15
    # Unread suffix identity gives i=end-rest.len() and i+len<=end.  Model tail
    # rows independently; the actual epoch length is divisible by 16.
    tail_start = len(tokens) * 16
    for p in range(30):
        n1 = min(tail, 30-p)
        assert 0 <= n1 <= tail
        assert tail_start + n1 <= tail_start + tail
    return out


def reference(tokens, epoch_tiles):
    out = []
    for begin in range(0, len(tokens), epoch_tiles):
        epoch = tokens[begin:begin+epoch_tiles]
        # Partition by equal tokens first, then split nonuniform blocks into tiles.
        cursor = begin
        for token, group in itertools.groupby(epoch):
            count = sum(1 for _ in group)
            if token < 2:
                out.append(('run', cursor, cursor+count, token))
            else:
                out.extend(('tile', i, i+1, token) for i in range(cursor, cursor+count))
            cursor += count
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('evidence', type=Path)
    args = ap.parse_args()
    d = args.evidence / 'docs/journal/2026-11-04-compact30-scanner-prototype'
    miri = {}
    rx = re.compile(r'--miri (\w+): (\d+) cases, (\d+) array comparisons, (\d+) walk outputs, (\d+) restorations: every check exact;')
    for p in sorted((d/'miri').glob('miri-*.console')):
        text = p.read_text()
        m = rx.search(text)
        assert m and text.rstrip().endswith('exit 0')
        assert not re.search(r'undefined behavior|error: Undefined|error: memory access', text, re.I)
        miri[m[1]] = dict(zip(('cases','arrays','walks','restorations'), map(int,m.groups()[1:])))
    assert set(miri) == {'tiny','tails','align','phase','held','endrun','epoch','malformed'}
    sums = {k:sum(x[k] for x in miri.values()) for k in ('cases','arrays','walks','restorations')}
    jsonl = {}
    for name in ['corpora','adversarial']:
        rows = [json.loads(s) for s in (d/(name+'.jsonl')).read_text().splitlines()]
        for x in rows:
            assert x['array_checks'] == 2*x['blocks']
            assert x['walk_checks'] == 3*x['blocks']
            assert x['restoration_checks'] == 2*x['blocks']
            assert x['fresh_pairs'] + x['lookahead_pairs'] == (x['rows']-x['tail_rows'])//16
            assert x['runs'] == x['held_reuses'] + x['runs_no_full_tile']
            assert x['fresh_pairs'] == x['tiles_nonuniform'] + x['runs_no_full_tile']
            assert x['rows_scalar'] == 16*x['tiles_nonuniform'] + x['tail_rows']
            assert x['rows'] == x['rows_bulk']+x['rows_bulk_raw']+x['rows_fragment']+x['rows_scalar']
        jsonl[name] = {'records':len(rows), **{k:sum(x[k] for x in rows) for k in ('blocks','array_checks','walk_checks','restoration_checks')}}
    for mutation in ('M1','M2'):
        text=(d/f'mutation-{mutation}.console').read_text()
        assert 'the counted build rejected valid counts' in text and text.rstrip().endswith('exit 101')
    cases=0
    for n in range(7):
        for tokens in itertools.product(range(4), repeat=n):
            for ep in (1,2,3,7,3840):
                expected=reference(tokens,ep)
                for tail in range(16):
                    assert scan(tokens,tail,ep) == expected
                    cases+=1
    advance_checks=0
    for p in range(30):
        for k in range(31):
            p1=p+k
            actual=p1-30 if p1>=30 else p1
            assert actual == (p+k)%30 and 0<=actual<30
            advance_checks+=1
    slices=0
    for p in range(30):
        n1=min(16,30-p)
        assert 0<n1<=16 and 0<=16-n1<16
        for i in (0,1,61424,61440,983010-16):
            assert i<=i+n1<=i+16<=983010
            slices+=1
    # Hypothesis counterexamples, not observations about current source:
    assert (29+31)-30 == 30  # Dropping k<=30 invalidates advance induction.
    assert min(16,30-0)>1   # Full-tile U1 applied to one-byte tail overreads.
    assert 0 <= 0 + 17 <= 32  # n1=17 does not violate U1 on an interior tile.
    assert 17 > 16  # The companion U2 range is reversed instead.
    assert 16 + 14 <= 32 < 16 + 17  # Isolated U2 final-tile negative control.
    source=args.evidence/'examples/c30walk/ctor2s.rs'
    report={'source_pin':'7f74e9f30e31af33b0d66fec3fe89af1758307d9',
            'source_blob_sha1':blob_sha(source),'miri':miri,'miri_totals':sums,
            'jsonl':jsonl,'abstract_scanner_cases':cases,
            'advance_domain_checks':advance_checks,'full_tile_range_checks':slices,
            'counterexamples':{'advance_without_k_bound':{'p':29,'k':31,'result':30},
                               'full_tile_contract_on_tail':{'i':0,'len':1,'p':0,'n1':16},
                               'n1_17_not_U1_oob':{'slice_len':32,'i':0,'n1':17,'first_range':[0,17],'valid':True,'second_range':[17,16],'second_valid':False},
                               'isolated_U2_negative_control':{'slice_len':32,'i':16,'p':16,'n1':14,'first_range':[16,30],'second_mutant_range':[30,33]}},
            'limits':'Finite abstract scanner/proof-domain model plus archived-log consistency; not a constructor implementation, new Miri run, UB proof, compiler result or timing.'}
    print(json.dumps(report,indent=2,sort_keys=True))

if __name__=='__main__': main()
