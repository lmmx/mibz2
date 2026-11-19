#!/usr/bin/env python3
"""
Exhaustive verifier for Theorem 3' (Multi-Boundary Parallel Rauzy-Veech).
Standard library only. Run with: python3 verify_thm3.py
"""
from __future__ import annotations
from dataclasses import dataclass
from itertools import combinations, permutations, product
from typing import Dict, List, Optional, Set, Tuple


@dataclass
class Node:
    nid: int
    is_sentinel: bool = False
    length: int = 0
    word: str = ""
    dom_prev: int = -1
    dom_next: int = -1
    cod_prev: int = -1
    cod_next: int = -1


class MultiBoundaryDIET:
    def __init__(
        self,
        comp_dom: List[List[Tuple[int, int, str]]],  # per comp: [(nid, len, word)]
        comp_cod: List[List[int]],                   # per comp: [nid]
    ) -> None:
        self.K = len(comp_dom)
        self.nodes: Dict[int, Node] = {}
        self.live: Set[int] = set()
        self.dom_left: List[int] = []
        self.dom_right: List[int] = []
        self.cod_left: List[int] = []
        self.cod_right: List[int] = []

        # Allocate negative IDs for sentinels so they never collide with interval IDs >= 0.
        sid = -1
        for j in range(self.K):
            for arr in (self.dom_left, self.dom_right, self.cod_left, self.cod_right):
                self.nodes[sid] = Node(nid=sid, is_sentinel=True)
                arr.append(sid)
                sid -= 1

        # Create interval nodes.
        for j in range(self.K):
            for nid, length, word in comp_dom[j]:
                assert nid >= 0 and length > 0 and nid not in self.nodes
                self.nodes[nid] = Node(nid=nid, is_sentinel=False, length=length, word=word)
                self.live.add(nid)

        # Wire domain and codomain doubly-linked lists per component.
        for j in range(self.K):
            d_chain = [self.dom_left[j]] + [nid for nid, _, _ in comp_dom[j]] + [self.dom_right[j]]
            for a, b in zip(d_chain, d_chain[1:]):
                self.nodes[a].dom_next = b
                self.nodes[b].dom_prev = a

            c_chain = [self.cod_left[j]] + list(comp_cod[j]) + [self.cod_right[j]]
            for a, b in zip(c_chain, c_chain[1:]):
                self.nodes[a].cod_next = b
                self.nodes[b].cod_prev = a

        self.check_invariants()

    def clone(self) -> "MultiBoundaryDIET":
        comp_dom: List[List[Tuple[int, int, str]]] = []
        comp_cod: List[List[int]] = []
        for j in range(self.K):
            d_list = []
            cur = self.nodes[self.dom_left[j]].dom_next
            while cur != self.dom_right[j]:
                nd = self.nodes[cur]
                d_list.append((nd.nid, nd.length, nd.word))
                cur = nd.dom_next
            c_list = []
            cur = self.nodes[self.cod_left[j]].cod_next
            while cur != self.cod_right[j]:
                c_list.append(cur)
                cur = self.nodes[cur].cod_next
            comp_dom.append(d_list)
            comp_cod.append(c_list)
        return MultiBoundaryDIET(comp_dom, comp_cod)

    def check_invariants(self) -> None:
        seen_dom: Set[int] = set()
        seen_cod: Set[int] = set()
        for j in range(self.K):
            # Domain list traversal + symmetry
            dl, dr = self.dom_left[j], self.dom_right[j]
            assert self.nodes[dl].dom_prev == -1 and self.nodes[dr].dom_next == -1
            cur = dl
            dom_len = 0
            while cur != dr:
                nxt = self.nodes[cur].dom_next
                assert nxt in self.nodes, f"Dangling dom_next from {cur}"
                assert self.nodes[nxt].dom_prev == cur, f"Broken dom symmetry {cur} <-> {nxt}"
                if nxt != dr:
                    assert not self.nodes[nxt].is_sentinel
                    assert nxt in self.live and nxt not in seen_dom
                    assert self.nodes[nxt].length > 0
                    seen_dom.add(nxt)
                    dom_len += self.nodes[nxt].length
                cur = nxt

            # Codomain list traversal + symmetry
            cl, cr = self.cod_left[j], self.cod_right[j]
            assert self.nodes[cl].cod_prev == -1 and self.nodes[cr].cod_next == -1
            cur = cl
            cod_len = 0
            while cur != cr:
                nxt = self.nodes[cur].cod_next
                assert nxt in self.nodes, f"Dangling cod_next from {cur}"
                assert self.nodes[nxt].cod_prev == cur, f"Broken cod symmetry {cur} <-> {nxt}"
                if nxt != cr:
                    assert not self.nodes[nxt].is_sentinel
                    assert nxt in self.live and nxt not in seen_cod
                    seen_cod.add(nxt)
                    cod_len += self.nodes[nxt].length
                cur = nxt

            assert dom_len == cod_len, f"Component {j} length mismatch: dom={dom_len}, cod={cod_len}"

        assert seen_dom == self.live and seen_cod == self.live, "Live set / list mismatch"

    def front_pair(self, j: int) -> Optional[Tuple[int, int]]:
        a = self.nodes[self.dom_right[j]].dom_prev
        b = self.nodes[self.cod_right[j]].cod_prev
        if a == self.dom_left[j]:
            assert b == self.cod_left[j]
            return None
        assert b != self.cod_left[j]
        return a, b

    def footprint(self, j: int) -> Optional[Set[int]]:
        pair = self.front_pair(j)
        if pair is None:
            return None
        a, b = pair
        if a == b:
            return None  # Case 3b handled outside normal batches
        na, nb = self.nodes[a], self.nodes[b]
        if na.length > nb.length:
            raw = {a, b, na.cod_next, nb.cod_prev}
        elif na.length < nb.length:
            raw = {a, b, nb.dom_next, na.dom_prev}
        else:
            raw = {a, b, na.dom_prev, na.cod_prev, na.cod_next, nb.cod_prev}
        return {x for x in raw if x >= 0}

    def is_admissible_batch(self, S: Tuple[int, ...]) -> bool:
        seen_fp: Set[int] = set()
        As: Set[int] = set()
        Bs: Set[int] = set()
        for j in S:
            pair = self.front_pair(j)
            if pair is None:
                return False
            a, b = pair
            if a == b:
                return False
            fp = self.footprint(j)
            assert fp is not None
            if seen_fp & fp:
                return False
            seen_fp |= fp
            As.add(a)
            Bs.add(b)
        if As & Bs:
            return False
        return True

    def to_row_map(self) -> Tuple[Dict[int, int], Dict[int, str], Dict[int, Tuple[int, int]]]:
        """Materialize row -> T(row), row -> word, row -> (interval_id, offset) in compressed coords."""
        dom_pos: Dict[int, int] = {}
        cod_pos: Dict[int, int] = {}
        pos = 0
        for j in range(self.K):
            cur = self.nodes[self.dom_left[j]].dom_next
            while cur != self.dom_right[j]:
                dom_pos[cur] = pos
                pos += self.nodes[cur].length
                cur = self.nodes[cur].dom_next
        pos_c = 0
        for j in range(self.K):
            cur = self.nodes[self.cod_left[j]].cod_next
            while cur != self.cod_right[j]:
                cod_pos[cur] = pos_c
                pos_c += self.nodes[cur].length
                cur = self.nodes[cur].cod_next
        assert pos == pos_c

        T: Dict[int, int] = {}
        W: Dict[int, str] = {}
        loc: Dict[int, Tuple[int, int]] = {}
        for nid in self.live:
            nd = self.nodes[nid]
            d0, c0 = dom_pos[nid], cod_pos[nid]
            for off in range(nd.length):
                T[d0 + off] = c0 + off
                W[d0 + off] = nd.word
                loc[d0 + off] = (nid, off)
        return T, W, loc

    def cut_rows(self, S: Tuple[int, ...]) -> Set[int]:
        """Exact set Y = union_{j in S} Y_j in current compressed coordinates."""
        Y: Set[int] = set()
        pos = 0
        for j in range(self.K):
            cur = self.nodes[self.dom_left[j]].dom_next
            while cur != self.dom_right[j]:
                pos += self.nodes[cur].length
                cur = self.nodes[cur].dom_next
            if j in S:
                pair = self.front_pair(j)
                assert pair is not None
                a, b = pair
                delta = min(self.nodes[a].length, self.nodes[b].length)
                for r in range(pos - delta, pos):
                    Y.add(r)
        return Y

    def apply_front(
        self,
        j: int,
        marks: Dict[int, Tuple[int, int, int]],  # p0 -> (I_p, theta_p, delta)
    ) -> None:
        pair = self.front_pair(j)
        assert pair is not None
        a, b = pair
        assert a != b
        na, nb = self.nodes[a], self.nodes[b]
        la, lb = na.length, nb.length
        old_wb = nb.word

        if la > lb:
            # Case 1
            new_la = la - lb
            na.length = new_la
            # Unlink b from codomain FIRST
            bp, bn = nb.cod_prev, nb.cod_next
            self.nodes[bp].cod_next = bn
            self.nodes[bn].cod_prev = bp
            # Splice b immediately after a in codomain
            an = na.cod_next
            na.cod_next = b
            nb.cod_prev = a
            nb.cod_next = an
            self.nodes[an].cod_prev = b
            nb.word = old_wb + na.word

            for p0, (ip, th, d) in list(marks.items()):
                if ip == a and th >= new_la:
                    marks[p0] = (b, th - new_la, d + len(old_wb))

        elif la < lb:
            # Case 2
            new_lb = lb - la
            nb.length = new_lb
            # Unlink a from domain FIRST
            ap, an = na.dom_prev, na.dom_next
            self.nodes[ap].dom_next = an
            self.nodes[an].dom_prev = ap
            # Splice a immediately after b in domain
            bn = nb.dom_next
            nb.dom_next = a
            na.dom_prev = b
            na.dom_next = bn
            self.nodes[bn].dom_prev = a
            na.word = old_wb + na.word

            for p0, (ip, th, d) in list(marks.items()):
                if ip == a:
                    marks[p0] = (a, th, d + len(old_wb))
                elif ip == b and th >= new_lb:
                    marks[p0] = (a, th - new_lb, d)

        else:
            # Case 3a
            # Unlink a from domain
            ap, an = na.dom_prev, na.dom_next
            self.nodes[ap].dom_next = an
            self.nodes[an].dom_prev = ap
            # Unlink b from codomain FIRST (handles a == cod_prev(b))
            bp, bn = nb.cod_prev, nb.cod_next
            self.nodes[bp].cod_next = bn
            self.nodes[bn].cod_prev = bp
            # Replace a with b in codomain
            acp, acn = na.cod_prev, na.cod_next
            self.nodes[acp].cod_next = b
            nb.cod_prev = acp
            nb.cod_next = acn
            self.nodes[acn].cod_prev = b
            nb.word = old_wb + na.word
            self.live.remove(a)
            del self.nodes[a]

            for p0, (ip, th, d) in list(marks.items()):
                if ip == a:
                    marks[p0] = (b, th, d + len(old_wb))

    def apply_self_loop(self, j: int, marks: Dict[int, Tuple[int, int, int]]) -> Dict[int, str]:
        """Case 3b: resolve A_j == B_j self-loop; returns decoded cyclic L-streams for marks inside."""
        pair = self.front_pair(j)
        assert pair is not None
        a, b = pair
        assert a == b
        na = self.nodes[a]
        resolved: Dict[int, str] = {}
        for p0, (ip, th, d) in list(marks.items()):
            if ip == a:
                w = na.word
                resolved[p0] = w[d:] + w[:d]
                del marks[p0]
        # Unlink from both lists and delete
        self.nodes[na.dom_prev].dom_next = na.dom_next
        self.nodes[na.dom_next].dom_prev = na.dom_prev
        self.nodes[na.cod_prev].cod_next = na.cod_next
        self.nodes[na.cod_next].cod_prev = na.cod_prev
        self.live.remove(a)
        del self.nodes[a]
        return resolved


def direct_first_return(
    T: Dict[int, int], W: Dict[int, str], Y: Set[int]
) -> Tuple[Dict[int, int], Dict[int, str], Dict[int, int]]:
    """Brute-force rowwise first-return map on X' = X \\ Y (compressed to 0..|X'|-1)."""
    X_prime = sorted(x for x in T if x not in Y)
    idx = {x: i for i, x in enumerate(X_prime)}
    T_ind: Dict[int, int] = {}
    W_ind: Dict[int, str] = {}
    for x in X_prime:
        cur = T[x]
        word = W[x]
        steps = 1
        while cur in Y:
            word += W[cur]
            cur = T[cur]
            steps += 1
            assert steps <= len(T), "Cycle completely inside Y"
        T_ind[idx[x]] = idx[cur]
        W_ind[idx[x]] = word
    return T_ind, W_ind, idx


def cyclic_stream(T: Dict[int, int], W: Dict[int, str], start: int) -> str:
    """Exact single-period cyclic L-stream starting at row `start`."""
    out = [W[start]]
    cur = T[start]
    while cur != start:
        out.append(W[cur])
        cur = T[cur]
    return "".join(out)


def count_maximal_continuity_intervals(T: Dict[int, int], W: Dict[int, str]) -> int:
    m = len(T)
    if m == 0:
        return 0
    runs = 1
    for i in range(1, m):
        if T[i] != T[i - 1] + 1 or W[i] != W[i - 1]:
            runs += 1
    return runs


def verify_single_batch(diet: MultiBoundaryDIET, S: Tuple[int, ...]) -> Tuple[bool, bool]:
    """
    Verifies Theorem 3' on (diet, S):
      - All orderings of S yield identical final state (confluence)
      - Exact match with direct rowwise first-return map and return words
      - Exact match for every primary mark p0 in [0..n-1]
      - All structural invariants and r(T_{X'}) <= r(T_X)
    Returns (passed, was_strict_refinement_of_maximal_intervals).
    """
    T0, W0, loc0 = diet.to_row_map()
    n = len(T0)
    r0 = len(diet.live)
    orig_streams = {p0: cyclic_stream(T0, W0, p0) for p0 in range(n)}
    Y = diet.cut_rows(S)
    T_ref, W_ref, _ = direct_first_return(T0, W0, Y)

    ref_state = None
    ref_marks = None
    orderings = list(permutations(S)) if len(S) <= 4 else [S, tuple(reversed(S))]

    for order in orderings:
        d_copy = diet.clone()
        marks = {p0: (loc0[p0][0], loc0[p0][1], 0) for p0 in range(n)}
        for j in order:
            d_copy.apply_front(j, marks)
        d_copy.check_invariants()

        T1, W1, loc1 = d_copy.to_row_map()
        inv_loc1 = {v: k for k, v in loc1.items()}
        assert len(d_copy.live) <= r0, f"Interval count grew: {r0} -> {len(d_copy.live)}"
        assert T1 == T_ref, f"Induced T mismatch for S={S}, order={order}"
        assert W1 == W_ref, f"Induced W mismatch for S={S}, order={order}"

        # Verify every primary mark p0
        for p0 in range(n):
            ip, th, delta = marks[p0]
            assert (ip, th) in inv_loc1, f"Invalid mark location for p0={p0}"
            row_p = inv_loc1[(ip, th)]
            assert 0 <= delta < len(W1[row_p]), f"Delta out of range for p0={p0}"
            s_new = cyclic_stream(T1, W1, row_p)
            rot = s_new[delta:] + s_new[:delta]
            assert rot == orig_streams[p0], f"Stream mismatch for p0={p0}: {rot} != {orig_streams[p0]}"

        state_sig = (T1, W1, sorted((nid, d_copy.nodes[nid].length, d_copy.nodes[nid].word) for nid in d_copy.live))
        if ref_state is None:
            ref_state, ref_marks = state_sig, dict(marks)
        else:
            assert state_sig == ref_state and marks == ref_marks, f"Non-confluent for S={S}"

    max_runs = count_maximal_continuity_intervals(T_ref, W_ref)
    strict_ref = len(ref_state[2]) > max_runs  # type: ignore[index]
    return True, strict_ref


def build_bwt_diet(text: str, cuts: Tuple[int, ...]) -> Tuple[MultiBoundaryDIET, int]:
    """Build exact BWT dIET for `text` with pinned codomain cuts in {C[c]}."""
    n = len(text)
    rots = sorted((text[i:] + text[:i], i) for i in range(n))
    p0 = next(idx for idx, (_, start) in enumerate(rots) if start == 0)
    L = "".join(text[(start - 1) % n] for _, start in rots)

    # Stable-rank LF
    chars = sorted(set(L))
    C: Dict[str, int] = {}
    acc = 0
    for c in chars:
        C[c] = acc
        acc += L.count(c)
    seen = {c: 0 for c in chars}
    LF = [0] * n
    for i, c in enumerate(L):
        LF[i] = C[c] + seen[c]
        seen[c] += 1

    # Maximal constant-symbol runs of L, split at any pinned cut b in cuts
    boundaries = {0, n} | set(cuts)
    for i in range(1, n):
        if L[i] != L[i - 1]:
            boundaries.add(i)
    dom_bounds = sorted(boundaries)

    intervals = []
    for nid, (a, b) in enumerate(zip(dom_bounds, dom_bounds[1:])):
        intervals.append((nid, a, b, LF[a], LF[a] + (b - a), L[a]))

    all_cuts = (0,) + cuts + (n,)
    comp_dom: List[List[Tuple[int, int, str]]] = []
    comp_cod: List[List[int]] = []
    for u, v in zip(all_cuts, all_cuts[1:]):
        d_items = [(nid, b - a, w) for nid, a, b, _, _, w in intervals if u <= a and b <= v]
        c_items = sorted(
            [(ca, nid) for nid, _, _, ca, cb, _ in intervals if u <= ca and cb <= v]
        )
        comp_dom.append(d_items)
        comp_cod.append([nid for _, nid in c_items])

    return MultiBoundaryDIET(comp_dom, comp_cod), p0


def verify_full_bwt_decode(text: str, cuts: Tuple[int, ...]) -> None:
    """Run multi-step batched induction all the way to completion and decode `text`."""
    diet, p0 = build_bwt_diet(text, cuts)
    T0, W0, loc0 = diet.to_row_map()
    marks = {p0: (loc0[p0][0], loc0[p0][1], 0)}
    resolved: Dict[int, str] = {}

    while diet.live:
        # Resolve any Case 3b self-loops first
        did_loop = False
        for j in range(diet.K):
            pair = diet.front_pair(j)
            if pair is not None and pair[0] == pair[1]:
                resolved.update(diet.apply_self_loop(j, marks))
                diet.check_invariants()
                did_loop = True
                break
        if did_loop:
            continue

        # Greedily form the largest conflict-free batch S
        batch: List[int] = []
        for j in range(diet.K):
            cand = tuple(batch + [j])
            if diet.is_admissible_batch(cand):
                batch.append(j)
        assert batch, "Deadlock: live intervals remain but no admissible front"
        for j in batch:
            diet.apply_front(j, marks)
        diet.check_invariants()

    assert p0 in resolved
    stream = resolved[p0]
    n = len(text)
    assert n % len(stream) == 0
    decoded = (stream[::-1]) * (n // len(stream))
    assert decoded == text, f"Decoded {decoded!r} != {text!r}"


def run_all() -> None:
    stats = {
        "bwt_configs": 0,
        "bwt_batches_tested": 0,
        "bwt_multi_front_batches": 0,
        "bwt_full_decodes": 0,
        "diet_configs": 0,
        "diet_batches_tested": 0,
        "diet_multi_front_batches": 0,
        "strict_refinements": 0,
    }

    # 1. Corrected Test A
    test_a = MultiBoundaryDIET(
        comp_dom=[
            [(0, 1, "a"), (1, 1, "b"), (2, 1, "c"), (3, 2, "d")],
            [(4, 1, "e"), (5, 1, "f"), (6, 1, "g"), (7, 2, "h")],
        ],
        comp_cod=[
            [7, 4, 5, 6],
            [3, 2, 0, 1],
        ],
    )
    assert test_a.is_admissible_batch((0, 1))
    ok, _ = verify_single_batch(test_a, (0, 1))
    assert ok
    print("Test A (n=10, r=8, K=2 cross-component 2-front batch): PASS")

    # 2. Exhaustive BWTs (binary n=2..8, ternary n=2..5)
    bwt_sets = [
        ("ab", range(2, 9)),
        ("abc", range(2, 6)),
    ]
    for alphabet, lengths in bwt_sets:
        for n in lengths:
            for tup in product(alphabet, repeat=n):
                text = "".join(tup)
                # Candidate codomain cuts from {C[c]} strictly inside (0, n)
                c_cuts = sorted({text.count(c_less) for _ in [0] for c_less in []})  # placeholder
                chars = sorted(set(text))
                acc = 0
                c_bounds = []
                for c in chars:
                    if 0 < acc < n:
                        c_bounds.append(acc)
                    acc += text.count(c)
                # Test K=1 (no cuts) and all nonempty subsets of c_bounds
                for r_cut in range(len(c_bounds) + 1):
                    for cuts in combinations(c_bounds, r_cut):
                        diet, _ = build_bwt_diet(text, cuts)
                        stats["bwt_configs"] += 1
                        for k_sz in range(1, diet.K + 1):
                            for S in combinations(range(diet.K), k_sz):
                                if diet.is_admissible_batch(S):
                                    _, strict = verify_single_batch(diet, S)
                                    stats["bwt_batches_tested"] += 1
                                    if len(S) >= 2:
                                        stats["bwt_multi_front_batches"] += 1
                                    if strict:
                                        stats["strict_refinements"] += 1
                        verify_full_bwt_decode(text, cuts)
                        stats["bwt_full_decodes"] += 1

    # 3. Exhaustive general dIETs (r=4,5 with K=2,3 and varied lengths)
    for r in (4, 5):
        words = [chr(ord("a") + i) for i in range(r)]
        for perm in permutations(range(r)):
            # Skip identity-free? No, include all permutations (multi-cycle, fixed points, cross-comp)
            for lengths in ([1] * r, [1 + (i % 2) for i in range(r)]):
                # Find all internal points in D(T) intersect C(T)
                dom_ends = set()
                acc = 0
                for i in range(r - 1):
                    acc += lengths[i]
                    dom_ends.add(acc)
                cod_ends = set()
                acc = 0
                for i in range(r - 1):
                    acc += lengths[perm[i]]
                    cod_ends.add(acc)
                common = sorted(dom_ends & cod_ends)
                for r_cut in range(1, len(common) + 1):
                    for cuts in combinations(common, r_cut):
                        all_cuts = (0,) + cuts + (sum(lengths),)
                        # Build component lists
                        comp_dom: List[List[Tuple[int, int, str]]] = []
                        comp_cod: List[List[int]] = []
                        d_pos = [0] * r
                        acc = 0
                        for i in range(r):
                            d_pos[i] = acc
                            acc += lengths[i]
                        c_pos = [0] * r
                        acc = 0
                        for i in range(r):
                            c_pos[perm[i]] = acc
                            acc += lengths[perm[i]]
                        for u, v in zip(all_cuts, all_cuts[1:]):
                            d_items = [(i, lengths[i], words[i]) for i in range(r) if u <= d_pos[i] < v]
                            c_items = [perm[i] for i in range(r) if u <= c_pos[perm[i]] < v]
                            comp_dom.append(d_items)
                            comp_cod.append(c_items)
                        diet = MultiBoundaryDIET(comp_dom, comp_cod)
                        stats["diet_configs"] += 1
                        for k_sz in range(1, diet.K + 1):
                            for S in combinations(range(diet.K), k_sz):
                                if diet.is_admissible_batch(S):
                                    _, strict = verify_single_batch(diet, S)
                                    stats["diet_batches_tested"] += 1
                                    if len(S) >= 2:
                                        stats["diet_multi_front_batches"] += 1
                                    if strict:
                                        stats["strict_refinements"] += 1

    print("ALL TESTS PASSED")
    for k, v in stats.items():
        print(f"  {k}: {v}")


if __name__ == "__main__":
    run_all()

