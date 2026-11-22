#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::OpenCells;
use mibz2::{BzReader, Error, ErrorKind, Kernels, Scratch};

pub mod pr4_hardened {
    use super::*;

    pub const DEFAULT_WIN_CAP: usize = 4 << 20;
    pub const MAX_LIVE_WINS: usize = 2;
    pub const READY_BYTES: usize = 12 << 20;
    pub const POOLED_FROM: usize = 64 << 10;

    #[derive(Clone, Copy, Debug)]
    pub struct SpanCand {
        pub rel_bit: u64,
        pub guessed_level: u8,
    }

    pub trait SpanFinder: Send + Sync {
        fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>);
    }

    /// Reference 4-byte disjoint anchor SpanFinder (Mechanisms C38b style test driver).
    pub struct FastAnchorFinder {
        bm: [u64; 1024],
    }

    impl FastAnchorFinder {
        pub fn new() -> Self {
            const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
            let mut bm = [0u64; 1024];
            for shift in 0..8u32 {
                let mut prev = 0u64;
                for extra in 0..256u64 {
                    let v = (prev << 56) | (BLOCK_MAGIC << (8 - shift)) | extra;
                    for byte_pos in 0..6usize {
                        let b0 = ((v >> (48 - 8 * byte_pos)) & 0xFF) as usize;
                        let b1 = ((v >> (40 - 8 * byte_pos)) & 0xFF) as usize;
                        let idx = (b0 << 8) | b1;
                        bm[idx >> 6] |= 1u64 << (idx & 63);
                    }
                    prev = extra;
                }
            }
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
            // Also index "BZ" so BZh headers inside a window refresh guessed_level even if
            // no magic anchor lands right next to it.
            let bz = ((b'B' as usize) << 8) | (b'Z' as usize);
            bm[bz >> 6] |= 1u64 << (bz & 63);
            let zh = ((b'Z' as usize) << 8) | (b'h' as usize);
            bm[zh >> 6] |= 1u64 << (zh & 63);
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
                {
                    *level = win[i+3] - b'0';
                }
                if *level == 0 || i + 6 >= win.len() { continue; }
                let w = ((win[i] as u64) << 48) | ((win[i+1] as u64) << 40) | ((win[i+2] as u64) << 32)
                    | ((win[i+3] as u64) << 24) | ((win[i+4] as u64) << 16) | ((win[i+5] as u64) << 8)
                    | (win[i+6] as u64);
                for sh in 0u32..8 {
                    let m = (w >> (8 - sh)) & 0xFFFF_FFFF_FFFF;
                    if m == BLOCK_MAGIC {
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
            let mut level = init_level;
            let mut last_bit: i64 = -1;
            let start_b = (rel_start_bit / 8) as usize;
            if start_b + 4 <= win.len() && win[start_b] == b'B' && win[start_b+1] == b'Z' && win[start_b+2] == b'h'
                && (b'1'..=b'9').contains(&win[start_b+3])
            {
                level = win[start_b+3] - b'0';
            }
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

    pub struct WindowBuf {
        pub abs_base_byte: u64,
        pub data: Vec<u8>,
        pub eof: bool,
    }

    enum SpecOutcome {
        Block { abs_end_bit: u64, crc: u32, out: Vec<u8> },
        NeedMoreOrTrunc,
        Err(ErrorKind),
    }
    struct SpecEntry {
        abs_start_bit: u64,
        level: u8,
        outcome: SpecOutcome,
    }
    struct SpanTask {
        win: Arc<WindowBuf>,
        cand_seq: usize,
        abs_start_bit: u64,
        rel_bit: u64,
        level: u8,
    }

    #[derive(Default, Debug, Clone, Copy)]
    pub struct Pr4Stats {
        pub fast_hits: usize,
        pub fallbacks: usize,
        pub dropped_fp: usize,
        pub blocks: usize,
        pub windows: usize,
        pub live_wins_peak: usize,
    }

    /// Hardened PR 4 streaming decoder supporting:
    /// - Multi-stream concat (BZh1..BZh9 + empty streams)
    /// - Arbitrary oversized blocks > win_cap (livelock-free strict progress + WinChainSource fallback)
    /// - False-positive candidate storms (> slots bogus cands/window)
    /// - Mid-stream cancellation via on_block returning false (0 deadlocks, 0 leaks)
    /// - Exact bzip2 error & prefix parity with BzReader
    pub fn decode_pr4_hardened<R: Read + Send, F: SpanFinder + Send + Sync>(
        mut reader: R,
        finder: &F,
        win_cap: usize,
        workers: usize,
        k: Kernels,
        mut on_block: impl FnMut(&[u8]) -> bool,
    ) -> Result<Pr4Stats, Error> {
        let win_cap = win_cap.max(4096);
        let slots = (4 * workers).max(workers);
        let ring_cap = (slots * 2).next_power_of_two();
        let floor = workers + 2;

        struct TaskQ {
            q: VecDeque<SpanTask>,
            closed: bool,
            waiting: usize,
            free_win: Vec<Vec<u8>>,
        }
        let tq = Arc::new((
            Mutex::new(TaskQ {
                q: VecDeque::with_capacity(256),
                closed: false,
                waiting: 0,
                free_win: Vec::with_capacity(MAX_LIVE_WINS),
            }),
            Condvar::new(),
            Condvar::new(),
        ));
        let live_wins = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicBool::new(false));
        let prod_err: Arc<Mutex<Option<Error>>> = Arc::new(Mutex::new(None));

        struct ReadyRing {
            ring: Vec<Option<SpecEntry>>,
            mask: usize,
            taken_seq: usize,
            ready_bytes: usize,
            rw: bool,
            ww: usize,
            free_out: Vec<Vec<u8>>,
            win_log: VecDeque<Arc<WindowBuf>>,
            win_closed: bool,
            published_seq: usize,
        }
        let ready = Arc::new(Mutex::new(ReadyRing {
            ring: (0..ring_cap).map(|_| None).collect(),
            mask: ring_cap - 1,
            taken_seq: 0,
            ready_bytes: 0,
            rw: false,
            ww: 0,
            free_out: Vec::with_capacity(slots),
            win_log: VecDeque::with_capacity(4),
            win_closed: false,
            published_seq: 0,
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let mut stats = Pr4Stats::default();
        let win_count = AtomicUsize::new(0);

        let release_win = |w: Arc<WindowBuf>, tq_ref: &(Mutex<TaskQ>, Condvar, Condvar)| {
            if let Ok(wb) = Arc::try_unwrap(w) {
                let mut q = tq_ref.0.lock().unwrap();
                q.free_win.push(wb.data);
                drop(q);
                tq_ref.2.notify_one();
            }
        };

        let res = std::thread::scope(|s| -> Result<(), Error> {
            let (tq_p, lw_p, rw_p, cvr_p, stp_p, perr_p, wc_p) = (
                tq.clone(),
                live_wins.clone(),
                ready.clone(),
                cv_r.clone(),
                stopped.clone(),
                prod_err.clone(),
                &win_count,
            );
            let prod = s.spawn(move || {
                let mut tail_carry: Vec<u8> = Vec::new();
                let mut abs_base: u64 = 0;
                let mut min_next_abs_bit: u64 = 0;
                let mut last_level: u8 = 0;
                let mut next_seq: usize = 0;
                let mut eof = false;
                let mut cands = Vec::with_capacity(128);

                while !eof && !stp_p.load(Ordering::Acquire) {
                    let mut buf = {
                        let mut q = tq_p.0.lock().unwrap();
                        loop {
                            if stp_p.load(Ordering::Acquire) {
                                return;
                            }
                            if let Some(v) = q.free_win.pop() {
                                break v;
                            }
                            if lw_p.load(Ordering::Acquire) < MAX_LIVE_WINS {
                                lw_p.fetch_add(1, Ordering::AcqRel);
                                break vec![0u8; win_cap];
                            }
                            q = tq_p.2.wait(q).unwrap();
                        }
                    };
                    if buf.len() < win_cap {
                        buf.resize(win_cap, 0);
                    }
                    let mut filled = tail_carry.len();
                    if filled > 0 {
                        buf[..filled].copy_from_slice(&tail_carry);
                        tail_carry.clear();
                    }
                    while filled < win_cap {
                        if stp_p.load(Ordering::Acquire) {
                            let mut q = tq_p.0.lock().unwrap();
                            q.free_win.push(buf);
                            return;
                        }
                        match reader.read(&mut buf[filled..win_cap]) {
                            Ok(0) => {
                                eof = true;
                                break;
                            }
                            Ok(n) => {
                                filled += n;
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                            Err(e) => {
                                *perr_p.lock().unwrap() = Some(Error {
                                    kind: ErrorKind::Io(e.kind(), e.to_string()),
                                    block: None,
                                    bit: Some(abs_base * 8),
                                });
                                let mut q = tq_p.0.lock().unwrap();
                                q.free_win.push(buf);
                                q.closed = true;
                                drop(q);
                                tq_p.1.notify_all();
                                let mut st = rw_p.lock().unwrap();
                                st.published_seq = next_seq;
                                st.win_closed = true;
                                if st.rw { cvr_p.notify_one(); }
                                return;
                            }
                        }
                    }
                    buf.truncate(filled);
                    if buf.is_empty() && eof {
                        let mut q = tq_p.0.lock().unwrap();
                        q.free_win.push(buf);
                        break;
                    }
                    wc_p.fetch_add(1, Ordering::Relaxed);

                    finder.find_cands(&buf, 0, last_level, &mut cands);
                    cands.retain(|c| abs_base * 8 + c.rel_bit >= min_next_abs_bit);
                    if let Some(last) = cands.last() {
                        last_level = last.guessed_level;
                    }

                    // Mandatory Invariant #3 (Livelock-Free Strict Progress):
                    // Never carry >= win_cap / 2 bytes on a non-EOF window! If a single block
                    // exceeds win_cap / 2 (oversized block > win_cap), advance strictly by at
                    // least win_cap / 2; WinChainSource fallback decodes it seamlessly across windows.
                    let max_carry = (win_cap / 2).max(16);
                    let publish_n = if !eof && cands.len() >= 2 {
                        let cut_byte = (cands.last().unwrap().rel_bit / 8) as usize;
                        let carry_len = buf.len() - cut_byte;
                        if carry_len <= max_carry {
                            tail_carry.extend_from_slice(&buf[cut_byte..]);
                            cands.len() - 1
                        } else {
                            // Oversized trailing candidate span: keep only 16-byte seam overlap for magic split,
                            // publish all cands in this window (oversized one will return NeedMoreOrTrunc -> WinChainSource).
                            let keep = buf.len().min(16);
                            tail_carry.extend_from_slice(&buf[buf.len() - keep..]);
                            cands.len()
                        }
                    } else if !eof && cands.len() == 1 {
                        let cut_byte = (cands[0].rel_bit / 8) as usize;
                        let carry_len = buf.len() - cut_byte;
                        if carry_len <= max_carry {
                            tail_carry.extend_from_slice(&buf[cut_byte..]);
                            0
                        } else {
                            // Single candidate near start of window spans > win_cap/2 (oversized block!).
                            // Keep 16-byte seam overlap so abs_base strictly advances by win_cap - 16.
                            let keep = buf.len().min(16);
                            tail_carry.extend_from_slice(&buf[buf.len() - keep..]);
                            1
                        }
                    } else if !eof {
                        let keep = buf.len().min(16);
                        tail_carry.extend_from_slice(&buf[buf.len() - keep..]);
                        0
                    } else {
                        cands.len()
                    };

                    let advance = buf.len() - tail_carry.len();
                    debug_assert!(eof || advance > 0, "producer must make strict forward progress");
                    let next_abs_base = abs_base + advance as u64;
                    if publish_n > 0 {
                        min_next_abs_bit = abs_base * 8 + cands[publish_n - 1].rel_bit + 1;
                    }

                    let win_arc = Arc::new(WindowBuf { abs_base_byte: abs_base, data: buf, eof });
                    {
                        let mut st = rw_p.lock().unwrap();
                        st.win_log.push_back(Arc::clone(&win_arc));
                        if st.rw {
                            cvr_p.notify_one();
                        }
                    }
                    if publish_n > 0 {
                        let mut q = tq_p.0.lock().unwrap();
                        for c in &cands[..publish_n] {
                            q.q.push_back(SpanTask {
                                win: Arc::clone(&win_arc),
                                cand_seq: next_seq,
                                abs_start_bit: abs_base * 8 + c.rel_bit,
                                rel_bit: c.rel_bit,
                                level: c.guessed_level,
                            });
                            next_seq += 1;
                        }
                        let wake = q.waiting > 0;
                        drop(q);
                        if wake {
                            tq_p.1.notify_all();
                        }
                    }
                    release_win(win_arc, &tq_p);
                    abs_base = next_abs_base;
                }
                {
                    let mut q = tq_p.0.lock().unwrap();
                    q.closed = true;
                }
                tq_p.1.notify_all();
                {
                    let mut st = rw_p.lock().unwrap();
                    st.published_seq = next_seq;
                    st.win_closed = true;
                    if st.rw {
                        cvr_p.notify_one();
                    }
                }
            });

            for _ in 0..workers {
                let (tq_w, rw_w, cvr_w, cvw_w, stp_w) =
                    (tq.clone(), ready.clone(), cv_r.clone(), cv_w.clone(), stopped.clone());
                s.spawn(move || {
                    let mut syms = Vec::new();
                    let mut sc = Scratch::<OpenCells>::default();
                    loop {
                        let task = {
                            let mut q = tq_w.0.lock().unwrap();
                            loop {
                                if stp_w.load(Ordering::Acquire) {
                                    return;
                                }
                                if let Some(t) = q.q.pop_front() {
                                    break Some(t);
                                }
                                if q.closed {
                                    break None;
                                }
                                q.waiting += 1;
                                q = tq_w.1.wait(q).unwrap();
                                q.waiting -= 1;
                            }
                        };
                        let Some(SpanTask { win, cand_seq, abs_start_bit, rel_bit, level }) = task else {
                            return;
                        };
                        {
                            let mut st = rw_w.lock().unwrap();
                            while !stp_w.load(Ordering::Acquire)
                                && (cand_seq >= st.taken_seq + slots
                                    || (st.ready_bytes >= READY_BYTES && cand_seq >= st.taken_seq + floor))
                            {
                                st.ww += 1;
                                st = cvw_w.wait(st).unwrap();
                                st.ww -= 1;
                            }
                            if stp_w.load(Ordering::Acquire) {
                                release_win(win, &tq_w);
                                return;
                            }
                        }
                        let mut br = Bits::at(&win.data, rel_bit);
                        let read_res = read_item(&mut br, level as u32, std::mem::take(&mut syms));
                        let over = br.overran();
                        let abs_end_bit = win.abs_base_byte * 8 + br.bit_pos();
                        release_win(win, &tq_w);
                        let outcome = match read_res {
                            Ok(Item::Block(b)) if !over => {
                                let is_pool = b.syms.len() >= 2048;
                                let want = b.max_len + b.max_len / 16 + 64;
                                let mut out = if is_pool {
                                    let mut v = rw_w
                                        .lock()
                                        .unwrap()
                                        .free_out
                                        .pop()
                                        .unwrap_or_else(|| Vec::with_capacity(want));
                                    if v.capacity() < want {
                                        v.reserve_exact(want - v.len());
                                    }
                                    v
                                } else {
                                    Vec::new()
                                };
                                out.clear();
                                let dec = mibz2::decode_block(&b, k, &mut sc, &mut out);
                                syms = b.syms;
                                match dec {
                                    Ok(crc) => SpecOutcome::Block { abs_end_bit, crc, out },
                                    Err(e) => SpecOutcome::Err(e),
                                }
                            }
                            Ok(Item::Block(_)) | Ok(Item::End(_)) | Err((ErrorKind::Truncated, _)) => SpecOutcome::NeedMoreOrTrunc,
                            Err(_) if over => SpecOutcome::NeedMoreOrTrunc,
                            Err((e, _)) => SpecOutcome::Err(e),
                        };
                        let bbytes = match &outcome {
                            SpecOutcome::Block { out, .. } => out.len(),
                            _ => 0,
                        };
                        let mut st = rw_w.lock().unwrap();
                        let idx = cand_seq & st.mask;
                        st.ring[idx] = Some(SpecEntry { abs_start_bit, level, outcome });
                        st.ready_bytes += bbytes;
                        let wake = st.rw && cand_seq == st.taken_seq;
                        drop(st);
                        if wake {
                            cvr_w.notify_one();
                        }
                    }
                });
            }

            let mut cur_win: Option<Arc<WindowBuf>> = None;
            let (mut abs_bit, mut first, mut in_stream, mut level, mut combined, mut block_idx) =
                (0u64, true, false, 0u8, 0u32, 0u64);
            let (mut fb_syms, mut fb_sc, mut fb_out) = (Vec::new(), Scratch::<OpenCells>::default(), Vec::new());
            let mut pending_spec: Option<SpecEntry> = None;

            let pop_next_win = |ready: &Mutex<ReadyRing>, cv_r: &Condvar| -> Option<Arc<WindowBuf>> {
                let mut st = ready.lock().unwrap();
                loop {
                    if let Some(w) = st.win_log.pop_front() {
                        return Some(w);
                    }
                    if st.win_closed {
                        return None;
                    }
                    st.rw = true;
                    st = cv_r.wait(st).unwrap();
                }
            };

            let ensure_win = |cur_win: &mut Option<Arc<WindowBuf>>, abs_bit: u64, min_bytes: usize| -> Result<bool, Error> {
                loop {
                    if let Some(ref w) = cur_win {
                        let base_b = w.abs_base_byte * 8;
                        let end_b = (w.abs_base_byte + w.data.len() as u64) * 8;
                        if abs_bit >= base_b && (abs_bit + (min_bytes as u64) * 8 <= end_b || w.eof) {
                            return Ok(true);
                        }
                    }
                    let next_w = pop_next_win(&ready, &cv_r);
                    if let Some(old) = cur_win.take() {
                        release_win(old, &tq);
                    }
                    match next_w {
                        Some(w) => *cur_win = Some(w),
                        None => {
                            if let Some(e) = prod_err.lock().unwrap().take() {
                                return Err(e);
                            }
                            return Ok(false);
                        }
                    }
                }
            };

            let take_next_spec = |pending: &mut Option<SpecEntry>| -> Option<SpecEntry> {
                if let Some(e) = pending.take() {
                    return Some(e);
                }
                let mut st = ready.lock().unwrap();
                loop {
                    let t = st.taken_seq;
                    let idx = t & st.mask;
                    if let Some(entry) = st.ring[idx].take() {
                        let bbytes = match &entry.outcome {
                            SpecOutcome::Block { out, .. } => out.len(),
                            _ => 0,
                        };
                        st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                        st.taken_seq = t + 1;
                        let ww = st.ww > 0;
                        st.rw = false;
                        drop(st);
                        if ww {
                            cv_w.notify_one();
                        }
                        return Some(entry);
                    }
                    if st.win_closed && st.taken_seq >= st.published_seq {
                        st.rw = false;
                        return None;
                    }
                    st.rw = true;
                    st = cv_r.wait(st).unwrap();
                }
            };

            let res: Result<(), Error> = loop {
                if !in_stream {
                    match ensure_win(&mut cur_win, abs_bit, 4) {
                        Err(e) => break Err(e),
                        Ok(false) => {
                            if first {
                                break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(0) });
                            }
                            break Ok(());
                        }
                        Ok(true) => {}
                    }
                    let w = cur_win.as_ref().unwrap();
                    let rem = &w.data[(abs_bit / 8 - w.abs_base_byte) as usize..];
                    if rem.is_empty() && w.eof {
                        if first {
                            break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(abs_bit) });
                        }
                        break Ok(());
                    }
                    match rem {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                            in_stream = true;
                            level = l - b'0';
                            combined = 0;
                            abs_bit += 32;
                        }
                        _ => {
                            break Err(Error {
                                kind: if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData },
                                block: None,
                                bit: Some(abs_bit),
                            });
                        }
                    }
                    continue;
                }

                match ensure_win(&mut cur_win, abs_bit, 12) {
                    Err(e) => break Err(e),
                    Ok(false) => {
                        break Err(Error {
                            kind: ErrorKind::Truncated,
                            block: Some(block_idx),
                            bit: Some(abs_bit),
                        });
                    }
                    Ok(true) => {}
                }
                let (magic, end_crc, next_bit, magic_over, eos_over) = {
                    let w = cur_win.as_ref().unwrap();
                    let mut br = Bits::at(&w.data, abs_bit - w.abs_base_byte * 8);
                    let m = ((br.bits(24) as u64) << 24) | (br.bits(24) as u64);
                    let mo = br.overran();
                    let sc = if !mo && m == 0x1772_4538_5090 { br.bits(32) } else { 0 };
                    let eo = br.overran();
                    (m, sc, ((w.abs_base_byte * 8 + br.bit_pos() + 7) / 8) * 8, mo, eo)
                };
                if magic_over {
                    break Err(Error {
                        kind: ErrorKind::Truncated,
                        block: Some(block_idx),
                        bit: Some(abs_bit),
                    });
                }
                if magic == 0x1772_4538_5090 {
                    if eos_over {
                        break Err(Error {
                            kind: ErrorKind::Truncated,
                            block: Some(block_idx),
                            bit: Some(abs_bit),
                        });
                    }
                    if end_crc != combined {
                        break Err(Error {
                            kind: ErrorKind::StreamCrc { stored: end_crc, computed: combined },
                            block: Some(block_idx),
                            bit: Some(abs_bit),
                        });
                    }
                    abs_bit = next_bit;
                    in_stream = false;
                    first = false;
                    continue;
                }
                if magic != 0x3141_5926_5359 {
                    break Err(Error {
                        kind: ErrorKind::BadBlockMagic,
                        block: Some(block_idx),
                        bit: Some(abs_bit),
                    });
                }

                let matched = loop {
                    match take_next_spec(&mut pending_spec) {
                        Some(e) if e.abs_start_bit < abs_bit => {
                            stats.dropped_fp += 1;
                            if let SpecOutcome::Block { out, .. } = e.outcome {
                                if out.capacity() >= POOLED_FROM {
                                    let mut st = ready.lock().unwrap();
                                    if st.free_out.len() < slots {
                                        st.free_out.push(out);
                                    }
                                }
                            }
                        }
                        Some(e) if e.abs_start_bit == abs_bit && e.level == level => break Some(e),
                        Some(e) if e.abs_start_bit == abs_bit => {
                            // Level mismatch (e.g., BZh changed across seam): drop and fall back.
                            stats.dropped_fp += 1;
                            if let SpecOutcome::Block { out, .. } = e.outcome {
                                if out.capacity() >= POOLED_FROM {
                                    let mut st = ready.lock().unwrap();
                                    if st.free_out.len() < slots {
                                        st.free_out.push(out);
                                    }
                                }
                            }
                            break None;
                        }
                        Some(e) => {
                            pending_spec = Some(e);
                            break None;
                        }
                        None => break None,
                    }
                };
                if let Some(entry) = matched {
                    match entry.outcome {
                        SpecOutcome::Block { abs_end_bit, crc, out } => {
                            stats.fast_hits += 1;
                            stats.blocks += 1;
                            combined = combined.rotate_left(1) ^ crc;
                            let keep_going = on_block(&out);
                            if out.capacity() >= POOLED_FROM {
                                let mut st = ready.lock().unwrap();
                                if st.free_out.len() < slots {
                                    st.free_out.push(out);
                                }
                            }
                            if !keep_going {
                                break Ok(());
                            }
                            abs_bit = abs_end_bit;
                            block_idx += 1;
                            continue;
                        }
                        SpecOutcome::Err(kind) => {
                            break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) });
                        }
                        SpecOutcome::NeedMoreOrTrunc => {}
                    }
                }

                // Fallback path: handles cross-window AND oversized (> win_cap) blocks!
                stats.fallbacks += 1;
                let start_byte = abs_bit / 8;
                let bit_rem = abs_bit % 8;
                let mut fb_buf: Vec<u8> = Vec::new();
                let mut fb_end_byte = start_byte;
                let fb_res: Result<(u64, u32), Error> = loop {
                    if let Some(ref w) = cur_win {
                        let w_end = w.abs_base_byte + w.data.len() as u64;
                        if fb_end_byte < w_end {
                            let off = (fb_end_byte - w.abs_base_byte) as usize;
                            fb_buf.extend_from_slice(&w.data[off..]);
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
                            fb_out.clear();
                            let crc = match mibz2::decode_block(&b, k, &mut fb_sc, &mut fb_out) {
                                Ok(c) => c,
                                Err(kind) => {
                                    fb_syms = b.syms;
                                    break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) });
                                }
                            };
                            fb_syms = b.syms;
                            break Ok((end_abs, crc));
                        }
                        Err((ErrorKind::Truncated, _)) if cur_win.as_ref().map_or(true, |w| !w.eof) => {
                            if let Some(old) = cur_win.take() {
                                release_win(old, &tq);
                            }
                            match pop_next_win(&ready, &cv_r) {
                                Some(nw) => cur_win = Some(nw),
                                None => {
                                    if let Some(ioe) = prod_err.lock().unwrap().take() {
                                        break Err(ioe);
                                    }
                                    break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) });
                                }
                            }
                        }
                        Err((kind, _)) => {
                            if let Some(ioe) = prod_err.lock().unwrap().take() {
                                break Err(ioe);
                            }
                            break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) });
                        }
                        Ok(Item::End(_)) => {
                            break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) });
                        }
                    }
                };
                match fb_res {
                    Ok((end_abs, crc)) => {
                        stats.blocks += 1;
                        combined = combined.rotate_left(1) ^ crc;
                        let keep_going = on_block(&fb_out);
                        if !keep_going {
                            break Ok(());
                        }
                        abs_bit = end_abs;
                        block_idx += 1;
                    }
                    Err(e) => break Err(e),
                }
            };

            // Mandatory Cancel / Shutdown Sequence (0 deadlocks even when all queues/windows full):
            stopped.store(true, Ordering::Release);
            tq.1.notify_all();
            tq.2.notify_all();
            cv_w.notify_all();
            cv_r.notify_all();
            if let Some(old) = cur_win.take() {
                release_win(old, &tq);
            }
            // Also drain remaining tasks/win_log so producer blocked on MAX_LIVE_WINS wakes immediately
            {
                let mut q = tq.0.lock().unwrap();
                q.q.clear();
            }
            {
                let mut st = ready.lock().unwrap();
                st.win_log.clear();
            }
            tq.2.notify_all();
            let _ = prod.join();
            res
        });

        stats.windows = win_count.load(Ordering::Relaxed);
        stats.live_wins_peak = live_wins.load(Ordering::Relaxed);
        res.map(|()| stats)
    }
}

use pr4_hardened::{decode_pr4_hardened, FastAnchorFinder, SpanCand, SpanFinder};

/// Adversarial SpamFinder: wraps FastAnchorFinder and injects `extra_fp` bogus candidates
/// (including random bit offsets and wrong guessed_levels) into every window!
struct SpamFinder<'a> {
    inner: &'a FastAnchorFinder,
    extra_fp: usize,
}
impl<'a> SpanFinder for SpamFinder<'a> {
    fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>) {
        self.inner.find_cands(win, rel_start_bit, init_level, out);
        if win.len() < 32 { return; }
        let mut extra = Vec::with_capacity(self.extra_fp);
        let mut x = 0x9e37_79b9_7f4a_7c15u64 ^ (win.len() as u64);
        let max_bit = ((win.len() - 16) * 8) as u64;
        for _ in 0..self.extra_fp {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let rb = rel_start_bit + (x % max_bit.saturating_sub(rel_start_bit).max(1));
            extra.push(SpanCand { rel_bit: rb, guessed_level: ((x >> 40) as u8 % 9) + 1 });
        }
        out.extend(extra);
        out.sort_unstable_by_key(|c| c.rel_bit);
        out.dedup_by_key(|c| c.rel_bit);
    }
}

/// Hostile TrickleReader: yields 1..max_chunk bytes per call and injects EINTR every 7th call.
struct TrickleReader<'a> {
    data: &'a [u8],
    pos: usize,
    calls: usize,
    max_chunk: usize,
    fail_at: Option<usize>,
}
impl<'a> Read for TrickleReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.calls += 1;
        if self.calls % 7 == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "injected EINTR"));
        }
        if let Some(fa) = self.fail_at {
            if self.pos >= fa {
                return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "injected EPIPE"));
            }
        }
        if self.pos >= self.data.len() {
            return Ok(0);
        }
        let step = ((self.calls * 13) % self.max_chunk).max(1);
        let n = buf.len().min(step).min(self.data.len() - self.pos);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

fn ref_decode(data: &[u8]) -> (Vec<u8>, Result<(), ErrorKind>) {
    let mut r = BzReader::new(data);
    let mut out = Vec::new();
    let mut tmp = [0u8; 65536];
    loop {
        match r.read(&mut tmp) {
            Ok(0) => return (out, Ok(())),
            Ok(n) => out.extend_from_slice(&tmp[..n]),
            Err(e) => {
                let k = e
                    .into_inner()
                    .and_then(|b| b.downcast::<Error>().ok())
                    .map(|e| e.kind)
                    .unwrap_or(ErrorKind::NotBz2);
                return (out, Err(k));
            }
        }
    }
}

fn pr4_decode_all<F: SpanFinder + Send + Sync>(
    data: &[u8],
    finder: &F,
    win_cap: usize,
    workers: usize,
) -> (Vec<u8>, Result<pr4_hardened::Pr4Stats, ErrorKind>) {
    let mut out = Vec::new();
    let res = decode_pr4_hardened(data, finder, win_cap, workers, Kernels::detect(), |blk| {
        out.extend_from_slice(blk);
        true
    })
    .map_err(|e| e.kind);
    (out, res)
}

fn empty_bz2(level: u8) -> Vec<u8> {
    let mut v = vec![b'B', b'Z', b'h', b'0' + level];
    v.extend_from_slice(&[0x17, 0x72, 0x45, 0x38, 0x50, 0x90, 0, 0, 0, 0]);
    v
}

fn main() {
    let finder = FastAnchorFinder::new();
    let wiki_l1 = std::fs::read("/tmp/wormhole_r6/wikidata_l1_36m.bz2").unwrap();
    let random_l9 = std::fs::read("/tmp/wormhole_r6/random_l9_11m.bz2").unwrap();
    let hetero_l9 = std::fs::read("/tmp/wormhole_r6/hetero_60b.bz2").unwrap();
    let wiki_l9 = std::fs::read("/tmp/wormhole_r6/wikidata_431b_210m.bz2").unwrap();

    println!("=== CYCLE 61: PR 4 ADVERSARIAL HARDENING & EDGE-CASE SUITE ===");

    // 1. Multi-stream concat with mixed BZh1 + empty BZh5 + BZh9 + empty BZh1 + BZh1
    let mut multi = Vec::new();
    multi.extend_from_slice(&wiki_l1);
    multi.extend_from_slice(&empty_bz2(5));
    multi.extend_from_slice(&random_l9[..random_l9.len()]); // full valid BZh9 stream
    multi.extend_from_slice(&empty_bz2(1));
    multi.extend_from_slice(&wiki_l1);
    let (ref_out, ref_res) = ref_decode(&multi);
    assert!(ref_res.is_ok());
    for &(wc, w) in &[(4 << 20, 4), (512 << 10, 4)] {
        let (p_out, p_res) = pr4_decode_all(&multi, &finder, wc, w);
        let st = p_res.expect("multi-stream concat must succeed");
        assert_eq!(p_out.len(), ref_out.len(), "multi len mismatch");
        assert_eq!(p_out, ref_out, "multi byte mismatch");
        println!("[PASS] 1. Multi-stream concat (BZh1+empty5+BZh9+empty1+BZh1) win={}K {}W: blocks={} fh={} fb={} fp={} wins={} peak_lw={}",
            wc >> 10, w, st.blocks, st.fast_hits, st.fallbacks, st.dropped_fp, st.windows, st.live_wins_peak);
    }

    // 2. Oversized single block > win_cap (random-l9 has ~904 KiB blocks; test at 256 KiB and 512 KiB windows!)
    let (ref_rand, _) = ref_decode(&random_l9);
    for &(wc, w) in &[(256 << 10, 4), (512 << 10, 4), (1024 << 10, 4)] {
        let (p_out, p_res) = pr4_decode_all(&random_l9, &finder, wc, w);
        let st = p_res.expect("oversized block > win_cap must succeed");
        assert_eq!(p_out, ref_rand, "oversized block byte mismatch at win={}K", wc >> 10);
        assert!(st.live_wins_peak <= pr4_hardened::MAX_LIVE_WINS);
        println!("[PASS] 2. Oversized block > win_cap (904 KiB blk in {}K win, {}W): blocks={} fh={} fb={} wins={} peak_lw={}",
            wc >> 10, w, st.blocks, st.fast_hits, st.fallbacks, st.windows, st.live_wins_peak);
    }

    // 3. Adversarial false-positive storm: 200 bogus cands/window (> 12x slots!) on wikidata-l1 & hetero-l9
    let spam = SpamFinder { inner: &finder, extra_fp: 200 };
    let (ref_l1, _) = ref_decode(&wiki_l1);
    let (p_out, p_res) = pr4_decode_all(&wiki_l1, &spam, 512 << 10, 4);
    let st = p_res.expect("spam finder must succeed");
    assert_eq!(p_out, ref_l1, "spam finder byte mismatch");
    assert!(st.dropped_fp >= 500, "expected heavy FP storm, got {}", st.dropped_fp);
    println!("[PASS] 3. False-positive storm (200 bogus cands/win, 512K win, 4W): blocks={} fh={} fb={} dropped_fp={}",
        st.blocks, st.fast_hits, st.fallbacks, st.dropped_fp);

    // 4. Mid-window early cancellation (0 deadlocks, fast teardown across k in {0, 1, 5, 100, 430})
    for &stop_after in &[0usize, 1, 5, 100, 430] {
        let t0 = Instant::now();
        let mut seen = 0usize;
        let st = decode_pr4_hardened(&wiki_l9[..], &finder, 4 << 20, 4, Kernels::detect(), |_| {
            seen += 1;
            seen < stop_after
        })
        .expect("cancel must return Ok");
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        assert!(seen <= stop_after.max(1));
        assert!(ms < 2000.0, "cancel took too long: {ms:.1} ms");
        println!("[PASS] 4. Early cancel after {stop_after:>3} blks (4W, 4M win): seen={seen} teardown={ms:.2}ms peak_lw={}",
            st.live_wins_peak);
    }

    // 5. Hostile TrickleReader (1..13B reads + EINTR every 7th read) on 64K win
    let mut trickle_out = Vec::new();
    let tr = TrickleReader { data: &wiki_l1[..200_000], pos: 0, calls: 0, max_chunk: 13, fail_at: None };
    let (ref_t, ref_tr) = ref_decode(&wiki_l1[..200_000]);
    let p_tr = decode_pr4_hardened(tr, &finder, 64 << 10, 4, Kernels::detect(), |blk| {
        trickle_out.extend_from_slice(blk);
        true
    }).map_err(|e| e.kind);
    assert_eq!(p_tr.as_ref().err(), ref_tr.as_ref().err(), "trickle error parity");
    assert_eq!(trickle_out, ref_t, "trickle exact prefix parity");
    println!("[PASS] 5. Hostile TrickleReader (1..13B + EINTR, 64K win): prefix={}B err={:?}",
        trickle_out.len(), ref_tr.err());

    // 6. Mid-stream I/O error propagation
    let tr_err = TrickleReader { data: &wiki_l1, pos: 0, calls: 0, max_chunk: 4096, fail_at: Some(150_000) };
    let io_res = decode_pr4_hardened(tr_err, &finder, 64 << 10, 4, Kernels::detect(), |_| true);
    assert!(matches!(io_res, Err(Error { kind: ErrorKind::Io(std::io::ErrorKind::BrokenPipe, _), .. })),
        "expected BrokenPipe Io error, got {io_res:?}");
    println!("[PASS] 6. Mid-stream Io(BrokenPipe) error propagation exact: {:?}", io_res.unwrap_err().kind);

    // 7. Exhaustive bzip2 error-kind & prefix parity matrix vs BzReader
    let mut bad_crc = wiki_l1[..50_000].to_vec();
    bad_crc[12] ^= 0xFF; // corrupt block 0 CRC
    let mut bad_magic = wiki_l1[..50_000].to_vec();
    bad_magic[6] ^= 0xFF; // corrupt block 0 magic
    let mut trailing = empty_bz2(9);
    trailing.extend_from_slice(b"JUNK");
    let mut bad_scrc = empty_bz2(9);
    let n = bad_scrc.len();
    bad_scrc[n - 1] ^= 1;

    let cases: &[(&str, &[u8])] = &[
        ("Empty", b""),
        ("NotBz2", b"PK\x03\x04notbz2"),
        ("Trunc_2B", &wiki_l1[..2]),
        ("Trunc_4B_hdr_only", &wiki_l1[..4]),
        ("Trunc_9B_mid_magic", &wiki_l1[..9]),
        ("Trunc_mid_blk0", &wiki_l1[..500]),
        ("Trunc_mid_stream", &wiki_l1[..500_000]),
        ("Trunc_EOS_minus_1", &wiki_l1[..wiki_l1.len() - 1]),
        ("TrailingData", &trailing),
        ("BadBlockMagic", &bad_magic),
        ("BlockCrc", &bad_crc),
        ("StreamCrc", &bad_scrc),
        ("EmptyValidStream", &empty_bz2(9)),
    ];
    for &(name, bytes) in cases {
        let (r_out, r_err) = ref_decode(bytes);
        let (p_out, p_err) = pr4_decode_all(bytes, &finder, 64 << 10, 4);
        assert_eq!(p_err.err(), r_err.err(), "error mismatch on {name}: pr4 vs ref");
        assert_eq!(p_out, r_out, "prefix byte mismatch on {name}");
    }
    println!("[PASS] 7. All {} error-kind & exact-prefix parity cases match BzReader 100%", cases.len());

    // 8. Bit-offset & level-flip false-positive injection inside raw compressed payload
    // Embed literal BZh1 + 0x314159265359 inside a valid hetero stream by using empty_bz2(1) + hetero_l9
    let mut flip = empty_bz2(1);
    flip.extend_from_slice(&hetero_l9);
    let (r_flip, _) = ref_decode(&flip);
    let (p_flip, st_flip) = pr4_decode_all(&flip, &finder, 4 << 20, 4);
    let st_flip = st_flip.expect("level flip concat must succeed");
    assert_eq!(p_flip, r_flip, "level flip byte mismatch");
    println!("[PASS] 8. Level-flip concat (empty BZh1 + hetero BZh9): blocks={} fh={} fb={} fp={}",
        st_flip.blocks, st_flip.fast_hits, st_flip.fallbacks, st_flip.dropped_fp);

    // 9. Exhaustive byte-by-byte truncation sweep 0..=24 on both wiki_l1 and empty_bz2(9)
    let emp9 = empty_bz2(9);
    for len in 0..=emp9.len() {
        let (r_out, r_err) = ref_decode(&emp9[..len]);
        let (p_out, p_err) = pr4_decode_all(&emp9[..len], &finder, 64 << 10, 4);
        assert_eq!(p_err.err(), r_err.err(), "empty9 trunc len={len} err mismatch");
        assert_eq!(p_out, r_out, "empty9 trunc len={len} out mismatch");
    }
    for len in 0..=24 {
        let (r_out, r_err) = ref_decode(&wiki_l1[..len]);
        let (p_out, p_err) = pr4_decode_all(&wiki_l1[..len], &finder, 64 << 10, 4);
        assert_eq!(p_err.err(), r_err.err(), "wiki_l1 trunc len={len} err mismatch");
        assert_eq!(p_out, r_out, "wiki_l1 trunc len={len} out mismatch");
    }
    println!("[PASS] 9. Exhaustive byte-by-byte truncation sweep (40 prefixes across empty9 & wiki_l1) 100% exact");

    println!("\nALL 9 CYCLE 61 ADVERSARIAL CATEGORIES PASSED 100%!");
}
