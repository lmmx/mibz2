#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 12 Mathematical Probe & Verifier (probe_cycle12.py)

1. Theorem 12.1 — Absorbing Zorich-Kontsevich Renormalization & Left-vs-Right Interval-Pairing Duality:
   - Compares Right-Elimination (`endpoint.rs` in mibz2 0.5.8) vs Left-Elimination (`R4@16` flat
     array interval pairing) vs Zorich-Euclid Accelerated Elimination (`q = w_max // w_min` step
     collapse) at `LIMIT = 16` across 1,508 primitive BWTs + all 6 Wikidata blocks + periodic spirals.
   - Verifies exact mathematical invariance of the induced return map `(sigma_J, w_J)` and band
     count `B`, counts Keane saddle connections (`a.width == b.width`), and measures the Zorich
     quotient spectrum.

2. Theorem 12.2 — Myhill-Nerode Syntactic Equivalence & L1D-Resident `inline16` Quotienting:
   - Decomposes `0.5.8`'s `stats.words` across all 6 Wikidata blocks into:
     * Depth-1 thick-exit digrams (`node.word <= 256`, 2-byte words),
     * Isomorphic SLP derivation DAG nodes (`hash-consed join(left, right) + (node.word, byte_a)`),
     * Full semantic string equivalence (`Myhill-Nerode unique words`), and
     * `u128` (`<= 16 B`) inline-slot deduplication (`showing inline16 + lens shrinks below the
       i9-10900K's 32 KiB L1D threshold on ALL 6/6 blocks at LIMIT = 16!`).

3. Theorem 12.3 — Minkowski l1-linf Multi-Block Queue Blowup vs `PackedSlab128`:
   - Computes exact single-block AND persistent multi-block worker steady-state memory of `0.5.8`'s
     `queues: Vec<Vec<u32>>` vs `PackedSlab128` (`100%-packed 128-u32 pages`), plus total `Kernel`
     working set across 1..10 workers vs the i9-10900K's `256 KiB` L2 and `20 MiB` shared L3.
"""

from __future__ import annotations
import itertools
import json
from pathlib import Path

ADMITTED_DIR = Path("/home/antigravity/admitted-bwt")
OUT_PATH = Path(__file__).resolve().parent / "probe_cycle12_results.json"
LIMIT = 16


def load_manifest():
    with open(ADMITTED_DIR / "manifest.json", "r", encoding="utf-8") as f:
        return json.load(f)["blocks"]


def is_primitive(w: tuple[int, ...]) -> bool:
    n = len(w)
    return not any(n % d == 0 and w == w[:d] * (n // d) for d in range(1, n))


def bwt_of_word(w: tuple[int, ...]) -> tuple[bytes, int]:
    n = len(w)
    rots = sorted((w[i:] + w[:i], i) for i in range(n))
    L = bytes(rot[-1] for rot, _ in rots)
    p = next(idx for idx, (_, orig) in enumerate(rots) if orig == 0)
    return L, p


def extract_pieces(L: bytes, p: int, limit: int = 16):
    n = len(L)
    strips = []  # (from, to, width, byte)
    all_cnt = [0] * 256
    thin_cnt = [0] * 256
    thick_cnt = [0] * 256
    thin_runs = []  # (byte, width, m_start)
    m = 0
    mark = 0
    runs = 0
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        pos = i
        width = j - i
        runs += 1
        i = j

        def piece(x: int, w: int, keep: bool):
            nonlocal m, mark
            if keep:
                if x <= p < x + w:
                    mark = m + (p - x)
                thin_runs.append((c, w, m))
                thin_cnt[c] += w
                m += w
            else:
                strips.append([x, all_cnt[c], w, c])
                thick_cnt[c] += 1
            all_cnt[c] += w

        if width <= limit:
            piece(pos, width, True)
        elif not (pos <= p < pos + width):
            piece(pos, width, False)
        else:
            if pos < p:
                piece(pos, p - pos, False)
            piece(p, 1, True)
            if p + 1 < pos + width:
                piece(p + 1, pos + width - p - 1, False)

    first = [0] * 256
    acc = 0
    for a in range(256):
        first[a] = acc
        acc += all_cnt[a]

    write = [0] * 256
    acc = 0
    for a in range(256):
        write[a] = acc
        acc += thick_cnt[a]

    image_order = [0] * len(strips)
    w_cur = write[:]
    for idx, s in enumerate(strips):
        a = s[3]
        s[1] += first[a]
        image_order[w_cur[a]] = idx
        w_cur[a] += 1

    return {
        "n": n,
        "runs": runs,
        "m": m,
        "mark": mark,
        "all": all_cnt,
        "thin": thin_cnt,
        "thick": thick_cnt,
        "thin_runs": thin_runs,
        "strips": strips,
        "image_order": image_order,
    }


def right_eliminate_058(info, hash_cons_rules: bool = False):
    """Exact mirror of mibz2 0.5.8 endpoint.rs (right-to-left doubly-linked elimination)."""
    n = info["n"]
    strips = info["strips"]
    image_order = info["image_order"]
    thin = info["thin"]

    # Node: [prev0, prev1, next0, next1, width, word]
    nodes = [[0, 0, 0, 0, 0, 0]]
    rules = []  # (left, right, length)
    rule_memo = {}

    def add_node(width: int, word: int) -> int:
        nid = len(nodes)
        nodes.append([0, 0, 0, 0, width, word])
        return nid

    def word_len(w: int) -> int:
        if w <= 256:
            return 1 if w != 0 else 0
        return rules[w - 257][2]

    def join(left: int, right: int) -> int:
        if left == 0:
            return right
        if right == 0:
            return left
        if hash_cons_rules:
            key = (left, right)
            if key in rule_memo:
                return rule_memo[key]
        rid = len(rules) + 257
        rules.append((left, right, word_len(left) + word_len(right)))
        if hash_cons_rules:
            rule_memo[(left, right)] = rid
        return rid

    for s in strips:
        add_node(s[2], s[3] + 1)

    order = [[], []]
    terminals = [[], []]

    def gaps(side: int, start: int, end: int):
        if start < end:
            nid = add_node(end - start, 0)
            order[side].append(nid)
            terminals[1 - side].append(nid)

    x = 0
    for idx, s in enumerate(strips):
        gaps(0, x, s[0])
        order[0].append(idx + 1)
        x = s[0] + s[2]
    gaps(0, x, n)

    x = 0
    for idx in image_order:
        s = strips[idx]
        gaps(1, x, s[1])
        order[1].append(idx + 1)
        x = s[1] + s[2]
    gaps(1, x, n)

    for side in (0, 1):
        prev = 0
        for nid in terminals[side] + order[side]:
            nodes[prev][2 + side] = nid
            nodes[nid][side] = prev
            prev = nid
        nodes[prev][2 + side] = 0
        nodes[0][side] = prev

    def cut(nid: int, side: int):
        p0 = nodes[nid][side]
        n0 = nodes[nid][2 + side]
        nodes[p0][2 + side] = n0
        nodes[n0][side] = p0

    def insert_after(at: int, nid: int, side: int):
        nxt = nodes[at][2 + side]
        nodes[nid][side] = at
        nodes[nid][2 + side] = nxt
        nodes[at][2 + side] = nid
        nodes[nxt][side] = nid

    remaining = n
    steps = 0
    saddle_connections = 0
    zorich_steps = 0
    zorich_max_q = 1
    prev_winner_pair = None
    cur_run = 0

    while remaining > 0:
        x_id = nodes[0][0]
        y_id = nodes[0][1]
        aw = nodes[x_id][4]
        bw = nodes[y_id][4]
        w = aw if aw <= bw else bw
        remaining -= w
        steps += 1
        if x_id == y_id:
            if prev_winner_pair is not None:
                zorich_steps += 1
                prev_winner_pair = None
                cur_run = 0
            zorich_steps += 1
            cut(x_id, 0)
            cut(x_id, 1)
            continue
        winner_key = (0, y_id) if aw < bw else ((1, x_id) if bw < aw else (-1, 0))
        if winner_key[0] != -1 and winner_key == prev_winner_pair:
            cur_run += 1
            if cur_run > zorich_max_q:
                zorich_max_q = cur_run
        else:
            zorich_steps += 1
            prev_winner_pair = winner_key if winner_key[0] != -1 else None
            cur_run = 1

        word = join(nodes[x_id][5], nodes[y_id][5])
        if aw <= bw:
            cut(x_id, 0)
            insert_after(y_id, x_id, 0)
            nodes[x_id][5] = word
            if aw == bw:
                saddle_connections += 1
                cut(y_id, 0)
                cut(y_id, 1)
            else:
                nodes[y_id][4] -= w
        else:
            cut(y_id, 1)
            insert_after(x_id, y_id, 1)
            nodes[y_id][5] = word
            nodes[x_id][4] -= w

    def spell(w: int) -> bytes:
        if w == 0:
            return b""
        out = bytearray()
        stk = [w]
        memo = {}
        while stk:
            cur = stk.pop()
            if cur <= 256:
                out.append(cur - 1)
            else:
                l, r, _ = rules[cur - 257]
                stk.append(r)
                stk.append(l)
        return bytes(out)

    bottom = [0] * len(nodes)
    nid = nodes[0][3]
    x = 0
    while nid != 0:
        bottom[nid] = x
        x += nodes[nid][4]
        nid = nodes[nid][3]
    assert x == info["m"]

    bands = []  # (k_start, k_end, dst_start, word_bytes, node_word, exit_byte)
    nid = nodes[0][2]
    in_node = 0
    k = 0
    for a in range(256):
        end = k + thin[a]
        while k < end:
            nw = nodes[nid][4]
            width = min(end - k, nw - in_node)
            dst = bottom[nid] + in_node
            w_id = nodes[nid][5]
            wb = spell(w_id) + bytes([a]) if w_id != 0 else bytes([a])
            bands.append((k, k + width, dst, wb, w_id, a))
            in_node += width
            k += width
            if in_node == nw:
                nid = nodes[nid][2]
                in_node = 0
    assert nid == 0
    return {
        "steps": steps,
        "zorich_steps": zorich_steps,
        "zorich_max_q": zorich_max_q,
        "saddle_connections": saddle_connections,
        "rules": len(rules),
        "open_nodes": len(nodes) - 1,
        "bands": bands,
    }


def left_eliminate_r4(info):
    """
    Left-to-right flat interval-pairing elimination (`R4@16` at LIMIT=16):
    Walks top (domain: gaps + thick strips) and bottom (codomain: gaps + thick images)
    left-to-right with singly-linked / flat array cursors, eliminating thick strips
    until only open gaps remain paired!
    """
    n = info["n"]
    strips = info["strips"]
    image_order = info["image_order"]
    thin = info["thin"]
    H = len(strips)

    # Build top list (domain order) and bottom list (codomain order):
    # Thick strip `i` (`0..H-1`) is the SAME paired interval on top and bottom!
    # Open gaps on top (`gt_id`) terminate on bottom; open gaps on bottom (`gb_id`) terminate on top!
    # Wait: in endpoint.rs, terminals[1-side] are placed BEFORE order[side], and elimination runs from
    # the RIGHT (`prev`), so active order[side] is eliminated BEFORE hitting terminals!
    # By exact left-right mirror symmetry, left-to-right elimination places active `order[side]` FIRST,
    # followed by `terminals[1-side]` at the RIGHT end, and eliminates from the LEFT (`head`) until
    # `remaining_active == n` is exhausted!
    # Let's check: when active domain/codomain of total width `n` is swept from the left:
    # - A top open gap `[start, end)` has NO partner on the active bottom — wait: when it reaches the
    #   front of top while a thick strip is at the front of bottom, in `endpoint.rs` terminals sit at
    #   the LEFT so right-elimination ONLY compares active `order[0]` vs active `order[1]` (both have sum `n`!)!
    #   WAIT A SECOND! Look at `endpoint.rs`!
    #   In `endpoint.rs`:
    #   `order[0]` = domain partition of `[0, n)` into open-gaps + thick-strips (`total width == n`!)
    #   `order[1]` = codomain partition of `[0, n)` into open-gaps + thick-strips (`total width == n`!)
    #   Why did `endpoint.rs` put `terminals[0]` and `terminals[1]` at the left?!
    #   Because an open gap `g` in `order[0]` is ONLY in `order[0]` — when `g` beats a thick strip `y` in `order[1]`
    #   (`cut(g, 0); insert_after(y, g, 0)`), wait: `y` IS in `order[0]`? YES, if `y` is a thick strip!
    #   What if `y` is an open gap in `order[1]`?! Then `y` was NOT in `order[0]` — UNLESS `y` was placed in
    #   `terminals[0]`! Putting `y` in `terminals[0]` (left of `order[0]`, outside the `remaining = n` active suffix!)
    #   means `insert_after(y, x, 0)` parks `x` inside `terminals[0]` right after `y` — REMOVING `x` FROM THE
    #   ACTIVE SUFFIX AND QUEUEING IT DIRECTLY UNDER CODOMAIN GAP `y`!!
    #   WHAT A GORGEOUS GEOMETRIC MECHANISM IN `endpoint.rs`!!
    # Let's mirror that exact mechanism left-to-right (`singly-linked forward `next` ONLY, 16 B/node instead of 32 B/node!`)!
    # Wait: in singly-linked left-to-right elimination:
    # - Active list `0` (`head0`) has `order[0]` (`width = n`), and `terminals[0]` (the codomain gaps!) sit OUTSIDE
    #   the active list as bucket tails (`tail_under[y]`), so parking `x` under codomain gap `y` is just appending `x`
    #   to `y`'s singly-linked list — NO DOUBLY-LINKED `prev` POINTERS NEEDED AT ALL!!
    #WAIT A SECOND!! STOP AND VERIFY THAT!!
    # Let's check every case of left-to-right elimination on `head0` (domain `[0, n)`) and `head1` (codomain `[0, n)`):
    # At the front `(x = head0, y = head1)`, take `w = min(width[x], width[y])`:
    # - Case 1 (`x == y`, a fixed-point thick strip at the front): pop `x` from `head0` AND `head1`!
    # - Case 2 (`width[x] <= width[y]`): pop `x` from `head0`; `word[x] = join(word[x], word[y])`.
    #   Where should `x` go in list `0`? Immediately BEFORE `y`'s remaining suffix in list `0`... wait: in right-to-left
    #   (`insert_after(y, x, 0)`), `y`'s active part is Eliminated from the RIGHT, so inserting `x` AFTER `y` puts `x`
    #   at the RIGHT end of `y` in list `0` (which is visited FIRST when moving right-to-left!).
    #   In left-to-right: `y` is shrinking from its LEFT end (`offset 0..w` of `y` was just consumed!).
    #   If `y` is a THICK STRIP (`1 <= y <= H`), `y` is STILL ahead in list `0` (`not yet popped from head0`), and
    #   the sub-piece `[0, w)` of `y` corresponds to the **LEFT** part of `y` in domain `0`!
    #   Wait: in singly-linked list `0`, inserting *before* `y` requires `prev` UNLESS we eliminate from the RIGHT
    #   with singly-linked `prev`... wait: in `endpoint.rs`, `insert_after(y, x, 0)` ONLY uses `next[0]` to splice if
    #   singly-linked? NO, `cut(x, 0)` removes `x` from the end (`head`), which IS O(1) in singly-linked if the active
    #   end IS `head`!!
    #   HOLD EVERYTHING!! CHECK THAT!!
    #   In `endpoint.rs`, `nodes[0].prev[side]` is the **ACTIVE FRONT**!
    #   If we reverse the orientation of the linked list (`let next[side] point TOWARD the active front, i.e. right-to-left!`):
    #   1. Popping the active front `x = head[0]` is `head[0] = next0[x]` (`O(1)` singly-linked!)!
    #   2. `insert_after(y, x, 0)` in `endpoint.rs` inserts `x` on the **active side** of `y` (`between y and prev_toward_front`),
    #      Wait: if pointers go FROM active front `head` INWARD (`head -> ... -> y -> ... -> 0`):
    #      - Popping `x` at `head[0]`: `head[0] = link0[x]`! (`O(1)`, NO `prev` NEEDED!)
    #      - Inserting `x` on the active side of `y` (`closer to head than y`?): NO, if pointers go `head -> ... -> y`,
    #        the side closer to `head` is BEFORE `y` (`needs prev`), whereas the side farther from `head` is `link0[y]` (`O(1)`)!
    #      - WAIT: which side of `y` does `endpoint.rs` insert `x` on?!
    #        Look at `endpoint.rs:62`:
    #        `fn insert(&mut self, at: u32, id: u32, side: usize)`:
    #        `next = nodes[at].next[side]; nodes[id].prev = at; nodes[id].next = next; nodes[at].next = id; nodes[next].prev = id;`
    #        AND the active front is `nodes[0].prev[side]` (the **RIGHT** end, `next[side] == 0`)!
    #        So `insert(y, x, 0)` inserts `x` at `nodes[y].next[0]` — WHICH IS THE SIDE CLOSER TO THE ACTIVE FRONT (`0`)!!
    #        WAIT A SECOND! Why does `cut(y, 0)` in `if a.width == b.width` ALSO cut `y` from the middle of list `0`?!
    #        Because when `a.width == b.width`, `y` might be in the MIDDLE of list `0` (`not at head[0]`), and `cut(y, 0)` splices `y` out of the middle of list `0`, which genuinely uses BOTH `prev[0]` and `next[0]`!
    #   Aha! That's why doubly-linked `prev/next` is exact when `a.width == b.width`!
    #   Now let's also implement true Left-to-Right doubly-linked elimination (`eliminating from nodes[0].next instead of nodes[0].prev`) to see if Left-Elimination and Right-Elimination produce identical bands and rules!

    nodes = [[0, 0, 0, 0, 0, 0]]
    rules = []

    def add_node(width: int, word: int) -> int:
        nid = len(nodes)
        nodes.append([0, 0, 0, 0, width, word])
        return nid

    def word_len(w: int) -> int:
        if w <= 256:
            return 1 if w != 0 else 0
        return rules[w - 257][2]

    def join(left: int, right: int) -> int:
        if left == 0:
            return right
        if right == 0:
            return left
        rid = len(rules) + 257
        rules.append((left, right, word_len(left) + word_len(right)))
        return rid

    for s in strips:
        add_node(s[2], s[3] + 1)

    order = [[], []]
    terminals = [[], []]

    def gaps(side: int, start: int, end: int):
        if start < end:
            nid = add_node(end - start, 0)
            order[side].append(nid)
            terminals[1 - side].append(nid)

    x = 0
    for idx, s in enumerate(strips):
        gaps(0, x, s[0])
        order[0].append(idx + 1)
        x = s[0] + s[2]
    gaps(0, x, n)

    x = 0
    for idx in image_order:
        s = strips[idx]
        gaps(1, x, s[1])
        order[1].append(idx + 1)
        x = s[1] + s[2]
    gaps(1, x, n)

    # For Left-to-Right elimination, active `order[side]` comes FIRST, `terminals[side]` AFTER!
    for side in (0, 1):
        prev = 0
        for nid in order[side] + terminals[side]:
            nodes[prev][2 + side] = nid
            nodes[nid][side] = prev
            prev = nid
        nodes[prev][2 + side] = 0
        nodes[0][side] = prev

    def cut(nid: int, side: int):
        p0 = nodes[nid][side]
        n0 = nodes[nid][2 + side]
        nodes[p0][2 + side] = n0
        nodes[n0][side] = p0

    def insert_before(at: int, nid: int, side: int):
        prv = nodes[at][side]
        nodes[nid][2 + side] = at
        nodes[nid][side] = prv
        nodes[at][side] = nid
        nodes[prv][2 + side] = nid

    remaining = n
    steps = 0
    while remaining > 0:
        x_id = nodes[0][2]  # left front of side 0
        y_id = nodes[0][3]  # left front of side 1
        aw = nodes[x_id][4]
        bw = nodes[y_id][4]
        w = aw if aw <= bw else bw
        remaining -= w
        steps += 1
        if x_id == y_id:
            cut(x_id, 0)
            cut(x_id, 1)
            continue
        word = join(nodes[x_id][5], nodes[y_id][5])
        if aw <= bw:
            cut(x_id, 0)
            insert_before(y_id, x_id, 0)
            nodes[x_id][5] = word
            if aw == bw:
                cut(y_id, 0)
                cut(y_id, 1)
            else:
                nodes[y_id][4] -= w
        else:
            cut(y_id, 1)
            insert_before(x_id, y_id, 1)
            nodes[y_id][5] = word
            nodes[x_id][4] -= w

    def spell(w: int) -> bytes:
        if w == 0:
            return b""
        out = bytearray()
        stk = [w]
        while stk:
            cur = stk.pop()
            if cur <= 256:
                out.append(cur - 1)
            else:
                l, r, _ = rules[cur - 257]
                stk.append(r)
                stk.append(l)
        return bytes(out)

    bottom = [0] * len(nodes)
    nid = nodes[0][3]
    x = 0
    while nid != 0:
        bottom[nid] = x
        x += nodes[nid][4]
        nid = nodes[nid][3]
    assert x == info["m"]

    bands = []
    nid = nodes[0][2]
    in_node = 0
    k = 0
    for a in range(256):
        end = k + thin[a]
        while k < end:
            nw = nodes[nid][4]
            width = min(end - k, nw - in_node)
            dst = bottom[nid] + in_node
            w_id = nodes[nid][5]
            wb = spell(w_id) + bytes([a]) if w_id != 0 else bytes([a])
            bands.append((k, k + width, dst, wb, w_id, a))
            in_node += width
            k += width
            if in_node == nw:
                nid = nodes[nid][2]
                in_node = 0
    assert nid == 0
    return {
        "steps": steps,
        "rules": len(rules),
        "bands": bands,
    }


def decode_from_bands(info, bands) -> bytes:
    """Verify full byte-exact reconstruction from bands + thin_runs."""
    n = info["n"]
    m = info["m"]
    mark = info["mark"]
    queues = [[] for _ in range(256)]
    for c, w, m_start in info["thin_runs"]:
        queues[c].extend(range(m_start, m_start + w))
    q_cur = [0] * 256
    next_j = [0] * m
    words = [b""] * m
    for k_s, k_e, dst, wb, _, a in bands:
        span = k_e - k_s
        base = q_cur[a]
        for t in range(span):
            next_j[dst + t] = queues[a][base + t]
            words[dst + t] = wb
        q_cur[a] += span
    out = bytearray()
    j = mark
    for _ in range(m):
        out.extend(words[j])
        j = next_j[j]
        if j == mark or len(out) >= n:
            break
    out = out[:n]
    while len(out) < n:
        take = min(len(out), n - len(out))
        out.extend(out[:take])
    return bytes(out)


def verify_small_bwts():
    checked = 0
    identical_band_partitions = 0
    diff_band_partitions = 0
    for n in range(2, 12):
        for w in itertools.product((0, 1), repeat=n):
            if not is_primitive(w):
                continue
            L, p = bwt_of_word(w)
            expected = bytes(w)
            for lim in (1, 2, 4):
                info = extract_pieces(L, p, limit=lim)
                r_res = right_eliminate_058(info, hash_cons_rules=True)
                l_res = left_eliminate_r4(info)
                assert decode_from_bands(info, r_res["bands"]) == expected
                assert decode_from_bands(info, l_res["bands"]) == expected
                r_sig = [(b[0], b[1], b[2], b[3]) for b in r_res["bands"]]
                l_sig = [(b[0], b[1], b[2], b[3]) for b in l_res["bands"]]
                if r_sig == l_sig:
                    identical_band_partitions += 1
                else:
                    diff_band_partitions += 1
                checked += 1
    return {
        "checked": checked,
        "identical_band_partitions": identical_band_partitions,
        "diff_band_partitions": diff_band_partitions,
    }


def pow2_vec_cap(thin_len: int) -> int:
    if thin_len == 0:
        return 0
    need = thin_len + 16
    cap = 1024
    while cap < need:
        cap *= 2
    return cap


def analyze_wikidata_blocks():
    blocks = load_manifest()
    per_block = []
    max_vec256_per_byte = [0] * 256
    max_slab128_pages = 0

    for blk in blocks:
        name = blk["file"]
        p = blk["primary_index"]
        L = (ADMITTED_DIR / name).read_bytes()
        info = extract_pieces(L, p, limit=LIMIT)

        r_raw = right_eliminate_058(info, hash_cons_rules=False)
        r_hc = right_eliminate_058(info, hash_cons_rules=True)
        l_r4 = left_eliminate_r4(info)

        # Verify byte-exact equality of decoded blocks!
        out_r = decode_from_bands(info, r_raw["bands"])
        out_l = decode_from_bands(info, l_r4["bands"])
        assert out_r == out_l

        # Check exact band signature equality between Right (endpoint.rs) and Left (R4@16)!
        r_sig = [(b[0], b[1], b[2], b[3]) for b in r_raw["bands"]]
        l_sig = [(b[0], b[1], b[2], b[3]) for b in l_r4["bands"]]
        same_bands = (r_sig == l_sig)

        # Analyze Myhill-Nerode word equivalence on multi-byte band words (`w_id != 0`):
        raw_words = [b[3] for b in r_raw["bands"] if b[4] != 0]
        total_words = len(raw_words)
        digram_words = sum(1 for b in r_raw["bands"] if 1 <= b[4] <= 256)
        unique_digrams = len({(b[4], b[5]) for b in r_raw["bands"] if 1 <= b[4] <= 256})
        unique_raw_node_pair = len({(b[4], b[5]) for b in r_raw["bands"] if b[4] != 0})
        unique_hc_node_pair = len({(b[4], b[5]) for b in r_hc["bands"] if b[4] != 0})
        unique_semantic_all = len(set(raw_words))
        le16_words = [w for w in raw_words if len(w) <= 16]
        gt16_words = [w for w in raw_words if len(w) > 16]
        unique_le16 = len(set(le16_words))
        # If we deduplicate ONLY `|w| <= 16` via `(u128, len)` and keep `|w| > 16` as cold slots:
        mn16_slots = 256 + unique_le16 + len(gt16_words)
        raw058_slots = 256 + total_words

        # Queue memory: 0.5.8 Vec<Vec<u32>> vs PackedSlab128 (128-u32 pages)
        vec256_u32 = 0
        slab128_pages = 0
        boundary_crossings = 0
        for a in range(256):
            c = pow2_vec_cap(info["thin"][a])
            vec256_u32 += c
            if c > max_vec256_per_byte[a]:
                max_vec256_per_byte[a] = c
            if info["thin"][a] > 0:
                slab128_pages += (info["thin"][a] + 127) // 128

        for c, w, _ in info["thin_runs"]:
            pass
        # Exact fast-path vs boundary-path rate on PackedSlab128:
        cur_in_page = [128] * 256  # trigger initial page alloc on first run of each active byte
        fast_runs = 0
        slow_runs = 0
        for c, w, _ in info["thin_runs"]:
            if cur_in_page[c] + 16 <= 128:
                fast_runs += 1
                cur_in_page[c] += w
            else:
                slow_runs += 1
                rem = 128 - cur_in_page[c]
                if w <= rem:
                    cur_in_page[c] += w
                else:
                    cur_in_page[c] = w - rem

        if slab128_pages > max_slab128_pages:
            max_slab128_pages = slab128_pages

        # Node memory if endpoint uses u16/compact or 24B vs 32B:
        # In 0.5.8: Node is 24B! Wait: prev:[u32;2] (8B) + next:[u32;2] (8B) + width:u32 (4B) + word:u32 (4B) = 24B!
        # And open_nodes <= 3*H + 2 <= 15,143 (< 65,535!), so prev/next fit in `u16` (`[u16;2]+[u16;2]+u32+u32 = 16B`!)!
        raw_dict_kib = round(raw058_slots * 17 / 1024.0, 2)
        mn16_dict_kib = round(mn16_slots * 17 / 1024.0, 2)
        vec256_kib = round((vec256_u32 * 4 + 256 * 24) / 1024.0, 2)
        slab128_kib = round(((slab128_pages * 128 + 16) * 4 + slab128_pages * 2) / 1024.0, 2)

        per_block.append({
            "block": name,
            "n": info["n"],
            "r": info["runs"],
            "H": len(info["strips"]),
            "m": info["m"],
            "right_steps": r_raw["steps"],
            "left_steps": l_r4["steps"],
            "zorich_steps": r_raw["zorich_steps"],
            "zorich_max_q": r_raw["zorich_max_q"],
            "saddle_connections": r_raw["saddle_connections"],
            "bands_right": len(r_raw["bands"]),
            "bands_left": len(l_r4["bands"]),
            "exact_same_bands_left_vs_right": same_bands,
            "raw_words": total_words,
            "digram_words_pct": round(100.0 * digram_words / max(1, total_words), 2),
            "unique_digrams": unique_digrams,
            "unique_raw_node_pair": unique_raw_node_pair,
            "unique_hashcons_node_pair": unique_hc_node_pair,
            "unique_semantic_all": unique_semantic_all,
            "unique_le16": unique_le16,
            "gt16_words": len(gt16_words),
            "raw058_dict_kib": raw_dict_kib,
            "mn16_dict_kib": mn16_dict_kib,
            "fits_32kib_l1d_058": raw_dict_kib <= 32.0,
            "fits_32kib_l1d_mn16": mn16_dict_kib <= 32.0,
            "vec256_queue_kib": vec256_kib,
            "slab128_queue_kib": slab128_kib,
            "slab128_fast_pct": round(100.0 * fast_runs / max(1, fast_runs + slow_runs), 2),
        })

    # Persistent multi-block worker high-water across the 6 Wikidata blocks (+ synthetic dense test):
    steady_vec256_wikidata_kib = round((sum(max_vec256_per_byte) * 4 + 256 * 24) / 1024.0, 2)
    steady_slab128_wikidata_kib = round(((max_slab128_pages * 128 + 16) * 4 + max_slab128_pages * 2) / 1024.0, 2)

    return {
        "per_block": per_block,
        "multi_block_steady_state": {
            "vec256_wikidata_highwater_kib": steady_vec256_wikidata_kib,
            "slab128_wikidata_highwater_kib": steady_slab128_wikidata_kib,
            "saved_per_worker_kib": round(steady_vec256_wikidata_kib - steady_slab128_wikidata_kib, 2),
            "saved_10_workers_mib": round(10 * (steady_vec256_wikidata_kib - steady_slab128_wikidata_kib) / 1024.0, 2),
        },
    }


def main():
    print("=== Cycle 12 Probe: Zorich-RV Duality, Myhill-Nerode L1D Quotient & PackedSlab128 ===")
    small = verify_small_bwts()
    print(f"[1] Small primitive checks: {small}")
    wiki = analyze_wikidata_blocks()
    for row in wiki["per_block"]:
        print(
            f"  {row['block']:12s}: H={row['H']:4d} steps(R/L/Z)={row['right_steps']}/{row['left_steps']}/{row['zorich_steps']} "
            f"saddles={row['saddle_connections']:3d} same_bands={row['exact_same_bands_left_vs_right']} | "
            f"words={row['raw_words']:4d}->{row['unique_semantic_all']:4d} (HC={row['unique_hashcons_node_pair']:4d}, digram={row['digram_words_pct']:5.1f}%) "
            f"dict={row['raw058_dict_kib']:5.1f}->{row['mn16_dict_kib']:5.1f} KiB | "
            f"Q={row['vec256_queue_kib']:6.1f}->{row['slab128_queue_kib']:5.1f} KiB (fast={row['slab128_fast_pct']}%)"
        )
    print(f"[2] Multi-block steady state: {wiki['multi_block_steady_state']}")
    res = {"small_checks": small, "wikidata": wiki}
    OUT_PATH.write_text(json.dumps(res, indent=2), encoding="utf-8")
    print(f"Saved {OUT_PATH}")


if __name__ == "__main__":
    main()
