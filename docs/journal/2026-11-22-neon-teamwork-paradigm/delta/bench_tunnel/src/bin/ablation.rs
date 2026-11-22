#![forbid(unsafe_code)]

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front_scalar, Runs};
use mibz2::Kernels;
use std::fs;
use std::path::Path;
use std::time::Instant;

struct BlockMeta {
    file: &'static str,
    n: usize,
    p: u32,
}

const BLOCKS: &[BlockMeta] = &[
    BlockMeta { file: "block-0.L",   n: 899931, p: 264045 },
    BlockMeta { file: "block-2.L",   n: 900000, p: 320807 },
    BlockMeta { file: "block-85.L",  n: 899935, p: 325088 },
    BlockMeta { file: "block-143.L", n: 899869, p: 802868 },
    BlockMeta { file: "block-207.L", n: 899998, p: 205180 },
    BlockMeta { file: "block-210.L", n: 899959, p: 791429 },
];

fn encode_symbols(l: &[u8], p: u32) -> Symbols {
    let mut present = [false; 256];
    for &b in l { present[b as usize] = true; }
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for b in 0..256 {
        if present[b] { seq_to_unseq[n_in_use] = b as u8; n_in_use += 1; }
    }
    let mut mtf = [0u8; 256];
    mtf[..n_in_use].copy_from_slice(&seq_to_unseq[..n_in_use]);
    let mut syms: Vec<u16> = Vec::with_capacity(l.len() / 2);
    let mut zero_run = 0usize;
    let flush_zeros = |zr: &mut usize, syms: &mut Vec<u16>| {
        let mut r = *zr;
        while r > 0 {
            if r & 1 == 1 { syms.push(0); r = (r - 1) >> 1; }
            else { syms.push(1); r = (r - 2) >> 1; }
        }
        *zr = 0;
    };
    for &b in l {
        let pos = mtf[..n_in_use].iter().position(|&x| x == b).unwrap();
        if pos == 0 { zero_run += 1; }
        else {
            flush_zeros(&mut zero_run, &mut syms);
            syms.push((pos + 1) as u16);
            for k in (1..=pos).rev() { mtf[k] = mtf[k - 1]; }
            mtf[0] = b;
        }
    }
    flush_zeros(&mut zero_run, &mut syms);
    let eob = (n_in_use + 1) as u16;
    syms.push(eob);
    Symbols { crc: 0, orig_ptr: p, seq_to_unseq, syms, eob, max_len: 900_000 }
}

/// Standalone Open-compatible edges + spelling snapshot for clean consumer-only & StaticTunnel ablation!
struct OpenSnap {
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    edges: Vec<u32>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    inline16: Vec<[u8; 16]>,
    lens: Vec<u8>,
}

fn build_snap(l: &[u8], p: u32) -> OpenSnap {
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
    let mut is_ret = vec![false; n];
    let mut i = 0;
    while i < n {
        let c = l[i];
        let st = i;
        i += 1;
        while i < n && l[i] == c { i += 1; }
        let w = i - st;
        if w <= 16 {
            for x in st..i { is_ret[x] = true; }
        } else if (p as usize) >= st && (p as usize) < i {
            is_ret[p as usize] = true;
        }
    }
    let mut ret_rows = Vec::new();
    let mut row_to_rank = vec![u32::MAX; n];
    for x in 0..n {
        if is_ret[x] {
            row_to_rank[x] = ret_rows.len() as u32;
            ret_rows.push(x as u32);
        }
    }
    let m = ret_rows.len();
    let mark = row_to_rank[p as usize];
    let bits = (u32::BITS - (m as u32).leading_zeros()).max(1);
    let mut spelling = Vec::new();
    let mut offsets = vec![0u32];
    let mut inline16: Vec<[u8; 16]> = (0..=255u8).map(|c| { let mut v = [0u8; 16]; v[0] = c; v }).collect();
    let mut lens = vec![1u8; 256];
    let mut word_map = std::collections::HashMap::<Vec<u8>, u32>::new();
    let mut edges = vec![0u32; m + 1];
    let mut buf = Vec::new();
    for (u, &row) in ret_rows.iter().enumerate() {
        buf.clear();
        let mut x = row as usize;
        loop {
            buf.push(f_col[x]);
            x = psi[x] as usize;
            if is_ret[x] { break; }
        }
        let next = row_to_rank[x];
        let code = if buf.len() == 1 {
            buf[0] as u32
        } else if let Some(&id) = word_map.get(&buf) {
            id
        } else {
            let id = (256 + offsets.len() - 1) as u32;
            spelling.extend_from_slice(&buf);
            offsets.push(spelling.len() as u32);
            let mut v = [0u8; 16];
            let h = buf.len().min(16);
            v[..h].copy_from_slice(&buf[..h]);
            inline16.push(v);
            lens.push(if buf.len() <= 16 { buf.len() as u8 } else { 255 });
            word_map.insert(buf.clone(), id);
            id
        };
        edges[u] = (code << bits) | next;
    }
    edges[m] = u32::MAX - 32;
    OpenSnap { n, m, mark, bits, edges, spelling, offsets, inline16, lens }
}

fn emit_plain(snap: &OpenSnap, out: &mut Vec<u8>) -> usize {
    out.resize(snap.n + 16, 0);
    let mut at = 0usize;
    let mask = (1u32 << snap.bits) - 1;
    let mut j = snap.mark;
    let mut steps = 0usize;
    for _ in 0..snap.m {
        steps += 1;
        let edge = snap.edges[j as usize];
        let code = (edge >> snap.bits) as usize;
        j = edge & mask;
        let len = snap.lens[code];
        if len != 255 {
            out[at..at + 16].copy_from_slice(&snap.inline16[code]);
            at += usize::from(len);
        } else {
            let w = code - 256;
            let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
            let take = word.len().min(snap.n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == snap.mark || at >= snap.n { break; }
    }
    out.truncate(at.min(snap.n));
    steps
}

/// Static pre-pass tunnel contraction on OpenSnap (modifies edges/spelling/offsets/inline16/lens in place).
/// Returns (scan_s, trace_s, tunnels_created, trace_steps).
/// Uses 1-bit L1-resident bitmaps to find tunnel heads and 1-rep tracing!
fn static_contract(
    snap: &mut OpenSnap,
    min_steps: usize,
    mlp: usize,
) -> (f64, f64, usize, usize) {
    let m = snap.m;
    let bits = snap.bits;
    let mask = (1u32 << bits) - 1;
    let mark = snap.mark as usize;
    let max_codes = (u32::MAX >> bits).saturating_sub(2) as usize;

    let t0 = Instant::now();
    // Phase 1: Sequential scan of edges[0..m] to find all 1-step strips of width >= 2
    // and mark their targets in an L1-resident bitmap `has_in` (only ~11-25 KB!).
    let words_bm = (m + 63) / 64;
    let mut has_in = vec![0u64; words_bm];
    let mut strips: Vec<(u32, u16)> = Vec::with_capacity(m / 5);

    let mark_edge = snap.edges[mark];
    snap.edges[mark] = u32::MAX - 32;
    snap.edges[m] = u32::MAX - 32;

    let mut u = 0usize;
    while u < m {
        let e = snap.edges[u];
        if snap.edges[u + 1] == e.wrapping_add(1) {
            let next = (e & mask) as usize;
            let max_w = (m - u).min(m - next);
            let mut w = 2usize;
            while w < max_w && snap.edges[u + w] == e.wrapping_add(w as u32) {
                w += 1;
            }
            // Record strip (u, w) and mark targets [next..next+w) in has_in!
            strips.push((u as u32, w as u16));
            // Mark first and last (or all w) targets in has_in:
            // Marking all w is tiny (average w ~ 3.5, all in same u64 word!)
            let end_n = next + w;
            let mut x = next;
            while x < end_n {
                has_in[x >> 6] |= 1u64 << (x & 63);
                x += 1;
            }
            u += w;
        } else {
            u += 1;
        }
    }

    // Collect tunnel heads: strips whose start `a` is NOT marked in `has_in`!
    // (Wait: if a wider strip maps partially into a narrower strip, some rows may not be marked;
    // checking `(has_in[a >> 6] >> (a & 63)) & 1 == 0` catches all pure heads!)
    let mut heads: Vec<(u32, u16)> = Vec::with_capacity(strips.len() / 4);
    for &(a, w) in &strips {
        let idx = a as usize;
        if (has_in[idx >> 6] >> (idx & 63)) & 1 == 0 {
            heads.push((a, w));
        }
    }
    let t1 = Instant::now();

    // Phase 2: Trace heads (either scalar mlp=1 or interleaved mlp=8!)
    let mut traced_bm = vec![0u64; words_bm];
    let mut tunnels_created = 0usize;
    let mut trace_steps = 0usize;

    let append_word = |code: usize,
                       spelling: &mut Vec<u8>,
                       offsets: &[u32],
                       inline16: &[[u8; 16]],
                       lens: &[u8],
                       buf: &mut Vec<u8>| {
        let len = lens[code];
        if len != 255 {
            buf.extend_from_slice(&inline16[code][..usize::from(len)]);
        } else {
            let w = code - 256;
            buf.extend_from_slice(&spelling[offsets[w] as usize..offsets[w + 1] as usize]);
        }
    };

    if mlp <= 1 {
        let mut buf = Vec::with_capacity(256);
        for &(a0, w0) in &heads {
            let a = a0 as usize;
            let mut w = w0 as usize;
            if (traced_bm[a >> 6] >> (a & 63)) & 1 != 0 {
                continue;
            }
            let e0 = snap.edges[a];
            let mut cur = (e0 & mask) as usize;
            let mut steps = 1usize;
            buf.clear();
            append_word(
                (e0 >> bits) as usize,
                &mut snap.spelling,
                &snap.offsets,
                &snap.inline16,
                &snap.lens,
                &mut buf,
            );
            loop {
                trace_steps += 1;
                if cur == mark || (traced_bm[cur >> 6] >> (cur & 63)) & 1 != 0 {
                    break;
                }
                let ce = snap.edges[cur];
                let cnext = (ce & mask) as usize;
                // Check how much of width `w` remains coupled at `cur`!
                let lim = w.min(m - cur).min(m - cnext);
                let mut nw = 1usize;
                while nw < lim && snap.edges[cur + nw] == ce.wrapping_add(nw as u32) {
                    nw += 1;
                }
                if nw < 2 {
                    break;
                }
                // Check self-overlap with [a, a + nw)
                if !(cur + nw <= a || cur >= a + nw) {
                    break;
                }
                traced_bm[cur >> 6] |= 1u64 << (cur & 63);
                w = nw;
                steps += 1;
                append_word(
                    (ce >> bits) as usize,
                    &mut snap.spelling,
                    &snap.offsets,
                    &snap.inline16,
                    &snap.lens,
                    &mut buf,
                );
                cur = cnext;
                if steps >= 4096 {
                    break;
                }
            }
            if steps >= min_steps && snap.lens.len() <= max_codes {
                let new_code = snap.lens.len() as u32;
                snap.spelling.extend_from_slice(&buf);
                snap.offsets.push(snap.spelling.len() as u32);
                let mut v = [0u8; 16];
                let h = buf.len().min(16);
                v[..h].copy_from_slice(&buf[..h]);
                snap.inline16.push(v);
                snap.lens.push(if buf.len() <= 16 { buf.len() as u8 } else { 255 });
                let shifted = new_code << bits;
                for d in 0..w {
                    snap.edges[a + d] = shifted | ((cur + d) as u32);
                }
                tunnels_created += 1;
            }
        }
    } else {
        // Interleaved MLP trace! Keep `K` active traces in flight simultaneously so L2/L3 misses overlap!
        const K: usize = 8;
        struct State {
            active: bool,
            a: usize,
            w: usize,
            cur: usize,
            steps: usize,
            buf: Vec<u8>,
        }
        let mut states: Vec<State> = (0..K)
            .map(|_| State {
                active: false,
                a: 0,
                w: 0,
                cur: 0,
                steps: 0,
                buf: Vec::with_capacity(256),
            })
            .collect();
        let mut hi = 0usize;
        let mut active_cnt = 0usize;

        // Fill initial slots
        for st in &mut states {
            while hi < heads.len() {
                let (a0, w0) = heads[hi];
                hi += 1;
                let a = a0 as usize;
                let e0 = snap.edges[a];
                st.active = true;
                st.a = a;
                st.w = w0 as usize;
                st.cur = (e0 & mask) as usize;
                st.steps = 1;
                st.buf.clear();
                append_word(
                    (e0 >> bits) as usize,
                    &mut snap.spelling,
                    &snap.offsets,
                    &snap.inline16,
                    &snap.lens,
                    &mut st.buf,
                );
                active_cnt += 1;
                break;
            }
        }

        while active_cnt > 0 {
            for st in &mut states {
                if !st.active { continue; }
                trace_steps += 1;
                let cur = st.cur;
                let done = if cur == mark || (traced_bm[cur >> 6] >> (cur & 63)) & 1 != 0 {
                    true
                } else {
                    let ce = snap.edges[cur];
                    let cnext = (ce & mask) as usize;
                    let lim = st.w.min(m - cur).min(m - cnext);
                    let mut nw = 1usize;
                    while nw < lim && snap.edges[cur + nw] == ce.wrapping_add(nw as u32) {
                        nw += 1;
                    }
                    if nw < 2 || !(cur + nw <= st.a || cur >= st.a + nw) {
                        true
                    } else {
                        traced_bm[cur >> 6] |= 1u64 << (cur & 63);
                        st.w = nw;
                        st.steps += 1;
                        append_word(
                            (ce >> bits) as usize,
                            &mut snap.spelling,
                            &snap.offsets,
                            &snap.inline16,
                            &snap.lens,
                            &mut st.buf,
                        );
                        st.cur = cnext;
                        st.steps >= 4096
                    }
                };
                if done {
                    if st.steps >= min_steps && snap.lens.len() <= max_codes {
                        let new_code = snap.lens.len() as u32;
                        snap.spelling.extend_from_slice(&st.buf);
                        snap.offsets.push(snap.spelling.len() as u32);
                        let mut v = [0u8; 16];
                        let h = st.buf.len().min(16);
                        v[..h].copy_from_slice(&st.buf[..h]);
                        snap.inline16.push(v);
                        snap.lens.push(if st.buf.len() <= 16 { st.buf.len() as u8 } else { 255 });
                        let shifted = new_code << bits;
                        for d in 0..st.w {
                            snap.edges[st.a + d] = shifted | ((st.cur + d) as u32);
                        }
                        tunnels_created += 1;
                    }
                    // Refill slot
                    st.active = false;
                    active_cnt -= 1;
                    if hi < heads.len() {
                        let (a0, w0) = heads[hi];
                        hi += 1;
                        let a = a0 as usize;
                        let e0 = snap.edges[a];
                        st.active = true;
                        st.a = a;
                        st.w = w0 as usize;
                        st.cur = (e0 & mask) as usize;
                        st.steps = 1;
                        st.buf.clear();
                        append_word(
                            (e0 >> bits) as usize,
                            &mut snap.spelling,
                            &snap.offsets,
                            &snap.inline16,
                            &snap.lens,
                            &mut st.buf,
                        );
                        active_cnt += 1;
                    }
                }
            }
        }
    }

    snap.edges[mark] = mark_edge;
    let t2 = Instant::now();
    ((t1 - t0).as_secs_f64(), (t2 - t1).as_secs_f64(), tunnels_created, trace_steps)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    let kernels = Kernels::detect();
    let mut base_snaps = Vec::new();
    let mut expecteds = Vec::new();
    let mut runs_vec = Vec::new();
    let mut shipped = OpenCells::default();

    for b in BLOCKS {
        let l = fs::read(dir.join(b.file)).expect("read");
        let syms = encode_symbols(&l, b.p);
        let mut runs = Runs::default();
        front_scalar(&syms, &mut runs).unwrap();
        let mut exp = Vec::new();
        shipped.invert_runs(&runs, b.n, b.p, &mut exp, kernels);
        let snap = build_snap(&l, b.p);
        let mut check_out = Vec::new();
        emit_plain(&snap, &mut check_out);
        assert_eq!(check_out, exp, "snap plain mismatch on {}", b.file);
        base_snaps.push(snap);
        expecteds.push(exp);
        runs_vec.push((b.file, b.n, b.p, runs));
    }

    // 1. Measure CONSUMER-ONLY ORACLE CEILING (pre-contracted edges -> unmodified emit_plain!)
    //    AND measure exact StaticTunnel (scan + trace + emit_plain) for mlp=1 and mlp=8!
    println!("=== STATIC TUNNEL & CONSUMER-ONLY ORACLE CEILING (6 BLOCKS SUMMED, 15 ROUNDS) ===");

    // First verify exactness & collect counts on all 6 blocks
    let mut tot_m = 0usize;
    let mut tot_q_after = 0usize;
    let mut tot_trace = 0usize;
    let mut tot_tunnels = 0usize;
    let mut pre_snaps_ms2 = Vec::new();
    for (bi, snap) in base_snaps.iter().enumerate() {
        let mut s2 = OpenSnap {
            n: snap.n,
            m: snap.m,
            mark: snap.mark,
            bits: snap.bits,
            edges: snap.edges.clone(),
            spelling: snap.spelling.clone(),
            offsets: snap.offsets.clone(),
            inline16: snap.inline16.clone(),
            lens: snap.lens.clone(),
        };
        let (_, _, tuns, tsteps) = static_contract(&mut s2, 2, 1);
        let mut out = Vec::new();
        let q_after = emit_plain(&s2, &mut out);
        assert_eq!(out, expecteds[bi], "StaticTunnel mismatch on {}", BLOCKS[bi].file);
        println!("  {:<12} m={:<6} -> Q_emit={:<6} (saved {:4.1}%) | trace_Q={:<6} | total_Q={:<6} ({:+.1}%) | tunnels={}",
            BLOCKS[bi].file, snap.m, q_after,
            100.0 * (snap.m - q_after) as f64 / snap.m as f64,
            tsteps, q_after + tsteps,
            100.0 * ((q_after + tsteps) as f64 - snap.m as f64) / snap.m as f64,
            tuns);
        tot_m += snap.m;
        tot_q_after += q_after;
        tot_trace += tsteps;
        tot_tunnels += tuns;
        pre_snaps_ms2.push(s2);
    }
    println!("  TOTAL        m={:<6} -> Q_emit={:<6} (saved {:4.1}%) | trace_Q={:<6} | total_Q={:<6} ({:+.1}%) | tunnels={}",
        tot_m, tot_q_after,
        100.0 * (tot_m - tot_q_after) as f64 / tot_m as f64,
        tot_trace, tot_q_after + tot_trace,
        100.0 * ((tot_q_after + tot_trace) as f64 - tot_m as f64) / tot_m as f64,
        tot_tunnels);

    // Time:
    // (1) Plain emit on base_snaps (baseline emit!)
    // (2) ORACLE CONSUMER-ONLY: Plain emit on pre_snaps_ms2 (FREE oracle shortcuts -> unmodified emit_plain!)
    // (3) StaticTunnel(mlp=1): scan + trace + emit_plain
    // (4) StaticTunnel(mlp=8): scan + interleaved-8 trace + emit_plain
    let mut t_plain = Vec::new();
    let mut t_oracle_emit = Vec::new();
    let mut t_st1_scan = Vec::new();
    let mut t_st1_trace = Vec::new();
    let mut t_st1_emit = Vec::new();
    let mut t_st8_scan = Vec::new();
    let mut t_st8_trace = Vec::new();
    let mut t_st8_emit = Vec::new();

    let mut out = Vec::with_capacity(900_016);
    for r in 0..17 {
        let mut s_plain = 0.0;
        let mut s_ora = 0.0;
        let mut s1_sc = 0.0;
        let mut s1_tr = 0.0;
        let mut s1_em = 0.0;
        let mut s8_sc = 0.0;
        let mut s8_tr = 0.0;
        let mut s8_em = 0.0;

        for bi in 0..base_snaps.len() {
            let t0 = Instant::now();
            emit_plain(&base_snaps[bi], &mut out);
            s_plain += t0.elapsed().as_secs_f64();

            let t0 = Instant::now();
            emit_plain(&pre_snaps_ms2[bi], &mut out);
            s_ora += t0.elapsed().as_secs_f64();

            // Static mlp=1
            let mut c1 = OpenSnap {
                n: base_snaps[bi].n, m: base_snaps[bi].m, mark: base_snaps[bi].mark, bits: base_snaps[bi].bits,
                edges: base_snaps[bi].edges.clone(), spelling: base_snaps[bi].spelling.clone(),
                offsets: base_snaps[bi].offsets.clone(), inline16: base_snaps[bi].inline16.clone(),
                lens: base_snaps[bi].lens.clone(),
            };
            let (sc, tr, _, _) = static_contract(&mut c1, 2, 1);
            let t0 = Instant::now();
            emit_plain(&c1, &mut out);
            let em = t0.elapsed().as_secs_f64();
            s1_sc += sc; s1_tr += tr; s1_em += em;

            // Static mlp=8
            let mut c8 = OpenSnap {
                n: base_snaps[bi].n, m: base_snaps[bi].m, mark: base_snaps[bi].mark, bits: base_snaps[bi].bits,
                edges: base_snaps[bi].edges.clone(), spelling: base_snaps[bi].spelling.clone(),
                offsets: base_snaps[bi].offsets.clone(), inline16: base_snaps[bi].inline16.clone(),
                lens: base_snaps[bi].lens.clone(),
            };
            let (sc, tr, _, _) = static_contract(&mut c8, 2, 8);
            let t0 = Instant::now();
            emit_plain(&c8, &mut out);
            let em = t0.elapsed().as_secs_f64();
            s8_sc += sc; s8_tr += tr; s8_em += em;
        }
        if r >= 2 {
            t_plain.push(s_plain * 1000.0);
            t_oracle_emit.push(s_ora * 1000.0);
            t_st1_scan.push(s1_sc * 1000.0);
            t_st1_trace.push(s1_tr * 1000.0);
            t_st1_emit.push(s1_em * 1000.0);
            t_st8_scan.push(s8_sc * 1000.0);
            t_st8_trace.push(s8_tr * 1000.0);
            t_st8_emit.push(s8_em * 1000.0);
        }
    }

    let mp = median(t_plain);
    let mo = median(t_oracle_emit);
    let m1s = median(t_st1_scan);
    let m1t = median(t_st1_trace);
    let m1e = median(t_st1_emit);
    let m8s = median(t_st8_scan);
    let m8t = median(t_st8_trace);
    let m8e = median(t_st8_emit);

    println!("\n=== TIMING SUMMARY (6 BLOCKS SUMMED, 1T MEDIAN ms) ===");
    println!("  Baseline emit_plain (Open L=16):          {:6.3} ms (1.000x)", mp);
    println!("  ORACLE pre-tunneled emit_plain (FREE!):   {:6.3} ms ({:.3}x, max possible emit saving = {:.3} ms)", mo, mo / mp, mp - mo);
    println!("  StaticTunnel (mlp=1): scan={:.3} + trace={:.3} + emit={:.3} = {:6.3} ms ({:.3}x vs plain emit!)",
        m1s, m1t, m1e, m1s + m1t + m1e, (m1s + m1t + m1e) / mp);
    println!("  StaticTunnel (mlp=8): scan={:.3} + trace={:.3} + emit={:.3} = {:6.3} ms ({:.3}x vs plain emit!)",
        m8s, m8t, m8e, m8s + m8t + m8e, (m8s + m8t + m8e) / mp);
}
