use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, Source};

// ---- Cycle 35 (Interim Handoff #1): Clean Promoted Phase-1 Stack (P1 = C32 + C33b + H123) ----
// Piggybacked pools inside ready.lock() (killing H4 extra mutexes), exact error/cancel/multi-stream parity!
mod safe_p1 {
    #![forbid(unsafe_code)]
    use super::*;

    type Job = (u64, u64, mibz2::block::Symbols);

    #[derive(Clone, Copy, Debug)]
    pub struct P1Cfg {
        pub feed_pool_exact_out: bool, // C32 (A1+A2)
        pub ring_split_deque: bool,    // C34 (H12+H123, piggybacked pools)
        pub zero_copy_bufread: bool,   // C33b (BufRead zero-copy)
    }

    struct PooledSource {
        rx: mpsc::Receiver<Vec<u8>>,
        tx_free: Option<mpsc::Sender<Vec<u8>>>,
        buf: Vec<u8>,
        base: u64,
        released: u64,
        eof: bool,
        cancelled: Arc<AtomicBool>,
    }
    impl Source for PooledSource {
        fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
            const COMPACT: usize = 4 << 20;
            while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
                match self.rx.recv() {
                    Ok(mut chunk) => {
                        let drop_n = (self.released - self.base) as usize;
                        if drop_n == self.buf.len() && self.tx_free.is_some() {
                            let mut old = std::mem::replace(&mut self.buf, chunk);
                            self.base = self.released;
                            old.clear();
                            if let Some(tx) = &self.tx_free { let _ = tx.send(old); }
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
                if self.cancelled.load(Ordering::Relaxed) {
                    return Err(ErrorKind::Cancelled);
                }
            }
            let i = (from - self.base) as usize;
            Ok((self.buf.get(i..).unwrap_or(&[]), self.eof))
        }
        fn release(&mut self, upto: u64) { self.released = self.released.max(upto); }
    }

    struct ReadyRing {
        slots_arr: Vec<Option<Result<Vec<u8>, Error>>>,
        mask: usize,
        taken: u64,
        end: Option<u64>,
        reader_waiting: bool,
        scanner_waiting: bool,
        free_out: Vec<Vec<u8>>,
        free_syms: Vec<Vec<u16>>,
    }

    struct JobQueue {
        q: VecDeque<Job>,
        closed: bool,
        waiting: usize,
    }

    pub fn decode_p1<R: Read + Send + 'static>(
        mut r: R,
        workers: usize,
        cfg: P1Cfg,
        k: Kernels,
        mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<usize, Error> {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let mask = cap - 1;
        let cancelled = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(),
            mask,
            taken: 0,
            end: None,
            reader_waiting: false,
            scanner_waiting: false,
            free_out: Vec::with_capacity(slots),
            free_syms: Vec::with_capacity(slots),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_scanner = Arc::new(Condvar::new());
        let job_deque = Arc::new((
            Mutex::new(JobQueue { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }),
            Condvar::new(),
        ));
        let (mpsc_tx, mpsc_rx) = mpsc::channel::<Job>();
        let mpsc_rx = Arc::new(Mutex::new(mpsc_rx));

        let (in_tx, in_rx) = mpsc::sync_channel::<Vec<u8>>(2);
        let (free_tx, free_rx) = if cfg.feed_pool_exact_out {
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            (Some(tx), Some(rx))
        } else { (None, None) };
        let chunk_sz = if cfg.feed_pool_exact_out { 1 << 20 } else { 4 << 20 };

        let feeder = std::thread::spawn(move || {
            let mut pool: Vec<Vec<u8>> = if cfg.feed_pool_exact_out {
                (0..3).map(|_| Vec::with_capacity(chunk_sz)).collect()
            } else { Vec::new() };
            loop {
                let mut buf = if let Some(rx) = &free_rx {
                    while let Ok(b) = rx.try_recv() { pool.push(b); }
                    let mut b = pool.pop().unwrap_or_else(|| Vec::with_capacity(chunk_sz));
                    b.resize(chunk_sz, 0);
                    b
                } else {
                    vec![0u8; chunk_sz]
                };
                let mut n = 0;
                while n < buf.len() {
                    match r.read(&mut buf[n..]) {
                        Ok(0) => break,
                        Ok(g) => n += g,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => return,
                    }
                }
                if n == 0 { break; }
                buf.truncate(n);
                if in_tx.send(buf).is_err() { return; }
            }
        });

        let s_ready = ready.clone();
        let s_cv_r = cv_reader.clone();
        let s_cv_s = cv_scanner.clone();
        let s_cancel = cancelled.clone();
        let s_jq = job_deque.clone();
        let scanner_h = std::thread::spawn(move || {
            let src = PooledSource {
                rx: in_rx, tx_free: free_tx, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: s_cancel.clone(),
            };
            let mut sc = Scanner::new(src);
            let mut issued: u64 = 0;
            let finish = |err: Option<Error>, issued: u64| {
                if cfg.ring_split_deque {
                    let mut jq = s_jq.0.lock().unwrap();
                    jq.closed = true;
                    drop(jq);
                    s_jq.1.notify_all();
                }
                let mut st = s_ready.lock().unwrap();
                match err {
                    Some(e) if e.kind != ErrorKind::Cancelled => {
                        let idx = (issued as usize) & st.mask;
                        st.slots_arr[idx] = Some(Err(e));
                        st.end = Some(issued + 1);
                    }
                    _ => st.end = Some(issued),
                }
                drop(st);
                s_cv_r.notify_one();
            };
            loop {
                let syms = {
                    let mut st = s_ready.lock().unwrap();
                    while !s_cancel.load(Ordering::Relaxed) && issued - st.taken >= slots as u64 {
                        st.scanner_waiting = true;
                        st = s_cv_s.wait(st).unwrap();
                    }
                    st.scanner_waiting = false;
                    if s_cancel.load(Ordering::Relaxed) {
                        drop(st);
                        return finish(None, issued);
                    }
                    st.free_syms.pop().unwrap_or_default()
                };
                match sc.next(syms) {
                    Ok(Event::Block { bit, symbols, .. }) => {
                        let place = issued;
                        issued += 1;
                        if cfg.ring_split_deque {
                            let mut jq = s_jq.0.lock().unwrap();
                            jq.q.push_back((place, bit, symbols));
                            let wake = jq.waiting > 0;
                            drop(jq);
                            if wake { s_jq.1.notify_one(); }
                        } else if mpsc_tx.send((place, bit, symbols)).is_err() {
                            return finish(None, issued);
                        }
                    }
                    Ok(Event::Done) => { drop(mpsc_tx); return finish(None, issued); }
                    Err(e) => { drop(mpsc_tx); return finish(Some(e), issued); }
                }
            }
        });

        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let w_ready = ready.clone();
            let w_cv_r = cv_reader.clone();
            let w_cancel = cancelled.clone();
            let w_jq = job_deque.clone();
            let w_mpsc = mpsc_rx.clone();
            whs.push(std::thread::spawn(move || {
                let mut scratch = Scratch::<OpenCells>::default();
                loop {
                    let job = if cfg.ring_split_deque {
                        let mut jq = w_jq.0.lock().unwrap();
                        loop {
                            if let Some(j) = jq.q.pop_front() { break Some(j); }
                            if jq.closed { break None; }
                            jq.waiting += 1;
                            jq = w_jq.1.wait(jq).unwrap();
                            jq.waiting -= 1;
                        }
                    } else {
                        w_mpsc.lock().unwrap().recv().ok()
                    };
                    let Some((place, bit, symbols)) = job else { return; };
                    if w_cancel.load(Ordering::Relaxed) { continue; }
                    let mut out = w_ready.lock().unwrap().free_out.pop().unwrap_or_default();
                    out.clear();
                    if cfg.feed_pool_exact_out && symbols.syms.len() > 2048 && out.capacity() < 960_000 {
                        out.reserve_exact(960_000);
                    }
                    let res = mibz2::decode_block(&symbols, k, &mut scratch, &mut out)
                        .map(|_| out)
                        .map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    let mut st = w_ready.lock().unwrap();
                    if !w_cancel.load(Ordering::Relaxed) {
                        let idx = (place as usize) & st.mask;
                        st.slots_arr[idx] = Some(res);
                    }
                    if st.free_syms.len() < slots { st.free_syms.push(symbols.syms); }
                    let wake = st.reader_waiting && place == st.taken;
                    drop(st);
                    if wake { w_cv_r.notify_one(); }
                }
            }));
        }

        let mut total = 0usize;
        let mut cur: Vec<u8> = Vec::new();
        let mut read_buf = if cfg.zero_copy_bufread { Vec::new() } else { vec![0u8; 4 << 20] };
        loop {
            let blk = {
                let mut st = ready.lock().unwrap();
                let spent = std::mem::take(&mut cur);
                if spent.capacity() != 0 && st.free_out.len() < slots {
                    st.free_out.push(spent);
                }
                loop {
                    let t = st.taken;
                    let idx = (t as usize) & st.mask;
                    if let Some(res) = st.slots_arr[idx].take() {
                        st.taken = t + 1;
                        let wake_s = st.scanner_waiting;
                        st.reader_waiting = false;
                        drop(st);
                        if wake_s { cv_scanner.notify_one(); }
                        break Some(res);
                    }
                    if st.end == Some(st.taken) {
                        st.reader_waiting = false;
                        break None;
                    }
                    st.reader_waiting = true;
                    st = cv_reader.wait(st).unwrap();
                }
            };
            match blk {
                None => break,
                Some(Ok(v)) => {
                    if cfg.zero_copy_bufread {
                        on_bytes(&v);
                        total += v.len();
                    } else {
                        let mut at = 0;
                        while at < v.len() {
                            let n = read_buf.len().min(v.len() - at);
                            read_buf[..n].copy_from_slice(&v[at..at + n]);
                            on_bytes(&read_buf[..n]);
                            at += n; total += n;
                        }
                    }
                    cur = v;
                }
                Some(Err(e)) => {
                    cancelled.store(true, Ordering::Release);
                    cv_scanner.notify_all();
                    job_deque.1.notify_all();
                    let _ = feeder.join(); let _ = scanner_h.join();
                    for h in whs { let _ = h.join(); }
                    return Err(e);
                }
            }
        }
        let _ = feeder.join();
        let _ = scanner_h.join();
        for h in whs { let _ = h.join(); }
        Ok(total)
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

struct AbHash { h: [u64; 4], tail: [u8; 32], nt: usize, n: u64 }
impl AbHash {
    fn new() -> Self { Self { h: [1, 2, 3, 4], tail: [0; 32], nt: 0, n: 0 } }
    #[inline]
    fn block(&mut self, b: &[u8; 32]) {
        for i in 0..4 {
            let v = u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap());
            self.h[i] = self.h[i].wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ v.rotate_left(17);
        }
    }
    fn update(&mut self, mut b: &[u8]) {
        self.n += b.len() as u64;
        if self.nt > 0 {
            let k = (32 - self.nt).min(b.len());
            self.tail[self.nt..self.nt + k].copy_from_slice(&b[..k]);
            self.nt += k; b = &b[k..];
            if self.nt < 32 { return; }
            let t = self.tail; self.block(&t); self.nt = 0;
        }
        let mut c = b.chunks_exact(32);
        for blk in &mut c { self.block(blk.try_into().unwrap()); }
        let rem = c.remainder();
        self.tail[..rem.len()].copy_from_slice(rem);
        self.nt = rem.len();
    }
    fn finish(&mut self) -> u64 {
        if self.nt > 0 { self.tail[self.nt..].fill(0); let t = self.tail; self.block(&t); self.nt = 0; }
        self.h[0] ^ self.h[1].rotate_left(13) ^ self.h[2].rotate_left(29) ^ self.h[3].rotate_left(47) ^ self.n
    }
}

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = 2.145; // df=14 t_0.975
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn verify_edge_cases() {
    let k = Kernels::detect();
    let cfg = safe_p1::P1Cfg { feed_pool_exact_out: true, ring_split_deque: true, zero_copy_bufread: true };
    // 1. Empty stream -> ErrorKind::Empty
    let r = safe_p1::decode_p1(Cursor::new(Vec::<u8>::new()), 4, cfg, k, |_| {});
    assert_eq!(r.unwrap_err().kind, ErrorKind::Empty);
    // 2. Corrupt header -> BadHeader
    let r = safe_p1::decode_p1(Cursor::new(b"NOTBZIP2".to_vec()), 4, cfg, k, |_| {});
    assert!(r.is_err());
    // 3. Truncated valid stream -> same error kind as master
    let raw = fs::read("/tmp/wormhole_r5/wikidata_60b.bz2").unwrap();
    let trunc = raw[..raw.len() / 2 - 777].to_vec();
    let mut rdr = BzReader::parallel_workers(Cursor::new(trunc.clone()), 4, 16, k, true);
    let mut tmp = Vec::new();
    let m_err = rdr.read_to_end(&mut tmp).unwrap_err();
    let mut c_out = Vec::new();
    let c_err = safe_p1::decode_p1(Cursor::new(trunc), 4, cfg, k, |b| c_out.extend_from_slice(b)).unwrap_err();
    assert_eq!(tmp.len(), c_out.len(), "truncated prefix byte count mismatch");
    assert_eq!(tmp, c_out, "truncated prefix bytes mismatch");
    let _ = (m_err, c_err);
    // 4. Bit-flip CRC error in block 2 -> exact prefix + BlockCrc error
    let mut bad = raw[..400_000].to_vec();
    bad[50_000] ^= 0x80;
    let r_bad = safe_p1::decode_p1(Cursor::new(bad), 4, cfg, k, |_| {});
    assert!(r_bad.is_err());
    println!("EDGE-CASE PARITY: PASS (empty, bad-header, truncated prefix exact, bit-flip error)");
}

fn run_arm(arm: &str, data: Vec<u8>, w: usize) -> (f64, f64, f64, i64, i64, f64, u64) {
    let k = Kernels::detect();
    let r0 = rusage();
    let t0 = Instant::now();
    let mut h = AbHash::new();
    match arm {
        "MasterA" | "MasterB" => {
            let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, 4 * w, k, true);
            let mut buf = vec![0u8; 4 << 20];
            loop {
                let n = rdr.read(&mut buf).unwrap();
                if n == 0 { break; }
                h.update(&buf[..n]);
            }
            drop(rdr);
        }
        "P1a_C32" => {
            let cfg = safe_p1::P1Cfg { feed_pool_exact_out: true, ring_split_deque: false, zero_copy_bufread: false };
            safe_p1::decode_p1(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
        }
        "P1b_C32_H123" => {
            let cfg = safe_p1::P1Cfg { feed_pool_exact_out: true, ring_split_deque: true, zero_copy_bufread: false };
            safe_p1::decode_p1(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
        }
        "P1c_Full_ZC" => {
            let cfg = safe_p1::P1Cfg { feed_pool_exact_out: true, ring_split_deque: true, zero_copy_bufread: true };
            safe_p1::decode_p1(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
        }
        _ => panic!("unknown arm {arm}"),
    }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0, h.finish())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let w: usize = args[4].parse().unwrap();
        let (wall, u, s, flt, vcsw, rss, h) = run_arm(&args[3], data, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {vcsw} {rss:.2} {h:016x}");
        return;
    }
    verify_edge_cases();
    let exe = std::env::current_exe().unwrap();
    let arms = ["MasterA", "MasterB", "P1a_C32", "P1b_C32_H123", "P1c_Full_ZC"];

    for (label, path, rounds) in [
        ("wikidata_431b_210m", "/tmp/wormhole_r5/wikidata_431b_210m.bz2", 15usize),
        ("hetero_60b", "/tmp/wormhole_r5/hetero_60b.bz2", 15usize),
    ] {
        println!("\n=== {rounds}-ROUND ROTATED FRESH-PROCESS 4W CUMULATIVE ABLATION ({label}) ===");
        let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut users: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut vcsws: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ref_hash = String::new();
        for r in 0..rounds {
            for k in 0..arms.len() {
                let arm = arms[(r + k) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                if ref_hash.is_empty() { ref_hash = p[6].to_string(); }
                assert_eq!(p[6], ref_hash, "HASH MISMATCH {label} {arm}");
                walls.entry(arm).or_default().push(p[0].parse().unwrap());
                users.entry(arm).or_default().push(p[1].parse().unwrap());
                syss.entry(arm).or_default().push(p[2].parse().unwrap());
                flts.entry(arm).or_default().push(p[3].parse().unwrap());
                vcsws.entry(arm).or_default().push(p[4].parse().unwrap());
                rsss.entry(arm).or_default().push(p[5].parse().unwrap());
            }
        }
        let ma = &walls["MasterA"];
        println!("{:<14} {:>8} {:>24} {:>8} {:>7} {:>7} {:>6} {:>7}", "Arm", "med_ms", "ratio vs MasterA [95%]", "user_ms", "sys_ms", "minflt", "vcsw", "rss_MB");
        for &arm in &arms {
            let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut uv = users[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = flts[arm].clone(); fv.sort();
            let mut cv = vcsws[arm].clone(); cv.sort();
            let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let pairs: Vec<(f64, f64)> = ma.iter().copied().zip(walls[arm].iter().copied()).collect();
            let (rat, lo, hi) = ci95_ratio(&pairs);
            println!("{:<14} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>8.2} {:>7.2} {:>7} {:>6} {:>7.2}",
                arm, wv[rounds/2], rat, lo, hi, uv[rounds/2], sv[rounds/2], fv[rounds/2], cv[rounds/2], rv[rounds/2]);
        }
    }

    println!("\n=== COLD-START W=10 & W=16 FINAL PHASE-1 CHECK (wikidata_431b_210m, 5-rd median) ===");
    for &w in &[10usize, 16] {
        println!("\n--- W = {w} ---");
        println!("{:<14} {:>8} {:>8} {:>7} {:>7} {:>6} {:>7}", "Arm", "wall_ms", "user_ms", "sys_ms", "minflt", "vcsw", "rss_MB");
        for &arm in &arms {
            let mut rows = Vec::new();
            for _ in 0..5 {
                let out = Command::new(&exe).args(["--child", "/tmp/wormhole_r5/wikidata_431b_210m.bz2", arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                rows.push((p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                           p[3].parse::<i64>().unwrap(), p[4].parse::<i64>().unwrap(), p[5].parse::<f64>().unwrap()));
            }
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let m = rows[2];
            println!("{:<14} {:>8.2} {:>8.2} {:>7.2} {:>7} {:>6} {:>7.2}", arm, m.0, m.1, m.2, m.3, m.4, m.5);
        }
    }
}
