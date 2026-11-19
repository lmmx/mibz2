"""Exploratory frozen controls, deliberately supplied a dense oracle. No timings."""
import json
import sys
from pathlib import Path

import numpy as np

from early import lf_data


def lz78(sequence):
    table, phrases, i = {}, 0, 0
    while i < len(sequence):
        word = -1
        while i < len(sequence):
            key = word, int(sequence[i])
            i += 1
            if key not in table:
                table[key] = len(table)
                break
            word = table[key]
        phrases += 1
    return dict(phrases=phrases, entries=len(table))


def control(column, primary):
    n = len(column)
    first, lf, _ = lf_data(column)
    labels = np.cumsum(np.r_[True, column[1:] != column[:-1]])-1
    orbit = np.empty(n, np.int32)
    x = primary
    for i in range(n):
        orbit[i], x = x, lf[x]
    assert x == primary and len(np.unique(orbit)) == n
    results = dict(lz78_bytes=lz78(column[orbit]), lz78_run_ids=lz78(labels[orbit]))

    # A single partial certificate per original forward domain. Outside the
    # last certificate, use its immutable original singleton-byte recipe.
    psi = np.empty(n, np.int32)
    psi[lf] = np.arange(n, dtype=np.int32)
    starts = np.r_[0,np.flatnonzero((np.diff(psi)!=1) | (first[1:]!=first[:-1]))+1,n]
    base = [(int(a), int(b), int(psi[a]-a), 1) for a,b in zip(starts, starts[1:])]
    root = np.repeat(np.arange(len(base)), np.diff(starts))
    words = base.copy()
    pos, x, prev = 0,primary,None
    phrases = misses = installs = 0
    while pos < n:
        k = root[x]
        a,b,e,m = words[k]
        if not a <= x < b:
            a,b,e,m = base[k]
            misses += 1
        phrases += 1
        if pos+m >= n:
            break
        if prev is not None:
            z,l,h,d,t = prev
            low,high = max(l,a-d),min(h,b-d)
            if high-low > 1:
                words[z] = (low, high, d+e,t+m)
                installs += 1
        prev = k,a,b,e,m
        x, pos = x+e, pos+m
    results["replace_partial"] = dict(phrases=phrases, installs=installs, misses=misses)

    # Backward split-partition screening with dense row-to-leaf handles.
    # Splits only overwrite the source cell; earlier spellings are carried implicitly.
    starts = np.r_[0,np.flatnonzero(column[1:]!=column[:-1])+1,n]
    cells = [(int(a),int(b),int(lf[a]-a),1) for a,b in zip(starts, starts[1:])]
    root = np.repeat(np.arange(len(cells)),np.diff(starts))
    prev, x, pos = None,primary,0
    phrases=installs=new_bounds=0
    while pos<n:
        k=root[x]
        a,b,e,m=cells[k]
        phrases += 1
        if pos+m>=n:
            break
        invalid=False
        if prev is not None:
            z,l,h,d,t=prev
            low,high=max(l,a-d),min(h,b-d)
            if high-low>1:
                if l<low:
                    root[l:low]=len(cells)
                    cells.append((l,low,d,t))
                    new_bounds+=1
                if high<h:
                    root[high:h]=len(cells)
                    cells.append((high,h,d,t))
                    new_bounds+=1
                cells[z]=(low,high,d+e,t+m)
                installs+=1
                invalid=z==k
        prev = None if invalid else (k,a,b,e,m)
        x,pos=x+e,pos+m
    results["backward_split"] = dict(phrases=phrases,installs=installs,new_bounds=new_bounds)
    return results


if __name__ == "__main__":
    print(json.dumps(control(np.fromfile(Path(sys.argv[1]),np.uint8),int(sys.argv[2])),indent=2))
