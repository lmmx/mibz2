use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BlockSpan, BzReader, Error, ErrorKind, Kernels, Scratch};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::stream::{Event, Scanner, Source};

// ---- Cycle 48: Multi-Regime Stress Verification (Level-Adaptive out_cap = max_len + max_len/16) ----
mod safe_c48 {
    #![forbid(unsafe_code)]
    use super::*;

    /// Level-adaptive exact-fit out buffer reservation:
    ///   - Level 1 (max_len=100,000): 106,250 B (instead of wasteful 960 KiB!)
    ///   - Level 9 (max_len=900,000): 956,250 B (eliminates 900K->1.8M doubling!)
    #[inline]
    pub fn exact_out_cap(max_len: usize) -> usize {
        max_len + (max_len / 16).max(4096)
    }

    #[inline]
    pub fn pool_min_cap(max_len: usize) -> usize {
        (max_len / 4).max(16_384)
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
                let ls = j + 5; let ll = se - ls;
                if ll > 0 { de -= ll; if de != ls { buf.copy_within(ls..se, de); } }
                let rl = 4 + extra as usize;
                de -= rl; buf[de..de + rl].fill(c); se = j;
            }
            return Ok(mibz2::crc::crc32(buf, k));
        }
        let shift = max_ov.max(0) as usize;
        let wl = (n + shift).max(final_len);
        if wl > buf.len() { buf.resize(wl, 0); }
        if shift > 0 { buf.copy_within(r0..n, r0 + shift); }
        let (mut src, mut dst) = (r0 + shift, r0);
        for &(j, c, extra) in runs_s.iter() {
            let rs = j + shift; let ll = rs - src;
            if ll > 0 { if dst != src { buf.copy_within(src..rs, dst); } dst += ll; }
            let rl = 4 + extra as usize;
            buf[dst..dst + rl].fill(c); dst += rl; src = rs + 5;
        }
        let tl = (n + shift) - src;
        if tl > 0 { if dst != src { buf.copy_within(src..n + shift, dst); } }
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
        slots_arr: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize, taken: usize, end: Option<usize>,
        reader_waiting: bool, bp_waiting: usize, ready_bytes: usize,
        free_out: Vec<Vec<u8>>, free_syms: Vec<Vec<u16>>,
    }
    struct JobQueue { q: VecDeque<Job>, closed: bool, waiting: usize }

    pub fn decode_r1_adaptive<R: Read + Send + 'static>(
        mut r: R, workers: usize, inplace_p2: bool, k: Kernels, mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<usize, Error> {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        const CAP_BYTES: usize = 12 << 20;
        let cancelled = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0, end: None,
            reader_waiting: false, bp_waiting: 0, ready_bytes: 0,
            free_out: Vec::with_capacity(slots), free_syms: Vec::with_capacity(slots),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_bp = Arc::new(Condvar::new());
        let jq = Arc::new((Mutex::new(JobQueue { q: VecDeque::with_capacity(slots), closed: false, waiting: 0 }), Condvar::new()));
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

        let (sr, scr, scb, sc, sjq) = (ready.clone(), cv_reader.clone(), cv_bp.clone(), cancelled.clone(), jq.clone());
        let scanner_h = std::thread::spawn(move || {
            let src = PooledSource { rx: in_rx, tx_free: free_tx, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: sc.clone() };
            let mut scnr = Scanner::new(src);
            let mut issued = 0usize;
            let finish = |err: Option<Error>, issued: usize| {
                { let mut q = sjq.0.lock().unwrap(); q.closed = true; }
                sjq.1.notify_all();
                let mut st = sr.lock().unwrap();
                match err {
                    Some(e) if e.kind != ErrorKind::Cancelled => {
                        let idx = issued & st.mask; st.slots_arr[idx] = Some(Err(e)); st.end = Some(issued + 1);
                    }
                    _ => st.end = Some(issued),
                }
                drop(st); scr.notify_one();
            };
            loop {
                let syms = {
                    let mut st = sr.lock().unwrap();
                    while !sc.load(Ordering::Relaxed) && (
                        issued - st.taken >= slots ||
                        (inplace_p2 && st.ready_bytes >= CAP_BYTES && issued - st.taken >= workers + 2)
                    ) {
                        st.bp_waiting += 1; st = scb.wait(st).unwrap(); st.bp_waiting -= 1;
                    }
                    if sc.load(Ordering::Relaxed) { drop(st); return finish(None, issued); }
                    st.free_syms.pop().unwrap_or_default()
                };
                match scnr.next(syms) {
                    Ok(Event::Block { bit, symbols, .. }) => {
                        let place = issued as u64; issued += 1;
                        let mut q = sjq.0.lock().unwrap();
                        q.q.push_back((place, bit, symbols));
                        let wake = q.waiting > 0; drop(q);
                        if wake { sjq.1.notify_one(); }
                    }
                    Ok(Event::Done) => return finish(None, issued),
                    Err(e) => return finish(Some(e), issued),
                }
            }
        });

        let mut whs = Vec::with_capacity(workers);
        for _ in 0..workers {
            let (wr, wcr, wc, wjq) = (ready.clone(), cv_reader.clone(), cancelled.clone(), jq.clone());
            whs.push(std::thread::spawn(move || {
                let mut open = OpenCells::default();
                let mut runs = mibz2::runs::Runs::default();
                let mut fallback = Scratch::<OpenCells>::default();
                let mut rle_runs = Vec::with_capacity(256);
                loop {
                    let job = {
                        let mut q = wjq.0.lock().unwrap();
                        loop {
                            if let Some(j) = q.q.pop_front() { break Some(j); }
                            if q.closed { break None; }
                            q.waiting += 1; q = wjq.1.wait(q).unwrap(); q.waiting -= 1;
                        }
                    };
                    let Some((place, bit, mut symbols)) = job else { return; };
                    if wc.load(Ordering::Relaxed) { continue; }
                    let want_cap = exact_out_cap(symbols.max_len);
                    let is_pool = symbols.syms.len() > 2048;
                    let mut syms_ret = false;
                    let res = if inplace_p2 && open.reads_runs() && mibz2::runs::takes_runs(&symbols) {
                        match mibz2::runs::front(&symbols, &mut runs, k) {
                            Err(e) => Err(e),
                            Ok(n) if symbols.orig_ptr as usize >= n => Err(ErrorKind::BadOrigPtr),
                            Ok(n) => {
                                let mut out = {
                                    let mut st = wr.lock().unwrap();
                                    let s = std::mem::take(&mut symbols.syms);
                                    if st.free_syms.len() < slots { st.free_syms.push(s); }
                                    syms_ret = true;
                                    if is_pool {
                                        let mut v = st.free_out.pop().unwrap_or_default();
                                        if v.capacity() < want_cap { let need = want_cap - v.len(); v.reserve_exact(need); }
                                        v
                                    } else { Vec::new() }
                                };
                                if open.invert_runs(&runs, n, symbols.orig_ptr, &mut out, k) {
                                    match unrle_in_place(&mut out, &mut rle_runs, k) {
                                        Ok(c) if c == symbols.crc => Ok(out),
                                        Ok(computed) => Err(ErrorKind::BlockCrc { stored: symbols.crc, computed }),
                                        Err(e) => Err(e),
                                    }
                                } else { Err(ErrorKind::BadOrigPtr) }
                            }
                        }
                    } else {
                        let mut out = if is_pool {
                            let mut v = wr.lock().unwrap().free_out.pop().unwrap_or_default();
                            if v.capacity() < want_cap { let need = want_cap - v.len(); v.reserve_exact(need); }
                            v
                        } else { Vec::new() };
                        out.clear();
                        mibz2::decode_block(&symbols, k, &mut fallback, &mut out).map(|_| out)
                    };
                    let res = res.map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                    let out_b = res.as_ref().map_or(0, |v| v.len());
                    let mut st = wr.lock().unwrap();
                    if !wc.load(Ordering::Relaxed) {
                        let idx = (place as usize) & st.mask;
                        st.slots_arr[idx] = Some(res);
                        st.ready_bytes += out_b;
                    }
                    if !syms_ret && st.free_syms.len() < slots { st.free_syms.push(symbols.syms); }
                    let wake = st.reader_waiting && place as usize == st.taken;
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
                if spent.capacity() >= 16_384 && st.free_out.len() < slots { st.free_out.push(spent); }
                loop {
                    let t = st.taken;
                    let idx = t & st.mask;
                    if let Some(res) = st.slots_arr[idx].take() {
                        st.ready_bytes = st.ready_bytes.saturating_sub(res.as_ref().map_or(0, |v| v.len()));
                        st.taken = t + 1;
                        let wake_s = st.bp_waiting > 0; st.reader_waiting = false;
                        drop(st);
                        if wake_s { cv_bp.notify_one(); }
                        break Some(res);
                    }
                    if st.end == Some(st.taken) { st.reader_waiting = false; break None; }
                    st.reader_waiting = true; st = cv_reader.wait(st).unwrap();
                }
            };
            match blk {
                None => break,
                Some(Ok(v)) => { on_bytes(&v); total += v.len(); cur = v; }
                Some(Err(e)) => {
                    cancelled.store(true, Ordering::Release);
                    cv_bp.notify_all(); jq.1.notify_all();
                    let _ = feeder.join(); let _ = scanner_h.join();
                    for h in whs { let _ = h.join(); }
                    return Err(e);
                }
            }
        }
        let _ = feeder.join(); let _ = scanner_h.join();
        for h in whs { let _ = h.join(); }
        Ok(total)
    }

    pub fn decode_r2_adaptive(
        data: &[u8], spans: &[BlockSpan], workers: usize, k: Kernels, mut on_bytes: impl FnMut(&[u8]),
    ) -> Result<usize, Error> {
        let n_blocks = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        const CAP_BYTES: usize = 12 << 20;
        let next_idx = Arc::new(AtomicUsize::new(0));
        let taken_at = Arc::new(AtomicUsize::new(0));
        let rbytes_at = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(Mutex::new(ReadyRing {
            slots_arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0, end: None,
            reader_waiting: false, bp_waiting: 0, ready_bytes: 0,
            free_out: Vec::with_capacity(slots), free_syms: Vec::new(),
        }));
        let cv_reader = Arc::new(Condvar::new());
        let cv_bp = Arc::new(Condvar::new());

        std::thread::scope(|s| {
            for _ in 0..workers {
                let (wr, wcr, wcb, wc, wnext, wt, wrb) = (
                    ready.clone(), cv_reader.clone(), cv_bp.clone(), cancelled.clone(),
                    next_idx.clone(), taken_at.clone(), rbytes_at.clone(),
                );
                s.spawn(move || {
                    let mut syms: Vec<u16> = Vec::new();
                    let mut open = OpenCells::default();
                    let mut runs = mibz2::runs::Runs::default();
                    let mut fallback = Scratch::<OpenCells>::default();
                    let mut rle_runs = Vec::with_capacity(256);
                    loop {
                        if wc.load(Ordering::Relaxed) { return; }
                        let place = wnext.fetch_add(1, Ordering::Relaxed);
                        if place >= n_blocks { return; }
                        let sp = &spans[place];
                        let t = wt.load(Ordering::Acquire);
                        let rb = wrb.load(Ordering::Relaxed);
                        if place >= t + slots || (rb >= CAP_BYTES && place >= t + workers + 2) {
                            let mut st = wr.lock().unwrap();
                            while !wc.load(Ordering::Relaxed) && (
                                place >= st.taken + slots ||
                                (st.ready_bytes >= CAP_BYTES && place >= st.taken + workers + 2)
                            ) {
                                st.bp_waiting += 1; st = wcb.wait(st).unwrap(); st.bp_waiting -= 1;
                            }
                            if wc.load(Ordering::Relaxed) { return; }
                        }
                        let mut br = Bits::at(data, sp.start_bit);
                        let res = match read_item(&mut br, sp.level, std::mem::take(&mut syms)) {
                            Ok(Item::Block(b)) => {
                                let want_cap = exact_out_cap(b.max_len);
                                let is_pool = b.syms.len() > 2048;
                                let mut out = if is_pool {
                                    let mut v = wr.lock().unwrap().free_out.pop().unwrap_or_default();
                                    if v.capacity() < want_cap { let need = want_cap - v.len(); v.reserve_exact(need); }
                                    v
                                } else { Vec::new() };
                                let r = if open.reads_runs() && mibz2::runs::takes_runs(&b) {
                                    match mibz2::runs::front(&b, &mut runs, k) {
                                        Err(e) => Err(e),
                                        Ok(n) if b.orig_ptr as usize >= n => Err(ErrorKind::BadOrigPtr),
                                        Ok(n) => {
                                            if open.invert_runs(&runs, n, b.orig_ptr, &mut out, k) {
                                                match unrle_in_place(&mut out, &mut rle_runs, k) {
                                                    Ok(c) if c == b.crc => Ok(out),
                                                    Ok(computed) => Err(ErrorKind::BlockCrc { stored: b.crc, computed }),
                                                    Err(e) => Err(e),
                                                }
                                            } else { out.clear(); mibz2::decode_block(&b, k, &mut fallback, &mut out).map(|_| out) }
                                        }
                                    }
                                } else {
                                    out.clear(); mibz2::decode_block(&b, k, &mut fallback, &mut out).map(|_| out)
                                };
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((kind, _)) => Err(kind),
                        }.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(sp.start_bit) });
                        let out_b = res.as_ref().map_or(0, |v| v.len());
                        let mut st = wr.lock().unwrap();
                        if !wc.load(Ordering::Relaxed) {
                            let idx = place & st.mask; st.slots_arr[idx] = Some(res);
                            st.ready_bytes += out_b; wrb.store(st.ready_bytes, Ordering::Relaxed);
                        }
                        let wake = st.reader_waiting && place == st.taken; drop(st);
                        if wake { wcr.notify_one(); }
                    }
                });
            }
            let mut total = 0usize; let mut cur: Vec<u8> = Vec::new();
            loop {
                let blk = {
                    let mut st = ready.lock().unwrap();
                    let spent = std::mem::take(&mut cur);
                    if spent.capacity() >= 16_384 && st.free_out.len() < slots { st.free_out.push(spent); }
                    loop {
                        let t = st.taken;
                        if t == n_blocks { st.reader_waiting = false; break None; }
                        let idx = t & st.mask;
                        if let Some(res) = st.slots_arr[idx].take() {
                            st.ready_bytes = st.ready_bytes.saturating_sub(res.as_ref().map_or(0, |v| v.len()));
                            st.taken = t + 1;
                            taken_at.store(t + 1, Ordering::Release);
                            rbytes_at.store(st.ready_bytes, Ordering::Relaxed);
                            let wake_w = st.bp_waiting > 0; st.reader_waiting = false; drop(st);
                            if wake_w { cv_bp.notify_one(); }
                            break Some(res);
                        }
                        st.reader_waiting = true; st = cv_reader.wait(st).unwrap();
                    }
                };
                match blk {
                    None => return Ok(total),
                    Some(Ok(v)) => { on_bytes(&v); total += v.len(); cur = v; }
                    Some(Err(e)) => { cancelled.store(true, Ordering::Release); cv_bp.notify_all(); return Err(e); }
                }
            }
        })
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

struct AbHash { h: [u64; 4], tail: [u8; 32], nt: usize, n: u64 }
impl AbHash {
    fn new() -> Self { Self { h: [1, 2, 3, 4], tail: [0; 32], nt: 0, n: 0 } }
    #[inline] fn block(&mut self, b: &[u8; 32]) {
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
        let rem = c.remainder(); self.tail[..rem.len()].copy_from_slice(rem); self.nt = rem.len();
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

fn load_spans(path: &str) -> Vec<BlockSpan> {
    let sp = format!("{path}.spans");
    if let Ok(txt) = fs::read_to_string(&sp) {
        return txt.lines().map(|l| {
            let p: Vec<u64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
            BlockSpan { index: p[0], start_bit: p[1], end_bit: p[2], crc: p[3] as u32, stream: p[4], level: p[5] as u32 }
        }).collect();
    }
    let data = fs::read(path).unwrap();
    let spans: Vec<BlockSpan> = mibz2::scan(&data).map(|r| r.unwrap()).collect();
    let mut s = String::new();
    for b in &spans { s.push_str(&format!("{} {} {} {} {} {}\n", b.index, b.start_bit, b.end_bit, b.crc, b.stream, b.level)); }
    fs::write(&sp, s).unwrap();
    spans
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--prep") {
        let _ = load_spans(&args[2]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let spans = load_spans(&args[2]);
        let arm = &args[3];
        let r0 = rusage(); let t0 = Instant::now();
        let mut h = AbHash::new();
        match arm.as_str() {
            "MasterA" | "MasterB" => {
                let mut rdr = BzReader::parallel_workers(Cursor::new(data), 4, 16, k, true);
                let mut buf = vec![0u8; 4 << 20];
                loop { let n = rdr.read(&mut buf).unwrap(); if n == 0 { break; } h.update(&buf[..n]); }
                drop(rdr);
            }
            "R1_P1W0" => { safe_c48::decode_r1_adaptive(Cursor::new(data), 4, false, k, |c| h.update(c)).unwrap(); }
            "R1_P2" => { safe_c48::decode_r1_adaptive(Cursor::new(data), 4, true, k, |c| h.update(c)).unwrap(); }
            "R2_Par_P2" => { safe_c48::decode_r2_adaptive(&data, &spans, 4, k, |c| h.update(c)).unwrap(); }
            _ => panic!(),
        }
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {:.2} {:016x}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_maxrss as f64 / 1024.0, h.finish());
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let corpora = [
        ("wikidata-l9 (211 MB, 431 blks)", "/tmp/wormhole_r5/wikidata_431b_210m.bz2"),
        ("wikidata-l1 (36 MB, 360 blks)", "/tmp/wormhole_r5/wikidata_l1_36m.bz2"),
        ("hetero-l9 (54 MB, 64 blks)", "/tmp/wormhole_r5/hetero_60b.bz2"),
        ("random-l9 (11 MB, 12 blks)", "/tmp/wormhole_r5/random_l9_11m.bz2"),
    ];
    let arms = ["MasterA", "MasterB", "R1_P1W0", "R1_P2", "R2_Par_P2"];
    for &(label, path) in &corpora {
        assert!(Command::new(&exe).args(["--prep", path]).status().unwrap().success());
        println!("\n=== 12-ROUND 4W MULTI-REGIME GATE: {label} ===");
        let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ref_h = String::new();
        for r in 0..12 {
            for i in 0..arms.len() {
                let arm = arms[(r + i) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                if ref_h.is_empty() { ref_h = p[5].to_string(); }
                assert_eq!(p[5], ref_h, "HASH MISMATCH {label} {arm}");
                walls.entry(arm).or_default().push(p[0].parse().unwrap());
                syss.entry(arm).or_default().push(p[2].parse().unwrap());
                flts.entry(arm).or_default().push(p[3].parse().unwrap());
                rsss.entry(arm).or_default().push(p[4].parse().unwrap());
            }
        }
        let ma = &walls["MasterA"];
        println!("{:<12} {:>8} {:>24} {:>7} {:>7} {:>7}", "Arm", "med_ms", "vs MasterA [95%]", "sys_ms", "minflt", "rss_MB");
        for &arm in &arms {
            let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = flts[arm].clone(); fv.sort();
            let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
            println!("{:<12} {:>8.2} {:>6.4} [{:.4}, {:.4}] {:>7.2} {:>7} {:>7.2}", arm, wv[6], rat, lo, hi, sv[6], fv[6], rv[6]);
        }
    }
}
