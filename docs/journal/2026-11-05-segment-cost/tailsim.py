"""master's room walk's tail on a real T-vector: bytes walked after the queue runs out, and the
rounds by live-lane count, to price the tail with the census fit t = a + lat / min(L, M)."""
import sys, struct, array
def load(p):
    b = open(p, 'rb').read()
    n, orig = struct.unpack('<II', b[:8])
    t = array.array('I'); t.frombytes(b[8:8 + 4 * n]); return n, orig, t
def rr(n, m):
    x = (0x2545f4914f6cdd1d ^ n) & (2**64 - 1); out = []
    for _ in range(m):
        x ^= (x << 13) & (2**64 - 1); x ^= x >> 7; x ^= (x << 17) & (2**64 - 1); out.append(x % n)
    return out
n, orig, t = load(sys.argv[1])
r0 = t[orig] >> 8
for S in [int(s) for s in sys.argv[2:]]:
    st = sorted(set(rr(n, S - 1) + [r0])); ns = len(st); isst = set(st)
    L = 32; q = 0; row = [0]*L; live = [True]*L; busy = 0
    for k in range(L): row[k] = t[st[q]] >> 8; q += 1; busy += 1
    # busy phase: offsets, lanes in order; a lane on a start takes the next (master's order)
    done = False
    while not done:
        marked = []
        for k in range(L):
            r = row[k]
            if r in isst: marked.append(k)
            else: row[k] = t[r] >> 8; busy += 1
        for k in marked:
            if q == ns: live[k] = False; done = True; break
            row[k] = t[st[q]] >> 8; q += 1; busy += 1
    # tail: the lanes still mid-segment (the marked ones after the one that ran out also sit on a start)
    lanes = [k for k in range(L) if live[k] and row[k] not in isst]
    rounds = {}; tailb = 0
    while lanes:
        kk = len(lanes); rounds[kk] = rounds.get(kk, 0) + 1
        nl = []
        for k in lanes:
            r = row[k]
            if r in isst: continue
            row[k] = t[r] >> 8; tailb += 1; nl.append(k)
        lanes = [k for k in nl if row[k] not in isst] + [k for k in nl if row[k] in isst and False]
    tot_rounds = sum(rounds.values())
    print(f'S {S} ns {ns}: busy bytes {busy} tail bytes {tailb} ({100*tailb/n:.2f}%) tail rounds {tot_rounds}')
    for name, a, lat, M in [('9V45 alone', 1.43, 47.1, 32), ('9V74 alone', 2.12, 52.0, 22), ('9V45 4copy', 4.49, 44.6, 22), ('9V74 4copy', 5.68, 54.8, 16)]:
        busy_c = a + lat / min(L, M)
        tail_c = sum(c * max(lat, k * a + 0 * lat) if False else c * (k * a + lat / min(k, M) * k if False else k * (a + lat / min(k, M))) for k, c in rounds.items())
        print(f'   {name}: tail {tail_c/1e3:.0f}K cycles vs {tailb*busy_c/1e3:.0f}K at the busy rate: excess {(tail_c - tailb*busy_c)/1e3:.0f}K cycles a block ({(tail_c - tailb*busy_c)/n:.3f} c/B)')
