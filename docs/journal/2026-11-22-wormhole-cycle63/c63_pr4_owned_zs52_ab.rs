use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::Cursor;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use mibz2::bits::Bits;
use mibz2::block::{read_item, Item, Symbols};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::{Error, ErrorKind, Kernels, Scratch, StreamDecoder};

mod pr4_owned_zs52 {
    #![forbid(unsafe_code)]
    use super::*;

    pub const POOLED_FROM: usize = 64 << 10;
    pub const READY_BYTES: usize = 12 << 20;
    const TILE: usize = 64 << 10;
    pub type Run = (u32, u8, u8);

    #[derive(Clone, Copy, Debug)]
    pub struct SpanCand { pub rel_bit: u64, pub guessed_level: u8 }

    pub trait SpanFinder: Send + Sync {
        fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>);
    }

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
        fn verify_around(&self, win: &[u8], center: usize, rel_start_bit: u64, level: &mut u8, out: &mut Vec<SpanCand>, last_bit: &mut i64) {
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
                        if bit >= rel_start_bit && (bit as i64) > *last_bit {
                            out.push(SpanCand { rel_bit: bit, guessed_level: *level });
                            *last_bit = bit as i64;
                        }
                    }
                }
            }
        }
    }
    impl SpanFinder for FastAnchorFinder {
        fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>) {
            out.clear();
            if win.len() < 10 { return; }
            let (mut level, mut last_bit) = (init_level, -1i64);
            let start_b = (rel_start_bit / 8) as usize;
            if start_b + 4 <= win.len() && win[start_b] == b'B' && win[start_b+1] == b'Z' && win[start_b+2] == b'h'
                && (b'1'..=b'9').contains(&win[start_b+3])
            { level = win[start_b+3] - b'0'; }
            let bm = &self.bm;
            let mut p = (start_b + 3) & !3;
            let n = win.len();
            while p + 14 <= n {
                let s = &win[p..p + 14];
                let i0 = ((s[0] as usize) << 8) | (s[1] as usize);
                let i1 = ((s[4] as usize) << 8) | (s[5] as usize);
                let i2 = ((s[8] as usize) << 8) | (s[9] as usize);
                let i3 = ((s[12] as usize) << 8) | (s[13] as usize);
                let h0 = (bm[i0 >> 6] >> (i0 & 63)) & 1;
                let h1 = (bm[i1 >> 6] >> (i1 & 63)) & 1;
                let h2 = (bm[i2 >> 6] >> (i2 & 63)) & 1;
                let h3 = (bm[i3 >> 6] >> (i3 & 63)) & 1;
                if (h0 | h1 | h2 | h3) != 0 {
                    if h0 != 0 { self.verify_around(win, p, rel_start_bit, &mut level, out, &mut last_bit); }
                    if h1 != 0 { self.verify_around(win, p + 4, rel_start_bit, &mut level, out, &mut last_bit); }
                    if h2 != 0 { self.verify_around(win, p + 8, rel_start_bit, &mut level, out, &mut last_bit); }
                    if h3 != 0 { self.verify_around(win, p + 12, rel_start_bit, &mut level, out, &mut last_bit); }
                }
                p += 16;
            }
            while p + 2 <= n {
                let idx = ((win[p] as usize) << 8) | (win[p + 1] as usize);
                if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 {
                    self.verify_around(win, p, rel_start_bit, &mut level, out, &mut last_bit);
                }
                p += 4;
            }
        }
    }

    /// Exact pure-safe-Rust SWAR 4-equal-byte run finder (5 bytes/iter, 0 FP, 0 FN).
    #[inline]
    pub fn find_runs_swar(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let mut p = 0usize;
        while p + 8 <= n {
            let w = u64::from_le_bytes(rle[p..p + 8].try_into().unwrap());
            let d = w ^ (w >> 8);
            let z = !(((d & 0x7F7F_7F7F_7F7F_7F7F) + 0x7F7F_7F7F_7F7F_7F7F) | d) & 0x8080_8080_8080_8080;
            let m = z & (z >> 8) & (z >> 16) & 0x0000_00FF_FFFF_FFFF;
            if m == 0 {
                p += 5;
            } else {
                let k = (m.trailing_zeros() >> 3) as usize;
                let idx = p + k;
                let Some(&extra) = rle.get(idx + 4) else { return Err(ErrorKind::RunUnfinished); };
                runs.push((idx as u32, rle[idx], extra));
                p = idx + 5;
            }
        }
        while p + 3 < n {
            let c = rle[p];
            if rle[p + 1] == c && rle[p + 2] == c && rle[p + 3] == c {
                let Some(&extra) = rle.get(p + 4) else { return Err(ErrorKind::RunUnfinished); };
                runs.push((p as u32, c, extra));
                p += 5;
            } else {
                p += 1;
            }
        }
        Ok(())
    }

    /// Exact #739 src/rle.rs:103-170 double-shift in-place RLE1 + 64 KiB tiled CLMUL CRC
    pub fn unrle_739_in_place(buf: &mut Vec<u8>, runs: &mut Vec<Run>, k: Kernels) -> Result<u32, ErrorKind> {
        let n = buf.len();
        find_runs_swar(buf, runs)?;
        let Some(&(first, _, _)) = runs.first() else {
            return Ok(mibz2::crc::crc32(buf, k));
        };
        let (mut grown, mut shift) = (0isize, 0isize);
        for &(_, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            shift = shift.max(grown);
        }
        let (shift, len) = (shift as usize, (n as isize + grown) as usize);
        let first = first as usize;
        if shift > 0 {
            if n + shift > buf.capacity() { buf.reserve_exact(n + shift - buf.len()); }
            buf.resize(n + shift, 0);
            buf.copy_within(first..n, first + shift);
        }
        let (mut s, mut at) = (!0u32, 0usize);
        let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= TILE {
                *s = mibz2::crc::update_with(*s, &buf[*at..*at + TILE], k);
                *at += TILE;
            }
        };
        tiles(&mut s, &mut at, buf, first);
        let (mut src, mut dst) = (first + shift, first);
        for &(j, c, extra) in runs.iter() {
            let from = j as usize + shift;
            if from > src {
                if dst != src { buf.copy_within(src..from, dst); }
                dst += from - src;
            }
            let run = 4 + extra as usize;
            buf[dst..dst + run].fill(c);
            dst += run;
            src = from + 5;
            tiles(&mut s, &mut at, buf, dst);
        }
        if n + shift > src && dst != src { buf.copy_within(src..n + shift, dst); }
        buf.truncate(len);
        s = mibz2::crc::update_with(s, &buf[at..], k);
        Ok(!s)
    }

    /// Theorem 52.1: Skip-Free Zero-Shift In-Place RLE1 + 64 KiB tiled CLMUL CRC (0 pre-shift, 0 staging fallback)
    pub fn unrle_zero_shift52(buf: &mut Vec<u8>, runs: &mut Vec<Run>, k: Kernels) -> Result<u32, ErrorKind> {
        let n = buf.len();
        find_runs_swar(buf, runs)?;
        if runs.is_empty() {
            return Ok(mibz2::crc::crc32(buf, k));
        }
        let mut grown = 0isize;
        let mut max_end = n;
        for &(j, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            let end_after = (j as usize + 5) as isize + grown;
            if end_after as usize > max_end { max_end = end_after as usize; }
        }
        let final_len = (n as isize + grown) as usize;
        if final_len > max_end { max_end = final_len; }
        if max_end > buf.len() {
            if max_end > buf.capacity() { buf.reserve_exact(max_end - buf.len()); }
            buf.resize(max_end, 0);
        }
        let (mut crc_s, mut crc_at) = (!0u32, 0usize);
        let crc_flush = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= TILE {
                *s = mibz2::crc::update_with(*s, &buf[*at..*at + TILE], k);
                *at += TILE;
            }
        };
        let rlen = runs.len();
        let (mut idx, mut src_pos, mut g) = (0usize, 0usize, 0isize);
        while idx < rlen {
            let (j0, c0, e0) = runs[idx];
            let j0u = j0 as usize;
            let g_after0 = g + (e0 as isize - 1);
            if g <= 0 && g_after0 <= 0 {
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos { buf.copy_within(src_pos..j0u, dst_lit); }
                let dst_run = (j0u as isize + g) as usize;
                let rl = 4 + e0 as usize;
                buf[dst_run..dst_run + rl].fill(c0);
                src_pos = j0u + 5;
                g = g_after0;
                idx += 1;
                crc_flush(&mut crc_s, &mut crc_at, buf, dst_run + rl);
            } else {
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos { buf.copy_within(src_pos..j0u, dst_lit); }
                crc_flush(&mut crc_s, &mut crc_at, buf, (j0u as isize + g) as usize);
                let hill_start = idx;
                let mut cur_g = g_after0;
                let mut k_i = idx + 1;
                let mut closed = false;
                while k_i < rlen {
                    let next_g = cur_g + (runs[k_i].2 as isize - 1);
                    if next_g == 0 { closed = true; break; }
                    cur_g = next_g;
                    k_i += 1;
                }
                let (hill_end, mut se, mut de, next_src, next_g) = if closed {
                    let jk = runs[k_i].0 as usize;
                    (k_i + 1, jk + 5, jk + 5, jk + 5, 0isize)
                } else {
                    (rlen, n, (n as isize + cur_g) as usize, n, cur_g)
                };
                for r_i in (hill_start..hill_end).rev() {
                    let (rj, rc, re) = runs[r_i];
                    let rju = rj as usize;
                    let lit_s = rju + 5;
                    if se > lit_s {
                        let ll = se - lit_s;
                        de -= ll;
                        if de != lit_s { buf.copy_within(lit_s..se, de); }
                    }
                    let rl = 4 + re as usize;
                    de -= rl;
                    buf[de..de + rl].fill(rc);
                    se = rju;
                }
                let hill_dst_end = if closed { runs[k_i].0 as usize + 5 } else { (n as isize + cur_g) as usize };
                crc_flush(&mut crc_s, &mut crc_at, buf, hill_dst_end);
                idx = hill_end;
                src_pos = next_src;
                g = next_g;
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

    pub struct OwnedWorkerState {
        pub inv: OpenCells,
        pub runs: Runs,
        pub rle_runs: Vec<Run>,
        pub fallback_sc: Option<Scratch<OpenCells>>,
    }
    impl OwnedWorkerState {
        pub fn new() -> Self {
            Self { inv: OpenCells::default(), runs: Runs::default(), rle_runs: Vec::new(), fallback_sc: None }
        }
        /// True #739 owned run-route (0 B s.ll!) inverting directly into `out` + pluggable in-place RLE1+CRC!
        pub fn decode_owned(
            &mut self,
            b: &Symbols,
            k: Kernels,
            use_zs52: bool,
            take_out: impl FnOnce(usize) -> Vec<u8>,
        ) -> Result<(u32, Vec<u8>), ErrorKind> {
            if self.inv.reads_runs() && runs::takes_runs(b) {
                let n = runs::front(b, &mut self.runs, k)?;
                if b.orig_ptr as usize >= n { return Err(ErrorKind::BadOrigPtr); }
                let mut out = take_out(n);
                let done = self.inv.invert_runs(&self.runs, n, b.orig_ptr, &mut out, k);
                if !done { return Err(ErrorKind::Internal("records don't tile".into())); }
                let computed = if use_zs52 {
                    unrle_zero_shift52(&mut out, &mut self.rle_runs, k)?
                } else {
                    unrle_739_in_place(&mut out, &mut self.rle_runs, k)?
                };
                if computed != b.crc {
                    return Err(ErrorKind::BlockCrc { stored: b.crc, computed });
                }
                return Ok((computed, out));
            }
            let sc = self.fallback_sc.get_or_insert_with(Scratch::<OpenCells>::default);
            let mut out = take_out(b.max_len);
            out.clear();
            let crc = mibz2::decode_block(b, k, sc, &mut out)?;
            Ok((crc, out))
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

    pub struct WindowBuf { pub abs_base_byte: u64, pub owned_end_byte: u64, pub data: Vec<u8>, pub eof: bool }
    enum SpecOutcome { Block { abs_end_bit: u64, crc: u32, out: Vec<u8> }, NeedMoreOrTrunc, Err(ErrorKind) }
    struct SpecEntry { abs_start_bit: u64, level: u8, outcome: SpecOutcome }
    struct SpanTask { win: Arc<WindowBuf>, cand_seq: usize, abs_start_bit: u64, rel_bit: u64, level: u8 }

    pub fn decode_pr4_owned<R: std::io::Read + Send, F: SpanFinder + Send + Sync>(
        mut reader: R, finder: &F, win_cap: usize, max_live_wins: usize, workers: usize, k: Kernels, use_zs52: bool,
        mut on_block: impl FnMut(&[u8]),
    ) -> Result<(usize, usize), Error> {
        let win_cap = win_cap.max(64 << 10);
        let slots = (4 * workers).max(workers);
        let ring_cap = (slots * 2).next_power_of_two();
        let floor = workers + 2;

        struct TaskQ { q: VecDeque<SpanTask>, closed: bool, waiting: usize, free_win: Vec<Vec<u8>> }
        let tq = Arc::new((
            Mutex::new(TaskQ { q: VecDeque::with_capacity(256), closed: false, waiting: 0, free_win: Vec::with_capacity(max_live_wins) }),
            Condvar::new(), Condvar::new(),
        ));
        let live_wins = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicBool::new(false));

        struct ReadyRing {
            ring: Vec<Option<SpecEntry>>, mask: usize, taken_seq: usize, ready_bytes: usize,
            rw: bool, ww: usize, free_out: Vec<Vec<u8>>, win_log: VecDeque<Arc<WindowBuf>>, win_closed: bool, published_seq: usize,
        }
        let ready = Arc::new(Mutex::new(ReadyRing {
            ring: (0..ring_cap).map(|_| None).collect(), mask: ring_cap - 1, taken_seq: 0, ready_bytes: 0,
            rw: false, ww: 0, free_out: Vec::with_capacity(slots), win_log: VecDeque::with_capacity(4), win_closed: false, published_seq: 0,
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let fast_hits = AtomicUsize::new(0);
        let fallbacks = AtomicUsize::new(0);

        let release_win = |w: Arc<WindowBuf>, tq_ref: &(Mutex<TaskQ>, Condvar, Condvar)| {
            if let Ok(wb) = Arc::try_unwrap(w) {
                let mut q = tq_ref.0.lock().unwrap();
                q.free_win.push(wb.data); drop(q); tq_ref.2.notify_one();
            }
        };

        std::thread::scope(|s| -> Result<(), Error> {
            let (tq_p, lw_p, rw_p, cvr_p, stp_p) = (tq.clone(), live_wins.clone(), ready.clone(), cv_r.clone(), stopped.clone());
            let prod = s.spawn(move || -> Result<(), Error> {
                let (mut tail_carry, mut cands) = (Vec::<u8>::new(), Vec::with_capacity(128));
                let (mut abs_base, mut min_next_abs_bit, mut last_level, mut next_seq, mut eof) = (0u64, 0u64, 0u8, 0usize, false);
                while !eof && !stp_p.load(Ordering::Relaxed) {
                    let mut buf = {
                        let mut q = tq_p.0.lock().unwrap();
                        loop {
                            if stp_p.load(Ordering::Relaxed) { return Ok(()); }
                            if let Some(v) = q.free_win.pop() { break v; }
                            if lw_p.load(Ordering::Acquire) < max_live_wins {
                                lw_p.fetch_add(1, Ordering::AcqRel);
                                break vec![0u8; win_cap];
                            }
                            q = tq_p.2.wait(q).unwrap();
                        }
                    };
                    if buf.len() < win_cap { buf.resize(win_cap, 0); }
                    let mut filled = tail_carry.len();
                    if filled > 0 { buf[..filled].copy_from_slice(&tail_carry); tail_carry.clear(); }
                    while filled < win_cap {
                        match reader.read(&mut buf[filled..win_cap]) {
                            Ok(0) => { eof = true; break; }
                            Ok(n) => { filled += n; }
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                            Err(e) => return Err(Error { kind: ErrorKind::Io(e.kind(), e.to_string()), block: None, bit: Some(abs_base * 8) }),
                        }
                    }
                    buf.truncate(filled);
                    if buf.is_empty() && eof { break; }

                    finder.find_cands(&buf, 0, last_level, &mut cands);
                    cands.retain(|c| abs_base * 8 + c.rel_bit >= min_next_abs_bit);
                    if let Some(last) = cands.last() { last_level = last.guessed_level; }

                    let max_carry = (win_cap / 2).max(16);
                    let publish_n = if !eof && cands.len() >= 2 {
                        let cut_byte = (cands.last().unwrap().rel_bit / 8) as usize;
                        if buf.len() - cut_byte <= max_carry {
                            tail_carry.extend_from_slice(&buf[cut_byte..]); cands.len() - 1
                        } else {
                            let keep = buf.len().min(16); tail_carry.extend_from_slice(&buf[buf.len() - keep..]); cands.len()
                        }
                    } else if !eof && cands.len() == 1 {
                        let cut_byte = (cands[0].rel_bit / 8) as usize;
                        if buf.len() - cut_byte <= max_carry {
                            tail_carry.extend_from_slice(&buf[cut_byte..]); 0
                        } else {
                            let keep = buf.len().min(16); tail_carry.extend_from_slice(&buf[buf.len() - keep..]); 1
                        }
                    } else if !eof {
                        let keep = buf.len().min(16); tail_carry.extend_from_slice(&buf[buf.len() - keep..]); 0
                    } else { cands.len() };
                    let next_abs_base = abs_base + (buf.len() - tail_carry.len()) as u64;
                    if publish_n > 0 { min_next_abs_bit = abs_base * 8 + cands[publish_n - 1].rel_bit + 1; }

                    let win_arc = Arc::new(WindowBuf { abs_base_byte: abs_base, owned_end_byte: next_abs_base, data: buf, eof });
                    { let mut st = rw_p.lock().unwrap(); st.win_log.push_back(Arc::clone(&win_arc)); if st.rw { cvr_p.notify_one(); } }
                    if publish_n > 0 {
                        let mut q = tq_p.0.lock().unwrap();
                        for c in &cands[..publish_n] {
                            q.q.push_back(SpanTask {
                                win: Arc::clone(&win_arc), cand_seq: next_seq,
                                abs_start_bit: abs_base * 8 + c.rel_bit, rel_bit: c.rel_bit, level: c.guessed_level,
                            });
                            next_seq += 1;
                        }
                        let wake = q.waiting > 0; drop(q);
                        if wake { tq_p.1.notify_all(); }
                    }
                    release_win(win_arc, &tq_p);
                    abs_base = next_abs_base;
                }
                { let mut q = tq_p.0.lock().unwrap(); q.closed = true; } tq_p.1.notify_all();
                { let mut st = rw_p.lock().unwrap(); st.published_seq = next_seq; st.win_closed = true; if st.rw { cvr_p.notify_one(); } }
                Ok(())
            });

            for _ in 0..workers {
                let (tq_w, rw_w, cvr_w, cvw_w, stp_w) = (tq.clone(), ready.clone(), cv_r.clone(), cv_w.clone(), stopped.clone());
                s.spawn(move || {
                    let mut syms = Vec::new();
                    let mut wst = OwnedWorkerState::new();
                    loop {
                        let task = {
                            let mut q = tq_w.0.lock().unwrap();
                            loop {
                                if stp_w.load(Ordering::Relaxed) { return; }
                                if let Some(t) = q.q.pop_front() { break Some(t); }
                                if q.closed { break None; }
                                q.waiting += 1; q = tq_w.1.wait(q).unwrap(); q.waiting -= 1;
                            }
                        };
                        let Some(SpanTask { win, cand_seq, abs_start_bit, rel_bit, level }) = task else { return; };
                        {
                            let mut st = rw_w.lock().unwrap();
                            while !stp_w.load(Ordering::Relaxed) && (
                                cand_seq >= st.taken_seq + slots || (st.ready_bytes >= READY_BYTES && cand_seq >= st.taken_seq + floor)
                            ) { st.ww += 1; st = cvw_w.wait(st).unwrap(); st.ww -= 1; }
                            if stp_w.load(Ordering::Relaxed) { release_win(win, &tq_w); return; }
                        }
                        let mut br = Bits::at(&win.data, rel_bit);
                        let read_res = read_item(&mut br, level as u32, std::mem::take(&mut syms));
                        let over = br.overran();
                        let abs_end_bit = win.abs_base_byte * 8 + br.bit_pos();
                        release_win(win, &tq_w);
                        let outcome = match read_res {
                            Ok(Item::Block(b)) if !over => {
                                let dec = wst.decode_owned(&b, k, use_zs52, |n| {
                                    let want = n + n / 16 + 64;
                                    if want >= POOLED_FROM {
                                        let mut v = rw_w.lock().unwrap().free_out.pop().unwrap_or_else(|| Vec::with_capacity(want));
                                        if v.capacity() < want { v.reserve_exact(want - v.len()); }
                                        v
                                    } else { Vec::with_capacity(want) }
                                });
                                syms = b.syms;
                                match dec {
                                    Ok((crc, out)) => SpecOutcome::Block { abs_end_bit, crc, out },
                                    Err(e) => SpecOutcome::Err(e),
                                }
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
                });
            }

            let mut cur_win: Option<Arc<WindowBuf>> = None;
            let (mut abs_bit, mut first, mut in_stream, mut level, mut combined, mut block_idx) = (0u64, true, false, 0u8, 0u32, 0u64);
            let (mut fb_syms, mut fb_wst) = (Vec::new(), OwnedWorkerState::new());
            let mut pending_spec: Option<SpecEntry> = None;

            let ensure_win = |cur_win: &mut Option<Arc<WindowBuf>>, abs_bit: u64, min_bytes: usize| -> bool {
                loop {
                    if let Some(ref w) = cur_win {
                        let base_b = w.abs_base_byte * 8;
                        let end_b = if w.eof { (w.abs_base_byte + w.data.len() as u64) * 8 } else { w.owned_end_byte * 8 };
                        if abs_bit >= base_b && (abs_bit < end_b && abs_bit + (min_bytes as u64) * 8 <= (w.abs_base_byte + w.data.len() as u64) * 8 || w.eof) {
                            return true;
                        }
                    }
                    if let Some(old) = cur_win.take() { release_win(old, &tq); }
                    let next_w = {
                        let mut st = ready.lock().unwrap();
                        loop {
                            if let Some(w) = st.win_log.pop_front() { break Some(w); }
                            if st.win_closed { break None; }
                            st.rw = true; st = cv_r.wait(st).unwrap();
                        }
                    };
                    match next_w { Some(w) => *cur_win = Some(w), None => return false }
                }
            };
            let take_next_spec = |pending: &mut Option<SpecEntry>| -> Option<SpecEntry> {
                if let Some(e) = pending.take() { return Some(e); }
                let mut st = ready.lock().unwrap();
                loop {
                    let t = st.taken_seq; let idx = t & st.mask;
                    if let Some(entry) = st.ring[idx].take() {
                        let bbytes = match &entry.outcome { SpecOutcome::Block { out, .. } => out.len(), _ => 0 };
                        st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                        st.taken_seq = t + 1;
                        let ww = st.ww > 0; st.rw = false; drop(st);
                        if ww { cv_w.notify_one(); }
                        return Some(entry);
                    }
                    if st.win_closed && st.taken_seq >= st.published_seq { st.rw = false; return None; }
                    st.rw = true; st = cv_r.wait(st).unwrap();
                }
            };

            let res: Result<(), Error> = loop {
                if !in_stream {
                    if !ensure_win(&mut cur_win, abs_bit, 4) {
                        if first { break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(0) }); }
                        break Ok(());
                    }
                    let w = cur_win.as_ref().unwrap();
                    let rem = &w.data[(abs_bit / 8 - w.abs_base_byte) as usize..];
                    if rem.is_empty() && w.eof {
                        if first { break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(abs_bit) }); }
                        break Ok(());
                    }
                    match rem {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => { in_stream = true; level = l - b'0'; combined = 0; abs_bit += 32; }
                        _ => break Err(Error { kind: if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData }, block: None, bit: Some(abs_bit) }),
                    }
                    continue;
                }
                if !ensure_win(&mut cur_win, abs_bit, 12) {
                    break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) });
                }
                let (magic, end_crc, next_bit, m_over, e_over) = {
                    let w = cur_win.as_ref().unwrap();
                    let mut br = Bits::at(&w.data, abs_bit - w.abs_base_byte * 8);
                    let m = ((br.bits(24) as u64) << 24) | (br.bits(24) as u64);
                    let mo = br.overran();
                    let sc = if !mo && m == 0x1772_4538_5090 { br.bits(32) } else { 0 };
                    let eo = br.overran();
                    (m, sc, ((w.abs_base_byte * 8 + br.bit_pos() + 7) / 8) * 8, mo, eo)
                };
                if m_over { break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }); }
                if magic == 0x1772_4538_5090 {
                    if e_over { break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }); }
                    if end_crc != combined { break Err(Error { kind: ErrorKind::StreamCrc { stored: end_crc, computed: combined }, block: Some(block_idx), bit: Some(abs_bit) }); }
                    abs_bit = next_bit; in_stream = false; first = false; continue;
                }
                if magic != 0x3141_5926_5359 { break Err(Error { kind: ErrorKind::BadBlockMagic, block: Some(block_idx), bit: Some(abs_bit) }); }

                let matched = loop {
                    match take_next_spec(&mut pending_spec) {
                        Some(e) if e.abs_start_bit < abs_bit => {
                            if let SpecOutcome::Block { out, .. } = e.outcome {
                                if out.capacity() >= POOLED_FROM { let mut st = ready.lock().unwrap(); if st.free_out.len() < slots { st.free_out.push(out); } }
                            }
                        }
                        Some(e) if e.abs_start_bit == abs_bit && e.level == level => break Some(e),
                        Some(e) => { pending_spec = Some(e); break None; }
                        None => break None,
                    }
                };
                if let Some(entry) = matched {
                    match entry.outcome {
                        SpecOutcome::Block { abs_end_bit, crc, out } => {
                            fast_hits.fetch_add(1, Ordering::Relaxed);
                            combined = combined.rotate_left(1) ^ crc;
                            on_block(&out);
                            if out.capacity() >= POOLED_FROM { let mut st = ready.lock().unwrap(); if st.free_out.len() < slots { st.free_out.push(out); } }
                            abs_bit = abs_end_bit; block_idx += 1; continue;
                        }
                        SpecOutcome::Err(kind) => break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }),
                        SpecOutcome::NeedMoreOrTrunc => {}
                    }
                }
                fallbacks.fetch_add(1, Ordering::Relaxed);
                let (start_byte, bit_rem) = (abs_bit / 8, abs_bit % 8);
                let (mut fb_buf, mut fb_end_byte) = (Vec::<u8>::new(), start_byte);
                loop {
                    if let Some(ref w) = cur_win {
                        let w_end = w.abs_base_byte + w.data.len() as u64;
                        if fb_end_byte < w_end {
                            fb_buf.extend_from_slice(&w.data[(fb_end_byte - w.abs_base_byte) as usize..]);
                            fb_end_byte = w_end;
                        }
                    }
                    let mut br = Bits::at(&fb_buf, bit_rem);
                    let r = read_item(&mut br, level as u32, std::mem::take(&mut fb_syms));
                    let r = match r {
                        Err((_, b)) if br.overran() => Err((ErrorKind::Truncated, b)),
                        Ok(_) if br.overran() => Err((ErrorKind::Truncated, Vec::new())),
                        other => other,
                    };
                    match r {
                        Ok(Item::Block(b)) => {
                            let end_abs = start_byte * 8 + br.bit_pos();
                            let dec = fb_wst.decode_owned(&b, k, use_zs52, |n| Vec::with_capacity(n + n / 16 + 64));
                            fb_syms = b.syms;
                            match dec {
                                Ok((crc, out)) => { combined = combined.rotate_left(1) ^ crc; on_block(&out); abs_bit = end_abs; block_idx += 1; break Ok(()); }
                                Err(kind) => break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }),
                            }
                        }
                        Err((ErrorKind::Truncated, _)) if cur_win.as_ref().map_or(true, |w| !w.eof) => {
                            if let Some(old) = cur_win.take() { release_win(old, &tq); }
                            let next_w = {
                                let mut st = ready.lock().unwrap();
                                loop {
                                    if let Some(nw) = st.win_log.pop_front() { break Some(nw); }
                                    if st.win_closed { break None; }
                                    st.rw = true; st = cv_r.wait(st).unwrap();
                                }
                            };
                            match next_w { Some(nw) => cur_win = Some(nw), None => break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }) }
                        }
                        Err((kind, _)) => break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }),
                        Ok(Item::End(_)) => break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }),
                    }
                }?;
            };
            stopped.store(true, Ordering::Release); tq.1.notify_all(); tq.2.notify_all(); cv_w.notify_all();
            if let Some(old) = cur_win.take() { release_win(old, &tq); }
            let _ = prod.join();
            res
        })?;
        Ok((fast_hits.load(Ordering::Relaxed), fallbacks.load(Ordering::Relaxed)))
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
    let t = match logs.len() { 5 => 2.776, 7 => 2.447, 10 => 2.262, 12 => 2.201, _ => 2.0 };
    let hw = t * (var / n).sqrt();
    (m.exp(), (m - hw).exp(), (m + hw).exp())
}

fn run_m739(data: &[u8], w: usize, k: Kernels) -> (u64, u64) {
    let dec = StreamDecoder::with_inverse::<OpenCells>(w, 4 * w, k);
    let mut h = pr4_owned_zs52::AbHash::new();
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
        let finder = pr4_owned_zs52::FastAnchorFinder::new();
        let r0 = rusage(); let t0 = Instant::now();
        let mut info = String::new();
        let (bytes, hash) = match arm {
            "M739_A" | "M739_B" => run_m739(&data, w, k),
            "PR4_Own_739" | "PR4_Own_ZS52" => {
                let zs = arm == "PR4_Own_ZS52";
                let mut h = pr4_owned_zs52::AbHash::new();
                let (fh, fb) = pr4_owned_zs52::decode_pr4_owned(
                    Cursor::new(&data), &finder, 2 << 20, 3, w, k, zs, |b| h.feed(b),
                ).unwrap();
                info = format!("fh={fh} fb={fb}");
                (h.bytes, h.h)
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

    let exe = std::env::current_exe().unwrap();
    let arms = ["M739_A", "M739_B", "PR4_Own_739", "PR4_Own_ZS52"];
    for &(label, path, w, rounds) in &[
        ("wikidata-l9 4W (210 MB, 431 blks)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 4usize, 10usize),
        ("wikidata-l9 10W (210 MB, 431 blks)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 10usize, 5usize),
        ("wikidata-l1 4W (38 MB, 379 blks)", "/tmp/wormhole_r6/wikidata_l1_36m.bz2", 4usize, 10usize),
        ("hetero-l9 4W (54 MB, 60 blks)", "/tmp/wormhole_r6/hetero_60b.bz2", 4usize, 10usize),
        ("random-l9 4W (11 MB, 12 blks LazyCells)", "/tmp/wormhole_r6/random_l9_11m.bz2", 4usize, 10usize),
    ] {
        println!("\n=== {rounds}-ROUND CYCLE 63 COMPOSED PR4 + OWNED RUN-ROUTE + ZEROSHIFT52 A/B: {label} ===");
        let mut ws: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        let mut us: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
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
                us.entry(arm).or_default().push(p[1].parse().unwrap());
                ss.entry(arm).or_default().push(p[2].parse().unwrap());
                fl.entry(arm).or_default().push(p[3].parse().unwrap());
                vc.entry(arm).or_default().push(p[4].parse().unwrap());
                rs.entry(arm).or_default().push(p[5].parse().unwrap());
                if p.len() > 8 { inf.insert(arm, p[8..].join(" ")); }
            }
        }
        let ma = &ws["M739_A"];
        println!("{:<13} {:>7} {:>24} {:>7} {:>6} {:>7} {:>5} {:>7}  {}", "Arm", "med_ms", "vs M739_A [95% CI]", "user", "sys", "minflt", "vcsw", "rssMB", "Hits");
        for &arm in &arms {
            let mut wv = ws[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut uv = us[arm].clone(); uv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = ss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = fl[arm].clone(); fv.sort();
            let mut cv = vc[arm].clone(); cv.sort();
            let mut rv = rs[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(ws[arm].iter().copied()).collect::<Vec<_>>());
            let m = rounds / 2;
            println!("{:<13} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>7.1} {:>6.2} {:>7} {:>5} {:>7.2}  {}",
                arm, wv[m], rat, lo, hi, uv[m], sv[m], fv[m], cv[m], rv[m], inf.get(arm).map(String::as_str).unwrap_or("-"));
        }
    }
}
