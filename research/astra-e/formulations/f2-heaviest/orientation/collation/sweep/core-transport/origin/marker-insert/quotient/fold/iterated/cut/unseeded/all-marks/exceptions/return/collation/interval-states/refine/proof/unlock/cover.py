"""Check the stable thick cover used for a fixed unit charge on true periodic texts."""
from collections import Counter
from itertools import combinations
import json


def check(k):
    g = k
    small = '1011'+'0'*(3*k-4)
    psi = sorted(range(len(small)),key=small.__getitem__)
    orbit = [0]
    while psi[orbit[-1]] != 0:
        orbit.append(psi[orbit[-1]])
    # A chosen rotation of the primitive cyclic text. Re-sort the rotations of its power.
    cyclic = ''.join(sorted(small)[rank] for rank in orbit)
    periodic = cyclic*g
    thick = ''.join(byte*g for byte in small)
    if k <= 8:
        rotations = sorted(periodic[x:]+periodic[:x] for x in range(len(periodic)))
        assert ''.join(row[-1] for row in rotations) == thick
    lift = sorted(range(len(thick)),key=thick.__getitem__)
    old = (len(small)-3)*g
    for t in range(1,len(thick)):
        if t==old:
            continue
        lo,hi = lift[t-1]+1,lift[t]
        z,j = divmod(t,g)
        if j:
            assert lo == hi == g*psi[z]+j
        else:
            aligned = range((lo+g-1)//g,hi//g+1)
            assert list(aligned) == list(range(psi[z-1]+1,psi[z]+1))
    chosen = [g*t for t in range(1,len(small),3)]
    def mass(seams):
        bounds = [0,*sorted([old,*seams]),len(thick)]
        return sum(max(Counter(thick[l:r]).values()) for l,r in zip(bounds,bounds[1:]))
    assert mass(chosen)-mass([])-len(chosen)==k


if __name__ == '__main__':
    for k in range(3,65):check(k)
    print(json.dumps({'all_supports_verified_k':[3,64],
                      'independently_sorted_cyclic_power_k':[3,8]},indent=2))
