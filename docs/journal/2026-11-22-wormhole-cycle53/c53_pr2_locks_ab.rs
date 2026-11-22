use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BlockSpan, Error, ErrorKind, Kernels, Scratch, StreamDecoder};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, Source};

// ---- Cycle 53: Charge 3 — Residual Lock, notify_all, & L1_AtomicFast Audit on 3296195e3 ----
mod safe_c53 {
    #![forbid(unsafe_code)]
    use super::*;

    pub struct AbHash { pub bytes: u64, pub h: u64 }
    impl AbHash {
        pub fn new() -> Self { Self { bytes: 0, h: 0xcbf29ce484222325 } }
        #[inline]
        pub fn feed(&mut self, s: &[u8]) {
            self.bytes += s.len() as u64;
            let mut h = self.h;
            let mut ch = s.chunks_exact(8);
            for c in &mut ch { h = (h ^ u64::from_le_bytes(c.try_into().unwrap())).wrapping_mul(0x100000001b3); }
            for &b in ch.remainder() { h = (h ^ b as u64).wrapping_mul(0x100000001b3); }
            self.h = h;
        }
    }

    #[derive(Default)]
    pub struct LockCounters {
        pub shared_locks: AtomicUsize,
        pub job_locks: AtomicUsize,
        pub notify_one: AtomicUsize,
        pub notify_all: AtomicUsize,
        pub bp_waits: AtomicUsize,
        pub l1_fast_hits: AtomicUsize,
    }

    struct PooledSrc {
        rx: mpsc::Receiver<Vec<u8>>, tx_free: mpsc::Sender<Vec<u8>>,
        buf: Vec<u8>, base: u64, released: u64, eof: bool, stopped: Arc<AtomicBool>,
    }
    impl Source for PooledSrc {
        fn view(&mut self, from: u64, want: usize) -> Result<(&[u8], bool), ErrorKind> {
            const COMPACT: usize = 4 << 20;
            while !self.eof && self.base + (self.buf.len() as u64) < from + want as u64 {
                match self.rx.recv() {
                    Ok(mut chunk) => {
                        let drop_n = (self.released - self.base) as usize;
                        if drop_n == self.buf.len() {
                            std::mem::swap(&mut self.buf, &mut chunk);
                            self.base = self.released;
                        } else {
                            if drop_n >= COMPACT { self.buf.drain(..drop_n); self.base = self.released; }
                            self.buf.extend_from_slice(&chunk);
                        }
                        if chunk.capacity() > 0 { chunk.clear(); let _ = self.tx_free.send(chunk); }
                    }
                    Err(_) => self.eof = true,
                }
                if self.stopped.load(Ordering::Relaxed) { return Err(ErrorKind::Cancelled); }
            }
            let off = (from - self.base) as usize;
            Ok((self.buf.get(off..).unwrap_or(&[]), self.eof))
        }
        fn release(&mut self, upto: u64) { self.released = self.released.max(upto); }
    }

    /// Streaming pipeline matching #739's Ready ring + 12 MiB cap + Jobs VecDeque,
    /// toggling: (a) redundant locks removed (1 shared.lock/blk in scanner + atomic stopped),
    /// (b) L1_AtomicFast in scanner (plus syms pool on jobs mutex so scanner skips shared.lock when not full!)
    pub fn run_stream_pr2(
        data: &[u8], workers: usize, k: Kernels,
        l1_atomic_stream: bool, count_locks: bool,
    ) -> ((u64, u64), (usize, usize, usize, usize, usize, usize)) {
        const READY_CAP: usize = 12 << 20;
        const POOLED_FROM: usize = 64 << 10;
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let floor = workers + 2;

        let cnt = Arc::new(LockCounters::default());
        let stopped = Arc::new(AtomicBool::new(false));
        let taken_a = Arc::new(AtomicUsize::new(0));
        let rbytes_a = Arc::new(AtomicUsize::new(0));

        let (tx_in, rx_in) = mpsc::sync_channel::<Vec<u8>>(2);
        let (tx_f, rx_f) = mpsc::channel::<Vec<u8>>();
        for _ in 0..3 { let _ = tx_f.send(Vec::with_capacity(4 << 20)); }

        type Job = (u64, u64, mibz2::block::Symbols);
        struct Jq { q: VecDeque<Job>, closed: bool, waiting: usize, free_syms: Vec<Vec<u16>> }
        let jobs = Arc::new((Mutex::new(Jq { q: VecDeque::with_capacity(slots), closed: false, waiting: 0, free_syms: Vec::with_capacity(slots) }), Condvar::new()));

        struct Ready {
            ring: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize,
            taken: usize, end: Option<usize>, ready_bytes: usize,
            rw: bool, sw: bool, free_out: Vec<Vec<u8>>,
        }
        let ready = Arc::new(Mutex::new(Ready {
            ring: (0..cap).map(|_| None).collect(), mask: cap - 1,
            taken: 0, end: None, ready_bytes: 0, rw: false, sw: false, free_out: Vec::with_capacity(slots),
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_s = Arc::new(Condvar::new());
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            let (txi, sf) = (tx_in.clone(), stopped.clone());
            s.spawn(move || {
                for c in data.chunks(4 << 20) {
                    if sf.load(Ordering::Relaxed) { return; }
                    let mut v = rx_f.try_recv().unwrap_or_else(|_| Vec::with_capacity(c.len()));
                    v.clear(); v.extend_from_slice(c);
                    if txi.send(v).is_err() { return; }
                }
            });
            drop(tx_in);

            let (js, rs, cvr, cvs, stp, ta, rba, cs) = (
                jobs.clone(), ready.clone(), cv_r.clone(), cv_s.clone(),
                stopped.clone(), taken_a.clone(), rbytes_a.clone(), cnt.clone(),
            );
            s.spawn(move || {
                let src = PooledSrc { rx: rx_in, tx_free: tx_f, buf: Vec::new(), base: 0, released: 0, eof: false, stopped: stp.clone() };
                let mut sc = Scanner::new(src);
                let mut issued = 0usize; // Scanner owns `issued` exclusively -> 0 lock needed to bump!
                loop {
                    let need_wait = if l1_atomic_stream {
                        let t = ta.load(Ordering::Acquire);
                        let rb = rba.load(Ordering::Relaxed);
                        let ahead = issued - t;
                        if ahead < slots && (rb < READY_CAP || ahead < floor) {
                            if count_locks { cs.l1_fast_hits.fetch_add(1, Ordering::Relaxed); }
                            false
                        } else { true }
                    } else { true };

                    if need_wait {
                        if count_locks { cs.shared_locks.fetch_add(1, Ordering::Relaxed); }
                        let mut st = rs.lock().unwrap();
                        while !stp.load(Ordering::Relaxed) && (
                            issued - st.taken >= slots
                            || (st.ready_bytes >= READY_CAP && issued - st.taken >= floor)
                        ) {
                            if count_locks { cs.bp_waits.fetch_add(1, Ordering::Relaxed); }
                            st.sw = true; st = cvs.wait(st).unwrap();
                        }
                        st.sw = false;
                        if stp.load(Ordering::Relaxed) { break; }
                    }
                    let syms = {
                        if count_locks { cs.job_locks.fetch_add(1, Ordering::Relaxed); }
                        js.0.lock().unwrap().free_syms.pop().unwrap_or_default()
                    };
                    match sc.next(syms) {
                        Ok(Event::Block { bit, symbols, .. }) => {
                            let place = issued as u64; issued += 1;
                            if count_locks { cs.job_locks.fetch_add(1, Ordering::Relaxed); }
                            let mut q = js.0.lock().unwrap();
                            q.q.push_back((place, bit, symbols));
                            let wake = q.waiting > 0;
                            drop(q);
                            if wake {
                                if count_locks { cs.notify_one.fetch_add(1, Ordering::Relaxed); }
                                js.1.notify_one();
                            }
                        }
                        Ok(Event::Done) => {
                            { let mut q = js.0.lock().unwrap(); q.closed = true; }
                            if count_locks { cs.notify_all.fetch_add(1, Ordering::Relaxed); }
                            js.1.notify_all();
                            let wake = { let mut st = rs.lock().unwrap(); st.end = Some(issued); st.rw };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                        Err(e) => {
                            { let mut q = js.0.lock().unwrap(); q.closed = true; }
                            js.1.notify_all();
                            let wake = {
                                let mut st = rs.lock().unwrap();
                                let idx = issued & st.mask;
                                st.ring[idx] = Some(Err(e)); st.end = Some(issued + 1); st.rw
                            };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                    }
                }
            });

            for _ in 0..workers {
                let (jw, rw, cvr, stp, rba, cw) = (
                    jobs.clone(), ready.clone(), cv_r.clone(), stopped.clone(), rbytes_a.clone(), cnt.clone(),
                );
                s.spawn(move || {
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        let job = {
                            if count_locks { cw.job_locks.fetch_add(1, Ordering::Relaxed); }
                            let mut q = jw.0.lock().unwrap();
                            loop {
                                if let Some(j) = q.q.pop_front() { break Some(j); }
                                if q.closed { break None; }
                                q.waiting += 1; q = jw.1.wait(q).unwrap(); q.waiting -= 1;
                            }
                        };
                        let Some((place, bit, b)) = job else { return; };
                        if stp.load(Ordering::Acquire) { continue; }
                        let mut out = if b.syms.len() >= 2048 {
                            if count_locks { cw.shared_locks.fetch_add(1, Ordering::Relaxed); }
                            let mut v = rw.lock().unwrap().free_out.pop().unwrap_or_else(|| {
                                Vec::with_capacity(b.max_len + b.max_len / 16 + 64)
                            });
                            if v.capacity() < b.max_len + b.max_len / 16 + 64 {
                                let need = (b.max_len + b.max_len / 16 + 64) - v.len();
                                v.reserve_exact(need);
                            }
                            v
                        } else { Vec::new() };
                        out.clear();
                        let res = mibz2::decode_block(&b, k, &mut scratch, &mut out)
                            .map(|_| out).map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                        {
                            if count_locks { cw.job_locks.fetch_add(1, Ordering::Relaxed); }
                            let mut q = jw.0.lock().unwrap();
                            if q.free_syms.len() < slots { q.free_syms.push(b.syms); }
                        }
                        let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                        if count_locks { cw.shared_locks.fetch_add(1, Ordering::Relaxed); }
                        let mut st = rw.lock().unwrap();
                        let idx = (place as usize) & st.mask;
                        st.ring[idx] = Some(res);
                        st.ready_bytes += bbytes;
                        rba.store(st.ready_bytes, Ordering::Relaxed);
                        let wake = st.rw && place as usize == st.taken;
                        drop(st);
                        if wake {
                            if count_locks { cw.notify_one.fetch_add(1, Ordering::Relaxed); }
                            cvr.notify_one();
                        }
                    }
                });
            }

            let mut cur: Vec<u8> = Vec::new();
            loop {
                // Zero-lock cancel check via AtomicBool!
                if stopped.load(Ordering::Relaxed) { break; }
                let spent = std::mem::take(&mut cur);
                if count_locks { cnt.shared_locks.fetch_add(1, Ordering::Relaxed); }
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if spent.capacity() >= POOLED_FROM && st.free_out.len() < slots { st.free_out.push(spent); }
                    loop {
                        let t = st.taken;
                        let idx = t & st.mask;
                        if let Some(res) = st.ring[idx].take() {
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                            st.taken = t + 1;
                            taken_a.store(st.taken, Ordering::Release);
                            rbytes_a.store(st.ready_bytes, Ordering::Relaxed);
                            let ws = st.sw; st.rw = false; drop(st);
                            if ws {
                                if count_locks { cnt.notify_one.fetch_add(1, Ordering::Relaxed); }
                                cv_s.notify_one();
                            }
                            break Some(res);
                        }
                        if st.end == Some(t) { st.rw = false; break None; }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk {
                    None => break,
                    Some(Ok(v)) => { h.feed(&v); cur = v; }
                    Some(Err(e)) => panic!("{e:?}"),
                }
            }
        });
        ((h.bytes, h.h), (
            cnt.shared_locks.load(Ordering::Relaxed),
            cnt.job_locks.load(Ordering::Relaxed),
            cnt.notify_one.load(Ordering::Relaxed),
            cnt.notify_all.load(Ordering::Relaxed),
            cnt.bp_waits.load(Ordering::Relaxed),
            cnt.l1_fast_hits.load(Ordering::Relaxed),
        ))
    }

    /// PR 4 ParScan worker pool: compares Mutex-BP vs L1_AtomicFast BP directly at 4W/10W/16W!
    pub fn run_parscan_l1_ab(
        data: &[u8], spans: &[BlockSpan], workers: usize, k: Kernels,
        use_l1_atomic: bool, count_locks: bool,
    ) -> ((u64, u64), (usize, usize, usize, usize)) {
        const READY_CAP: usize = 12 << 20;
        const POOLED_FROM: usize = 64 << 10;
        let n = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let floor = workers + 2;

        let next_idx = Arc::new(AtomicUsize::new(0));
        let taken_a = Arc::new(AtomicUsize::new(0));
        let rbytes_a = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicBool::new(false));
        let cnt = Arc::new(LockCounters::default());

        struct Ready {
            ring: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize,
            taken: usize, ready_bytes: usize, rw: bool, ww: usize, free_out: Vec<Vec<u8>>,
        }
        let ready = Arc::new(Mutex::new(Ready {
            ring: (0..cap).map(|_| None).collect(), mask: cap - 1,
            taken: 0, ready_bytes: 0, rw: false, ww: 0, free_out: Vec::with_capacity(slots),
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            for _ in 0..workers {
                let (wr, wcr, wcw, stp, wnext, ta, rba, cw) = (
                    ready.clone(), cv_r.clone(), cv_w.clone(), stopped.clone(),
                    next_idx.clone(), taken_a.clone(), rbytes_a.clone(), cnt.clone(),
                );
                s.spawn(move || {
                    let mut syms: Vec<u16> = Vec::new();
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        if stp.load(Ordering::Relaxed) { return; }
                        let place = wnext.fetch_add(1, Ordering::Relaxed);
                        if place >= n { return; }

                        let need_lock = if use_l1_atomic {
                            let t = ta.load(Ordering::Acquire);
                            let rb = rba.load(Ordering::Relaxed);
                            if place < t + slots && (rb < READY_CAP || place < t + floor) {
                                if count_locks { cw.l1_fast_hits.fetch_add(1, Ordering::Relaxed); }
                                false
                            } else { true }
                        } else { true };

                        if need_lock {
                            if count_locks { cw.shared_locks.fetch_add(1, Ordering::Relaxed); }
                            let mut st = wr.lock().unwrap();
                            while !stp.load(Ordering::Relaxed) && (
                                place >= st.taken + slots
                                || (st.ready_bytes >= READY_CAP && place >= st.taken + floor)
                            ) {
                                if count_locks { cw.bp_waits.fetch_add(1, Ordering::Relaxed); }
                                st.ww += 1; st = wcw.wait(st).unwrap(); st.ww -= 1;
                            }
                            if stp.load(Ordering::Relaxed) { return; }
                        }

                        let sp = &spans[place];
                        let mut br = Bits::at(data, sp.start_bit);
                        let res = match read_item(&mut br, sp.level, std::mem::take(&mut syms)) {
                            Ok(Item::Block(b)) => {
                                let is_full = b.syms.len() > 2048;
                                let want = b.max_len + b.max_len / 16 + 64;
                                let mut out = if is_full {
                                    if count_locks { cw.shared_locks.fetch_add(1, Ordering::Relaxed); }
                                    let mut v = wr.lock().unwrap().free_out.pop().unwrap_or_else(|| Vec::with_capacity(want));
                                    if v.capacity() < want { v.reserve_exact(want - v.len()); }
                                    v
                                } else { Vec::new() };
                                out.clear();
                                let r = mibz2::decode_block(&b, k, &mut scratch, &mut out).map(|_| out);
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((kind, _)) => Err(kind),
                        }.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(sp.start_bit) });

                        let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                        if count_locks { cw.shared_locks.fetch_add(1, Ordering::Relaxed); }
                        let mut st = wr.lock().unwrap();
                        let idx = place & st.mask;
                        st.ring[idx] = Some(res);
                        st.ready_bytes += bbytes;
                        rba.store(st.ready_bytes, Ordering::Relaxed);
                        let wake = st.rw && place == st.taken;
                        drop(st);
                        if wake {
                            if count_locks { cw.notify_one.fetch_add(1, Ordering::Relaxed); }
                            wcr.notify_one();
                        }
                    }
                });
            }

            let mut cur: Vec<u8> = Vec::new();
            loop {
                let spent = std::mem::take(&mut cur);
                if count_locks { cnt.shared_locks.fetch_add(1, Ordering::Relaxed); }
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if spent.capacity() >= POOLED_FROM && st.free_out.len() < slots { st.free_out.push(spent); }
                    loop {
                        let t = st.taken;
                        if t == n { st.rw = false; break None; }
                        let idx = t & st.mask;
                        if let Some(res) = st.ring[idx].take() {
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                            st.taken = t + 1;
                            taken_a.store(st.taken, Ordering::Release);
                            rbytes_a.store(st.ready_bytes, Ordering::Relaxed);
                            let ww = st.ww > 0; st.rw = false; drop(st);
                            if ww {
                                if count_locks { cnt.notify_one.fetch_add(1, Ordering::Relaxed); }
                                cv_w.notify_one();
                            }
                            break Some(res);
                        }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk {
                    None => break,
                    Some(Ok(v)) => { h.feed(&v); cur = v; }
                    Some(Err(e)) => panic!("{e:?}"),
                }
            }
        });
        ((h.bytes, h.h), (
            cnt.shared_locks.load(Ordering::Relaxed),
            cnt.notify_one.load(Ordering::Relaxed),
            cnt.bp_waits.load(Ordering::Relaxed),
            cnt.l1_fast_hits.load(Ordering::Relaxed),
        ))
    }
}

#[repr(C)] #[derive(Default, Clone, Copy)] struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)] #[derive(Default, Clone, Copy)] struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval, ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64, ru_inblock: i64, ru_oublock: i64,
    ru_msgsnd: i64, ru_msgrcv: i64, ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" { fn getrusage(who: i32, usage: *mut Rusage) -> i32; }
fn rusage() -> Rusage { let mut r = Rusage::default(); unsafe { getrusage(0, &mut r); } r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let t = if logs.len() >= 12 { 2.201 } else { 2.776 };
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn load_spans(path: &str) -> Vec<BlockSpan> {
    fs::read_to_string(format!("{path}.spans")).unwrap().lines().map(|l| {
        let p: Vec<u64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
        BlockSpan { index: p[0], start_bit: p[1], end_bit: p[2], crc: p[3] as u32, stream: p[4], level: p[5] as u32 }
    }).collect()
}

fn run_master_739(data: &[u8], w: usize, k: Kernels) -> (u64, u64) {
    let dec = StreamDecoder::with_inverse::<OpenCells>(w, 4 * w, k);
    let mut h = safe_c53::AbHash::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in data.chunks(4 << 20) {
                let mut v = dec.spare(); v.clear(); v.extend_from_slice(c);
                if dec.write(v).is_err() { return; }
            }
            dec.close();
        });
        let mut spent = Vec::new();
        while let Some(b) = dec.read_block(spent).unwrap() { h.feed(&b); spent = b; }
    });
    (h.bytes, h.h)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let arm = args[3].as_str();
        let w: usize = args[4].parse().unwrap();
        let spans = if arm.starts_with("Par_") { load_spans(&args[2]) } else { Vec::new() };
        let r0 = rusage(); let t0 = Instant::now();
        let mut extra = String::new();
        let (bytes, hash) = match arm {
            "M739_A" | "M739_B" => run_master_739(&data, w, k),
            "Str_TrimLock" => {
                let (bh, c) = safe_c53::run_stream_pr2(&data, w, k, false, true);
                extra = format!("shL={} jqL={} n1={} nA={} bpW={} l1H={}", c.0, c.1, c.2, c.3, c.4, c.5);
                bh
            }
            "Str_L1Atom" => {
                let (bh, c) = safe_c53::run_stream_pr2(&data, w, k, true, true);
                extra = format!("shL={} jqL={} n1={} nA={} bpW={} l1H={}", c.0, c.1, c.2, c.3, c.4, c.5);
                bh
            }
            "Par_MutexBP" => {
                let (bh, c) = safe_c53::run_parscan_l1_ab(&data, &spans, w, k, false, true);
                extra = format!("shL={} n1={} bpW={} l1H={}", c.0, c.1, c.2, c.3);
                bh
            }
            "Par_L1Atom" => {
                let (bh, c) = safe_c53::run_parscan_l1_ab(&data, &spans, w, k, true, true);
                extra = format!("shL={} n1={} bpW={} l1H={}", c.0, c.1, c.2, c.3);
                bh
            }
            _ => panic!(),
        };
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {bytes} {hash:016x} {extra}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
        return;
    }

    let exe = std::env::current_exe().unwrap();
    for &(label, path, rounds, w) in &[
        ("wikidata-l9 4W (431 blks, 210 MB)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 12usize, 4usize),
        ("wikidata-l1 4W (379 blks, 38 MB, 3x blk rate)", "/tmp/wormhole_r6/wikidata_l1_36m.bz2", 12usize, 4usize),
        ("wikidata-l9 10W (431 blks, 210 MB)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 5usize, 10usize),
        ("wikidata-l9 16W (431 blks, 210 MB)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 5usize, 16usize),
    ] {
        let arms = ["M739_A", "M739_B", "Str_TrimLock", "Str_L1Atom", "Par_MutexBP", "Par_L1Atom"];
        println!("\n=== {rounds}-ROUND ROTATED {label} ===");
        let mut ws: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut vc: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rs: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ex: BTreeMap<&str, String> = BTreeMap::new();
        let mut ref_bh: Option<(String, String)> = None;
        for r in 0..rounds {
            for i in 0..arms.len() {
                let arm = arms[(r + i) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, &w.to_string()]).output().unwrap();
                assert!(out.status.success(), "{arm}: {}", String::from_utf8_lossy(&out.stderr));
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                let bh = (p[6].to_string(), p[7].to_string());
                if let Some(ref e) = ref_bh { assert_eq!(&bh, e); } else { ref_bh = Some(bh); }
                ws.entry(arm).or_default().push(p[0].parse().unwrap());
                ss.entry(arm).or_default().push(p[2].parse().unwrap());
                vc.entry(arm).or_default().push(p[4].parse().unwrap());
                rs.entry(arm).or_default().push(p[5].parse().unwrap());
                if p.len() > 8 { ex.insert(arm, p[8..].join(" ")); }
            }
        }
        let ma = &ws["M739_A"];
        let pm = &ws["Par_MutexBP"];
        println!("{:<14} {:>7} {:>24} {:>6} {:>5} {:>6}  {}", "Arm", "med_ms", "vs Ref [95% CI]", "sys", "vcsw", "rssMB", "LockCounters");
        for &arm in &arms {
            let mut wv = ws[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = ss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut cv = vc[arm].clone(); cv.sort();
            let mut rv = rs[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let base = if arm == "Par_L1Atom" { pm } else { ma };
            let (rat, lo, hi) = ci95_ratio(&base.iter().copied().zip(ws[arm].iter().copied()).collect::<Vec<_>>());
            let mid = rounds / 2;
            println!("{:<14} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>6.2} {:>5} {:>6.2}  {}",
                arm, wv[mid], rat, lo, hi, sv[mid], cv[mid], rv[mid], ex.get(arm).map(String::as_str).unwrap_or("-"));
        }
    }
}
