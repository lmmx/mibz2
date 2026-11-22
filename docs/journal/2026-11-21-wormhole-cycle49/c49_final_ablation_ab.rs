use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Instant;
use mibz2::{BlockSpan, Error, ErrorKind, Kernels, Scratch};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::stream::{Event, Scanner, Source};

// ---- Cycle 49: Final 6-Step Cumulative Ablation & Before/After Timeline Verification ----
mod safe_c49 {
    #![forbid(unsafe_code)]
    use super::*;

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

    pub struct AbHash { pub bytes: u64, pub h: u64 }
    impl AbHash {
        pub fn new() -> Self { Self { bytes: 0, h: 0xcbf29ce484222325 } }
        #[inline]
        pub fn feed(&mut self, s: &[u8]) {
            self.bytes += s.len() as u64;
            let mut h = self.h;
            let mut chunks = s.chunks_exact(8);
            for c in &mut chunks {
                h = (h ^ u64::from_le_bytes(c.try_into().unwrap())).wrapping_mul(0x100000001b3);
            }
            for &b in chunks.remainder() { h = (h ^ b as u64).wrapping_mul(0x100000001b3); }
            self.h = h;
        }
    }

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
                if self.cancelled.load(Ordering::Relaxed) { return Err(ErrorKind::Truncated); }
            }
            let off = (from - self.base) as usize;
            Ok((self.buf.get(off..).unwrap_or(&[]), self.eof))
        }
        fn release(&mut self, up_to: u64) { if up_to > self.released { self.released = up_to; } }
    }

    fn decode_block_inplace(
        b: &mibz2::block::Symbols, k: Kernels,
        inv: &mut OpenCells, runs: &mut mibz2::runs::Runs, runs_s: &mut Vec<(usize, u8, u8)>,
        fallback: &mut Scratch<OpenCells>, out: &mut Vec<u8>,
    ) -> Result<(), ErrorKind> {
        if inv.reads_runs() && mibz2::runs::takes_runs(b) {
            let n = mibz2::runs::front(b, runs, k)?;
            if b.orig_ptr as usize >= n { return Err(ErrorKind::BadOrigPtr); }
            let want = exact_out_cap(b.max_len);
            if out.capacity() < want { out.reserve_exact(want - out.len()); }
            if !inv.invert_runs(runs, n, b.orig_ptr, out, k) { return Err(ErrorKind::BadOrigPtr); }
            let crc = unrle_in_place(out, runs_s, k)?;
            if crc != b.crc { return Err(ErrorKind::BlockCrc { stored: b.crc, computed: crc }); }
            Ok(())
        } else {
            out.clear();
            mibz2::decode_block(b, k, fallback, out).map(|_| ())
        }
    }

    /// Step 1 (`S1_AllocFeed`): C32 A1 (exact_out_cap) + A2 (1 MiB feeder pool) on v0.5.12 BTreeMap + notify_all + copying reader
    pub fn run_s1_alloc_feed(data: &[u8], workers: usize, k: Kernels) -> (u64, u64) {
        let slots = (4 * workers).max(workers);
        let (tx_in, rx_in) = mpsc::sync_channel::<Vec<u8>>(4);
        let (tx_f, rx_f) = mpsc::channel::<Vec<u8>>();
        for _ in 0..6 { let _ = tx_f.send(Vec::with_capacity(1 << 20)); }
        type Job = (u64, u64, mibz2::block::Symbols);
        let (tx_j, rx_j) = mpsc::sync_channel::<Job>(slots);
        let rx_j = Arc::new(Mutex::new(rx_j));
        let cancelled = Arc::new(AtomicBool::new(false));
        struct State { ready: BTreeMap<u64, Result<Vec<u8>, Error>>, next: u64, taken: u64, eof: bool, free_out: Vec<Vec<u8>>, free_syms: Vec<Vec<u16>> }
        let mu = Arc::new(Mutex::new(State { ready: BTreeMap::new(), next: 0, taken: 0, eof: false, free_out: Vec::new(), free_syms: Vec::new() }));
        let cv = Arc::new(Condvar::new());
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            let (m_f, c_f) = (tx_in.clone(), cancelled.clone());
            s.spawn(move || {
                for ch in data.chunks(1 << 20) {
                    if c_f.load(Ordering::Relaxed) { return; }
                    let mut v = rx_f.try_recv().unwrap_or_else(|_| Vec::with_capacity(1 << 20));
                    v.clear(); v.extend_from_slice(ch);
                    if m_f.send(v).is_err() { return; }
                }
            });
            drop(tx_in);
            let (ms, cvs, cs) = (mu.clone(), cv.clone(), cancelled.clone());
            s.spawn(move || {
                let src = PooledSource { rx: rx_in, tx_free: tx_f, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: cs.clone() };
                let mut sc = Scanner::new(src);
                loop {
                    let sym = {
                        let mut st = ms.lock().unwrap();
                        while !cs.load(Ordering::Relaxed) && st.next >= st.taken + slots as u64 { st = cvs.wait(st).unwrap(); }
                        if cs.load(Ordering::Relaxed) { return; }
                        st.free_syms.pop().unwrap_or_default()
                    };
                    match sc.next(sym) {
                        Ok(Event::Block { bit, symbols, .. }) => {
                            let place = { let mut st = ms.lock().unwrap(); let p = st.next; st.next += 1; p };
                            if tx_j.send((place, bit, symbols)).is_err() { return; }
                        }
                        Ok(Event::Done) => { let mut st = ms.lock().unwrap(); st.eof = true; cvs.notify_all(); return; }
                        Err(e) => { let mut st = ms.lock().unwrap(); let p = st.next; st.next += 1; st.ready.insert(p, Err(e)); st.eof = true; cvs.notify_all(); return; }
                    }
                }
            });
            for _ in 0..workers {
                let (rj, mw, cvw) = (rx_j.clone(), mu.clone(), cv.clone());
                s.spawn(move || {
                    let mut scratch = Scratch::<OpenCells>::default();
                    loop {
                        let Ok((place, bit, b)) = rj.lock().unwrap().recv() else { return; };
                        let want = exact_out_cap(b.max_len);
                        let mut out = {
                            let mut st = mw.lock().unwrap();
                            st.free_out.pop().unwrap_or_else(|| Vec::with_capacity(want))
                        };
                        if out.capacity() < want { out.reserve_exact(want - out.len()); }
                        out.clear();
                        let res = mibz2::decode_block(&b, k, &mut scratch, &mut out)
                            .map(|_| out).map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                        let mut st = mw.lock().unwrap();
                        if st.free_syms.len() < slots { st.free_syms.push(b.syms); }
                        st.ready.insert(place, res);
                        cvw.notify_all();
                    }
                });
            }
            let mut copy_buf = vec![0u8; 4 << 20];
            let mut cur: Vec<u8> = Vec::new();
            let mut at = 0usize;
            loop {
                if at == cur.len() {
                    let prev = std::mem::take(&mut cur); at = 0;
                    let blk = {
                        let mut st = mu.lock().unwrap();
                        if prev.capacity() > 0 && st.free_out.len() < slots { st.free_out.push(prev); }
                        loop {
                            let t = st.taken;
                            if let Some(res) = st.ready.remove(&t) { st.taken = t + 1; cv.notify_all(); break Some(res); }
                            if st.eof && t == st.next { break None; }
                            st = cv.wait(st).unwrap();
                        }
                    };
                    match blk { None => break, Some(Ok(v)) => cur = v, Some(Err(e)) => panic!("{e:?}") }
                }
                let n = (cur.len() - at).min(copy_buf.len());
                copy_buf[..n].copy_from_slice(&cur[at..at + n]);
                h.feed(&copy_buf[..n]);
                at += n;
            }
        });
        (h.bytes, h.h)
    }

    /// Unified Streaming Pipeline covering Steps S2 (`RingSplit`), S3 (`+BufReadZC`), S4 (`+FullPoolW0`), S5 (`+P2 InPlace + W1 + 12M cap`)
    pub fn run_stream_step(
        data: &[u8], workers: usize, k: Kernels,
        zero_copy_read: bool, full_pool_sep: bool, p2_inplace_12m: bool,
    ) -> (u64, u64) {
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let ready_cap_bytes: usize = if p2_inplace_12m { 12 << 20 } else { usize::MAX };
        let floor_slots: usize = workers + 2;

        let (tx_in, rx_in) = mpsc::sync_channel::<Vec<u8>>(4);
        let (tx_f, rx_f) = mpsc::channel::<Vec<u8>>();
        for _ in 0..6 { let _ = tx_f.send(Vec::with_capacity(1 << 20)); }
        type Job = (u64, u64, mibz2::block::Symbols);
        struct Jobs { q: VecDeque<Job>, closed: bool, free_syms: Vec<Vec<u16>> }
        let jobs = Arc::new(Mutex::new(Jobs { q: VecDeque::with_capacity(slots), closed: false, free_syms: Vec::with_capacity(slots) }));
        let cv_job = Arc::new(Condvar::new());
        let cv_job_sp = Arc::new(Condvar::new());

        struct Ready {
            arr: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize, next: u64, taken: u64,
            ready_bytes: usize, eof: bool, rw: bool, sw: bool, free_out: Vec<Vec<u8>>,
        }
        let ready = Arc::new(Mutex::new(Ready {
            arr: (0..cap).map(|_| None).collect(), mask: cap - 1, next: 0, taken: 0,
            ready_bytes: 0, eof: false, rw: false, sw: false, free_out: Vec::with_capacity(slots),
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_s = Arc::new(Condvar::new());
        let cancelled = Arc::new(AtomicBool::new(false));
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            let (m_f, c_f) = (tx_in.clone(), cancelled.clone());
            s.spawn(move || {
                for ch in data.chunks(1 << 20) {
                    if c_f.load(Ordering::Relaxed) { return; }
                    let mut v = rx_f.try_recv().unwrap_or_else(|_| Vec::with_capacity(1 << 20));
                    v.clear(); v.extend_from_slice(ch);
                    if m_f.send(v).is_err() { return; }
                }
            });
            drop(tx_in);

            let (js, cvj, cvjs, rs, cvr, cvs, cs) = (jobs.clone(), cv_job.clone(), cv_job_sp.clone(), ready.clone(), cv_r.clone(), cv_s.clone(), cancelled.clone());
            s.spawn(move || {
                let src = PooledSource { rx: rx_in, tx_free: tx_f, buf: Vec::new(), base: 0, released: 0, eof: false, cancelled: cs.clone() };
                let mut sc = Scanner::new(src);
                loop {
                    {
                        let mut st = rs.lock().unwrap();
                        while !cs.load(Ordering::Relaxed) && (
                            st.next >= st.taken + slots as u64
                            || (st.ready_bytes >= ready_cap_bytes && st.next >= st.taken + floor_slots as u64)
                        ) {
                            st.sw = true; st = cvs.wait(st).unwrap();
                        }
                        st.sw = false;
                        if cs.load(Ordering::Relaxed) { return; }
                    }
                    let sym = js.lock().unwrap().free_syms.pop().unwrap_or_default();
                    match sc.next(sym) {
                        Ok(Event::Block { bit, symbols, .. }) => {
                            let place = { let mut st = rs.lock().unwrap(); let p = st.next; st.next += 1; p };
                            let mut jq = js.lock().unwrap();
                            while !cs.load(Ordering::Relaxed) && jq.q.len() >= slots { jq = cvjs.wait(jq).unwrap(); }
                            if cs.load(Ordering::Relaxed) { return; }
                            jq.q.push_back((place, bit, symbols));
                            drop(jq);
                            cvj.notify_one();
                        }
                        Ok(Event::Done) => {
                            { let mut jq = js.lock().unwrap(); jq.closed = true; } cvj.notify_all();
                            let wake = { let mut st = rs.lock().unwrap(); st.eof = true; st.rw };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                        Err(e) => {
                            { let mut jq = js.lock().unwrap(); jq.closed = true; } cvj.notify_all();
                            let wake = {
                                let mut st = rs.lock().unwrap();
                                let p = st.next; st.next += 1;
                                let idx = (p as usize) & st.mask;
                                st.arr[idx] = Some(Err(e)); st.eof = true; st.rw
                            };
                            if wake { cvr.notify_one(); }
                            return;
                        }
                    }
                }
            });

            for _ in 0..workers {
                let (jw, cvj, cvjs, rw, cvr, cw) = (jobs.clone(), cv_job.clone(), cv_job_sp.clone(), ready.clone(), cv_r.clone(), cancelled.clone());
                s.spawn(move || {
                    let mut scratch = Scratch::<OpenCells>::default();
                    let mut inv = OpenCells::default();
                    let mut runs = mibz2::runs::Runs::default();
                    let mut runs_s: Vec<(usize, u8, u8)> = Vec::new();
                    loop {
                        let job = {
                            let mut jq = jw.lock().unwrap();
                            loop {
                                if cw.load(Ordering::Relaxed) { return; }
                                if let Some(j) = jq.q.pop_front() {
                                    let was_full = jq.q.len() + 1 >= slots;
                                    drop(jq);
                                    if was_full { cvjs.notify_one(); }
                                    break j;
                                }
                                if jq.closed { return; }
                                jq = cvj.wait(jq).unwrap();
                            }
                        };
                        let (place, bit, mut b) = job;
                        let is_full = b.syms.len() > 2048;
                        let want = exact_out_cap(b.max_len);
                        let mut out = if !full_pool_sep || is_full {
                            let mut v = rw.lock().unwrap().free_out.pop().unwrap_or_else(|| {
                                let mut x = Vec::new(); x.reserve_exact(want); x
                            });
                            if v.capacity() < want { v.reserve_exact(want - v.len()); }
                            v
                        } else {
                            Vec::new()
                        };
                        let res = if p2_inplace_12m && is_full && inv.reads_runs() && mibz2::runs::takes_runs(&b) {
                            match mibz2::runs::front(&b, &mut runs, k) {
                                Err(kind) => {
                                    let s = std::mem::take(&mut b.syms);
                                    { let mut jq = jw.lock().unwrap(); if jq.free_syms.len() < slots { jq.free_syms.push(s); } }
                                    Err(Error { kind, block: Some(place), bit: Some(bit) })
                                }
                                Ok(n) if b.orig_ptr as usize >= n => {
                                    let s = std::mem::take(&mut b.syms);
                                    { let mut jq = jw.lock().unwrap(); if jq.free_syms.len() < slots { jq.free_syms.push(s); } }
                                    Err(Error { kind: ErrorKind::BadOrigPtr, block: Some(place), bit: Some(bit) })
                                }
                                Ok(n) => {
                                    let crc_exp = b.crc;
                                    let p_idx = b.orig_ptr;
                                    let s = std::mem::take(&mut b.syms);
                                    { let mut jq = jw.lock().unwrap(); if jq.free_syms.len() < slots { jq.free_syms.push(s); } }
                                    if inv.invert_runs(&runs, n, p_idx, &mut out, k) {
                                        match unrle_in_place(&mut out, &mut runs_s, k) {
                                            Ok(crc) if crc == crc_exp => Ok(out),
                                            Ok(computed) => Err(Error { kind: ErrorKind::BlockCrc { stored: crc_exp, computed }, block: Some(place), bit: Some(bit) }),
                                            Err(kind) => Err(Error { kind, block: Some(place), bit: Some(bit) }),
                                        }
                                    } else {
                                        Err(Error { kind: ErrorKind::BadOrigPtr, block: Some(place), bit: Some(bit) })
                                    }
                                }
                            }
                        } else {
                            out.clear();
                            let r = mibz2::decode_block(&b, k, &mut scratch, &mut out)
                                .map(|_| out).map_err(|kind| Error { kind, block: Some(place), bit: Some(bit) });
                            { let mut jq = jw.lock().unwrap(); if jq.free_syms.len() < slots { jq.free_syms.push(b.syms); } }
                            r
                        };
                        let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                        let mut st = rw.lock().unwrap();
                        let idx = (place as usize) & st.mask;
                        st.arr[idx] = Some(res);
                        st.ready_bytes += bbytes;
                        let wake = st.rw && place == st.taken;
                        drop(st);
                        if wake { cvr.notify_one(); }
                    }
                });
            }

            let min_pool = pool_min_cap(900_000);
            let mut copy_buf = if zero_copy_read { Vec::new() } else { vec![0u8; 4 << 20] };
            let mut cur: Vec<u8> = Vec::new();
            loop {
                let prev = std::mem::take(&mut cur);
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if prev.capacity() > 0 && (!full_pool_sep || prev.capacity() >= min_pool) && st.free_out.len() < slots {
                        st.free_out.push(prev);
                    }
                    loop {
                        let t = st.taken;
                        let idx = (t as usize) & st.mask;
                        if let Some(res) = st.arr[idx].take() {
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                            st.taken = t + 1;
                            let ws = st.sw; st.rw = false; drop(st);
                            if ws { cv_s.notify_one(); }
                            break Some(res);
                        }
                        if st.eof && t == st.next { st.rw = false; break None; }
                        st.rw = true; st = cv_r.wait(st).unwrap();
                    }
                };
                match blk {
                    None => break,
                    Some(Ok(v)) => {
                        if zero_copy_read {
                            h.feed(&v);
                        } else {
                            let mut at = 0;
                            while at < v.len() {
                                let n = (v.len() - at).min(copy_buf.len());
                                copy_buf[..n].copy_from_slice(&v[at..at + n]);
                                h.feed(&copy_buf[..n]);
                                at += n;
                            }
                        }
                        cur = v;
                    }
                    Some(Err(e)) => panic!("{e:?}"),
                }
            }
        });
        (h.bytes, h.h)
    }

    #[derive(Default)]
    pub struct TimelineStats {
        pub peak_ready_blks: usize,
        pub peak_ready_bytes: usize,
        pub wake_reader: usize,
        pub wake_worker: usize,
        pub atomic_waits: usize,
        pub peak_free_out: usize,
        pub out_doublings: usize,
    }

    /// Steps S6 (`ParScan_P1W0`) & S7 (`ParScan_P2`: full WormholeScaleDecoder Regime 2 with optional timeline counters)
    pub fn run_parscan_step(
        data: &[u8], spans: &[BlockSpan], workers: usize, k: Kernels, p2_inplace_12m: bool,
    ) -> ((u64, u64), TimelineStats) {
        let n = spans.len();
        let slots = (4 * workers).max(workers);
        let cap = slots.next_power_of_two();
        let ready_cap_bytes: usize = if p2_inplace_12m { 12 << 20 } else { usize::MAX };
        let floor_slots: usize = workers + 2;

        let next_idx = Arc::new(AtomicUsize::new(0));
        let taken_a = Arc::new(AtomicUsize::new(0));
        let rbytes_a = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));
        let out_dbls = Arc::new(AtomicUsize::new(0));
        let a_waits = Arc::new(AtomicUsize::new(0));

        struct Ready {
            arr: Vec<Option<Result<Vec<u8>, Error>>>, mask: usize, taken: usize,
            ready_blks: usize, ready_bytes: usize, rw: bool, ww: usize,
            free_out: Vec<Vec<u8>>,
            pk_blks: usize, pk_bytes: usize, w_r: usize, w_w: usize, pk_free: usize,
        }
        let ready = Arc::new(Mutex::new(Ready {
            arr: (0..cap).map(|_| None).collect(), mask: cap - 1, taken: 0,
            ready_blks: 0, ready_bytes: 0, rw: false, ww: 0,
            free_out: Vec::with_capacity(slots),
            pk_blks: 0, pk_bytes: 0, w_r: 0, w_w: 0, pk_free: 0,
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let mut h = AbHash::new();

        std::thread::scope(|s| {
            for _ in 0..workers {
                let (wr, wcr, wcw, wc, wnext, wta, wrb, wdbl, waw) = (
                    ready.clone(), cv_r.clone(), cv_w.clone(), cancelled.clone(),
                    next_idx.clone(), taken_a.clone(), rbytes_a.clone(), out_dbls.clone(), a_waits.clone(),
                );
                s.spawn(move || {
                    let mut syms: Vec<u16> = Vec::new();
                    let mut scratch = Scratch::<OpenCells>::default();
                    let mut inv = OpenCells::default();
                    let mut runs = mibz2::runs::Runs::default();
                    let mut runs_s: Vec<(usize, u8, u8)> = Vec::new();
                    loop {
                        if wc.load(Ordering::Relaxed) { return; }
                        let place = wnext.fetch_add(1, Ordering::Relaxed);
                        if place >= n { return; }
                        let t_now = wta.load(Ordering::Acquire);
                        let rb_now = wrb.load(Ordering::Relaxed);
                        if place >= t_now + slots || (rb_now >= ready_cap_bytes && place >= t_now + floor_slots) {
                            let mut st = wr.lock().unwrap();
                            while !wc.load(Ordering::Relaxed) && (
                                place >= st.taken + slots
                                || (st.ready_bytes >= ready_cap_bytes && place >= st.taken + floor_slots)
                            ) {
                                waw.fetch_add(1, Ordering::Relaxed);
                                st.ww += 1; st = wcw.wait(st).unwrap(); st.ww -= 1;
                            }
                            if wc.load(Ordering::Relaxed) { return; }
                        }
                        let sp = &spans[place];
                        let mut br = Bits::at(data, sp.start_bit);
                        let res = match read_item(&mut br, sp.level, std::mem::take(&mut syms)) {
                            Ok(Item::Block(b)) => {
                                let is_full = b.syms.len() > 2048;
                                let want = exact_out_cap(b.max_len);
                                let mut out = if is_full {
                                    let mut v = wr.lock().unwrap().free_out.pop().unwrap_or_else(|| {
                                        let mut x = Vec::new(); x.reserve_exact(want); x
                                    });
                                    if v.capacity() < want { v.reserve_exact(want - v.len()); }
                                    v
                                } else { Vec::new() };
                                let cap_before = out.capacity();
                                let r = if p2_inplace_12m && is_full {
                                    decode_block_inplace(&b, k, &mut inv, &mut runs, &mut runs_s, &mut scratch, &mut out).map(|_| out)
                                } else {
                                    out.clear();
                                    mibz2::decode_block(&b, k, &mut scratch, &mut out).map(|_| out)
                                };
                                if let Ok(ref v) = r {
                                    if is_full && v.capacity() > cap_before { wdbl.fetch_add(1, Ordering::Relaxed); }
                                }
                                syms = b.syms;
                                r
                            }
                            Ok(Item::End(_)) => Err(ErrorKind::Truncated),
                            Err((kind, _)) => Err(kind),
                        }.map_err(|kind| Error { kind, block: Some(place as u64), bit: Some(sp.start_bit) });

                        let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                        let mut st = wr.lock().unwrap();
                        let idx = place & st.mask;
                        st.arr[idx] = Some(res);
                        st.ready_blks += 1;
                        st.ready_bytes += bbytes;
                        if st.ready_blks > st.pk_blks { st.pk_blks = st.ready_blks; }
                        if st.ready_bytes > st.pk_bytes { st.pk_bytes = st.ready_bytes; }
                        wrb.store(st.ready_bytes, Ordering::Relaxed);
                        let wake = st.rw && place == st.taken;
                        if wake { st.w_r += 1; }
                        drop(st);
                        if wake { wcr.notify_one(); }
                    }
                });
            }

            let min_pool = pool_min_cap(900_000);
            let mut cur: Vec<u8> = Vec::new();
            loop {
                let prev = std::mem::take(&mut cur);
                let blk = {
                    let mut st = ready.lock().unwrap();
                    if prev.capacity() >= min_pool && st.free_out.len() < slots {
                        st.free_out.push(prev);
                        if st.free_out.len() > st.pk_free { st.pk_free = st.free_out.len(); }
                    }
                    loop {
                        let t = st.taken;
                        if t == n { st.rw = false; break None; }
                        let idx = t & st.mask;
                        if let Some(res) = st.arr[idx].take() {
                            let bbytes = res.as_ref().map(|v| v.len()).unwrap_or(0);
                            st.ready_blks = st.ready_blks.saturating_sub(1);
                            st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                            st.taken = t + 1;
                            taken_a.store(st.taken, Ordering::Release);
                            rbytes_a.store(st.ready_bytes, Ordering::Relaxed);
                            let ww = st.ww > 0;
                            if ww { st.w_w += 1; }
                            st.rw = false;
                            drop(st);
                            if ww { cv_w.notify_one(); }
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
        let st = ready.lock().unwrap();
        let tl = TimelineStats {
            peak_ready_blks: st.pk_blks,
            peak_ready_bytes: st.pk_bytes,
            wake_reader: st.w_r,
            wake_worker: st.w_w,
            atomic_waits: a_waits.load(Ordering::Relaxed),
            peak_free_out: st.pk_free,
            out_doublings: out_dbls.load(Ordering::Relaxed),
        };
        ((h.bytes, h.h), tl)
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
    let t = if logs.len() >= 15 { 2.145 } else if logs.len() >= 12 { 2.201 } else { 2.776 };
    (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
}

fn load_spans(path: &str) -> Vec<BlockSpan> {
    let txt = fs::read_to_string(format!("{path}.spans")).unwrap();
    txt.lines().map(|l| {
        let p: Vec<u64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
        BlockSpan { index: p[0], start_bit: p[1], end_bit: p[2], crc: p[3] as u32, stream: p[4], level: p[5] as u32 }
    }).collect()
}

fn run_master(data: &[u8], w: usize, k: Kernels) -> (u64, u64) {
    let dec = mibz2::StreamDecoder::workers(w, 4 * w, k, true);
    let mut h = safe_c49::AbHash::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in data.chunks(4 << 20) { if dec.write(c.to_vec()).is_err() { return; } }
            dec.close();
        });
        let mut buf = vec![0u8; 4 << 20];
        loop { match dec.read(&mut buf).unwrap() { 0 => break, n => h.feed(&buf[..n]) } }
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
        let spans = if arm.starts_with("S6") || arm.starts_with("S7") { load_spans(&args[2]) } else { Vec::new() };
        let r0 = rusage(); let t0 = Instant::now();
        let mut tl_str = String::new();
        let (bytes, hash) = match arm {
            "S0_MasterA" | "S0_MasterB" => run_master(&data, w, k),
            "S1_AllocFeed" => safe_c49::run_s1_alloc_feed(&data, w, k),
            "S2_RingSplit" => safe_c49::run_stream_step(&data, w, k, false, false, false),
            "S3_BufReadZC" => safe_c49::run_stream_step(&data, w, k, true, false, false),
            "S4_FullPoolW0" => safe_c49::run_stream_step(&data, w, k, true, true, false),
            "S5_P2_InPlace12M" => safe_c49::run_stream_step(&data, w, k, true, true, true),
            "S6_ParScan_P1W0" => safe_c49::run_parscan_step(&data, &spans, w, k, false).0,
            "S7_ParScan_P2" => {
                let (bh, tl) = safe_c49::run_parscan_step(&data, &spans, w, k, true);
                tl_str = format!("pkBlks={} pkMB={:.2} wR={} wW={} aWait={} pkFree={} dbl={}",
                    tl.peak_ready_blks, tl.peak_ready_bytes as f64 / (1024.0 * 1024.0),
                    tl.wake_reader, tl.wake_worker, tl.atomic_waits, tl.peak_free_out, tl.out_doublings);
                bh
            }
            _ => panic!("unknown arm {arm}"),
        };
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {bytes} {hash:016x} {tl_str}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let path = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";

    let arms = [
        "S0_MasterA", "S0_MasterB", "S1_AllocFeed", "S2_RingSplit",
        "S3_BufReadZC", "S4_FullPoolW0", "S5_P2_InPlace12M", "S6_ParScan_P1W0", "S7_ParScan_P2",
    ];
    println!("=== PART 1: 15-ROUND ROTATED FRESH-PROCESS 4W CUMULATIVE ABLATION (211 MB, 431 BLKS) ===");
    let mut walls: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut users: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut syss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut flts: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut vcsws: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    let mut rsss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut ref_bh: Option<(String, String)> = None;

    for r in 0..15 {
        for i in 0..arms.len() {
            let arm = arms[(r + i) % arms.len()];
            let out = Command::new(&exe).args(["--child", path, arm, "4"]).output().unwrap();
            assert!(out.status.success(), "{arm} failed: {}", String::from_utf8_lossy(&out.stderr));
            let s = String::from_utf8(out.stdout).unwrap();
            let p: Vec<&str> = s.split_whitespace().collect();
            let bh = (p[6].to_string(), p[7].to_string());
            if let Some(ref exp) = ref_bh { assert_eq!(&bh, exp, "hash mismatch on {arm}"); } else { ref_bh = Some(bh); }
            walls.entry(arm).or_default().push(p[0].parse().unwrap());
            users.entry(arm).or_default().push(p[1].parse().unwrap());
            syss.entry(arm).or_default().push(p[2].parse().unwrap());
            flts.entry(arm).or_default().push(p[3].parse().unwrap());
            vcsws.entry(arm).or_default().push(p[4].parse().unwrap());
            rsss.entry(arm).or_default().push(p[5].parse().unwrap());
        }
    }
    let ma = &walls["S0_MasterA"];
    println!("{:<18} {:>7} {:>24} {:>7} {:>6} {:>7} {:>6} {:>7}", "Arm", "med_ms", "vs MasterA [95% CI]", "user", "sys", "minflt", "vcsw", "rss_MB");
    for &arm in &arms {
        let mut wv = walls[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
        let mut uv = users[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
        let mut sv = syss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
        let mut fv = flts[arm].clone(); fv.sort();
        let mut cv = vcsws[arm].clone(); cv.sort();
        let mut rv = rsss[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
        let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(walls[arm].iter().copied()).collect::<Vec<_>>());
        println!("{:<18} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>7.1} {:>6.2} {:>7} {:>6} {:>7.2}",
            arm, wv[7], rat, lo, hi, uv[7], sv[7], fv[7], cv[7], rv[7]);
    }

    let scale_arms = ["S0_MasterA", "S4_FullPoolW0", "S5_P2_InPlace12M", "S6_ParScan_P1W0", "S7_ParScan_P2"];
    for w in [7usize, 10, 16] {
        println!("\n=== PART 2: 5-ROUND COLD-START SCALING AT {w}W (211 MB, 431 BLKS) ===");
        let mut sw: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut su: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut sf: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut sc: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut sr: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut last_tl = String::new();
        for r in 0..5 {
            for i in 0..scale_arms.len() {
                let arm = scale_arms[(r + i) % scale_arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, &w.to_string()]).output().unwrap();
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                assert_eq!((p[6].to_string(), p[7].to_string()), *ref_bh.as_ref().unwrap());
                sw.entry(arm).or_default().push(p[0].parse().unwrap());
                su.entry(arm).or_default().push(p[1].parse().unwrap());
                ss.entry(arm).or_default().push(p[2].parse().unwrap());
                sf.entry(arm).or_default().push(p[3].parse().unwrap());
                sc.entry(arm).or_default().push(p[4].parse().unwrap());
                sr.entry(arm).or_default().push(p[5].parse().unwrap());
                if arm == "S7_ParScan_P2" && p.len() > 8 { last_tl = p[8..].join(" "); }
            }
        }
        let ma = &sw["S0_MasterA"];
        println!("{:<18} {:>7} {:>8} {:>7} {:>6} {:>7} {:>6} {:>7}", "Arm", "med_ms", "vs Mast", "user", "sys", "minflt", "vcsw", "rss_MB");
        for &arm in &scale_arms {
            let mut wv = sw[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut uv = su[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = ss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = sf[arm].clone(); fv.sort();
            let mut cv = sc[arm].clone(); cv.sort();
            let mut rv = sr[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, _, _) = ci95_ratio(&ma.iter().copied().zip(sw[arm].iter().copied()).collect::<Vec<_>>());
            println!("{:<18} {:>7.2} {:>7.4}x {:>7.1} {:>6.2} {:>7} {:>6} {:>7.2}",
                arm, wv[2], rat, uv[2], sv[2], fv[2], cv[2], rv[2]);
        }
        println!("  [S7_ParScan_P2 {w}W Timeline] {last_tl}");
    }
    let out4 = Command::new(&exe).args(["--child", path, "S7_ParScan_P2", "4"]).output().unwrap();
    let s4 = String::from_utf8(out4.stdout).unwrap();
    let p4: Vec<&str> = s4.split_whitespace().collect();
    println!("\n  [S7_ParScan_P2 4W Timeline] {}", p4[8..].join(" "));
}
