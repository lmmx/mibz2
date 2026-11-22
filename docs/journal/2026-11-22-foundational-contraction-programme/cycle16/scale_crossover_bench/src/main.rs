#![forbid(unsafe_code)]

mod b4_contract;
mod open_scaled;
mod sais;

use b4_contract::B4Contract;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use open_scaled::{FullRvSlp, OpenVariant, ScaledOpen};
use std::fs;
use std::time::Instant;

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = vals.len();
    if m % 2 == 1 {
        vals[m / 2]
    } else {
        0.5 * (vals[m / 2 - 1] + vals[m / 2])
    }
}

/// Fast deterministic SplitMix64 PRNG for reproducible point mutations.
struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn next_usize(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
}

/// Decode real Wikidata JSON text up to `target_bytes` from `corpora/wikidata.bz2` (stripping any 0x00).
fn load_wikidata_text(target_bytes: usize) -> Vec<u8> {
    let k = Kernels::detect();
    let bz2 = fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").expect("read wikidata.bz2");
    let mut scanner = Scanner::new(SliceSource(&bz2));
    let mut syms = Vec::new();
    let mut inv = OpenCells::default();
    let mut buf = Vec::with_capacity(900_064);
    let mut text = Vec::with_capacity(target_bytes + 900_000);
    while text.len() < target_bytes {
        match scanner.next(syms).expect("scan") {
            Event::Done => break,
            Event::Block { symbols, .. } => {
                let mut runs = Runs::default();
                let n = front(&symbols, &mut runs, k).expect("front");
                inv.invert_runs(&runs, n, symbols.orig_ptr, &mut buf, k);
                for &b in &buf[..n] {
                    text.push(if b == 0 { b' ' } else { b });
                }
                syms = symbols.syms;
            }
        }
    }
    text.truncate(target_bytes);
    text
}

/// Build realistic multi-version repetitive text of length `n` from `base_wiki`:
/// `k_copies` noisy versions of a `n / k_copies` Wikidata JSON slice with `mut_ppm` parts-per-million
/// deterministic point substitutions (and unique version tag to guarantee primitivity!).
fn make_repetitive_text(base_wiki: &[u8], n: usize, k_copies: usize, mut_ppm: u64, seed: u64) -> Vec<u8> {
    let mut rng = Rng(seed);
    let mut out = Vec::with_capacity(n);
    if k_copies <= 1 {
        out.extend_from_slice(&base_wiki[..n]);
        if mut_ppm > 0 {
            let n_muts = ((n as u64) * mut_ppm / 1_000_000) as usize;
            for _ in 0..n_muts {
                let idx = rng.next_usize(n);
                let new_b = b'!' + (rng.next_u64() % 93) as u8;
                out[idx] = new_b;
            }
        }
        return out;
    }

    let unit_len = (n + k_copies - 1) / k_copies;
    let template = &base_wiki[..unit_len];
    let muts_per_copy = ((unit_len as u64) * mut_ppm / 1_000_000).max(1) as usize;
    for copy_id in 0..k_copies {
        let start = out.len();
        let take = unit_len.min(n - start);
        if take == 0 {
            break;
        }
        out.extend_from_slice(&template[..take]);
        // Unique 4-byte ASCII version stamp at start of each copy (guarantees primitive single-cycle BWT!)
        if take >= 6 {
            let tag = format!("V{:04}", copy_id % 10000);
            out[start..start + 5].copy_from_slice(tag.as_bytes());
        }
        let m_here = if copy_id == 0 { 0 } else { (muts_per_copy * take) / unit_len.max(1) };
        for _ in 0..m_here {
            let off = 5 + rng.next_usize(take.saturating_sub(5).max(1));
            if start + off < out.len() {
                let new_b = b'!' + (rng.next_u64() % 93) as u8;
                out[start + off] = new_b;
            }
        }
    }
    assert_eq!(out.len(), n);
    out
}

enum CaseSource {
    CorpusBlock(&'static str, u64), // label, 0-based index in wikidata.bz2
    Synthetic(&'static str, usize, usize, u64, u64), // label, n, k_copies, mut_ppm, seed
}

fn load_corpus_block(target_idx: u64) -> (usize, u32, Vec<u16>, usize, Vec<u8>) {
    let k = Kernels::detect();
    let bz2 = fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").expect("read wikidata.bz2");
    let mut scanner = Scanner::new(SliceSource(&bz2));
    let mut syms = Vec::new();
    let mut inv = OpenCells::default();
    let mut truth = Vec::with_capacity(900_064);
    loop {
        match scanner.next(syms).expect("scan") {
            Event::Done => panic!("block {target_idx} not found"),
            Event::Block { index, symbols, .. } => {
                if index == target_idx {
                    let mut runs = Runs::default();
                    let n = front(&symbols, &mut runs, k).expect("front");
                    let recs = runs.records().to_vec();
                    let mut r0 = 0usize;
                    let mut i = 0usize;
                    while i < recs.len() {
                        let c = recs[i] >> 8;
                        i += 1;
                        while i < recs.len() && (recs[i] >> 8) == c {
                            i += 1;
                        }
                        r0 += 1;
                    }
                    inv.invert_runs(&runs, n, symbols.orig_ptr, &mut truth, k);
                    return (n, symbols.orig_ptr, recs, r0, truth);
                }
                syms = symbols.syms;
            }
        }
    }
}

fn main() {
    println!("=========================================================================================================");
    println!("CYCLE 16: REAL SILICON DEPLOYMENT CROSSOVER BENCHMARK (Open0512 / LeanNoMarkXor vs B4_Contract / FullRV_SLP)");
    println!("=========================================================================================================");

    // Load 32 MiB of real decoded Wikidata JSON text for multi-scale cases
    let t_load = Instant::now();
    let wiki_32m = load_wikidata_text(32 * 1024 * 1024);
    println!(
        "Loaded {:.2} MiB of real decoded Wikidata JSON from corpora/wikidata.bz2 in {:.2}s.",
        wiki_32m.len() as f64 / (1024.0 * 1024.0),
        t_load.elapsed().as_secs_f64()
    );

    let cases: Vec<CaseSource> = vec![
        // 900 KiB bzip2 scale: real block-210 (index 387, rho~7.0), real block-0 (index 1, rho~17.6), mod (rho~45), hi (rho~154), ultra (rho~550)
        CaseSource::CorpusBlock("900K_auth_b210(rho~7)", 387),
        CaseSource::CorpusBlock("900K_auth_b0(rho~18)", 1),
        CaseSource::Synthetic("900K_mod(K16,0.3%)", 900_000, 16, 3_000, 101),
        CaseSource::Synthetic("900K_hi(K64,0.08%)", 900_000, 64, 800, 102),
        CaseSource::Synthetic("900K_ultra(K256,0.02%)", 900_000, 256, 200, 103),
        // 4 MiB scale (exceeds 2 MiB L2!): rho~6.5, real wiki (rho~19), mod (~48), hi (~157), ultra (~550)
        CaseSource::Synthetic("4M_wiki_mut(rho~7)", 4 * 1024 * 1024, 1, 25_000, 200),
        CaseSource::Synthetic("4M_wiki_real", 4 * 1024 * 1024, 1, 0, 201),
        CaseSource::Synthetic("4M_mod(K16,0.3%)", 4 * 1024 * 1024, 16, 3_000, 202),
        CaseSource::Synthetic("4M_hi(K64,0.08%)", 4 * 1024 * 1024, 64, 800, 203),
        CaseSource::Synthetic("4M_ultra(K256,0.02%)", 4 * 1024 * 1024, 256, 200, 204),
        // 16 MiB scale (8x L2!): rho~7, real wiki (~19), mod (~49), hi (~163), ultra (~591), pangenome (~1600)
        CaseSource::Synthetic("16M_wiki_mut(rho~7)", 16 * 1024 * 1024, 1, 25_000, 300),
        CaseSource::Synthetic("16M_wiki_real", 16 * 1024 * 1024, 1, 0, 301),
        CaseSource::Synthetic("16M_mod(K16,0.3%)", 16 * 1024 * 1024, 16, 3_000, 302),
        CaseSource::Synthetic("16M_hi(K64,0.08%)", 16 * 1024 * 1024, 64, 800, 303),
        CaseSource::Synthetic("16M_ultra(K256,0.02%)", 16 * 1024 * 1024, 256, 200, 304),
        CaseSource::Synthetic("16M_pan(K512,0.005%)", 16 * 1024 * 1024, 512, 50, 305),
        // 32 MiB scale (16x L2!): mod (~50), hi (~163), ultra (~621), pangenome (~1800)
        CaseSource::Synthetic("32M_mod(K16,0.3%)", 32 * 1024 * 1024, 16, 3_000, 402),
        CaseSource::Synthetic("32M_hi(K64,0.08%)", 32 * 1024 * 1024, 64, 800, 403),
        CaseSource::Synthetic("32M_ultra(K256,0.02%)", 32 * 1024 * 1024, 256, 200, 404),
        CaseSource::Synthetic("32M_pan(K512,0.005%)", 32 * 1024 * 1024, 512, 50, 405),
        // 64 MiB scale (32x L2!): hi (~182), ultra (~690), pangenome (~2100)
        CaseSource::Synthetic("64M_hi(K128,0.08%)", 64 * 1024 * 1024, 128, 800, 503),
        CaseSource::Synthetic("64M_ultra(K512,0.02%)", 64 * 1024 * 1024, 512, 200, 504),
        CaseSource::Synthetic("64M_pan(K1024,0.005%)", 64 * 1024 * 1024, 1024, 50, 505),
    ];

    println!(
        "\n{:23} | {:>6} {:>7} {:>6} | {:>7} {:>1} {:>6} | {:>19} | {:>7} | {:>20} | {:>20} | {:>7} {:>7} {:>7} {:>7}",
        "Case",
        "n_MB",
        "r0",
        "rho",
        "m16",
        "W",
        "edgMB",
        "Open0512 b/e/tot/pk",
        "NoMk_ms",
        "B4_Cnt c/x/tot/pk",
        "FullRV c/x/tot/pk",
        "B4/Op",
        "RV/Op",
        "B4c/Opb",
        "PkB4/Op"
    );
    println!("{}", "-".repeat(170));

    let prefix_filter = std::env::args().nth(1);
    for src in &cases {
        let lbl_peek = match *src {
            CaseSource::CorpusBlock(l, _) | CaseSource::Synthetic(l, ..) => l,
        };
        if let Some(ref pf) = prefix_filter {
            if !lbl_peek.starts_with(pf.as_str()) {
                continue;
            }
        }
        let (label, n, p, recs, r0, truth_text) = match *src {
            CaseSource::CorpusBlock(lbl, idx) => {
                let (n, p, recs, r0, truth) = load_corpus_block(idx);
                (lbl, n, p, recs, r0, truth)
            }
            CaseSource::Synthetic(lbl, n, k_copies, mut_ppm, seed) => {
                let text = make_repetitive_text(&wiki_32m, n, k_copies, mut_ppm, seed);
                let (raw_l, orig_p) = sais::circular_bwt(&text);
                let (recs, r0) = sais::to_r127_records(&raw_l);
                (lbl, n, orig_p, recs, r0, text)
            }
        };

        let rho = n as f64 / r0 as f64;
        let rounds = if n <= 1_000_000 {
            25
        } else if n <= 4 * 1024 * 1024 {
            5
        } else {
            3
        };

        // Fresh kernel instances per grid point so peak_kernel_bytes is 100% isolated!
        let mut k_op = ScaledOpen::default();
        let mut k_nm = ScaledOpen::default();
        let mut k_b4 = B4Contract::default();
        let mut k_rv = FullRvSlp::default();
        let mut out = Vec::with_capacity(n + 32);

        // Warmup + STRICT BYTE-FOR-BYTE VERIFICATION on all 4 kernels!
        let st_op0 = k_op.decode(&recs, n, p, OpenVariant::Open0512, &mut out);
        assert_eq!(out, truth_text, "Open0512 MISMATCH on {label}");

        k_nm.decode(&recs, n, p, OpenVariant::LeanNoMarkXor, &mut out);
        assert_eq!(out, truth_text, "LeanNoMarkXor MISMATCH on {label}");

        let st_b40 = k_b4.decode(&recs, n, p, &mut out);
        assert_eq!(out, truth_text, "B4_Contract MISMATCH on {label}");

        let st_rv0 = k_rv.decode(&recs, n, p, &mut out);
        assert_eq!(out, truth_text, "FullRV_SLP MISMATCH on {label}");

        // Paired multi-round median timing
        let (mut op_b, mut op_e, mut op_t) = (Vec::new(), Vec::new(), Vec::new());
        let mut nm_t = Vec::new();
        let (mut b4_c, mut b4_x, mut b4_t) = (Vec::new(), Vec::new(), Vec::new());
        let (mut rv_c, mut rv_x, mut rv_t) = (Vec::new(), Vec::new(), Vec::new());

        for _ in 0..rounds {
            let s = k_op.decode(&recs, n, p, OpenVariant::Open0512, &mut out);
            op_b.push(s.build_ms);
            op_e.push(s.emit_ms);
            op_t.push(s.total_ms);

            let s = k_nm.decode(&recs, n, p, OpenVariant::LeanNoMarkXor, &mut out);
            nm_t.push(s.total_ms);

            let s = k_b4.decode(&recs, n, p, &mut out);
            b4_c.push(s.contract_ms);
            b4_x.push(s.expand_ms);
            b4_t.push(s.total_ms);

            let s = k_rv.decode(&recs, n, p, &mut out);
            rv_c.push(s.contract_ms);
            rv_x.push(s.expand_ms);
            rv_t.push(s.total_ms);
        }

        let (mob, moe, mot) = (median(&mut op_b), median(&mut op_e), median(&mut op_t));
        let mnt = median(&mut nm_t);
        let (mbc, mbx, mbt) = (median(&mut b4_c), median(&mut b4_x), median(&mut b4_t));
        let (mrc, mrx, mrt) = (median(&mut rv_c), median(&mut rv_x), median(&mut rv_t));

        let op_pk = st_op0.peak_kernel_bytes as f64 / (1024.0 * 1024.0);
        let b4_pk = st_b40.peak_kernel_bytes as f64 / (1024.0 * 1024.0);
        let rv_pk = st_rv0.peak_kernel_bytes as f64 / (1024.0 * 1024.0);
        let edg_mb = st_op0.edges_bytes as f64 / (1024.0 * 1024.0);
        let sp_len_mb = st_op0.spelling_len as f64 / (1024.0 * 1024.0);
        let sp_cap_mb = st_op0.spelling_cap as f64 / (1024.0 * 1024.0);
        let avg_memo = if st_rv0.memo_hits > 0 {
            st_rv0.memo_bytes as f64 / st_rv0.memo_hits as f64
        } else {
            0.0
        };

        println!(
            "{:23} | {:6.2} {:7} {:6.1} | {:7} {:>1} {:6.2} | {:5.1}/{:5.1}/{:5.1}/{:4.1}M | {:7.2} | {:5.1}/{:4.1}/{:5.1}/{:4.1}M | {:5.1}/{:4.1}/{:5.1}/{:4.1}M | {:6.3}x {:6.3}x {:6.3}x {:6.3}x",
            label,
            n as f64 / (1024.0 * 1024.0),
            r0,
            rho,
            st_op0.m,
            if st_op0.wide { "Y" } else { "N" },
            edg_mb,
            mob,
            moe,
            mot,
            op_pk,
            mnt,
            mbc,
            mbx,
            mbt,
            b4_pk,
            mrc,
            mrx,
            mrt,
            rv_pk,
            mbt / mot,
            mrt / mot,
            mbc / mob.max(1e-6),
            b4_pk / op_pk.max(1e-6),
        );
        println!(
            "    -> [EXACT PASS] Open: ns={:6}, codes={:5}, spell={:.2}/{:.2}M | B4: ops={:7}, hub={:6}, slp={:7} | RV: steps={:7}, slp={:7}, pk={:.2}M, memo={:6} hits (avg {:.0}B, {:.1}% n)",
            st_op0.ns,
            st_op0.codes,
            sp_len_mb,
            sp_cap_mb,
            st_b40.b4_ops,
            st_b40.hub_pieces,
            st_b40.slp_nodes,
            st_rv0.rv_steps,
            st_rv0.slp_rules,
            rv_pk,
            st_rv0.memo_hits,
            avg_memo,
            100.0 * st_rv0.memo_bytes as f64 / n as f64,
        );
    }
}
