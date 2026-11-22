use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, Source};

// ---- Safe-Rust Candidate Parallel Decoder (Cycle 32: Allocator / Page-Fault Attack) ----
mod safe_c32 {
    #![forbid(unsafe_code)]
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub struct AllocCfg {
        pub out_init_cap: usize,   // 0 = master (triggers 900K->1.8M doubling), 960_000 = A1 exact fit
        pub syms_init_cap: usize,  // 0 = master (power-of-2 reallocs), 262_144 = A3 exact fit
        pub chunk_size: usize,     // 4<<20 = master, 1<<20 = A2
        pub recycle_chunks: bool,  // false = master (alloc+munmap per chunk), true = A2
        pub slots: usize,
    }

    struct State {
        ready: BTreeMap<u64, Result<Vec<u8>, Error>>,
        issued: u64,
        taken: u64,
        end: Option<u64>,
        cancelled: bool,
        free_out: Vec<Vec<u8>>,
        free_syms: Vec<Vec<u16>>,
    }

    struct Shared {
        state: Mutex<State>,
        cv: Condvar,
        slots: u64,
    }

    struct PooledChannelSource<'a> {
        rx: mpsc::Receiver<Vec<u8>>,
        tx_free: Option<mpsc::Sender<Vec<u8>>>,
        buf: Vec<u8>,
        base: u64,
        released: u64,
        eof: bool,
        shared: &'a Shared,
    }

    impl Source for PooledChannelSource<'_> {
        fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
            const COMPACT: usize = 4 << 20;
            while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
                match self.rx.recv() {
                    Ok(mut chunk) => {
                        let drop_n = (self.released - self.base) as usize;
                        if drop_n == self.buf.len() && self.tx_free.is_some() {
                            // Zero-copy swap when previous buffer fully consumed!
                            let mut old = std::mem::replace(&mut self.buf, chunk);
                            self.base = self.released;
                            old.clear();
                            if let Some(tx) = &self.tx_free {
                                let _ = tx.send(old);
                            }
                        } else {
                            if drop_n >= COMPACT {
                                self.buf.drain(..drop_n);
                                self.base = self.released;
                            }
                            self.buf.extend_from_slice(&chunk);
                            if let Some(tx) = &self.tx_free {
                                chunk.clear();
                                let _ = tx.send(chunk);
                            }
                        }
                    }
                    Err(_) => self.eof = true,
                }
                if self.shared.state.lock().unwrap().cancelled {
                    return Err(ErrorKind::Cancelled);
                }
            }
            let i = (from - self.base) as usize;
            Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
        }

        fn release(&mut self, upto: u64) {
            self.released = self.released.max(upto);
        }
    }

    pub fn decode_c32_stream<R: Read + Send + 'static>(
        mut r: R,
        workers: usize,
        cfg: AllocCfg,
        k: Kernels,
        mut on_chunk: impl FnMut(&[u8]),
    ) -> Result<(usize, usize), Error> {
        let slots = cfg.slots.max(workers) as u64;
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                ready: BTreeMap::new(),
                issued: 0,
                taken: 0,
                end: None,
                cancelled: false,
                free_out: Vec::with_capacity(slots as usize),
                free_syms: Vec::with_capacity(slots as usize),
            }),
            cv: Condvar::new(),
            slots,
        });

        let (in_tx, in_rx) = mpsc::sync_channel::<Vec<u8>>(2);
        let (free_tx, free_rx) = if cfg.recycle_chunks {
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        let (job_tx, job_rx) = mpsc::channel::<(u64, u64, mibz2::block::Symbols)>();
        let job_rx = Arc::new(Mutex::new(job_rx));

        let feeder = std::thread::spawn(move || {
            let mut local_pool: Vec<Vec<u8>> = Vec::new();
            if cfg.recycle_chunks {
                for _ in 0..3 {
                    local_pool.push(Vec::with_capacity(cfg.chunk_size));
                }
            }
            loop {
                let mut buf = if let Some(rx) = &free_rx {
                    while let Ok(b) = rx.try_recv() {
                        local_pool.push(b);
                    }
                    local_pool.pop().unwrap_or_else(|| Vec::with_capacity(cfg.chunk_size))
                } else {
                    vec![0u8; cfg.chunk_size]
                };
                if cfg.recycle_chunks {
                    buf.resize(cfg.chunk_size, 0);
                }
                let mut n = 0;
                while n < buf.len() {
                    match r.read(&mut buf[n..]) {
                        Ok(0) => break,
                        Ok(got) => n += got,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => return,
                    }
                }
                if n == 0 { break; }
                buf.truncate(n);
                if in_tx.send(buf).is_err() { return; }
            }
        });

        let sh_s = shared.clone();
        let scanner_h = std::thread::spawn(move || {
            let src = PooledChannelSource {
                rx: in_rx,
                tx_free: free_tx,
                buf: Vec::new(),
                base: 0,
                released: 0,
                eof: false,
                shared: &sh_s,
            };
            let mut sc = Scanner::new(src);
            loop {
                let syms = {
                    let mut st = sh_s.state.lock().unwrap();
                    while !st.cancelled && st.issued - st.taken >= sh_s.slots {
                        st = sh_s.cv.wait(st).unwrap();
                    }
                    if st.cancelled {
                        st.end = Some(st.issued);
                        drop(st);
                        sh_s.cv.notify_all();
                        return;
                    }
                    st.free_syms.pop().unwrap_or_else(|| {
                        if cfg.syms_init_cap > 0 {
                            Vec::with_capacity(cfg.syms_init_cap)
                        } else {
                            Vec::new()
                        }
                    })
                };
                match sc.next(syms) {
                    Ok(Event::Block { bit, symbols, .. }) => {
                        let mut st = sh_s.state.lock().unwrap();
                        let place = st.issued;
                        st.issued += 1;
                        drop(st);
                        if job_tx.send((place, bit, symbols)).is_err() {
                            let mut st = sh_s.state.lock().unwrap();
                            st.end = Some(st.issued);
                            drop(st);
                            sh_s.cv.notify_all();
                            return;
                        }
                    }
                    Ok(Event::Done) => {
                        let mut st = sh_s.state.lock().unwrap();
                        st.end = Some(st.issued);
                        drop(st);
                        sh_s.cv.notify_all();
                        return;
                    }
                    Err(e) => {
                        let mut st = sh_s.state.lock().unwrap();
                        let at = st.issued;
                        st.ready.insert(at, Err(e));
                        st.issued += 1;
                        st.end = Some(at + 1);
                        drop(st);
                        sh_s.cv.notify_all();
                        return;
                    }
                }
            }
        });

        let mut worker_hs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let sh_w = shared.clone();
            let rx_w = job_rx.clone();
            worker_hs.push(std::thread::spawn(move || {
                let mut scratch = Scratch::<OpenCells>::default();
                loop {
                    let job = { rx_w.lock().unwrap().recv() };
                    let Ok((place, bit, symbols)) = job else { return; };
                    let mut out = {
                        let mut st = sh_w.state.lock().unwrap();
                        if st.cancelled { continue; }
                        st.free_out.pop().unwrap_or_default()
                    };
                    out.clear();
                    if cfg.out_init_cap > 0 && symbols.syms.len() > 2048 && out.capacity() < cfg.out_init_cap {
                        out.reserve_exact(cfg.out_init_cap);
                    }
                    let res = mibz2::decode_block(&symbols, k, &mut scratch, &mut out)
                        .map(|_| out)
                        .map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    let mut st = sh_w.state.lock().unwrap();
                    if !st.cancelled {
                        st.ready.insert(place, res);
                    }
                    if st.free_syms.len() < sh_w.slots as usize {
                        st.free_syms.push(symbols.syms);
                    }
                    drop(st);
                    sh_w.cv.notify_all();
                }
            }));
        }

        // Reader drain (exact same copy into 4 MiB caller buf for apples-to-apples W2-only test!)
        let mut read_buf = vec![0u8; 4 << 20];
        let mut total = 0usize;
        let mut reads = 0usize;
        let mut cur: Vec<u8> = Vec::new();
        let mut out_cap_max = 0usize;
        let mut out_doubled = 0usize;
        loop {
            let next_block = {
                let mut st = shared.state.lock().unwrap();
                let spent = std::mem::take(&mut cur);
                if spent.capacity() != 0 && st.free_out.len() < shared.slots as usize {
                    st.free_out.push(spent);
                }
                loop {
                    let taken = st.taken;
                    if let Some(res) = st.ready.remove(&taken) {
                        st.taken += 1;
                        shared.cv.notify_all();
                        break Some(res);
                    }
                    if st.end == Some(st.taken) {
                        break None;
                    }
                    st = shared.cv.wait(st).unwrap();
                }
            };
            match next_block {
                None => break,
                Some(Ok(v)) => {
                    if v.capacity() > out_cap_max { out_cap_max = v.capacity(); }
                    if v.capacity() > 1_200_000 { out_doubled += 1; }
                    let mut at = 0;
                    while at < v.len() {
                        let n = read_buf.len().min(v.len() - at);
                        read_buf[..n].copy_from_slice(&v[at..at + n]);
                        on_chunk(&read_buf[..n]);
                        at += n;
                        total += n;
                        reads += 1;
                    }
                    cur = v;
                }
                Some(Err(e)) => {
                    shared.state.lock().unwrap().cancelled = true;
                    shared.cv.notify_all();
                    let _ = feeder.join();
                    let _ = scanner_h.join();
                    for h in worker_hs { let _ = h.join(); }
                    return Err(e);
                }
            }
        }
        let _ = (reads, out_cap_max);
        let _ = feeder.join();
        let _ = scanner_h.join();
        for h in worker_hs { let _ = h.join(); }
        Ok((total, out_doubled))
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval,
    ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64,
    ru_inblock: i64, ru_oublock: i64, ru_msgsnd: i64, ru_msgrcv: i64,
    ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" { fn getrusage(who: i32, usage: *mut Rusage) -> i32; }
fn rusage() -> Rusage { let mut r = Rusage::default(); unsafe { getrusage(0, &mut r); } r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

#[inline]
fn hash_chunk(h: &mut u64, b: &[u8]) {
    for c in b.chunks_exact(8) {
        *h = h.wrapping_mul(0x100000001b3) ^ u64::from_le_bytes(c.try_into().unwrap());
    }
    for &x in b.chunks_exact(8).remainder() {
        *h = h.wrapping_mul(0x100000001b3) ^ x as u64;
    }
}

fn run_arm(arm: &str, data: Vec<u8>, w: usize) -> (f64, f64, f64, i64, f64, usize, u64, usize) {
    let k = Kernels::detect();
    let r0 = rusage();
    let t0 = Instant::now();
    let mut h: u64 = 0xcbf29ce484222325;
    let mut bytes = 0usize;
    let mut doubled = 0usize;
    match arm {
        "MasterA" | "MasterB" => {
            let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, 4 * w, k, true);
            let mut buf = vec![0u8; 4 << 20];
            loop {
                let n = rdr.read(&mut buf).unwrap();
                if n == 0 { break; }
                hash_chunk(&mut h, &buf[..n]);
                bytes += n;
            }
            drop(rdr);
        }
        "A1_Out960K" => {
            let cfg = safe_c32::AllocCfg {
                out_init_cap: 960_000, syms_init_cap: 0, chunk_size: 4 << 20, recycle_chunks: false, slots: 4 * w,
            };
            let (b, d) = safe_c32::decode_c32_stream(Cursor::new(data), w, cfg, k, |c| hash_chunk(&mut h, c)).unwrap();
            bytes = b; doubled = d;
        }
        "A12_FeedPool" => {
            let cfg = safe_c32::AllocCfg {
                out_init_cap: 960_000, syms_init_cap: 0, chunk_size: 1 << 20, recycle_chunks: true, slots: 4 * w,
            };
            let (b, d) = safe_c32::decode_c32_stream(Cursor::new(data), w, cfg, k, |c| hash_chunk(&mut h, c)).unwrap();
            bytes = b; doubled = d;
        }
        "A123_Syms262K" => {
            let cfg = safe_c32::AllocCfg {
                out_init_cap: 960_000, syms_init_cap: 262_144, chunk_size: 1 << 20, recycle_chunks: true, slots: 4 * w,
            };
            let (b, d) = safe_c32::decode_c32_stream(Cursor::new(data), w, cfg, k, |c| hash_chunk(&mut h, c)).unwrap();
            bytes = b; doubled = d;
        }
        "A1234_Slots2W" => {
            let cfg = safe_c32::AllocCfg {
                out_init_cap: 960_000, syms_init_cap: 262_144, chunk_size: 1 << 20, recycle_chunks: true, slots: (2 * w).max(w + 4),
            };
            let (b, d) = safe_c32::decode_c32_stream(Cursor::new(data), w, cfg, k, |c| hash_chunk(&mut h, c)).unwrap();
            bytes = b; doubled = d;
        }
        "A1234b_SlotsW6" => {
            let cfg = safe_c32::AllocCfg {
                out_init_cap: 960_000, syms_init_cap: 262_144, chunk_size: 1 << 20, recycle_chunks: true, slots: w + 6,
            };
            let (b, d) = safe_c32::decode_c32_stream(Cursor::new(data), w, cfg, k, |c| hash_chunk(&mut h, c)).unwrap();
            bytes = b; doubled = d;
        }
        _ => panic!("unknown arm {arm}"),
    }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0, bytes, h, doubled)
}

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = 2.201; // df=11 t_0.975
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let path = &args[2];
        let arm = &args[3];
        let w: usize = args[4].parse().unwrap();
        let data = fs::read(path).unwrap();
        let (wall, u, s, flt, rss, bytes, h, dbl) = run_arm(arm, data, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {rss:.2} {bytes} {h:016x} {dbl}");
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let path210 = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    let path60 = "/tmp/wormhole_r5/wikidata_60b.bz2";
    let path_het = "/tmp/wormhole_r5/hetero_60b.bz2";

    let arms = ["MasterA", "MasterB", "A1_Out960K", "A12_FeedPool", "A123_Syms262K", "A1234_Slots2W", "A1234b_SlotsW6"];

    // 1. Fresh-process cold-start ablation at W=4, 10, 16 on wikidata_431b_210m.bz2 (3 rounds median)
    println!("=== PART 1: FRESH-PROCESS COLD-START ABLATION (wikidata_431b_210m.bz2, 3-rd median) ===");
    for &w in &[4usize, 10, 16] {
        println!("\n--- W = {w} ---");
        println!("{:<16} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6}", "Arm", "wall_ms", "user_ms", "sys_ms", "minflt", "rss_MiB", "dbl");
        let mut ref_hash = String::new();
        for &arm in &arms {
            let mut rows = Vec::new();
            for _ in 0..3 {
                let out = Command::new(&exe).args(["--child", path210, arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                if ref_hash.is_empty() { ref_hash = p[6].to_string(); }
                assert_eq!(p[6], ref_hash, "HASH MISMATCH for {arm} at W={w}");
                rows.push((p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                           p[3].parse::<i64>().unwrap(), p[4].parse::<f64>().unwrap(), p[7].parse::<usize>().unwrap()));
            }
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let m = rows[1];
            println!("{:<16} {:>8.2} {:>8.2} {:>8.2} {:>8} {:>8.2} {:>6}", arm, m.0, m.1, m.2, m.3, m.4, m.5);
        }
    }

    // 2. 12-round rotated paired cold-start 4W A/B on wikidata_60b.bz2 & hetero_60b.bz2
    for (label, path) in [("wikidata_60b", path60), ("hetero_60b", path_het)] {
        println!("\n=== PART 2: 12-ROUND ROTATED FRESH-PROCESS 4W A/B ({label}) ===");
        let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ref_hash = String::new();
        for r in 0..12 {
            for k in 0..arms.len() {
                let arm = arms[(r + k) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                if ref_hash.is_empty() { ref_hash = p[6].to_string(); }
                assert_eq!(p[6], ref_hash, "HASH MISMATCH {label} {arm}");
                walls.entry(arm).or_default().push(p[0].parse().unwrap());
                syss.entry(arm).or_default().push(p[2].parse().unwrap());
                flts.entry(arm).or_default().push(p[3].parse().unwrap());
                rsss.entry(arm).or_default().push(p[4].parse().unwrap());
            }
        }
        let ma = &walls["MasterA"];
        println!("{:<16} {:>8} {:>24} {:>8} {:>8} {:>8}", "Arm", "med_ms", "ratio vs MasterA [95%]", "sys_ms", "minflt", "rss_MiB");
        for &arm in &arms {
            let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = flts[arm].clone(); fv.sort();
            let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let pairs: Vec<(f64, f64)> = ma.iter().copied().zip(walls[arm].iter().copied()).collect();
            let (rat, lo, hi) = ci95_ratio(&pairs);
            println!("{:<16} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>8.2} {:>8} {:>8.2}",
                arm, wv[6], rat, lo, hi, sv[6], fv[6], rv[6]);
        }
    }
}
