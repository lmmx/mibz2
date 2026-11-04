#!/usr/bin/env python3
"""Print a function's instructions executed exactly F times, from a profile and its listing.
    python3 atfreq.py PROFILE LISTING SYMBOL F [F ...]"""
import pathlib, sys
from comb_replay import profile, asm
p = profile(sys.argv[1]); L = asm(pathlib.Path(sys.argv[2])); base = min(L)
counts = p['addresses'][sys.argv[3]]
for f in map(int, sys.argv[4:]):
    print(f'== {sys.argv[3]} at {f}:')
    for a in sorted(L):
        if counts.get(a, {}).get('Ir', 0) == f:
            print(f'   +{a - base:#06x} {L[a]}')
