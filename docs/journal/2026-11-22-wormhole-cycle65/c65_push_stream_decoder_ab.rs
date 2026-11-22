use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use mibz2::bits::Bits;
use mibz2::block::{read_item, Item, Symbols};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::{Error, ErrorKind, Kernels, Scratch, StreamDecoder};

mod pr4_push_api {
    #![forbid(unsafe_code)]
    use super::*;

    pub const DEFAULT_WIN_CAP: usize = 2 << 20;
    pub const MAX_LIVE_WINS: usize = 3;
    pub const READY_BYTES: usize = 12 << 20;
    pub const POOLED_FROM: usize = 64 << 10;
    const TILE: usize = 64 << 10;
    pub type Run = (u32, u8, u8);

    #[derive(Clone, Copy, Debug)]
    pub struct SpanCand { pub rel_bit: u64, pub guessed_level: u8 }

    pub struct FastAnchorFinder { bm: [u64; 1024] }
    impl FastAnchorFinder {
        pub fn new() -> Self {
            const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
            let mut bm = [0u64; 1024];
            for a in 0..256usize {
                for b in 0..256usize {
                    let v = ((a as u64) << 56) | (BLOCK_MAGIC << 8) | (b as u64);
                    for shift in 0..8u32 {
                        let sv = v >> shift;
                        for pos in 0..5usize {
                            let pair = ((sv >> (8 * pos)) & 0xFFFF) as usize;
                            bm[pair >> 6] |= 1u64 << (pair & 63);
                        }
                    }
                }
            }
            for &(x, y) in &[(b'B', b'Z'), (b'Z', b'h')] {
                let idx = ((x as usize) << 8) | (y as usize);
                bm[idx >> 6] |= 1u64 << (idx & 63);
            }
            Self { bm }
        }
        #[inline]
        fn verify_around(&self, win: &[u8], center: usize, level: &mut u8, out: &mut Vec<SpanCand>, last_bit: &mut i64) {
            const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
            let lo = center.saturating_sub(4);
            let hi = (center + 2).min(win.len().saturating_sub(4));
            for i in lo..=hi {
                if i + 4 <= win.len() && win[i] == b'B' && win[i+1] == b'Z' && win[i+2] == b'h'
                    && (b'1'..=b'9').contains(&win[i+3])
                { *level = win[i+3] - b'0'; }
                if *level == 0 || i + 6 >= win.len() { continue; }
                let w = ((win[i] as u64) << 48) | ((win[i+1] as u64) << 40) | ((win[i+2] as u64) << 32)
                    | ((win[i+3] as u64) << 24) | ((win[i+4] as u64) << 16) | ((win[i+5] as u64) << 8)
                    | (win[i+6] as u64);
                for sh in 0u32..8 {
                    if ((w >> (8 - sh)) & 0xFFFF_FFFF_FFFF) == BLOCK_MAGIC {
                        let bit = (i as u64) * 8 + sh as u64;
                        if (bit as i64) > *last_bit {
                            out.push(SpanCand { rel_bit: bit, guessed_level: *level });
                            *last_bit = bit as i64;
                        }
                    }
                }
            }
        }
        pub fn find_cands(&self, win: &[u8], init_level: u8, out: &mut Vec<SpanCand>) {
            out.clear();
            if win.len() < 10 { return; }
            let (mut level, mut last_bit) = (init_level, -1i64);
            if win[0] == b'B' && win[1] == b'Z' && win[2] == b'h' && (b'1'..=b'9').contains(&win[3]) {
                level = win[3] - b'0';
            }
            let bm = &self.bm;
            let (mut p, n) = (0usize, win.len());
            while p + 14 <= n {
                let s = &win[p..p + 14];
                let i0 = ((s[0] as usize) << 8) | (s[1] as usize);
                let i1 = ((s[4] as usize) << 8) | (s[5] as usize);
                let i2 = ((s[8] as usize) << 8) | (s[9] as usize);
                let i3 = ((s[12] as usize) << 8) | (s[13] as usize);
                if (((bm[i0>>6]>>(i0&63)) | (bm[i1>>6]>>(i1&63)) | (bm[i2>>6]>>(i2&63)) | (bm[i3>>6]>>(i3&63))) & 1) != 0 {
                    if ((bm[i0>>6]>>(i0&63)) & 1) != 0 { self.verify_around(win, p, &mut level, out, &mut last_bit); }
                    if ((bm[i1>>6]>>(i1&63)) & 1) != 0 { self.verify_around(win, p + 4, &mut level, out, &mut last_bit); }
                    if ((bm[i2>>6]>>(i2&63)) & 1) != 0 { self.verify_around(win, p + 8, &mut level, out, &mut last_bit); }
                    if ((bm[i3>>6]>>(i3&63)) & 1) != 0 { self.verify_around(win, p + 12, &mut level, out, &mut last_bit); }
                }
                p += 16;
            }
            while p + 2 <= n {
                let idx = ((win[p] as usize) << 8) | (win[p + 1] as usize);
                if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 { self.verify_around(win, p, &mut level, out, &mut last_bit); }
                p += 4;
            }
        }
    }

    #[inline]
    fn find_runs_swar(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        runs.clear();
        let (n, mut p) = (rle.len(), 0usize);
        while p + 8 <= n {
            let w = u64::from_le_bytes(rle[p..p + 8].try_into().unwrap());
            let d = w ^ (w >> 8);
            let z = !(((d & 0x7F7F_7F7F_7F7F_7F7F) + 0x7F7F_7F7F_7F7F_7F7F) | d) & 0x8080_8080_8080_8080;
            let m = z & (z >> 8) & (z >> 16) & 0x0000_00FF_FFFF_FFFF;
            if m == 0 { p += 5; } else {
                let idx = p + (m.trailing_zeros() >> 3) as usize;
                let Some(&extra) = rle.get(idx + 4) else { return Err(ErrorKind::RunUnfinished); };
                runs.push((idx as u32, rle[idx], extra));
                p = idx + 5;
            }
        }
        while p + 3 < n {
            let c = rle[p];
            if rle[p+1] == c && rle[p+2] == c && rle[p+3] == c {
                let Some(&extra) = rle.get(p + 4) else { return Err(ErrorKind::RunUnfinished); };
                runs.push((p as u32, c, extra));
                p += 5;
            } else { p += 1; }
        }
        Ok(())
    }

    fn unrle_zs52(buf: &mut Vec<u8>, runs: &mut Vec<Run>, k: Kernels) -> Result<u32, ErrorKind> {
        let n = buf.len();
        find_runs_swar(buf, runs)?;
        if runs.is_empty() { return Ok(mibz2::crc::crc32(buf, k)); }
        let (mut grown, mut max_end) = (0isize, n);
        for &(j, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            let e = (j as usize + 5) as isize + grown;
            if e as usize > max_end { max_end = e as usize; }
        }
        let final_len = (n as isize + grown) as usize;
        if final_len > max_end { max_end = final_len; }
        if max_end > buf.len() {
            if max_end > buf.capacity() { buf.reserve_exact(max_end - buf.len()); }
            buf.resize(max_end, 0);
        }
        let (mut crc_s, mut crc_at) = (!0u32, 0usize);
        let crc_flush = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= TILE { *s = mibz2::crc::update_with(*s, &buf[*at..*at + TILE], k); *at += TILE; }
        };
        let (rlen, mut idx, mut src_pos, mut g) = (runs.len(), 0usize, 0usize, 0isize);
        while idx < rlen {
            let (j0, c0, e0) = runs[idx];
            let (j0u, g_after0) = (j0 as usize, g + (e0 as isize - 1));
            if g <= 0 && g_after0 <= 0 {
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos { buf.copy_within(src_pos..j0u, dst_lit); }
                let dst_run = (j0u as isize + g) as usize;
                let rl = 4 + e0 as usize;
                buf[dst_run..dst_run + rl].fill(c0);
                src_pos = j0u + 5; g = g_after0; idx += 1;
                crc_flush(&mut crc_s, &mut crc_at, buf, dst_run + rl);
            } else {
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos { buf.copy_within(src_pos..j0u, dst_lit); }
                crc_flush(&mut crc_s, &mut crc_at, buf, (j0u as isize + g) as usize);
                let (hill_start, mut cur_g, mut k_i, mut closed) = (idx, g_after0, idx + 1, false);
                while k_i < rlen {
                    let ng = cur_g + (runs[k_i].2 as isize - 1);
                    if ng == 0 { closed = true; break; }
                    cur_g = ng; k_i += 1;
                }
                let (hill_end, mut se, mut de, next_src, next_g) = if closed {
                    let jk = runs[k_i].0 as usize;
                    (k_i + 1, jk + 5, jk + 5, jk + 5, 0isize)
                } else { (rlen, n, (n as isize + cur_g) as usize, n, cur_g) };
                for r_i in (hill_start..hill_end).rev() {
                    let (rj, rc, re) = runs[r_i];
                    let (rju, lit_s) = (rj as usize, rj as usize + 5);
                    if se > lit_s {
                        let ll = se - lit_s; de -= ll;
                        if de != lit_s { buf.copy_within(lit_s..se, de); }
                    }
                    let rl = 4 + re as usize; de -= rl; buf[de..de + rl].fill(rc); se = rju;
                }
                let hill_dst_end = if closed { runs[k_i].0 as usize + 5 } else { (n as isize + cur_g) as usize };
                crc_flush(&mut crc_s, &mut crc_at, buf, hill_dst_end);
                idx = hill_end; src_pos = next_src; g = next_g;
            }
        }
        if src_pos < n {
            let dst_lit = (src_pos as isize + g) as usize;
            if dst_lit != src_pos { buf.copy_within(src_pos..n, dst_lit); }
        }
        buf.truncate(final_len);
        crc_s = mibz2::crc::update_with(crc_s, &buf[crc_at..], k);
        Ok(!crc_s)
    }

    struct OwnedWorker { inv: OpenCells, runs: Runs, rle_runs: Vec<Run>, fb_sc: Option<Scratch<OpenCells>> }
    impl OwnedWorker {
        fn new() -> Self { Self { inv: OpenCells::default(), runs: Runs::default(), rle_runs: Vec::new(), fb_sc: None } }
        fn decode_owned(&mut self, b: &Symbols, k: Kernels, take_out: impl FnOnce(usize) -> Vec<u8>) -> Result<(u32, Vec<u8>), ErrorKind> {
            if self.inv.reads_runs() && runs::takes_runs(b) {
                let n = runs::front(b, &mut self.runs, k)?;
                if b.orig_ptr as usize >= n { return Err(ErrorKind::BadOrigPtr); }
                let mut out = take_out(n);
                if !self.inv.invert_runs(&self.runs, n, b.orig_ptr, &mut out, k) {
                    return Err(ErrorKind::Internal("records don't tile".into()));
                }
                let computed = unrle_zs52(&mut out, &mut self.rle_runs, k)?;
                if computed != b.crc { return Err(ErrorKind::BlockCrc { stored: b.crc, computed }); }
                return Ok((computed, out));
            }
            let sc = self.fb_sc.get_or_insert_with(Scratch::<OpenCells>::default);
            let mut out = take_out(b.max_len);
            out.clear();
            let crc = mibz2::decode_block(b, k, sc, &mut out)?;
            Ok((crc, out))
        }
    }

    struct WindowBuf { abs_base_byte: u64, owned_end_byte: u64, data: Vec<u8>, eof: bool }
    enum SpecOutcome { Block { abs_end_bit: u64, crc: u32, out: Vec<u8> }, NeedMoreOrTrunc, Err(ErrorKind) }
    struct SpecEntry { abs_start_bit: u64, level: u8, outcome: SpecOutcome }
    struct SpanTask { win: Arc<WindowBuf>, cand_seq: usize, abs_start_bit: u64, rel_bit: u64, level: u8 }

    struct TaskQ { q: VecDeque<SpanTask>, closed: bool, waiting: usize, free_win: Vec<Vec<u8>>, free_in: Vec<Vec<u8>> }
    struct ReadyRing {
        ring: Vec<Option<SpecEntry>>, mask: usize, taken_seq: usize, ready_bytes: usize,
        rw: bool, ww: usize, free_out: Vec<Vec<u8>>, win_log: VecDeque<Arc<WindowBuf>>, win_closed: bool, published_seq: usize,
    }

    struct PushState {
        finder: FastAnchorFinder,
        win_cap: usize,
        max_live_wins: usize,
        staging: Vec<u8>,
        carry: Vec<u8>,
        abs_base: u64,
        min_next_abs_bit: u64,
        last_level: u8,
        next_seq: usize,
        cands: Vec<SpanCand>,
        closed: bool,
    }

    struct PullState {
        cur_win: Option<Arc<WindowBuf>>,
        abs_bit: u64,
        first: bool,
        in_stream: bool,
        level: u8,
        combined: u32,
        block_idx: u64,
        done: bool,
        pending_spec: Option<SpecEntry>,
        fb_syms: Vec<u16>,
        fb_wst: OwnedWorker,
    }

    /// Exact drop-in push/pull StreamDecoder backed by PR 4 W2M_L3 (ZERO dedicated scanner thread!).
    /// - `spare()` returns a recycled input buffer (`free_win` / `Vec`)
    /// - `write(Vec<u8>)` accumulates to `win_cap`, runs inline anchor scan (~0.35 ms), publishes SpanTasks
    /// - `close()` flushes final EOF window
    /// - `read_block(spent)` recycles `spent` into `free_out` and runs ChainValidator inline
    /// - `cancel()` tears down with 0 deadlocks
    pub struct Pr4StreamDecoder {
        tq: Arc<(Mutex<TaskQ>, Condvar, Condvar)>,
        ready: Arc<Mutex<ReadyRing>>,
        cv_r: Arc<Condvar>,
        cv_w: Arc<Condvar>,
        live_wins: Arc<AtomicUsize>,
        stopped: Arc<AtomicBool>,
        slots: usize,
        k: Kernels,
        push: Mutex<PushState>,
        pull: Mutex<PullState>,
        workers: Mutex<Vec<JoinHandle<()>>>,
        pub fast_hits: AtomicUsize,
        pub fallbacks: AtomicUsize,
    }

    fn release_win(w: Arc<WindowBuf>, tq: &(Mutex<TaskQ>, Condvar, Condvar)) {
        if let Ok(wb) = Arc::try_unwrap(w) {
            let mut q = tq.0.lock().unwrap();
            q.free_win.push(wb.data);
            drop(q);
            tq.2.notify_one();
        }
    }

    impl Pr4StreamDecoder {
        pub fn new(threads: usize, slots: usize, k: Kernels) -> Self {
            let slots = slots.max(threads);
            let ring_cap = (slots * 2).next_power_of_two();
            let floor = threads + 2;
            let tq = Arc::new((
                Mutex::new(TaskQ { q: VecDeque::with_capacity(256), closed: false, waiting: 0, free_win: Vec::with_capacity(MAX_LIVE_WINS), free_in: Vec::with_capacity(2) }),
                Condvar::new(), Condvar::new(),
            ));
            let ready = Arc::new(Mutex::new(ReadyRing {
                ring: (0..ring_cap).map(|_| None).collect(), mask: ring_cap - 1, taken_seq: 0, ready_bytes: 0,
                rw: false, ww: 0, free_out: Vec::with_capacity(slots), win_log: VecDeque::with_capacity(4), win_closed: false, published_seq: 0,
            }));
            let cv_r = Arc::new(Condvar::new());
            let cv_w = Arc::new(Condvar::new());
            let live_wins = Arc::new(AtomicUsize::new(0));
            let stopped = Arc::new(AtomicBool::new(false));

            let mut handles = Vec::with_capacity(threads);
            for _ in 0..threads {
                let (tq_w, rw_w, cvr_w, cvw_w, stp_w) = (tq.clone(), ready.clone(), cv_r.clone(), cv_w.clone(), stopped.clone());
                handles.push(std::thread::spawn(move || {
                    let mut syms = Vec::new();
                    let mut wst = OwnedWorker::new();
                    loop {
                        let task = {
                            let mut q = tq_w.0.lock().unwrap();
                            loop {
                                if stp_w.load(Ordering::Acquire) { return; }
                                if let Some(t) = q.q.pop_front() { break Some(t); }
                                if q.closed { break None; }
                                q.waiting += 1; q = tq_w.1.wait(q).unwrap(); q.waiting -= 1;
                            }
                        };
                        let Some(SpanTask { win, cand_seq, abs_start_bit, rel_bit, level }) = task else { return; };
                        {
                            let mut st = rw_w.lock().unwrap();
                            while !stp_w.load(Ordering::Acquire) && (
                                cand_seq >= st.taken_seq + slots || (st.ready_bytes >= READY_BYTES && cand_seq >= st.taken_seq + floor)
                            ) { st.ww += 1; st = cvw_w.wait(st).unwrap(); st.ww -= 1; }
                            if stp_w.load(Ordering::Acquire) { release_win(win, &tq_w); return; }
                        }
                        let mut br = Bits::at(&win.data, rel_bit);
                        let read_res = read_item(&mut br, level as u32, std::mem::take(&mut syms));
                        let over = br.overran();
                        let abs_end_bit = win.abs_base_byte * 8 + br.bit_pos();
                        release_win(win, &tq_w);
                        let outcome = match read_res {
                            Ok(Item::Block(b)) if !over => {
                                let dec = wst.decode_owned(&b, k, |n| {
                                    let want = n + n / 16 + 64;
                                    if want >= POOLED_FROM {
                                        let mut v = rw_w.lock().unwrap().free_out.pop().unwrap_or_else(|| Vec::with_capacity(want));
                                        if v.capacity() < want { v.reserve_exact(want - v.len()); }
                                        v
                                    } else { Vec::with_capacity(want) }
                                });
                                syms = b.syms;
                                match dec { Ok((crc, out)) => SpecOutcome::Block { abs_end_bit, crc, out }, Err(e) => SpecOutcome::Err(e) }
                            }
                            Ok(Item::Block(_)) | Ok(Item::End(_)) | Err((ErrorKind::Truncated, _)) => SpecOutcome::NeedMoreOrTrunc,
                            Err(_) if over => SpecOutcome::NeedMoreOrTrunc,
                            Err((e, _)) => SpecOutcome::Err(e),
                        };
                        let bbytes = match &outcome { SpecOutcome::Block { out, .. } => out.len(), _ => 0 };
                        let mut st = rw_w.lock().unwrap();
                        let idx = cand_seq & st.mask;
                        st.ring[idx] = Some(SpecEntry { abs_start_bit, level, outcome });
                        st.ready_bytes += bbytes;
                        let wake = st.rw && cand_seq == st.taken_seq; drop(st);
                        if wake { cvr_w.notify_one(); }
                    }
                }));
            }

            Self {
                tq, ready, cv_r, cv_w, live_wins, stopped, slots, k,
                push: Mutex::new(PushState {
                    finder: FastAnchorFinder::new(), win_cap: DEFAULT_WIN_CAP, max_live_wins: MAX_LIVE_WINS,
                    staging: Vec::new(), carry: Vec::new(), abs_base: 0, min_next_abs_bit: 0, last_level: 0, next_seq: 0,
                    cands: Vec::with_capacity(128), closed: false,
                }),
                pull: Mutex::new(PullState {
                    cur_win: None, abs_bit: 0, first: true, in_stream: false, level: 0, combined: 0, block_idx: 0,
                    done: false, pending_spec: None, fb_syms: Vec::new(), fb_wst: OwnedWorker::new(),
                }),
                workers: Mutex::new(handles),
                fast_hits: AtomicUsize::new(0),
                fallbacks: AtomicUsize::new(0),
            }
        }

        fn acquire_win_buf(&self, max_live_wins: usize, win_cap: usize) -> Option<Vec<u8>> {
            let mut q = self.tq.0.lock().unwrap();
            loop {
                if self.stopped.load(Ordering::Acquire) { return None; }
                if let Some(v) = q.free_win.pop() { return Some(v); }
                if self.live_wins.load(Ordering::Acquire) < max_live_wins {
                    self.live_wins.fetch_add(1, Ordering::AcqRel);
                    return Some(Vec::with_capacity(win_cap));
                }
                q = self.tq.2.wait(q).unwrap();
            }
        }

        /// Publish a ready `WindowBuf` (zero-copy move of `buf` into `Arc<WindowBuf>`),
        /// returning the tail carry bytes to retain in `ps.staging`.
        fn publish_window(&self, ps: &mut PushState, buf: Vec<u8>, eof: bool) {
            if buf.is_empty() && eof { return; }
            ps.finder.find_cands(&buf, ps.last_level, &mut ps.cands);
            let abs_base = ps.abs_base;
            let min_bit = ps.min_next_abs_bit;
            ps.cands.retain(|c| abs_base * 8 + c.rel_bit >= min_bit);
            if let Some(last) = ps.cands.last() { ps.last_level = last.guessed_level; }

            let max_carry = (ps.win_cap / 2).max(16);
            ps.carry.clear();
            let publish_n = if !eof && ps.cands.len() >= 2 {
                let cut_byte = (ps.cands.last().unwrap().rel_bit / 8) as usize;
                if buf.len() - cut_byte <= max_carry {
                    ps.carry.extend_from_slice(&buf[cut_byte..]);
                    ps.cands.len() - 1
                } else {
                    let keep = buf.len().min(16);
                    ps.carry.extend_from_slice(&buf[buf.len() - keep..]);
                    ps.cands.len()
                }
            } else if !eof && ps.cands.len() == 1 {
                let cut_byte = (ps.cands[0].rel_bit / 8) as usize;
                if buf.len() - cut_byte <= max_carry {
                    ps.carry.extend_from_slice(&buf[cut_byte..]);
                    0
                } else {
                    let keep = buf.len().min(16);
                    ps.carry.extend_from_slice(&buf[buf.len() - keep..]);
                    1
                }
            } else if !eof {
                let keep = buf.len().min(16);
                ps.carry.extend_from_slice(&buf[buf.len() - keep..]);
                0
            } else { ps.cands.len() };

            let next_abs_base = abs_base + (buf.len() - ps.carry.len()) as u64;
            if publish_n > 0 {
                ps.min_next_abs_bit = abs_base * 8 + ps.cands[publish_n - 1].rel_bit + 1;
            }
            let win_arc = Arc::new(WindowBuf { abs_base_byte: abs_base, owned_end_byte: next_abs_base, data: buf, eof });
            {
                let mut st = self.ready.lock().unwrap();
                st.win_log.push_back(Arc::clone(&win_arc));
                if st.rw { self.cv_r.notify_one(); }
            }
            if publish_n > 0 {
                let mut q = self.tq.0.lock().unwrap();
                for c in &ps.cands[..publish_n] {
                    q.q.push_back(SpanTask {
                        win: Arc::clone(&win_arc), cand_seq: ps.next_seq,
                        abs_start_bit: abs_base * 8 + c.rel_bit, rel_bit: c.rel_bit, level: c.guessed_level,
                    });
                    ps.next_seq += 1;
                }
                let wake = q.waiting > 0; drop(q);
                if wake { self.tq.1.notify_all(); }
            }
            release_win(win_arc, &self.tq);
            ps.abs_base = next_abs_base;
            // ps.carry holds tail bytes; ps.staging is filled from recycled free_win on next write!
        }

        /// Same signature as `StreamDecoder::spare()`.
        pub fn spare(&self) -> Vec<u8> {
            self.tq.0.lock().unwrap().free_in.pop().unwrap_or_default()
        }

        /// Same push semantics as `StreamDecoder::write(Vec<u8>)`: accepts ANY chunk size, ZERO 2 MiB drops!
        pub fn write(&self, data: Vec<u8>) -> Result<(), ()> {
            if self.stopped.load(Ordering::Acquire) { return Err(()); }
            if data.is_empty() { return Ok(()); }
            let mut ps_g = self.push.lock().unwrap(); let ps = &mut *ps_g;
            if ps.closed { return Err(()); }
            let win_cap = ps.win_cap;
            let max_lw = ps.max_live_wins;

            let mut off = 0usize;
            while off < data.len() {
                if ps.staging.capacity() < win_cap {
                    let Some(mut wbuf) = self.acquire_win_buf(max_lw, win_cap) else { return Err(()); };
                    wbuf.clear();
                    if !ps.carry.is_empty() {
                        wbuf.extend_from_slice(&ps.carry);
                        ps.carry.clear();
                    }
                    ps.staging = wbuf;
                } else if ps.staging.is_empty() && !ps.carry.is_empty() {
                    ps.staging.extend_from_slice(&ps.carry);
                    ps.carry.clear();
                }
                let need = win_cap.saturating_sub(ps.staging.len());
                let take = (data.len() - off).min(need);
                ps.staging.extend_from_slice(&data[off..off + take]);
                off += take;
                if ps.staging.len() >= win_cap {
                    let wbuf = std::mem::take(&mut ps.staging);
                    self.publish_window(ps, wbuf, false);
                }
            }
            let mut q = self.tq.0.lock().unwrap();
            if q.free_in.len() < 2 { q.free_in.push(data); }
            Ok(())
        }

        pub fn close(&self) {
            let mut ps_g = self.push.lock().unwrap(); let ps = &mut *ps_g;
            if ps.closed { return; }
            ps.closed = true;
            if ps.staging.is_empty() && !ps.carry.is_empty() {
                let c = std::mem::take(&mut ps.carry);
                self.publish_window(ps, c, true);
            } else if !ps.staging.is_empty() {
                let buf = std::mem::take(&mut ps.staging);
                self.publish_window(ps, buf, true);
            }
            let total = ps.next_seq;
            drop(ps_g);
            { let mut q = self.tq.0.lock().unwrap(); q.closed = true; }
            self.tq.1.notify_all();
            { let mut st = self.ready.lock().unwrap(); st.published_seq = total; st.win_closed = true; if st.rw { self.cv_r.notify_one(); } }
        }

        pub fn cancel(&self) {
            self.stopped.store(true, Ordering::Release);
            { let mut q = self.tq.0.lock().unwrap(); q.closed = true; q.q.clear(); }
            { let mut st = self.ready.lock().unwrap(); st.win_closed = true; st.win_log.clear(); }
            self.tq.1.notify_all();
            self.tq.2.notify_all();
            self.cv_w.notify_all();
            self.cv_r.notify_all();
        }

        /// Same pull semantics as `StreamDecoder::read_block(spent: Vec<u8>) -> Result<Option<Vec<u8>>, Error>`.
        pub fn read_block(&self, spent: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
            if spent.capacity() >= POOLED_FROM {
                let mut st = self.ready.lock().unwrap();
                if st.free_out.len() < self.slots { st.free_out.push(spent); }
            }
            let mut pl = self.pull.lock().unwrap();
            if pl.done { return Ok(None); }

            let ensure_win = |cur_win: &mut Option<Arc<WindowBuf>>, abs_bit: u64, min_bytes: usize| -> bool {
                loop {
                    if let Some(ref w) = cur_win {
                        let base_b = w.abs_base_byte * 8;
                        let end_b = if w.eof { (w.abs_base_byte + w.data.len() as u64) * 8 } else { w.owned_end_byte * 8 };
                        if abs_bit >= base_b && (abs_bit < end_b && abs_bit + (min_bytes as u64) * 8 <= (w.abs_base_byte + w.data.len() as u64) * 8 || w.eof) {
                            return true;
                        }
                    }
                    if let Some(old) = cur_win.take() { release_win(old, &self.tq); }
                    let next_w = {
                        let mut st = self.ready.lock().unwrap();
                        loop {
                            if let Some(w) = st.win_log.pop_front() { break Some(w); }
                            if st.win_closed { break None; }
                            st.rw = true; st = self.cv_r.wait(st).unwrap();
                        }
                    };
                    match next_w { Some(w) => *cur_win = Some(w), None => return false }
                }
            };
            let take_next_spec = |pending: &mut Option<SpecEntry>| -> Option<SpecEntry> {
                if let Some(e) = pending.take() { return Some(e); }
                let mut st = self.ready.lock().unwrap();
                loop {
                    let t = st.taken_seq; let idx = t & st.mask;
                    if let Some(entry) = st.ring[idx].take() {
                        let bbytes = match &entry.outcome { SpecOutcome::Block { out, .. } => out.len(), _ => 0 };
                        st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                        st.taken_seq = t + 1;
                        let ww = st.ww > 0; st.rw = false; drop(st);
                        if ww { self.cv_w.notify_one(); }
                        return Some(entry);
                    }
                    if st.win_closed && st.taken_seq >= st.published_seq { st.rw = false; return None; }
                    st.rw = true; st = self.cv_r.wait(st).unwrap();
                }
            };

            loop {
                if !pl.in_stream {
                    let ab = pl.abs_bit;
                    if !ensure_win(&mut pl.cur_win, ab, 4) {
                        pl.done = true;
                        if pl.first { return Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(0) }); }
                        return Ok(None);
                    }
                    let (empty_eof, hdr_level) = {
                        let w = pl.cur_win.as_ref().unwrap();
                        let rem = &w.data[(ab / 8 - w.abs_base_byte) as usize..];
                        let lv = match rem { [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => Some(*l - b'0'), _ => None };
                        (rem.is_empty() && w.eof, lv)
                    };
                    if empty_eof {
                        pl.done = true;
                        if pl.first { return Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(ab) }); }
                        return Ok(None);
                    }
                    match hdr_level {
                        Some(lv) => {
                            pl.in_stream = true; pl.level = lv; pl.combined = 0; pl.abs_bit += 32;
                        }
                        _ => {
                            pl.done = true;
                            return Err(Error { kind: if pl.first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData }, block: None, bit: Some(ab) });
                        }
                    }
                    continue;
                }
                let ab = pl.abs_bit;
                if !ensure_win(&mut pl.cur_win, ab, 12) {
                    pl.done = true;
                    return Err(Error { kind: ErrorKind::Truncated, block: Some(pl.block_idx), bit: Some(ab) });
                }
                let (magic, end_crc, next_bit, m_over, e_over) = {
                    let w = pl.cur_win.as_ref().unwrap();
                    let mut br = Bits::at(&w.data, ab - w.abs_base_byte * 8);
                    let m = ((br.bits(24) as u64) << 24) | (br.bits(24) as u64);
                    let mo = br.overran();
                    let sc = if !mo && m == 0x1772_4538_5090 { br.bits(32) } else { 0 };
                    let eo = br.overran();
                    (m, sc, ((w.abs_base_byte * 8 + br.bit_pos() + 7) / 8) * 8, mo, eo)
                };
                if m_over { pl.done = true; return Err(Error { kind: ErrorKind::Truncated, block: Some(pl.block_idx), bit: Some(ab) }); }
                if magic == 0x1772_4538_5090 {
                    if e_over { pl.done = true; return Err(Error { kind: ErrorKind::Truncated, block: Some(pl.block_idx), bit: Some(ab) }); }
                    if end_crc != pl.combined { pl.done = true; return Err(Error { kind: ErrorKind::StreamCrc { stored: end_crc, computed: pl.combined }, block: Some(pl.block_idx), bit: Some(ab) }); }
                    pl.abs_bit = next_bit; pl.in_stream = false; pl.first = false; continue;
                }
                if magic != 0x3141_5926_5359 { pl.done = true; return Err(Error { kind: ErrorKind::BadBlockMagic, block: Some(pl.block_idx), bit: Some(ab) }); }

                let matched = loop {
                    match take_next_spec(&mut pl.pending_spec) {
                        Some(e) if e.abs_start_bit < ab => {
                            if let SpecOutcome::Block { out, .. } = e.outcome {
                                if out.capacity() >= POOLED_FROM { let mut st = self.ready.lock().unwrap(); if st.free_out.len() < self.slots { st.free_out.push(out); } }
                            }
                        }
                        Some(e) if e.abs_start_bit == ab && e.level == pl.level => break Some(e),
                        Some(e) => { pl.pending_spec = Some(e); break None; }
                        None => break None,
                    }
                };
                if let Some(entry) = matched {
                    match entry.outcome {
                        SpecOutcome::Block { abs_end_bit, crc, out } => {
                            self.fast_hits.fetch_add(1, Ordering::Relaxed);
                            pl.combined = pl.combined.rotate_left(1) ^ crc;
                            pl.abs_bit = abs_end_bit;
                            pl.block_idx += 1;
                            return Ok(Some(out));
                        }
                        SpecOutcome::Err(kind) => { pl.done = true; return Err(Error { kind, block: Some(pl.block_idx), bit: Some(ab) }); }
                        SpecOutcome::NeedMoreOrTrunc => {}
                    }
                }
                self.fallbacks.fetch_add(1, Ordering::Relaxed);
                let (start_byte, bit_rem) = (ab / 8, ab % 8);
                let (mut fb_buf, mut fb_end_byte) = (Vec::<u8>::new(), start_byte);
                let lv = pl.level;
                loop {
                    if let Some(ref w) = pl.cur_win {
                        let w_end = w.abs_base_byte + w.data.len() as u64;
                        if fb_end_byte < w_end {
                            fb_buf.extend_from_slice(&w.data[(fb_end_byte - w.abs_base_byte) as usize..]);
                            fb_end_byte = w_end;
                        }
                    }
                    let mut br = Bits::at(&fb_buf, bit_rem);
                    let r = read_item(&mut br, lv as u32, std::mem::take(&mut pl.fb_syms));
                    let r = match r {
                        Err((_, b)) if br.overran() => Err((ErrorKind::Truncated, b)),
                        Ok(_) if br.overran() => Err((ErrorKind::Truncated, Vec::new())),
                        other => other,
                    };
                    match r {
                        Ok(Item::Block(b)) => {
                            let end_abs = start_byte * 8 + br.bit_pos();
                            let dec = pl.fb_wst.decode_owned(&b, self.k, |n| Vec::with_capacity(n + n / 16 + 64));
                            pl.fb_syms = b.syms;
                            match dec {
                                Ok((crc, out)) => {
                                    pl.combined = pl.combined.rotate_left(1) ^ crc;
                                    pl.abs_bit = end_abs; pl.block_idx += 1;
                                    return Ok(Some(out));
                                }
                                Err(kind) => { pl.done = true; return Err(Error { kind, block: Some(pl.block_idx), bit: Some(ab) }); }
                            }
                        }
                        Err((ErrorKind::Truncated, _)) if pl.cur_win.as_ref().map_or(true, |w| !w.eof) => {
                            if let Some(old) = pl.cur_win.take() { release_win(old, &self.tq); }
                            let next_w = {
                                let mut st = self.ready.lock().unwrap();
                                loop {
                                    if let Some(nw) = st.win_log.pop_front() { break Some(nw); }
                                    if st.win_closed { break None; }
                                    st.rw = true; st = self.cv_r.wait(st).unwrap();
                                }
                            };
                            match next_w { Some(nw) => pl.cur_win = Some(nw), None => { pl.done = true; return Err(Error { kind: ErrorKind::Truncated, block: Some(pl.block_idx), bit: Some(ab) }); } }
                        }
                        Err((kind, _)) => { pl.done = true; return Err(Error { kind, block: Some(pl.block_idx), bit: Some(ab) }); }
                        Ok(Item::End(_)) => { pl.done = true; return Err(Error { kind: ErrorKind::Truncated, block: Some(pl.block_idx), bit: Some(ab) }); }
                    }
                }
            }
        }
    }

    impl Drop for Pr4StreamDecoder {
        fn drop(&mut self) {
            self.cancel();
            if let Ok(mut pl) = self.pull.lock() {
                if let Some(old) = pl.cur_win.take() { release_win(old, &self.tq); }
            }
            if let Ok(mut ws) = self.workers.lock() {
                for h in ws.drain(..) { let _ = h.join(); }
            }
        }
    }

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
}

#[repr(C)] #[derive(Clone, Copy)] struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)] #[derive(Clone, Copy)] struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval, ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64, ru_inblock: i64, ru_oublock: i64, ru_msgsnd: i64,
    ru_msgrcv: i64, ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" { fn getrusage(who: i32, usage: *mut Rusage) -> i32; }
fn rusage() -> Rusage { let mut r: Rusage = unsafe { std::mem::zeroed() }; unsafe { getrusage(0, &mut r) }; r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

fn ci95_ratio(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let logs: Vec<f64> = pairs.iter().map(|&(a, b)| (b / a).ln()).collect();
    let n = logs.len() as f64;
    let m = logs.iter().sum::<f64>() / n;
    let var = if logs.len() > 1 { logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) } else { 0.0 };
    let t = match logs.len() { 5 => 2.776, 8 => 2.365, 10 => 2.262, _ => 2.0 };
    let hw = t * (var / n).sqrt();
    (m.exp(), (m - hw).exp(), (m + hw).exp())
}

fn run_m739(data: &[u8], w: usize, k: Kernels) -> (u64, u64) {
    let dec = StreamDecoder::with_inverse::<OpenCells>(w, 4 * w, k);
    let mut h = pr4_push_api::AbHash::new();
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

fn run_pr4_push(data: &[u8], chunk: usize, w: usize, k: Kernels) -> (u64, u64, usize, usize) {
    let dec = pr4_push_api::Pr4StreamDecoder::new(w, 4 * w, k);
    let mut h = pr4_push_api::AbHash::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in data.chunks(chunk) {
                let mut v = dec.spare(); v.clear(); v.extend_from_slice(c);
                if dec.write(v).is_err() { return; }
            }
            dec.close();
        });
        let mut spent = Vec::new();
        while let Some(b) = dec.read_block(spent).unwrap() { h.feed(&b); spent = b; }
    });
    (h.bytes, h.h, dec.fast_hits.load(Ordering::Relaxed), dec.fallbacks.load(Ordering::Relaxed))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let arm = args[3].as_str();
        let w: usize = args[4].parse().unwrap();
        let r0 = rusage(); let t0 = Instant::now();
        let mut info = String::new();
        let (bytes, hash) = match arm {
            "M739_A" | "M739_B" => run_m739(&data, w, k),
            "PR4_Push_2M" | "PR4_Push_64K" => {
                let ch = if arm == "PR4_Push_64K" { 64 << 10 } else { 2 << 20 };
                let (b, h, fh, fb) = run_pr4_push(&data, ch, w, k);
                info = format!("fh={fh} fb={fb}");
                (b, h)
            }
            _ => panic!(),
        };
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {bytes} {hash:016x} {info}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
        return;
    }

    // First verify arbitrary chunk-size parity (17B, 4KiB, 64KiB, 1MiB, 2MiB, 4MiB) on wikidata-l1!
    let l1 = fs::read("/tmp/wormhole_r6/wikidata_l1_36m.bz2").unwrap();
    let (ref_b, ref_h) = run_m739(&l1, 4, k);
    for &ch in &[17usize, 4096, 65536, 1 << 20, 2 << 20, 4 << 20] {
        let (b, h, fh, fb) = run_pr4_push(&l1, ch, 4, k);
        assert_eq!((b, h), (ref_b, ref_h), "chunk={ch} parity mismatch");
        println!("[PARITY PASS] Pr4StreamDecoder::write(chunk={ch:>7}B) on wikidata-l1 4W: fh={fh} fb={fb} hash={h:016x}");
    }

    let exe = std::env::current_exe().unwrap();
    let arms = ["M739_A", "M739_B", "PR4_Push_2M", "PR4_Push_64K"];
    for &(label, path, w, rounds) in &[
        ("wikidata-l9 4W (210 MB)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 4usize, 8usize),
        ("wikidata-l9 10W (210 MB)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 10usize, 5usize),
        ("wikidata-l1 4W (38 MB)", "/tmp/wormhole_r6/wikidata_l1_36m.bz2", 4usize, 8usize),
        ("hetero-l9 4W (54 MB)", "/tmp/wormhole_r6/hetero_60b.bz2", 4usize, 8usize),
    ] {
        println!("\n=== {rounds}-ROUND CYCLE 65 DROP-IN PUSH/PULL StreamDecoder A/B: {label} ===");
        let mut ws: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut ss: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut fl: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut vc: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
        let mut rs: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut inf: BTreeMap<&str, String> = BTreeMap::new();
        let mut ref_bh: Option<(String, String)> = None;
        for r in 0..rounds {
            for i in 0..arms.len() {
                let arm = arms[(r + i) % arms.len()];
                let out = Command::new(&exe).args(["--child", path, arm, &w.to_string()]).output().unwrap();
                assert!(out.status.success(), "{arm}: {}", String::from_utf8_lossy(&out.stderr));
                let s = String::from_utf8(out.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                let bh = (p[6].to_string(), p[7].to_string());
                if let Some(ref e) = ref_bh { assert_eq!(&bh, e, "hash mismatch on {arm} {label}"); } else { ref_bh = Some(bh); }
                ws.entry(arm).or_default().push(p[0].parse().unwrap());
                ss.entry(arm).or_default().push(p[2].parse().unwrap());
                fl.entry(arm).or_default().push(p[3].parse().unwrap());
                vc.entry(arm).or_default().push(p[4].parse().unwrap());
                rs.entry(arm).or_default().push(p[5].parse().unwrap());
                if p.len() > 8 { inf.insert(arm, p[8..].join(" ")); }
            }
        }
        let ma = &ws["M739_A"];
        println!("{:<13} {:>7} {:>24} {:>6} {:>7} {:>5} {:>7}  {}", "Arm", "med_ms", "vs M739_A [95% CI]", "sys", "minflt", "vcsw", "rssMB", "Hits");
        for &arm in &arms {
            let mut wv = ws[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = ss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = fl[arm].clone(); fv.sort();
            let mut cv = vc[arm].clone(); cv.sort();
            let mut rv = rs[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(ws[arm].iter().copied()).collect::<Vec<_>>());
            let m = rounds / 2;
            println!("{:<13} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>6.2} {:>7} {:>5} {:>7.2}  {}",
                arm, wv[m], rat, lo, hi, sv[m], fv[m], cv[m], rv[m], inf.get(arm).map(String::as_str).unwrap_or("-"));
        }
    }
}
