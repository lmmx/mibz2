#!/usr/bin/env python3
"""
Empirical anatomy of the incompressible width-1 core on ALL 6 authenticated Wikidata BWT blocks.
Verifies SHA-256 against /tmp/mibz2-blocks/manifest.json.
Measures:
  1. Exact Open (LIMIT=16) retained set S_16, width-1 subset m_1, thick-bridge counts.
  2. Chain-length distributions of consecutive w=1 steps and consecutive width-1 rows in g_FL.
  3. Exact UUID/GUID, SHA1, and format-landmark structure, including per-offset retention rates
     across all 36 UUID byte positions (8-4-4-4-12) and surrounding delimiters ('$', '"').
  4. Local k-context uniqueness (LCP distribution on width-1 rows vs thick rows).
  5. Exact decomposition of m_16 into isolated random segments bounded by thick-strip anchors
     or format landmarks.
"""
import hashlib
import json
import os
import re
from collections import Counter


def verify_and_load():
    manifest_path = "/tmp/mibz2-blocks/manifest.json"
    with open(manifest_path) as f:
        manifest = json.load(f)
    blocks = []
    for entry in manifest["blocks"]:
        path = os.path.join("/tmp/mibz2-blocks", entry["file"])
        data = open(path, "rb").read()
        sha = hashlib.sha256(data).hexdigest()
        assert sha == entry["sha256"], f"SHA mismatch for {entry['file']}"
        assert len(data) == entry["n"]
        blocks.append((entry, data))
    return blocks


def compute_LF_and_text(L, p):
    n = len(L)
    cnt = [0] * 256
    for b in L:
        cnt[b] += 1
    C = [0] * 256
    s = 0
    for i in range(256):
        C[i] = s
        s += cnt[i]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = cur[b]
        cur[b] += 1
    # FL = LF^{-1}
    FL = [0] * n
    for i, j in enumerate(LF):
        FL[j] = i

    # Reconstruct text T[0..n) and row_of_pos[t], pos_of_row[row]
    # Convention in mibz2 / standard bzip2 BWT:
    # row p is rotation 0 (starts at T[0], ends at T[n-1] = L[p]).
    # LF moves backwards: L[p] = T[n-1], L[LF[p]] = T[n-2], ...
    # FL moves forwards: F[p] = T[0], F[FL[p]] = T[1], ...
    # Wait: verify F[row] == L[LF[row]]? No! L[row] = T[t-1], and F[LF[row]] = L[row] = T[t-1]!
    # So F[FL[row]] = T[t+1] when F[row] = T[t] and L[FL[row]] = T[t] = F[row]!
    # Let's fill T via LF from p:
    T = bytearray(n)
    pos_of_row = [-1] * n
    row_of_pos = [-1] * n
    row = p
    for k in range(n - 1, -1, -1):
        # L[row] = T[k]; row itself represents rotation starting at (k+1) % n
        t_start = (k + 1) % n
        pos_of_row[row] = t_start
        row_of_pos[t_start] = row
        T[k] = L[row]
        row = LF[row]
    assert row == p, f"Block is multi-cycle! Stopped early or missed p"
    return LF, FL, C, bytes(T), pos_of_row, row_of_pos


def compute_runs_and_retained(L, p, limit=16):
    n = len(L)
    run_len_of_row = [0] * n
    is_retained = bytearray(n)
    r_total = 0
    r_w1 = 0
    m_1 = 0
    m_2_16 = 0
    i = 0
    while i < n:
        b = L[i]
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        w = j - i
        r_total += 1
        if w == 1:
            r_w1 += 1
        for k in range(i, j):
            run_len_of_row[k] = w
        if w <= limit:
            for k in range(i, j):
                is_retained[k] = 1
            if w == 1:
                m_1 += w
            else:
                m_2_16 += w
        i = j
    p_split = 0
    if not is_retained[p]:
        is_retained[p] = 1
        p_split = 1
    m_16 = sum(is_retained)
    return run_len_of_row, is_retained, r_total, r_w1, m_16, m_1, m_2_16, p_split


def percentiles(vals):
    if not vals:
        return {}
    s = sorted(vals)
    n = len(s)
    return {
        "count": n,
        "sum": sum(s),
        "mean": sum(s) / n,
        "p50": s[int(0.50 * (n - 1))],
        "p75": s[int(0.75 * (n - 1))],
        "p90": s[int(0.90 * (n - 1))],
        "p95": s[int(0.95 * (n - 1))],
        "p99": s[int(0.99 * (n - 1))],
        "max": s[-1],
    }


# Regexes matching Wikidata JSON statement GUIDs and hashes
# Standard UUID inside Wikidata GUID: e.g. Q12345$A1B2C3D4-E5F6-4789-ABCD-EF0123456789 (upper or lower hex)
UUID_RE = re.compile(rb"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
GUID_FULL_RE = re.compile(rb"[QPLMqplm]\d+\$[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
SHA1_RE = re.compile(rb'"[0-9a-fA-F]{40}"')


def analyze_block(entry, L):
    name = entry["file"]
    n = entry["n"]
    p = entry["primary_index"]
    LF, FL, C, T, pos_of_row, row_of_pos = compute_LF_and_text(L, p)
    run_len, is_ret, r_total, r_w1, m_16, m_1, m_2_16, p_split = compute_runs_and_retained(L, p, 16)

    # Note: In Open, row r is retained if L[r] is in a thin run of L.
    # Remember L[r] = T[(pos_of_row[r] - 1) % n]!
    # So whether text byte T[t] corresponds to a retained L-row is determined by row = row_of_pos[(t + 1) % n]!
    # Wait: let's check BOTH views:
    #   L-view: row r holds L[r] = T[(pos_of_row[r] - 1) % n]. Its right context is T[pos_of_row[r]..].
    #   In Open's forward walk g_FL on S_16: from retained row r, we step FL repeatedly until next retained row r'.
    #   Let w_FL(r) = number of FL steps from r to next retained row r'.
    #   If w_FL(r) == 1, then FL[r] is retained immediately (no thick strip between r and FL[r]).
    #   If w_FL(r) > 1, then FL[r]..FL^{w-1}[r] are thick-strip rows (eliminated by Open)!

    # Let's walk T in forward text order t = 0..n-1, tracking row_t = row_of_pos[t]
    # Note: row_of_pos[t] has L[row_of_pos[t]] = T[t-1] and F[row_of_pos[t]] = T[t].
    # And FL[row_of_pos[t]] = row_of_pos[(t+1)%n]!
    # So consecutive retained rows in g_FL with w_FL == 1 are literally consecutive t where is_ret[row_of_pos[t]] == 1!

    ret_seq = [1 if is_ret[row_of_pos[t]] else 0 for t in range(n)]
    w1_seq = [1 if run_len[row_of_pos[t]] == 1 else 0 for t in range(n)]

    # Extract maximal chains of consecutive retained rows (w_FL == 1 segments) and consecutive width-1 rows
    def extract_chains(seq):
        # cyclic! rotate to start after a 0 if any 0 exists
        try:
            z = seq.index(0)
        except ValueError:
            return [len(seq)]
        chains = []
        cur = 0
        for i in range(n):
            v = seq[(z + i) % n]
            if v:
                cur += 1
            else:
                if cur > 0:
                    chains.append(cur)
                    cur = 0
        if cur > 0:
            chains.append(cur)
        return chains

    ret_chains = extract_chains(ret_seq)
    w1_chains = extract_chains(w1_seq)

    # Number of retained rows with w_FL == 1 vs w_FL > 1 (thick-strip exits)
    # In a chain of length L of consecutive retained rows, L-1 have w_FL == 1 and the last 1 has w_FL > 1.
    n_thick_exits = len(ret_chains)
    n_w_eq_1 = m_16 - n_thick_exits

    # Classify bytes by UUID/GUID/SHA1 using L-byte position:
    # row r holds byte L[r] = T[(pos_of_row[r] - 1) % n], whose right-context starts at T[pos_of_row[r]].
    # Let's mark text byte positions t_byte = (t - 1) % n for row_of_pos[t]!
    # Wait: let's be crystal clear:
    #   For a text position pos in 0..n-1, the BWT row where T[pos] appears in L is:
    #       row_L(pos) = row_of_pos[(pos + 1) % n].
    #   Why? Because L[row_of_pos[(pos + 1) % n]] = T[pos]!
    #   And the right context of that L-entry is T[pos+1 ..]!
    #   Let's verify this on t=0..10:
    for pos in range(10):
        assert L[row_of_pos[(pos + 1) % n]] == T[pos]

    uuid_matches = list(UUID_RE.finditer(T))
    guid_matches = list(GUID_FULL_RE.finditer(T))
    sha1_matches = list(SHA1_RE.finditer(T))

    # Per-offset retention stats for 36-char UUIDs, PLUS 1 byte before ('$' or similar) and 1 byte after ('"' or similar)
    # Offset -1 = char before UUID, 0..35 = UUID chars, 36 = char after UUID
    # Check BOTH row_L(pos) [whether T[pos] is in a thin/w1 run of L] AND row_F(pos) = row_of_pos[pos]
    offset_w1 = [0] * 38
    offset_ret = [0] * 38
    offset_thick = [0] * 38
    pre_chars = Counter()
    post_chars = Counter()
    ver_nibbles = Counter()
    var_nibbles = Counter()
    hex_case = Counter()

    in_uuid_byte = bytearray(n)
    in_guid_byte = bytearray(n)
    in_sha1_byte = bytearray(n)

    for m in guid_matches:
        for pos in range(m.start(), m.end()):
            in_guid_byte[pos] = 1
    for m in sha1_matches:
        for pos in range(m.start() + 1, m.end() - 1):
            in_sha1_byte[pos] = 1

    for m in uuid_matches:
        s = m.start()
        if s > 0:
            pre_chars[chr(T[s - 1])] += 1
        if s + 36 < n:
            post_chars[chr(T[s + 36])] += 1
        ver_nibbles[chr(T[s + 14])] += 1
        var_nibbles[chr(T[s + 19])] += 1
        seg = T[s : s + 36]
        if seg == seg.lower():
            hex_case["lower"] += 1
        elif seg == seg.upper():
            hex_case["upper"] += 1
        else:
            hex_case["mixed"] += 1
        for pos in range(s, s + 36):
            in_uuid_byte[pos] = 1
        for off in range(-1, 37):
            pos = s + off
            if 0 <= pos < n:
                r_idx = row_of_pos[(pos + 1) % n]  # row where T[pos] is in L
                rl = run_len[r_idx]
                idx = off + 1
                if rl == 1:
                    offset_w1[idx] += 1
                    offset_ret[idx] += 1
                elif rl <= 16:
                    offset_ret[idx] += 1
                else:
                    offset_thick[idx] += 1

    # Count how many m_16 and m_1 rows belong to UUID, GUID, SHA1 (via L-byte position T[pos] = L[row])
    uuid_m1 = uuid_m16 = 0
    guid_m1 = guid_m16 = 0
    sha1_m1 = sha1_m16 = 0
    for pos in range(n):
        r_idx = row_of_pos[(pos + 1) % n]
        if is_ret[r_idx]:
            is_w1 = run_len[r_idx] == 1
            if in_uuid_byte[pos]:
                uuid_m16 += 1
                if is_w1:
                    uuid_m1 += 1
            if in_guid_byte[pos]:
                guid_m16 += 1
                if is_w1:
                    guid_m1 += 1
            if in_sha1_byte[pos]:
                sha1_m16 += 1
                if is_w1:
                    sha1_m1 += 1

    # Check exact right-context LCP (k-context uniqueness) for rows in S_16 and m_1!
    # For row r (0..n-1), its right context is T[pos_of_row[r]..].
    # Its unique-identifying prefix length is 1 + max(LCP(r, r-1), LCP(r, r+1)).
    # We compute Kasai's LCP in O(n)!
    lcp = [0] * n
    k = 0
    for pos in range(n):
        r_idx = row_of_pos[pos]
        if r_idx == 0:
            k = 0
            continue
        prev_pos = pos_of_row[r_idx - 1]
        while k < n and T[(pos + k) % n] == T[(prev_pos + k) % n]:
            k += 1
        lcp[r_idx] = k
        if k > 0:
            k -= 1

    uniq_len_w1 = []
    uniq_len_uuid_w1 = []
    uniq_len_nonuuid_w1 = []
    for r_idx in range(n):
        if run_len[r_idx] == 1:
            l_prev = lcp[r_idx]
            l_next = lcp[r_idx + 1] if r_idx + 1 < n else 0
            u = 1 + (l_prev if l_prev > l_next else l_next)
            uniq_len_w1.append(u)
            pos_L = (pos_of_row[r_idx] - 1) % n
            if in_uuid_byte[pos_L]:
                uniq_len_uuid_w1.append(u)
            else:
                uniq_len_nonuuid_w1.append(u)

    # Check dashes '-' in L and in F!
    dash_byte = ord("-")
    dash_in_L_w1 = dash_in_L_thin = dash_in_L_thick = 0
    dash_in_F_w1 = dash_in_F_thin = dash_in_F_thick = 0
    # Compute F range for '-'
    dash_F_lo = C[dash_byte]
    dash_F_hi = C[dash_byte + 1]
    for r_idx in range(n):
        rl = run_len[r_idx]
        if L[r_idx] == dash_byte:
            if rl == 1:
                dash_in_L_w1 += 1
            elif rl <= 16:
                dash_in_L_thin += 1
            else:
                dash_in_L_thick += 1
        if dash_F_lo <= r_idx < dash_F_hi:
            if rl == 1:
                dash_in_F_w1 += 1
            elif rl <= 16:
                dash_in_F_thin += 1
            else:
                dash_in_F_thick += 1

    # Histogram of ret_chains and w1_chains (both by chain count AND by total rows in chains of that length!)
    def chain_row_hist(chains):
        bins = [(1, 1), (2, 4), (5, 8), (9, 16), (17, 32), (33, 40), (41, 64), (65, 10**9)]
        out = {}
        for lo, hi in bins:
            label = f"{lo}" if lo == hi else (f"{lo}+" if hi >= 10**9 else f"{lo}-{hi}")
            cnt_c = sum(1 for c in chains if lo <= c <= hi)
            cnt_r = sum(c for c in chains if lo <= c <= hi)
            out[label] = (cnt_c, cnt_r)
        return out

    # Also grab 3 sample UUID contexts from block-0
    samples = []
    for m in uuid_matches[:3]:
        s = max(0, m.start() - 15)
        e = min(n, m.end() + 15)
        samples.append(T[s:e].decode("utf-8", errors="replace"))

    print(f"\n==================== {name} (n={n}, r={r_total}, r/n={r_total/n:.4f}) ====================")
    print(f"  m_16 = {m_16} ({100*m_16/n:.2f}% of n), p_split={p_split}")
    print(f"  m_1  = {m_1} ({100*m_1/m_16:.1f}% of m_16, {100*r_w1/r_total:.1f}% of r), m_2..16 = {m_2_16}")
    print(f"  In g_FL: w==1 transitions = {n_w_eq_1} ({100*n_w_eq_1/m_16:.1f}% of m_16), thick-strip bridges (w>1) = {n_thick_exits} ({100*n_thick_exits/m_16:.1f}%)")
    print(f"  UUIDs={len(uuid_matches)}, full GUIDs={len(guid_matches)}, SHA1s={len(sha1_matches)}")
    print(f"    UUID owns: {uuid_m1}/{m_1} ({100*uuid_m1/m_1:.1f}%) of m_1; {uuid_m16}/{m_16} ({100*uuid_m16/m_16:.1f}%) of m_16")
    print(f"    GUID owns: {guid_m1}/{m_1} ({100*guid_m1/m_1:.1f}%) of m_1; {guid_m16}/{m_16} ({100*guid_m16/m_16:.1f}%) of m_16")
    print(f"    SHA1 owns: {sha1_m1}/{m_1} ({100*sha1_m1/m_1:.1f}%) of m_1; {sha1_m16}/{m_16} ({100*sha1_m16/m_16:.1f}%) of m_16")
    print(f"  UUID pre={pre_chars.most_common(4)}, post={post_chars.most_common(4)}, case={dict(hex_case)}")
    print(f"  UUID version nibbles (off 14)={ver_nibbles.most_common(5)}, variant (off 19)={var_nibbles.most_common(5)}")
    print(f"  Dash '-' in L: w1={dash_in_L_w1}, w2..16={dash_in_L_thin}, thick={dash_in_L_thick}")
    print(f"  Dash '-' in F: w1={dash_in_F_w1}, w2..16={dash_in_F_thin}, thick={dash_in_F_thick}")
    print(f"  Consecutive S_16 chains (bounded by thick strips): {percentiles(ret_chains)}")
    print(f"    Row share by chain length: {chain_row_hist(ret_chains)}")
    print(f"  Consecutive w1 chains:                             {percentiles(w1_chains)}")
    print(f"    Row share by chain length: {chain_row_hist(w1_chains)}")
    print(f"  Unique right-context len (1+max LCP) on all w1:    {percentiles(uniq_len_w1)}")
    print(f"  Unique right-context len on UUID w1:               {percentiles(uniq_len_uuid_w1)}")
    print(f"  Unique right-context len on non-UUID w1:           {percentiles(uniq_len_nonuuid_w1)}")
    if name == "block-0.L":
        print("  Sample UUID contexts:")
        for s in samples:
            print(f"    {s!r}")
    return offset_w1, offset_ret, offset_thick, len(uuid_matches)


def main():
    blocks = verify_and_load()
    print(f"[VERIFIED] All {len(blocks)} blocks passed SHA-256 verification against manifest.json")
    agg_w1 = [0] * 38
    agg_ret = [0] * 38
    agg_thick = [0] * 38
    agg_uuids = 0
    for entry, data in blocks:
        ow1, oret, othick, nu = analyze_block(entry, data)
        for i in range(38):
            agg_w1[i] += ow1[i]
            agg_ret[i] += oret[i]
            agg_thick[i] += othick[i]
        agg_uuids += nu

    print(f"\n==================== AGGREGATE UUID PER-OFFSET RETENTION (total UUIDs = {agg_uuids}) ====================")
    print("off  role        w1%    thin(2-16)%  thick(>16)%")
    for off in range(-1, 37):
        idx = off + 1
        tot = agg_ret[idx] + agg_thick[idx]
        if tot == 0:
            continue
        w1p = 100.0 * agg_w1[idx] / tot
        t2p = 100.0 * (agg_ret[idx] - agg_w1[idx]) / tot
        thp = 100.0 * agg_thick[idx] / tot
        if off == -1:
            role = "pre ('$')"
        elif off == 36:
            role = "post ('\"')"
        elif off in (8, 13, 18, 23):
            role = "DASH '-'"
        elif off == 14:
            role = "ver (4)"
        elif off == 19:
            role = "var (89ab)"
        else:
            role = "hex"
        print(f"{off:3d}  {role:10s}  {w1p:5.1f}%     {t2p:5.1f}%      {thp:5.1f}%")


if __name__ == "__main__":
    main()
