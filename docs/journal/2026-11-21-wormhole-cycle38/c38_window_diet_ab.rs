use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::stream::{Event, Scanner, Source};

// ---- Cycle 38: Window L3 Pollution Diet (Immediate syms Return + Byte-Budgeted Ready Cap) ----
mod safe_c38 {
    #![forbid(unsafe_code)]
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub struct WinCfg {
        pub slots: usize,
        pub immed_syms_return: bool, // W1: return syms to pool immediately after runs::front
        pub max_ready_bytes: usize,  // W3: 0 = unlimited (slots only), else byte cap on dirty ready queue
        pub direct_inplace_rle: bool,// compose with C37a
    }

    #[inline]
    fn find_run4_swar(s: &[u8], mut i: usize) -> Option<usize> {
        let n = s.len();
        if n < 4 || i + 4 > n { return None; }
        while i + 11 <= n {
            let w0 = u64::from_ne_bytes(s[i..i + 8].try_into().unwrap());
            let w1 = u64::from_ne_bytes(s[i + 1..i + 9].try_into().unwrap());
            let w2 = u64::from_ne_bytes(s[i + 2..i + 10].try_into().unwrap());
            let w3 = u64::from_ne_bytes(s[i + 3..i + 11].try_into().unwrap());
            let d = (w0 ^ w1) | (w0 ^ w2) | (w0 ^ w3);
            let z = d.wrapping_sub(0x0101_0101_0101_0101) & !d & 0x8080_8080_8080_8080;
            if z != 0 { return Some(i + (z.trailing_zeros() >> 3) as usize); }
            i += 8;
        }
        while i + 3 < n {
            let c = s[i];
            if s[i + 1] == c && s[i + 2] == c && s[i + 3] == c { return Some(i); }
            i += 1;
        }
        None
    }

    fn unrle_in_place(buf: &mut Vec<u8>, runs_s: &mut Vec<(usize, u8, u8)>, k: Kernels) -> Result<u32, ErrorKind> {
        let n = buf.len();
        runs_s.clear();
        let (mut i, mut delta, mut max_ov, mut min_d) = (0usize, 0isize, 0isize, 0isize);
        while i < n {
            let Some(j) = find_run4_swar(buf, i) else { break; };
            let c = buf[j];
            let Some(&extra) = buf.get(j + 4) else { return Err(ErrorKind::RunUnfinished); };
            runs_s.push((j, c, extra));
            delta += extra as isize - 1;
            if delta > max_ov { max_ov = delta; }
            if delta < min_d { min_d = delta; }
            i = j + 5;
        }
        if runs_s.is_empty() { return Ok(mibz2::crc::crc32(buf, k)); }
        let final_len = (n as isize + delta) as usize;
        let r0 = runs_s[0].0;
        if min_d >= 0 {
            if final_len > buf.len() { buf.resize(final_len, 0); }
            let (mut se, mut de) = (n, final_len);
            for &(j, c, extra) in runs_s.iter().rev() {
                let ls = j + 5;
                let ll = se - ls;
                if ll > 0 { de -= ll; if de != ls { buf.copy_within(ls..se, de); } }
                let rl = 4 + extra as usize;
                de -= rl;
                buf[de..de + rl].fill(c);
                se = j;
            }
            return Ok(mibz2::crc::crc32(buf, k));
        }
        let shift = max_ov.max(0) as usize;
        let wl = (n + shift).max(final_len);
        if wl > buf.len() { buf.resize(wl, 0); }
        if shift > 0 { buf.copy_within(r0..n, r0 + shift); }
        let (mut src, mut dst) = (r0 + shift, r0);
        for &(j, c, extra) in runs_s.iter() {
            let rs = j + shift;
            let ll = rs - src;
            if ll > 0 { if dst != src { buf.copy_within(src..rs, dst); } dst += ll; }
            let rl = 4 + extra as usize;
            buf[dst..dst + rl].fill(c);
            dst += rl; src = rs + 5;
        }
        let tl = (n + shift) - src;
        if tl > 0 { if dst != src { buf.copy_within(src..n + shift, dst); } dst += tl; }
        buf.truncate(final_len);
        Ok(mibz2::crc::crc32(buf, k))
    }

    type Job = (u64, u64, mibz2::block::Symbols);
    struct PooledSource {
        rx: mpsc::Receiver<Vec<u8>>, tx_free: mpsc::Sender<Vec<u8>>,
        buf: Vec<u8>, base: u64, released: u64, eof: bool, cancelled: Arc<AtomicBool>,
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
                            self.base = self.released; old.clear();
                            let _ = self.tx_free.send(old);
                        } else {
                            if drop_n >= COMPACT { self.buf.drain(..drop_n); self.base = self.released; }
                            self.buf.extend_from_slice(&chunk);
                            chunk.clear(); let _ = self.tx_free.send(chunk);
                        }
                    }
                    Err(_) => self.eof = true,
                }
                if self.cancelled.load(Ordering::Relaxed) { return Err(ErrorKind::Cancelled); }
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
        ready_bytes: usize,
        peak_ready_bytes: usize,
        peak_out_pool: usize,
        peak_syms_pool: usize,
        free_out: Vec<Vec<u8>>,
        free_syms: Vec<Vec<u16>>,
    }
    struct JobQueue { q: VecDeque<Job>, closed: bool, waiting: usize }

    pub struct WinStats {
        pub bytes: usize,
        pub peak_ready_mib: f64,
        pub peak_out_bufs: usize,
        pub peak_syms_bufs: usize,
    }

    pub fn decode_c38<R: Read + Send + 'static>(
        mut r: R, workers: usize, cfg: WinCfg, k: Kernels, mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<WinStats, Error> {
        let slots = cfg.slots.max(workers);
        let cap = slots.next_power_of_two();
        let cancelled = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0, end: None,
            reader_waiting: false, scanner_waiting: false,
            ready_bytes: 0, peak_ready_bytes: 0, peak_out_pool: 0, peak_syms_pool: 0,
            free_out: Vec::with_capacity(slots), free_syms: Vec::with_capacity(slots),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_scanner = Arc::new(Condvar::new());
        let job_deque = Arc::new((Mutex::new(JobQueue { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }), Condvar::new()));
        let (in_tx, in_rx) = mpsc::sync_channel::<Vec<u8>>(2);
        let (free_tx, free_rx) = mpsc::channel::<Vec<u8>>();

        let feeder = std::thread::spawn(move || {
            let mut pool: Vec<Vec<u8>> = (0..3).map(|_| Vec::with_capacity(1 << 20)).collect();
            loop {
                while let Ok(b) = free_rx.try_recv() { pool.push(b); }
                let mut buf = pool.pop().unwrap_or_else(|| Vec::with_capacity(1 << 20));
                buf.resize(1 << 20, 0);
                let mut n = 0;
                while n < buf.len() {
                    match r.read(&mut buf[n..]) {
                        Ok(0) => break, Ok(g) => n += g,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => return,
                    }
                }
                if n == 0 { break; }
                buf.truncate(n);
                if in_tx.send(buf).is_err() { return; }
            }
        });

        let (sr, scr, scs, sc, sjq) = (ready.clone(), cv_reader.clone(), cv_scanner.clone(), cancelled.clone(), job_deque.clone());
        let scanner_h = std::thread::spawn(move || {
            let src = PooledSource { rx: in_rx, tx_free: free_tx, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: sc.clone() };
            let mut scnr = Scanner::new(src);
            let mut issued: u64 = 0;
            let finish = |err: Option<Error>, issued: u64| {
                { let mut jq = sjq.0.lock().unwrap(); jq.closed = true; }
                sjq.1.notify_all();
                let mut st = sr.lock().unwrap();
                match err {
                    Some(e) if e.kind != ErrorKind::Cancelled => {
                        let idx = (issued as usize) & st.mask;
                        st.slots_arr[idx] = Some(Err(e));
                        st.end = Some(issued + 1);
                    }
                    _ => st.end = Some(issued),
                }
                drop(st); scr.notify_one();
            };
            loop {
                let syms = {
                    let mut st = sr.lock().unwrap();
                    // Allow scanner to proceed if within slot limit AND either byte cap disabled OR
                    // (ready_bytes < cap OR in-flight < workers + 2 so workers never starve on HOL!)
                    while !sc.load(Ordering::Relaxed) && (
                        issued - st.taken >= slots as u64 ||
                        (cfg.max_ready_bytes > 0 && st.ready_bytes >= cfg.max_ready_bytes && issued - st.taken >= (workers + 2) as u64)
                    ) {
                        st.scanner_waiting = true;
                        st = scs.wait(st).unwrap();
                    }
                    st.scanner_waiting = false;
                    if sc.load(Ordering::Relaxed) { drop(st); return finish(None, issued); }
                    if let Some(s) = st.free_syms.pop() {
                        s
                    } else {
                        st.peak_syms_pool += 1;
                        Vec::new()
                    }
                };
                match scnr.next(syms) {
                    Ok(Event::Block { bit, symbols, .. }) => {
                        let place = issued; issued += 1;
                        let mut jq = sjq.0.lock().unwrap();
                        jq.q.push_back((place, bit, symbols));
                        let wake = jq.waiting > 0;
                        drop(jq);
                        if wake { sjq.1.notify_one(); }
                    }
                    Ok(Event::Done) => return finish(None, issued),
                    Err(e) => return finish(Some(e), issued),
                }
            }
        });

        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (wr, wcr, wc, wjq) = (ready.clone(), cv_reader.clone(), cancelled.clone(), job_deque.clone());
            whs.push(std::thread::spawn(move || {
                let mut open = OpenCells::default();
                let mut runs = mibz2::runs::Runs::default();
                let mut fallback = Scratch::<OpenCells>::default();
                let mut rle_runs = Vec::with_capacity(256);
                loop {
                    let job = {
                        let mut jq = wjq.0.lock().unwrap();
                        loop {
                            if let Some(j) = jq.q.pop_front() { break Some(j); }
                            if jq.closed { break None; }
                            jq.waiting += 1; jq = wjq.1.wait(jq).unwrap(); jq.waiting -= 1;
                        }
                    };
                    let Some((place, bit, mut symbols)) = job else { return; };
                    if wc.load(Ordering::Relaxed) { continue; }

                    let mut syms_returned = false;
                    let res: Result<Vec<u8>, ErrorKind> = if cfg.direct_inplace_rle && open.reads_runs() && mibz2::runs::takes_runs(&symbols) {
                        match mibz2::runs::front(&symbols, &mut runs, k) {
                            Err(e) => Err(e),
                            Ok(n) if symbols.orig_ptr as usize >= n => Err(ErrorKind::BadOrigPtr),
                            Ok(n) => {
                                let is_full = symbols.syms.len() > 2048;
                                // W1: IMMEDIATELY return `symbols.syms` BEFORE OpenCells runs AND pop `out` in ONE lock!
                                let mut out = {
                                    let mut st = wr.lock().unwrap();
                                    if cfg.immed_syms_return {
                                        let s = std::mem::take(&mut symbols.syms);
                                        if st.free_syms.len() < slots { st.free_syms.push(s); }
                                        syms_returned = true;
                                    }
                                    if is_full {
                                        if let Some(v) = st.free_out.pop() { v } else {
                                            st.peak_out_pool += 1;
                                            let mut v = Vec::new();
                                            v.reserve_exact(960_000);
                                            v
                                        }
                                    } else {
                                        Vec::new()
                                    }
                                };
                                if open.invert_runs(&runs, n, symbols.orig_ptr, &mut out, k) {
                                    match unrle_in_place(&mut out, &mut rle_runs, k) {
                                        Ok(c) if c == symbols.crc => Ok(out),
                                        Ok(computed) => Err(ErrorKind::BlockCrc { stored: symbols.crc, computed }),
                                        Err(e) => Err(e),
                                    }
                                } else {
                                    // Fallback needs symbols; only reachable if runs didn't tile
                                    Err(ErrorKind::BadOrigPtr)
                                }
                            }
                        }
                    } else {
                        let mut out = {
                            let mut st = wr.lock().unwrap();
                            if let Some(v) = st.free_out.pop() { v } else {
                                st.peak_out_pool += 1;
                                Vec::new()
                            }
                        };
                        out.clear();
                        if symbols.syms.len() > 2048 && out.capacity() < 960_000 { out.reserve_exact(960_000); }
                        mibz2::decode_block(&symbols, k, &mut fallback, &mut out).map(|_| out)
                    };

                    let res = res.map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    let out_bytes = res.as_ref().map_or(0, |v| v.len());
                    let mut st = wr.lock().unwrap();
                    if !wc.load(Ordering::Relaxed) {
                        let idx = (place as usize) & st.mask;
                        st.slots_arr[idx] = Some(res);
                        st.ready_bytes += out_bytes;
                        if st.ready_bytes > st.peak_ready_bytes { st.peak_ready_bytes = st.ready_bytes; }
                    }
                    if !syms_returned && st.free_syms.len() < slots {
                        st.free_syms.push(symbols.syms);
                    }
                    let wake = st.reader_waiting && place == st.taken;
                    drop(st);
                    if wake { wcr.notify_one(); }
                }
            }));
        }

        let mut total = 0usize;
        let mut cur: Vec<u8> = Vec::new();
        loop {
            let blk = {
                let mut st = ready.lock().unwrap();
                let spent = std::mem::take(&mut cur);
                if spent.capacity() >= 65_536 && st.free_out.len() < slots { st.free_out.push(spent); }
                loop {
                    let t = st.taken;
                    let idx = (t as usize) & st.mask;
                    if let Some(res) = st.slots_arr[idx].take() {
                        let blen = res.as_ref().map_or(0, |v| v.len());
                        st.ready_bytes = st.ready_bytes.saturating_sub(blen);
                        st.taken = t + 1;
                        let wake_s = st.scanner_waiting;
                        st.reader_waiting = false;
                        drop(st);
                        if wake_s { cv_scanner.notify_one(); }
                        break Some(res);
                    }
                    if st.end == Some(st.taken) { st.reader_waiting = false; break None; }
                    st.reader_waiting = true;
                    st = cv_reader.wait(st).unwrap();
                }
            };
            match blk {
                None => break,
                Some(Ok(v)) => { on_bytes(&v); total += v.len(); cur = v; }
                Some(Err(e)) => {
                    cancelled.store(true, Ordering::Release);
                    cv_scanner.notify_all(); job_deque.1.notify_all();
                    let _ = feeder.join(); let _ = scanner_h.join();
                    for h in whs { let _ = h.join(); }
                    return Err(e);
                }
            }
        }
        let _ = feeder.join(); let _ = scanner_h.join();
        for h in whs { let _ = h.join(); }
        let st = ready.lock().unwrap();
        Ok(WinStats {
            bytes: total,
            peak_ready_mib: st.peak_ready_bytes as f64 / 1048576.0,
            peak_out_bufs: st.peak_out_pool,
            peak_syms_bufs: st.peak_syms_pool,
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

fn run_arm(arm: &str, data: Vec<u8>, w: usize) -> (f64, f64, f64, i64, f64, u64, f64, usize, usize) {
    let k = Kernels::detect();
    let r0 = rusage();
    let t0 = Instant::now();
    let mut h = AbHash::new();
    let (mut pr, mut po, mut ps) = (0.0, 0, 0);
    match arm {
        "MasterA" | "MasterB" => {
            let mut rdr = BzReader::parallel_workers(Cursor::new(data), w, 4 * w, k, true);
            let mut buf = vec![0u8; 4 << 20];
            loop { let n = rdr.read(&mut buf).unwrap(); if n == 0 { break; } h.update(&buf[..n]); }
            drop(rdr);
        }
        "W0_C37_4W" => {
            let cfg = safe_c38::WinCfg { slots: 4 * w, immed_syms_return: false, max_ready_bytes: 0, direct_inplace_rle: true };
            let st = safe_c38::decode_c38(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            pr = st.peak_ready_mib; po = st.peak_out_bufs; ps = st.peak_syms_bufs;
        }
        "W1_ImmedSyms" => {
            let cfg = safe_c38::WinCfg { slots: 4 * w, immed_syms_return: true, max_ready_bytes: 0, direct_inplace_rle: true };
            let st = safe_c38::decode_c38(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            pr = st.peak_ready_mib; po = st.peak_out_bufs; ps = st.peak_syms_bufs;
        }
        "W13a_Cap8M" => {
            let cfg = safe_c38::WinCfg { slots: 4 * w, immed_syms_return: true, max_ready_bytes: 8 << 20, direct_inplace_rle: true };
            let st = safe_c38::decode_c38(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            pr = st.peak_ready_mib; po = st.peak_out_bufs; ps = st.peak_syms_bufs;
        }
        "W13b_Cap12M" => {
            let cfg = safe_c38::WinCfg { slots: 4 * w, immed_syms_return: true, max_ready_bytes: 12 << 20, direct_inplace_rle: true };
            let st = safe_c38::decode_c38(Cursor::new(data), w, cfg, k, |c| h.update(c)).unwrap();
            pr = st.peak_ready_mib; po = st.peak_out_bufs; ps = st.peak_syms_bufs;
        }
        _ => panic!("unknown arm {arm}"),
    }
    let wall = t0.elapsed().as_secs_f64() * 1e3;
    let r1 = rusage();
    (wall, tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
     r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0, h.finish(), pr, po, ps)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let w: usize = args[4].parse().unwrap();
        let (wall, u, s, flt, rss, h, pr, po, ps) = run_arm(&args[3], data, w);
        println!("{wall:.3} {u:.3} {s:.3} {flt} {rss:.2} {h:016x} {pr:.2} {po} {ps}");
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let arms = ["MasterA", "MasterB", "W0_C37_4W", "W1_ImmedSyms", "W13a_Cap8M", "W13b_Cap12M"];
    for (label, path) in [
        ("wikidata_431b_210m", "/tmp/wormhole_r5/wikidata_431b_210m.bz2"),
        ("hetero_60b", "/tmp/wormhole_r5/hetero_60b.bz2"),
    ] {
        println!("\n=== 12-ROUND ROTATED FRESH-PROCESS 4W A/B ({label}) ===");
        let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut metas: BTreeMap<&str, (f64, usize, usize)> = BTreeMap::new();
        let mut ref_hash = String::new();
        for r in 0..12 {
            for k in 0..arms.len() {
                let arm = arms[(r + k) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                if ref_hash.is_empty() { ref_hash = p[5].to_string(); }
                assert_eq!(p[5], ref_hash, "HASH MISMATCH {label} {arm}");
                walls.entry(arm).or_default().push(p[0].parse().unwrap());
                syss.entry(arm).or_default().push(p[2].parse().unwrap());
                flts.entry(arm).or_default().push(p[3].parse().unwrap());
                rsss.entry(arm).or_default().push(p[4].parse().unwrap());
                metas.insert(arm, (p[6].parse().unwrap(), p[7].parse().unwrap(), p[8].parse().unwrap()));
            }
        }
        let ma = &walls["MasterA"];
        println!("{:<14} {:>8} {:>24} {:>7} {:>7} {:>7} {:>8} {:>9}", "Arm", "med_ms", "ratio vs MasterA [95%]", "sys_ms", "minflt", "rss_MB", "pkRdyMB", "out/syms#");
        for &arm in &arms {
            let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = flts[arm].clone(); fv.sort();
            let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let pairs: Vec<(f64, f64)> = ma.iter().copied().zip(walls[arm].iter().copied()).collect();
            let (rat, lo, hi) = ci95_ratio(&pairs);
            let (pr, po, ps) = metas[arm];
            println!("{:<14} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>7.2} {:>7} {:>7.2} {:>8.2} {:>4}/{:<4}",
                arm, wv[6], rat, lo, hi, sv[6], fv[6], rv[6], pr, po, ps);
        }
    }

    println!("\n=== COLD-START W=10 & W=16 WINDOW L3 FOOTPRINT CHECK (wikidata_431b_210m, 3-rd median) ===");
    for &w in &[10usize, 16] {
        println!("\n--- W = {w} ---");
        println!("{:<14} {:>8} {:>8} {:>7} {:>7} {:>7} {:>8} {:>9}", "Arm", "wall_ms", "user_ms", "sys_ms", "minflt", "rss_MB", "pkRdyMB", "out/syms#");
        for &arm in &arms {
            let mut rows = Vec::new();
            for _ in 0..3 {
                let out = Command::new(&exe).args(["--child", "/tmp/wormhole_r5/wikidata_431b_210m.bz2", arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                rows.push((p[0].parse::<f64>().unwrap(), p[1].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                           p[3].parse::<i64>().unwrap(), p[4].parse::<f64>().unwrap(),
                           p[6].parse::<f64>().unwrap(), p[7].parse::<usize>().unwrap(), p[8].parse::<usize>().unwrap()));
            }
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let m = rows[1];
            println!("{:<14} {:>8.2} {:>8.2} {:>7.2} {:>7} {:>7.2} {:>8.2} {:>4}/{:<4}",
                arm, m.0, m.1, m.2, m.3, m.4, m.5, m.6, m.7);
        }
    }
}
