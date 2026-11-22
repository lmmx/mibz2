#!/usr/bin/env python3
"""
Investigator Zeta — Quantitative Cross-Block Reuse Census for exact inverse-BWT (mibz2).
Verifies SHA-256 of the 6 authenticated blocks in /tmp/mibz2-blocks/, decodes them exactly,
runs Open's exact LIMIT=16 decomposition, and measures cross-block overlap across:
  1. Open's band/excursion words & spelling table
  2. Thick strips & RV elimination DAG sub-words
  3. Thin/retained-row tokens (GUIDs, sha1, entity IDs, JSON keys) & cross-block substring matches
  4. BWT/LF coordinate & shift stability
  5. Strictly adjacent 450 KB slices (re-BWT'd) to measure delta=1 adjacency.
"""

import hashlib
import json
import os
import re
import time
from collections import Counter, defaultdict

BLOCKS_DIR = "/tmp/mibz2-blocks"
LIMIT = 16


def verify_and_load_blocks():
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)
    blocks = []
    for entry in manifest["blocks"]:
        path = os.path.join(BLOCKS_DIR, entry["file"])
        with open(path, "rb") as bf:
            data = bf.read()
        sha = hashlib.sha256(data).hexdigest()
        assert sha == entry["sha256"], f"SHA-256 mismatch for {entry['file']}"
        assert len(data) == entry["n"], f"Length mismatch for {entry['file']}"
        blocks.append({
            "name": entry["file"].replace(".L", ""),
            "file": entry["file"],
            "n": entry["n"],
            "p": entry["primary_index"],
            "r_manifest": entry["r"],
            "window": entry["provenance"]["window"],
            "block_in_window": entry["provenance"]["block_in_window"],
            "block_in_corpus": entry["provenance"]["block_in_corpus"],
            "L": data,
        })
    # Sort by block_in_corpus for logical ordering
    blocks.sort(key=lambda b: b["block_in_corpus"])
    return blocks


def rle_maximal_runs(L: bytes):
    """Return list of (byte, pos, width) for maximal runs of L."""
    n = len(L)
    if n == 0:
        return []
    runs = []
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        runs.append((c, i, j - i))
        i = j
    return runs


def decode_bwt_and_row_map(L: bytes, p: int):
    """
    Exact inverse BWT from L and primary index p.
    In bzip2 / mibz2:
      LF(i) = C[L[i]] + rank(L[i], i)
      Wait: verify direction! Let's match src/open.rs!
    In src/open.rs:
      `emit` walks the marked row p using forward-order `edges`:
      Wait: in mibz2, T[0] = F[p], and walking forward (Phi = LF^{-1}) from p gives T[0], T[1], ...
      Equivalently, walking backward (LF) from p gives T[n-1], T[n-2], ..., T[0].
      Let's check: in open.rs, `trace` accumulates `t.symbol` (which is `s.byte`, i.e., L[x])
      along LF, then REVERSES it, then appends `c` (the byte of the F-interval / exit!).
      So walking `edges` from `mark` emits `T[0], T[1], ..., T[n-1]`!
      And `T[n - 1 - step] = L[LF^step(p)]`!
      Let's verify this by checking that our decoded text is valid JSON / matches Open's exact emit!
    """
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    tot = 0
    for c in range(256):
        C[c] = tot
        tot += counts[c]

    # Build LF array
    occ = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = occ[b]
        occ[b] += 1

    # Walk backward from p: step k=0..n-1 puts L[row] at text pos n-1-k?
    # Wait: let's check both conventions on block-0!
    # In bzip2: row p is the original string T[0..n].
    # So L[p] = T[n-1], F[p] = T[0].
    # LF(p) is the rotation T[n-1] T[0..n-2], so L[LF(p)] = T[n-2], F[LF(p)] = T[n-1] = L[p].
    # Thus text[n - 1 - k] = L[LF^k(p)], and row_of_text_pos[n - 1 - k] = LF^k(p) (the row whose L-byte is text[n-1-k],
    # i.e. whose F-byte is text[(n - k) % n]).
    text = bytearray(n)
    row_at_L = [0] * n  # row_at_L[pos] = row i such that L[i] produces text[pos]
    row = p
    for k in range(n):
        pos = n - 1 - k
        text[pos] = L[row]
        row_at_L[pos] = row
        row = LF[row]
    cycle_len = n
    if row != p:
        # Handle short cycle if any (not expected for real blocks, but be exact)
        pass
    return bytes(text), C, counts, LF, row_at_L


def run_open_exact(L: bytes, p: int, limit: int = LIMIT):
    """
    Exact faithful mirror of src/open.rs + src/open/endpoint.rs (INDUCE=true, LIMIT=16).
    Returns rich diagnostics:
      - decoded bytes (verified == exact BWT decode)
      - strips: list of (from, to, width, byte)
      - bands: list of (k_start, k_end, width, dst, code, word_bytes)
      - words: list of non-literal Open words (code >= 256), each as bytes
      - word_emit_counts: Counter of how many times each word (including literals) is emitted during walk
      - retained_rows: list of original BWT row indices that are retained (length m)
      - retained_word: list of Open word (bytes) emitted at each retained rank j in 0..m
      - rule_words: set of all strings produced by internal SLP rules in endpoint elimination
      - spelling_bytes: total len(spelling)
    """
    n = len(L)
    runs = rle_maximal_runs(L)

    # First pass: split maximal runs into thin (kept) and thick strips
    all_cnt = [0] * 256
    thin_cnt = [0] * 256
    thick_cnt = [0] * 256
    strips = []  # (from, to_rel, width, byte)
    thin_pieces = []  # (byte, x, w, m_start)
    m = 0
    mark = 0
    retained_orig_rows = []  # maps retained rank j -> original row index in 0..n

    def take_piece(c, x, w, keep):
        nonlocal m, mark
        if keep:
            if x <= p < x + w:
                mark = m + (p - x)
            thin_pieces.append((c, x, w, m))
            for r in range(x, x + w):
                retained_orig_rows.append(r)
            thin_cnt[c] += w
            m += w
        else:
            strips.append([x, all_cnt[c], w, c])
            thick_cnt[c] += 1
        all_cnt[c] += w

    for c, pos, width in runs:
        end = pos + width
        if width > limit and pos <= p < end:
            if pos < p:
                take_piece(c, pos, p - pos, False)
            take_piece(c, p, 1, True)
            if p + 1 < end:
                take_piece(c, p + 1, end - p - 1, False)
        else:
            take_piece(c, pos, width, width <= limit)

    # Starts
    first = [0] * 256
    tot = 0
    for a in range(256):
        first[a] = tot
        tot += all_cnt[a]

    write = [0] * 256
    tot = 0
    for a in range(256):
        write[a] = tot
        tot += thick_cnt[a]

    ns = len(strips)
    image_order = [0] * ns
    for i, s in enumerate(strips):
        a = s[3]
        s[1] += first[a]  # s.to
        image_order[write[a]] = i
        write[a] += 1

    # Endpoint elimination (exact mirror of src/open/endpoint.rs)
    # Nodes: prev0, next0, prev1, next1, width, word
    prev0 = [0]
    next0 = [0]
    prev1 = [0]
    next1 = [0]
    n_width = [0]
    n_word = [0]

    def add_node(w, wd):
        nid = len(n_width)
        prev0.append(0)
        next0.append(0)
        prev1.append(0)
        next1.append(0)
        n_width.append(w)
        n_word.append(wd)
        return nid

    for s in strips:
        add_node(s[2], s[3] + 1)

    term0 = [0, 0]
    ord0 = [0, 0]
    term1 = [0, 0]
    ord1 = [0, 0]

    def append_to(lst, nid, side):
        if lst[1] == 0:
            lst[0] = nid
        else:
            if side == 0:
                next0[lst[1]] = nid
            else:
                next1[lst[1]] = nid
        if side == 0:
            prev0[nid] = lst[1]
        else:
            prev1[nid] = lst[1]
        lst[1] = nid

    x = 0
    for i, s in enumerate(strips):
        s_from, s_to, s_w, s_c = s
        if x < s_from:
            nid = add_node(s_from - x, 0)
            append_to(ord0, nid, 0)
            append_to(term1, nid, 1)
        append_to(ord0, i + 1, 0)
        x = s_from + s_w
    if x < n:
        nid = add_node(n - x, 0)
        append_to(ord0, nid, 0)
        append_to(term1, nid, 1)

    x = 0
    for idx in image_order:
        s_from, s_to, s_w, s_c = strips[idx]
        if x < s_to:
            nid = add_node(s_to - x, 0)
            append_to(ord1, nid, 1)
            append_to(term0, nid, 0)
        append_to(ord1, idx + 1, 1)
        x = s_to + s_w
    if x < n:
        nid = add_node(n - x, 0)
        append_to(ord1, nid, 1)
        append_to(term0, nid, 0)

    def close_side(side, first_l, second_l):
        p_id = 0
        for head, tail in (first_l, second_l):
            if tail == 0:
                continue
            if side == 0:
                next0[p_id] = head
                prev0[head] = p_id
            else:
                next1[p_id] = head
                prev1[head] = p_id
            p_id = tail
        if side == 0:
            next0[p_id] = 0
            prev0[0] = p_id
        else:
            next1[p_id] = 0
            prev1[0] = p_id

    close_side(0, term0, ord0)
    close_side(1, term1, ord1)

    # Rules: list of (left, right, length)
    rules = []

    def wlen(wd):
        if wd <= 256:
            return 1 if wd != 0 else 0
        return rules[wd - 257][2]

    def join(l, r):
        if l == 0:
            return r
        if r == 0:
            return l
        rid = len(rules) + 257
        rules.append((l, r, wlen(l) + wlen(r)))
        return rid

    def cut(nid, side):
        if side == 0:
            pr, nx = prev0[nid], next0[nid]
            next0[pr] = nx
            prev0[nx] = pr
        else:
            pr, nx = prev1[nid], next1[nid]
            next1[pr] = nx
            prev1[nx] = pr

    def insert_after(at, nid, side):
        if side == 0:
            nx = next0[at]
            prev0[nid] = at
            next0[nid] = nx
            next0[at] = nid
            prev0[nx] = nid
        else:
            nx = next1[at]
            prev1[nid] = at
            next1[nid] = nx
            next1[at] = nid
            prev1[nx] = nid

    remaining = n
    steps = 0
    while remaining > 0:
        xn = prev0[0]
        yn = prev1[0]
        aw = n_width[xn]
        bw = n_width[yn]
        w = aw if aw < bw else bw
        remaining -= w
        steps += 1
        if xn == yn:
            cut(xn, 0)
            cut(xn, 1)
            continue
        wd = join(n_word[xn], n_word[yn])
        if aw <= bw:
            cut(xn, 0)
            insert_after(yn, xn, 0)
            n_word[xn] = wd
            if aw == bw:
                cut(yn, 0)
                cut(yn, 1)
            else:
                n_width[yn] = bw - w
        else:
            cut(yn, 1)
            insert_after(xn, yn, 1)
            n_word[yn] = wd
            n_width[xn] = aw - w

    # Compute bottom coords
    bottom = [0] * len(n_width)
    nid = next1[0]
    x = 0
    while nid != 0:
        bottom[nid] = x
        x += n_width[nid]
        nid = next1[nid]
    assert x == m

    # Memoized rule expansion
    memo = {}

    def expand_word(wd):
        if wd == 0:
            return b""
        if wd <= 256:
            return bytes([wd - 1])
        if wd in memo:
            return memo[wd]
        # iterative expansion to avoid recursion depth issues
        stack = [wd]
        order = []
        seen = set()
        while stack:
            cur = stack.pop()
            if cur <= 256 or cur in memo or cur in seen:
                continue
            seen.add(cur)
            order.append(cur)
            l, r, _ = rules[cur - 257]
            if l > 256 and l not in memo:
                stack.append(l)
            if r > 256 and r not in memo:
                stack.append(r)
        # process in reverse topological order (children before parents)
        # simpler: since rule ids are strictly increasing (l < cur, r < cur), sorting order suffices!
        order.sort()
        for cur in order:
            l, r, _ = rules[cur - 257]
            lb = bytes([l - 1]) if l <= 256 else memo[l]
            rb = bytes([r - 1]) if r <= 256 else memo[r]
            memo[cur] = lb + rb
        return memo[wd]

    # Build bands & words
    cursors = [0] * 256
    bands = []  # (k_start, k_end, dst, code, word_bytes)
    words = []  # non-literal words (code = 256 + idx)
    spelling_bytes = 0

    nid = next0[0]
    in_node = 0
    k = 0
    for a in range(256):
        cursors[a] = len(bands)
        end = k + thin_cnt[a]
        while k < end:
            nw = n_width[nid]
            nwd = n_word[nid]
            w = min(end - k, nw - in_node)
            dst = bottom[nid] + in_node
            if nwd == 0:
                code = a
                wb = bytes([a])
            else:
                wb = expand_word(nwd) + bytes([a])
                code = 256 + len(words)
                words.append(wb)
                spelling_bytes += len(wb)
            bands.append((k, k + w, dst, code, wb))
            in_node += w
            k += w
            if in_node == nw:
                nid = next0[nid]
                in_node = 0
    assert nid == 0

    # Wire edges (exact mirror of non-queued path in open.rs, mathematically identical to QUEUE=true)
    stream = [0] * 256
    tot = 0
    for a in range(256):
        stream[a] = tot
        tot += thin_cnt[a]

    edges_next = [0] * m
    edges_code = [0] * m
    cur_band = cursors[:]
    j = 0
    for c, x, w, m_start in thin_pieces:
        z = stream[c]
        end = z + w
        while z < end:
            b_start, b_end, b_dst, b_code, _ = bands[cur_band[c]]
            span = min(end, b_end) - z
            dest = b_dst + (z - b_start)
            for off in range(span):
                edges_next[dest + off] = j + off
                edges_code[dest + off] = b_code
            j += span
            z += span
            if z == b_end:
                cur_band[c] += 1
        stream[c] = end

    # Emit walk from mark
    out = bytearray()
    j = mark
    word_emit_counts = Counter()
    retained_text_pos = [-1] * m  # start position in out where retained rank j emits its word
    retained_emitted_word = [None] * m
    emitted_steps = 0
    for _ in range(m):
        code = edges_code[j]
        nxt = edges_next[j]
        wb = bytes([code]) if code < 256 else words[code - 256]
        retained_text_pos[j] = len(out)
        retained_emitted_word[j] = wb
        word_emit_counts[wb] += 1
        out.extend(wb)
        emitted_steps += 1
        j = nxt
        if j == mark or len(out) >= n:
            break
    if len(out) < n:
        # repeat short cycle
        period = bytes(out)
        while len(out) < n:
            out.extend(period[: n - len(out)])
    out = bytes(out[:n])

    # Collect all internal SLP rule strings (up to reasonable cap, actually rules is small ~10k)
    # Wait: let's expand all rules that were used in bands (already in memo!)
    rule_words = set(memo.values())

    return {
        "out": out,
        "m": m,
        "mark": mark,
        "runs": len(runs),
        "strips": strips,
        "steps": steps,
        "num_rules": len(rules),
        "bands": bands,
        "words": words,  # list of non-literal words (length >= 2)
        "spelling_bytes": spelling_bytes,
        "word_emit_counts": word_emit_counts,
        "retained_orig_rows": retained_orig_rows,
        "retained_text_pos": retained_text_pos,
        "retained_emitted_word": retained_emitted_word,
        "edges_next": edges_next,
        "edges_code": edges_code,
        "rule_words": rule_words,
        "emitted_steps": emitted_steps,
    }


def classify_text_tokens(text: bytes):
    """
    Classify bytes and extract exact tokens in decoded Wikidata JSON (pre-RLE1).
    Returns:
      - byte_class: bytearray of class ids per byte position
      - pos_to_token: dict or list mapping byte pos -> (class_id, token_bytes) for named token classes
      - tokens_by_class: dict[class_name -> Counter of token bytes]
    Class IDs:
      0 = other/structure
      1 = GUID (e.g. Q123$xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx or bare UUID hex-with-dashes)
      2 = sha1 (40 lowercase hex chars in quotes or after hash/sha1)
      3 = entity_id (Q/P/L + digits)
      4 = json_key ("...":)
    """
    n = len(text)
    byte_class = bytearray(n)
    token_at = [None] * n  # (cls_name, token_bytes)
    tokens_by_class = defaultdict(Counter)

    # 1. GUIDs: statement IDs like q123$uuid or standard 8-4-4-4-12 hex UUIDs
    re_guid = re.compile(rb'[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}')
    for m in re_guid.finditer(text):
        tok = m.group(0)
        tokens_by_class["GUID"][tok] += 1
        s, e = m.span()
        for i in range(s, e):
            byte_class[i] = 1
            token_at[i] = ("GUID", tok)

    # 2. sha1: 40 hex chars (quoted, or standalone 40-hex not overlapping GUID)
    re_sha1 = re.compile(rb'"([0-9a-f]{40})"')
    for m in re_sha1.finditer(text):
        tok = m.group(1)
        s, e = m.span(1)
        if byte_class[s] == 0:
            tokens_by_class["sha1"][tok] += 1
            for i in range(s, e):
                byte_class[i] = 2
                token_at[i] = ("sha1", tok)

    # 3. Entity IDs: Q/P/L followed by digits
    re_ent = re.compile(rb'\b([QPL][0-9]{1,10})\b')
    for m in re_ent.finditer(text):
        tok = m.group(1)
        s, e = m.span(1)
        if byte_class[s] == 0:
            tokens_by_class["entity_id"][tok] += 1
            for i in range(s, e):
                byte_class[i] = 3
                token_at[i] = ("entity_id", tok)

    # 4. JSON keys: "key":
    re_key = re.compile(rb'"([^"\\]{1,64})":')
    for m in re_key.finditer(text):
        tok = m.group(1)
        s, e = m.span(1)
        if byte_class[s] == 0:
            tokens_by_class["json_key"][tok] += 1
            for i in range(s, e):
                if byte_class[i] == 0:
                    byte_class[i] = 4
                    token_at[i] = ("json_key", tok)

    return byte_class, token_at, tokens_by_class


def cross_block_match_lengths(text_A: bytes, text_B: bytes, k_min: int = 8):
    """
    For every position i in text_B, compute whether text_B[i:i+k] appears in text_A
    for k in [8, 12, 16, 24, 32, 40].
    Fast exact check using rolling/sliced hash sets of k-grams in text_A?
    Even better: build index of 8-grams in text_A? Wait: 900K bytes has ~900K 8-grams,
    a Python set of 900K bytes(8) uses ~100 MB and takes ~0.3s!
    Even cleaner: for each k in (8, 12, 16, 24, 32, 40), check membership at retained positions ONLY!
    Wait: retained positions in B are only ~100K-200K!
    Wait: to check if a retained byte at pos i belongs to ANY cross-block match of length >= k
    (even if i is in the middle of the match), we can compute for every position i in text_B
    the length of the longest match starting at i (or simply build sets of k-grams of text_A
    at step 1 for k=8,16,32 and mark covered intervals [i, i+k) in text_B).
    Let's do exact covered-byte marking for k in (8, 12, 16, 24, 32, 40)!
    """
    # Actually let's check both:
    # (a) exact k-gram starting at pos (right-context match of length k)
    # (b) covered by ANY k-gram in text_A (exact substring match covering byte pos)
    # To keep runtime super fast (~1-2s per pair), build k-gram set of A only for selected pairs!
    nB = len(text_B)
    results = {}
    for k in (8, 12, 16, 24, 32, 40):
        # Set of all k-grams in A
        grams_A = set()
        nA = len(text_A)
        for i in range(nA - k + 1):
            grams_A.add(text_A[i : i + k])
        covered = bytearray(nB)
        starts_match = bytearray(nB)
        last_end = 0
        for i in range(nB - k + 1):
            if text_B[i : i + k] in grams_A:
                starts_match[i] = 1
                s = i if i > last_end else last_end
                e = i + k
                if e > s:
                    covered[s:e] = b"\x01" * (e - s)
                    last_end = e
        results[k] = (starts_match, covered)
    return results


def build_bwt_from_text(text: bytes):
    """Build exact BWT (L, p) of text using fast prefix-doubling suffix sort (or Python sorted? Wait: 450KB!)."""
    # Wait: for 450 KB, we can use a tiny helper or suffix array in C/Rust, or Python suffix doubling?
    # Let's do a fast SA-IS or radix doubling in Python, OR even better, call a 30-line C helper or
    # Python's suffix doubling? Actually let's test how fast pure Python is or compile a tiny C SA-IS?
    # Even simpler: Python's Timsort on 225K indices with 32-byte keys + tiebreak!
    # For Wikidata JSON, almost all suffixes separate within 64-256 bytes, except long repeats.
    # Let's do exact prefix doubling (Manber-Myers with Python sort) - takes ~0.5s for 450KB!
    n = len(text)
    # Append circular doubling: BWT sorts cyclic rotations of text!
    # Wait: bzip2's BWT sorts CYCLIC rotations of text (without sentinel!).
    # Let's be 100% exact: cyclic rotations of `text` of length n!
    t2 = text + text
    # Initial ranks from 4-byte chunks
    sa = list(range(n))
    rank = [int.from_bytes(t2[i : i + 4], "big") for i in range(n)]
    # Compress initial ranks
    sa.sort(key=lambda i: rank[i])
    new_rank = [0] * n
    r = 0
    new_rank[sa[0]] = 0
    for idx in range(1, n):
        if rank[sa[idx]] != rank[sa[idx - 1]]:
            r += 1
        new_rank[sa[idx]] = r
    rank = new_rank
    step = 4
    while r < n - 1 and step < n:
        # Pair key: (rank[i], rank[(i + step) % n]) packed into one int
        shift = (n).bit_length()
        keys = [(rank[i] << shift) | rank[(i + step) % n] for i in range(n)]
        sa.sort(key=lambda i: keys[i])
        r = 0
        new_rank = [0] * n
        new_rank[sa[0]] = 0
        for idx in range(1, n):
            if keys[sa[idx]] != keys[sa[idx - 1]]:
                r += 1
            new_rank[sa[idx]] = r
        rank = new_rank
        step *= 2
    L = bytes(text[(i - 1) % n] for i in sa)
    p = sa.index(0)
    return L, p


def main():
    t0 = time.time()
    print("=== 1. VERIFYING & DECODING 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    blocks = verify_and_load_blocks()
    for b in blocks:
        text, C, counts, LF, row_at_L = decode_bwt_and_row_map(b["L"], b["p"])
        op = run_open_exact(b["L"], b["p"], LIMIT)
        assert op["out"] == text, f"Open output mismatch on {b['name']}!"
        assert op["runs"] == b["r_manifest"], f"Run count mismatch on {b['name']}: {op['runs']} vs {b['r_manifest']}"

        # Compute run width of every original row in L
        row_run_width = [0] * b["n"]
        for c, pos, w in rle_maximal_runs(b["L"]):
            for i in range(pos, pos + w):
                row_run_width[i] = w

        # Map each text position to its BWT row and whether that row is retained in Open!
        # Wait: in Open's emit, each retained rank j emits its word `wb` starting at `retained_text_pos[j]`!
        # Specifically, the LAST byte of `wb` is the thin row itself (`c`), and the earlier bytes of `wb`
        # are the thick-strip bytes preceding it!
        # Let's verify which text byte corresponds to retained rank j's OWN thin row:
        # In `decode_bwt_and_row_map`, `row_at_L[pos]` is the row whose L-byte is `text[pos]`.
        # Wait: in `open.rs`: `code` is either literal `a` (from thin F-row of byte `a`) or `expand(nwd) + [a]`.
        # Let's check exact correspondence between `retained_orig_rows[j]` and `retained_text_pos[j]`!
        # When `j` emits `wb` at `[tpos, tpos + len(wb))`:
        # `wb[-1]` is the byte `a` of the thin row `j` in `L` (or `F`? Wait: let's check!).
        # Check on first 5 retained rows:
        is_retained_orig = bytearray(b["n"])
        for r_orig in op["retained_orig_rows"]:
            is_retained_orig[r_orig] = 1

        # Let's also record for each retained rank j:
        #   - its text position of emission end: end_pos = (retained_text_pos[j] + len(wb) - 1) % n
        #   - its original row width
        b["text"] = text
        b["C"] = C
        b["counts"] = counts
        b["LF"] = LF
        b["row_at_L"] = row_at_L
        b["row_run_width"] = row_run_width
        b["open"] = op
        b["is_retained_orig"] = is_retained_orig

        byte_cls, token_at, tok_counts = classify_text_tokens(text)
        b["byte_cls"] = byte_cls
        b["token_at"] = token_at
        b["tok_counts"] = tok_counts

        # Check sample of decoded text
        preview = text[:80].decode("latin1").replace("\n", "\\n")
        print(
            f"  {b['name']:9s} ({b['window']} #{b['block_in_window']:2d}, corpus #{b['block_in_corpus']:3d}): "
            f"n={b['n']}, r={op['runs']}, m16={op['m']} ({100*op['m']/b['n']:.2f}%), "
            f"thick={len(op['strips'])}, bands={len(op['bands'])}, words={len(op['words'])}, "
            f"distinct_words={len(set(op['words']))}, spelling={op['spelling_bytes']}B, "
            f"rules={op['num_rules']}, steps={op['steps']}"
        )
        print(f"            preview: {preview!r}")

    # Verify exact relationship between retained_orig_rows and text positions
    b0 = blocks[0]
    op0 = b0["open"]
    matches_end = 0
    matches_start = 0
    for j in range(min(1000, op0["m"])):
        r_orig = op0["retained_orig_rows"][j]
        tpos = op0["retained_text_pos"][j]
        wb = op0["retained_emitted_word"][j]
        # Note: in emit, at step from `mark` (which is row p),
        # F[p] = text[0], and L[p] = text[n-1]!
        # Let's check what byte L[r_orig] and F[r_orig] equal!
        if tpos >= 0:
            # F[r_orig] is the byte at tpos? Or L[r_orig] is wb[-1]?
            if b0["L"][r_orig] == b0["text"][(tpos - 1) % b0["n"]]:
                matches_start += 1
            if b0["L"][r_orig] == wb[-1]:
                matches_end += 1
    print(f"\n  [Sanity check on block-0 first 1000 retained rows]: L[r_orig]==text[tpos-1]: {matches_start}/1000, L[r_orig]==wb[-1]: {matches_end}/1000")

    # Wait: let's check which row's byte is wb[-1]!
    # In open.rs:
    #   j = mark (corresponds to original row p)
    #   edge[j] gives (code, next_j).
    #   In wiring: dest = b.dst + (z - b.start) is the retained rank of the EXIT of LF from thin-row z!
    #   And value written into edges[dest] is (b.code, j_thin) where j_thin is the retained rank of the thin row of byte `c`!
    #   Wait: `z` is in F-order of thin rows of byte `c`, so `j_thin` is the retained row with L[j_thin] = c,
    #   and `dest` is the retained row that LF(thick-excursion) lands on!
    #   So walking `edges` from `dest` -> `j_thin` goes in Φ = LF^{-1} direction (forward in text!),
    #   emitting `expand(nwd) + [c]`, where `c = L[j_thin]` is the byte at `tpos + len(wb) - 1`!
    # Let's verify that `L[retained_orig_rows[edges_next[j]]] == wb[-1]`!
    ok_nxt = 0
    for j in range(op0["m"]):
        nxt = op0["edges_next"][j]
        wb = op0["retained_emitted_word"][j]
        if wb is not None and b0["L"][op0["retained_orig_rows"][nxt]] == wb[-1]:
            ok_nxt += 1
    print(f"  [Exact alignment check]: L[orig_row(edges_next[j])] == wb[-1]: {ok_nxt}/{op0['m']}")
    # And check text[tpos + len(wb) - 1] == L[orig_row(edges_next[j])]:
    ok_pos = 0
    for j in range(op0["m"]):
        nxt = op0["edges_next"][j]
        wb = op0["retained_emitted_word"][j]
        tpos = op0["retained_text_pos"][j]
        if wb is not None:
            r_orig = op0["retained_orig_rows"][nxt]
            epos = (tpos + len(wb) - 1) % b0["n"]
            if b0["row_at_L"][epos] == r_orig:
                ok_pos += 1
    print(f"  [Exact row<->text alignment]: row_at_L[tpos + len(wb) - 1] == orig_row(edges_next[j]): {ok_pos}/{op0['m']}")

    print("\n=== 2. CROSS-BLOCK OVERLAP: OPEN WORDS, SPELLING, THICK STRIPS & SLP RULES ===")
    # Pairs of primary interest + all pairs + cumulative history
    focus_pairs = [
        (0, 1, "block-0 -> block-2 (w00, +2)"),
        (3, 4, "block-207 -> block-210 (w10, +3)"),
        (1, 2, "block-2 -> block-85 (w00->w04)"),
        (2, 3, "block-85 -> block-143 (w04->w07)"),
    ]
    # Wait: check indices after sorting by block_in_corpus!
    idx_by_name = {b["name"]: i for i, b in enumerate(blocks)}
    focus_pairs = [
        (idx_by_name["block-0"], idx_by_name["block-2"], "block-0 -> block-2 (w00, +2)"),
        (idx_by_name["block-207"], idx_by_name["block-210"], "block-207 -> block-210 (w10, +3)"),
        (idx_by_name["block-2"], idx_by_name["block-85"], "block-2 -> block-85 (w00->w04)"),
        (idx_by_name["block-85"], idx_by_name["block-143"], "block-85 -> block-143 (w04->w07)"),
        (idx_by_name["block-143"], idx_by_name["block-207"], "block-143 -> block-207 (w07->w10)"),
        (idx_by_name["block-0"], idx_by_name["block-210"], "block-0 -> block-210 (w00->w10)"),
    ]

    for iA, iB, label in focus_pairs:
        A, B = blocks[iA], blocks[iB]
        opA, opB = A["open"], B["open"]

        wordsA_set = set(opA["words"])  # non-literal Open words (len >= 2)
        wordsB_set = set(opB["words"])
        shared_distinct = wordsA_set & wordsB_set

        # Spelling bytes in B that belong to words already in A's Open word set
        shared_spelling_bytes = sum(len(w) for w in opB["words"] if w in wordsA_set)
        shared_word_slots = sum(1 for w in opB["words"] if w in wordsA_set)

        # Retained rows & emitted bytes in B using words in A (non-literal only, and including literals)
        mB = opB["m"]
        lit_rows_B = sum(cnt for w, cnt in opB["word_emit_counts"].items() if len(w) == 1)
        nonlit_rows_B = mB - lit_rows_B
        nonlit_bytes_B = sum(len(w) * cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2)

        shared_nonlit_rows = sum(cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in wordsA_set)
        shared_nonlit_bytes = sum(len(w) * cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in wordsA_set)

        # Also check: how many non-literal Open words of B appear ANYWHERE as a substring in text_A?
        in_textA_distinct = sum(1 for w in wordsB_set if w in A["text"])
        in_textA_rows = sum(cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in A["text"])
        in_textA_bytes = sum(len(w) * cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in A["text"])

        # SLP rule sub-words overlap
        rulesA = opA["rule_words"]
        rulesB = opB["rule_words"]
        shared_rules = len(rulesA & rulesB)

        # Thick strips: exact (byte, width) multiset overlap (not geometrically meaningful, just census)
        stripsA = Counter((s[3], s[2]) for s in opA["strips"])
        stripsB = Counter((s[3], s[2]) for s in opB["strips"])
        shared_strip_bw = sum((stripsA & stripsB).values())

        print(f"\n  Pair: {label}")
        print(f"    Distinct non-lit Open words: B={len(wordsB_set)}, in Open(A)={len(shared_distinct)} ({100*len(shared_distinct)/len(wordsB_set):.1f}%), anywhere in text(A)={in_textA_distinct} ({100*in_textA_distinct/len(wordsB_set):.1f}%)")
        print(f"    Open word table slots (bands>=2): B={len(opB['words'])}, in Open(A)={shared_word_slots} ({100*shared_word_slots/len(opB['words']):.1f}%)")
        print(f"    Open spelling bytes: B={opB['spelling_bytes']}B, in Open(A)={shared_spelling_bytes}B ({100*shared_spelling_bytes/opB['spelling_bytes']:.1f}%)")
        print(f"    Retained rows m16={mB} (lit={lit_rows_B} [{100*lit_rows_B/mB:.1f}%], nonlit={nonlit_rows_B} [{100*nonlit_rows_B/mB:.1f}%]):")
        print(f"      nonlit rows with word in Open(A): {shared_nonlit_rows}/{nonlit_rows_B} ({100*shared_nonlit_rows/nonlit_rows_B:.1f}% of nonlit, {100*shared_nonlit_rows/mB:.1f}% of all m16)")
        print(f"      nonlit rows with word in text(A): {in_textA_rows}/{nonlit_rows_B} ({100*in_textA_rows/nonlit_rows_B:.1f}% of nonlit, {100*in_textA_rows/mB:.1f}% of all m16)")
        print(f"    Emitted bytes (n={B['n']}, nonlit={nonlit_bytes_B}B):")
        print(f"      from words in Open(A): {shared_nonlit_bytes}B ({100*shared_nonlit_bytes/B['n']:.1f}% of n, {100*shared_nonlit_bytes/nonlit_bytes_B:.1f}% of nonlit bytes)")
        print(f"      from words in text(A): {in_textA_bytes}B ({100*in_textA_bytes/B['n']:.1f}% of n, {100*in_textA_bytes/nonlit_bytes_B:.1f}% of nonlit bytes)")
        print(f"    SLP rule distinct strings: B={len(rulesB)}, shared with A={shared_rules} ({100*shared_rules/max(1,len(rulesB)):.1f}%)")
        print(f"    Thick strips (byte, width) multiset overlap: {shared_strip_bw}/{len(opB['strips'])} ({100*shared_strip_bw/len(opB['strips']):.1f}%)")

    # Cumulative history: union of all 5 other blocks -> block B
    print("\n  Cumulative cross-block Open word overlap (union of all OTHER 5 blocks -> B):")
    for iB, B in enumerate(blocks):
        opB = B["open"]
        union_words = set()
        for iA, A in enumerate(blocks):
            if iA != iB:
                union_words.update(A["open"]["words"])
        wordsB_set = set(opB["words"])
        sh_dist = len(wordsB_set & union_words)
        sh_spell = sum(len(w) for w in opB["words"] if w in union_words)
        sh_rows = sum(cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in union_words)
        sh_bytes = sum(len(w) * cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in union_words)
        print(
            f"    -> {B['name']:9s}: distinct={sh_dist}/{len(wordsB_set)} ({100*sh_dist/len(wordsB_set):.1f}%), "
            f"spelling={sh_spell}/{opB['spelling_bytes']}B ({100*sh_spell/opB['spelling_bytes']:.1f}%), "
            f"m16_nonlit_rows={sh_rows}/{opB['m']} ({100*sh_rows/opB['m']:.1f}% of m16), "
            f"emitted_bytes={100*sh_bytes/B['n']:.1f}% of n"
        )

    print("\n=== 3. RETAINED-ROW TOKEN CENSUS & CROSS-BLOCK REPETITION (WHO OWNS m_16?) ===")
    for iA, iB, label in [
        (idx_by_name["block-0"], idx_by_name["block-2"], "block-0 -> block-2 (w00, +2)"),
        (idx_by_name["block-207"], idx_by_name["block-210"], "block-207 -> block-210 (w10, +3)"),
    ]:
        A, B = blocks[iA], blocks[iB]
        opB = B["open"]
        mB = opB["m"]

        # Map each retained row (its thin-row byte position in text_B) to token class, width,
        # within-block repetition count, and whether the exact token appeared in A!
        cls_ret = Counter()
        cls_ret_in_A = Counter()
        cls_ret_w1 = Counter()
        cls_ret_w1_in_A = Counter()
        cls_ret_uniqB_inA = Counter()  # token occurs once in B, but >=1 in A!

        # Also check exact cross-block substring coverage (k-grams) at retained-row byte positions!
        # Wait: let's collect retained-row text positions:
        ret_tpos = []
        ret_w = []
        for j in range(mB):
            nxt = opB["edges_next"][j]
            wb = opB["retained_emitted_word"][j]
            tpos = opB["retained_text_pos"][j]
            epos = (tpos + len(wb) - 1) % B["n"]
            r_orig = opB["retained_orig_rows"][nxt]
            w_run = B["row_run_width"][r_orig]
            ret_tpos.append(epos)
            ret_w.append(w_run)

            tok_info = B["token_at"][epos]
            if tok_info is None:
                cname = "other"
                inA = False
                uniqB = False
            else:
                cname, tok = tok_info
                inA = tok in A["tok_counts"][cname]
                uniqB = B["tok_counts"][cname][tok] == 1
            cls_ret[cname] += 1
            if inA:
                cls_ret_in_A[cname] += 1
                if uniqB:
                    cls_ret_uniqB_inA[cname] += 1
            if w_run == 1:
                cls_ret_w1[cname] += 1
                if inA:
                    cls_ret_w1_in_A[cname] += 1

        print(f"\n  Pair: {label} (m16={mB}, width-1 retained={sum(cls_ret_w1.values())})")
        print(f"    {'Class':10s} | {'ret rows':>9s} {'(%m16)':>6s} | {'tok in A':>9s} {'(%cls)':>6s} | {'uniqB&inA':>9s} | {'w=1 rows':>9s} | {'w=1 in A':>9s}")
        for cname in ("GUID", "sha1", "entity_id", "json_key", "other"):
            rr = cls_ret[cname]
            ra = cls_ret_in_A[cname]
            ru = cls_ret_uniqB_inA[cname]
            w1 = cls_ret_w1[cname]
            w1a = cls_ret_w1_in_A[cname]
            pct_m = 100 * rr / mB
            pct_a = 100 * ra / max(1, rr)
            print(f"    {cname:10s} | {rr:9d} {pct_m:5.1f}% | {ra:9d} {pct_a:5.1f}% | {ru:9d} | {w1:9d} | {w1a:9d}")

        # Exact cross-block substring coverage (LZ77-style k-gram oracle) on retained rows and width-1 rows!
        k_res = cross_block_match_lengths(A["text"], B["text"])
        print("    Cross-block exact substring coverage of retained rows & width-1 rows (oracle):")
        w1_total = sum(1 for w in ret_w if w == 1)
        for k in (8, 12, 16, 24, 32, 40):
            starts_m, cov = k_res[k]
            ret_cov = sum(1 for pos in ret_tpos if cov[pos])
            ret_starts = sum(1 for pos in ret_tpos if starts_m[pos])
            w1_cov = sum(1 for pos, w in zip(ret_tpos, ret_w) if w == 1 and cov[pos])
            all_cov = sum(cov)
            print(
                f"      k={k:2d}: all_bytes={100*all_cov/B['n']:5.1f}%, "
                f"ret_covered={ret_cov:6d}/{mB} ({100*ret_cov/mB:5.1f}%), "
                f"ret_starts={100*ret_starts/mB:5.1f}%, "
                f"w1_covered={w1_cov:5d}/{w1_total} ({100*w1_cov/max(1,w1_total):5.1f}%)"
            )

        # Also check sha1 distinct-token overlap specifically
        shaA = A["tok_counts"]["sha1"]
        shaB = B["tok_counts"]["sha1"]
        sh_sha = set(shaA.keys()) & set(shaB.keys())
        occ_sh = sum(shaB[s] for s in sh_sha)
        tot_shaB = sum(shaB.values())
        print(
            f"    sha1 detail: distinct A={len(shaA)}, B={len(shaB)}, shared={len(sh_sha)} ({100*len(sh_sha)/max(1,len(shaB)):.1f}%); "
            f"occurrences in B from A={occ_sh}/{tot_shaB} ({100*occ_sh/max(1,tot_shaB):.1f}%)"
        )
        # Check entity_id distinct-token overlap
        entA = A["tok_counts"]["entity_id"]
        entB = B["tok_counts"]["entity_id"]
        sh_ent = set(entA.keys()) & set(entB.keys())
        occ_ent = sum(entB[s] for s in sh_ent)
        tot_entB = sum(entB.values())
        print(
            f"    entity_id detail: distinct A={len(entA)}, B={len(entB)}, shared={len(sh_ent)} ({100*len(sh_ent)/max(1,len(entB)):.1f}%); "
            f"occurrences in B from A={occ_ent}/{tot_entB} ({100*occ_ent/max(1,tot_entB):.1f}%)"
        )

    print("\n=== 4. BWT / LF SIGNATURE STABILITY ACROSS BLOCKS ===")
    for iA, iB, label in [
        (idx_by_name["block-0"], idx_by_name["block-2"], "block-0 -> block-2 (w00, +2)"),
        (idx_by_name["block-207"], idx_by_name["block-210"], "block-207 -> block-210 (w10, +3)"),
    ]:
        A, B = blocks[iA], blocks[iB]
        # 4a. Cumulative count drift |C_A[c] - C_B[c]|
        diffs_C = [abs(A["C"][c] - B["C"][c]) for c in range(256) if A["counts"][c] + B["counts"][c] > 0]
        l1_freq = sum(abs(A["counts"][c] / A["n"] - B["counts"][c] / B["n"]) for c in range(256)) / 2.0
        max_C_diff = max(diffs_C)
        med_C_diff = sorted(diffs_C)[len(diffs_C) // 2]

        # 4b. Compare BWT intervals [l, r) for shared Open words (length >= 6) via backward search
        # Build quick rank checkpoints or just compare band start/end?
        # Even better: compare exact (b_start, b_end, dst, delta) of bands with identical Open word!
        # Map word -> list of (orig_row_start_in_F_approx, band_k_start, band_delta, band_width)
        # Wait: even more direct — map word -> exact BWT interval [l, r) of rotations starting with word!
        # Let's do exact backward search for 200 shared Open words of length >= 6!
        # Precompute rank table? Actually, for 200 words, we can build prefix-rank every 1024 rows in 0.1s!
        def make_rank_fn(L, C):
            step = 1024
            n = len(L)
            chk = []
            cur = [0] * 256
            for i in range(0, n, step):
                chk.append(cur[:])
                for b in L[i : min(i + step, n)]:
                    cur[b] += 1
            chk.append(cur[:])

            def rank(c, pos):
                blk = pos // step
                cnt = chk[blk][c]
                base = blk * step
                # L[base:pos].count(c) is in C in CPython!
                return C[c] + cnt + L[base:pos].count(c)

            return rank

        rankA = make_rank_fn(A["L"], A["C"])
        rankB = make_rank_fn(B["L"], B["C"])

        def bsearch(w, n, rank_fn):
            l, r = 0, n
            for c in reversed(w):
                l = rank_fn(c, l)
                r = rank_fn(c, r)
                if l >= r:
                    return (l, r)
            return (l, r)

        shared_long = [w for w in (set(A["open"]["words"]) & set(B["open"]["words"])) if len(w) >= 6]
        shared_long.sort(key=lambda w: -len(w))
        sample = shared_long[:500]
        row_drifts = []
        rel_drifts = []
        width_exact = 0
        lf_shift_exact = 0
        lf_shift_drifts = []
        for w in sample:
            lA, rA = bsearch(w, A["n"], rankA)
            lB, rB = bsearch(w, B["n"], rankB)
            if lA < rA and lB < rB:
                row_drifts.append(abs(lA - lB))
                rel_drifts.append(abs(lA / A["n"] - lB / B["n"]))
                if (rA - lA) == (rB - lB):
                    width_exact += 1
                # Compare LF(lA) - lA vs LF(lB) - lB (local LF shift)
                dA = A["LF"][lA] - lA
                dB = B["LF"][lB] - lB
                if dA == dB:
                    lf_shift_exact += 1
                lf_shift_drifts.append(abs(dA - dB))

        row_drifts.sort()
        lf_shift_drifts.sort()
        print(f"\n  Pair: {label}")
        print(f"    Symbol freq TV distance: {l1_freq:.4f}; |C_A[c]-C_B[c]| median={med_C_diff}, max={max_C_diff}")
        print(
            f"    Shared Open words (len>=6, sample={len(row_drifts)}): "
            f"|l_A(w)-l_B(w)| min={row_drifts[0]}, med={row_drifts[len(row_drifts)//2]}, max={row_drifts[-1]}; "
            f"exact width match={width_exact}/{len(row_drifts)} ({100*width_exact/len(row_drifts):.1f}%)"
        )
        print(
            f"    Local LF shift stability: exact match={lf_shift_exact}/{len(lf_shift_drifts)}; "
            f"|delta_A - delta_B| min={lf_shift_drifts[0]}, med={lf_shift_drifts[len(lf_shift_drifts)//2]}, max={lf_shift_drifts[-1]}"
        )

    print("\n=== 5. STRICTLY ADJACENT SLICES (delta = 1, 450 KB -> 450 KB WITHIN EACH BLOCK) ===")
    for b in blocks[:2] + [blocks[idx_by_name["block-207"]]]:
        half = b["n"] // 2
        tA = b["text"][:half]
        tB = b["text"][half : 2 * half]
        LA, pA = build_bwt_from_text(tA)
        LB, pB = build_bwt_from_text(tB)
        opA = run_open_exact(LA, pA, LIMIT)
        opB = run_open_exact(LB, pB, LIMIT)
        assert opA["out"] == tA and opB["out"] == tB

        wordsA_set = set(opA["words"])
        wordsB_set = set(opB["words"])
        sh_dist = len(wordsA_set & wordsB_set)
        sh_spell = sum(len(w) for w in opB["words"] if w in wordsA_set)
        sh_rows = sum(cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in wordsA_set)
        sh_bytes = sum(len(w) * cnt for w, cnt in opB["word_emit_counts"].items() if len(w) >= 2 and w in wordsA_set)

        # Token overlap on adjacent 450KB halves
        _, _, tokA = classify_text_tokens(tA)
        _, tok_at_B, tokB = classify_text_tokens(tB)
        shaA, shaB = tokA["sha1"], tokB["sha1"]
        sh_sha = set(shaA.keys()) & set(shaB.keys())
        occ_sh = sum(shaB[s] for s in sh_sha)

        # Retained rows covered by tokens in tA
        # Compute exact row_at_L for tB
        _, _, _, _, row_at_L_B = decode_bwt_and_row_map(LB, pB)
        row_w_B = [0] * len(LB)
        for c, pos, w in rle_maximal_runs(LB):
            for i in range(pos, pos + w):
                row_w_B[i] = w
        ret_in_A_tok = Counter()
        ret_tot_cls = Counter()
        for j in range(opB["m"]):
            nxt = opB["edges_next"][j]
            wb = opB["retained_emitted_word"][j]
            tpos = opB["retained_text_pos"][j]
            epos = (tpos + len(wb) - 1) % len(tB)
            ti = tok_at_B[epos]
            cname = "other" if ti is None else ti[0]
            ret_tot_cls[cname] += 1
            if ti is not None and ti[1] in tokA[cname]:
                ret_in_A_tok[cname] += 1

        print(
            f"  Adjacent 450KB split of {b['name']}: m16_B={opB['m']} ({100*opB['m']/len(tB):.1f}%), "
            f"Open distinct words shared={sh_dist}/{len(wordsB_set)} ({100*sh_dist/len(wordsB_set):.1f}%), "
            f"spelling shared={100*sh_spell/opB['spelling_bytes']:.1f}%, "
            f"m16_nonlit_shared={100*sh_rows/opB['m']:.1f}%, emitted_bytes={100*sh_bytes/len(tB):.1f}%"
        )
        print(
            f"    Adjacent sha1 overlap: {len(sh_sha)}/{len(shaB)} distinct ({100*len(sh_sha)/max(1,len(shaB)):.1f}%), "
            f"occ={occ_sh}/{sum(shaB.values())} ({100*occ_sh/max(1,sum(shaB.values())):.1f}%); "
            f"ret rows in A: GUID={ret_in_A_tok['GUID']}/{ret_tot_cls['GUID']}, "
            f"sha1={ret_in_A_tok['sha1']}/{ret_tot_cls['sha1']}, "
            f"entity={ret_in_A_tok['entity_id']}/{ret_tot_cls['entity_id']}, "
            f"key={ret_in_A_tok['json_key']}/{ret_tot_cls['json_key']}"
        )

    print(f"\nDone in {time.time() - t0:.2f}s.")


if __name__ == "__main__":
    main()
