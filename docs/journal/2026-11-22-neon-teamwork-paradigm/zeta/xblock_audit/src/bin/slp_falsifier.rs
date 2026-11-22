use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;

// Measure exact paid-cost of cross-block SLP hash-consing vs Open's plain Spelling on all 233 large blocks!
// Separates strictly cross-block hits vs within-block hits, and times an actual safe-Rust open-addressing
// hash-consing table vs Open's plain memoized spelling!

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0u32;
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        visit(c, pos, width);
        pos += width;
    }
}

fn split(
    c: u8,
    pos: u32,
    width: u32,
    p: u32,
    limit: u32,
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
    let end = pos + width;
    if width > limit && pos <= p && p < end {
        if pos < p {
            visit(c, pos, p - pos, false);
        }
        visit(c, p, 1, true);
        if p + 1 < end {
            visit(c, p + 1, end - p - 1, false);
        }
    } else {
        visit(c, pos, width, width <= limit);
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut tot = 0u32;
    counts.map(|w| {
        let s = tot;
        tot += w;
        s
    })
}

struct ElimTrace {
    rules: Vec<(u32, u32, u32)>, // (l, r, len)
    band_roots: Vec<(u32, u8)>,  // (nwd, exit_byte)
    spelling_bytes: usize,
    plain_spell_ns: u64,
}

fn extract_elim(records: &[u16], n: usize, p: u32) -> ElimTrace {
    let n32 = n as u32;
    let mut all = [0u32; 256];
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut strips: Vec<Strip> = Vec::with_capacity(4096);
    maximal(records, |c, pos, width| {
        split(c, pos, width, p, LIMIT, |c, x, w, keep| {
            let a = usize::from(c);
            if keep {
                thin[a] += w;
            } else {
                strips.push(Strip {
                    from: x,
                    to: all[a],
                    width: w,
                    byte: c,
                });
                thick[a] += 1;
            }
            all[a] += w;
        });
    });

    let first = starts(all);
    let mut write = starts(thick);
    let ns = strips.len();
    let mut image_order = vec![0u32; ns];
    for (i, s) in strips.iter_mut().enumerate() {
        let a = usize::from(s.byte);
        s.to += first[a];
        image_order[write[a] as usize] = i as u32;
        write[a] += 1;
    }

    let most = 3 * ns + 3;
    let mut prev0 = Vec::with_capacity(most);
    let mut next0 = Vec::with_capacity(most);
    let mut prev1 = Vec::with_capacity(most);
    let mut next1 = Vec::with_capacity(most);
    let mut n_width = Vec::with_capacity(most);
    let mut n_word = Vec::with_capacity(most);
    prev0.push(0);
    next0.push(0);
    prev1.push(0);
    next1.push(0);
    n_width.push(0);
    n_word.push(0);

    macro_rules! add_node {
        ($w:expr, $wd:expr) => {{
            let id = n_width.len() as u32;
            prev0.push(0);
            next0.push(0);
            prev1.push(0);
            next1.push(0);
            n_width.push($w);
            n_word.push($wd);
            id
        }};
    }
    for s in &strips {
        add_node!(s.width, u32::from(s.byte) + 1);
    }
    let mut term0 = (0u32, 0u32);
    let mut ord0 = (0u32, 0u32);
    let mut term1 = (0u32, 0u32);
    let mut ord1 = (0u32, 0u32);
    let append_to = |lst: &mut (u32, u32), id: u32, prev: &mut [u32], next: &mut [u32]| {
        if lst.1 == 0 {
            lst.0 = id;
        } else {
            next[lst.1 as usize] = id;
        }
        prev[id as usize] = lst.1;
        lst.1 = id;
    };
    let mut x = 0u32;
    for (i, s) in strips.iter().enumerate() {
        if x < s.from {
            let id = add_node!(s.from - x, 0);
            append_to(&mut ord0, id, &mut prev0, &mut next0);
            append_to(&mut term1, id, &mut prev1, &mut next1);
        }
        append_to(&mut ord0, i as u32 + 1, &mut prev0, &mut next0);
        x = s.from + s.width;
    }
    if x < n32 {
        let id = add_node!(n32 - x, 0);
        append_to(&mut ord0, id, &mut prev0, &mut next0);
        append_to(&mut term1, id, &mut prev1, &mut next1);
    }
    x = 0;
    for &idx in &image_order {
        let s = strips[idx as usize];
        if x < s.to {
            let id = add_node!(s.to - x, 0);
            append_to(&mut ord1, id, &mut prev1, &mut next1);
            append_to(&mut term0, id, &mut prev0, &mut next0);
        }
        append_to(&mut ord1, idx + 1, &mut prev1, &mut next1);
        x = s.to + s.width;
    }
    if x < n32 {
        let id = add_node!(n32 - x, 0);
        append_to(&mut ord1, id, &mut prev1, &mut next1);
        append_to(&mut term0, id, &mut prev0, &mut next0);
    }
    let close = |first_l: (u32, u32), second_l: (u32, u32), prev: &mut [u32], next: &mut [u32]| {
        let mut p_id = 0u32;
        for (h, t) in [first_l, second_l] {
            if t == 0 {
                continue;
            }
            next[p_id as usize] = h;
            prev[h as usize] = p_id;
            p_id = t;
        }
        next[p_id as usize] = 0;
        prev[0] = p_id;
    };
    close(term0, ord0, &mut prev0, &mut next0);
    close(term1, ord1, &mut prev1, &mut next1);

    let mut rules: Vec<(u32, u32, u32)> = Vec::with_capacity(most);
    let mut remaining = n32;
    while remaining > 0 {
        let xn = prev0[0];
        let yn = prev1[0];
        let aw = n_width[xn as usize];
        let bw = n_width[yn as usize];
        let w = aw.min(bw);
        remaining -= w;
        if xn == yn {
            let pr = prev0[xn as usize];
            let nx = next0[xn as usize];
            next0[pr as usize] = nx;
            prev0[nx as usize] = pr;
            let pr = prev1[xn as usize];
            let nx = next1[xn as usize];
            next1[pr as usize] = nx;
            prev1[nx as usize] = pr;
            continue;
        }
        let l = n_word[xn as usize];
        let r = n_word[yn as usize];
        let wd = if l == 0 {
            r
        } else if r == 0 {
            l
        } else {
            let id = rules.len() as u32 + 257;
            let ll = if l <= 256 { 1 } else { rules[(l - 257) as usize].2 };
            let rl = if r <= 256 { 1 } else { rules[(r - 257) as usize].2 };
            rules.push((l, r, ll + rl));
            id
        };
        if aw <= bw {
            let pr = prev0[xn as usize];
            let nx = next0[xn as usize];
            next0[pr as usize] = nx;
            prev0[nx as usize] = pr;
            let nx = next0[yn as usize];
            prev0[xn as usize] = yn;
            next0[xn as usize] = nx;
            next0[yn as usize] = xn;
            prev0[nx as usize] = xn;
            n_word[xn as usize] = wd;
            if aw == bw {
                let pr = prev0[yn as usize];
                let nx = next0[yn as usize];
                next0[pr as usize] = nx;
                prev0[nx as usize] = pr;
                let pr = prev1[yn as usize];
                let nx = next1[yn as usize];
                next1[pr as usize] = nx;
                prev1[nx as usize] = pr;
            } else {
                n_width[yn as usize] = bw - w;
            }
        } else {
            let pr = prev1[yn as usize];
            let nx = next1[yn as usize];
            next1[pr as usize] = nx;
            prev1[nx as usize] = pr;
            let nx = next1[xn as usize];
            prev1[yn as usize] = xn;
            next1[yn as usize] = nx;
            next1[xn as usize] = yn;
            prev1[nx as usize] = yn;
            n_word[yn as usize] = wd;
            n_width[xn as usize] = aw - w;
        }
    }

    // Collect band roots
    let mut band_roots = Vec::new();
    let mut nid = next0[0];
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
        let end = k + thin[a];
        while k < end {
            let nw = n_width[nid as usize];
            let nwd = n_word[nid as usize];
            let w = (end - k).min(nw - in_node);
            if nwd != 0 {
                band_roots.push((nwd, a as u8));
            }
            in_node += w;
            k += w;
            if in_node == nw {
                nid = next0[nid as usize];
                in_node = 0;
            }
        }
    }

    // Time plain memoized spelling + inline16 (Open's exact cost)
    let mut rule_first = vec![u32::MAX; rules.len()];
    let mut spelling = Vec::with_capacity(65536);
    let mut offsets = Vec::with_capacity(band_roots.len() + 1);
    let mut stack = Vec::with_capacity(256);
    let mut inline16 = Vec::with_capacity(256 + band_roots.len());
    let mut lens = Vec::with_capacity(256 + band_roots.len());
    let t0 = Instant::now();
    offsets.push(0u32);
    for &(nwd, a) in &band_roots {
        stack.clear();
        stack.push(nwd);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                spelling.push((id - 1) as u8);
            } else {
                let ri = (id - 257) as usize;
                if rule_first[ri] == u32::MAX {
                    rule_first[ri] = spelling.len() as u32;
                    stack.push(rules[ri].1);
                    stack.push(rules[ri].0);
                } else {
                    let st = rule_first[ri] as usize;
                    let ln = rules[ri].2 as usize;
                    spelling.extend_from_within(st..st + ln);
                }
            }
        }
        spelling.push(a);
        offsets.push(spelling.len() as u32);
    }
    for w in offsets.windows(2) {
        let word = &spelling[w[0] as usize..w[1] as usize];
        let mut v = [0u8; 16];
        let head = word.len().min(16);
        v[..head].copy_from_slice(&word[..head]);
        inline16.push(v);
        lens.push(if word.len() <= 16 { word.len() as u8 } else { 255 });
    }
    let plain_spell_ns = t0.elapsed().as_nanos() as u64;
    std::hint::black_box((&inline16, &lens));

    ElimTrace {
        rules,
        band_roots,
        spelling_bytes: spelling.len(),
        plain_spell_ns,
    }
}

fn main() {
    let data = fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").unwrap();
    let kernels = Kernels::detect();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut runs = Runs::default();
    let mut windows: Vec<Vec<ElimTrace>> = Vec::new();

    loop {
        match scanner.next(syms).unwrap() {
            Event::Done => break,
            Event::Block {
                stream, symbols, ..
            } => {
                let s = stream as usize;
                while windows.len() <= s {
                    windows.push(Vec::new());
                }
                if runs::takes_runs(&symbols) {
                    let n = runs::front(&symbols, &mut runs, kernels).unwrap();
                    if n >= 65_536 {
                        windows[s].push(extract_elim(runs.records(), n, symbols.orig_ptr));
                    }
                }
                syms = symbols.syms;
            }
        }
    }

    // 1. Separate strictly cross-block tree hits (from prev blocks only!) vs within-block tree hits!
    let mut tot_joins = 0usize;
    let mut cross_hits = 0usize;
    let mut within_hits = 0usize;
    let mut misses = 0usize;
    let mut tot_plain_ns = 0u64;
    let mut tot_hc_ns = 0u64;
    let mut tot_spell_b = 0usize;

    // Fast custom open-addressing hash-consing table (65536 slots, power-of-2, linear probing, FNV/splitmix)
    const CAP: usize = 1 << 16;
    const MASK: usize = CAP - 1;

    for win in &windows {
        let mut keys = vec![0u64; CAP];
        let mut vals = vec![0u32; CAP];
        let mut origin_blk = vec![u32::MAX; CAP];
        let mut next_cid = 257u32;

        // Also benchmark realistic fast open-addressing lookup time
        for (bi, tr) in win.iter().enumerate() {
            tot_plain_ns += tr.plain_spell_ns;
            tot_spell_b += tr.spelling_bytes;

            let mut local_to_canon = vec![0u32; tr.rules.len()];
            let t0 = Instant::now();
            for (ri, &(l, r, _)) in tr.rules.iter().enumerate() {
                let cl = if l <= 256 { l } else { local_to_canon[(l - 257) as usize] };
                let cr = if r <= 256 { r } else { local_to_canon[(r - 257) as usize] };
                let key = ((cl as u64) << 32) | (cr as u64);
                let mut h = (key.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 48) as usize & MASK;
                loop {
                    let k = keys[h];
                    if k == key {
                        local_to_canon[ri] = vals[h];
                        if bi > 0 {
                            tot_joins += 1;
                            if origin_blk[h] < bi as u32 {
                                cross_hits += 1;
                            } else {
                                within_hits += 1;
                            }
                        }
                        break;
                    }
                    if k == 0 {
                        let cid = next_cid;
                        next_cid += 1;
                        keys[h] = key;
                        vals[h] = cid;
                        origin_blk[h] = bi as u32;
                        local_to_canon[ri] = cid;
                        if bi > 0 {
                            tot_joins += 1;
                            misses += 1;
                        }
                        break;
                    }
                    h = (h + 1) & MASK;
                }
            }
            tot_hc_ns += t0.elapsed().as_nanos() as u64;
            std::hint::black_box(&local_to_canon);
        }
    }

    println!("=== CROSS-BLOCK vs WITHIN-BLOCK STRUCTURAL SLP HASH-CONSING (221 consecutive blocks) ===");
    println!("  Total joins: {} | cross-block hits: {} ({:.2}%) | within-block-only hits: {} ({:.2}%) | misses: {} ({:.2}%)",
        tot_joins,
        cross_hits, 100.0 * cross_hits as f64 / tot_joins as f64,
        within_hits, 100.0 * within_hits as f64 / tot_joins as f64,
        misses, 100.0 * misses as f64 / tot_joins as f64
    );
    println!("  Open's ENTIRE plain Spelling + inline16 time (233 blocks): {:.2} ms ({:.1} KiB/blk)",
        tot_plain_ns as f64 / 1e6, tot_spell_b as f64 / 233.0 / 1024.0);
    println!("  Just the 64K-slot open-addressing hash-consing probes ALONE (even BEFORE spelling misses or building inline16!): {:.2} ms",
        tot_hc_ns as f64 / 1e6);
    println!("  Ratio (hash-consing overhead alone / entire plain Spelling): {:.2}x",
        tot_hc_ns as f64 / tot_plain_ns as f64);
}
