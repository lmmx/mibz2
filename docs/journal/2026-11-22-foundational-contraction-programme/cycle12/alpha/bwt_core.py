"""Self-contained exact BWT & Run-Overlap Graph core helpers for Cycle 9+ scripts."""

def bwt_encode(T: bytes):
    n = len(T)
    rots = sorted(range(n), key=lambda i: T[i:] + T[:i])
    L = bytes(T[(i - 1) % n] for i in rots)
    p = rots.index(0)
    return L, p


def ref_ibwt(L: bytes, p: int) -> bytes:
    n = len(L)
    if n == 0:
        return b""
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = cur[b]
        cur[b] += 1
    out = bytearray(n)
    x = p
    for k in range(n - 1, -1, -1):
        out[k] = L[x]
        x = LF[x]
    return bytes(out)


def combine_mark(m1: int, len1: int, m2: int) -> int:
    if m1 >= 0:
        return m1
    if m2 >= 0:
        return len1 + m2
    return -1


class WordStore:
    __slots__ = ("kind", "a", "b", "length", "inline_bytes", "slp_gt16")

    def __init__(self):
        self.kind = [0] * 256
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256
        self.inline_bytes = [bytes([c]) for c in range(256)]
        self.slp_gt16 = 0

    def concat(self, u: int, v: int) -> int:
        lu, lv = self.length[u], self.length[v]
        if lu + lv <= 16 and self.kind[u] == 0 and self.kind[v] == 0:
            idx = len(self.kind)
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.length.append(lu + lv)
            self.inline_bytes.append(self.inline_bytes[u] + self.inline_bytes[v])
            return idx
        idx = len(self.kind)
        self.kind.append(1); self.a.append(u); self.b.append(v)
        self.length.append(lu + lv)
        self.inline_bytes.append(b"")
        self.slp_gt16 += 1
        return idx

    def power(self, u: int, k: int) -> int:
        if k == 1:
            return u
        lu = self.length[u]
        if lu * k <= 16 and self.kind[u] == 0:
            idx = len(self.kind)
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.length.append(lu * k)
            self.inline_bytes.append(self.inline_bytes[u] * k)
            return idx
        idx = len(self.kind)
        self.kind.append(2); self.a.append(u); self.b.append(k)
        self.length.append(lu * k)
        self.inline_bytes.append(b"")
        self.slp_gt16 += 1
        return idx

    def expand(self, nid: int) -> bytes:
        if self.kind[nid] == 0:
            return self.inline_bytes[nid]
        out = bytearray()
        st = [(nid, 1)]
        while st:
            u, mult = st.pop()
            k = self.kind[u]
            if k == 0:
                b = self.inline_bytes[u]
                out.extend(b if mult == 1 else b * mult)
            elif k == 1:
                if mult == 1:
                    st.append((self.b[u], 1)); st.append((self.a[u], 1))
                else:
                    out.extend(self.expand(u) * mult)
            else:
                st.append((self.a[u], self.b[u] * mult))
        return bytes(out)


def load_runs_and_open_baseline(L_bytes: bytes, p: int):
    n = len(L_bytes)
    runs = []
    counts = [0] * 256
    i = 0; m16 = 0; p_thin = False
    while i < n:
        c = L_bytes[i]; j = i + 1
        while j < n and L_bytes[j] == c: j += 1
        ln = j - i
        runs.append((c, i, j))
        counts[c] += ln
        if ln <= 16:
            m16 += ln
            if i <= p < j: p_thin = True
        i = j
    if not p_thin: m16 += 1
    r0 = len(runs)
    C = [0] * 256; acc = 0
    for c in range(256): C[c] = acc; acc += counts[c]
    cur_t = C[:]
    strips = []
    by_c = [[] for _ in range(256)]
    for c, lo, hi in runs:
        d = cur_t[c] - lo; cur_t[c] += (hi - lo)
        if lo <= p < hi:
            if lo < p:
                uid = len(strips); strips.append((lo, p, d, c, -1)); by_c[c].append(uid)
            uid = len(strips); strips.append((p, p + 1, d, c, 0)); by_c[c].append(uid)
            if p + 1 < hi:
                uid = len(strips); strips.append((p + 1, hi, d, c, -1)); by_c[c].append(uid)
        else:
            uid = len(strips); strips.append((lo, hi, d, c, -1)); by_c[c].append(uid)
    j_order = []
    for c in range(256): j_order.extend(by_c[c])
    return n, r0, m16, strips, j_order, C


def build_initial_strips(L_bytes: bytes, p: int, isolate_p: bool = True):
    n, r0, m16, st_tup, j_order, _ = load_runs_and_open_baseline(L_bytes, p)
    strips = [[lo, hi, d, c, m, True] for lo, hi, d, c, m in st_tup]
    return n, r0, m16, strips, j_order, WordStore()


def build_full_incidence_256cursor(strips, j_order):
    q0 = len(strips)
    out_deg = [0] * q0; in_deg = [0] * q0
    out_target = [-1] * q0; in_source = [-1] * q0; next_r1 = [-1] * q0
    ii = 0; jj = 0
    while ii < q0 and jj < q0:
        v = ii; u = j_order[jj]
        v_lo, v_hi = strips[v][0], strips[v][1]
        u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        if u_thi <= v_lo: jj += 1
        elif v_hi <= u_tlo: ii += 1
        else:
            out_deg[u] += 1; in_deg[v] += 1
            out_target[u] = v; in_source[v] = u
            if u_tlo == v_lo and u_thi == v_hi and u != v: next_r1[u] = v
            if u_thi < v_hi: jj += 1
            elif v_hi < u_thi: ii += 1
            else: ii += 1; jj += 1
    for u in range(q0):
        if out_deg[u] != 1: out_target[u] = -1
        if in_deg[u] != 1: in_source[u] = -1
    return out_deg, in_deg, out_target, in_source, next_r1
