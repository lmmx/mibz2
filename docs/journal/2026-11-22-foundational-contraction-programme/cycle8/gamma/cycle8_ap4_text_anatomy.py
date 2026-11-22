#!/usr/bin/env python3
"""
Cycle 8 — Doorway 2: Working Backwards from the Text (`FRONTIER.md` Pending Lead #6)
Exact Text-Class, Survival-Reason, Width-Spectrum, & Charged-Copy/LZ77/Stallings-Fold
Anatomy of Alpha's `AP4_max` Surviving 0.135 m_16 Critical Core (~120,660 rows)
across ALL 6 authenticated Wikidata blocks (/tmp/mibz2-blocks/block-*.L)!
"""
import hashlib
import json
import os
import re
import sys
from bisect import bisect_right
from collections import defaultdict

BLOCKS_DIR = "/tmp/mibz2-blocks"

# Semantic token classes (1 byte tag per text position)
CLS_JSON_KEY   = 0  # JSON punctuation + Wikidata schema keys/enum values
CLS_GUID       = 1  # Statement GUIDs (Q...$xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx or UUID hex-hyphen)
CLS_SHA1       = 2  # 40-char lowercase hex reference hashes ("hash":"<40hex>")
CLS_QID_PID    = 3  # Wikidata entity/property IDs (Q\d+, P\d+)
CLS_URL        = 4  # http:// or https:// URLs
CLS_TIME_NUM   = 5  # Timestamps (+YYYY-MM-DD...) & numeric literals/IDs
CLS_NL_OTHER   = 6  # Natural-language labels/descriptions/sitelinks/other payloads

CLS_NAMES = [
    "JSON_STRUCT_KEY",
    "GUID_CLAIM_ID",
    "SHA1_HASH_40HEX",
    "QID_PID",
    "URL",
    "TIMESTAMP_NUM",
    "NL_TEXT_OTHER",
]

# Precompiled regexes on decoded Latin-1 text (1 byte == 1 char!)
RE_SHA1 = re.compile(r'"hash":"([0-9a-f]{40})"')
RE_GUID = re.compile(r'[Qq]\d+\$[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}|[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}')
RE_URL = re.compile(r'https?:\\?/\\?/[^"\s]+')
RE_TIME = re.compile(r'[+-]\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z')
RE_QPID = re.compile(r'\b[QP]\d+\b')
RE_SCHEMA = re.compile(
    r'"(?:claims|mainsnak|snaktype|property|datavalue|value|type|datatype|references|snaks|snaks-order|'
    r'qualifiers|qualifiers-order|rank|id|hash|entity-type|numeric-id|time|timezone|before|after|'
    r'precision|calendarmodel|text|language|latitude|longitude|altitude|globe|amount|unit|upperBound|'
    r'lowerBound|labels|descriptions|aliases|sitelinks|site|title|badges|pageid|ns|lastrevid|modified|'
    r'wikibase-entityid|wikibase-item|wikibase-property|wikibase-lexeme|string|external-id|monolingualtext|'
    r'globecoordinate|quantity|url|commonsMedia|item|normal|preferred|deprecated|somevalue|novalue|statement)"'
)
RE_NUM = re.compile(r'-?\d+(?:\.\d+)?')
RE_PUNCT = re.compile(r'[{}\[\]:,"]+')


def classify_wikidata_text(raw_bytes):
    """Assign every byte position t in [0, n) to one of 7 disjoint semantic classes."""
    n = len(raw_bytes)
    s = raw_bytes.decode("latin1")
    tags = bytearray([CLS_NL_OTHER] * n)

    # Apply from general to specific so specific tokens override generic ones cleanly!
    for m in RE_PUNCT.finditer(s):
        a, b = m.span()
        tags[a:b] = bytes([CLS_JSON_KEY]) * (b - a)
    for m in RE_NUM.finditer(s):
        a, b = m.span()
        tags[a:b] = bytes([CLS_TIME_NUM]) * (b - a)
    for m in RE_SCHEMA.finditer(s):
        a, b = m.span()
        tags[a:b] = bytes([CLS_JSON_KEY]) * (b - a)
    for m in RE_QPID.finditer(s):
        a, b = m.span()
        tags[a:b] = bytes([CLS_QID_PID]) * (b - a)
    for m in RE_TIME.finditer(s):
        a, b = m.span()
        tags[a:b] = bytes([CLS_TIME_NUM]) * (b - a)
    for m in RE_URL.finditer(s):
        a, b = m.span()
        tags[a:b] = bytes([CLS_URL]) * (b - a)
    for m in RE_GUID.finditer(s):
        a, b = m.span()
        tags[a:b] = bytes([CLS_GUID]) * (b - a)
    for m in RE_SHA1.finditer(s):
        # Mark `"hash":"` as JSON_KEY and the 40 hex chars as SHA1_HASH!
        a, b = m.span(1)
        tags[a:b] = bytes([CLS_SHA1]) * (b - a)
        a0 = m.start()
        tags[a0:a] = bytes([CLS_JSON_KEY]) * (a - a0)
    return tags


def load_blocks():
    manifest = json.load(open(os.path.join(BLOCKS_DIR, "manifest.json")))
    out = []
    for blk in manifest["blocks"]:
        data = open(os.path.join(BLOCKS_DIR, blk["file"]), "rb").read()
        assert hashlib.sha256(data).hexdigest() == blk["sha256"]
        assert len(data) == blk["n"]
        out.append((blk, data))
    return out


def build_strips_and_decode(data, p):
    """
    Build exact Open/Alpha maximal strips (lo, hi, delta, byte, mark_off),
    decode exact text `T[0..n)`, and compute exact `row_to_tpos[row]` where
    `T[row_to_tpos[row]] == L[row]` (i.e. `L[p]` is at `T[n-1]`, `L[LF^k(p)]` is at `T[n-1-k]`)!
    """
    n = len(data)
    counts = [0] * 256
    for b in data:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]

    occ = C[:]
    strips = []
    LF = [0] * n
    i = 0
    while i < n:
        b = data[i]
        j = i + 1
        while j < n and data[j] == b:
            j += 1
        w = j - i
        dst = occ[b]
        occ[b] = dst + w
        delta = dst - i
        m_off = (p - i) if (i <= p < j) else -1
        strips.append((i, j, delta, b, m_off))
        for k in range(w):
            LF[i + k] = dst + k
        i = j

    # Exact decode + exact row -> text position map!
    text = bytearray(n)
    row_to_tpos = [0] * n
    cur = p
    for t in range(n - 1, -1, -1):
        text[t] = data[cur]
        row_to_tpos[cur] = t
        cur = LF[cur]
    assert cur == p, "Expected single-cycle Wikidata block!"
    return strips, LF, bytes(text), row_to_tpos


def run_ap4_max_with_reasons(strips, n, p, cap_len=15):
    """
    Exact faithful reproduction of Alpha's `AP4_max` (`test_morse_apparent_pairs.py`)
    recording the EXACT survival reason for every critical strip!
    """
    q0 = len(strips)
    j_order = sorted(range(q0), key=lambda u: strips[u][0] + strips[u][2])

    out_deg = [0] * q0
    in_deg = [0] * q0
    out_target = [-1] * q0
    ii = jj = 0
    while ii < q0 and jj < q0:
        v = ii
        u = j_order[jj]
        v_lo, v_hi = strips[v][0], strips[v][1]
        u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        if u_thi <= v_lo:
            jj += 1
        elif v_hi <= u_tlo:
            ii += 1
        else:
            out_deg[u] += 1
            in_deg[v] += 1
            out_target[u] = v
            if u_thi < v_hi:
                jj += 1
            elif v_hi < u_thi:
                ii += 1
            else:
                ii += 1
                jj += 1
    for u in range(q0):
        if out_deg[u] != 1:
            out_target[u] = -1

    # Initial AP4_max matching: every thin strip u (w <= 16, m < 0) with out_deg == 1 and out_target != u
    par = [-1] * q0
    reason = [""] * q0  # why unmatched if thin!
    for u, (lo, hi, d, c, m) in enumerate(strips):
        w = hi - lo
        if m >= 0:
            reason[u] = "MARK"
        elif w > 16:
            reason[u] = "THICK"
        elif out_deg[u] >= 2:
            reason[u] = "SPLIT_OUT>=2"
        elif out_target[u] == u:
            reason[u] = "SELF_LOOP"
        else:
            par[u] = out_target[u]

    # Break cycles
    color = [0] * q0
    for u in range(q0):
        if color[u] != 0 or par[u] < 0:
            continue
        path = []
        cur = u
        while cur >= 0 and color[cur] == 0:
            color[cur] = 1
            path.append(cur)
            cur = par[cur]
        if cur >= 0 and color[cur] == 1:
            par[cur] = -1
            reason[cur] = "CYCLE_BREAK"
        for x in path:
            color[x] = 2

    # Enforce cap_len = 15 (chain_len <= 14) in postorder DFS
    chain_len = [-1] * q0
    final_delta = [0] * q0
    final_word = [None] * q0
    for u in range(q0):
        if par[u] < 0 or chain_len[u] >= 0:
            continue
        stack = []
        cur = u
        while cur >= 0 and par[cur] >= 0 and chain_len[cur] < 0:
            stack.append(cur)
            cur = par[cur]
        for node in reversed(stack):
            v = par[node]
            v_clen = chain_len[v] if (v >= 0 and par[v] >= 0) else 0
            if v_clen + 1 > cap_len - 1:
                par[node] = -1
                chain_len[node] = 0
                reason[node] = "CAP15_STOP"
            else:
                chain_len[node] = v_clen + 1
                if v >= 0 and par[v] >= 0:
                    final_delta[node] = strips[node][2] + final_delta[v]
                    final_word[node] = bytes([strips[node][3]]) + final_word[v]
                else:
                    final_delta[node] = strips[node][2]
                    final_word[node] = bytes([strips[node][3]])

    # Collect surviving critical rows & their exact emitted words (in forward text order!)
    starts = [s[0] for s in strips]
    crit_rows = []
    crit_strip_idx = []
    row_to_idx = {}
    for u, (lo, hi, d, c, m) in enumerate(strips):
        w = hi - lo
        if par[u] < 0 and (w <= 16 or m >= 0):
            for r in range(lo, hi):
                row_to_idx[r] = len(crit_rows)
                crit_rows.append(r)
                crit_strip_idx.append(u)

    m_crit = len(crit_rows)
    step_word_fwd = [None] * m_crit  # forward text order word emitted by critical row u_idx
    for u_idx, r0 in enumerate(crit_rows):
        cur = r0
        chunks = []
        while True:
            si = bisect_right(starts, cur) - 1
            lo, hi, d, c, m = strips[si]
            if par[si] >= 0:
                chunks.append(final_word[si])
                cur = cur + final_delta[si]
            else:
                chunks.append(bytes([c]))
                cur = cur + d
            if cur in row_to_idx:
                # chunks is in backward-LF order: chunks[0] is L[r0] (at text[tpos]),
                # chunks[-1] is earliest byte! Reverse to get exact forward text slice!
                step_word_fwd[u_idx] = b"".join(chunks)[::-1]
                break

    return par, reason, crit_rows, crit_strip_idx, step_word_fwd


def lz77_earlier_match_flags(text, k_len=8):
    """
    For each position t in [0, n - k_len], flag whether `text[t : t + k_len]` appeared
    strictly earlier in `text` (exact 8-byte / 16-byte LZ77 charged-copy oracle!).
    """
    n = len(text)
    seen = set()
    matched = bytearray(n)
    for t in range(n - k_len + 1):
        sub = text[t:t + k_len]
        if sub in seen:
            matched[t] = 1
        else:
            seen.add(sub)
    return matched


def main():
    blocks = load_blocks()

    # Aggregators across all 6 blocks
    tot_n = 0
    tot_m16 = 0
    tot_mcrit = 0
    reason_rows = defaultdict(int)
    reason_w1 = defaultdict(int)
    reason_strips = defaultdict(int)
    crit_width_hist = defaultdict(int)

    cls_all_bytes = [0] * 7
    cls_open_rows = [0] * 7
    cls_crit_rows = [0] * 7
    cls_crit_emitted_bytes = [0] * 7

    # Repetition / Charged-Copy / Stallings-Fold / LCP-Merge stats on AP4_max critical rows
    tot_distinct_words = 0
    tot_crit_words = 0
    wlen_buckets = defaultdict(int)
    lz8_hit_crit = 0
    lz16_hit_crit = 0
    lz8_hit_by_cls = [0] * 7
    lz16_hit_by_cls = [0] * 7
    lz8_hit_open = 0
    lcp4_adj = 0
    lcp8_adj = 0
    exact_same_adj = 0

    print("=== 1. PER-BLOCK AP4_max VERIFICATION (EXACT MATCH TO ALPHA'S 120,660 CORE) ===")
    for blk, data in blocks:
        n = len(data)
        p = blk['primary_index']
        strips, LF, text, row_to_tpos = build_strips_and_decode(data, p)
        tags = classify_wikidata_text(text)

        par, reason, crit_rows, crit_strip_idx, step_word_fwd = run_ap4_max_with_reasons(strips, n, p, cap_len=15)
        m_open = sum((hi - lo) for lo, hi, _, _, m in strips if (hi - lo) <= 16 or m >= 0)
        m_crit = len(crit_rows)
        tot_n += n
        tot_m16 += m_open
        tot_mcrit += m_crit

        # Verify every critical row's step_word_fwd matches `text` at its exact text position!
        emitted_sum = 0
        for u_idx, r0 in enumerate(crit_rows):
            w = step_word_fwd[u_idx]
            wl = len(w)
            emitted_sum += wl
            t_end = row_to_tpos[r0] + 1  # L[r0] is text[t_end - 1]!
            t_start = t_end - wl
            if t_start >= 0:
                assert text[t_start:t_end] == w, f"Word mismatch at row {r0}!"
        assert emitted_sum == n, f"Total emitted bytes {emitted_sum} != {n}"

        # Per-block summary
        dist_w = len(set(step_word_fwd))
        tot_distinct_words += dist_w
        tot_crit_words += m_crit
        print(f"  {blk['file']:11s}: m_16={m_open:6d} -> AP4_max m_crit={m_crit:6d} ({m_crit/m_open:.3f}x m16) | "
              f"distinct_words={dist_w:5d} ({100*dist_w/m_crit:4.1f}%) | avg_word={n/m_crit:5.1f} B [EXACT!]")

        # Survival reasons & width spectrum
        seen_s = set()
        for u_idx, si in enumerate(crit_strip_idx):
            lo, hi, _, _, _ = strips[si]
            w = hi - lo
            rsn = reason[si]
            reason_rows[rsn] += 1
            if w == 1:
                reason_w1[rsn] += 1
            if si not in seen_s:
                seen_s.add(si)
                reason_strips[rsn] += 1
            wk = "w=1" if w == 1 else ("w=2" if w == 2 else ("w=3..4" if w <= 4 else ("w=5..8" if w <= 8 else ("w=9..16" if w <= 16 else "w>16(mark)"))))
            crit_width_hist[wk] += 1

        # Text-class histograms: all n bytes, Open m_16 rows, AP4_max m_crit rows!
        for t in range(n):
            cls_all_bytes[tags[t]] += 1
        lz8 = lz77_earlier_match_flags(text, 8)
        lz16 = lz77_earlier_match_flags(text, 16)

        for lo, hi, _, _, m in strips:
            if (hi - lo) <= 16 or m >= 0:
                for r in range(lo, hi):
                    t = row_to_tpos[r]
                    cls_open_rows[tags[t]] += 1
                    if t >= 7 and lz8[t - 7]:
                        lz8_hit_open += 1

        for u_idx, r0 in enumerate(crit_rows):
            t = row_to_tpos[r0]
            c_tag = tags[t]
            w = step_word_fwd[u_idx]
            wl = len(w)
            cls_crit_rows[c_tag] += 1
            cls_crit_emitted_bytes[c_tag] += wl
            # Word length buckets
            wb = "len=1" if wl == 1 else ("len=2..8" if wl <= 8 else ("len=9..15" if wl <= 15 else ("len=16" if wl == 16 else ("len=17..32" if wl <= 32 else ("len=33..64" if wl <= 64 else "len>64")))))
            wlen_buckets[wb] += 1
            # LZ77 check at critical row's text position (ending at t): does 8B / 16B ending at t match earlier?
            if t >= 7 and lz8[t - 7]:
                lz8_hit_crit += 1
                lz8_hit_by_cls[c_tag] += 1
            if t >= 15 and lz16[t - 15]:
                lz16_hit_crit += 1
                lz16_hit_by_cls[c_tag] += 1
            # Adjacent critical row word LCP (suffix/prefix similarity in BWT order!)
            if u_idx > 0:
                prev_w = step_word_fwd[u_idx - 1]
                if prev_w == w:
                    exact_same_adj += 1
                # Common suffix (since BWT rows are sorted by following context, words end at L[r0]!)
                # Also check common prefix of emitted forward word!
                lim = min(len(prev_w), wl)
                cp = 0
                while cp < lim and prev_w[cp] == w[cp]:
                    cp += 1
                cs = 0
                while cs < lim and prev_w[len(prev_w) - 1 - cs] == w[wl - 1 - cs]:
                    cs += 1
                cm = cp if cp > cs else cs
                if cm >= 4:
                    lcp4_adj += 1
                if cm >= 8:
                    lcp8_adj += 1

    print(f"\n  6-BLK SUM: m_16={tot_m16} -> AP4_max m_crit={tot_mcrit} ({tot_mcrit/tot_m16:.3f}x m16) | "
          f"sum(distinct_words)={tot_distinct_words} ({100*tot_distinct_words/tot_mcrit:.1f}% of m_crit)")

    print("\n=== 2. EXACT SURVIVAL REASON & WIDTH SPECTRUM OF AP4_max's 120,660 CRITICAL ROWS (6-BLK SUM) ===")
    for rsn in ("SPLIT_OUT>=2", "CAP15_STOP", "CYCLE_BREAK", "SELF_LOOP", "MARK"):
        rr = reason_rows[rsn]
        rs = reason_strips[rsn]
        w1 = reason_w1[rsn]
        print(f"  {rsn:14s}: {rr:7d} rows ({100*rr/tot_mcrit:5.1f}% of core) | {rs:6d} strips | w=1: {w1:6d} ({100*w1/max(1,rr):4.1f}%)")
    print("  Width spectrum of critical rows:")
    for wk in ("w=1", "w=2", "w=3..4", "w=5..8", "w=9..16", "w>16(mark)"):
        c = crit_width_hist[wk]
        print(f"    {wk:10s}: {c:7d} rows ({100*c/tot_mcrit:5.1f}%)")

    print("\n=== 3. EXACT TEXT-CLASS ANATOMY: ALL TEXT vs OPEN m_16 vs AP4_max CRITICAL CORE (6-BLK SUM) ===")
    print(f"{'Text Class':18s} | {'All Text%':>9s} | {'Open m16':>8s} {'%m16':>6s} | {'AP4 Crit':>8s} {'%Core':>6s} {'Enrich':>6s} | {'EmitB%':>6s} | {'LZ8%':>5s} {'LZ16%':>5s}")
    print("-" * 106)
    for ci, cname in enumerate(CLS_NAMES):
        pct_all = 100.0 * cls_all_bytes[ci] / tot_n
        pct_op = 100.0 * cls_open_rows[ci] / tot_m16
        pct_cr = 100.0 * cls_crit_rows[ci] / tot_mcrit
        enrich = pct_cr / max(0.01, pct_all)
        pct_eb = 100.0 * cls_crit_emitted_bytes[ci] / tot_n
        l8 = 100.0 * lz8_hit_by_cls[ci] / max(1, cls_crit_rows[ci])
        l16 = 100.0 * lz16_hit_by_cls[ci] / max(1, cls_crit_rows[ci])
        print(f"{cname:18s} | {pct_all:8.1f}% | {cls_open_rows[ci]:8d} {pct_op:5.1f}% | "
              f"{cls_crit_rows[ci]:8d} {pct_cr:5.1f}% {enrich:5.2f}x | {pct_eb:5.1f}% | {l8:4.1f}% {l16:4.1f}%")

    print("\n=== 4. CHARGED-COPY / LZ77 / STALLINGS-FOLD / ADJACENT-LCP ORACLE HEADROOM ON AP4_max CORE ===")
    print(f"  LZ77 8-byte earlier match:  AP4_max core = {lz8_hit_crit:6d}/{tot_mcrit} ({100*lz8_hit_crit/tot_mcrit:.1f}%)  |  Open m_16 = {lz8_hit_open:6d}/{tot_m16} ({100*lz8_hit_open/tot_m16:.1f}%)")
    print(f"  LZ77 16-byte earlier match: AP4_max core = {lz16_hit_crit:6d}/{tot_mcrit} ({100*lz16_hit_crit/tot_mcrit:.1f}%)")
    print(f"  Adjacent BWT-order critical rows: exact_same_word = {exact_same_adj:6d} ({100*exact_same_adj/tot_mcrit:.1f}%), "
          f"common_affix>=4 = {lcp4_adj:6d} ({100*lcp4_adj/tot_mcrit:.1f}%), common_affix>=8 = {lcp8_adj:6d} ({100*lcp8_adj/tot_mcrit:.1f}%)")
    print("  Critical step emitted word-length distribution:")
    for wb in ("len=1", "len=2..8", "len=9..15", "len=16", "len=17..32", "len=33..64", "len>64"):
        c = wlen_buckets[wb]
        print(f"    {wb:10s}: {c:7d} ({100*c/tot_mcrit:5.1f}%)")


if __name__ == "__main__":
    main()
