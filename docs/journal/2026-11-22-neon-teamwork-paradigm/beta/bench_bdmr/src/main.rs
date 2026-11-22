#![forbid(unsafe_code)]

mod bdmr;
mod sha256;

use bdmr::{BdmrKernel, ColorMode};
use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front, Runs};
use mibz2::Kernels;
use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

struct BlockSpec {
    file: &'static str,
    n: usize,
    p: u32,
    r: usize,
    sha256: &'static str,
}

const BLOCKS: [BlockSpec; 6] = [
    BlockSpec {
        file: "block-0.L",
        n: 899_931,
        p: 264_045,
        r: 51_184,
        sha256: "db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67",
    },
    BlockSpec {
        file: "block-2.L",
        n: 900_000,
        p: 320_807,
        r: 54_532,
        sha256: "8b47073232d1d9cff7a0688399cfd405211045e15228706f213780e083fc4174",
    },
    BlockSpec {
        file: "block-85.L",
        n: 899_935,
        p: 325_088,
        r: 79_375,
        sha256: "5def63ec66c914e1986a97f8206a8debd09f6e936f0bc8d8f4cab8544ff64fe8",
    },
    BlockSpec {
        file: "block-143.L",
        n: 899_869,
        p: 802_868,
        r: 108_335,
        sha256: "eb443421d4d7b459a1db02a00c068f4c2f1e3b7e70f85674228ea2cba30ceba2",
    },
    BlockSpec {
        file: "block-210.L",
        n: 899_959,
        p: 791_429,
        r: 128_732,
        sha256: "dd4d0c7d03dd6bc7ec8752ad03b2fd5607222c54bebab8d81ae98a2c277f3dcd",
    },
    BlockSpec {
        file: "block-207.L",
        n: 899_998,
        p: 205_180,
        r: 87_111,
        sha256: "7343ef019b44f0e838e68abc09e36d9806a0f45249cf2286702a2e482e555099",
    },
];

/// Encode raw BWT column L into exact bzip2 MTF + RUNA/RUNB Symbols so mibz2::runs::front
/// produces the exact production `mibz2::runs::Runs` object.
fn build_runs_from_column(ll: &[u8], p: u32, k: Kernels) -> Runs {
    let mut present = [false; 256];
    for &b in ll {
        present[b as usize] = true;
    }
    let mut seq_to_unseq = [0u8; 256];
    let mut mtf_list = [0u8; 256];
    let mut alpha = 0usize;
    for b in 0..256usize {
        if present[b] {
            seq_to_unseq[alpha] = b as u8;
            mtf_list[alpha] = b as u8;
            alpha += 1;
        }
    }

    let mut syms: Vec<u16> = Vec::with_capacity(ll.len() / 2);
    let mut zrun = 0usize;
    let flush_zrun = |z: &mut usize, out: &mut Vec<u16>| {
        let mut r = *z;
        while r > 0 {
            if (r & 1) == 1 {
                out.push(0); // RUNA
                r = (r - 1) >> 1;
            } else {
                out.push(1); // RUNB
                r = (r - 2) >> 1;
            }
        }
        *z = 0;
    };

    for &b in ll {
        if mtf_list[0] == b {
            zrun += 1;
        } else {
            flush_zrun(&mut zrun, &mut syms);
            let mut pos = 1usize;
            while mtf_list[pos] != b {
                pos += 1;
            }
            for j in (1..=pos).rev() {
                mtf_list[j] = mtf_list[j - 1];
            }
            mtf_list[0] = b;
            syms.push((pos + 1) as u16);
        }
    }
    flush_zrun(&mut zrun, &mut syms);
    let eob = (alpha + 1) as u16;
    syms.push(eob);

    let symbols = Symbols {
        crc: 0,
        orig_ptr: p,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    };

    let mut runs = Runs::default();
    let n_out = front(&symbols, &mut runs, k).expect("runs::front failed");
    assert_eq!(n_out, ll.len());

    // Verify runs.records() expands to ll byte-for-byte!
    let mut recon = Vec::with_capacity(ll.len());
    for &rec in runs.records() {
        let byte = (rec >> 8) as u8;
        let len = (rec & 255) as usize;
        recon.resize(recon.len() + len, byte);
    }
    assert_eq!(recon.as_slice(), ll, "Runs::records() mismatch vs raw L!");
    runs
}

/// Exact Open retained row count m_16 (thin maximal runs w <= 16, plus p if inside thick strip).
fn count_m16(ll: &[u8], p: u32) -> usize {
    let mut m = 0usize;
    let mut i = 0usize;
    let pu = p as usize;
    while i < ll.len() {
        let c = ll[i];
        let mut j = i + 1;
        while j < ll.len() && ll[j] == c {
            j += 1;
        }
        let w = j - i;
        if w <= 16 {
            m += w;
        } else if i <= pu && pu < j {
            m += 1;
        }
        i = j;
    }
    m
}

fn ref_inverse_small(ll: &[u8], p: u32) -> Vec<u8> {
    let n = ll.len();
    let mut counts = [0usize; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    let mut c_off = [0usize; 256];
    let mut tot = 0;
    for c in 0..256 {
        c_off[c] = tot;
        tot += counts[c];
    }
    let mut f = vec![0u8; n];
    for c in 0..256 {
        f[c_off[c]..c_off[c] + counts[c]].fill(c as u8);
    }
    let mut pos = c_off;
    let mut fl = vec![0usize; n];
    for (i, &c) in ll.iter().enumerate() {
        fl[pos[c as usize]] = i;
        pos[c as usize] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut cur = p as usize;
    for _ in 0..n {
        out.push(f[cur]);
        cur = fl[cur];
    }
    out
}

fn verify_exhaustive_small() {
    let configs = [
        (ColorMode::HashThresh, 1),
        (ColorMode::HashThresh, 4),
        (ColorMode::GreedyThresh, 1),
        (ColorMode::GreedyThresh, 4),
        (ColorMode::ThreshOnly, 1),
    ];
    for &(mode, t_cut) in &configs {
        let mut k = BdmrKernel::new(mode, t_cut);
        let mut out = Vec::new();
        let mut total = 0usize;
        for &(base, max_n) in &[(2usize, 8usize), (3, 5)] {
            for n in 1..=max_n {
                let lim = base.pow(n as u32);
                for mut z in 0..lim {
                    let mut ll = Vec::with_capacity(n);
                    for _ in 0..n {
                        ll.push((z % base) as u8);
                        z /= base;
                    }
                    let counts = [0u32; 256];
                    for p in 0..(n as u32) {
                        let ref_out = ref_inverse_small(&ll, p);
                        k.invert(&ll, &counts, p, &mut out);
                        assert_eq!(out, ref_out, "FAIL {:?} t_cut={} L={:?} p={}", mode, t_cut, ll, p);
                        total += 1;
                    }
                }
            }
        }
        assert_eq!(total, 5227);
    }
    println!("[VERIFIED] Exhaustive small cases: 5,227 / 5,227 x 5 configs (26,135 runs) PASSED!");
}

fn median_iqr(vals: &mut [f64]) -> (f64, f64) {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = vals.len();
    let med = if n % 2 == 1 {
        vals[n / 2]
    } else {
        0.5 * (vals[n / 2 - 1] + vals[n / 2])
    };
    let q1 = vals[n / 4];
    let q3 = vals[(3 * n) / 4];
    (med, q3 - q1)
}

struct PreparedBlock {
    spec: &'static BlockSpec,
    runs: Runs,
    m16: usize,
    ref_out: Vec<u8>,
}

fn bench_paired_1t<FA, FB>(mut run_a: FA, mut run_b: FB, warmups: usize, iters: usize) -> (f64, f64, f64, f64)
where
    FA: FnMut(),
    FB: FnMut(),
{
    for _ in 0..warmups {
        run_a();
        run_b();
    }
    let mut ta = Vec::with_capacity(iters);
    let mut tb = Vec::with_capacity(iters);
    for r in 0..iters {
        if r % 2 == 0 {
            let t0 = Instant::now();
            run_a();
            ta.push(t0.elapsed().as_secs_f64() * 1e3);
            let t1 = Instant::now();
            run_b();
            tb.push(t1.elapsed().as_secs_f64() * 1e3);
        } else {
            let t1 = Instant::now();
            run_b();
            tb.push(t1.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now();
            run_a();
            ta.push(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    let (ma, ia) = median_iqr(&mut ta);
    let (mb, ib) = median_iqr(&mut tb);
    (ma, ia, mb, ib)
}

fn run_4w_batch<K: InverseBwt + 'static>(
    blocks: &Arc<Vec<PreparedBlock>>,
    make_k: &(impl Fn() -> K + Sync),
    kernels: Kernels,
) -> f64 {
    let barrier = Arc::new(Barrier::new(4));
    let mut elapsed_ms = 0.0f64;
    thread::scope(|s| {
        let mut handles = Vec::with_capacity(4);
        for w in 0..4 {
            let b = Arc::clone(blocks);
            let bar = Arc::clone(&barrier);
            handles.push(s.spawn(move || {
                let mut k = make_k();
                let mut out = Vec::with_capacity(900_000 + 32);
                // Warm up kernel allocations once outside clock
                let _ = k.invert_runs(&b[w % b.len()].runs, b[w % b.len()].spec.n, b[w % b.len()].spec.p, &mut out, kernels);
                bar.wait();
                let t0 = Instant::now();
                // Each worker decodes all 6 blocks (staggered start order so workers hit different blocks simultaneously)
                for step in 0..b.len() {
                    let blk = &b[(w + step) % b.len()];
                    let ok = k.invert_runs(&blk.runs, blk.spec.n, blk.spec.p, &mut out, kernels);
                    assert!(ok);
                    std::hint::black_box(&out);
                }
                bar.wait();
                t0.elapsed().as_secs_f64() * 1e3
            }));
        }
        let mut max_t = 0.0f64;
        for h in handles {
            let t = h.join().unwrap();
            if t > max_t {
                max_t = t;
            }
        }
        elapsed_ms = max_t;
    });
    elapsed_ms
}

fn main() {
    let k = Kernels::detect();
    println!("=== BDMR vs OpenCells (mibz2 0.5.11) Native Safe-Rust A/B Benchmark ===");
    println!("Kernels detected: {:?}\n", k);

    // 1. Exhaustive small-case verification
    verify_exhaustive_small();

    // 2. Load & SHA-256 authenticate all 6 blocks
    let mut blocks: Vec<PreparedBlock> = Vec::with_capacity(BLOCKS.len());
    let mut open_ref = OpenCells::default();
    for spec in &BLOCKS {
        let path = format!("/tmp/mibz2-blocks/{}", spec.file);
        let ll = fs::read(&path).expect("failed to read block");
        let got_sha = sha256::sha256_hex(&ll);
        assert_eq!(got_sha, spec.sha256, "SHA-256 mismatch on {}", spec.file);
        assert_eq!(ll.len(), spec.n);
        let runs = build_runs_from_column(&ll, spec.p, k);
        let m16 = count_m16(&ll, spec.p);
        let mut ref_out = Vec::new();
        let ok = open_ref.invert_runs(&runs, spec.n, spec.p, &mut ref_out, k);
        assert!(ok && ref_out.len() == spec.n);
        blocks.push(PreparedBlock { spec, runs, m16, ref_out });
    }
    println!("[AUTHENTICATED] All 6 Wikidata blocks SHA-256 verified and Runs built identically to mibz2::runs::front.\n");

    // 3. Exactness check & detailed stage/cost diagnostics across BDMR configurations
    let bdmr_configs = [
        ("GreedyThresh T=1", ColorMode::GreedyThresh, 1u32),
        ("GreedyThresh T=4096", ColorMode::GreedyThresh, 4096u32),
        ("HashThresh   T=1", ColorMode::HashThresh, 1u32),
        ("HashThresh   T=4096", ColorMode::HashThresh, 4096u32),
        ("ThreshOnly   T=4096", ColorMode::ThreshOnly, 4096u32),
    ];

    println!("=== EXACTNESS & STAGE/COST DIAGNOSTICS (6 BLOCKS) ===");
    for &(label, mode, t_cut) in &bdmr_configs {
        let mut ker = BdmrKernel::new(mode, t_cut);
        ker.record_stats = true;
        let mut out = Vec::with_capacity(900_000);
        println!("\n--- Config: {} ---", label);
        for blk in &blocks {
            // Warmup once so scratch capacity is steady, then measure instrumented
            ker.invert_runs(&blk.runs, blk.spec.n, blk.spec.p, &mut out, k);
            assert_eq!(out, blk.ref_out, "EXACTNESS MISMATCH {} on {}", label, blk.spec.file);
            ker.invert_runs(&blk.runs, blk.spec.n, blk.spec.p, &mut out, k);
            let s = ker.last_stats;
            let q_con = s.q_indep_scatter + s.q_indep_stream;
            let q_exp = s.q_expand_visits + s.q_expand_copies;
            let tot_ms = (s.acquire_ns + s.contract_ns + s.residual_ns + s.expand_ns + s.rotate_ns) as f64 / 1e6;
            println!(
                "  {:12} | r0={:6} m16={:6} | rnd={:2} sum_r={:7} ({:.2}x) | Q_con={:7} (scat={:6}+strm={:6}, {:.2}x m16) Q_exp={:6} (vis={:5}+cpy={:5}) | live_M={:4}K alloc_M={:4}K | ms={:5.2} [acq={:.2} con={:.2} res={:.2} exp={:.2} rot={:.2}]",
                blk.spec.file,
                blk.spec.r,
                blk.m16,
                s.rounds,
                s.sum_r,
                s.sum_r as f64 / blk.spec.r as f64,
                q_con,
                s.q_indep_scatter,
                s.q_indep_stream,
                q_con as f64 / blk.m16 as f64,
                q_exp,
                s.q_expand_visits,
                s.q_expand_copies,
                s.peak_scratch_bytes / 1024,
                ker.scratch_bytes_allocated() / 1024,
                tot_ms,
                s.acquire_ns as f64 / 1e6,
                s.contract_ns as f64 / 1e6,
                s.residual_ns as f64 / 1e6,
                s.expand_ns as f64 / 1e6,
                s.rotate_ns as f64 / 1e6,
            );
        }
    }

    // 4. 1-THREAD PAIRED INTERLEAVED A/A & A/B BENCHMARK (NO INSTRUMENTATION OVERHEAD!)
    println!("\n=== 1-THREAD PAIRED INTERLEAVED BENCHMARK (15 paired iters, 3 warmups, median ± IQR ms) ===");
    let mut open_a = OpenCells::default();
    let mut open_b = OpenCells::default();
    let mut out_a = Vec::with_capacity(900_000 + 32);
    let mut out_b = Vec::with_capacity(900_000 + 32);

    println!("\n[A/A Control: OpenCells vs OpenCells (1T)]");
    for blk in &blocks {
        let (ma, ia, mb, ib) = bench_paired_1t(
            || {
                let ok = open_a.invert_runs(&blk.runs, blk.spec.n, blk.spec.p, &mut out_a, k);
                assert!(ok);
                std::hint::black_box(&out_a);
            },
            || {
                let ok = open_b.invert_runs(&blk.runs, blk.spec.n, blk.spec.p, &mut out_b, k);
                assert!(ok);
                std::hint::black_box(&out_b);
            },
            3,
            15,
        );
        println!(
            "  {:12}: Open_A={:6.3}±{:.3} ms | Open_B={:6.3}±{:.3} ms | B/A={:.3}x",
            blk.spec.file, ma, ia, mb, ib, mb / ma
        );
    }

    let ab_configs = [
        ("BDMR HashThresh T=4096", ColorMode::HashThresh, 4096u32),
        ("BDMR GreedyThresh T=4096", ColorMode::GreedyThresh, 4096u32),
        ("BDMR ThreshOnly T=4096", ColorMode::ThreshOnly, 4096u32),
    ];

    for &(label, mode, t_cut) in &ab_configs {
        let mut bdmr_k = BdmrKernel::new(mode, t_cut);
        bdmr_k.record_stats = false;
        println!("\n[A/B (1T): {} vs OpenCells]", label);
        let mut sum_open = 0.0;
        let mut sum_bdmr = 0.0;
        for blk in &blocks {
            let (ma, ia, mb, ib) = bench_paired_1t(
                || {
                    let ok = open_a.invert_runs(&blk.runs, blk.spec.n, blk.spec.p, &mut out_a, k);
                    assert!(ok);
                    std::hint::black_box(&out_a);
                },
                || {
                    let ok = bdmr_k.invert_runs(&blk.runs, blk.spec.n, blk.spec.p, &mut out_b, k);
                    assert!(ok);
                    std::hint::black_box(&out_b);
                },
                3,
                15,
            );
            sum_open += ma;
            sum_bdmr += mb;
            println!(
                "  {:12} (m16={:6}): Open={:6.3}±{:.3} ms | BDMR={:6.3}±{:.3} ms | BDMR/Open={:.2}x",
                blk.spec.file, blk.m16, ma, ia, mb, ib, mb / ma
            );
        }
        println!(
            "  TOTAL 6-BLOCK (5.40 MB): Open={:6.2} ms ({:.1} MB/s) | BDMR={:6.2} ms ({:.1} MB/s) | BDMR/Open={:.2}x",
            sum_open,
            5.399692 / (sum_open / 1e3),
            sum_bdmr,
            5.399692 / (sum_bdmr / 1e3),
            sum_bdmr / sum_open
        );
    }

    // 5. 4-CONCURRENT-WORKER (4W) PAIRED INTERLEAVED BENCHMARK (4 workers x 6 blocks = 21.60 MB per batch!)
    println!("\n=== 4-CONCURRENT-WORKER (4W) PAIRED INTERLEAVED BENCHMARK (4 workers x 6 blocks = 21.60 MB/batch, 11 paired batches) ===");
    let blocks_arc = Arc::new(blocks);

    // A/A control at 4W
    let mut aa_a = Vec::new();
    let mut aa_b = Vec::new();
    for r in 0..11 {
        if r % 2 == 0 {
            aa_a.push(run_4w_batch(&blocks_arc, &OpenCells::default, k));
            aa_b.push(run_4w_batch(&blocks_arc, &OpenCells::default, k));
        } else {
            aa_b.push(run_4w_batch(&blocks_arc, &OpenCells::default, k));
            aa_a.push(run_4w_batch(&blocks_arc, &OpenCells::default, k));
        }
    }
    let (ma, ia) = median_iqr(&mut aa_a);
    let (mb, ib) = median_iqr(&mut aa_b);
    println!(
        "  [A/A Control 4W] Open_A={:6.2}±{:.2} ms ({:.1} MB/s) | Open_B={:6.2}±{:.2} ms ({:.1} MB/s) | B/A={:.3}x",
        ma, ia, 21.598768 / (ma / 1e3), mb, ib, 21.598768 / (mb / 1e3), mb / ma
    );

    for &(label, mode, t_cut) in &ab_configs {
        let mut ta = Vec::new();
        let mut tb = Vec::new();
        for r in 0..11 {
            if r % 2 == 0 {
                ta.push(run_4w_batch(&blocks_arc, &OpenCells::default, k));
                tb.push(run_4w_batch(&blocks_arc, &|| BdmrKernel::new(mode, t_cut), k));
            } else {
                tb.push(run_4w_batch(&blocks_arc, &|| BdmrKernel::new(mode, t_cut), k));
                ta.push(run_4w_batch(&blocks_arc, &OpenCells::default, k));
            }
        }
        let (ma, ia) = median_iqr(&mut ta);
        let (mb, ib) = median_iqr(&mut tb);
        println!(
            "  [A/B 4W: {:24}] Open={:6.2}±{:.2} ms ({:.1} MB/s) | BDMR={:6.2}±{:.2} ms ({:.1} MB/s) | BDMR/Open={:.2}x",
            label,
            ma, ia, 21.598768 / (ma / 1e3),
            mb, ib, 21.598768 / (mb / 1e3),
            mb / ma
        );
    }
}
