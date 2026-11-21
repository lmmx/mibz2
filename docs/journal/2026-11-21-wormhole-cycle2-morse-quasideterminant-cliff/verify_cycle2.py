#!/usr/bin/env python3
"""Cycle 2 Verifier & Census: Filtered Fox-Morse Collapse, Quasideterminants, and Perseus Z[t, t^-1].

Verifies:
  1. Theorem 2.1 (Filtered Fox-Morse Collapse M_Lambda & Single-Use Flattening Cliff):
     - Exhaustive arrow-by-arrow + byte-exact verification across 26,135 small (L, p, Lambda)
       instances (binary n<=8, ternary n<=5, all 0<=p<n, Lambda in {0, 1, 2, 4, 16}).
     - Exact 6-block filtration census across Lambda in {16, 8, 4, 2, 1} (30 block decodes =
       26,998,460 bytes verified byte-exact against Plain::invert).
  2. Theorem 2.2 (Milnor-Fox-Perseus Augmentation Z<Sigma*> -> Z[t, t^-1] & Gap Witness):
     - Exact verification that integer-only exponent reduction (0 SLP rules, 0 spelling bytes)
       recovers exact orbit distances / SA positions tau_p(x) on all critical rows across all
       5,227 small (L, p) pairs (Lambda=1) and all 6 Wikidata blocks.
     - Minimal primitive witness proving critical (x, g(x), L[x], F[x], ell(x)) + (n, C, p) at
       Lambda=1 fails to uniquely determine interior eliminated bytes without Fox words.
  3. Theorem 2.3 (Perseus Index-Order Local Apparent-Pair Dispersion):
     - Exact cache-line (64B = 16 u32s) and L1d-tile (32 KiB = 8,192 u32s) locality census of
       j -> g(j) on [0, m_Lambda) across all 6 blocks.
"""

from __future__ import annotations

import hashlib
import json
import os
import pathlib


def plain_invert_and_orbit_tau(L: bytes, p: int):
    """Return (decoded_bytes, orbit_len_d, tau_dict_mapping_row_to_text_end_offset)."""
    n = len(L)
    counts = [0] * 256
    for c in L:
        counts[c] += 1
    ftab = [0] * 256
    acc = 0
    for a in range(256):
        ftab[a] = acc
        acc += counts[a]
    psi = [0] * n
    for i, c in enumerate(L):
        k = ftab[c]
        psi[k] = i
        ftab[c] = k + 1

    # Walk Psi from p: at step t (0 <= t < d), row cur = Psi^{t+1}(p) writes out[t] = L[cur]!
    # So L[p] is written at step d - 1! Let pos_L[row] be the step t in [0, d) where L[row] is emitted!
    out = bytearray(n)
    pos_L = {}
    cur = psi[p]
    d = 0
    for t in range(n):
        b = L[cur]
        out[t] = b
        if cur not in pos_L:
            pos_L[cur] = t
            d += 1
        cur = psi[cur]
    return bytes(out), d, pos_L


def fox_morse_construct(L: bytes, p: int, limit: int, weight_only: bool = False):
    """Construct Filtered Fox-Morse reduction M_limit via Endpoint pairing.

    When weight_only=True (Perseus Z[t, t^-1] augmentation), carries ONLY integer path lengths
    (0 SLP rules, 0 spelling bytes!). Otherwise builds exact SLP + flattened band words.
    """
    n = len(L)
    all_c = [0] * 256
    thin_c = [0] * 256
    thick_c = [0] * 256
    strips = []
    crit_rows = []
    m = 0
    mark = 0
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w = j - i
        pos = i
        i = j

        def piece(x: int, pw: int, keep: bool):
            nonlocal m, mark
            if keep:
                if x <= p < x + pw:
                    mark = m + (p - x)
                for off in range(pw):
                    crit_rows.append(x + off)
                thin_c[c] += pw
                m += pw
            else:
                strips.append([x, all_c[c], pw, c])
                thick_c[c] += 1
            all_c[c] += pw

        if w <= limit:
            piece(pos, w, True)
        elif pos <= p < pos + w:
            if pos < p:
                piece(pos, p - pos, False)
            piece(p, 1, True)
            if p + 1 < pos + w:
                piece(p + 1, pos + w - p - 1, False)
        else:
            piece(pos, w, False)

    first = [0] * 256
    acc = 0
    for a in range(256):
        first[a] = acc
        acc += all_c[a]
    write = [0] * 256
    acc = 0
    for a in range(256):
        write[a] = acc
        acc += thick_c[a]
    H = len(strips)
    image_order = [0] * H
    for idx, s in enumerate(strips):
        a = s[3]
        s[1] += first[a]
        image_order[write[a]] = idx
        write[a] += 1

    max_nodes = 1 + 3 * H + 4
    prev0 = [0] * max_nodes
    prev1 = [0] * max_nodes
    next0 = [0] * max_nodes
    next1 = [0] * max_nodes
    n_width = [0] * max_nodes
    n_word = [0] * max_nodes  # word handle (or integer length if weight_only)
    num_nodes = 1

    def add_node(w: int, word: int):
        nonlocal num_nodes
        nid = num_nodes
        num_nodes += 1
        n_width[nid] = w
        n_word[nid] = word
        return nid

    order0, order1, term0, term1 = [], [], [], []
    for s in strips:
        add_node(s[2], 1 if weight_only else (s[3] + 1))
    x = 0
    for idx, s in enumerate(strips):
        if x < s[0]:
            nid = add_node(s[0] - x, 0)
            order0.append(nid)
            term1.append(nid)
        order0.append(idx + 1)
        x = s[0] + s[2]
    if x < n:
        nid = add_node(n - x, 0)
        order0.append(nid)
        term1.append(nid)

    x = 0
    for idx in image_order:
        s = strips[idx]
        if x < s[1]:
            nid = add_node(s[1] - x, 0)
            order1.append(nid)
            term0.append(nid)
        order1.append(idx + 1)
        x = s[1] + s[2]
    if x < n:
        nid = add_node(n - x, 0)
        order1.append(nid)
        term0.append(nid)

    prev = 0
    for nid in term0 + order0:
        next0[prev] = nid
        prev0[nid] = prev
        prev = nid
    next0[prev] = 0
    prev0[0] = prev

    prev = 0
    for nid in term1 + order1:
        next1[prev] = nid
        prev1[nid] = prev
        prev = nid
    next1[prev] = 0
    prev1[0] = prev

    rules = []  # (left, right, len, first_pos)

    def wlen(w: int) -> int:
        if weight_only:
            return w
        if w <= 256:
            return 1 if w != 0 else 0
        return rules[w - 257][2]

    def join(lw: int, rw: int) -> int:
        if weight_only:
            return lw + rw
        if lw == 0:
            return rw
        if rw == 0:
            return lw
        rid = len(rules) + 257
        rules.append([lw, rw, wlen(lw) + wlen(rw), -1])
        return rid

    rem = n
    steps = 0
    while rem > 0:
        nx = prev0[0]
        ny = prev1[0]
        aw = n_width[nx]
        bw = n_width[ny]
        w = aw if aw < bw else bw
        rem -= w
        steps += 1
        if nx == ny:
            p0, q0 = prev0[nx], next0[nx]
            next0[p0], prev0[q0] = q0, p0
            p1, q1 = prev1[nx], next1[nx]
            next1[p1], prev1[q1] = q1, p1
            continue
        wj = join(n_word[nx], n_word[ny])
        if aw <= bw:
            p0, q0 = prev0[nx], next0[nx]
            next0[p0], prev0[q0] = q0, p0
            qn = next0[ny]
            prev0[nx], next0[nx] = ny, qn
            next0[ny], prev0[qn] = nx, nx
            n_word[nx] = wj
            if aw == bw:
                p0, q0 = prev0[ny], next0[ny]
                next0[p0], prev0[q0] = q0, p0
                p1, q1 = prev1[ny], next1[ny]
                next1[p1], prev1[q1] = q1, p1
            else:
                n_width[ny] -= w
        else:
            p1, q1 = prev1[ny], next1[ny]
            next1[p1], prev1[q1] = q1, p1
            qn = next1[nx]
            prev1[ny], next1[ny] = nx, qn
            next1[nx], prev1[qn] = ny, ny
            n_word[ny] = wj
            n_width[nx] -= w

    bottom = [0] * num_nodes
    nid = next1[0]
    x = 0
    while nid != 0:
        bottom[nid] = x
        x += n_width[nid]
        nid = next1[nid]
    assert x == m

    # Build queues of retained ranks per byte
    queues = [[] for _ in range(256)]
    for rank_j, row_x in enumerate(crit_rows):
        queues[L[row_x]].append(rank_j)

    # Walk bands and wire
    spelling = bytearray()
    offsets = [0]
    stack = []

    def append_slp(wd: int, dest: bytearray):
        stack.clear()
        stack.append(wd)
        while stack:
            cur = stack.pop()
            if cur <= 256:
                dest.append(cur - 1)
            else:
                rl = rules[cur - 257]
                if rl[3] == -1:
                    rl[3] = len(dest)
                    stack.append(rl[1])
                    stack.append(rl[0])
                else:
                    st = rl[3]
                    dest.extend(dest[st : st + rl[2]])

    next_j = [0] * m
    edge_code = [0] * m  # code (or step length if weight_only)
    fwd_next_j = [0] * m
    fwd_len = [0] * m

    nid = next0[0]
    in_node = 0
    k = 0
    bands = 0
    words = 0
    unit_words = 0
    unit_spell_bytes = 0
    gt16_steps = 0

    for a in range(256):
        end = k + thin_c[a]
        q_idx = 0
        q_a = queues[a]
        while k < end:
            nw = n_width[nid]
            span = min(end - k, nw - in_node)
            dst = bottom[nid] + in_node
            wd = n_word[nid]
            step_len = wlen(wd) + 1
            if step_len > 16:
                gt16_steps += span
            if weight_only:
                code = step_len
            else:
                if wd == 0:
                    code = a
                else:
                    words += 1
                    append_slp(wd, spelling)
                    spelling.append(a)
                    offsets.append(len(spelling))
                    code = 255 + words
                    if span == 1:
                        unit_words += 1
                        unit_spell_bytes += step_len

            for d in range(span):
                src_j = q_a[q_idx + d]
                next_j[dst + d] = src_j
                edge_code[dst + d] = code
                fwd_next_j[src_j] = dst + d
                fwd_len[src_j] = step_len

            q_idx += span
            bands += 1
            in_node += span
            k += span
            if in_node == nw:
                nid = next0[nid]
                in_node = 0

    return {
        "m": m,
        "mark": mark,
        "crit_rows": crit_rows,
        "H": H,
        "steps": steps,
        "rules": len(rules),
        "bands": bands,
        "words": words,
        "spelling": bytes(spelling),
        "offsets": offsets,
        "next_j": next_j,
        "edge_code": edge_code,
        "fwd_next_j": fwd_next_j,
        "fwd_len": fwd_len,
        "unit_words": unit_words,
        "unit_spell_bytes": unit_spell_bytes,
        "gt16_steps": gt16_steps,
    }


def emit_from_morse(st: dict, n: int) -> bytes:
    m = st["m"]
    mark = st["mark"]
    next_j = st["next_j"]
    edge_code = st["edge_code"]
    spelling = st["spelling"]
    offsets = st["offsets"]
    out = bytearray()
    j = mark
    for _ in range(m):
        c = edge_code[j]
        j = next_j[j]
        if c < 256:
            out.append(c)
        else:
            w = c - 256
            out.extend(spelling[offsets[w] : offsets[w + 1]])
        if j == mark or len(out) >= n:
            break
    del out[n:]
    while len(out) < n:
        take = min(len(out), n - len(out))
        out.extend(out[:take])
    return bytes(out)


def verify_perseus_laurent_coordinates(L: bytes, p: int, limit: int) -> None:
    """Verify Theorem 2.2: integer-only Z[t, t^-1] Morse reduction recovers exact SA/orbit offsets."""
    _, d, pos_L = plain_invert_and_orbit_tau(L, p)
    st = fox_morse_construct(L, p, limit, weight_only=True)
    assert st["rules"] == 0 and len(st["spelling"]) == 0
    # Walk forward LF-return map fwd_next_j from mark:
    # Note L[p] is emitted at orbit position d - 1! Each forward LF-return step from j to dst = fwd_next_j[j]
    # moves BACKWARD in text by fwd_len[j] bytes (mod d)!
    # Equivalently, walking backward Psi-return map next_j from mark:
    # Step 0 starts at j_0 = mark; edge_code[j_0] has length ell_0, landing at j_1 = next_j[j_0], whose L[crit_rows[j_1]]
    # is the LAST byte of that ell_0-byte word, i.e., at text offset (cursor + ell_0 - 1) mod d!
    cursor = 0
    j = st["mark"]
    visited = 0
    while True:
        ell = st["edge_code"][j]
        j = st["next_j"][j]
        row_x = st["crit_rows"][j]
        if cursor + ell - 1 < d:
            assert pos_L[row_x] == cursor + ell - 1, (
                f"Coordinate mismatch: {pos_L[row_x]} != {cursor + ell - 1} on L={L}, p={p}"
            )
        cursor += ell
        visited += 1
        if j == st["mark"] or visited > st["m"]:
            break
    assert cursor == d, f"Total Laurent exponent sum {cursor} != orbit period {d}"


def check_unlabelled_gap_witness() -> dict:
    """Construct & verify minimal primitive single-cycle witness for Theorem 2.2(2).

    Finds two distinct primitive BWTs (L1, p) and (L2, p) with:
      - identical n, identical p, identical symbol counts C,
      - identical critical row indices V_crit(M_1), identical critical L-bytes and F-bytes,
      - identical reduced Z[t, t^-1] Morse maps (next_j AND integer path lengths ell_j),
      - YET distinct decoded texts T1 != T2 because eliminated multi-row runs swap order!
    """
    from itertools import product

    seen = {}
    for n in range(5, 9):
        for tup in product((0, 1, 2), repeat=n):
            L = bytes(tup)
            counts = [L.count(a) for a in range(3)]
            if any(c == 0 for c in counts):
                continue
            for p in range(n):
                out, d, _ = plain_invert_and_orbit_tau(L, p)
                if d != n:
                    continue  # primitive single-cycle only!
                st = fox_morse_construct(L, p, limit=1, weight_only=True)
                if st["m"] == n:
                    continue
                # Build F column on critical rows
                F_col = []
                for a in range(3):
                    F_col.extend([a] * counts[a])
                sig = (
                    n,
                    p,
                    tuple(counts),
                    tuple(st["crit_rows"]),
                    tuple(L[x] for x in st["crit_rows"]),
                    tuple(F_col[x] for x in st["crit_rows"]),
                    tuple(st["next_j"]),
                    tuple(st["edge_code"]),
                )
                if sig in seen:
                    L_prev, out_prev = seen[sig]
                    if out_prev != out:
                        return {
                            "n": n,
                            "p": p,
                            "counts": counts,
                            "L1": list(L_prev),
                            "L2": list(L),
                            "T1": list(out_prev),
                            "T2": list(out),
                            "crit_rows": st["crit_rows"],
                            "next_j": st["next_j"],
                            "lengths": st["edge_code"],
                        }
                else:
                    seen[sig] = (L, out)
    raise AssertionError("No witness found up to n=8")


def run_small_exhaustive() -> tuple[int, int]:
    morse_cases = 0
    laurent_cases = 0
    for base, max_n in ((2, 8), (3, 5)):
        for n in range(1, max_n + 1):
            for z in range(base**n):
                tmp = z
                col = bytearray(n)
                for i in range(n):
                    col[i] = tmp % base
                    tmp //= base
                L = bytes(col)
                for p in range(n):
                    ref, _, _ = plain_invert_and_orbit_tau(L, p)
                    for lim in (0, 1, 2, 4, 16):
                        st = fox_morse_construct(L, p, lim, weight_only=False)
                        got = emit_from_morse(st, n)
                        assert got == ref, f"Morse mismatch at L={L}, p={p}, lim={lim}"
                        morse_cases += 1
                    verify_perseus_laurent_coordinates(L, p, limit=1)
                    laurent_cases += 1
    return morse_cases, laurent_cases


def main():
    morse_cases, laurent_cases = run_small_exhaustive()
    print(
        f"[G1 SMALL EXHAUSTIVE] PASS: {morse_cases} Fox-Morse (L, p, Lambda) cases + "
        f"{laurent_cases} Perseus Z[t, t^-1] coordinate checks"
    )
    witness = check_unlabelled_gap_witness()
    print(
        f"[THEOREM 2.2 GAP WITNESS] Found minimal primitive single-cycle twin at n={witness['n']}, "
        f"p={witness['p']}, C={witness['counts']}: L1={witness['L1']} vs L2={witness['L2']} "
        f"share identical V_crit={witness['crit_rows']}, next_j={witness['next_j']}, "
        f"lengths={witness['lengths']}, yet T1={witness['T1']} != T2={witness['T2']}"
    )

    blocks_dir = pathlib.Path(os.environ.get("BLOCKS_DIR", "/home/antigravity/admitted-bwt"))
    manifest = json.loads((blocks_dir / "manifest.json").read_text())
    results = {
        "morse_small_cases": morse_cases,
        "laurent_small_cases": laurent_cases,
        "gap_witness": witness,
        "blocks": [],
    }

    for entry in manifest["blocks"]:
        fname = entry["file"]
        L = (blocks_dir / fname).read_bytes()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        n = entry["n"]
        p = entry["primary_index"]
        ref, _, _ = plain_invert_and_orbit_tau(L, p)

        # Verify Perseus Z[t, t^-1] exact coordinates on real block at Lambda=16 and Lambda=1
        verify_perseus_laurent_coordinates(L, p, limit=16)
        verify_perseus_laurent_coordinates(L, p, limit=1)

        lims_out = {}
        for lim in (16, 8, 4, 2, 1):
            st = fox_morse_construct(L, p, lim, weight_only=False)
            got = emit_from_morse(st, n)
            assert got == ref, f"6-block mismatch on {fname} at limit={lim}"

            m = st["m"]
            nxt = st["next_j"]
            same_cl64 = sum(1 for j in range(m) if (j >> 4) == (nxt[j] >> 4))
            same_l1_32k = sum(1 for j in range(m) if (j >> 13) == (nxt[j] >> 13))

            bits = max(1, (m - 1).bit_length())
            words = st["words"]
            wide = (255 + words) > (0xFFFFFFFF >> bits)
            edges_kib = (4 * m * (2 if wide else 1)) / 1024.0
            spell_kib = len(st["spelling"]) / 1024.0
            inline16_kib = (16 * (256 + words)) / 1024.0
            dict_kib = spell_kib + inline16_kib

            lims_out[str(lim)] = {
                "m": m,
                "H": st["H"],
                "steps": st["steps"],
                "rules": st["rules"],
                "bands": st["bands"],
                "words": words,
                "wide": wide,
                "edges_kib": round(edges_kib, 1),
                "spell_kib": round(spell_kib, 1),
                "dict_kib": round(dict_kib, 1),
                "unit_words": st["unit_words"],
                "unit_words_pct": round(100.0 * st["unit_words"] / max(1, words), 1),
                "unit_spell_kib": round(st["unit_spell_kib"] if "unit_spell_kib" in st else st["unit_spell_bytes"] / 1024.0, 1),
                "gt16_steps": st["gt16_steps"],
                "same_cl64_pct": round(100.0 * same_cl64 / m, 3),
                "same_l1_32k_pct": round(100.0 * same_l1_32k / m, 2),
            }

        results["blocks"].append({"file": fname, "n": n, "r": entry["r"], "limits": lims_out})
        l16 = lims_out["16"]
        l4 = lims_out["4"]
        l1 = lims_out["1"]
        print(
            f"[6-BLOCK EXACT] {fname:11s} | "
            f"L16: m={l16['m']:6d} (edges={l16['edges_kib']:5.1f}K, dict={l16['dict_kib']:5.1f}K, L1_hit={l16['same_l1_32k_pct']:4.2f}%) -> "
            f"L4: m={l4['m']:6d} (edges={l4['edges_kib']:5.1f}K, dict={l4['dict_kib']:5.1f}K) -> "
            f"L1: m={l1['m']:6d} (edges={l1['edges_kib']:5.1f}K, dict={l1['dict_kib']:5.1f}K, unit_spell={l1['unit_spell_kib']:5.1f}K/{l1['spell_kib']:5.1f}K)"
        )

    out_path = pathlib.Path(__file__).resolve().parent / "cycle2_results.json"
    out_path.write_text(json.dumps(results, indent=2) + "\n")
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
