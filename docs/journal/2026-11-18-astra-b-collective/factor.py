"""Reference factorisation checks. Factor from run fragments and an explicit tile reference."""
from bisect import bisect_right
from collections import Counter, defaultdict
from math import gcd


def first_column(bands):
    hist = Counter()
    for _, _, w, c in bands:
        hist[c] += w
    ends, bytes_ = [], []
    end = 0
    for c,w in sorted(hist.items()):
        end += w
        ends.append(end)
        bytes_.append(c)
    return ends,bytes_


def pure_rotation(bands, n, primary):
    shifts = {(y-x) % n for x,y,_,_ in bands}
    if len(shifts) != 1:
        return None
    s, = shifts
    ends, alphabet = first_column(bands)
    if s == 0:
        return bytes([alphabet[bisect_right(ends,primary)]])*n
    if s in (1,n-1):
        # Emit whole monochromatic portions of F, in the required direction.
        x, left = primary, n
        out = bytearray()
        while left:
            i = bisect_right(ends,x)
            take = min(left, x-(ends[i-1] if i else 0)+1 if s == 1 else ends[i]-x)
            out.extend(bytes([alphabet[i]])*take)
            x = (x-take*s)%n
            left -= take
        return bytes(out)
    period = n//gcd(s,n)
    word = bytes(alphabet[bisect_right(ends,(primary-k*s)%n)] for k in range(period))
    return word*(n//period)


def fragments(bands, n, width):
    # Extend to a permutation on padded whole tiles, by fixing the dummy rows.
    dummy = (-n)%width
    spans = list(bands)
    if dummy:
        spans.append((n,n,dummy,0))
    for x,y,w,_ in spans:
        while w:
            take = min(w,width-x%width,width-y%width)
            yield x,y,take
            x += take
            y += take
            w -= take


def choose(bands,n,width):
    k = (n+width-1)//width
    mass = [Counter() for _ in range(k)]
    for x,y,w in fragments(bands,n,width):
        mass[x//width][y//width,(y-x)%width] += w
    perm, shift = [-1]*k, [0]*k
    for i,row in enumerate(mass):
        (j,s),count = row.most_common(1)[0]
        if 2*count > width:
            perm[i],shift[i] = j,s
    used = {j for j in perm if j >= 0}
    free = iter(j for j in range(k) if j not in used)
    for i in range(k):
        if perm[i] < 0:
            j = perm[i] = next(free)
            shift[i] = max(range(width),key=lambda s: mass[i][j,s])
    assert sorted(perm) == list(range(k))
    return perm,shift


def phases(perm,shift,width,exceptions):
    by_tile = defaultdict(list)
    for x in exceptions:
        by_tile[x//width].append(x)
    visited = set()
    succ = {}
    coords = {}
    for start in range(len(perm)):
        if start in visited:
            continue
        order, prefix = [], []
        lanes = defaultdict(list)
        j = start
        total = 0
        while j not in visited:
            i = len(order)
            visited.add(j)
            order.append(j)
            prefix.append(total)
            for x in by_tile[j]:
                lanes[(x%width-total)%width].append((i,x))
            total = (total+shift[j])%width
            j = perm[j]
        assert j == start
        t = len(order)
        g = gcd(total,width)
        laps = width//g
        for r in range(g):
            points = []
            for lap in range(laps):
                z = (r+lap*total)%width
                for i,x in lanes[z]:
                    phase = lap*t+i
                    points.append((phase,x))
                    coords[x] = (order,prefix,total,r,phase)
            if points:
                for idx,(phase,x) in enumerate(points):
                    other,y = points[(idx+1)%len(points)]
                    succ[x] = (y,(other-phase)%(t*laps) or t*laps)
    return succ,coords


def factor_decode(bands,n,mark,width,perm,shift):
    inv = [0]*len(perm)
    for i,j in enumerate(perm):
        inv[j] = i
    exceptional = {}
    for x,y,w in fragments(bands,n,width):
        i,j = x//width,y//width
        if perm[i] == j and (y-x)%width == shift[i]:
            continue
        src = inv[j]
        for k in range(w):
            exceptional[x+k] = src*width+(y+k-shift[src])%width
    exceptional.setdefault(mark,mark)
    assert set(exceptional) == set(exceptional.values())
    succ,coords = phases(perm,shift,width,exceptional)
    ends,alphabet = first_column(bands)
    row = mark
    spelling = bytearray()
    while True:
        u = exceptional[row]
        row, gap = succ[u]
        order,prefix,total,r,phase = coords[u]
        for k in range(1,gap+1):
            lap,i = divmod(phase+k,len(order))
            rank = order[i]*width+(r+lap*total+prefix[i])%width
            assert rank < n
            spelling.append(alphabet[bisect_right(ends,rank)])
        if row == mark:
            break
    forward = spelling[::-1]
    return bytes((forward*(1+(n-1)//len(forward)))[:n]), len(forward)
