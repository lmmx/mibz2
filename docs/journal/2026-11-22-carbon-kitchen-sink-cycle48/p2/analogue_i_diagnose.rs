#![forbid(unsafe_code)]

//! Cycle 48 P2 — Analogue I: Run-Stream / Zero-Spill Streaming Queue Monoid (`FQ4` & `Obligation 5`)
//! Exact failure, OutQ16 memory-traffic, thin-run compression, Shannon queue-rate floor,
//! and CAP=127 split-record diagnosis across:
//!   - Solved Analogue I synthetic BWTs (sigma <= 16, uncapped runs, L1D-resident queue)
//!   - 6 Admitted Wikidata Blocks (/home/antigravity/admitted-bwt/block-{0,2,85,143,207,210}.L)
//!   - All 233 Full wikidata.bz2 Blocks (/tmp/mibz2_corpora/wikidata_431_blocks.bin)
//!   - 6 Real Non-Wikidata Blocks (/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin)

use std::fs;

const LIMIT: u32 = 16;

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                records,
            }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && route_class > 1 {
            continue;
        }
        let name = if let Some(ns) = names {
            if b_idx < ns.len() {
                ns[b_idx].to_string()
            } else {
                format!("blk-{idx}")
            }
        } else {
            format!("w431-{idx}")
        };
        blocks.push(Block {
            name,
            n,
            p,
            records,
        });
    }
    blocks
}

/// Build a genuine Solved Analogue I BWT block (sigma = 12 <= 16, rich thin runs w in [2, 16]
/// with m/r_thin ~ 4..6 AND thick template runs > 127, valid cyclic BWT).
/// Each record pairs a shared template prefix (producing thick strips > 127) with a 6-byte
/// key drawn from a pool of ~n/40 distinct keys over 12 letters (repeated ~5-8x each -> thin runs!).
fn build_solved_analogue_i_block(name: &str, n: usize, seed: u64) -> Block {
    let alpha = b"abcdefghijkl";
    let mut text = Vec::with_capacity(n);
    let mut rng = seed;
    let next_u64 = |s: &mut u64| -> usize {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*s >> 33) as usize
    };
    let n_keys = (n / 120).max(512);
    let mut keys: Vec<[u8; 8]> = Vec::with_capacity(n_keys);
    for _ in 0..n_keys {
        let mut k = [0u8; 8];
        for b in &mut k {
            *b = alpha[4 + (next_u64(&mut rng) % 8)];
        }
        keys.push(k);
    }
    let tmpl = b"abcdabdc";
    while text.len() < n {
        let k = keys[next_u64(&mut rng) % n_keys];
        for &ch in tmpl {
            if text.len() < n {
                text.push(ch);
            }
        }
        for &ch in &k {
            if text.len() < n {
                text.push(ch);
            }
        }
    }
    let (l, p) = bwt_cyclic(&text);
    Block {
        name: name.to_string(),
        n,
        p,
        records: records_of(&l),
    }
}

fn bwt_cyclic(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<u32> = t.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut cnt = vec![0usize; n.max(256)];
    let mut sa2 = vec![0usize; n];
    // Initial counting sort by first byte
    for &b in t {
        cnt[b as usize] += 1;
    }
    let mut sum = 0usize;
    for c in cnt.iter_mut().take(256) {
        let v = *c;
        *c = sum;
        sum += v;
    }
    for i in 0..n {
        let b = t[i] as usize;
        sa[cnt[b]] = i;
        cnt[b] += 1;
    }
    let mut classes = 1u32;
    tmp[sa[0]] = 0;
    for i in 1..n {
        if t[sa[i]] != t[sa[i - 1]] {
            classes += 1;
        }
        tmp[sa[i]] = classes - 1;
    }
    std::mem::swap(&mut rank, &mut tmp);

    let mut h = 1usize;
    while h < n && (classes as usize) < n {
        for i in 0..n {
            let v = sa[i];
            sa2[i] = if v >= h { v - h } else { v + n - h };
        }
        let c_len = classes as usize;
        cnt[..c_len].fill(0);
        for &idx in &sa2 {
            cnt[rank[idx] as usize] += 1;
        }
        let mut s = 0usize;
        for c in cnt.iter_mut().take(c_len) {
            let v = *c;
            *c = s;
            s += v;
        }
        for &idx in &sa2 {
            let r = rank[idx] as usize;
            sa[cnt[r]] = idx;
            cnt[r] += 1;
        }
        tmp[sa[0]] = 0;
        let mut new_classes = 1u32;
        for i in 1..n {
            let cur = sa[i];
            let prev = sa[i - 1];
            let cur2 = if cur + h < n { cur + h } else { cur + h - n };
            let prev2 = if prev + h < n {
                prev + h
            } else {
                prev + h - n
            };
            if rank[cur] != rank[prev] || rank[cur2] != rank[prev2] {
                new_classes += 1;
            }
            tmp[cur] = new_classes - 1;
        }
        std::mem::swap(&mut rank, &mut tmp);
        classes = new_classes;
        h <<= 1;
    }
    let mut l = vec![0u8; n];
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
            l[i] = t[n - 1];
        } else {
            l[i] = t[pos - 1];
        }
    }
    (l, p)
}

#[derive(Default, Clone)]
struct BlockDiag {
    name: String,
    n: usize,
    nrec: usize,
    r_max: usize,
    delta_127: usize,
    delta_p: usize,
    thick_gt_127: usize,
    max_run_len: u32,
    r_thin: usize,
    ns: usize,
    m: usize,
    w_hist: [usize; 17], // 1..=16
    sigma_all: usize,
    sigma_thin: usize,
    adj_same_thin: usize,
    // OutQ16 chunk counts and allocated bytes for CHUNK in {64, 128, 256, 512}
    chunks: [usize; 4],
    ovf_chunks: [usize; 4],
    cuts_64k: usize,
    // Compact thin-run queue word counts (u16 words) and cuts
    rp_e4k_words: usize,
    rp_e4k_cuts: usize,
    rp_e4k_chunks128: usize,
    rp_e16k_words: usize,
    rp_e16k_cuts: usize,
    rp_e16k_chunks128: usize,
    rp_e32k_words: usize,
    rp_e32k_cuts: usize,
    rp_e32k_chunks128: usize,
    rp_u32_bytes: usize,
    // Shannon (1948) & MacMahon (1915) / de Moivre (1730) queue-rate bounds (bytes)
    shannon_pos_bytes: f64,
    shannon_width_bytes: f64,
    shannon_indep_bytes: f64,
    macmahon_unres_comp_bytes: f64,
    macmahon_cap16_comp_bytes: f64,
    // Exact MacMahon/Markov-Dirichlet composition slack & identity checks
    s2_slack: usize,
    md_identity_ok: bool,
    exact_split_id_ok: bool,
    exact_rp_slack_ok: bool,
    md_raw_overshoot: i64,
}

fn log2_factorial(n: usize) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    // Stirling + exact for small n (accurate to <1e-9 bits)
    if n <= 64 {
        let mut s = 0.0f64;
        for k in 2..=n {
            s += (k as f64).log2();
        }
        return s;
    }
    let nf = n as f64;
    let ln_fact = nf * nf.ln() - nf + 0.5 * (2.0 * std::f64::consts::PI * nf).ln()
        + 1.0 / (12.0 * nf)
        - 1.0 / (360.0 * nf * nf * nf);
    ln_fact / std::f64::consts::LN_2
}

fn log2_binom(n: usize, k: usize) -> f64 {
    if k == 0 || k >= n {
        return 0.0;
    }
    log2_factorial(n) - log2_factorial(k) - log2_factorial(n - k)
}

/// Exact saddle-point evaluation of log2 N_L(m, r) where
/// N_L(m, r) = [x^m] (x + x^2 + ... + x^L)^r (de Moivre 1730; MacMahon 1915, Vol. I, Sec. IV, Ch. I).
fn log2_macmahon_bounded_comp(m: usize, r: usize, cap: usize) -> f64 {
    if r == 0 || m <= r || m >= r * cap {
        return 0.0;
    }
    let target = m as f64 / r as f64;
    // Solve E_z[K] = sum_{k=1..=cap} k z^k / sum_{k=1..=cap} z^k = target using Newton on t = ln z.
    let mut t = 0.0f64;
    for _ in 0..40 {
        let mut s0 = 0.0f64;
        let mut s1 = 0.0f64;
        let mut s2 = 0.0f64;
        for k in 1..=cap {
            let kf = k as f64;
            let w = (kf * t).exp();
            s0 += w;
            s1 += kf * w;
            s2 += kf * kf * w;
        }
        let mu = s1 / s0;
        let var = (s2 / s0 - mu * mu).max(1e-12);
        let diff = mu - target;
        if diff.abs() < 1e-11 {
            let ln_p = s0.ln();
            let ln_n = (r as f64) * (ln_p - target * t)
                - 0.5 * (2.0 * std::f64::consts::PI * (r as f64) * var).ln();
            return (ln_n / std::f64::consts::LN_2).max(0.0);
        }
        t -= (diff / var).clamp(-1.5, 1.5);
    }
    let mut s0 = 0.0f64;
    let mut s1 = 0.0f64;
    let mut s2 = 0.0f64;
    for k in 1..=cap {
        let kf = k as f64;
        let w = (kf * t).exp();
        s0 += w;
        s1 += kf * w;
        s2 += kf * kf * w;
    }
    let mu = s1 / s0;
    let var = (s2 / s0 - mu * mu).max(1e-12);
    let ln_n = (r as f64) * (s0.ln() - target * t)
        - 0.5 * (2.0 * std::f64::consts::PI * (r as f64) * var).ln();
    (ln_n / std::f64::consts::LN_2).max(0.0)
}

fn diagnose_block(blk: &Block) -> BlockDiag {
    let mut d = BlockDiag {
        name: blk.name.clone(),
        n: blk.n,
        nrec: blk.records.len(),
        ..BlockDiag::default()
    };

    let mut all_cnt = [0u32; 256];
    let mut thin_rows = [0u32; 256];
    let mut thin_runs_per_byte = [0usize; 256];
    let mut rp_e4k_per_byte = [0usize; 256];
    let mut rp_e16k_per_byte = [0usize; 256];
    let mut rp_e32k_per_byte = [0usize; 256];

    let mut m = 0u32;
    let mut pos = 0u32;
    let mut i = 0usize;
    let nrec = blk.records.len();
    let mut last_thin_byte: Option<u8> = None;

    let mut visit_piece = |c: u8, w: u32, keep: bool| {
        let a = usize::from(c);
        all_cnt[a] += w;
        if keep {
            if last_thin_byte == Some(c) {
                d.adj_same_thin += 1;
            }
            last_thin_byte = Some(c);
            d.r_thin += 1;
            thin_runs_per_byte[a] += 1;
            thin_rows[a] += w;
            if (w as usize) <= 16 {
                d.w_hist[w as usize] += 1;
            }
            // Simulate exact epoch splitting for 64K, 32K, 16K, 4K
            let m0 = m;
            let m1 = m + w;
            d.cuts_64k += ((m1 / 65_536) - (m0 / 65_536)) as usize;
            // E4K: split at 4096 boundaries; each piece = 1 u16 word
            {
                let mut cur = m0;
                let mut left = w;
                while left > 0 {
                    let lo = cur & 4095;
                    let take = left.min(4096 - lo);
                    rp_e4k_per_byte[a] += 1;
                    d.rp_e4k_words += 1;
                    cur += take;
                    left -= take;
                    if cur % 4096 == 0 {
                        d.rp_e4k_cuts += 1;
                    }
                }
            }
            // E16K: split at 16384 boundaries; w<=3 -> 1 u16, w>=4 -> 2 u16
            {
                let mut cur = m0;
                let mut left = w;
                while left > 0 {
                    let lo = cur & 16_383;
                    let take = left.min(16_384 - lo);
                    let words = if take <= 3 { 1 } else { 2 };
                    rp_e16k_per_byte[a] += words;
                    d.rp_e16k_words += words;
                    cur += take;
                    left -= take;
                    if cur % 16_384 == 0 {
                        d.rp_e16k_cuts += 1;
                    }
                }
            }
            // E32K: split at 32768 boundaries; w==1 -> 1 u16, w>=2 -> 2 u16
            {
                let mut cur = m0;
                let mut left = w;
                while left > 0 {
                    let lo = cur & 32_767;
                    let take = left.min(32_768 - lo);
                    let words = if take == 1 { 1 } else { 2 };
                    rp_e32k_per_byte[a] += words;
                    d.rp_e32k_words += words;
                    cur += take;
                    left -= take;
                    if cur % 32_768 == 0 {
                        d.rp_e32k_cuts += 1;
                    }
                }
            }
            m = m1;
        } else {
            d.ns += 1;
        }
    };

    while i < nrec {
        let rec = blk.records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        if width == 127 {
            while i < nrec && (blk.records[i] >> 8) as u8 == c {
                width += u32::from(blk.records[i] & 255);
                i += 1;
            }
        }
        d.r_max += 1;
        if width > 127 {
            d.thick_gt_127 += 1;
        }
        if width > d.max_run_len {
            d.max_run_len = width;
        }
        let end = pos + width;
        if width > LIMIT && (pos..end).contains(&blk.p) {
            let mut pieces = 0usize;
            if pos < blk.p {
                visit_piece(c, blk.p - pos, false);
                pieces += 1;
            }
            visit_piece(c, 1, true);
            pieces += 1;
            if blk.p + 1 < end {
                visit_piece(c, end - blk.p - 1, false);
                pieces += 1;
            }
            d.delta_p = pieces - 1;
        } else {
            visit_piece(c, width, width <= LIMIT);
        }
        pos = end;
    }
    assert_eq!(pos as usize, blk.n);
    d.m = m as usize;
    d.delta_127 = d.nrec - d.r_max;
    d.rp_u32_bytes = 4 * d.r_thin;

    for a in 0..256 {
        if all_cnt[a] > 0 {
            d.sigma_all += 1;
        }
        if thin_rows[a] > 0 {
            d.sigma_thin += 1;
        }
    }

    let chunk_sizes = [64usize, 128, 256, 512];
    let out_bytes = blk.n + 16;
    for (ci, &cs) in chunk_sizes.iter().enumerate() {
        let mut ch = 0usize;
        for &tr in &thin_rows {
            if tr > 0 {
                ch += (tr as usize).div_ceil(cs);
            }
        }
        let in_out = out_bytes / (2 * cs);
        d.chunks[ci] = ch;
        d.ovf_chunks[ci] = ch.saturating_sub(in_out);
    }
    for a in 0..256 {
        if rp_e4k_per_byte[a] > 0 {
            d.rp_e4k_chunks128 += rp_e4k_per_byte[a].div_ceil(128);
        }
        if rp_e16k_per_byte[a] > 0 {
            d.rp_e16k_chunks128 += rp_e16k_per_byte[a].div_ceil(128);
        }
        if rp_e32k_per_byte[a] > 0 {
            d.rp_e32k_chunks128 += rp_e32k_per_byte[a].div_ceil(128);
        }
    }

    // Shannon (1948) + MacMahon (1915, Vol. I, Sec. III & IV) / de Moivre (1730) queue-rate bounds:
    let mut h_byte_bits = log2_factorial(d.r_thin);
    let mut h_indep_pos_bits = 0.0f64;
    for &c in &thin_runs_per_byte {
        h_byte_bits -= log2_factorial(c);
        if c > 0 {
            h_indep_pos_bits += log2_binom(d.m, c);
        }
    }
    let mut h_width_bits = 0.0f64;
    if d.r_thin > 0 {
        let rt = d.r_thin as f64;
        for &cnt in &d.w_hist[1..=16] {
            if cnt > 0 {
                let p = cnt as f64 / rt;
                h_width_bits -= (cnt as f64) * p.log2();
            }
        }
    }
    d.shannon_pos_bytes = h_byte_bits / 8.0;
    d.shannon_width_bytes = h_width_bits / 8.0;
    d.shannon_indep_bytes = (h_indep_pos_bits + h_width_bits) / 8.0;
    d.macmahon_unres_comp_bytes = if d.r_thin > 0 && d.m >= d.r_thin {
        (h_byte_bits + log2_binom(d.m - 1, d.r_thin - 1)) / 8.0
    } else {
        0.0
    };
    d.macmahon_cap16_comp_bytes =
        (h_byte_bits + log2_macmahon_bounded_comp(d.m, d.r_thin, 16)) / 8.0;

    // Exact MacMahon (1915) Composition-Slack & Refined Cycle 47/48 Split-Record Identity:
    //   S_2 := sum_{k=3..=16} (k - 2) * c_k == w_1 - (2 * r_thin - m) >= 0
    //   2*(nrec - ns) - m == (w_1 - S_2) + 2*Delta_127 - 2*delta_p
    //   W_E32K_nosplit = 2*r_thin - w_1  =>  m - W_E32K_nosplit == S_2
    //   W_E16K_nosplit = 2*r_thin - (c_1 + c_2 + c_3) => m - W_E16K_nosplit == S_2 + c_2 + c_3
    let s2: usize = (3..=16).map(|k| (k - 2) * d.w_hist[k]).sum();
    d.s2_slack = s2;
    let md_true = 2 * (d.r_thin as i64) - (d.m as i64);
    let md_raw = 2 * ((d.nrec as i64) - (d.ns as i64)) - (d.m as i64);
    let w1 = d.w_hist[1] as i64;
    d.md_identity_ok = md_true <= w1 && (w1 - md_true) == s2 as i64;
    d.exact_split_id_ok = (d.r_thin + d.ns == d.r_max + d.delta_p)
        && (md_raw == (w1 - s2 as i64) + 2 * (d.delta_127 as i64) - 2 * (d.delta_p as i64));
    let w_e32k_nosplit = 2 * d.r_thin - d.w_hist[1];
    let w_e16k_nosplit = 2 * d.r_thin - (d.w_hist[1] + d.w_hist[2] + d.w_hist[3]);
    d.exact_rp_slack_ok = (d.m - w_e32k_nosplit == s2)
        && (d.m - w_e16k_nosplit == s2 + d.w_hist[2] + d.w_hist[3])
        && (d.rp_e32k_words >= w_e32k_nosplit)
        && (d.rp_e16k_words >= w_e16k_nosplit)
        && (d.rp_e4k_words >= d.r_thin);
    d.md_raw_overshoot = md_raw - w1;

    d
}

fn print_corpus_report(title: &str, blocks: &[Block], per_block: bool) {
    println!("\n====================================================================================================");
    println!("ANALOGUE I MEMORY-TRAFFIC & CAP=127 DIAGNOSIS — {title} ({} blocks)", blocks.len());
    println!("====================================================================================================");

    let diags: Vec<BlockDiag> = blocks.iter().map(diagnose_block).collect();

    if per_block {
        println!("\n[1] OutQ16 Working Set, Chunk Allocation & Internal Fragmentation (CHUNK = 512 vs 256 vs 128 vs 64):");
        println!(
            "{:<13} {:>7} {:>7} {:>4} {:>4} | {:>7} {:>7} {:>7} | {:>7} {:>7} {:>7} {:>7} | {:>6}",
            "Block", "n", "m", "sigA", "sigT", "Use2m", "Al512", "Fr512", "Fr256", "Fr128", "Fr64", "Act512", "S14B/B"
        );
        for d in &diags {
            let use_k = (2 * d.m) as f64 / 1024.0;
            let al512 = (d.chunks[3] * 1024) as f64 / 1024.0;
            let fr512 = (d.chunks[3] * 1024 - 2 * d.m) as f64 / 1024.0;
            let fr256 = (d.chunks[2] * 512 - 2 * d.m) as f64 / 1024.0;
            let fr128 = (d.chunks[1] * 256 - 2 * d.m) as f64 / 1024.0;
            let fr64 = (d.chunks[0] * 128 - 2 * d.m) as f64 / 1024.0;
            let act512 = d.sigma_thin as f64; // KiB since 1024 B/byte
            let s14_bb = (4 * d.m) as f64 / d.n as f64; // S1 write 2m + S4 read 2m
            println!(
                "{:<13} {:>7} {:>7} {:>4} {:>4} | {:>6.1}K {:>6.1}K {:>6.1}K | {:>6.1}K {:>6.1}K {:>6.1}K {:>6.1}K | {:>6.3}",
                d.name, d.n, d.m, d.sigma_all, d.sigma_thin, use_k, al512, fr512, fr256, fr128, fr64, act512, s14_bb
            );
        }

        println!("\n[2] Thin-Run Width Profile & Compact Queue Compression vs OutQ16 (2*m) & Shannon (1948) Bound:");
        println!(
            "{:<13} {:>7} {:>7} {:>5} {:>5} {:>5} | {:>7} {:>7} {:>7} {:>7} {:>7} | {:>7}",
            "Block", "r_thin", "m/r_t", "w=1%", "w2-3%", "w4+%", "OutQ16", "E4K+ct", "E16K+c", "E32K+c", "U32", "Shan48"
        );
        for d in &diags {
            let w1_pct = 100.0 * d.w_hist[1] as f64 / d.r_thin.max(1) as f64;
            let w23_pct = 100.0 * (d.w_hist[2] + d.w_hist[3]) as f64 / d.r_thin.max(1) as f64;
            let w4p: usize = d.w_hist[4..=16].iter().sum();
            let w4p_pct = 100.0 * w4p as f64 / d.r_thin.max(1) as f64;
            let q16 = (2 * d.m) as f64 / 1024.0;
            let e4k = (2 * d.rp_e4k_words + 1024 * d.rp_e4k_cuts) as f64 / 1024.0;
            let e16k = (2 * d.rp_e16k_words + 1024 * d.rp_e16k_cuts) as f64 / 1024.0;
            let e32k = (2 * d.rp_e32k_words + 1024 * d.rp_e32k_cuts) as f64 / 1024.0;
            let u32 = d.rp_u32_bytes as f64 / 1024.0;
            let shan = (d.shannon_pos_bytes + d.shannon_width_bytes) / 1024.0;
            println!(
                "{:<13} {:>7} {:>5.2} {:>4.1}% {:>4.1}% {:>4.1}% | {:>6.1}K {:>6.1}K {:>6.1}K {:>6.1}K {:>6.1}K | {:>6.1}K",
                d.name,
                d.r_thin,
                d.m as f64 / d.r_thin.max(1) as f64,
                w1_pct,
                w23_pct,
                w4p_pct,
                q16,
                e4k,
                e16k,
                e32k,
                u32,
                shan
            );
        }

        println!("\n[3] CAP=127 Split-Record Overhead (Delta_127) & Input Stream Sizes:");
        println!(
            "{:<13} {:>7} {:>7} {:>6} {:>6} {:>6} {:>7} | {:>7} {:>7} {:>7}",
            "Block", "r_max", "nrec", "D_127", "D127%", "gt127", "maxLen", "Cap127", "UncU32", "Esc16"
        );
        for d in &diags {
            let cap_k = (2 * d.nrec) as f64 / 1024.0;
            let u32_k = (4 * d.r_max) as f64 / 1024.0;
            let esc_k = (2 * (d.r_max + d.thick_gt_127)) as f64 / 1024.0;
            println!(
                "{:<13} {:>7} {:>7} {:>6} {:>5.1}% {:>6} {:>7} | {:>6.1}K {:>6.1}K {:>6.1}K",
                d.name,
                d.r_max,
                d.nrec,
                d.delta_127,
                100.0 * d.delta_127 as f64 / d.r_max.max(1) as f64,
                d.thick_gt_127,
                d.max_run_len,
                cap_k,
                u32_k,
                esc_k
            );
        }
    }

    // Aggregate summary across all blocks in corpus
    let nb = diags.len() as f64;
    let tot_n: usize = diags.iter().map(|d| d.n).sum();
    let tot_m: usize = diags.iter().map(|d| d.m).sum();
    let tot_r_thin: usize = diags.iter().map(|d| d.r_thin).sum();
    let tot_ns: usize = diags.iter().map(|d| d.ns).sum();
    let tot_r_max: usize = diags.iter().map(|d| d.r_max).sum();
    let tot_nrec: usize = diags.iter().map(|d| d.nrec).sum();
    let tot_d127: usize = diags.iter().map(|d| d.delta_127).sum();
    let tot_gt127: usize = diags.iter().map(|d| d.thick_gt_127).sum();
    let mut w_hist = [0usize; 17];
    for d in &diags {
        for w in 1..=16 {
            w_hist[w] += d.w_hist[w];
        }
    }
    let avg_sig_all = diags.iter().map(|d| d.sigma_all).sum::<usize>() as f64 / nb;
    let avg_sig_thin = diags.iter().map(|d| d.sigma_thin).sum::<usize>() as f64 / nb;
    let max_sig_thin = diags.iter().map(|d| d.sigma_thin).max().unwrap_or(0);
    let tot_chunks = [0usize, 1, 2, 3].map(|ci| diags.iter().map(|d| d.chunks[ci]).sum::<usize>());
    let tot_ovf = [0usize, 1, 2, 3].map(|ci| diags.iter().map(|d| d.ovf_chunks[ci]).sum::<usize>());
    let tot_cuts_64k: usize = diags.iter().map(|d| d.cuts_64k).sum();
    let tot_e4k_w: usize = diags.iter().map(|d| d.rp_e4k_words).sum();
    let tot_e4k_c: usize = diags.iter().map(|d| d.rp_e4k_cuts).sum();
    let tot_e16k_w: usize = diags.iter().map(|d| d.rp_e16k_words).sum();
    let tot_e16k_c: usize = diags.iter().map(|d| d.rp_e16k_cuts).sum();
    let tot_e32k_w: usize = diags.iter().map(|d| d.rp_e32k_words).sum();
    let tot_e32k_c: usize = diags.iter().map(|d| d.rp_e32k_cuts).sum();
    let tot_u32_b: usize = diags.iter().map(|d| d.rp_u32_bytes).sum();
    let tot_shan_b: f64 = diags
        .iter()
        .map(|d| d.shannon_pos_bytes + d.shannon_width_bytes)
        .sum();
    let md_ok = diags.iter().filter(|d| d.md_identity_ok).count();
    let md_ov = diags.iter().filter(|d| d.md_raw_overshoot > 0).count();

    println!("\n--- AGGREGATE SUMMARY ({title}, {} blocks, total n = {}) ---", diags.len(), tot_n);
    println!(
        "  Active bytes: mean sigma_all={:.1}, mean sigma_thin={:.1} (max={}) -> Active CHUNK frontier: 512={:.1}KiB (max {}KiB), 256={:.1}KiB, 128={:.1}KiB, 64={:.1}KiB",
        avg_sig_all,
        avg_sig_thin,
        max_sig_thin,
        avg_sig_thin * 1.0,
        max_sig_thin,
        avg_sig_thin * 0.5,
        avg_sig_thin * 0.25,
        avg_sig_thin * 0.125
    );
    let useful_2m = 2 * tot_m;
    let cs_arr = [64usize, 128, 256, 512];
    for (ci, &cs) in cs_arr.iter().enumerate() {
        let alloc_b = tot_chunks[ci] * 2 * cs;
        let frag_b = alloc_b - useful_2m;
        let next_b = 4 * tot_chunks[ci];
        println!(
            "  OutQ16 CHUNK={:<3} ({:>4}B): chunks/blk={:>6.1} | alloc/blk={:>6.1}KiB (useful={:>6.1}KiB, frag={:>5.1}KiB [{:>4.1}%]) | S1+S4_useful={:.3} B/B, S1+S4_alloc={:.3} B/B | ovf={} | next/blk={:.1}KiB",
            cs,
            2 * cs,
            tot_chunks[ci] as f64 / nb,
            alloc_b as f64 / (1024.0 * nb),
            useful_2m as f64 / (1024.0 * nb),
            frag_b as f64 / (1024.0 * nb),
            100.0 * frag_b as f64 / alloc_b.max(1) as f64,
            2.0 * useful_2m as f64 / tot_n as f64,
            2.0 * alloc_b as f64 / tot_n as f64,
            tot_ovf[ci],
            next_b as f64 / (1024.0 * nb)
        );
    }
    println!(
        "  OutQ16 64K cuts/blk = {:.2} ({:.2} KiB/blk)",
        tot_cuts_64k as f64 / nb,
        tot_cuts_64k as f64 / nb
    );

    let w1_rows = w_hist[1];
    let w23_runs = w_hist[2] + w_hist[3];
    let w23_rows = 2 * w_hist[2] + 3 * w_hist[3];
    let w4p_runs: usize = w_hist[4..=16].iter().sum();
    let w4p_rows: usize = (4..=16).map(|w| w * w_hist[w]).sum();
    println!(
        "  Thin runs vs rows: r_thin/blk={:.1}, m/blk={:.1}, ns/blk={:.1} | m/r_thin={:.3} (r_thin/m={:.3})",
        tot_r_thin as f64 / nb,
        tot_m as f64 / nb,
        tot_ns as f64 / nb,
        tot_m as f64 / tot_r_thin.max(1) as f64,
        tot_r_thin as f64 / tot_m.max(1) as f64
    );
    println!(
        "  Width split: w=1: {:.1}% runs ({:.1}% rows) | w=2..3: {:.1}% runs ({:.1}% rows) | w=4..16: {:.1}% runs ({:.1}% rows)",
        100.0 * w_hist[1] as f64 / tot_r_thin.max(1) as f64,
        100.0 * w1_rows as f64 / tot_m.max(1) as f64,
        100.0 * w23_runs as f64 / tot_r_thin.max(1) as f64,
        100.0 * w23_rows as f64 / tot_m.max(1) as f64,
        100.0 * w4p_runs as f64 / tot_r_thin.max(1) as f64,
        100.0 * w4p_rows as f64 / tot_m.max(1) as f64
    );
    let q16_b = useful_2m as f64;
    let e4k_q = (2 * tot_e4k_w) as f64;
    let e4k_all = e4k_q + (1024 * tot_e4k_c) as f64;
    let e16k_q = (2 * tot_e16k_w) as f64;
    let e16k_all = e16k_q + (1024 * tot_e16k_c) as f64;
    let e32k_q = (2 * tot_e32k_w) as f64;
    let e32k_all = e32k_q + (1024 * tot_e32k_c) as f64;
    let u32_b = tot_u32_b as f64;
    println!(
        "  Compact Queue Bytes/blk (S1 write = S4 read; S1+S4 = 2x):"
    );
    println!(
        "    OutQ16 (expanded u16): {:>6.1} KiB/blk (1.000x, S1+S4={:.3} B/B)",
        q16_b / (1024.0 * nb),
        2.0 * q16_b / tot_n as f64
    );
    println!(
        "    RunPair-E4K  (2B/run): {:>6.1} KiB/blk queue ({:.3}x) + {:>5.1} KiB cuts ({:.1}/blk) = {:>6.1} KiB ({:.3}x, S1+S4_q={:.3} B/B)",
        e4k_q / (1024.0 * nb),
        e4k_q / q16_b,
        (1024 * tot_e4k_c) as f64 / (1024.0 * nb),
        tot_e4k_c as f64 / nb,
        e4k_all / (1024.0 * nb),
        e4k_all / q16_b,
        2.0 * e4k_q / tot_n as f64
    );
    println!(
        "    RunPair-E16K (2B w<=3):{:>6.1} KiB/blk queue ({:.3}x) + {:>5.1} KiB cuts ({:.1}/blk) = {:>6.1} KiB ({:.3}x, S1+S4_q={:.3} B/B)",
        e16k_q / (1024.0 * nb),
        e16k_q / q16_b,
        (1024 * tot_e16k_c) as f64 / (1024.0 * nb),
        tot_e16k_c as f64 / nb,
        e16k_all / (1024.0 * nb),
        e16k_all / q16_b,
        2.0 * e16k_q / tot_n as f64
    );
    println!(
        "    RunPair-E32K (2B w=1): {:>6.1} KiB/blk queue ({:.3}x) + {:>5.1} KiB cuts ({:.1}/blk) = {:>6.1} KiB ({:.3}x, S1+S4_q={:.3} B/B)",
        e32k_q / (1024.0 * nb),
        e32k_q / q16_b,
        (1024 * tot_e32k_c) as f64 / (1024.0 * nb),
        tot_e32k_c as f64 / nb,
        e32k_all / (1024.0 * nb),
        e32k_all / q16_b,
        2.0 * e32k_q / tot_n as f64
    );
    println!(
        "    RunPair-U32  (4B/run): {:>6.1} KiB/blk queue ({:.3}x, 0 cuts, S1+S4_q={:.3} B/B)",
        u32_b / (1024.0 * nb),
        u32_b / q16_b,
        2.0 * u32_b / tot_n as f64
    );
    let tot_shan_indep: f64 = diags.iter().map(|d| d.shannon_indep_bytes).sum();
    let tot_mm_unres: f64 = diags.iter().map(|d| d.macmahon_unres_comp_bytes).sum();
    let tot_mm_cap16: f64 = diags.iter().map(|d| d.macmahon_cap16_comp_bytes).sum();
    let tot_s2: usize = diags.iter().map(|d| d.s2_slack).sum();
    let tot_adj_same: usize = diags.iter().map(|d| d.adj_same_thin).sum();
    let delta_p_nz = diags.iter().filter(|d| d.delta_p > 0).count();
    let split_ok = diags.iter().filter(|d| d.exact_split_id_ok).count();
    let rp_ok = diags.iter().filter(|d| d.exact_rp_slack_ok).count();
    println!(
        "    Shannon (1948) joint : {:>6.1} KiB/blk ({:.3}x of OutQ16, {:.3} B/B one-way) | indep per-byte Q: {:>6.1} KiB/blk ({:.3}x, {:.3} B/B)",
        tot_shan_b / (1024.0 * nb),
        tot_shan_b / q16_b,
        tot_shan_b / tot_n as f64,
        tot_shan_indep / (1024.0 * nb),
        tot_shan_indep / q16_b,
        tot_shan_indep / tot_n as f64
    );
    println!(
        "    MacMahon (1915) comp : cap16 N_16(m,r) = {:>6.1} KiB/blk ({:.3}x) | unres C(m-1,r-1) = {:>6.1} KiB/blk ({:.3}x) | adj-same thin runs = {:.2}%",
        tot_mm_cap16 / (1024.0 * nb),
        tot_mm_cap16 / q16_b,
        tot_mm_unres / (1024.0 * nb),
        tot_mm_unres / q16_b,
        100.0 * tot_adj_same as f64 / tot_r_thin.max(1) as f64
    );
    let cap_in_b = 2 * tot_nrec;
    let u32_in_b = 4 * tot_r_max;
    let esc_in_b = 2 * (tot_r_max + tot_gt127);
    println!(
        "  CAP=127 Split-Record Census: r_max/blk={:.1}, nrec/blk={:.1}, Delta_127/blk={:.1} (+{:.2}%), thick>127/blk={:.1}",
        tot_r_max as f64 / nb,
        tot_nrec as f64 / nb,
        tot_d127 as f64 / nb,
        100.0 * tot_d127 as f64 / tot_r_max.max(1) as f64,
        tot_gt127 as f64 / nb
    );
    println!(
        "    Input Stream Bytes/blk: CAP=127 (&[u16]) = {:.1} KiB (1.000x) | Uncapped-U32 (&[u32]) = {:.1} KiB ({:.3}x) | Uncapped-Esc16 (&[u16]) = {:.1} KiB ({:.3}x)",
        cap_in_b as f64 / (1024.0 * nb),
        u32_in_b as f64 / (1024.0 * nb),
        u32_in_b as f64 / cap_in_b.max(1) as f64,
        esc_in_b as f64 / (1024.0 * nb),
        esc_in_b as f64 / cap_in_b.max(1) as f64
    );
    println!(
        "    MacMahon/MD Exact Identities: `w1-(2r_thin-m)==S2` {}/{} PASS | `2(nrec-ns)-m==(w1-S2)+2D_127-2d_p` {}/{} PASS (d_p>0 on {} blks) | RunPair Slack Theorem {}/{} PASS",
        md_ok,
        diags.len(),
        split_ok,
        diags.len(),
        delta_p_nz,
        rp_ok,
        diags.len()
    );
    println!(
        "    MacMahon Slack S2/blk = {:.1} rows ({:.1}% of m = exact E32K savings ceiling) | `md_raw > w1` overshoot blocks: {}",
        tot_s2 as f64 / nb,
        100.0 * tot_s2 as f64 / tot_m.max(1) as f64,
        md_ov
    );
}

fn resolve_corpus_path(rel: &str) -> Option<String> {
    let candidates = [
        format!("/tmp/mibz2_corpora/{rel}"),
        format!("/home/antigravity/mibz2_corpora_cache/{rel}"),
    ];
    for p in candidates {
        if std::path::Path::new(&p).exists() {
            return Some(p);
        }
    }
    None
}

fn main() {
    let solved = vec![
        build_solved_analogue_i_block("AnaI-300k", 300_000, 0xA11CE_0001),
        build_solved_analogue_i_block("AnaI-900k", 900_000, 0xA11CE_0002),
    ];
    let wiki6 = load_six_admitted();
    let nonwiki_names = [
        "jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0",
    ];
    let nonwiki_path = resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin")
        .expect("non_wikidata_blocks.bin not found");
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false);

    let w431_path = resolve_corpus_path("wikidata_431_blocks.bin")
        .expect("wikidata_431_blocks.bin not found");
    let w233 = load_bin_blocks(&w431_path, None, true);

    print_corpus_report("Solved Analogue I (sigma=12 Synthetic BWTs)", &solved, true);
    print_corpus_report("6 Admitted Wikidata Blocks", &wiki6, true);
    print_corpus_report("All 233 Full wikidata.bz2 Blocks", &w233, false);
    print_corpus_report("6 Real Non-Wikidata Blocks (JSONL + Syslog)", &nonwiki6, true);

    if let Some(code_path) = resolve_corpus_path("non_wikidata/code_docs_blocks.bin") {
        let code_names = ["code-0", "code-1", "code-2", "code-3"];
        let code4 = load_bin_blocks(&code_path, Some(&code_names), false);
        print_corpus_report("4 Real Code & Docs Blocks (code_docs_blocks.bin)", &code4, true);
    }
}
