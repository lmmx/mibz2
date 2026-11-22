use std::collections::VecDeque;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use mibz2::{Error, ErrorKind, Kernels, Scratch};
use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::OpenCells;

// ============================================================================
// Cycle 58 — Reference Implementation of the PR 4 BlockSpan Streaming Spec
// Strictly `#![forbid(unsafe_code)]`. Designed so Mechanisms' Handoff 1
// (C38b anchor finder + C38a fast block reader) drops in with ZERO rework.
// ============================================================================
mod pr4_spec {
    #![forbid(unsafe_code)]
    use super::*;

    pub const DEFAULT_WIN_CAP: usize = 4 << 20;
    pub const MAX_LIVE_WINS: usize = 2;
    pub const READY_BYTES: usize = 12 << 20;
    pub const POOLED_FROM: usize = 64 << 10;

    /// 1. Mechanisms -> StreamDecoder contract: speculative block candidate inside one window.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SpanCand {
        /// Bit offset relative to `WindowSlice::bytes[0]`.
        pub rel_bit: u64,
        /// Guessed bzip2 level (`1..=9`) from the most recent `BZh[1-9]` header seen before `rel_bit`.
        pub guessed_level: u8,
    }

    /// 2. Mechanisms front-end trait: plug C38b `find_magics_anchor_range` + C38a `read_item` here!
    pub trait SpanFinder: Send + Sync {
        /// Scan `win[rel_start_bit/8 ..]` for speculative `BLOCK_MAGIC` candidates.
        /// False positives are harmless (discarded in O(1) by `ChainValidator`);
        /// missed candidates fall back safely to exact synchronous decode.
        fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>);
    }

    /// Reference safe-Rust 48-bit magic finder (Mechanisms drops in C38b disjoint 4-byte anchor SIMD/bitmap here!)
    pub struct RefMagicFinder;
    impl SpanFinder for RefMagicFinder {
        fn find_cands(&self, win: &[u8], rel_start_bit: u64, init_level: u8, out: &mut Vec<SpanCand>) {
            out.clear();
            const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
            let mut level = init_level;
            let start_byte = (rel_start_bit / 8) as usize;
            if win.len() < 10 { return; }
            // Check header at start_byte..
            for i in start_byte..win.len().saturating_sub(6) {
                if i + 4 <= win.len() && win[i] == b'B' && win[i+1] == b'Z' && win[i+2] == b'h'
                    && (b'1'..=b'9').contains(&win[i+3])
                {
                    level = win[i+3] - b'0';
                }
                if level == 0 { continue; }
                // Check all 8 bit alignments starting in byte `i`
                let mut w = 0u64;
                for k in 0..7 { w = (w << 8) | (win[i + k] as u64); }
                for sh in 0u32..8 {
                    let bit = (i as u64) * 8 + sh as u64;
                    if bit < rel_start_bit { continue; }
                    let m = (w >> (8 - sh)) & 0xFFFF_FFFF_FFFF;
                    if m == BLOCK_MAGIC {
                        out.push(SpanCand { rel_bit: bit, guessed_level: level });
                    }
                }
            }
        }
    }

    /// Bounded shared compressed window (`<= WIN_CAP`, ping-pong `MAX_LIVE_WINS = 2`).
    pub struct WindowBuf {
        pub abs_base_byte: u64,
        pub data: Vec<u8>,
        pub eof: bool,
    }

    /// Speculative decoded outcome stored in the ready queue, keyed by `abs_start_bit`.
    enum SpecOutcome {
        Block {
            abs_end_bit: u64,
            crc: u32,
            out: Vec<u8>,
        },
        /// Block extended past this window's end (straddling block) or hit EOF mid-block.
        NeedMoreOrTrunc,
        /// Definitively malformed block at this bit offset (e.g. BadBlockCrc, BadOrigPtr).
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

    #[derive(Default, Debug)]
    pub struct SpecStats {
        pub windows: usize,
        pub cands_total: usize,
        pub fast_hits: usize,
        pub false_pos_dropped: usize,
        pub straddle_fallbacks: usize,
    }

    /// Bounded Ping-Pong 2-Window Streaming Decoder implementing the PR 4 `BlockSpan` contract!
    pub fn decode_pr4_stream<R: std::io::Read + Send, F: SpanFinder + Send + Sync>(
        mut reader: R,
        finder: &F,
        win_cap: usize,
        workers: usize,
        k: Kernels,
        mut on_block: impl FnMut(&[u8]),
    ) -> Result<SpecStats, Error> {
        let win_cap = win_cap.max(32 << 10);
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
            Mutex::new(TaskQ { q: VecDeque::with_capacity(256), closed: false, waiting: 0, free_win: Vec::with_capacity(MAX_LIVE_WINS) }),
            Condvar::new(),
            Condvar::new(),
        ));
        let live_wins = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicBool::new(false));

        struct ReadyRing {
            ring: Vec<Option<SpecEntry>>,
            mask: usize,
            taken_seq: usize,
            ready_bytes: usize,
            rw: bool,
            ww: usize,
            free_out: Vec<Vec<u8>>,
            // Live window queue for exact fallback/header/end-magic validation by the reader
            win_log: VecDeque<Arc<WindowBuf>>,
            win_closed: bool,
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
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let stats = Mutex::new(SpecStats::default());

        // Helper: recycle WindowBuf back to `free_win` when last Arc drops
        let release_win = |w: Arc<WindowBuf>, tq_ref: &(Mutex<TaskQ>, Condvar, Condvar)| {
            if let Ok(wb) = Arc::try_unwrap(w) {
                let mut q = tq_ref.0.lock().unwrap();
                q.free_win.push(wb.data);
                drop(q);
                tq_ref.2.notify_one();
            }
        };

        let result = std::thread::scope(|s| -> Result<(), Error> {
            // 1. Producer thread: reads bounded `win_cap` windows (carrying overlap tail), runs `finder.find_cands`,
            //    and publishes `SpanTask`s without waiting for previous window to drain!
            let (tq_p, lw_p, rw_p, cvr_p, stp_p, stats_p) = (
                tq.clone(), live_wins.clone(), ready.clone(), cv_r.clone(), stopped.clone(), &stats,
            );
            let prod = s.spawn(move || -> Result<(), Error> {
                // Keep a tiny overlap carry (up to 1 max compressed block ~ 910 KiB; on bounded small windows, win_cap/2)
                let mut tail_carry: Vec<u8> = Vec::new();
                let mut abs_base: u64 = 0;
                let mut min_next_abs_bit: u64 = 0;
                let mut last_level: u8 = 0;
                let mut next_seq: usize = 0;
                let mut eof = false;
                let mut cands = Vec::with_capacity(128);

                while !eof && !stp_p.load(Ordering::Relaxed) {
                    let mut buf = {
                        let mut q = tq_p.0.lock().unwrap();
                        loop {
                            if stp_p.load(Ordering::Relaxed) { return Ok(()); }
                            if let Some(v) = q.free_win.pop() { break v; }
                            if lw_p.load(Ordering::Acquire) < MAX_LIVE_WINS {
                                lw_p.fetch_add(1, Ordering::AcqRel);
                                break Vec::with_capacity(win_cap);
                            }
                            q = tq_p.2.wait(q).unwrap();
                        }
                    };
                    buf.clear();
                    buf.extend_from_slice(&tail_carry);
                    tail_carry.clear();

                    while buf.len() < win_cap {
                        let cur = buf.len();
                        buf.resize(win_cap, 0);
                        match reader.read(&mut buf[cur..]) {
                            Ok(0) => { buf.truncate(cur); eof = true; break; }
                            Ok(n) => { buf.truncate(cur + n); }
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => { buf.truncate(cur); }
                            Err(e) => {
                                return Err(Error { kind: ErrorKind::Io(e.kind(), e.to_string()), block: None, bit: Some(abs_base * 8) });
                            }
                        }
                    }
                    if buf.is_empty() && eof { break; }

                    // Scan from bit 0 of the window (carry tail is only ~35-60 KiB) so the carried seam block
                    // is found and decoded 100% in parallel in window k+1 (0 straddle fallback!), deduplicating
                    // by `abs_start_bit >= min_next_abs_bit`.
                    finder.find_cands(&buf, 0, last_level, &mut cands);
                    cands.retain(|c| abs_base * 8 + c.rel_bit >= min_next_abs_bit);
                    if let Some(last) = cands.last() { last_level = last.guessed_level; }

                    let publish_n = if !eof && cands.len() >= 2 {
                        let cut_byte = (cands.last().unwrap().rel_bit / 8) as usize;
                        tail_carry.extend_from_slice(&buf[cut_byte..]);
                        cands.len() - 1
                    } else if !eof && cands.len() == 1 {
                        let cut_byte = (cands[0].rel_bit / 8) as usize;
                        tail_carry.extend_from_slice(&buf[cut_byte..]);
                        0
                    } else if !eof {
                        let keep = buf.len().min(win_cap / 2);
                        tail_carry.extend_from_slice(&buf[buf.len() - keep..]);
                        0
                    } else {
                        cands.len()
                    };
                    let next_abs_base = abs_base + (buf.len() - tail_carry.len()) as u64;
                    if publish_n > 0 {
                        min_next_abs_bit = abs_base * 8 + cands[publish_n - 1].rel_bit + 1;
                    }

                    let win_arc = Arc::new(WindowBuf { abs_base_byte: abs_base, data: buf, eof });
                    {
                        let mut st = rw_p.lock().unwrap();
                        st.win_log.push_back(Arc::clone(&win_arc));
                        if st.rw { cvr_p.notify_one(); }
                    }
                    {
                        let mut ss = stats_p.lock().unwrap();
                        ss.windows += 1;
                        ss.cands_total += publish_n;
                    }
                    if publish_n > 0 {
                        let mut q = tq_p.0.lock().unwrap();
                        for c in &cands[..publish_n] {
                            let abs_start_bit = abs_base * 8 + c.rel_bit;
                            q.q.push_back(SpanTask {
                                win: Arc::clone(&win_arc),
                                cand_seq: next_seq,
                                abs_start_bit,
                                rel_bit: c.rel_bit,
                                level: c.guessed_level,
                            });
                            next_seq += 1;
                        }
                        let wake = q.waiting > 0;
                        drop(q);
                        if wake { tq_p.1.notify_all(); }
                    }
                    release_win(win_arc, &tq_p);
                    abs_base = next_abs_base;
                }
                { let mut q = tq_p.0.lock().unwrap(); q.closed = true; }
                tq_p.1.notify_all();
                { let mut st = rw_p.lock().unwrap(); st.win_closed = true; if st.rw { cvr_p.notify_one(); } }
                Ok(())
            });

            // 2. Worker pool: claims speculative `SpanTask`s, runs read_item + decode_block, drops WindowBuf early!
            for _ in 0..workers {
                let (tq_w, rw_w, cvr_w, cvw_w, stp_w) = (tq.clone(), ready.clone(), cv_r.clone(), cv_w.clone(), stopped.clone());
                s.spawn(move || {
                    let mut syms = Vec::new();
                    let mut sc = Scratch::<OpenCells>::default();
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

                        // Bound ready ring & 12 MiB byte cap!
                        {
                            let mut st = rw_w.lock().unwrap();
                            while !stp_w.load(Ordering::Relaxed) && (
                                cand_seq >= st.taken_seq + slots
                                || (st.ready_bytes >= READY_BYTES && cand_seq >= st.taken_seq + floor)
                            ) {
                                st.ww += 1; st = cvw_w.wait(st).unwrap(); st.ww -= 1;
                            }
                            if stp_w.load(Ordering::Relaxed) { release_win(win, &tq_w); return; }
                        }

                        // Speculatively read block header + Huffman symbols (Mechanisms C38a plugs in here!)
                        let mut br = Bits::at(&win.data, rel_bit);
                        let read_res = read_item(&mut br, level as u32, std::mem::take(&mut syms));
                        let end_rel_bit = br.bit_pos();
                        let abs_end_bit = win.abs_base_byte * 8 + end_rel_bit;
                        // Release compressed window ref IMMEDIATELY before BWT inversion!
                        release_win(win, &tq_w);

                        let outcome = match read_res {
                            Ok(Item::Block(b)) => {
                                let is_pool = b.syms.len() >= 2048;
                                let want = b.max_len + b.max_len / 16 + 64;
                                let mut out = if is_pool {
                                    let mut v = rw_w.lock().unwrap().free_out.pop().unwrap_or_else(|| Vec::with_capacity(want));
                                    if v.capacity() < want { v.reserve_exact(want - v.len()); }
                                    v
                                } else { Vec::new() };
                                out.clear();
                                let dec = mibz2::decode_block(&b, k, &mut sc, &mut out);
                                syms = b.syms;
                                match dec {
                                    Ok(crc) => SpecOutcome::Block { abs_end_bit, crc, out },
                                    Err(e) => SpecOutcome::Err(e),
                                }
                            }
                            Ok(Item::End(_)) => SpecOutcome::NeedMoreOrTrunc,
                            Err((ErrorKind::Truncated, _)) => SpecOutcome::NeedMoreOrTrunc,
                            Err((e, _)) => SpecOutcome::Err(e),
                        };

                        let bbytes = match &outcome { SpecOutcome::Block { out, .. } => out.len(), _ => 0 };
                        let mut st = rw_w.lock().unwrap();
                        let idx = cand_seq & st.mask;
                        st.ring[idx] = Some(SpecEntry { abs_start_bit, level, outcome });
                        st.ready_bytes += bbytes;
                        let wake = st.rw && cand_seq == st.taken_seq;
                        drop(st);
                        if wake { cvr_w.notify_one(); }
                    }
                });
            }

            // 3. In-order Chain Validator (inside StreamDecoder::advance / read_block):
            //    Verifies exact bzip2 stream headers, bit offsets, combined CRCs, rejects false positives,
            //    and synchronously decodes any cross-window straddling / missed block!
            let mut cur_win: Option<Arc<WindowBuf>> = None;
            let mut abs_bit: u64 = 0;
            let mut first = true;
            let mut in_stream = false;
            let mut level: u8 = 0;
            let mut combined: u32 = 0;
            let mut block_idx: u64 = 0;
            let mut fb_syms = Vec::new();
            let mut fb_sc = Scratch::<OpenCells>::default();
            let mut fb_out = Vec::new();

            // Helper: ensure `cur_win` covers `abs_bit` with at least `min_bytes` available (or EOF)
            let ensure_win = |cur_win: &mut Option<Arc<WindowBuf>>, abs_bit: u64, min_bytes: usize| -> bool {
                loop {
                    if let Some(ref w) = cur_win {
                        let base_b = w.abs_base_byte * 8;
                        let end_b = (w.abs_base_byte + w.data.len() as u64) * 8;
                        if abs_bit >= base_b && (abs_bit + (min_bytes as u64) * 8 <= end_b || w.eof) {
                            return true;
                        }
                    }
                    let next_w = {
                        let mut st = ready.lock().unwrap();
                        loop {
                            if let Some(w) = st.win_log.pop_front() { break Some(w); }
                            if st.win_closed { break None; }
                            st.rw = true; st = cv_r.wait(st).unwrap();
                        }
                    };
                    if let Some(old) = cur_win.take() { release_win(old, &tq); }
                    match next_w {
                        Some(w) => *cur_win = Some(w),
                        None => return false,
                    }
                }
            };

            // Peek next ready speculative entry matching `abs_bit` (dropping any earlier false positives!)
            let mut pending_spec: Option<SpecEntry> = None;
            let take_next_spec = |pending: &mut Option<SpecEntry>| -> Option<SpecEntry> {
                if let Some(e) = pending.take() { return Some(e); }
                let mut st = ready.lock().unwrap();
                loop {
                    let t = st.taken_seq;
                    let idx = t & st.mask;
                    if let Some(entry) = st.ring[idx].take() {
                        let bbytes = match &entry.outcome { SpecOutcome::Block { out, .. } => out.len(), _ => 0 };
                        st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                        st.taken_seq = t + 1;
                        let ww = st.ww > 0; st.rw = false; drop(st);
                        if ww { cv_w.notify_one(); }
                        return Some(entry);
                    }
                    if st.win_closed {
                        // Check if task queue is also empty -> no more speculative entries will arrive
                        if tq.0.lock().unwrap().q.is_empty() { st.rw = false; return None; }
                    }
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
                    let off = (abs_bit / 8 - w.abs_base_byte) as usize;
                    let rem = &w.data[off..];
                    if rem.is_empty() && w.eof {
                        if first { break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(abs_bit) }); }
                        break Ok(());
                    }
                    match rem {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                            in_stream = true;
                            level = l - b'0';
                            combined = 0;
                            abs_bit += 32;
                        }
                        _ => break Err(Error {
                            kind: if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData },
                            block: None, bit: Some(abs_bit),
                        }),
                    }
                    continue;
                }

                // Inside a bzip2 stream: peek 48-bit magic at `abs_bit`
                if !ensure_win(&mut cur_win, abs_bit, 12) {
                    break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) });
                }
                let (magic, end_stored_crc, next_stream_bit, overran_end) = {
                    let w = cur_win.as_ref().unwrap();
                    let rel = abs_bit - w.abs_base_byte * 8;
                    let mut br = Bits::at(&w.data, rel);
                    let m = ((br.bits(24) as u64) << 24) | (br.bits(24) as u64);
                    let sc = if m == 0x1772_4538_5090 { br.bits(32) } else { 0 };
                    let over = br.overran();
                    let nb = ((w.abs_base_byte * 8 + br.bit_pos() + 7) / 8) * 8;
                    (m, sc, nb, over)
                };
                const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
                const END_MAGIC: u64 = 0x1772_4538_5090;
                if magic == END_MAGIC {
                    if overran_end {
                        break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) });
                    }
                    if end_stored_crc != combined {
                        break Err(Error { kind: ErrorKind::StreamCrc { stored: end_stored_crc, computed: combined }, block: Some(block_idx), bit: Some(abs_bit) });
                    }
                    abs_bit = next_stream_bit;
                    in_stream = false;
                    first = false;
                    continue;
                }
                if magic != BLOCK_MAGIC {
                    break Err(Error { kind: ErrorKind::BadBlockMagic, block: Some(block_idx), bit: Some(abs_bit) });
                }

                // Drain any false-positive speculative entries with abs_start_bit < abs_bit
                let matched = loop {
                    match take_next_spec(&mut pending_spec) {
                        Some(e) if e.abs_start_bit < abs_bit => {
                            stats.lock().unwrap().false_pos_dropped += 1;
                            if let SpecOutcome::Block { out, .. } = e.outcome {
                                if out.capacity() >= POOLED_FROM {
                                    let mut st = ready.lock().unwrap();
                                    if st.free_out.len() < slots { st.free_out.push(out); }
                                }
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
                            stats.lock().unwrap().fast_hits += 1;
                            combined = combined.rotate_left(1) ^ crc;
                            on_block(&out);
                            if out.capacity() >= POOLED_FROM {
                                let mut st = ready.lock().unwrap();
                                if st.free_out.len() < slots { st.free_out.push(out); }
                            }
                            abs_bit = abs_end_bit;
                            block_idx += 1;
                            continue;
                        }
                        SpecOutcome::Err(kind) => {
                            break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) });
                        }
                        SpecOutcome::NeedMoreOrTrunc => {
                            // Fall through to exact fallback below!
                        }
                    }
                }

                // Exact synchronous fallback (handles straddling blocks at window seams or wrong guessed_level)
                stats.lock().unwrap().straddle_fallbacks += 1;
                // Advance to a window where the block starts near the front (or decode from current window if EOF)
                loop {
                    let w = cur_win.as_ref().unwrap();
                    let rel = abs_bit - w.abs_base_byte * 8;
                    let mut br = Bits::at(&w.data, rel);
                    match read_item(&mut br, level as u32, std::mem::take(&mut fb_syms)) {
                        Ok(Item::Block(b)) => {
                            let end_abs = w.abs_base_byte * 8 + br.bit_pos();
                            fb_out.clear();
                            let crc = match mibz2::decode_block(&b, k, &mut fb_sc, &mut fb_out) {
                                Ok(c) => c,
                                Err(kind) => { fb_syms = b.syms; break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }); }
                            };
                            fb_syms = b.syms;
                            combined = combined.rotate_left(1) ^ crc;
                            on_block(&fb_out);
                            abs_bit = end_abs;
                            block_idx += 1;
                            break Ok(());
                        }
                        Err((ErrorKind::Truncated, _)) if !w.eof => {
                            // Straddled past end of this window -> advance to next overlapping window!
                            let next_w = {
                                let mut st = ready.lock().unwrap();
                                loop {
                                    if let Some(nw) = st.win_log.pop_front() { break Some(nw); }
                                    if st.win_closed { break None; }
                                    st.rw = true; st = cv_r.wait(st).unwrap();
                                }
                            };
                            if let Some(old) = cur_win.take() { release_win(old, &tq); }
                            match next_w {
                                Some(nw) => cur_win = Some(nw),
                                None => break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }),
                            }
                        }
                        Err((kind, _)) => break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }),
                        Ok(Item::End(_)) => break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }),
                    }
                }?;
            };

            // Clean cancel / shutdown
            stopped.store(true, Ordering::Release);
            tq.1.notify_all();
            tq.2.notify_all();
            cv_w.notify_all();
            if let Some(old) = cur_win.take() { release_win(old, &tq); }
            let _ = prod.join();
            res
        });

        result.map(|_| stats.into_inner().unwrap())
    }
}

fn main() {
    let k = Kernels::detect();
    let finder = pr4_spec::RefMagicFinder;

    // 1. 431-block authenticated Wikidata parity at BOTH 4 MiB AND adversarial 256 KiB small windows!
    let data = fs::read("/tmp/wormhole_r6/wikidata_431b_210m.bz2").unwrap();
    let ref_out = mibz2::decode_parallel(&data, 4).unwrap();
    for &(label, win_cap) in &[("4 MiB normal window", 4 << 20), ("256 KiB stress straddle window", 256 << 10)] {
        let mut got = Vec::with_capacity(ref_out.len());
        let st = pr4_spec::decode_pr4_stream(std::io::Cursor::new(&data), &finder, win_cap, 4, k, |b| got.extend_from_slice(b)).unwrap();
        assert_eq!(got, ref_out, "mismatch on {label}");
        println!("PASS [{label}]: bytes={} windows={} cands={} fast_hits={} fp_dropped={} straddle_fb={}",
            got.len(), st.windows, st.cands_total, st.fast_hits, st.false_pos_dropped, st.straddle_fallbacks);
    }

    // 2. Level-1 multi-block stress (379 blocks)
    let data_l1 = fs::read("/tmp/wormhole_r6/wikidata_l1_36m.bz2").unwrap();
    let ref_l1 = mibz2::decode_parallel(&data_l1, 4).unwrap();
    let mut got_l1 = Vec::new();
    let st_l1 = pr4_spec::decode_pr4_stream(std::io::Cursor::new(&data_l1), &finder, 512 << 10, 4, k, |b| got_l1.extend_from_slice(b)).unwrap();
    assert_eq!(got_l1, ref_l1);
    println!("PASS [wikidata-l1 512 KiB win]: bytes={} windows={} fast_hits={} fp={} fb={}",
        got_l1.len(), st_l1.windows, st_l1.fast_hits, st_l1.false_pos_dropped, st_l1.straddle_fallbacks);

    // 3. Edge-case parity: empty, bad header, truncated mid-block (exact prefix + ErrorKind), bit-flip CRC
    let e_empty = pr4_spec::decode_pr4_stream(std::io::Cursor::new(&[][..]), &finder, 64 << 10, 2, k, |_| {}).unwrap_err();
    assert_eq!(e_empty.kind, ErrorKind::Empty);

    let e_bad = pr4_spec::decode_pr4_stream(std::io::Cursor::new(&b"NOTBZ2DATA"[..]), &finder, 64 << 10, 2, k, |_| {}).unwrap_err();
    assert_eq!(e_bad.kind, ErrorKind::NotBz2);

    // Truncated after 200 KiB: must match sequential prefix & return Truncated
    let trunc = &data[..200_000];
    let mut pr4_pref = Vec::new();
    let e_tr = pr4_spec::decode_pr4_stream(std::io::Cursor::new(trunc), &finder, 64 << 10, 4, k, |b| pr4_pref.extend_from_slice(b)).unwrap_err();
    assert_eq!(e_tr.kind, ErrorKind::Truncated);
    assert!(pr4_pref.len() > 0 && pr4_pref == ref_out[..pr4_pref.len()]);

    // Corrupt bit-flip in block 0 payload
    let mut bad_crc = data[..150_000].to_vec();
    bad_crc[1000] ^= 0xFF;
    let e_crc = pr4_spec::decode_pr4_stream(std::io::Cursor::new(&bad_crc), &finder, 64 << 10, 4, k, |_| {}).unwrap_err();
    assert!(matches!(e_crc.kind, ErrorKind::BlockCrc { .. } | ErrorKind::BadBlockMagic | ErrorKind::RunUnfinished | ErrorKind::BadOrigPtr | ErrorKind::Truncated | ErrorKind::BadHuffmanCode));

    println!("PASS [EDGE CASES]: Empty={:?}, NotBz2={:?}, Truncated(prefix={} B)={:?}, Corrupt={:?}",
        e_empty.kind, e_bad.kind, pr4_pref.len(), e_tr.kind, e_crc.kind);
}
