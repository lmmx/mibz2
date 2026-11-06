#!/usr/bin/env python3
"""Integer/static-listing model only; never builds or runs compact30 code."""
import argparse
import hashlib
import json
import gzip
from pathlib import Path
import re

MAX_N = 983010
EPOCH = 61440
SCAN = Path('docs/journal/2026-11-04-compact30-scanner-prototype')


def phi(i):
    return 32 * (i // 30) + i % 30


def instructions(path, lo, hi):
    found = []
    for line in path.read_text().splitlines():
        m = re.match(r'\s*([0-9a-f]+):\s+(.*)', line)
        if m and lo <= int(m[1], 16) <= hi:
            found.append({'address': m[1], 'instruction': m[2]})
    return found


def archive_counts(path, wanted_function):
    """Read existing Ir profile; call-associated costs are not self instructions."""
    lines = gzip.open(path, 'rt').read().splitlines()
    names = {}
    for line in lines:
        m = re.match(r'(?:c?fn)=\((\d+)\) (.+)', line)
        if m:
            names[m[1]] = m[2]
    current = None
    positions = []
    previous = []
    result = {}
    pending_call = False
    for line in lines:
        if line.startswith('positions:'):
            positions = line.split()[1:]
            previous = [0] * len(positions)
        elif line.startswith('events:'):
            assert line.split()[1:] == ['Ir']
        elif line.startswith('fn='):
            m = re.match(r'fn=\((\d+)\)', line)
            current = names[m[1]] if m else line[3:]
        elif line.startswith('calls='):
            pending_call = True
        elif line and (line[0].isdigit() or line[0] in '+-*'):
            fields = line.split()
            for j, value in enumerate(fields[:len(positions)]):
                if value != '*':
                    previous[j] = previous[j] + int(value, 0) if value[0] in '+-' else int(value, 0)
            if current == wanted_function and not pending_call:
                address = previous[positions.index('instr')]
                result[address] = result.get(address, 0) + int(fields[len(positions)])
            pending_call = False
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo', type=Path, required=True)
    args = parser.parse_args()
    root = args.repo
    mapping_count = 0
    for i in range(MAX_N + 1):
        y = phi(i)
        assert y & 31 == i % 30
        assert (y & 65535) & 31 == i % 30
        assert y < 1 << 20
        mapping_count += 1

    # Algebra depends on p and k only; multiple high-word/epoch boundary samples
    # verify the finite model and avoid claiming testing itself is the proof.
    groups = [0, 1, 2047, 2048, 4095, 4096, 16383, 16384, 32766, 32767]
    transitions = labels = bulk = 0
    for g in groups:
        for p in range(30):
            i = 30 * g + p
            if i > MAX_N:
                continue
            y = phi(i)
            for k in range(31):
                if i + k > MAX_N:
                    continue
                y1 = y + k + 2 * (p + k >= 30)
                assert y1 == phi(i + k)
                assert y1 & 31 == (p + k) % 30
                transitions += 1
            for length in range(17):
                if i + length > MAX_N:
                    continue
                n1 = min(length, 30 - (y & 31))
                for t in range(length):
                    u = (y + t + 2 * (t >= n1)) & 65535
                    assert u == phi(i + t) & 65535
                    labels += 1
            for whole in (0, 1, 2, 2048):
                if i + 30 * whole <= MAX_N:
                    assert phi(i + 30 * whole) == y + 32 * whole
                    assert (y + 32 * whole) & 31 == p
                    bulk += 1

    reachable = sorted({(16 * k) % 30 for k in range(15)})
    assert reachable == list(range(0, 30, 2))
    assert EPOCH % 16 == EPOCH % 30 == 0
    # Counterexamples to invalid replacements/extended preconditions.
    counterexamples = {
        'mod_30_is_wrong': {'i': 30, 'y': phi(30), 'y_mod_30': phi(30) % 30, 'p': 0},
        'arbitrary_y_is_not_a_valid_phase': {'y': 30, 'y_and_31': 30},
        'one_crossing_update_requires_k_le_30': {'p': 29, 'k': 31,
            'single_crossing_y': 29 + 31 + 2, 'correct_y': phi(60)},
        'full_tile_equality_boundary': {'p': 14, 'k': 16, 'crossing': True},
    }
    paths = [SCAN / 'listings/ctor2s-build-false.asm',
             SCAN / 'listings/v2-build-false-same-binary.asm', SCAN / 'corpora.jsonl',
             Path('examples/c30walk/ctor2s.rs')]
    pins = {}
    for path in paths:
        data = (root / path).read_bytes()
        pins[str(path)] = {'sha256': hashlib.sha256(data).hexdigest(),
            'git_blob': hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()}
    scanner = instructions(root / paths[0], 0x41a10, 0x41a42)
    b2 = instructions(root / paths[1], 0x3b1d0, 0x3b200)
    assert len(scanner) == 12 and len(b2) == 11
    rows = [json.loads(line) for line in (root / paths[2]).read_text().splitlines()]
    selected = [r for r in rows if r['input'] == 'wikidata-20261026.bz2'
                and r['permille'] == 0 and not r['extra_starts']]
    assert len(selected) == 1
    s = selected[0]['tiles_nonuniform']
    assert s == 2393313
    profile = SCAN / 'callgrind/per-instruction.callgrind.gz'
    counts = archive_counts(root / profile, '<c30walk::ctor2s::Ctor2s>::build::<false>')
    assert counts[0x41a10] == s
    refill_count = counts[0x41bb3]
    assert refill_count == counts[0x41be8] == 996634
    for room in list(range(0, 100)) + [(1 << 64) - 1, (1 << 64) - 2]:
        assert (room < 16) == (min(16, room) < 16)
    data = (root / profile).read_bytes()
    pins[str(profile)] = {'sha256': hashlib.sha256(data).hexdigest(),
        'git_blob': hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()}
    result = {
        'kind': 'abstract-equivalence-and-conditional-target, not constructor execution',
        'pin': '7f74e9f30e31af33b0d66fec3fe89af1758307d9',
        'mapping_cases': mapping_count, 'advance_cases': transitions,
        'temporary_label_cases': labels, 'bulk_cases': bulk,
        'full_tile_reachable_phases': reachable, 'counterexamples': counterexamples,
        'pins': pins, 'scanner_advance': scanner, 'b2_advance': b2,
        'S': s, 'conditional_caps': {'new_advance_max_instructions': 8,
            'added_tile_setup_max_instructions': 1,
            'scanner_scoped_credit_minus_H': 3 * s,
            'B2_scoped_credit_minus_H': 2 * s,
            'H': 'all other whole-constructor changes; unresolved; not zero'},
        'achieved_net_credit': None,
        'discriminator_reuse': {'F_first_segment_refills': refill_count,
            'conditional_exposed_credit_minus_H': 2 * refill_count,
            'store_address': '0x41bb3', 'reload_address': '0x41be8',
            'predicate_address': '0x41c1b', 'extra_range_assumption': False},
    }
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
