#![forbid(unsafe_code)]

//! Programme Investigator P3 (Cycle 46) — Analogue C (Periodic Texts)
//! Exact Assembled Stepanov-Transplanted Chain (`ExactPeriodicFast`, `AnalogueC-LPTQ`, `DSID`),
//! Exhaustive Exactness Audit (5,700+ small + periodic + almost-periodic + non-Wikidata + 6 blocks),
//! and Native 1T / 2T / 4W Timing & Memory Benchmark.
//!
//! Compile & run:
//!   RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 \
//!     -C target-cpu=native -C codegen-units=1 \
//!     analogue_c_chain.rs -o /tmp/cks_c46/p3/analogue_c_chain
//!   /tmp/cks_c46/p3/analogue_c_chain

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const LIMIT: usize = 16;

const BLOCKS: &[(&str, usize)] = &[
    ("block-0.L", 264_045),
    ("block-2.L", 320_807),
    ("block-85.L", 325_088),
    ("block-143.L", 802_868),
    ("block-207.L", 205_180),
    ("block-210.L", 791_429),
];

// ============================================================================
// Core BWT & PRNG helpers
// ============================================================================

struct XorShift64(u64);
impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn build_bwt(text: &[u8]) -> (Vec<u8>, usize) {
    let n = text.len();
    assert!(n > 0);
    // Suffix-doubling circular sort for fast O(n log n) BWT on up to 300k bytes!
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rnk: Vec<usize> = text.iter().map(|&b| b as usize).collect();
    let mut tmp = vec![0usize; n];
    let mut cnt = vec![0usize; n.max(256) + 1];
    let mut sa2 = vec![0usize; n];

    // Initial counting sort by first byte
    for &r in &rnk {
        cnt[r + 1] += 1;
    }
    for i in 1..=256 {
        cnt[i] += cnt[i - 1];
    }
    for i in 0..n {
        let r = rnk[i];
        sa[cnt[r]] = i;
        cnt[r] += 1;
    }
    let mut classes = 1usize;
    tmp[sa[0]] = 0;
    for i in 1..n {
        if rnk[sa[i]] != rnk[sa[i - 1]] {
            classes += 1;
        }
        tmp[sa[i]] = classes - 1;
    }
    rnk.copy_from_slice(&tmp);

    let mut step = 1usize;
    while step < n && classes < n {
        for i in 0..n {
            sa2[i] = (sa[i] + n - step) % n;
        }
        for v in &mut cnt[..=classes] {
            *v = 0;
        }
        for &p in &sa2 {
            cnt[rnk[p] + 1] += 1;
        }
        for i in 1..=classes {
            cnt[i] += cnt[i - 1];
        }
        for &p in &sa2 {
            let r = rnk[p];
            sa[cnt[r]] = p;
            cnt[r] += 1;
        }
        tmp[sa[0]] = 0;
        let mut new_classes = 1usize;
        for i in 1..n {
            let a = sa[i - 1];
            let b = sa[i];
            if rnk[a] != rnk[b] || rnk[(a + step) % n] != rnk[(b + step) % n] {
                new_classes += 1;
            }
            tmp[b] = new_classes - 1;
        }
        rnk.copy_from_slice(&tmp);
        classes = new_classes;
        step <<= 1;
    }

    // Break exact ties (periodic texts) stably by starting index
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && rnk[sa[j]] == rnk[sa[i]] {
            j += 1;
        }
        if j - i > 1 {
            sa[i..j].sort_unstable();
        }
        i = j;
    }

    let mut l_col = Vec::with_capacity(n);
    let mut primary = 0usize;
    for (row, &rot) in sa.iter().enumerate() {
        if rot == 0 {
            primary = row;
        }
        l_col.push(text[(rot + n - 1) % n]);
    }
    (l_col, primary)
}

fn build_lf(l_col: &[u8]) -> (Vec<u32>, [usize; 257]) {
    let n = l_col.len();
    let mut counts = [0usize; 256];
    for &b in l_col {
        counts[b as usize] += 1;
    }
    let mut c_arr = [0usize; 257];
    for i in 0..256 {
        c_arr[i + 1] = c_arr[i] + counts[i];
    }
    let mut next_pos = [0usize; 256];
    next_pos.copy_from_slice(&c_arr[..256]);
    let mut lf = vec![0u32; n];
    for (i, &b) in l_col.iter().enumerate() {
        let bi = b as usize;
        lf[i] = next_pos[bi] as u32;
        next_pos[bi] += 1;
    }
    (lf, c_arr)
}

fn invert_naive(l_col: &[u8], primary: usize) -> Vec<u8> {
    let n = l_col.len();
    let (lf, _) = build_lf(l_col);
    let mut out = vec![0u8; n];
    let mut r = primary;
    for k in 0..n {
        out[n - 1 - k] = l_col[r];
        r = lf[r] as usize;
    }
    out
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

// ============================================================================
// 1. ExactPeriodicFast — Literal Stepanov Analogue C Proof (Steps C1-C6)
//    Recognises exact period-d texts (q = n/d >= 2) from L alone, inverts size d,
//    tiles q times; returns None if block is not exact periodic (exact fallback!).
// ============================================================================

fn recognise_and_invert_exact_periodic(l_col: &[u8], primary: usize) -> Option<(Vec<u8>, usize)> {
    let n = l_col.len();
    if n < 2 {
        return None;
    }
    // Step C1: Compute g = gcd of all maximal run widths AND all byte counts in one sequential pass.
    let mut counts = [0usize; 256];
    let mut g_run = 0usize;
    let mut i = 0;
    while i < n {
        let b = l_col[i];
        let mut j = i + 1;
        while j < n && l_col[j] == b {
            j += 1;
        }
        let w = j - i;
        counts[b as usize] += w;
        g_run = gcd(g_run, w);
        if g_run == 1 {
            return None; // Immediate O(first w=1 run) exit!
        }
        i = j;
    }
    let mut g = g_run;
    for &c in &counts {
        if c > 0 {
            g = gcd(g, c);
        }
    }
    if g < 2 {
        return None;
    }

    // Step C2-C3: By Step C1, EVERY maximal run width and EVERY byte count is divisible by g.
    // Wait: does g | w_i and g | count[c] ALREADY guarantee LF(k*g + j) = LF_u(k)*g + j?!
    // Proof: Because every maximal run starts and ends at a multiple of g, the rank of byte c
    // before row k*g is a multiple of g, and C[c] is a multiple of g! Therefore:
    //   LF(k*g + j) = C[c] + rank_c(k*g) + j = g * (C[c]/g + rank_c(k*g)/g) + j = LF_u(k)*g + j!
    // Hence Step C2 holds AUTOMATICALLY whenever Step C1 yields g >= 2!
    // Step C4: Downsample L -> L_u of length d = n / g and invert in O(d).
    let d = n / g;
    let mut l_u = Vec::with_capacity(d);
    for k in 0..d {
        l_u.push(l_col[k * g]);
    }
    let mut c_u = [0usize; 256];
    let mut acc = 0usize;
    for b in 0..256 {
        c_u[b] = acc;
        acc += counts[b] / g;
    }
    let mut lf_u = vec![0u32; d];
    for (k, &b) in l_u.iter().enumerate() {
        let bi = b as usize;
        lf_u[k] = c_u[bi] as u32;
        c_u[bi] += 1;
    }

    // Step C4-C5: Walk d steps from k0 = primary / g; verify it forms a cycle dividing d!
    let mut period = vec![0u8; d];
    let k0 = primary / g;
    let mut cur = k0;
    for t in 0..d {
        period[d - 1 - t] = l_u[cur];
        cur = lf_u[cur] as usize;
    }
    if cur != k0 {
        return None;
    }
    // Step C6: Bulk sequential repeat(period, g)
    let mut out = vec![0u8; n];
    for rep in 0..g {
        out[rep * d..(rep + 1) * d].copy_from_slice(&period);
    }
    Some((out, g))
}

// ============================================================================
// 2. Clean Open<16> State (Compact Flat Spelling + Inline16 + Edges/Codes)
// ============================================================================

#[derive(Clone)]
struct OpenState {
    n: usize,
    m: usize,
    mark: usize,
    edges: Vec<u32>,
    codes: Vec<u32>,
    /// Compact flat spelling dictionary: word `id` is `&spell[word_off[id]..word_off[id]+word_len[id]]`
    /// stored in FORWARD text order (ready for direct copy_from_slice into `out`!)
    spell: Vec<u8>,
    word_off: Vec<u32>,
    word_len: Vec<u16>,
    /// Pre-built XOR table for mu=2 bidirectional emit
    xor_edges: Vec<u32>,
    mark_prev: usize,
}

fn build_open_state(l_col: &[u8], primary: usize, limit: usize) -> OpenState {
    let n = l_col.len();
    let (lf, _) = build_lf(l_col);
    let mut is_thin = vec![false; n];
    let mut i = 0;
    while i < n {
        let b = l_col[i];
        let mut j = i + 1;
        while j < n && l_col[j] == b {
            j += 1;
        }
        let w = j - i;
        if w <= limit {
            for slot in &mut is_thin[i..j] {
                *slot = true;
            }
        } else if primary >= i && primary < j {
            is_thin[primary] = true;
        }
        i = j;
    }

    let mut ret_row = Vec::new();
    let mut row_to_rank = vec![u32::MAX; n];
    for x in 0..n {
        if is_thin[x] {
            row_to_rank[x] = ret_row.len() as u32;
            ret_row.push(x as u32);
        }
    }
    let m = ret_row.len();
    let mark = row_to_rank[primary] as usize;

    let mut edges = vec![0u32; m];
    let mut codes = vec![0u32; m];
    let mut spell = Vec::with_capacity(65_536);
    let mut word_off = Vec::with_capacity(2048);
    let mut word_len = Vec::with_capacity(2048);
    let mut intern: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut buf = Vec::with_capacity(64);

    for j in 0..m {
        buf.clear();
        let mut cur = ret_row[j] as usize;
        let mut steps = 0usize;
        loop {
            buf.push(l_col[cur]);
            cur = lf[cur] as usize;
            steps += 1;
            if row_to_rank[cur] != u32::MAX || steps == n {
                break;
            }
        }
        edges[j] = row_to_rank[cur];
        // Reverse buf so it is in FORWARD text order (last emitted byte first)
        buf.reverse();
        let wid = if let Some(&id) = intern.get(&buf) {
            id
        } else {
            let id = word_off.len() as u32;
            word_off.push(spell.len() as u32);
            word_len.push(buf.len() as u16);
            spell.extend_from_slice(&buf);
            intern.insert(buf.clone(), id);
            id
        };
        codes[j] = wid;
    }

    // Build XOR table for mu=2 emit: xor_edges[u] = next(u) ^ prev(u)
    let mut xor_edges = vec![0u32; m];
    let mut mark_prev = 0usize;
    for u in 0..m {
        let nxt = edges[u] as usize;
        xor_edges[u] ^= nxt as u32;
        xor_edges[nxt] ^= u as u32;
        if nxt == mark {
            mark_prev = u;
        }
    }

    OpenState {
        n,
        m,
        mark,
        edges,
        codes,
        spell,
        word_off,
        word_len,
        xor_edges,
        mark_prev,
    }
}

fn emit_open_serial(st: &OpenState, out: &mut [u8]) {
    let mut pos = st.n;
    let mut cur = st.mark;
    while pos > 0 {
        let code = st.codes[cur] as usize;
        let len = st.word_len[code] as usize;
        let off = st.word_off[code] as usize;
        if len <= pos {
            pos -= len;
            out[pos..pos + len].copy_from_slice(&st.spell[off..off + len]);
        } else {
            // Short periodic cycle wrap at start of text
            let take = pos;
            out[0..take].copy_from_slice(&st.spell[off + len - take..off + len]);
            pos = 0;
        }
        cur = st.edges[cur] as usize;
    }
}

/// Exact mu=2 XOR-Peel bidirectional emit (MSC 94B degree-2 linear erasure code).
/// Forward cursor writes backward from `n` using `cur = mark`;
/// Backward cursor writes forward from `0` using `rev = mark_prev`!
/// Exact on both single-cycle (primitive) and short-cycle (periodic) columns!
fn emit_open_xor2(st: &OpenState, out: &mut [u8]) {
    if st.m <= 2 {
        emit_open_serial(st, out);
        return;
    }
    let mut hi = st.n;
    let mut lo = 0usize;

    // Forward cursor: starts at mark, knows prev = mark_prev
    let mut f_prev = st.mark_prev as u32;
    let mut f_cur = st.mark as u32;

    // Backward cursor: starts at mark_prev, knows next = mark
    let mut b_next = st.mark as u32;
    let mut b_cur = st.mark_prev as u32;

    loop {
        // Step forward cursor (writes backward from hi)
        let fc = st.codes[f_cur as usize] as usize;
        let flen = st.word_len[fc] as usize;
        let foff = st.word_off[fc] as usize;
        if hi.saturating_sub(lo) <= flen {
            let need = hi - lo;
            out[lo..hi].copy_from_slice(&st.spell[foff + flen - need..foff + flen]);
            return;
        }
        hi -= flen;
        out[hi..hi + flen].copy_from_slice(&st.spell[foff..foff + flen]);

        // Step backward cursor (writes forward from lo)
        let bc = st.codes[b_cur as usize] as usize;
        let blen = st.word_len[bc] as usize;
        let boff = st.word_off[bc] as usize;
        if hi.saturating_sub(lo) <= blen {
            let need = hi - lo;
            out[lo..hi].copy_from_slice(&st.spell[boff..boff + need]);
            return;
        }
        out[lo..lo + blen].copy_from_slice(&st.spell[boff..boff + blen]);
        lo += blen;

        // Advance both cursors via degree-2 XOR peeling!
        let f_nxt = st.xor_edges[f_cur as usize] ^ f_prev;
        f_prev = f_cur;
        f_cur = f_nxt;

        let b_prv = st.xor_edges[b_cur as usize] ^ b_next;
        b_next = b_cur;
        b_cur = b_prv;

        // Detect periodic short-cycle wrap (f_cur returns to mark before hi == lo)
        if f_cur as usize == st.mark {
            emit_open_serial(st, out);
            return;
        }
    }
}

// ============================================================================
// 3. AnalogueC-LPTQ — Single-Pass In-Place Path-Compressing Local-Period Tube Quotient
//    Implements Steps C2/C3/C5/C6 substitutes over Open<16>:
//    - Scans edges/codes sequentially for width >= min_w affine runs
//    - Absorbs downstream affine blocks [d, d+w) with uniform code (disjoint from [a,a+w) & mark)
//    - Spells each tube word ONCE into contiguous spell buffer
//    - Preserves exact bijection on active rows (supporting BOTH serial and XOR2 emit!)
// ============================================================================

struct LptqState {
    open: OpenState,
    tubes_contracted: usize,
    hops_eliminated: usize,
    extra_spell_bytes: usize,
}

fn contract_lptq(base: &OpenState, min_w: usize, build_xor: bool) -> LptqState {
    let m = base.m;
    let mark = base.mark;
    let mut edges = base.edges.clone();
    let mut codes = base.codes.clone();
    let mut spell = base.spell.clone();
    let mut word_off = base.word_off.clone();
    let mut word_len = base.word_len.clone();

    let mut bypassed = vec![0u64; (m + 63) / 64];
    let is_bypassed = |bp: &[u64], idx: usize| -> bool { (bp[idx >> 6] >> (idx & 63)) & 1 != 0 };
    let set_bypassed = |bp: &mut [u64], idx: usize| {
        bp[idx >> 6] |= 1u64 << (idx & 63);
    };

    let mut tubes_contracted = 0usize;
    let mut hops_eliminated = 0usize;
    let orig_spell_len = spell.len();
    let mut chain_codes: Vec<u32> = Vec::with_capacity(64);

    let mut a = 0usize;
    while a < m {
        if is_bypassed(&bypassed, a) {
            a += 1;
            continue;
        }
        // Find maximal contiguous non-bypassed affine run [a, end) with same code
        let c0 = codes[a];
        let mut end = a + 1;
        while end < m
            && !is_bypassed(&bypassed, end)
            && codes[end] == c0
            && edges[end] == edges[end - 1].wrapping_add(1)
        {
            end += 1;
        }
        let w = end - a;
        if w < min_w {
            a = end;
            continue;
        }

        // Try chasing forward from [a, a+w) while image [d, d+w) is uniform-code, disjoint,
        // does not contain mark, and does not exceed u16 word_len!
        chain_codes.clear();
        chain_codes.push(c0);
        let mut tot_bytes = word_len[c0 as usize] as usize;
        let mut cur_d = edges[a] as usize;
        let mut absorbed_steps = 0usize;
        let mut final_edges_src: Option<usize> = None;

        loop {
            // Check bounds, disjointness from [a, a+w), and mark exclusion
            if cur_d + w > m {
                break;
            }
            if !(cur_d + w <= a || a + w <= cur_d) {
                break;
            }
            if mark >= cur_d && mark < cur_d + w {
                break;
            }
            // Check all w rows in [cur_d, cur_d+w) are non-bypassed and share the exact same code!
            let cd = codes[cur_d];
            let add_len = word_len[cd as usize] as usize;
            if tot_bytes + add_len > 60_000 {
                break;
            }
            let mut uniform = !is_bypassed(&bypassed, cur_d);
            if uniform {
                for r in 1..w {
                    if is_bypassed(&bypassed, cur_d + r) || codes[cur_d + r] != cd {
                        uniform = false;
                        break;
                    }
                }
            }
            if !uniform {
                break;
            }

            // Absorb [cur_d, cur_d+w)! Mark all w rows bypassed!
            for r in 0..w {
                set_bypassed(&mut bypassed, cur_d + r);
            }
            chain_codes.push(cd);
            tot_bytes += add_len;
            absorbed_steps += 1;
            final_edges_src = Some(cur_d);

            // Check if [cur_d, cur_d+w) ALSO exits affinely so we can keep chaining!
            let nxt_d = edges[cur_d] as usize;
            let mut affine_exit = true;
            for r in 1..w {
                if edges[cur_d + r] != (nxt_d as u32).wrapping_add(r as u32) {
                    affine_exit = false;
                    break;
                }
            }
            if !affine_exit {
                break;
            }
            cur_d = nxt_d;
        }

        if absorbed_steps > 0 {
            let last_src = final_edges_src.unwrap();
            // Create new macro-word in forward text order:
            // Note: step 0 spells LAST in text (closest to end), step k spells FIRST in text!
            // So forward-text concatenation is reverse order of chain_codes!
            let new_code = word_off.len() as u32;
            let off = spell.len() as u32;
            for &cid in chain_codes.iter().rev() {
                let co = word_off[cid as usize] as usize;
                let cl = word_len[cid as usize] as usize;
                // Copy from existing spell slice cleanly without borrowing overlap
                let cur_len = spell.len();
                spell.resize(cur_len + cl, 0);
                spell.copy_within(co..co + cl, cur_len);
            }
            word_off.push(off);
            word_len.push(tot_bytes as u16);

            // Rewire entry rows [a, a+w) directly to edges[last_src .. last_src+w]!
            for r in 0..w {
                edges[a + r] = edges[last_src + r];
                codes[a + r] = new_code;
            }
            tubes_contracted += 1;
            hops_eliminated += w * absorbed_steps;
        }

        a = end;
    }

    let mut xor_edges = Vec::new();
    let mut mark_prev = 0usize;
    if build_xor {
        xor_edges.resize(m, 0u32);
        for u in 0..m {
            if !is_bypassed(&bypassed, u) {
                let nxt = edges[u] as usize;
                xor_edges[u] ^= nxt as u32;
                xor_edges[nxt] ^= u as u32;
                if nxt == mark {
                    mark_prev = u;
                }
            }
        }
    }

    let extra_spell_bytes = spell.len() - orig_spell_len;
    LptqState {
        open: OpenState {
            n: base.n,
            m,
            mark,
            edges,
            codes,
            spell,
            word_off,
            word_len,
            xor_edges,
            mark_prev,
        },
        tubes_contracted,
        hops_eliminated,
        extra_spell_bytes,
    }
}

// ============================================================================
// 4. DSID — MSC 94B Differential-Syndrome Interval Decoder
//    Stores ONLY the m_syn non-zero differential-syndrome intervals (a_i -> b_i, code_i)
//    plus a tiny 256-entry block-start directory to shrink binary search range.
// ============================================================================

struct DsidState {
    n: usize,
    mark: usize,
    /// Sorted start ranks of the m_syn non-zero syndrome intervals
    starts: Vec<u32>,
    /// Target start rank for each interval
    dsts: Vec<u32>,
    /// Band word code for each interval
    codes: Vec<u32>,
    /// 1024-bucket direct accelerator table (rank >> shift -> interval range)
    bucket_idx: Vec<u32>,
    shift: u32,
    spell: Vec<u8>,
    word_off: Vec<u32>,
    word_len: Vec<u16>,
}

fn build_dsid(base: &OpenState) -> DsidState {
    let m = base.m;
    let mut starts = Vec::with_capacity(m / 2);
    let mut dsts = Vec::with_capacity(m / 2);
    let mut codes = Vec::with_capacity(m / 2);

    let mut i = 0;
    while i < m {
        let mut j = i + 1;
        while j < m
            && base.edges[j] == base.edges[j - 1].wrapping_add(1)
            && base.codes[j] == base.codes[i]
        {
            j += 1;
        }
        starts.push(i as u32);
        dsts.push(base.edges[i]);
        codes.push(base.codes[i]);
        i = j;
    }

    // Build 1024-bucket index to cut binary search from ~17 steps to ~7 steps
    let shift = if m > 1024 {
        (usize::BITS - (m / 1024).leading_zeros()).max(1)
    } else {
        1
    };
    let nbuckets = (m >> shift) + 2;
    let mut bucket_idx = vec![0u32; nbuckets];
    let mut seg = 0usize;
    for b in 0..nbuckets {
        let pos = (b << shift) as u32;
        while seg + 1 < starts.len() && starts[seg + 1] <= pos {
            seg += 1;
        }
        bucket_idx[b] = seg as u32;
    }

    DsidState {
        n: base.n,
        mark: base.mark,
        starts,
        dsts,
        codes,
        bucket_idx,
        shift,
        spell: base.spell.clone(),
        word_off: base.word_off.clone(),
        word_len: base.word_len.clone(),
    }
}

fn emit_dsid(st: &DsidState, out: &mut [u8]) {
    let mut pos = st.n;
    let mut cur = st.mark as u32;
    let ns = st.starts.len();
    while pos > 0 {
        let b = (cur as usize) >> st.shift;
        let mut lo = st.bucket_idx[b] as usize;
        let mut hi = (st.bucket_idx[b + 1] as usize + 1).min(ns);
        // Binary search for largest idx in [lo, hi) with starts[idx] <= cur
        while lo + 1 < hi {
            let mid = (lo + hi) >> 1;
            if st.starts[mid] <= cur {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let offset = cur - st.starts[lo];
        let code = st.codes[lo] as usize;
        let len = st.word_len[code] as usize;
        let off = st.word_off[code] as usize;
        if len <= pos {
            pos -= len;
            out[pos..pos + len].copy_from_slice(&st.spell[off..off + len]);
        } else {
            let take = pos;
            out[0..take].copy_from_slice(&st.spell[off + len - take..off + len]);
            pos = 0;
        }
        cur = st.dsts[lo] + offset;
    }
}

// ============================================================================
// 5. MSC 46 Per-Byte Neumann Resolvent Correlation Check
// ============================================================================

fn check_per_byte_neumann(l_col: &[u8], base: &OpenState) -> (f64, f64) {
    let n = l_col.len();
    let (_, c_arr) = build_lf(l_col);
    let mut active = Vec::new();
    let mut b2a = [usize::MAX; 256];
    for b in 0..256 {
        if c_arr[b + 1] > c_arr[b] {
            b2a[b] = active.len();
            active.push(b as u8);
        }
    }
    let k = active.len();
    let mut run_w = vec![0usize; n];
    let mut i = 0;
    while i < n {
        let b = l_col[i];
        let mut j = i + 1;
        while j < n && l_col[j] == b {
            j += 1;
        }
        let w = j - i;
        for slot in &mut run_w[i..j] {
            *slot = w;
        }
        i = j;
    }
    let mut p_thick = vec![vec![0.0f64; k]; k];
    for fi in 0..k {
        let bf = active[fi] as usize;
        let denom = (c_arr[bf + 1] - c_arr[bf]) as f64;
        for x in c_arr[bf]..c_arr[bf + 1] {
            if run_w[x] > LIMIT {
                let li = b2a[l_col[x] as usize];
                p_thick[fi][li] += 1.0 / denom;
            }
        }
    }
    // e = (I - P_thick)^{-1} 1
    let mut e_vec = vec![1.0f64; k];
    let mut term = vec![1.0f64; k];
    let mut next = vec![0.0f64; k];
    for _ in 0..400 {
        let mut mx = 0.0f64;
        for r in 0..k {
            let mut s = 0.0;
            for c in 0..k {
                s += p_thick[r][c] * term[c];
            }
            next[r] = s;
            if s > mx {
                mx = s;
            }
        }
        for r in 0..k {
            e_vec[r] += next[r];
            term[r] = next[r];
        }
        if mx < 1e-10 {
            break;
        }
    }

    // Compare e_vec[b] against TRUE Open<16> mean word length per starting L-byte b!
    let mut true_sum = vec![0u64; k];
    let mut true_cnt = vec![0u64; k];
    for j in 0..base.m {
        let code = base.codes[j] as usize;
        let len = base.word_len[code] as usize;
        let off = base.word_off[code] as usize;
        // Forward-order last byte is the starting L[x] byte!
        let start_b = base.spell[off + len - 1] as usize;
        let ai = b2a[start_b];
        true_sum[ai] += len as u64;
        true_cnt[ai] += 1;
    }

    // Weighted Pearson r and relative MAE across bytes with >= 50 thin rows
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut ws = Vec::new();
    for ai in 0..k {
        if true_cnt[ai] >= 50 {
            xs.push(e_vec[ai]);
            ys.push(true_sum[ai] as f64 / true_cnt[ai] as f64);
            ws.push(true_cnt[ai] as f64);
        }
    }
    let wsum: f64 = ws.iter().sum();
    let mx: f64 = xs.iter().zip(&ws).map(|(x, w)| x * w).sum::<f64>() / wsum;
    let my: f64 = ys.iter().zip(&ws).map(|(y, w)| y * w).sum::<f64>() / wsum;
    let mut cov = 0.0;
    let mut vx = 0.0;
    let mut vy = 0.0;
    let mut w_abs_err = 0.0;
    for idx in 0..xs.len() {
        let dx = xs[idx] - mx;
        let dy = ys[idx] - my;
        cov += ws[idx] * dx * dy;
        vx += ws[idx] * dx * dx;
        vy += ws[idx] * dy * dy;
        w_abs_err += ws[idx] * (xs[idx] - ys[idx]).abs();
    }
    let pearson = cov / (vx * vy).sqrt().max(1e-12);
    let rel_mae = (w_abs_err / wsum) / my.max(1e-12);
    (pearson, rel_mae)
}

// ============================================================================
// 6. Non-Wikidata Synthetic Structured Log + Repeated Hashes + Unique UUIDs
// ============================================================================

fn generate_non_wikidata_log(target_len: usize) -> Vec<u8> {
    let mut rng = XorShift64(0xC46A_C0DE_5EED_1234);
    let hex = b"0123456789abcdef";
    // Pool of 60 repeated 40-hex session hashes (each repeated ~8-12x)
    let mut hashes = Vec::new();
    for _ in 0..60 {
        let mut h = [0u8; 40];
        for b in &mut h {
            *b = hex[(rng.next() & 15) as usize];
        }
        hashes.push(h);
    }
    let endpoints = [
        "/api/v2/events/ingest",
        "/api/v2/users/profile",
        "/api/v2/billing/invoice",
        "/api/v2/search/query",
        "/api/v2/metrics/export",
    ];
    let statuses = ["200", "201", "204", "400", "404"];
    let mut out = Vec::with_capacity(target_len + 256);
    let mut seq = 10_000usize;
    while out.len() < target_len {
        let ep = endpoints[(rng.next() as usize) % endpoints.len()];
        let st = statuses[(rng.next() as usize) % statuses.len()];
        let h = &hashes[(rng.next() as usize) % hashes.len()];
        // Unique UUIDv4
        let mut uuid = [0u8; 36];
        for i in 0..36 {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                uuid[i] = b'-';
            } else {
                uuid[i] = hex[(rng.next() & 15) as usize];
            }
        }
        let line = format!(
            "{{\"ts\":\"2026-11-22T04:{:02}:{:02}Z\",\"req_id\":\"{}\",\"session\":\"{}\",\"path\":\"{}\",\"status\":{},\"seq\":{}}}\n",
            (seq / 60) % 60,
            seq % 60,
            std::str::from_utf8(&uuid).unwrap(),
            std::str::from_utf8(h).unwrap(),
            ep,
            st,
            seq
        );
        out.extend_from_slice(line.as_bytes());
        seq += 1;
    }
    out.truncate(target_len);
    out
}

// ============================================================================
// 7. Exhaustive Exactness Audit Suite
// ============================================================================

fn verify_all_decoders(text: &[u8], limit: usize, check_exact_periodic: bool) {
    let (l_col, primary) = build_bwt(text);
    let naive = invert_naive(&l_col, primary);
    assert_eq!(naive, text, "naive BWT inverse mismatch!");

    if let Some((per_out, _g)) = recognise_and_invert_exact_periodic(&l_col, primary) {
        assert_eq!(per_out, text, "ExactPeriodicFast mismatch!");
    } else if check_exact_periodic {
        panic!("ExactPeriodicFast failed to recognise exact periodic text!");
    }

    let base = build_open_state(&l_col, primary, limit);
    let mut buf = vec![0u8; text.len()];
    emit_open_serial(&base, &mut buf);
    assert_eq!(buf, text, "Open16-Serial mismatch!");

    buf.fill(0);
    emit_open_xor2(&base, &mut buf);
    assert_eq!(buf, text, "Open16-XOR2 mismatch!");

    for &min_w in &[2usize, 3usize] {
        let lptq = contract_lptq(&base, min_w, true);
        buf.fill(0);
        emit_open_serial(&lptq.open, &mut buf);
        assert_eq!(buf, text, "LPTQ-Serial (min_w={}) mismatch!", min_w);

        buf.fill(0);
        emit_open_xor2(&lptq.open, &mut buf);
        assert_eq!(buf, text, "LPTQ-XOR2 (min_w={}) mismatch!", min_w);
    }

    let dsid = build_dsid(&base);
    buf.fill(0);
    emit_dsid(&dsid, &mut buf);
    assert_eq!(buf, text, "DSID mismatch!");
}

fn run_exhaustive_exactness_audit() {
    println!("=== EXHAUSTIVE EXACTNESS AUDIT (Analogue C Transplanted Chain) ===");
    let mut total_cases = 0usize;

    // A. All binary strings n = 1..=10 (2,046 strings)
    for n in 1..=10usize {
        for mask in 0..(1usize << n) {
            let mut t = Vec::with_capacity(n);
            for bit in 0..n {
                t.push(if (mask >> bit) & 1 == 0 { b'a' } else { b'b' });
            }
            verify_all_decoders(&t, 2, false);
            total_cases += 1;
        }
    }

    // B. All ternary strings n = 1..=6 (1,092 strings)
    for n in 1..=6usize {
        let count = 3usize.pow(n as u32);
        for mut v in 0..count {
            let mut t = Vec::with_capacity(n);
            for _ in 0..n {
                t.push(b'a' + (v % 3) as u8);
                v /= 3;
            }
            verify_all_decoders(&t, 2, false);
            total_cases += 1;
        }
    }

    // C. 2,500 random multi-alphabet strings (n in 8..=256, alphabet 2..=64)
    let mut rng = XorShift64(0x9E37_79B9_7F4A_7C15);
    for _ in 0..2_500 {
        let n = 8 + (rng.next() as usize % 249);
        let alpha = [2usize, 4, 8, 16, 64][(rng.next() as usize) % 5];
        let mut t = Vec::with_capacity(n);
        for _ in 0..n {
            t.push(b' ' + (rng.next() as usize % alpha) as u8);
        }
        verify_all_decoders(&t, 4, false);
        total_cases += 1;
    }

    // D. Exact periodic texts u^q (d in 1..=16, q in 2..=8) — 112 exact periodic cases
    let mut periodic_cases = 0usize;
    for d in 1..=16usize {
        let mut u = Vec::with_capacity(d);
        for _ in 0..d {
            u.push(b'a' + (rng.next() as usize % 6) as u8);
        }
        for q in 2..=8usize {
            let mut t = Vec::with_capacity(d * q);
            for _ in 0..q {
                t.extend_from_slice(&u);
            }
            verify_all_decoders(&t, 4, true);
            periodic_cases += 1;
            total_cases += 1;
        }
    }

    // E. Perturbed almost-periodic texts (u^q with k=1,2,5,20 random byte mutations)
    let mut almost_cases = 0usize;
    let base_u = b"{\"k\":\"v\",\"h\":\"0123456789abcdef\"},";
    for &muts in &[1usize, 2, 5, 20, 50] {
        let mut t = Vec::new();
        for _ in 0..40 {
            t.extend_from_slice(base_u);
        }
        for _ in 0..muts {
            let pos = (rng.next() as usize) % t.len();
            t[pos] = b'A' + (rng.next() as usize % 26) as u8;
        }
        verify_all_decoders(&t, LIMIT, false);
        almost_cases += 1;
        total_cases += 1;
    }

    // F. Malformed / multi-cycle doubled columns (Bug #1 regression: L="bbaabb", p=0 & d=2..=8, g=2)
    assert!(
        recognise_and_invert_exact_periodic(b"bbaabb", 0).is_none(),
        "ExactPeriodicFast MUST reject non-dividing multi-cycle malformed column L=\"bbaabb\", p=0!"
    );
    let bbaabb_ref = invert_naive(b"bbaabb", 0);
    let bbaabb_open = build_open_state(b"bbaabb", 0, 2);
    let mut bbaabb_buf = vec![0u8; 6];
    emit_open_serial(&bbaabb_open, &mut bbaabb_buf);
    assert_eq!(bbaabb_buf, bbaabb_ref, "Fallback Open16 mismatch on L=\"bbaabb\", p=0");

    let mut malformed_doubled = 0usize;
    let mut malformed_rejected = 0usize;
    let mut malformed_accepted = 0usize;
    for d in 2..=8usize {
        for mask in 0..(1usize << d) {
            let mut l_col = Vec::with_capacity(2 * d);
            for bit in 0..d {
                let b = if (mask >> bit) & 1 == 0 { b'a' } else { b'b' };
                l_col.push(b);
                l_col.push(b);
            }
            for p in 0..(2 * d) {
                let want = invert_naive(&l_col, p);
                if let Some((got, _g)) = recognise_and_invert_exact_periodic(&l_col, p) {
                    assert_eq!(
                        got, want,
                        "ExactPeriodicFast accepted malformed doubled column d={d} mask={mask} p={p} with wrong output!"
                    );
                    malformed_accepted += 1;
                } else {
                    malformed_rejected += 1;
                }
                malformed_doubled += 1;
            }
        }
    }
    assert!(malformed_rejected > 0, "expected non-dividing doubled columns to be rejected");

    println!(
        "  PASSED {total_cases} synthetic cases (2,046 binary + 1,092 ternary + 2,500 random + {periodic_cases} exact periodic + {almost_cases} perturbed almost-periodic) + {malformed_doubled} malformed doubled columns ({malformed_accepted} exact-accepted, {malformed_rejected} cleanly rejected + fallback exact): ALL 6 decoders (Naive, ExactPeriodicFast, Open16-Serial, Open16-XOR2, LPTQ-Serial/XOR2 w>=2,3, DSID) 100% BYTE-EXACT [VERIFIED]"
    );
}

// ============================================================================
// 8. Native Timing & Memory Benchmark (1T, 2T, 4W) on 6 Blocks + Non-Wikidata + Periodic
// ============================================================================

struct BenchBlock {
    name: String,
    l_col: Vec<u8>,
    primary: usize,
    expected: Vec<u8>,
    open: OpenState,
    lptq2_ser: LptqState,
    lptq2_xor: LptqState,
    lptq3_xor: LptqState,
    dsid: DsidState,
}

fn bench_us(rounds: usize, mut f: impl FnMut()) -> f64 {
    // Warmup
    f();
    let mut times = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let t0 = Instant::now();
        f();
        times.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[rounds / 2] // median us
}

fn bench_parallel_sum_us(workers: usize, passes: usize, blocks: &Arc<Vec<BenchBlock>>, mode: usize) -> f64 {
    let mut times = Vec::with_capacity(passes);
    for _ in 0..passes {
        let t0 = Instant::now();
        let mut handles = Vec::with_capacity(workers);
        for w in 0..workers {
            let blks = Arc::clone(blocks);
            handles.push(thread::spawn(move || {
                let mut out = vec![0u8; 900_000];
                // Each worker processes all 6 Wikidata blocks (index 0..6), rotated by w
                for idx in 0..6 {
                    let b = &blks[(idx + w) % 6];
                    match mode {
                        0 => emit_open_serial(&b.open, &mut out[..b.open.n]),
                        1 => {
                            let lq = contract_lptq(&b.open, 2, false);
                            emit_open_serial(&lq.open, &mut out[..lq.open.n]);
                        }
                        2 => emit_open_xor2(&b.open, &mut out[..b.open.n]),
                        3 => {
                            let lq = contract_lptq(&b.open, 2, true);
                            emit_open_xor2(&lq.open, &mut out[..lq.open.n]);
                        }
                        4 => {
                            let lq = contract_lptq(&b.open, 3, true);
                            emit_open_xor2(&lq.open, &mut out[..lq.open.n]);
                        }
                        5 => emit_open_xor2(&b.lptq2_xor.open, &mut out[..b.open.n]),
                        6 => emit_dsid(&b.dsid, &mut out[..b.open.n]),
                        _ => unreachable!(),
                    }
                }
                out[0]
            }));
        }
        let mut sink = 0u8;
        for h in handles {
            sink ^= h.join().unwrap();
        }
        std::hint::black_box(sink);
        times.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[passes / 2]
}

fn main() {
    run_exhaustive_exactness_audit();

    let base_dir = Path::new("/home/antigravity/admitted-bwt");
    let mut bench_blocks: Vec<BenchBlock> = Vec::new();

    println!("\n=== LOADING & VERIFYING 6 WIKIDATA BLOCKS + NON-WIKIDATA LOG + EXACT PERIODIC ===");
    for &(fname, primary) in BLOCKS {
        let l_col = fs::read(base_dir.join(fname)).expect("read block");
        let expected = invert_naive(&l_col, primary);
        assert!(recognise_and_invert_exact_periodic(&l_col, primary).is_none());

        let open = build_open_state(&l_col, primary, LIMIT);
        let lptq2_ser = contract_lptq(&open, 2, false);
        let lptq2_xor = contract_lptq(&open, 2, true);
        let lptq3_xor = contract_lptq(&open, 3, true);
        let dsid = build_dsid(&open);

        // Byte-for-byte verification of EVERY assembled variant on EVERY block!
        let mut out = vec![0u8; expected.len()];
        emit_open_serial(&open, &mut out);
        assert_eq!(out, expected);
        out.fill(0);
        emit_open_xor2(&open, &mut out);
        assert_eq!(out, expected);
        out.fill(0);
        emit_open_serial(&lptq2_ser.open, &mut out);
        assert_eq!(out, expected);
        out.fill(0);
        emit_open_xor2(&lptq2_xor.open, &mut out);
        assert_eq!(out, expected);
        out.fill(0);
        emit_open_xor2(&lptq3_xor.open, &mut out);
        assert_eq!(out, expected);
        out.fill(0);
        emit_dsid(&dsid, &mut out);
        assert_eq!(out, expected);

        let (pearson, rel_mae) = check_per_byte_neumann(&l_col, &open);
        println!(
            "  {:12} | m={:6} | LPTQ(w>=2): tubes={:5}, elim={:6} ({:4.1}%), +spell={:5.1} KiB | LPTQ(w>=3): elim={:6} ({:4.1}%) | DSID: m_syn={:6} ({:4.1}%) | MSC46 per-byte Neumann r={:.3}, relMAE={:.3} [EXACT]",
            fname,
            open.m,
            lptq2_xor.tubes_contracted,
            lptq2_xor.hops_eliminated,
            100.0 * lptq2_xor.hops_eliminated as f64 / open.m as f64,
            lptq2_xor.extra_spell_bytes as f64 / 1024.0,
            lptq3_xor.hops_eliminated,
            100.0 * lptq3_xor.hops_eliminated as f64 / open.m as f64,
            dsid.starts.len(),
            100.0 * dsid.starts.len() as f64 / open.m as f64,
            pearson,
            rel_mae,
        );

        bench_blocks.push(BenchBlock {
            name: fname.to_string(),
            l_col,
            primary,
            expected,
            open,
            lptq2_ser,
            lptq2_xor,
            lptq3_xor,
            dsid,
        });
    }

    // Non-Wikidata synthetic structured log block (300,000 bytes)
    let log_text = generate_non_wikidata_log(300_000);
    let (log_l, log_p) = build_bwt(&log_text);
    assert!(recognise_and_invert_exact_periodic(&log_l, log_p).is_none());
    let log_open = build_open_state(&log_l, log_p, LIMIT);
    let log_lptq2_ser = contract_lptq(&log_open, 2, false);
    let log_lptq2_xor = contract_lptq(&log_open, 2, true);
    let log_lptq3_xor = contract_lptq(&log_open, 3, true);
    let log_dsid = build_dsid(&log_open);
    let mut out = vec![0u8; log_text.len()];
    emit_open_xor2(&log_lptq2_xor.open, &mut out);
    assert_eq!(out, log_text);
    out.fill(0);
    emit_dsid(&log_dsid, &mut out);
    assert_eq!(out, log_text);
    let (log_r, log_mae) = check_per_byte_neumann(&log_l, &log_open);
    println!(
        "  {:12} | m={:6} | LPTQ(w>=2): tubes={:5}, elim={:6} ({:4.1}%), +spell={:5.1} KiB | LPTQ(w>=3): elim={:6} ({:4.1}%) | DSID: m_syn={:6} ({:4.1}%) | MSC46 per-byte Neumann r={:.3}, relMAE={:.3} [EXACT]",
        "non-wiki-300k",
        log_open.m,
        log_lptq2_xor.tubes_contracted,
        log_lptq2_xor.hops_eliminated,
        100.0 * log_lptq2_xor.hops_eliminated as f64 / log_open.m as f64,
        log_lptq2_xor.extra_spell_bytes as f64 / 1024.0,
        log_lptq3_xor.hops_eliminated,
        100.0 * log_lptq3_xor.hops_eliminated as f64 / log_open.m as f64,
        log_dsid.starts.len(),
        100.0 * log_dsid.starts.len() as f64 / log_open.m as f64,
        log_r,
        log_mae,
    );
    bench_blocks.push(BenchBlock {
        name: "non-wiki-300k".to_string(),
        l_col: log_l,
        primary: log_p,
        expected: log_text,
        open: log_open,
        lptq2_ser: log_lptq2_ser,
        lptq2_xor: log_lptq2_xor,
        lptq3_xor: log_lptq3_xor,
        dsid: log_dsid,
    });

    // Exact periodic synthetic 300k block (d=150, q=2000) to time ExactPeriodicFast!
    let mut per_u = Vec::with_capacity(150);
    let mut rng = XorShift64(0x1234_5678_9ABC_DEF0);
    for _ in 0..150 {
        per_u.push(b'a' + (rng.next() as usize % 26) as u8);
    }
    let mut per_text = Vec::with_capacity(300_000);
    for _ in 0..2_000 {
        per_text.extend_from_slice(&per_u);
    }
    let (per_l, per_p) = build_bwt(&per_text);
    let (per_dec, per_g) = recognise_and_invert_exact_periodic(&per_l, per_p).expect("exact periodic");
    assert_eq!(per_dec, per_text);
    let per_fast_us = bench_us(25, || {
        let res = recognise_and_invert_exact_periodic(&per_l, per_p).unwrap();
        std::hint::black_box(res.0[0]);
    });
    println!(
        "  {:12} | d=150, q={} (n=300,000) | ExactPeriodicFast full decode (recognise+invert+tile) = {:.1} us [EXACT]",
        "periodic-300k", per_g, per_fast_us
    );

    // ========================================================================
    // 1T Native Per-Block Timing: Recognise, LPTQ Contract, Emit (Serial/XOR2), DSID
    // ========================================================================
    println!("\n=== 1T NATIVE TIMING BENCHMARK (median of 15 rounds, us) ===");
    println!(
        "{:<14} | {:>7} | {:>8} {:>8} {:>8} {:>8} | {:>8} {:>8} {:>8} {:>8} | {:>8} {:>8}",
        "block", "C_recog", "Op16-Ser", "LQ2-Ctr", "LQ2-Emit", "LQ2-Tot",
        "Op16-XR2", "LQ2X-Ctr", "LQ2X-Emt", "LQ2X-Tot", "DSID-Pck", "DSID-Emt"
    );

    let mut sum_recog = 0.0;
    let mut sum_op_ser = 0.0;
    let mut sum_lq2_ctr = 0.0;
    let mut sum_lq2_emt = 0.0;
    let mut sum_op_xr2 = 0.0;
    let mut sum_lq2x_ctr = 0.0;
    let mut sum_lq2x_emt = 0.0;
    let mut sum_lq3x_ctr = 0.0;
    let mut sum_lq3x_emt = 0.0;
    let mut sum_dsid_pck = 0.0;
    let mut sum_dsid_emt = 0.0;

    let mut scratch_out = vec![0u8; 900_000];
    for (idx, b) in bench_blocks.iter().enumerate() {
        let recog_us = bench_us(15, || {
            let r = recognise_and_invert_exact_periodic(&b.l_col, b.primary);
            std::hint::black_box(r.is_some());
        });
        let op_ser_us = bench_us(15, || {
            emit_open_serial(&b.open, &mut scratch_out[..b.open.n]);
            std::hint::black_box(scratch_out[0]);
        });
        let lq2_ctr_us = bench_us(15, || {
            let st = contract_lptq(&b.open, 2, false);
            std::hint::black_box(st.hops_eliminated);
        });
        let lq2_emt_us = bench_us(15, || {
            emit_open_serial(&b.lptq2_ser.open, &mut scratch_out[..b.open.n]);
            std::hint::black_box(scratch_out[0]);
        });
        let op_xr2_us = bench_us(15, || {
            emit_open_xor2(&b.open, &mut scratch_out[..b.open.n]);
            std::hint::black_box(scratch_out[0]);
        });
        let lq2x_ctr_us = bench_us(15, || {
            let st = contract_lptq(&b.open, 2, true);
            std::hint::black_box(st.hops_eliminated);
        });
        let lq2x_emt_us = bench_us(15, || {
            emit_open_xor2(&b.lptq2_xor.open, &mut scratch_out[..b.open.n]);
            std::hint::black_box(scratch_out[0]);
        });
        let lq3x_ctr_us = bench_us(15, || {
            let st = contract_lptq(&b.open, 3, true);
            std::hint::black_box(st.hops_eliminated);
        });
        let lq3x_emt_us = bench_us(15, || {
            emit_open_xor2(&b.lptq3_xor.open, &mut scratch_out[..b.open.n]);
            std::hint::black_box(scratch_out[0]);
        });
        let dsid_pck_us = bench_us(15, || {
            let st = build_dsid(&b.open);
            std::hint::black_box(st.starts.len());
        });
        let dsid_emt_us = bench_us(15, || {
            emit_dsid(&b.dsid, &mut scratch_out[..b.open.n]);
            std::hint::black_box(scratch_out[0]);
        });

        println!(
            "{:<14} | {:7.2} | {:8.1} {:8.1} {:8.1} {:8.1} | {:8.1} {:8.1} {:8.1} {:8.1} | {:8.1} {:8.1}",
            b.name,
            recog_us,
            op_ser_us,
            lq2_ctr_us,
            lq2_emt_us,
            lq2_ctr_us + lq2_emt_us,
            op_xr2_us,
            lq2x_ctr_us,
            lq2x_emt_us,
            lq2x_ctr_us + lq2x_emt_us,
            dsid_pck_us,
            dsid_emt_us,
        );

        if idx < 6 {
            sum_recog += recog_us;
            sum_op_ser += op_ser_us;
            sum_lq2_ctr += lq2_ctr_us;
            sum_lq2_emt += lq2_emt_us;
            sum_op_xr2 += op_xr2_us;
            sum_lq2x_ctr += lq2x_ctr_us;
            sum_lq2x_emt += lq2x_emt_us;
            sum_lq3x_ctr += lq3x_ctr_us;
            sum_lq3x_emt += lq3x_emt_us;
            sum_dsid_pck += dsid_pck_us;
            sum_dsid_emt += dsid_emt_us;
        }
    }

    println!(
        "SUM (6 wiki)   | {:7.2} | {:8.1} {:8.1} {:8.1} {:8.1} | {:8.1} {:8.1} {:8.1} {:8.1} | {:8.1} {:8.1}",
        sum_recog,
        sum_op_ser, sum_lq2_ctr, sum_lq2_emt, sum_lq2_ctr + sum_lq2_emt,
        sum_op_xr2, sum_lq2x_ctr, sum_lq2x_emt, sum_lq2x_ctr + sum_lq2x_emt,
        sum_dsid_pck, sum_dsid_emt
    );
    println!("  1T Ratios vs Open16 (6 Wikidata blocks sum):");
    println!(
        "    Serial: LPTQ2 emit-only = {:.3}x | LPTQ2 (contract+emit) = {:.3}x (delta = {:+.1} us)",
        sum_lq2_emt / sum_op_ser,
        (sum_lq2_ctr + sum_lq2_emt) / sum_op_ser,
        (sum_lq2_ctr + sum_lq2_emt) - sum_op_ser
    );
    println!(
        "    XOR2  : LPTQ2 emit-only = {:.3}x | LPTQ2 (contract+emit) = {:.3}x (delta = {:+.1} us)",
        sum_lq2x_emt / sum_op_xr2,
        (sum_lq2x_ctr + sum_lq2x_emt) / sum_op_xr2,
        (sum_lq2x_ctr + sum_lq2x_emt) - sum_op_xr2
    );
    println!(
        "    XOR2  : LPTQ3 emit-only = {:.3}x | LPTQ3 (contract+emit) = {:.3}x (delta = {:+.1} us)",
        sum_lq3x_emt / sum_op_xr2,
        (sum_lq3x_ctr + sum_lq3x_emt) / sum_op_xr2,
        (sum_lq3x_ctr + sum_lq3x_emt) - sum_op_xr2
    );
    println!(
        "    DSID  : emit-only = {:.3}x vs Op16-Ser | (pack+emit) = {:.3}x vs Op16-Ser",
        sum_dsid_emt / sum_op_ser,
        (sum_dsid_pck + sum_dsid_emt) / sum_op_ser
    );

    // ========================================================================
    // 1T / 2T / 4W Contention Scaling Benchmark (6 Wikidata blocks per worker)
    // ========================================================================
    let shared = Arc::new(bench_blocks);
    println!("\n=== 1T / 2T / 4W SCALING (median wall us across workers, 6 Wikidata blocks/worker) ===");
    for &w in &[1usize, 2usize, 4usize] {
        let t_op_ser = bench_parallel_sum_us(w, 9, &shared, 0);
        let t_lq2_ser = bench_parallel_sum_us(w, 9, &shared, 1);
        let t_op_xr2 = bench_parallel_sum_us(w, 9, &shared, 2);
        let t_lq2_xr2 = bench_parallel_sum_us(w, 9, &shared, 3);
        let t_lq3_xr2 = bench_parallel_sum_us(w, 9, &shared, 4);
        let t_lq2_emt_only = bench_parallel_sum_us(w, 9, &shared, 5);
        let t_dsid = bench_parallel_sum_us(w, 9, &shared, 6);
        println!(
            "  {}W: Op16-Ser={:7.1}us | LQ2-Ser(ctr+emt)={:7.1}us ({:.3}x) | Op16-XR2={:7.1}us | LQ2-XR2(emt)={:7.1}us ({:.3}x) | LQ2-XR2(ctr+emt)={:7.1}us ({:.3}x) | LQ3-XR2(ctr+emt)={:7.1}us ({:.3}x) | DSID-Emt={:7.1}us ({:.3}x vs Ser)",
            w,
            t_op_ser,
            t_lq2_ser, t_lq2_ser / t_op_ser,
            t_op_xr2,
            t_lq2_emt_only, t_lq2_emt_only / t_op_xr2,
            t_lq2_xr2, t_lq2_xr2 / t_op_xr2,
            t_lq3_xr2, t_lq3_xr2 / t_op_xr2,
            t_dsid, t_dsid / t_op_ser,
        );
    }

    // Verify expected field was used (avoid any dead-code lint)
    let total_verified_bytes: usize = shared.iter().map(|b| b.expected.len()).sum();
    println!("\nTotal verified block bytes (6 Wikidata + non-Wikidata): {} bytes [ALL EXACT]", total_verified_bytes);
}
