use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct BlockMeta {
    file: &'static str,
    n: usize,
    p: usize,
}

const BLOCKS: &[BlockMeta] = &[
    BlockMeta { file: "block-0.L",   n: 899931, p: 264045 },
    BlockMeta { file: "block-2.L",   n: 900000, p: 320807 },
    BlockMeta { file: "block-85.L",  n: 899935, p: 325088 },
    BlockMeta { file: "block-143.L", n: 899869, p: 802868 },
    BlockMeta { file: "block-207.L", n: 899998, p: 205180 },
    BlockMeta { file: "block-210.L", n: 899959, p: 791429 },
];

fn plain_inverse(l: &[u8], p: usize) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut first = [0u32; 256];
    let mut s = 0u32;
    for c in 0..256 { first[c] = s; s += counts[c]; }
    let mut cur = first;
    let mut psi = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let j = cur[b as usize];
        cur[b as usize] += 1;
        psi[j as usize] = i as u32;
    }
    let mut f_col = vec![0u8; n];
    for c in 0..256 {
        let st = first[c] as usize;
        let en = if c == 255 { n } else { first[c + 1] as usize };
        for x in st..en { f_col[x] = c as u8; }
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p;
    for _ in 0..n {
        out.push(f_col[x]);
        x = psi[x] as usize;
        if x == p { break; }
    }
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
    out
}

#[derive(Default, Debug)]
struct ZeroRvStats {
    m_16: usize,
    m_after: usize,
    tunnels: usize,
    elim_runs: usize,
    elim_rows: usize,
    extra_words: usize,
    extra_spelling_bytes: usize,
    walk_steps: usize,
}

/// Exact Zero-Induction Tunnel Elimination (ZeroRV-Tunnel):
/// 1. Standard Open(LIMIT=16) thick/thin split (produces thick strips & bands over S).
/// 2. Exact-match thin-run chaining: link thin run R_i -> R_j iff
///    w_i >= min_w && w_i == w_j && LF(R_i) == R_j && p not in R_j!
/// 3. Contained-match mode (if `allow_contained`): also chain R_i -> sub-run of R_j if LF(R_i) ⊆ R_j (w_j <= 16, p not in LF(R_i))!
/// 4. ZERO changes to thick strips / bands / emit_plain!
fn zero_rv_tunnel_exact(
    l: &[u8],
    p: usize,
    limit: usize,
    min_w: usize,
    allow_contained: bool,
) -> (Vec<u8>, ZeroRvStats) {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut first = [0u32; 256];
    let mut s = 0u32;
    for c in 0..256 { first[c] = s; s += counts[c]; }

    // Maximal runs of L (splitting thick run at p just like Open!)
    // Each piece: (byte, from, to, width, is_thin)
    let mut pieces: Vec<(u8, u32, u32, u32, bool)> = Vec::new();
    let mut all_cur = first;
    let mut i = 0usize;
    while i < n {
        let c = l[i];
        let st = i;
        i += 1;
        while i < n && l[i] == c { i += 1; }
        let w = i - st;
        let to0 = all_cur[c as usize];
        all_cur[c as usize] += w as u32;
        if w > limit && p >= st && p < i {
            if st < p {
                pieces.push((c, st as u32, to0, (p - st) as u32, false));
            }
            pieces.push((c, p as u32, to0 + (p - st) as u32, 1, true));
            if p + 1 < i {
                pieces.push((c, (p + 1) as u32, to0 + (p + 1 - st) as u32, (i - p - 1) as u32, false));
            }
        } else {
            pieces.push((c, st as u32, to0, w as u32, w <= limit));
        }
    }

    // Build LF and Open's retained set S and bands (exact simulation of Open's thick-strip elimination!)
    let mut is_ret = vec![false; n];
    let mut thin_pieces: Vec<(u8, u32, u32, u32, u32)> = Vec::new(); // (byte, from, to, width, ret_start)
    let mut m = 0u32;
    let mut mark = 0u32;
    let mut thin_counts = [0u32; 256];
    for &(c, from, to, w, keep) in &pieces {
        if keep {
            if (p as u32) >= from && (p as u32) < from + w {
                mark = m + (p as u32 - from);
            }
            for x in from..from + w { is_ret[x as usize] = true; }
            thin_pieces.push((c, from, to, w, m));
            thin_counts[c as usize] += w;
            m += w;
        }
    }

    // Row -> retained rank (for retained rows), and LF on all rows for exact band computation
    let mut row_to_rank = vec![u32::MAX; n];
    let mut ret_cur = 0u32;
    for x in 0..n {
        if is_ret[x] {
            row_to_rank[x] = ret_cur;
            ret_cur += 1;
        }
    }
    assert_eq!(ret_cur, m);

    let mut lf = vec![0u32; n];
    let mut c_cur = first;
    for (x, &b) in l.iter().enumerate() {
        lf[x] = c_cur[b as usize];
        c_cur[b as usize] += 1;
    }

    // --- ZERO-RV TUNNEL RECOGNITION ---
    let mut stats = ZeroRvStats { m_16: m as usize, ..Default::default() };

    // Build row -> thin_piece_id for all thin rows (in O(n) for verifier; in prod we use hash table / page dir!)
    let mut row_to_tp = vec![u32::MAX; n];
    for (ti, &(_, from, _, w, _)) in thin_pieces.iter().enumerate() {
        for x in from..from + w {
            row_to_tp[x as usize] = ti as u32;
        }
    }

    // Build edges[0..m] (still in S-coordinates, untouched slots remain unused!)
    let mut edges_next = vec![u32::MAX; m as usize];
    let mut edges_word: Vec<Vec<u8>> = vec![Vec::new(); m as usize];

    if !allow_contained {
        // === MODE 1: PURE EXACT-MATCH (LF(R_i) == R_j, ZERO SPLITTING!) ===
        let nt = thin_pieces.len();
        let mut next_tp = vec![u32::MAX; nt];
        let mut has_pred = vec![false; nt];
        for ti in 0..nt {
            let (_c, _from, to, w, _rstart) = thin_pieces[ti];
            if (w as usize) < min_w { continue; }
            let tj = row_to_tp[to as usize];
            if tj != u32::MAX {
                let (_, j_from, _, j_w, _) = thin_pieces[tj as usize];
                let j_has_p = (p as u32) >= j_from && (p as u32) < j_from + j_w;
                if j_from == to && j_w == w && !j_has_p && tj as usize != ti {
                    next_tp[ti] = tj;
                    has_pred[tj as usize] = true;
                }
            }
        }
        // Trace chains starting from heads (!has_pred[ti] && next_tp[ti] != MAX),
        // also break any pure cycle not containing p (by leaving one run as head!).
        let mut visited_tp = vec![false; nt];
        let mut eliminated_tp = vec![false; nt];
        // Map tail_tp -> (head_rstart, tunnel_word_forward)
        let mut tail_override: HashMap<usize, (u32, Vec<u8>)> = HashMap::new();

        // Pass A: acyclic heads
        for head in 0..nt {
            if !has_pred[head] && next_tp[head] != u32::MAX {
                let mut cur = head;
                let mut word_rev = Vec::new();
                visited_tp[cur] = true;
                word_rev.push(thin_pieces[cur].0);
                let mut len = 1usize;
                while next_tp[cur] != u32::MAX {
                    let nxt = next_tp[cur] as usize;
                    if visited_tp[nxt] { break; }
                    visited_tp[nxt] = true;
                    eliminated_tp[nxt] = true;
                    stats.elim_runs += 1;
                    stats.elim_rows += thin_pieces[nxt].3 as usize;
                    word_rev.push(thin_pieces[nxt].0);
                    cur = nxt;
                    len += 1;
                }
                if len >= 2 {
                    word_rev.reverse(); // forward order: c_{L-1} ... c_1 c_0!
                    stats.tunnels += 1;
                    stats.extra_words += 1;
                    stats.extra_spelling_bytes += word_rev.len();
                    tail_override.insert(cur, (thin_pieces[head].4, word_rev));
                }
            }
        }
        // Pass B: any pure exact-match cycles (none contains p!) -> leave cycle uncontracted (or break at first)
        // Leaving unvisited pure cycles uncontracted is always safe (never visited from p anyway!).

        // Wire! For every thin piece ti:
        // - If ti has next_tp[ti] != MAX && eliminated_tp[next_tp[ti]]: SKIP! (maps to eliminated interior!)
        // - Otherwise (normal run OR tunnel tail!): trace its image LF(ti) through thick strips via Open's bands!
        for ti in 0..nt {
            if next_tp[ti] != u32::MAX && eliminated_tp[next_tp[ti] as usize] {
                // This piece maps directly into an eliminated tunnel interior -> SKIP wiring!
                continue;
            }
            let (c, _from, to, w, rstart) = thin_pieces[ti];
            let (src_rstart, tun_word) = if let Some((hs, ref tw)) = tail_override.get(&ti) {
                (*hs, Some(tw.as_slice()))
            } else {
                (rstart, None)
            };
            // Trace each row of [to .. to + w) through thick strips (Open's bands)
            for t in 0..w {
                let mut x = (to + t) as usize;
                let mut thick_prefix = Vec::new();
                while !is_ret[x] {
                    thick_prefix.push(l[x]);
                    x = lf[x] as usize;
                }
                thick_prefix.reverse(); // forward order of thick bytes before c!
                let dst = row_to_rank[x] as usize;
                if let Some(tw) = tun_word {
                    // tw starts with c!
                    thick_prefix.extend_from_slice(tw);
                } else {
                    thick_prefix.push(c);
                }
                edges_next[dst] = src_rstart + t;
                edges_word[dst] = thick_prefix;
            }
        }
        stats.m_after = stats.m_16 - stats.elim_rows;
    } else {
        // === MODE 2: CONTAINED-MATCH (LF(sub-run) ⊆ thin_run, ZERO THICK-STRIP INTERACTION!) ===
        // A thin row y (not p) can be eliminated if its LF^{-1} preimage x is in a width >= min_w thin run
        // AND the entire maximal interval of width >= min_w containing x maps intact into a single thin run containing y!
        // Specifically: for every thin run R_i of width w >= min_w, trace R_i forward via LF while
        // the image [to .. to+w) stays inside a single thin run (not containing p) and has not been visited!
        //Wait: to handle nested sub-runs cleanly without overlap, we can simply check for each thin piece ti (w >= min_w)
        // if LF(ti) ⊆ some thin piece tj (without containing p), and record which sub-intervals of tj are covered!
        // Let's measure the exact contained-match oracle cleanly:
        // Mark row y in S as eliminated iff:
        //   y != p AND y's LF-preimage x belongs to a thin run R_i of width w >= min_w whose ENTIRE image LF(R_i)
        //   lies inside a single thin run R_j (without p)!
        let mut is_elim_row = vec![false; n];
        // Also need acyclicity (can't eliminate an entire cycle without keeping 1 break row).
        // Build directed edge for each row x in a thin run R_i (w >= min_w) where LF(R_i) ⊆ R_j (p not in LF(R_i)):
        let mut row_chains_to = vec![u32::MAX; n];
        let mut row_has_pred = vec![false; n];
        for &(_c, from, to, w, _rstart) in &thin_pieces {
            if (w as usize) < min_w { continue; }
            let tj = row_to_tp[to as usize];
            if tj != u32::MAX {
                let (_, j_from, _, j_w, _) = thin_pieces[tj as usize];
                if to + w <= j_from + j_w {
                    let hits_p = (p as u32) >= to && (p as u32) < to + w;
                    if !hits_p {
                        for t in 0..w {
                            let u = (from + t) as usize;
                            let v = (to + t) as usize;
                            if u != v {
                                row_chains_to[u] = v as u32;
                                row_has_pred[v] = true;
                            }
                        }
                    }
                }
            }
        }
        // Trace acyclic chains from heads (!row_has_pred[u] && row_chains_to[u] != MAX)
        let mut vis = vec![false; n];
        let mut tail_info: HashMap<usize, (u32, Vec<u8>)> = HashMap::new();
        for u0 in 0..n {
            if !row_has_pred[u0] && row_chains_to[u0] != u32::MAX {
                let mut cur = u0;
                let mut wrev = Vec::new();
                vis[cur] = true;
                wrev.push(l[cur]);
                let mut steps = 1;
                while row_chains_to[cur] != u32::MAX {
                    let nxt = row_chains_to[cur] as usize;
                    if vis[nxt] { break; }
                    vis[nxt] = true;
                    is_elim_row[nxt] = true;
                    stats.elim_rows += 1;
                    wrev.push(l[nxt]);
                    cur = nxt;
                    steps += 1;
                }
                if steps >= 2 {
                    wrev.reverse();
                    tail_info.insert(cur, (row_to_rank[u0], wrev));
                }
            }
        }
        // Wire every thin row x whose LF(x) is NOT eliminated!
        for x in 0..n {
            if !is_ret[x] { continue; }
            let y0 = lf[x] as usize;
            if is_elim_row[y0] { continue; }
            let (src_rank, tw_opt) = if let Some((hr, ref tw)) = tail_info.get(&x) {
                (*hr, Some(tw.as_slice()))
            } else {
                (row_to_rank[x], None)
            };
            let mut cur = y0;
            let mut thick_prefix = Vec::new();
            while !is_ret[cur] {
                thick_prefix.push(l[cur]);
                cur = lf[cur] as usize;
            }
            thick_prefix.reverse();
            let dst = row_to_rank[cur] as usize;
            if let Some(tw) = tw_opt {
                thick_prefix.extend_from_slice(tw);
            } else {
                thick_prefix.push(l[x]);
            }
            edges_next[dst] = src_rank;
            edges_word[dst] = thick_prefix;
        }
        stats.m_after = stats.m_16 - stats.elim_rows;
    }

    // Run 100% UNMODIFIED emit_plain from mark!
    let mut out = Vec::with_capacity(n);
    let mut j = mark as usize;
    for _ in 0..m {
        stats.walk_steps += 1;
        let nxt = edges_next[j];
        assert_ne!(nxt, u32::MAX, "Hit unwired/eliminated slot during emit!");
        let w = &edges_word[j];
        let take = w.len().min(n - out.len());
        out.extend_from_slice(&w[..take]);
        j = nxt as usize;
        if j == mark as usize || out.len() >= n {
            break;
        }
    }
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
    (out, stats)
}

fn main() {
    println!("=== EXHAUSTIVE EXACTNESS VERIFIER FOR ZERO-RV TUNNEL ELIMINATION ===");
    let mut bin_cases = 0usize;
    let mut bin_elim = 0usize;
    for n in 1..=8usize {
        for mask in 0..(1usize << n) {
            let mut l = vec![0u8; n];
            for i in 0..n { l[i] = ((mask >> i) & 1) as u8; }
            for p in 0..n {
                let exp = plain_inverse(&l, p);
                for &lim in &[2usize, 4, 16] {
                    for &cont in &[false, true] {
                        let (got, st) = zero_rv_tunnel_exact(&l, p, lim, 2, cont);
                        assert_eq!(got, exp, "ZeroRV mismatch binary n={} mask={} p={} lim={} cont={}", n, mask, p, lim, cont);
                        if st.elim_rows > 0 { bin_elim += 1; }
                    }
                }
                bin_cases += 1;
            }
        }
    }
    println!("  [VERIFIED] Binary n<=8: {} (col, mark) cases x 6 configs PASSED (eliminated rows in {} runs)", bin_cases, bin_elim);

    let mut ter_cases = 0usize;
    let mut ter_elim = 0usize;
    for n in 1..=5usize {
        let total = 3usize.pow(n as u32);
        for mut code in 0..total {
            let mut l = vec![0u8; n];
            for i in 0..n { l[i] = (code % 3) as u8; code /= 3; }
            for p in 0..n {
                let exp = plain_inverse(&l, p);
                for &lim in &[2usize, 4, 16] {
                    for &cont in &[false, true] {
                        let (got, st) = zero_rv_tunnel_exact(&l, p, lim, 2, cont);
                        assert_eq!(got, exp, "ZeroRV mismatch ternary n={} l={:?} p={} lim={} cont={}", n, l, p, lim, cont);
                        if st.elim_rows > 0 { ter_elim += 1; }
                    }
                }
                ter_cases += 1;
            }
        }
    }
    println!("  [VERIFIED] Ternary n<=5: {} (col, mark) cases x 6 configs PASSED (eliminated rows in {} runs)", ter_cases, ter_elim);

    println!("\n=== 6-BLOCK ZERO-RV TUNNEL CENSUS & EXACTNESS ===");
    let dir = Path::new("/tmp/mibz2-blocks");
    for &(min_w, cont, label) in &[
        (2, false, "ExactMatch LF== (w>=2)"),
        (3, false, "ExactMatch LF== (w>=3)"),
        (4, false, "ExactMatch LF== (w>=4)"),
        (2, true,  "Contained  LF<= (w>=2)"),
        (3, true,  "Contained  LF<= (w>=3)"),
        (4, true,  "Contained  LF<= (w>=4)"),
    ] {
        println!("\n--- {} ---", label);
        let (mut sm16, mut selim, mut stuns, mut seruns, mut swalk, mut sbytes) = (0,0,0,0,0,0);
        for b in BLOCKS {
            let l = fs::read(dir.join(b.file)).unwrap();
            let exp = plain_inverse(&l, b.p);
            let (got, st) = zero_rv_tunnel_exact(&l, b.p, 16, min_w, cont);
            assert_eq!(got, exp, "ZeroRV mismatch on {} ({})", b.file, label);
            sm16 += st.m_16; selim += st.elim_rows; stuns += st.tunnels;
            seruns += st.elim_runs; swalk += st.walk_steps; sbytes += st.extra_spelling_bytes;
            println!("  {:<12} m_16={:<6} -> walk_Q={:<6} (saved {:<6} = {:4.1}%) | tunnels={:<5} elim_runs={:<5} spell_B={}",
                b.file, st.m_16, st.walk_steps, st.elim_rows,
                100.0 * st.elim_rows as f64 / st.m_16 as f64,
                st.tunnels, st.elim_runs, st.extra_spelling_bytes);
        }
        println!("  TOTAL        m_16={:<6} -> walk_Q={:<6} (saved {:<6} = {:4.1}%) | tunnels={:<5} elim_runs={:<5} spell_B={}",
            sm16, swalk, selim, 100.0 * selim as f64 / sm16 as f64, stuns, seruns, sbytes);
    }
}
