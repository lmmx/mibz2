//! Carbon Cycle 19 Gate G5 Confrontation Harness (`g5_unified_c19.rs`):
//! Evaluates the drop-in `#[forbid(unsafe_code)]` Rust 1.80 candidate `candidate_open_c19.rs`:
//!   - Arm 0: `Master-0.5.8` (`mibz2::bwt::OpenCells`, shipped baseline, `228/431` run blocks)
//!   - Arm 1: `C19-SameGate-Mux16` (`SameGateEdgesMuxOpenCells`, exact `0.5.8` `Gate 0.15`, `228/431`)
//!   - Arm 2: `C19-AhtEdgesMux16` (`Candidate` = `AhtEdgesMuxOpenCells`, `383/431` primary candidate)
//!   - Arm 3: `C19-AhtEdgesMux14` (`AhtEdgesMux14OpenCells`, `383/431` `LIMIT = 14` L3 co-variant)
//!
//! Measures:
//!   - G5.1: Adversarial spirals (`ra > rb` & `ra < rb`), poisoned-`edges` `ovf_chunks` spill,
//!           sticky `m = u32::MAX` `< 32 KiB` early abort, pure column-block idle overhead,
//!           431-block byte-exactness, and `BzReader` 209.7 MB parallel stream exactness.
//!   - G5.2: `TrackAlloc` per-worker retained & peak heap on both the `228`-block `0.5.8` subset
//!           AND `10`-worker concurrent whole-corpus (`431` blocks).
//!   - G5.3: Dual-suite `chaingrind` (`cg_s9` on `sample9_admitted.bz2` AND `cg_dense` on the 9-block
//!           mixed high-density window `[40, 41, 55, 56, 164, 165, 269, 270, 278]`) at `W = 8`
//!           (`--llc-kb=2560`) and `W = 10` (`--llc-kb=2048`), reporting `instructions`,
//!           `dep_past_l2_loads`, `DRAM B/outB`, and `cycles/B`.
//!   - G5.4: 15-round rotated native `lto=fat` timing across `W = 1, 2, 8, 10` on BOTH the
//!           `228`-block `0.5.8` subset AND all `431` blocks + 11-round end-to-end
//!           `BzReader::parallel_workers_with` (`4 * W` window) at `W = 8, 10`.

pub use mibz2::{bwt, runs, Kernels};

#[path = "candidate_open_c19.rs"]
pub mod candidate_open_c19;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use candidate_open_c19::{
    AhtEdgesMux14OpenCells, AhtEdgesMuxOpenCells, SameGateEdgesMuxOpenCells, UnifiedEdgesMuxKernel,
};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::mtf;
use mibz2::runs::Runs;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::BzReader;

struct TrackAlloc;
thread_local! {
    static TLS_LIVE: Cell<isize> = const { Cell::new(0) };
    static TLS_PEAK: Cell<isize> = const { Cell::new(0) };
}
fn tls_reset() {
    TLS_LIVE.with(|l| {
        l.set(0);
        TLS_PEAK.with(|p| p.set(0));
    });
}
fn tls_stats() -> (usize, usize) {
    (
        TLS_LIVE.with(|l| l.get()).max(0) as usize,
        TLS_PEAK.with(|p| p.get()).max(0) as usize,
    )
}
unsafe impl GlobalAlloc for TrackAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let _ = TLS_LIVE.try_with(|l| {
                let nl = l.get() + layout.size() as isize;
                l.set(nl);
                let _ = TLS_PEAK.try_with(|pk| if nl > pk.get() { pk.set(nl) });
            });
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            let _ = TLS_LIVE.try_with(|l| {
                let nl = l.get() + layout.size() as isize;
                l.set(nl);
                let _ = TLS_PEAK.try_with(|pk| if nl > pk.get() { pk.set(nl) });
            });
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = TLS_LIVE.try_with(|l| l.set(l.get() - layout.size() as isize));
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let np = unsafe { System.realloc(ptr, layout, new_size) };
        if !np.is_null() {
            let d = new_size as isize - layout.size() as isize;
            let _ = TLS_LIVE.try_with(|l| {
                let nl = l.get() + d;
                l.set(nl);
                let _ = TLS_PEAK.try_with(|pk| if nl > pk.get() { pk.set(nl) });
            });
        }
        np
    }
}
#[global_allocator]
static GLOBAL: TrackAlloc = TrackAlloc;

#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn client_request(default: u64, args: &[u64; 6]) -> u64 {
    let mut r = default;
    let ptr = std::hint::black_box(args.as_ptr());
    unsafe {
        std::arch::asm!(
            "rol rdi, 3", "rol rdi, 13", "rol rdi, 61", "rol rdi, 51", "xchg rbx, rbx",
            inout("rdx") r,
            in("rax") ptr,
            options(nostack),
        );
    }
    r
}
const REQ_BEGIN: u64 = (b'M' as u64) << 24 | (b'S' as u64) << 16;
const REQ_END: u64 = REQ_BEGIN + 1;
const REQ_WARM: u64 = REQ_BEGIN + 2;

enum CorpusBlk {
    RunsBlk {
        runs: Runs,
        n: usize,
        p: u32,
        admitted_058: bool,
    },
    ColBlk {
        ll: Vec<u8>,
        counts: [u32; 256],
        p: u32,
    },
}

fn load_all(bz2_path: &str) -> Vec<CorpusBlk> {
    let data = std::fs::read(bz2_path).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut syms = Vec::new();
    let mut out = Vec::new();
    while let Ok(Event::Block { symbols: b, .. }) = sc.next(std::mem::take(&mut syms)) {
        let p = b.orig_ptr;
        if runs::takes_runs(&b) {
            let mut r = Runs::default();
            let n = runs::front(&b, &mut r, k).unwrap();
            let nr = r.records().len();
            let admitted_058 = nr * 20 < n * 3 && nr * 64 >= n;
            out.push(CorpusBlk::RunsBlk {
                runs: r,
                n,
                p,
                admitted_058,
            });
            syms = b.syms;
        } else {
            let mut ll = Vec::new();
            let mut counts = [0u32; 256];
            mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
            out.push(CorpusBlk::ColBlk { ll, counts, p });
            syms = b.syms;
        }
    }
    out
}

fn load_dense_picks(bz2_path: &str) -> Vec<CorpusBlk> {
    let picks = [40usize, 41, 55, 56, 164, 165, 269, 270, 278];
    let max_pick = *picks.iter().max().unwrap();
    let data = std::fs::read(bz2_path).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut syms = Vec::new();
    let mut out = Vec::new();
    let mut idx = 0usize;
    while let Ok(Event::Block { symbols: b, .. }) = sc.next(std::mem::take(&mut syms)) {
        if picks.contains(&idx) {
            let p = b.orig_ptr;
            if runs::takes_runs(&b) {
                let mut r = Runs::default();
                let n = runs::front(&b, &mut r, k).unwrap();
                let nr = r.records().len();
                let admitted_058 = nr * 20 < n * 3 && nr * 64 >= n;
                out.push(CorpusBlk::RunsBlk {
                    runs: r,
                    n,
                    p,
                    admitted_058,
                });
            } else {
                let mut ll = Vec::new();
                let mut counts = [0u32; 256];
                mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
                out.push(CorpusBlk::ColBlk { ll, counts, p });
            }
        }
        syms = b.syms;
        if idx >= max_pick {
            break;
        }
        idx += 1;
    }
    assert_eq!(out.len(), picks.len());
    out
}

fn invert_any<K: InverseBwt>(kern: &mut K, blk: &CorpusBlk, out: &mut Vec<u8>, k: Kernels) {
    match blk {
        CorpusBlk::RunsBlk { runs, n, p, .. } => {
            assert!(kern.invert_runs(runs, *n, *p, out, k));
        }
        CorpusBlk::ColBlk { ll, counts, p } => {
            kern.invert(ll, counts, *p, out);
        }
    }
}

fn run_cg_suite<K: InverseBwt>(all: &[CorpusBlk]) {
    let k = Kernels::detect();
    let mut kern = K::default();
    let mut out = Vec::with_capacity(900_064);
    for b in all.iter().skip(1) {
        invert_any(&mut kern, b, &mut out, k);
    }
    client_request(0, &[REQ_WARM, 0, 0, 0, 0, 0]);
    invert_any(&mut kern, &all[0], &mut out, k);
    let mut total = 0u64;
    client_request(0, &[REQ_BEGIN, 0, 0, 0, 0, 0]);
    for b in all.iter().skip(1) {
        invert_any(&mut kern, b, &mut out, k);
        std::hint::black_box(out.as_ptr());
        total += out.len() as u64;
    }
    client_request(0, &[REQ_END, total, 0, 0, 0, 0]);
}

#[derive(Clone, Copy, Debug)]
struct CgMetrics {
    instrs_m: f64,
    dep_l3_k: f64,
    dram_b_per_b: f64,
    cyc_per_b: f64,
}

fn spawn_cg_matrix(exe: &str) -> std::collections::HashMap<(String, usize, String), CgMetrics> {
    let arms = ["058", "same16", "mux16", "mux14"];
    let suites = ["s9", "dense"];
    let llcs = [2560usize, 2048];
    let mut jobs = Vec::new();
    for &su in &suites {
        for &llc in &llcs {
            for &arm in &arms {
                jobs.push((su.to_string(), llc, arm.to_string()));
            }
        }
    }
    let next = AtomicUsize::new(0);
    let results = std::sync::Mutex::new(std::collections::HashMap::new());
    std::thread::scope(|sc| {
        let jobs_r = &jobs;
        let next_r = &next;
        let res_r = &results;
        for _ in 0..4 {
            sc.spawn(move || loop {
                let i = next_r.fetch_add(1, Ordering::Relaxed);
                if i >= jobs_r.len() {
                    break;
                }
                let (ref su, llc, ref arm) = jobs_r[i];
                let bz_arg = if su == "s9" {
                    "/tmp/sample9_admitted.bz2"
                } else {
                    "/tmp/wikidata.bz2"
                };
                let out = std::process::Command::new("valgrind")
                    .env("VALGRIND_LIB", "/tmp/chaingrind_lib")
                    .arg("--tool=chaingrind")
                    .arg(format!("--llc-kb={llc}"))
                    .arg(exe)
                    .arg(format!("cg_{su}:{arm}"))
                    .arg(bz_arg)
                    .output()
                    .expect("failed to run valgrind chaingrind");
                assert!(
                    out.status.success(),
                    "chaingrind failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
                let stderr = String::from_utf8_lossy(&out.stderr);
                let line = stderr
                    .lines()
                    .find(|l| l.contains("MOONSHOT-PROXY {"))
                    .expect("missing MOONSHOT-PROXY line");
                let json_str = &line[line.find('{').unwrap()..];
                let get_u64 = |key: &str| -> u64 {
                    let pat = format!("\"{key}\":");
                    let start = json_str.find(&pat).unwrap() + pat.len();
                    let rest = &json_str[start..];
                    let end = rest.find([',', '}']).unwrap();
                    rest[..end].trim().parse::<u64>().unwrap()
                };
                let ob = get_u64("out_bytes") as f64;
                let ins = get_u64("instructions") as f64 / 1e6;
                let dep = get_u64("dep_past_l2_loads") as f64 / 1e3;
                let dram_lines =
                    get_u64("dram_read_lines") + get_u64("dram_write_lines") + get_u64("dram_drain_lines");
                let dram_b = (dram_lines as f64 * 64.0) / ob;
                let cyc_b = get_u64("cycles") as f64 / ob;
                res_r.lock().unwrap().insert(
                    (su.clone(), llc, arm.clone()),
                    CgMetrics {
                        instrs_m: ins,
                        dep_l3_k: dep,
                        dram_b_per_b: dram_b,
                        cyc_per_b: cyc_b,
                    },
                );
            });
        }
    });
    results.into_inner().unwrap()
}

fn measure_workers_slice<K: InverseBwt + Default>(
    blks: &[&CorpusBlk],
    workers: usize,
) -> (usize, usize, usize, f64, f64) {
    let k = Kernels::detect();
    let idx = AtomicUsize::new(0);
    let max_ret = AtomicUsize::new(0);
    let max_pk = AtomicUsize::new(0);
    let sum_pk = AtomicUsize::new(0);
    let cpu_ns = AtomicUsize::new(0);
    let t0 = Instant::now();
    std::thread::scope(|sc| {
        let idx_r = &idx;
        let mr_r = &max_ret;
        let mp_r = &max_pk;
        let sp_r = &sum_pk;
        let cpu_r = &cpu_ns;
        for _ in 0..workers {
            sc.spawn(move || {
                let mut out = Vec::with_capacity(900_064);
                tls_reset();
                let mut kern = K::default();
                let mut my = 0usize;
                loop {
                    let i = idx_r.fetch_add(1, Ordering::Relaxed);
                    if i >= blks.len() {
                        break;
                    }
                    let ts = Instant::now();
                    invert_any(&mut kern, blks[i], &mut out, k);
                    std::hint::black_box(out.as_ptr());
                    my += ts.elapsed().as_nanos() as usize;
                }
                let (ret, pk) = tls_stats();
                mr_r.fetch_max(ret, Ordering::Relaxed);
                mp_r.fetch_max(pk, Ordering::Relaxed);
                sp_r.fetch_add(pk, Ordering::Relaxed);
                cpu_r.fetch_add(my, Ordering::Relaxed);
                drop(kern);
            });
        }
    });
    (
        max_ret.load(Ordering::Relaxed),
        max_pk.load(Ordering::Relaxed),
        sum_pk.load(Ordering::Relaxed),
        t0.elapsed().as_secs_f64() * 1e3,
        cpu_ns.load(Ordering::Relaxed) as f64 / 1e6,
    )
}

fn time_bzreader<K: InverseBwt + Send + 'static>(bz2_bytes: &Arc<[u8]>, workers: usize) -> f64 {
    let k = Kernels::detect();
    let t0 = Instant::now();
    let mut dec = BzReader::parallel_workers_with::<K>(
        std::io::Cursor::new(Arc::clone(bz2_bytes)),
        workers,
        4 * workers,
        k,
    );
    let mut buf = vec![0u8; 256 * 1024];
    let mut tot = 0usize;
    loop {
        let got = dec.read(&mut buf).unwrap();
        if got == 0 {
            break;
        }
        tot += got;
    }
    assert_eq!(tot, 209_700_196);
    t0.elapsed().as_secs_f64() * 1e3
}

fn run_adversarial_g51(all431: &[CorpusBlk], bz2_bytes: &Arc<[u8]>) {
    let k = Kernels::detect();
    let mut k_no = UnifiedEdgesMuxKernel::<16, false>::default();
    let mut k_aht = UnifiedEdgesMuxKernel::<16, true>::default();
    let mut o_no = Vec::new();
    let mut o_aht = Vec::new();

    // 1. Both `ra > rb` (`1^1 0^299999`) AND `ra < rb` (`1^299999 0^1`) Ostrowski spirals!
    {
        let mut recs_gt = vec![(1u16 << 8) | 1];
        let mut rem = 299_999usize;
        while rem > 0 {
            let tk = rem.min(127);
            recs_gt.push((0u16 << 8) | (tk as u16));
            rem -= tk;
        }
        assert!(k_no.try_run::<900_000, 900_000>(&recs_gt, 300_000, 0, 60_000, &mut o_no));
        assert!(k_aht.try_run::<256_000, 16_384>(&recs_gt, 300_000, 0, 60_000, &mut o_aht));
        assert!(k_no.r4_loop_steps > 250_000 && k_aht.r4_loop_steps <= 8);
        assert_eq!(o_no, o_aht, "ra > rb spiral mismatch");

        let mut recs_lt = Vec::new();
        rem = 299_999;
        while rem > 0 {
            let tk = rem.min(127);
            recs_lt.push((1u16 << 8) | (tk as u16));
            rem -= tk;
        }
        recs_lt.push((0u16 << 8) | 1);
        assert!(k_no.try_run::<900_000, 900_000>(&recs_lt, 300_000, 299_999, 60_000, &mut o_no));
        assert!(k_aht.try_run::<256_000, 16_384>(&recs_lt, 300_000, 299_999, 60_000, &mut o_aht));
        assert!(k_no.r4_loop_steps > 250_000 && k_aht.r4_loop_steps <= 8);
        assert_eq!(o_no, o_aht, "ra < rb spiral mismatch");
    }

    // 2. Poisoned-`edges` (`EdgesMux`) + forced `cap_override = 16` `ovf_chunks` spill test!
    {
        let s9 = load_all("/tmp/sample9_admitted.bz2");
        let mut ref_k = OpenCells::default();
        let mut o_ref = Vec::with_capacity(900_064);
        for blk in &s9 {
            if let CorpusBlk::RunsBlk { runs, n, p, .. } = blk {
                assert!(ref_k.invert_runs(runs, *n, *p, &mut o_ref, k));
                k_aht.poison_tapes_for_test();
                assert!(k_aht.try_run::<256_000, 16_384>(runs.records(), *n, *p, 16, &mut o_aht));
                assert!(k_aht.last_ovf_chunks_used > 0);
                assert_eq!(o_ref, o_aht, "poisoned EdgesMux + ovf_chunks mismatch");
            }
        }
    }

    // 3. Sticky `m = u32::MAX` early-abort peak heap (`<= 32 KiB`) + pure column-block idle overhead (`<= 2 KiB`)!
    {
        let mut recs = Vec::new();
        let mut pos = 0usize;
        let mut b = 0u8;
        while pos < 900_000 {
            let w = if (recs.len() % 2) == 0 { 16 } else { 1 }.min(900_000 - pos);
            recs.push(((b as u16) << 8) | (w as u16));
            pos += w;
            b = b.wrapping_add(1);
        }
        let mut out_syn = vec![0u8; 900_032];
        tls_reset();
        let mut fresh_k = UnifiedEdgesMuxKernel::<16, true>::default();
        let ok = fresh_k.try_run::<256_000, 16_384>(&recs, 900_000, 0, 60_000, &mut out_syn);
        let (_, abort_pk) = tls_stats();
        assert!(!ok && abort_pk <= 32 * 1024, "early-abort heap too large: {abort_pk}");

        let col_blk = all431
            .iter()
            .find(|b| matches!(b, CorpusBlk::ColBlk { .. }))
            .unwrap();
        let mut out_col = Vec::with_capacity(900_064);
        tls_reset();
        let mut col_bwt = AhtEdgesMuxOpenCells::default();
        invert_any(&mut col_bwt, col_blk, &mut out_col, k);
        let (_, col_pk) = tls_stats();
        assert!(col_pk <= 2048, "column-only idle heap too large: {col_pk}");
    }

    // 4. All 431 blocks + full BzReader parallel stream byte-exactness at W=8 and W=10!
    {
        let mut ref_k = OpenCells::default();
        let mut c_same = SameGateEdgesMuxOpenCells::default();
        let mut c_16 = AhtEdgesMuxOpenCells::default();
        let mut c_14 = AhtEdgesMux14OpenCells::default();
        let mut o0 = Vec::with_capacity(900_064);
        let mut o1 = Vec::with_capacity(900_064);
        let mut o2 = Vec::with_capacity(900_064);
        let mut o3 = Vec::with_capacity(900_064);
        for (bi, blk) in all431.iter().enumerate() {
            invert_any(&mut ref_k, blk, &mut o0, k);
            invert_any(&mut c_same, blk, &mut o1, k);
            invert_any(&mut c_16, blk, &mut o2, k);
            invert_any(&mut c_14, blk, &mut o3, k);
            assert_eq!(o0, o1, "SameGate mismatch on blk {bi}");
            assert_eq!(o0, o2, "AhtEdgesMux16 mismatch on blk {bi}");
            assert_eq!(o0, o3, "AhtEdgesMux14 mismatch on blk {bi}");
        }

        let mut want_stream = Vec::with_capacity(209_700_196);
        BzReader::parallel_workers_with::<OpenCells>(
            std::io::Cursor::new(Arc::clone(bz2_bytes)),
            8,
            32,
            k,
        )
        .read_to_end(&mut want_stream)
        .unwrap();
        assert_eq!(want_stream.len(), 209_700_196);
        for &w in &[8usize, 10] {
            let mut got16 = Vec::with_capacity(209_700_196);
            BzReader::parallel_workers_with::<AhtEdgesMuxOpenCells>(
                std::io::Cursor::new(Arc::clone(bz2_bytes)),
                w,
                4 * w,
                k,
            )
            .read_to_end(&mut got16)
            .unwrap();
            assert_eq!(want_stream, got16, "BzReader AhtEdgesMux16 mismatch at W={w}");

            let mut got14 = Vec::with_capacity(209_700_196);
            BzReader::parallel_workers_with::<AhtEdgesMux14OpenCells>(
                std::io::Cursor::new(Arc::clone(bz2_bytes)),
                w,
                4 * w,
                k,
            )
            .read_to_end(&mut got14)
            .unwrap();
            assert_eq!(want_stream, got14, "BzReader AhtEdgesMux14 mismatch at W={w}");
        }
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map_or("all", String::as_str);
    let path = args.get(2).map_or("/tmp/wikidata.bz2", String::as_str);

    if let Some(arm) = mode.strip_prefix("cg_s9:") {
        let all = load_all(path);
        match arm {
            "058" => run_cg_suite::<OpenCells>(&all),
            "same16" => run_cg_suite::<SameGateEdgesMuxOpenCells>(&all),
            "mux16" => run_cg_suite::<AhtEdgesMuxOpenCells>(&all),
            "mux14" => run_cg_suite::<AhtEdgesMux14OpenCells>(&all),
            _ => panic!("unknown {arm}"),
        }
        return;
    }
    if let Some(arm) = mode.strip_prefix("cg_dense:") {
        let picks = load_dense_picks(path);
        match arm {
            "058" => run_cg_suite::<OpenCells>(&picks),
            "same16" => run_cg_suite::<SameGateEdgesMuxOpenCells>(&picks),
            "mux16" => run_cg_suite::<AhtEdgesMuxOpenCells>(&picks),
            "mux14" => run_cg_suite::<AhtEdgesMux14OpenCells>(&picks),
            _ => panic!("unknown {arm}"),
        }
        return;
    }

    let bz2_bytes: Arc<[u8]> = std::fs::read(path).unwrap().into();
    let all431 = load_all(path);
    assert_eq!(all431.len(), 431);
    let refs431: Vec<&CorpusBlk> = all431.iter().collect();
    let blks228: Vec<&CorpusBlk> = all431
        .iter()
        .filter(|b| matches!(b, CorpusBlk::RunsBlk { admitted_058: true, .. }))
        .collect();
    assert_eq!(blks228.len(), 228);

    println!("Running G5.1 Ostrowski ra>rb & ra<rb spirals, poisoned-EdgesMux ovf, sticky m=u32::MAX abort, 431-block & BzReader W=8,10 checks...");
    run_adversarial_g51(&all431, &bz2_bytes);
    let g51_pass = true;
    println!("  G5.1 Exactness, Spiral Contraction, Disjoint Lifetimes & Fallback Safety: PASS");

    let names = [
        "0: Master-0.5.8 (OpenCells, 228/431)",
        "1: C19-SameGate-Mux16 (Gate 0.15, 228/431)",
        "2: C19-AhtEdgesMux16 (StructGate, Primary)",
        "3: C19-AhtEdgesMux14 (StructGate, L=14)",
    ];

    let h228_0 = measure_workers_slice::<OpenCells>(&blks228, 1);
    let h228_1 = measure_workers_slice::<SameGateEdgesMuxOpenCells>(&blks228, 1);
    let h228_2 = measure_workers_slice::<AhtEdgesMuxOpenCells>(&blks228, 1);
    let h228_3 = measure_workers_slice::<AhtEdgesMux14OpenCells>(&blks228, 1);
    let h228 = [h228_0, h228_1, h228_2, h228_3];

    let h431_0 = measure_workers_slice::<OpenCells>(&refs431, 10);
    let h431_1 = measure_workers_slice::<SameGateEdgesMuxOpenCells>(&refs431, 10);
    let h431_2 = measure_workers_slice::<AhtEdgesMuxOpenCells>(&refs431, 10);
    let h431_3 = measure_workers_slice::<AhtEdgesMux14OpenCells>(&refs431, 10);
    let h431 = [h431_0, h431_1, h431_2, h431_3];

    println!("\n=== G5.2 Per-Worker Heap Footprint (`228`-Block Subset & `431`-Block 10-Worker Whole Corpus) ===");
    for i in 0..4 {
        println!(
            "  {:<42} | 228 Peak = {:7.1} KiB ({:5.3}x) | 431 10W MaxPeak = {:7.1} KiB ({:5.3}x) | 10W Sum = {:6.2} MiB ({:5.3}x)",
            names[i],
            h228[i].1 as f64 / 1024.0,
            h228[i].1 as f64 / h228_0.1 as f64,
            h431[i].1 as f64 / 1024.0,
            h431[i].1 as f64 / h431_0.1 as f64,
            h431[i].2 as f64 / (1024.0 * 1024.0),
            h431[i].2 as f64 / h431_0.2 as f64,
        );
    }
    let g52_pass = (h228_2.1 as f64 / h228_0.1 as f64) <= 0.310
        && (h431_2.1 as f64 / h431_0.1 as f64) <= 0.165
        && (h431_2.2 as f64 / h431_0.2 as f64) <= 0.220;
    println!("  G5.2 Per-Worker Heap & Working-Set Gate: {}", if g52_pass { "PASS" } else { "FAIL" });

    println!("\n=== G5.3 Dual-Suite `chaingrind` Shared-L3 & DRAM Traffic (`W=8` 2560 KiB & `W=10` 2048 KiB) ===");
    let cg = spawn_cg_matrix(&args[0]);
    let arm_keys = ["058", "same16", "mux16", "mux14"];
    for su in ["s9", "dense"] {
        for llc in [2560usize, 2048] {
            let b = cg[&(su.to_string(), llc, "058".to_string())];
            println!("  --- Suite `{su}` at LLC = {llc} KiB ---");
            for (ai, &ak) in arm_keys.iter().enumerate() {
                let m = cg[&(su.to_string(), llc, ak.to_string())];
                println!(
                    "    {:<42} | ins={:6.2}M ({:5.3}x) | depL3={:6.1}k | DRAM={:6.3} B/B ({:5.3}x) | cyc/B={:6.3} ({:5.3}x)",
                    names[ai],
                    m.instrs_m,
                    m.instrs_m / b.instrs_m,
                    m.dep_l3_k,
                    m.dram_b_per_b,
                    m.dram_b_per_b / b.dram_b_per_b,
                    m.cyc_per_b,
                    m.cyc_per_b / b.cyc_per_b,
                );
            }
        }
    }
    let s9_8_b = cg[&("s9".to_string(), 2560, "058".to_string())];
    let s9_8_c = cg[&("s9".to_string(), 2560, "mux16".to_string())];
    let s9_10_b = cg[&("s9".to_string(), 2048, "058".to_string())];
    let s9_10_c = cg[&("s9".to_string(), 2048, "mux16".to_string())];
    let dn_8_b = cg[&("dense".to_string(), 2560, "058".to_string())];
    let dn_8_c = cg[&("dense".to_string(), 2560, "mux16".to_string())];
    let dn_10_b = cg[&("dense".to_string(), 2048, "058".to_string())];
    let dn_10_c = cg[&("dense".to_string(), 2048, "mux16".to_string())];

    let g53_s9_pass = (s9_8_c.instrs_m / s9_8_b.instrs_m) <= 0.925
        && (s9_8_c.dram_b_per_b / s9_8_b.dram_b_per_b) <= 0.200
        && (s9_10_c.dram_b_per_b / s9_10_b.dram_b_per_b) <= 0.085
        && (s9_8_c.cyc_per_b / s9_8_b.cyc_per_b) <= 0.925
        && (s9_10_c.cyc_per_b / s9_10_b.cyc_per_b) <= 0.860;
    let g53_dn_pass = (dn_8_c.dram_b_per_b / dn_8_b.dram_b_per_b) <= 0.030
        && (dn_10_c.dram_b_per_b / dn_10_b.dram_b_per_b) <= 0.080
        && (dn_8_c.cyc_per_b / dn_8_b.cyc_per_b) <= 0.500
        && (dn_10_c.cyc_per_b / dn_10_b.cyc_per_b) <= 0.500;
    let g53_pass = g53_s9_pass && g53_dn_pass;
    println!(
        "  G5.3 Dual-Suite Shared-L3 & DRAM Gate: {} (s9={}, dense={})",
        if g53_pass { "PASS" } else { "FAIL" },
        if g53_s9_pass { "PASS" } else { "FAIL" },
        if g53_dn_pass { "PASS" } else { "FAIL" }
    );

    let rounds = 15usize;
    let mut g54_pass = true;
    for &w in &[1usize, 2, 8, 10] {
        let mut wall = vec![vec![0.0; rounds]; 4];
        let mut cpu = vec![vec![0.0; rounds]; 4];
        for r in 0..rounds + 1 {
            for step in 0..4 {
                let arm = (step + r) % 4;
                let (_, _, _, wm, cm) = match arm {
                    0 => measure_workers_slice::<OpenCells>(&blks228, w),
                    1 => measure_workers_slice::<SameGateEdgesMuxOpenCells>(&blks228, w),
                    2 => measure_workers_slice::<AhtEdgesMuxOpenCells>(&blks228, w),
                    3 => measure_workers_slice::<AhtEdgesMux14OpenCells>(&blks228, w),
                    _ => unreachable!(),
                };
                if r >= 1 {
                    wall[arm][r - 1] = wm;
                    cpu[arm][r - 1] = cm;
                }
            }
        }
        let bw = median(&mut wall[0].clone());
        let bc = median(&mut cpu[0].clone());
        println!("\n=== G5.4a `228`-Block Subset Kernel Stage (`lto=fat`) at W = {w} ({rounds} rounds) ===");
        for arm in 0..4 {
            let mw = median(&mut wall[arm]);
            let mc = median(&mut cpu[arm]);
            println!(
                "  {:<42} | wall = {:6.1} ms ({:5.3}x 0.5.8) | sum_cpu = {:7.1} ms ({:5.3}x 0.5.8)",
                names[arm], mw, mw / bw, mc, mc / bc
            );
            if arm == 2 {
                let bar = if w >= 8 { 0.920 } else { 0.950 };
                if mw / bw > bar || mc / bc > bar {
                    g54_pass = false;
                }
            }
        }
    }

    for &w in &[1usize, 2, 8, 10] {
        let mut wall = vec![vec![0.0; rounds]; 4];
        let mut cpu = vec![vec![0.0; rounds]; 4];
        for r in 0..rounds + 1 {
            for step in 0..4 {
                let arm = (step + r) % 4;
                let (_, _, _, wm, cm) = match arm {
                    0 => measure_workers_slice::<OpenCells>(&refs431, w),
                    1 => measure_workers_slice::<SameGateEdgesMuxOpenCells>(&refs431, w),
                    2 => measure_workers_slice::<AhtEdgesMuxOpenCells>(&refs431, w),
                    3 => measure_workers_slice::<AhtEdgesMux14OpenCells>(&refs431, w),
                    _ => unreachable!(),
                };
                if r >= 1 {
                    wall[arm][r - 1] = wm;
                    cpu[arm][r - 1] = cm;
                }
            }
        }
        let bw = median(&mut wall[0].clone());
        let bc = median(&mut cpu[0].clone());
        println!("\n=== G5.4b Whole-Corpus (`431` Blocks) Kernel Stage (`lto=fat`) at W = {w} ({rounds} rounds) ===");
        for arm in 0..4 {
            let mw = median(&mut wall[arm]);
            let mc = median(&mut cpu[arm]);
            println!(
                "  {:<42} | wall = {:6.1} ms ({:5.3}x 0.5.8) | sum_cpu = {:7.1} ms ({:5.3}x 0.5.8)",
                names[arm], mw, mw / bw, mc, mc / bc
            );
            if arm == 2 {
                let bar = if w >= 8 { 0.890 } else { 0.930 };
                if mw / bw > bar || mc / bc > bar {
                    g54_pass = false;
                }
            }
        }
    }

    for &w in &[8usize, 10] {
        let mut bzr = vec![vec![0.0; 11]; 4];
        for r in 0..12 {
            for step in 0..4 {
                let arm = (step + r) % 4;
                let wm = match arm {
                    0 => time_bzreader::<OpenCells>(&bz2_bytes, w),
                    1 => time_bzreader::<SameGateEdgesMuxOpenCells>(&bz2_bytes, w),
                    2 => time_bzreader::<AhtEdgesMuxOpenCells>(&bz2_bytes, w),
                    3 => time_bzreader::<AhtEdgesMux14OpenCells>(&bz2_bytes, w),
                    _ => unreachable!(),
                };
                if r >= 1 {
                    bzr[arm][r - 1] = wm;
                }
            }
        }
        let b0 = median(&mut bzr[0].clone());
        println!("\n=== G5.4c End-to-End `BzReader::parallel_workers_with` (`431` Blocks, `4*W` window) at W = {w} (11 rounds) ===");
        for arm in 0..4 {
            let mw = median(&mut bzr[arm]);
            println!("  {:<42} | wall = {:6.1} ms ({:5.3}x 0.5.8)", names[arm], mw, mw / b0);
            if arm == 2 && mw / b0 > 0.925 {
                g54_pass = false;
            }
        }
    }
    println!("\n  G5.4 Multi-Worker Native Speed Gate: {}", if g54_pass { "PASS" } else { "FAIL" });

    let overall = g51_pass && g52_pass && g53_pass && g54_pass;
    println!(
        "\n======================================================================\nOVERALL CARBON CYCLE 19 STRICT GATE G5 VERDICT: {} (G5.1={}, G5.2={}, G5.3={}, G5.4={})\n======================================================================",
        if overall { "PASS (4/4)" } else { "FAIL" },
        if g51_pass { "PASS" } else { "FAIL" },
        if g52_pass { "PASS" } else { "FAIL" },
        if g53_pass { "PASS" } else { "FAIL" },
        if g54_pass { "PASS" } else { "FAIL" },
    );
    if !overall {
        std::process::exit(1);
    }
}
