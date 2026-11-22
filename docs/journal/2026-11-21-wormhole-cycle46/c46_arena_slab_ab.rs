use std::time::Instant;
use std::process::Command;
use std::collections::BTreeMap;

// ---- Cycle 46: Contiguous Slab / Arena vs Separate 960 KiB Vecs (VMA / mmap_lock / THP Study) ----
// Tests cold-start allocation + full-write + read + drop of N = 16 and N = 40 x 960 KiB output buffers
// across 10 worker threads in fresh processes!
// Compares:
//   1. Separate960K: 16/40 individual Vec::with_capacity(960_000) (>128 KiB glibc M_MMAP_THRESHOLD -> 16/40 separate mmap/munmap VMAs!)
//   2. SeparateDoubling: master's 900K -> 1.8M doubling per buffer!
//   3. SingleSlabSafe: ONE contiguous Vec<u8> of N * 960 KiB (#![forbid(unsafe_code)], 1 mmap/1 munmap/1 VMA!)
//   4. SingleSlabTHP: ONE contiguous 2 MiB-aligned slab + madvise(MADV_HUGEPAGE) (benchmark-only diagnostic to measure 2 MiB THP floor!)

#[repr(C)] #[derive(Default, Clone, Copy)] struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)] #[derive(Default, Clone, Copy)] struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval, ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64, ru_inblock: i64, ru_oublock: i64,
    ru_msgsnd: i64, ru_msgrcv: i64, ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
    fn madvise(addr: *mut u8, len: usize, advice: i32) -> i32;
}
const MADV_HUGEPAGE: i32 = 14;
fn rusage() -> Rusage { let mut r = Rusage::default(); unsafe { getrusage(0, &mut r); } r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

mod safe_slab {
    #![forbid(unsafe_code)]
    pub const SLOT_BYTES: usize = 960_000;

    /// Simulate 10 workers concurrently faulting, writing (3 passes), and releasing `n_bufs` output buffers
    pub fn run_separate_doubling(n_bufs: usize, threads: usize) -> u64 {
        let mut bufs: Vec<Vec<u8>> = (0..n_bufs).map(|_| Vec::new()).collect();
        let per = (n_bufs + threads - 1) / threads;
        let sum = std::sync::atomic::AtomicU64::new(0);
        std::thread::scope(|s| {
            for chunk in bufs.chunks_mut(per) {
                let sref = &sum;
                s.spawn(move || {
                    let mut acc = 0u64;
                    for b in chunk {
                        b.reserve(899_995);
                        b.resize(899_995, 0x41);
                        b.extend_from_slice(&[0x42; 64]); // triggers 900K -> 1.8M doubling!
                        for pass in 0..2 {
                            for i in (0..900_000).step_by(64) { b[i] = (i as u8).wrapping_add(pass); }
                            acc = acc.wrapping_add(b[899_900] as u64);
                        }
                    }
                    sref.fetch_add(acc, std::sync::atomic::Ordering::Relaxed);
                });
            }
        });
        drop(bufs);
        sum.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn run_separate_exact(n_bufs: usize, threads: usize) -> u64 {
        let mut bufs: Vec<Vec<u8>> = (0..n_bufs).map(|_| Vec::new()).collect();
        let per = (n_bufs + threads - 1) / threads;
        let sum = std::sync::atomic::AtomicU64::new(0);
        std::thread::scope(|s| {
            for chunk in bufs.chunks_mut(per) {
                let sref = &sum;
                s.spawn(move || {
                    let mut acc = 0u64;
                    for b in chunk {
                        b.reserve_exact(SLOT_BYTES);
                        b.resize(900_059, 0x41);
                        for pass in 0..2 {
                            for i in (0..900_000).step_by(64) { b[i] = (i as u8).wrapping_add(pass); }
                            acc = acc.wrapping_add(b[899_900] as u64);
                        }
                    }
                    sref.fetch_add(acc, std::sync::atomic::Ordering::Relaxed);
                });
            }
        });
        drop(bufs);
        sum.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn run_single_slab_safe(n_bufs: usize, threads: usize) -> u64 {
        let mut slab = vec![0u8; n_bufs * SLOT_BYTES];
        let per_bytes = ((n_bufs + threads - 1) / threads) * SLOT_BYTES;
        let sum = std::sync::atomic::AtomicU64::new(0);
        std::thread::scope(|s| {
            for chunk in slab.chunks_mut(per_bytes) {
                let sref = &sum;
                s.spawn(move || {
                    let mut acc = 0u64;
                    for slot in chunk.chunks_exact_mut(SLOT_BYTES) {
                        slot[..900_059].fill(0x41);
                        for pass in 0..2 {
                            for i in (0..900_000).step_by(64) { slot[i] = (i as u8).wrapping_add(pass); }
                            acc = acc.wrapping_add(slot[899_900] as u64);
                        }
                    }
                    sref.fetch_add(acc, std::sync::atomic::Ordering::Relaxed);
                });
            }
        });
        drop(slab);
        sum.load(std::sync::atomic::Ordering::Relaxed)
    }
}

fn run_single_slab_thp(n_bufs: usize, threads: usize) -> u64 {
    let slot_bytes = safe_slab::SLOT_BYTES;
    let total = n_bufs * slot_bytes;
    // Allocate without zero-filling before madvise by using Vec::with_capacity!
    let mut slab = Vec::<u8>::with_capacity(total + (2 << 20));
    unsafe {
        let ptr = slab.as_mut_ptr();
        let aligned = ((ptr as usize + (2 << 20) - 1) & !((2 << 20) - 1)) as *mut u8;
        let off = aligned.offset_from(ptr) as usize;
        let _ = madvise(aligned, total, MADV_HUGEPAGE);
        slab.set_len(off + total);
        let region = &mut slab[off..off + total];
        let per_bytes = ((n_bufs + threads - 1) / threads) * slot_bytes;
        let sum = std::sync::atomic::AtomicU64::new(0);
        std::thread::scope(|s| {
            for chunk in region.chunks_mut(per_bytes) {
                let sref = &sum;
                s.spawn(move || {
                    let mut acc = 0u64;
                    for slot in chunk.chunks_exact_mut(slot_bytes) {
                        slot[..900_059].fill(0x41);
                        for pass in 0..2 {
                            for i in (0..900_000).step_by(64) { slot[i] = (i as u8).wrapping_add(pass); }
                            acc = acc.wrapping_add(slot[899_900] as u64);
                        }
                    }
                    sref.fetch_add(acc, std::sync::atomic::Ordering::Relaxed);
                });
            }
        });
        sum.load(std::sync::atomic::Ordering::Relaxed)
    }
}

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = 2.201;
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let arm = &args[2];
        let n_bufs: usize = args[3].parse().unwrap();
        let threads: usize = args[4].parse().unwrap();
        let r0 = rusage(); let t0 = Instant::now();
        let acc = match arm.as_str() {
            "Sep_Double" => safe_slab::run_separate_doubling(n_bufs, threads),
            "Sep_Exact" => safe_slab::run_separate_exact(n_bufs, threads),
            "Slab_Safe" => safe_slab::run_single_slab_safe(n_bufs, threads),
            "Slab_THP" => run_single_slab_thp(n_bufs, threads),
            _ => panic!(),
        };
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {:.2} {acc}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0);
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let arms = ["Sep_Double", "Sep_Exact", "Slab_Safe", "Slab_THP"];
    for &(n_bufs, threads) in &[(16usize, 4usize), (16, 10), (40, 10)] {
        println!("\n=== 12-ROUND FRESH-PROCESS SLAB/THP STUDY (N={n_bufs} x 960 KiB, T={threads} threads) ===");
        let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        for r in 0..12 {
            for i in 0..arms.len() {
                let arm = arms[(r + i) % arms.len()];
                let out = Command::new(&exe).args(["--child", arm, &n_bufs.to_string(), &threads.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                walls.entry(arm).or_default().push(p[0].parse().unwrap());
                syss.entry(arm).or_default().push(p[2].parse().unwrap());
                flts.entry(arm).or_default().push(p[3].parse().unwrap());
                rsss.entry(arm).or_default().push(p[4].parse().unwrap());
            }
        }
        let base = &walls["Sep_Double"];
        println!("{:<12} {:>8} {:>24} {:>7} {:>7} {:>7}", "Arm", "med_ms", "vs Sep_Double [95%]", "sys_ms", "minflt", "rss_MB");
        for &arm in &arms {
            let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = flts[arm].clone(); fv.sort();
            let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&base.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
            println!("{:<12} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>7.2} {:>7} {:>7.2}", arm, wv[6], rat, lo, hi, sv[6], fv[6], rv[6]);
        }
    }
}
