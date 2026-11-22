#!/usr/bin/env python3
"""
Complete Verification, Cost-Vector Instrumentation, and Paid-Cost Falsifier
for Two-Sided & Max Lazard Free-Monoid Quotient Contraction (Lazard-SLP).

1. Exhaustively verifies MAX and BOTH (Two-Sided Batched) Lazard exact reconstruction
   (with exact primary-mark tracking!) on:
     - All binary strings n <= 8, all primary marks p (3,586 cases)
     - All ternary strings n <= 5, all primary marks p (1,641 cases)
     - All binary L arrays n <= 8, all primary marks p (3,586 cases incl. 439 multi-cycle)
2. Verifies bit-for-bit exactness of MAX and BOTH on all 6 authenticated Wikidata blocks
   (after SHA-256 verification against manifest.json) and measures:
     - R (127-capped record count in I0) vs r (maximal runs)
     - Complete (W, D, B, Q, M) breakdown including:
       * sequential vs non-sequential fx_runs hops
       * in-place length-1 overwrites vs linked-list splices
       * SLP grammar size, height, expansions, memcpys, avg memcpy length
       * exact peak live memory M in bytes
3. Paid-cost falsifier on adversarial & synthetic regimes:
     - Fibonacci word F_20 (n=6765, r=2)
     - Random binary (n=50000)
     - Random 64-symbol (n=50000)
     - Adversarial monotone alphabet chain (0 1 2 ... 255)
"""

import hashlib
import json
import os
import random
from collections import defaultdict, deque


def bwt_encode(T: bytes):
    n = len(T)
    order = sorted(range(n), key=lambda i: T[i:] + T[:i])
    L = bytes(T[(i - 1) % n] for i in order)
    p = order.index(0)
    return L, p


def bwt_decode_reference(L: bytes, p: int) -> bytes:
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    s = 0
    for c in range(256):
        C[c] = s
        s += counts[c]
    occ = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = occ[b]
        occ[b] += 1
    out = bytearray(n)
    cur = p
    for t in range(n - 1, -1, -1):
        out[t] = L[cur]
        cur = LF[cur]
    return bytes(out)


class RunNode:
    __slots__ = ("id", "sym", "length", "prev", "next", "alive", "has_p", "p_off")
    def __init__(self, nid, sym, length):
        self.id = nid
        self.sym = sym
        self.length = length
        self.prev = None
        self.next = None
        self.alive = True
        self.has_p = False
        self.p_off = -1


def lazard_decode(L: bytes, p: int, mode: str = "both", measure: bool = False):
    """
    Exact Inverse BWT via Lazard Free-Monoid Quotient Contraction.
    mode in {'min', 'max', 'both'}:
      - 'min': single-sided prefix-wavefront elimination from min(Sigma)
      - 'max': single-sided suffix-wavefront elimination from max(Sigma)
      - 'both': two-sided simultaneous prefix+suffix independent-set wavefront elimination
    """
    n = len(L)
    if n == 0:
        return b"", {}

    # Count R (bzip2 127-capped records) and build maximal runs r in one sequential pass
    head = tail = None
    sym_runs = defaultdict(deque)
    sym_freq = defaultdict(int)
    r_init = 0
    R_capped = 0
    next_nid = 0
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        length = j - i
        R_capped += (length + 126) // 127
        node = RunNode(next_nid, c, length)
        next_nid += 1
        if i <= p < j:
            node.has_p = True
            node.p_off = p - i
        if tail is None:
            head = tail = node
        else:
            tail.next = node
            node.prev = tail
            tail = node
        sym_runs[c].append(node)
        sym_freq[c] += length
        r_init += 1
        i = j

    init_syms = sorted(sym_freq.keys())
    sym_prev = {}
    sym_next = {}
    for idx, s in enumerate(init_syms):
        sym_prev[s] = init_syms[idx - 1] if idx > 0 else None
        sym_next[s] = init_syms[idx + 1] if idx + 1 < len(init_syms) else None
    sym_head = init_syms[0]
    sym_tail = init_syms[-1]

    next_sym_id = 256
    prod = {}  # sid -> (y, x, k) meaning w(sid) = w(y) + w(x) * k
    sym_len = {s: 1 for s in init_syms}
    sym_height = {s: 0 for s in init_syms}

    # Metrics
    rounds = 0
    steps = 0
    fx_runs_tot = 0
    fx_seq_hops = 0     # hops where next node id == prev node id +/- 1 (contiguous!)
    fx_nonseq_hops = 0  # irregular pointer hops along L
    ux_runs_tot = 0
    ux_inplace = 0      # ux run replaced by 1 single run (in-place sym overwrite, no prev/next touch!)
    ux_multi_splice = 0 # ux run replaced by >=2 runs (needs linked-list splice)
    spliced_runs_tot = 0
    live_runs = r_init
    max_live_runs = r_init
    active_syms = len(init_syms)
    max_active_syms = active_syms
    total_syms_created = active_syms
    p_byte_shift = 0
    root_sym = None
    root_mult = 1

    def insert_sym_before(new_sym, target_sym):
        nonlocal sym_head, active_syms, max_active_syms, total_syms_created
        pr = sym_prev[target_sym]
        sym_prev[new_sym] = pr
        sym_next[new_sym] = target_sym
        sym_prev[target_sym] = new_sym
        if pr is None:
            sym_head = new_sym
        else:
            sym_next[pr] = new_sym
        active_syms += 1
        total_syms_created += 1
        if active_syms > max_active_syms:
            max_active_syms = active_syms

    def insert_sym_after(new_sym, target_sym):
        nonlocal sym_tail, active_syms, max_active_syms, total_syms_created
        nx = sym_next[target_sym]
        sym_next[new_sym] = nx
        sym_prev[new_sym] = target_sym
        sym_next[target_sym] = new_sym
        if nx is None:
            sym_tail = new_sym
        else:
            sym_prev[nx] = new_sym
        active_syms += 1
        total_syms_created += 1
        if active_syms > max_active_syms:
            max_active_syms = active_syms

    def remove_sym(x):
        nonlocal sym_head, sym_tail, active_syms
        pr = sym_prev[x]
        nx = sym_next[x]
        if pr is None:
            sym_head = nx
        else:
            sym_next[pr] = nx
        if nx is None:
            sym_tail = pr
        else:
            sym_prev[nx] = pr
        del sym_prev[x]
        del sym_next[x]
        active_syms -= 1

    def elim_min(x):
        nonlocal head, tail, next_sym_id, next_nid, steps, fx_runs_tot, fx_seq_hops, fx_nonseq_hops
        nonlocal ux_runs_tot, ux_inplace, ux_multi_splice, spliced_runs_tot, live_runs, max_live_runs
        nonlocal p_byte_shift, root_sym, root_mult
        fx = sym_freq[x]
        steps += 1
        remove_sym(x)

        fx_runs = []
        rem = fx
        prev_nid = None
        while rem > 0:
            node = head
            if prev_nid is not None:
                if node.id == prev_nid + 1:
                    fx_seq_hops += 1
                else:
                    fx_nonseq_hops += 1
            prev_nid = node.id
            if node.length <= rem:
                fx_runs.append((node.sym, node.length, node.has_p, node.p_off))
                rem -= node.length
                head = node.next
                if head is not None:
                    head.prev = None
                else:
                    tail = None
                node.alive = False
                live_runs -= 1
                popped = sym_runs[node.sym].popleft()
                assert popped is node
            else:
                hp = False
                po = -1
                if node.has_p:
                    if node.p_off < rem:
                        hp = True
                        po = node.p_off
                        node.has_p = False
                        node.p_off = -1
                    else:
                        node.p_off -= rem
                fx_runs.append((node.sym, rem, hp, po))
                node.length -= rem
                rem = 0
        fx_runs_tot += len(fx_runs)

        if fx_runs and fx_runs[0][0] == x:
            sym0, len0, hp0, po0 = fx_runs[0]
            if hp0:
                root_sym = x
                root_mult = len0 if (head is None and len(fx_runs) == 1) else 1
                return True
            fx_runs.pop(0)
        if not fx_runs:
            del sym_runs[x]
            del sym_freq[x]
            return False

        exit_runs = []
        q_idx = 0
        q_off = 0
        for sym, length, hp, po in fx_runs:
            if sym != x:
                exit_runs.append([sym, 1, length, po if hp else -1, 0])
            else:
                rem_len = length
                cur_in = 0
                while rem_len > 0:
                    ey, ek, elen, epo, epsteps = exit_runs[q_idx]
                    avail = elen - q_off
                    take = rem_len if rem_len < avail else avail
                    new_po = -1
                    new_psteps = 0
                    if hp and cur_in <= po < cur_in + take:
                        new_po = po - cur_in
                        new_psteps = 0
                    elif epo != -1 and q_off <= epo < q_off + take:
                        new_po = epo - q_off
                        new_psteps = epsteps + 1
                    exit_runs.append([ey, ek + 1, take, new_po, new_psteps])
                    cur_in += take
                    rem_len -= take
                    q_off += take
                    if q_off == elen:
                        q_idx += 1
                        q_off = 0

        new_sym_map = {}
        y_to_ks = defaultdict(set)
        for idx_e in range(q_idx, len(exit_runs)):
            ey, ek, elen, epo, epsteps = exit_runs[idx_e]
            if idx_e == q_idx and q_off == elen:
                continue
            y_to_ks[ey].add(ek)
        for ey, ks in y_to_ks.items():
            for ek in sorted(ks, reverse=True):
                sid = next_sym_id
                next_sym_id += 1
                prod[sid] = (ey, x, ek)
                sym_len[sid] = sym_len[ey] + ek * sym_len[x]
                sym_height[sid] = max(sym_height[ey], sym_height[x]) + 1
                new_sym_map[(ey, ek)] = sid
                insert_sym_before(sid, ey)

        ux_nodes = sym_runs[x]
        ux_runs_tot += len(ux_nodes)
        e_idx = q_idx
        e_off = q_off
        while ux_nodes:
            unode = ux_nodes.popleft()
            rem = unode.length
            u_hp = unode.has_p
            u_po = unode.p_off
            u_cur = 0

            # Check fast in-place case (single replacement run of exact length == unode.length)
            ey0, ek0, elen0, epo0, epsteps0 = exit_runs[e_idx]
            if elen0 - e_off >= rem:
                ux_inplace += 1
                sid = new_sym_map[(ey0, ek0)]
                unode.sym = sid
                if not u_hp and epo0 != -1 and e_off <= epo0 < e_off + rem:
                    unode.has_p = True
                    unode.p_off = epo0 - e_off
                    p_byte_shift += (epsteps0 + 1) * sym_len[x]
                sym_runs[sid].append(unode)
                sym_freq[sid] += rem
                sym_freq[ey0] -= rem
                spliced_runs_tot += 1
                e_off += rem
                if e_off == elen0:
                    e_idx += 1
                    e_off = 0
                continue

            ux_multi_splice += 1
            chain_head = chain_tail = None
            while rem > 0:
                ey, ek, elen, epo, epsteps = exit_runs[e_idx]
                avail = elen - e_off
                take = rem if rem < avail else avail
                sid = new_sym_map[(ey, ek)]
                rnode = RunNode(next_nid, sid, take)
                next_nid += 1
                if u_hp and u_cur <= u_po < u_cur + take:
                    rnode.has_p = True
                    rnode.p_off = u_po - u_cur
                elif epo != -1 and e_off <= epo < e_off + take:
                    rnode.has_p = True
                    rnode.p_off = epo - e_off
                    p_byte_shift += (epsteps + 1) * sym_len[x]
                sym_runs[sid].append(rnode)
                sym_freq[sid] += take
                sym_freq[ey] -= take
                if chain_tail is None:
                    chain_head = chain_tail = rnode
                else:
                    chain_tail.next = rnode
                    rnode.prev = chain_tail
                    chain_tail = rnode
                spliced_runs_tot += 1
                live_runs += 1
                u_cur += take
                rem -= take
                e_off += take
                if e_off == elen:
                    e_idx += 1
                    e_off = 0
            pr = unode.prev
            nx = unode.next
            if pr is not None:
                pr.next = chain_head
                chain_head.prev = pr
            else:
                head = chain_head
            if nx is not None:
                nx.prev = chain_tail
                chain_tail.next = nx
            else:
                tail = chain_tail
            unode.alive = False
            live_runs -= 1
            if live_runs > max_live_runs:
                max_live_runs = live_runs

        del sym_runs[x]
        del sym_freq[x]
        return False

    def elim_max(x):
        nonlocal head, tail, next_sym_id, next_nid, steps, fx_runs_tot, fx_seq_hops, fx_nonseq_hops
        nonlocal ux_runs_tot, ux_inplace, ux_multi_splice, spliced_runs_tot, live_runs, max_live_runs
        nonlocal p_byte_shift, root_sym, root_mult
        fx = sym_freq[x]
        steps += 1
        remove_sym(x)

        # Pop suffix of L of length fx (store in right-to-left order; p_off measured from RIGHT of each slice!)
        # Specifically: if node has length L_n and p_off in [0, L_n) from left,
        # then from the RIGHT end of the node, its offset is (L_n - 1 - p_off).
        fx_runs_rev = []  # (sym, length, has_p, p_off_from_right)
        rem = fx
        prev_nid = None
        while rem > 0:
            node = tail
            if prev_nid is not None:
                if node.id == prev_nid - 1:
                    fx_seq_hops += 1
                else:
                    fx_nonseq_hops += 1
            prev_nid = node.id
            if node.length <= rem:
                pr_off = (node.length - 1 - node.p_off) if node.has_p else -1
                fx_runs_rev.append((node.sym, node.length, node.has_p, pr_off))
                rem -= node.length
                tail = node.prev
                if tail is not None:
                    tail.next = None
                else:
                    head = None
                node.alive = False
                live_runs -= 1
                popped = sym_runs[node.sym].pop()
                assert popped is node
            else:
                # Split from RIGHT: back `rem` goes to F_x, front `node.length - rem` stays in L
                cut = node.length - rem
                hp = False
                pr_off = -1
                if node.has_p and node.p_off >= cut:
                    hp = True
                    pr_off = (node.length - 1 - node.p_off)
                    node.has_p = False
                    node.p_off = -1
                fx_runs_rev.append((node.sym, rem, hp, pr_off))
                node.length = cut
                rem = 0
        fx_runs_tot += len(fx_runs_rev)

        # Pure x-suffix at the VERY END of F_x (fx_runs_rev[0])
        if fx_runs_rev and fx_runs_rev[0][0] == x:
            sym0, len0, hp0, pro0 = fx_runs_rev[0]
            if hp0:
                root_sym = x
                root_mult = len0 if (head is None and len(fx_runs_rev) == 1) else 1
                return True
            fx_runs_rev.pop(0)
        if not fx_runs_rev:
            del sym_runs[x]
            del sym_freq[x]
            return False

        # Queue scan RIGHT-TO-LEFT (forward in fx_runs_rev)
        exit_runs_rev = []  # [y, k, length, p_off_from_right_or_-1, p_steps]
        q_idx = 0
        q_off = 0
        for sym, length, hp, pro in fx_runs_rev:
            if sym != x:
                exit_runs_rev.append([sym, 1, length, pro if hp else -1, 0])
            else:
                rem_len = length
                cur_in = 0
                while rem_len > 0:
                    ey, ek, elen, epro, epsteps = exit_runs_rev[q_idx]
                    avail = elen - q_off
                    take = rem_len if rem_len < avail else avail
                    new_pro = -1
                    new_psteps = 0
                    if hp and cur_in <= pro < cur_in + take:
                        new_pro = pro - cur_in
                        new_psteps = 0
                    elif epro != -1 and q_off <= epro < q_off + take:
                        new_pro = epro - q_off
                        new_psteps = epsteps + 1
                    exit_runs_rev.append([ey, ek + 1, take, new_pro, new_psteps])
                    cur_in += take
                    rem_len -= take
                    q_off += take
                    if q_off == elen:
                        q_idx += 1
                        q_off = 0

        # Extract suffix of exit_runs_rev (from q_idx, q_off), reverse to left-to-right,
        # converting p_off_from_right back to p_off_from_left!
        exit_lr = []
        for idx_e in range(q_idx, len(exit_runs_rev)):
            ey, ek, elen, epro, epsteps = exit_runs_rev[idx_e]
            start = q_off if idx_e == q_idx else 0
            sub_len = elen - start
            if sub_len <= 0:
                continue
            # Within this sub-slice (right-to-left offsets [start .. elen)),
            # an element at right-offset epro has sub-right-offset (epro - start),
            # hence LEFT-offset = sub_len - 1 - (epro - start)!
            if epro != -1 and start <= epro < elen:
                p_left = sub_len - 1 - (epro - start)
            else:
                p_left = -1
            exit_lr.append([ey, ek, sub_len, p_left, epsteps])
        exit_lr.reverse()

        new_sym_map = {}
        y_to_ks = defaultdict(set)
        for ey, ek, elen, epo, epsteps in exit_lr:
            y_to_ks[ey].add(ek)
        for ey, ks in y_to_ks.items():
            for ek in sorted(ks, reverse=True):
                sid = next_sym_id
                next_sym_id += 1
                prod[sid] = (ey, x, ek)
                sym_len[sid] = sym_len[ey] + ek * sym_len[x]
                sym_height[sid] = max(sym_height[ey], sym_height[x]) + 1
                new_sym_map[(ey, ek)] = sid
                insert_sym_after(sid, ey)

        ux_nodes = sym_runs[x]
        ux_runs_tot += len(ux_nodes)
        e_idx = 0
        e_off = 0
        while ux_nodes:
            unode = ux_nodes.popleft()
            rem = unode.length
            u_hp = unode.has_p
            u_po = unode.p_off
            u_cur = 0

            ey0, ek0, elen0, epo0, epsteps0 = exit_lr[e_idx]
            if elen0 - e_off >= rem:
                ux_inplace += 1
                sid = new_sym_map[(ey0, ek0)]
                unode.sym = sid
                if not u_hp and epo0 != -1 and e_off <= epo0 < e_off + rem:
                    unode.has_p = True
                    unode.p_off = epo0 - e_off
                    p_byte_shift += (epsteps0 + 1) * sym_len[x]
                sym_runs[sid].append(unode)
                sym_freq[sid] += rem
                sym_freq[ey0] -= rem
                spliced_runs_tot += 1
                e_off += rem
                if e_off == elen0:
                    e_idx += 1
                    e_off = 0
                continue

            ux_multi_splice += 1
            chain_head = chain_tail = None
            while rem > 0:
                ey, ek, elen, epo, epsteps = exit_lr[e_idx]
                avail = elen - e_off
                take = rem if rem < avail else avail
                sid = new_sym_map[(ey, ek)]
                rnode = RunNode(next_nid, sid, take)
                next_nid += 1
                if u_hp and u_cur <= u_po < u_cur + take:
                    rnode.has_p = True
                    rnode.p_off = u_po - u_cur
                elif epo != -1 and e_off <= epo < e_off + take:
                    rnode.has_p = True
                    rnode.p_off = epo - e_off
                    p_byte_shift += (epsteps + 1) * sym_len[x]
                sym_runs[sid].append(rnode)
                sym_freq[sid] += take
                sym_freq[ey] -= take
                if chain_tail is None:
                    chain_head = chain_tail = rnode
                else:
                    chain_tail.next = rnode
                    rnode.prev = chain_tail
                    chain_tail = rnode
                spliced_runs_tot += 1
                live_runs += 1
                u_cur += take
                rem -= take
                e_off += take
                if e_off == elen:
                    e_idx += 1
                    e_off = 0
            pr = unode.prev
            nx = unode.next
            if pr is not None:
                pr.next = chain_head
                chain_head.prev = pr
            else:
                head = chain_head
            if nx is not None:
                nx.prev = chain_tail
                chain_tail.next = nx
            else:
                tail = chain_tail
            unode.alive = False
            live_runs -= 1
            if live_runs > max_live_runs:
                max_live_runs = live_runs

        del sym_runs[x]
        del sym_freq[x]
        return False

    # Main wavefront loop
    done = False
    while sym_head is not None and not done:
        while sym_head is not None and sym_freq[sym_head] == 0:
            x = sym_head
            remove_sym(x)
            del sym_runs[x]
            del sym_freq[x]
        while sym_tail is not None and sym_freq[sym_tail] == 0:
            x = sym_tail
            remove_sym(x)
            del sym_runs[x]
            del sym_freq[x]
        if sym_head is None:
            break

        S_left = []
        S_right = []
        S_set = set()
        forbidden = set()

        if mode in ("min", "both"):
            cand = sym_head
            pn = head
            po = 0
            while cand is not None:
                fc = sym_freq[cand]
                if fc == 0:
                    cand = sym_next[cand]
                    continue
                if cand in forbidden or cand in S_set:
                    break
                rem = fc
                tn, to = pn, po
                csyms = set()
                conflict = False
                while rem > 0:
                    avail = tn.length - to
                    if tn.sym in S_set:
                        conflict = True
                        break
                    if tn.sym != cand:
                        csyms.add(tn.sym)
                    if avail <= rem:
                        rem -= avail
                        tn = tn.next
                        to = 0
                    else:
                        to += rem
                        rem = 0
                if conflict:
                    break
                S_left.append(cand)
                S_set.add(cand)
                forbidden.update(csyms)
                pn, po = tn, to
                cand = sym_next[cand]

        if mode in ("max", "both"):
            cand = sym_tail
            pn = tail
            po = 0
            while cand is not None:
                fc = sym_freq[cand]
                if fc == 0:
                    cand = sym_prev[cand]
                    continue
                if cand in forbidden or cand in S_set:
                    break
                rem = fc
                tn, to = pn, po
                csyms = set()
                conflict = False
                while rem > 0:
                    avail = tn.length - to
                    if tn.sym in S_set:
                        conflict = True
                        break
                    if tn.sym != cand:
                        csyms.add(tn.sym)
                    if avail <= rem:
                        rem -= avail
                        tn = tn.prev
                        to = 0
                    else:
                        to += rem
                        rem = 0
                if conflict:
                    break
                S_right.append(cand)
                S_set.add(cand)
                forbidden.update(csyms)
                pn, po = tn, to
                cand = sym_prev[cand]

        rounds += 1
        for x in S_left:
            if elim_min(x):
                done = True
                break
        if not done:
            for x in S_right:
                if elim_max(x):
                    done = True
                    break

    assert root_sym is not None

    # First-occurrence-memcpy SLP emission directly into output buffer!
    L_root = sym_len[root_sym]
    buf = bytearray(L_root)
    first_pos = {}
    slp_exp = 0
    slp_cpy = 0
    slp_cpy_bytes = 0
    slp_lit_bytes = 0
    pos = 0
    stack = [(root_sym, 1)]
    while stack:
        s, mult = stack.pop()
        if s < 256:
            buf[pos:pos + mult] = bytes([s]) * mult
            pos += mult
            slp_lit_bytes += mult
        elif s in first_pos:
            src = first_pos[s]
            slen = sym_len[s]
            for _ in range(mult):
                buf[pos:pos + slen] = buf[src:src + slen]
                pos += slen
            slp_cpy += 1
            slp_cpy_bytes += slen * mult
        else:
            first_pos[s] = pos
            slp_exp += 1
            if mult > 1:
                stack.append((s, mult - 1))
            y, x, k = prod[s]
            stack.append((x, k))
            stack.append((y, 1))
    assert pos == L_root

    shift = (-p_byte_shift) % L_root
    unit = bytes(buf[shift:]) + bytes(buf[:shift])
    T_out = unit * root_mult if root_mult > 1 else unit

    stats = {}
    if measure:
        stats = {
            "n": n, "R": R_capped, "r": r_init, "rounds": rounds, "steps": steps,
            "fx_runs": fx_runs_tot, "fx_seq": fx_seq_hops, "fx_nonseq": fx_nonseq_hops,
            "ux_runs": ux_runs_tot, "ux_inplace": ux_inplace, "ux_multi": ux_multi_splice,
            "spliced": spliced_runs_tot, "max_live_r": max_live_runs,
            "max_syms": max_active_syms, "tot_syms": total_syms_created,
            "slp_height": sym_height[root_sym], "slp_exp": slp_exp, "slp_cpy": slp_cpy,
            "slp_cpy_bytes": slp_cpy_bytes, "slp_lit_bytes": slp_lit_bytes,
        }
    return T_out, stats


def run_exhaustive():
    print("=== EXHAUSTIVE EXACTNESS VERIFICATION (MAX & BOTH MODES) ===")
    for mode in ("max", "both"):
        bin_c = 0
        for n in range(1, 9):
            for mask in range(1 << n):
                T = bytes(((mask >> i) & 1) + ord('a') for i in range(n))
                L, p = bwt_encode(T)
                for pt in range(n):
                    ref = bwt_decode_reference(L, pt)
                    got, _ = lazard_decode(L, pt, mode=mode)
                    assert got == ref, f"{mode} bin FAIL n={n} T={T} p={pt}: {ref} vs {got}"
                    bin_c += 1
        ter_c = 0
        for n in range(1, 6):
            for code in range(3 ** n):
                tmp = code
                arr = []
                for _ in range(n):
                    arr.append((tmp % 3) + ord('a'))
                    tmp //= 3
                T = bytes(arr)
                L, p = bwt_encode(T)
                for pt in range(n):
                    ref = bwt_decode_reference(L, pt)
                    got, _ = lazard_decode(L, pt, mode=mode)
                    assert got == ref, f"{mode} ter FAIL n={n} T={T} p={pt}: {ref} vs {got}"
                    ter_c += 1
        mc_c = 0
        for n in range(1, 9):
            for mask in range(1 << n):
                L = bytes(((mask >> i) & 1) + ord('a') for i in range(n))
                counts = [0] * 256
                for b in L:
                    counts[b] += 1
                C = [0] * 256
                s = 0
                for c in range(256):
                    C[c] = s
                    s += counts[c]
                occ = C[:]
                LF = [0] * n
                for i, b in enumerate(L):
                    LF[i] = occ[b]
                    occ[b] += 1
                for pt in range(n):
                    cyc = []
                    cur = pt
                    while True:
                        cyc.append(L[cur])
                        cur = LF[cur]
                        if cur == pt:
                            break
                    cyc_word = bytes(reversed(cyc))
                    got, _ = lazard_decode(L, pt, mode=mode)
                    if got != cyc_word:
                        assert len(got) % len(cyc_word) == 0 and got == cyc_word * (len(got) // len(cyc_word)), (
                            f"{mode} mc FAIL L={L} p={pt}: {cyc_word} vs {got}"
                        )
                    mc_c += 1
        print(f"[verified] mode={mode:4s}: bin={bin_c} + ter={ter_c} + multi-cycle={mc_c} = {bin_c+ter_c+mc_c} cases PASSED")


def run_wikidata():
    print("\n=== ALL 6 AUTHENTICATED WIKIDATA BLOCKS: EXACTNESS + FULL COST VECTOR (BOTH & MAX) ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    for entry in manifest["blocks"]:
        fpath = os.path.join("/tmp/mibz2-blocks", entry["file"])
        with open(fpath, "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        T_ref = bwt_decode_reference(L, p)
        print(f"\n{entry['file']} (n={len(L)}, r={entry['r']}):")
        for mode in ("max", "both"):
            T_got, st = lazard_decode(L, p, mode=mode, measure=True)
            assert T_got == T_ref, f"MISMATCH on {entry['file']} mode={mode}!"
            r = st["r"]
            # Memory calculation (packed native representation):
            # - Run arena: max_live_r nodes * 20 bytes (sym:u32, len:u32, prev:u32, next:u32, sym_next:u32)
            # - Active sym metadata (ring/pool of max_syms): max_syms * 24 bytes (head, tail, freq, prev, next, len)
            # - Grammar prod table: tot_syms * 8 bytes (packed y:20b, x:20b, k:4b, len:20b)
            # - Emission: first_pos[tot_syms]*4B + T[n]
            M_phase1 = st["max_live_r"] * 20 + st["max_syms"] * 24 + st["tot_syms"] * 8
            M_phase2 = st["tot_syms"] * 12 + st["n"]
            M_peak = max(M_phase1, M_phase2)
            avg_cpy = st["slp_cpy_bytes"] / st["slp_cpy"] if st["slp_cpy"] else 0.0
            print(
                f"  [{mode:4s}] EXACT! R={st['R']} | D_rounds={st['rounds']:4d} | steps={st['steps']:5d} | "
                f"fx={st['fx_runs']} ({st['fx_runs']/r:.2f}r, nonseq={st['fx_nonseq']/r:.2f}r) | "
                f"ux={st['ux_runs']} ({st['ux_runs']/r:.2f}r, inplace={100*st['ux_inplace']/st['ux_runs']:.1f}%) | "
                f"tot_syms={st['tot_syms']} ({st['tot_syms']/r:.2f}r) h={st['slp_height']} | "
                f"emit: exp={st['slp_exp']/r:.2f}r cpy={st['slp_cpy']/r:.2f}r (avg {avg_cpy:.1f}B, {100*st['slp_cpy_bytes']/st['n']:.1f}%) | "
                f"M_p1={M_phase1/1e6:.2f}MB M_p2={M_phase2/1e6:.2f}MB"
            )


def run_falsifier():
    print("\n=== PAID-COST FALSIFIER: SYNTHETIC & ADVERSARIAL REGIMES ===")
    # 1. Fibonacci word F_20 (n = 6765, r = 2)
    a, b = b"a", b"ab"
    while len(b) < 6765:
        a, b = b, b + a
    L_fib, p_fib = bwt_encode(b)
    T_got, st = lazard_decode(L_fib, p_fib, mode="both", measure=True)
    assert T_got == b
    print(f"  Fibonacci F_20 (n={st['n']}, r={st['r']}): rounds={st['rounds']}, steps={st['steps']}, fx={st['fx_runs']}, ux={st['ux_runs']}, tot_syms={st['tot_syms']}, h={st['slp_height']}")

    # 2. Random binary string (n = 50,000)
    rng = random.Random(42)
    T_rbin = bytes(rng.choice([ord('a'), ord('b')]) for _ in range(50000))
    L_rbin, p_rbin = bwt_encode(T_rbin)
    T_got, st = lazard_decode(L_rbin, p_rbin, mode="both", measure=True)
    assert T_got == T_rbin
    r = st["r"]
    print(f"  Random binary  (n={st['n']}, r={r}): rounds={st['rounds']}, steps={st['steps']}, fx={st['fx_runs']/r:.2f}r, ux={st['ux_runs']/r:.2f}r, tot_syms={st['tot_syms']/r:.2f}r, h={st['slp_height']}")

    # 3. Random 64-byte alphabet (n = 50,000) — models pure GUID / high-entropy regime!
    T_r64 = bytes(rng.randint(33, 96) for _ in range(50000))
    L_r64, p_r64 = bwt_encode(T_r64)
    T_got, st = lazard_decode(L_r64, p_r64, mode="both", measure=True)
    assert T_got == T_r64
    r = st["r"]
    print(f"  Random |S|=64  (n={st['n']}, r={r}): rounds={st['rounds']}, steps={st['steps']}, fx={st['fx_runs']/r:.2f}r, ux={st['ux_runs']/r:.2f}r, tot_syms={st['tot_syms']/r:.2f}r, h={st['slp_height']}")

    # 4. Adversarial sequential chain: 0 1 2 ... 255 (n = 256, r = 256)
    T_chain = bytes(range(256))
    L_ch, p_ch = bwt_encode(T_chain)
    for mode in ("min", "both"):
        T_got, st = lazard_decode(L_ch, p_ch, mode=mode, measure=True)
        assert T_got == T_chain
        print(f"  Adv chain 0..255 [{mode:4s}] (n={st['n']}, r={st['r']}): rounds={st['rounds']}, steps={st['steps']}, h={st['slp_height']}")

    # 5. Adversarial large chain: binary de Bruijn / long ascending chunks (repeating 0..255 with perturbations)
    # Wait: what about a 50,000-byte worst-case ascending chain?
    # T = b"".join(bytes([i % 256]) for i in range(50000)) is periodic; break periodicity with 1 byte!
    T_near_per = bytearray(i % 256 for i in range(50000))
    T_near_per[0] = 255
    L_np, p_np = bwt_encode(bytes(T_near_per))
    T_got, st = lazard_decode(L_np, p_np, mode="both", measure=True)
    assert T_got == bytes(T_near_per)
    print(f"  Near-per 0..255 (n={st['n']}, r={st['r']}): rounds={st['rounds']}, steps={st['steps']}, fx={st['fx_runs']}, ux={st['ux_runs']}, h={st['slp_height']}")


if __name__ == "__main__":
    run_exhaustive()
    run_wikidata()
    run_falsifier()
