use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, Source};

// ---- Cycle 34: Block Hand-Off, Ring Slots, Split CVs, Bounded Deque & Atomic Slot Fast-Path ----
mod safe_c34 {
    #![forbid(unsafe_code)]
    use super::*;

    type Job = (u64, u64, mibz2::block::Symbols);

    struct PooledSource {
        rx: mpsc::Receiver<Vec<u8>>,
        tx_free: mpsc::Sender<Vec<u8>>,
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
                        if drop_n == self.buf.len() {
                            let mut old = std::mem::replace(&mut self.buf, chunk);
                            self.base = self.released;
                            old.clear();
                            let _ = self.tx_free.send(old);
                        } else {
                            if drop_n >= COMPACT {
                                self.buf.drain(..drop_n);
                                self.base = self.released;
                            }
                            self.buf.extend_from_slice(&chunk);
                            chunk.clear();
                            let _ = self.tx_free.send(chunk);
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

    // ---- H12: Ring-indexed ready array + Split directed CVs (keeping mpsc job queue) ----
    // ---- H123: + Bounded VecDeque job queue (replacing Mutex<mpsc::Receiver>) ----
    // ---- H1234: + Split free_syms/free_out pools + AtomicU64 taken fast-path (0 scanner locks!) ----
    #[derive(Clone, Copy, Debug)]
    pub struct HandoffCfg {
        pub ring_and_split_cv: bool, // H12
        pub deque_job_queue: bool,   // H3
        pub split_pools_atomic: bool,// H4
        pub zero_copy_reader: bool,  // compose with C33b BufRead zero-copy
    }

    struct ReadyRing {
        slots_arr: Vec<Option<Result<Vec<u8>, Error>>>,
        mask: usize,
        taken: u64,
        end: Option<u64>,
        reader_waiting: bool,
        scanner_waiting: bool,
        // used when !split_pools_atomic:
        free_out: Vec<Vec<u8>>,
        free_syms: Vec<Vec<u16>>,
    }

    struct JobQueue {
        q: VecDeque<Job>,
        closed: bool,
        waiting: usize,
    }

    pub struct Stats {
        pub bytes: usize,
        pub wake_reader: usize,
        pub wake_scanner: usize,
        pub wake_worker: usize,
        pub atomic_fast_slots: usize,
    }

    pub fn decode_c34<R: Read + Send + 'static>(
        mut r: R,
        workers: usize,
        cfg: HandoffCfg,
        k: Kernels,
        mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<Stats, Error> {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let mask = cap - 1;
        let cancelled = Arc::new(AtomicBool::new(false));
        let taken_atomic = Arc::new(AtomicU64::new(0));
        let wake_reader = Arc::new(AtomicUsize::new(0));
        let wake_scanner = Arc::new(AtomicUsize::new(0));
        let wake_worker = Arc::new(AtomicUsize::new(0));
        let atomic_fast = Arc::new(AtomicUsize::new(0));

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

        let free_out_split = Arc::new(Mutex::new(Vec::<Vec<u8>>::with_capacity(slots)));
        let free_syms_split = Arc::new(Mutex::new(Vec::<Vec<u16>>::with_capacity(slots)));

        let (in_tx, in_rx) = mpsc::sync_channel::<Vec<u8>>(2);
        let (free_tx, free_rx) = mpsc::channel::<Vec<u8>>();

        // Job dispatch: either Mutex<VecDeque> + Condvar (H3) or Mutex<mpsc::Receiver>
        let job_deque = Arc::new((
            Mutex::new(JobQueue { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }),
            Condvar::new(),
        ));
        let (mpsc_tx, mpsc_rx) = mpsc::channel::<Job>();
        let mpsc_rx = Arc::new(Mutex::new(mpsc_rx));

        let feeder = std::thread::spawn(move || {
            let mut pool: Vec<Vec<u8>> = (0..3).map(|_| Vec::with_capacity(1 << 20)).collect();
            loop {
                while let Ok(b) = free_rx.try_recv() { pool.push(b); }
                let mut buf = pool.pop().unwrap_or_else(|| Vec::with_capacity(1 << 20));
                buf.resize(1 << 20, 0);
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

        // Scanner thread
        let s_ready = ready.clone();
        let s_cv_r = cv_reader.clone();
        let s_cv_s = cv_scanner.clone();
        let s_cancel = cancelled.clone();
        let s_taken = taken_atomic.clone();
        let s_fsyms = free_syms_split.clone();
        let s_jq = job_deque.clone();
        let s_ww = wake_worker.clone();
        let s_af = atomic_fast.clone();
        let scanner_h = std::thread::spawn(move || {
            let src = PooledSource {
                rx: in_rx, tx_free: free_tx, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: s_cancel.clone(),
            };
            let mut sc = Scanner::new(src);
            let mut issued: u64 = 0;
            let finish = |err: Option<Error>, issued: u64| {
                if cfg.deque_job_queue {
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
                // Slot wait + syms pop
                let syms = if cfg.split_pools_atomic {
                    if issued - s_taken.load(Ordering::Acquire) < slots as u64 {
                        s_af.fetch_add(1, Ordering::Relaxed);
                    } else {
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
                    }
                    if s_cancel.load(Ordering::Relaxed) {
                        return finish(None, issued);
                    }
                    s_fsyms.lock().unwrap().pop().unwrap_or_default()
                } else {
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
                        if cfg.deque_job_queue {
                            let mut jq = s_jq.0.lock().unwrap();
                            jq.q.push_back((place, bit, symbols));
                            let need_wake = jq.waiting > 0;
                            drop(jq);
                            if need_wake {
                                s_ww.fetch_add(1, Ordering::Relaxed);
                                s_jq.1.notify_one();
                            }
                        } else if mpsc_tx.send((place, bit, symbols)).is_err() {
                            return finish(None, issued);
                        }
                    }
                    Ok(Event::Done) => {
                        drop(mpsc_tx);
                        return finish(None, issued);
                    }
                    Err(e) => {
                        drop(mpsc_tx);
                        return finish(Some(e), issued);
                    }
                }
            }
        });

        // Worker threads
        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let w_ready = ready.clone();
            let w_cv_r = cv_reader.clone();
            let w_cancel = cancelled.clone();
            let w_fout = free_out_split.clone();
            let w_fsyms = free_syms_split.clone();
            let w_jq = job_deque.clone();
            let w_mpsc = mpsc_rx.clone();
            let w_wr = wake_reader.clone();
            whs.push(std::thread::spawn(move || {
                let mut scratch = Scratch::<OpenCells>::default();
                loop {
                    let job = if cfg.deque_job_queue {
                        let mut jq = w_jq.0.lock().unwrap();
                        loop {
                            if let Some(j) = jq.q.pop_front() {
                                break Some(j);
                            }
                            if jq.closed {
                                break None;
                            }
                            jq.waiting += 1;
                            jq = w_jq.1.wait(jq).unwrap();
                            jq.waiting -= 1;
                        }
                    } else {
                        w_mpsc.lock().unwrap().recv().ok()
                    };
                    let Some((place, bit, symbols)) = job else { return; };
                    if w_cancel.load(Ordering::Relaxed) { continue; }
                    let mut out = if cfg.split_pools_atomic {
                        w_fout.lock().unwrap().pop().unwrap_or_default()
                    } else {
                        w_ready.lock().unwrap().free_out.pop().unwrap_or_default()
                    };
                    out.clear();
                    if symbols.syms.len() > 2048 && out.capacity() < 960_000 {
                        out.reserve_exact(960_000);
                    }
                    let res = mibz2::decode_block(&symbols, k, &mut scratch, &mut out)
                        .map(|_| out)
                        .map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    if cfg.split_pools_atomic {
                        {
                            let mut fs = w_fsyms.lock().unwrap();
                            if fs.len() < slots { fs.push(symbols.syms); }
                        }
                        let mut st = w_ready.lock().unwrap();
                        if !w_cancel.load(Ordering::Relaxed) {
                            let idx = (place as usize) & st.mask;
                            st.slots_arr[idx] = Some(res);
                        }
                        let wake = st.reader_waiting && place == st.taken;
                        drop(st);
                        if wake {
                            w_wr.fetch_add(1, Ordering::Relaxed);
                            w_cv_r.notify_one();
                        }
                    } else {
                        let mut st = w_ready.lock().unwrap();
                        if !w_cancel.load(Ordering::Relaxed) {
                            let idx = (place as usize) & st.mask;
                            st.slots_arr[idx] = Some(res);
                        }
                        if st.free_syms.len() < slots {
                            st.free_syms.push(symbols.syms);
                        }
                        let wake = st.reader_waiting && place == st.taken;
                        drop(st);
                        if wake {
                            w_wr.fetch_add(1, Ordering::Relaxed);
                            w_cv_r.notify_one();
                        }
                    }
                }
            }));
        }

        // Reader thread
        let mut total = 0usize;
        let mut cur: Vec<u8> = Vec::new();
        let mut read_buf = if cfg.zero_copy_reader { Vec::new() } else { vec![0u8; 4 << 20] };
        loop {
            let mut spent = Some(std::mem::take(&mut cur));
            if cfg.split_pools_atomic {
                if let Some(v) = spent.take() {
                    if v.capacity() != 0 {
                        let mut fo = free_out_split.lock().unwrap();
                        if fo.len() < slots { fo.push(v); }
                    }
                }
            }
            let blk = {
                let mut st = ready.lock().unwrap();
                if let Some(v) = spent.take() {
                    if v.capacity() != 0 && st.free_out.len() < slots {
                        st.free_out.push(v);
                    }
                }
                loop {
                    let t = st.taken;
                    let idx = (t as usize) & st.mask;
                    if let Some(res) = st.slots_arr[idx].take() {
                        st.taken = t + 1;
                        taken_atomic.store(t + 1, Ordering::Release);
                        let wake_s = st.scanner_waiting;
                        st.reader_waiting = false;
                        drop(st);
                        if wake_s {
                            wake_scanner.fetch_add(1, Ordering::Relaxed);
                            cv_scanner.notify_one();
                        }
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
                    if cfg.zero_copy_reader {
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
        Ok(Stats {
            bytes: total,
            wake_reader: wake_reader.load(Ordering::Relaxed),
            wake_scanner: wake_scanner.load(Ordering::Relaxed),
            wake_worker: wake_worker.load(Ordering::Relaxed),
            atomic_fast_slots: atomic_fast.load(Ordering::Relaxed),
        })
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
    let t = 2.201;
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn run_arm(arm: &str, data: Vec<u8>, w: usize) -> (f64, f64, f64, i64, i64, f64, u64, usize, usize, usize) {
    let k = Kernels::detect();
    let r0 = rusage();
    let t0 = Instant::now();
    let mut h = AbHash::new();
    let (mut wr, mut ws, mut ww) = (0, 0, 0);
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
        "H12_RingSplitCV" => {
            let cfg = safe_c34::HandoffCfg { ring_and_split_cv: true, deque_job_queue: false, split_pools_atomic: false, zero_copy_reader: false };
            let st = safe_c34::decode_c34(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            wr = st.wake_reader; ws = st.wake_scanner; ww = st.wake_worker;
        }
        "H123_DequeQ" => {
            let cfg = safe_c34::HandoffCfg { ring_and_split_cv: true, deque_job_queue: true, split_pools_atomic: false, zero_copy_reader: false };
            let st = safe_c34::decode_c34(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            wr = st.wake_reader; ws = st.wake_scanner; ww = st.wake_worker;
        }
        "H1234_FullCopy" => {
            let cfg = safe_c34::HandoffCfg { ring_and_split_cv: true, deque_job_queue: true, split_pools_atomic: true, zero_copy_reader: false };
            let st = safe_c34::decode_c34(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            wr = st.wake_reader; ws = st.wake_scanner; ww = st.wake_worker;
        }
        "H1234_ZeroCopy" => {
            let cfg = safe_c34::HandoffCfg { ring_and_split_cv: true, deque_job_queue: true, split_pools_atomic: true, zero_copy_reader: true };
            let st = safe_c34::decode_c34(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            wr = st.wake_reader; ws = st.wake_scanner; ww = st.wake_worker;
        }
        _ => panic!("unknown arm {arm}"),
    }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0, h.finish(), wr, ws, ww)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let w: usize = args[4].parse().unwrap();
        let (wall, u, s, flt, vcsw, rss, h, wr, ws, ww) = run_arm(&args[3], data, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {vcsw} {rss:.2} {h:016x} {wr} {ws} {ww}");
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let path210 = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    let arms = ["MasterA", "MasterB", "H12_RingSplitCV", "H123_DequeQ", "H1234_FullCopy", "H1234_ZeroCopy"];

    println!("=== PART 1: 12-ROUND ROTATED FRESH-PROCESS 4W A/B (wikidata_431b_210m.bz2, 211 MB, AbHash) ===");
    let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut vcsws: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut wakes: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    let mut ref_hash = String::new();
    for r in 0..12 {
        for k in 0..arms.len() {
            let arm = arms[(r + k) % arms.len()];
            let out = Command::new(&exe).args(["--child", path210, arm, "4"]).output().unwrap();
            let s = String::from_utf8(out.stdout).unwrap();
            let p: Vec<&str> = s.split_whitespace().collect();
            if ref_hash.is_empty() { ref_hash = p[6].to_string(); }
            assert_eq!(p[6], ref_hash, "HASH MISMATCH {arm}");
            walls.entry(arm).or_default().push(p[0].parse().unwrap());
            syss.entry(arm).or_default().push(p[2].parse().unwrap());
            flts.entry(arm).or_default().push(p[3].parse().unwrap());
            vcsws.entry(arm).or_default().push(p[4].parse().unwrap());
            rsss.entry(arm).or_default().push(p[5].parse().unwrap());
            wakes.insert(arm, (p[7].parse().unwrap(), p[8].parse().unwrap(), p[9].parse().unwrap()));
        }
    }
    let ma = &walls["MasterA"];
    println!("{:<16} {:>8} {:>24} {:>7} {:>7} {:>6} {:>7} {:>13}", "Arm", "med_ms", "ratio vs MasterA [95%]", "sys_ms", "minflt", "vcsw", "rss_MB", "wakes(r/s/w)");
    for &arm in &arms {
        let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = flts[arm].clone(); fv.sort();
        let mut cv = vcsws[arm].clone(); cv.sort();
        let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let pairs: Vec<(f64, f64)> = ma.iter().copied().zip(walls[arm].iter().copied()).collect();
        let (rat, lo, hi) = ci95_ratio(&pairs);
        let (wr, ws, ww) = wakes[arm];
        println!("{:<16} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>7.2} {:>7} {:>6} {:>7.2} {:>4}/{:>3}/{:>4}",
            arm, wv[6], rat, lo, hi, sv[6], fv[6], cv[6], rv[6], wr, ws, ww);
    }

    println!("\n=== PART 2: W=10 & W=16 COLD-START CONTENTION CHECK (wikidata_431b_210m.bz2, 3-rd median) ===");
    for &w in &[10usize, 16] {
        println!("\n--- W = {w} ---");
        println!("{:<16} {:>8} {:>8} {:>7} {:>7} {:>6} {:>7} {:>13}", "Arm", "wall_ms", "user_ms", "sys_ms", "minflt", "vcsw", "rss_MB", "wakes(r/s/w)");
        for &arm in &arms {
            let mut rows = Vec::new();
            for _ in 0..3 {
                let out = Command::new(&exe).args(["--child", path210, arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                assert_eq!(p[6], ref_hash);
                rows.push((p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                           p[3].parse::<i64>().unwrap(), p[4].parse::<i64>().unwrap(), p[5].parse::<f64>().unwrap(),
                           p[7].parse::<usize>().unwrap(), p[8].parse::<usize>().unwrap(), p[9].parse::<usize>().unwrap()));
            }
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let m = rows[1];
            println!("{:<16} {:>8.2} {:>8.2} {:>7.2} {:>7} {:>6} {:>7.2} {:>4}/{:>3}/{:>4}",
                arm, m.0, m.1, m.2, m.3, m.4, m.5, m.6, m.7, m.8);
        }
    }
}
