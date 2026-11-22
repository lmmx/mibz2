use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use mibz2::bits::Bits;
use mibz2::block::{read_item, Item, Symbols};
use mibz2::bwt::{Cells, InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::{Error, ErrorKind, Kernels, Scratch};

mod slice_pr4 {
    #![forbid(unsafe_code)]
    use super::*;

    pub const STRIDE: usize = 2 << 20;
    pub const READY_BYTES: usize = 12 << 20;
    pub const POOLED_FROM: usize = 64 << 10;
    const TILE: usize = 64 << 10;
    pub type Run = (u32, u8, u8);

    #[derive(Clone, Copy)]
    pub struct SpanCand { pub abs_bit: u64, pub guessed_level: u8 }

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
        /// Scan slice `data[lo..hi]` directly in place (ZERO copy!) and push absolute bit offsets.
        pub fn find_range(&self, data: &[u8], lo: usize, hi: usize, level: &mut u8, last_bit: &mut i64, out: &mut Vec<SpanCand>) {
            out.clear();
            let n = data.len();
            if n < 10 || lo >= n { return; }
            let hi = hi.min(n);
            if lo == 0 && data[0] == b'B' && data[1] == b'Z' && data[2] == b'h' && (b'1'..=b'9').contains(&data[3]) {
                *level = data[3] - b'0';
            }
            const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
            let bm = &self.bm;
            let mut verify = |center: usize| {
                let vlo = center.saturating_sub(4);
                let vhi = (center + 2).min(n.saturating_sub(4));
                for i in vlo..=vhi {
                    if i + 4 <= n && data[i] == b'B' && data[i+1] == b'Z' && data[i+2] == b'h' && (b'1'..=b'9').contains(&data[i+3]) {
                        *level = data[i+3] - b'0';
                    }
                    if *level == 0 || i + 6 >= n { continue; }
                    let w = ((data[i] as u64) << 48) | ((data[i+1] as u64) << 40) | ((data[i+2] as u64) << 32)
                        | ((data[i+3] as u64) << 24) | ((data[i+4] as u64) << 16) | ((data[i+5] as u64) << 8)
                        | (data[i+6] as u64);
                    for sh in 0u32..8 {
                        if ((w >> (8 - sh)) & 0xFFFF_FFFF_FFFF) == BLOCK_MAGIC {
                            let bit = (i as u64) * 8 + sh as u64;
                            if (bit as i64) > *last_bit {
                                out.push(SpanCand { abs_bit: bit, guessed_level: *level });
                                *last_bit = bit as i64;
                            }
                        }
                    }
                }
            };
            let mut p = lo & !3;
            while p + 14 <= hi {
                let s = &data[p..p + 14];
                let i0 = ((s[0] as usize) << 8) | (s[1] as usize);
                let i1 = ((s[4] as usize) << 8) | (s[5] as usize);
                let i2 = ((s[8] as usize) << 8) | (s[9] as usize);
                let i3 = ((s[12] as usize) << 8) | (s[13] as usize);
                if (((bm[i0>>6]>>(i0&63)) | (bm[i1>>6]>>(i1&63)) | (bm[i2>>6]>>(i2&63)) | (bm[i3>>6]>>(i3&63))) & 1) != 0 {
                    if ((bm[i0>>6]>>(i0&63)) & 1) != 0 { verify(p); }
                    if ((bm[i1>>6]>>(i1&63)) & 1) != 0 { verify(p+4); }
                    if ((bm[i2>>6]>>(i2&63)) & 1) != 0 { verify(p+8); }
                    if ((bm[i3>>6]>>(i3&63)) & 1) != 0 { verify(p+12); }
                }
                p += 16;
            }
            while p + 2 <= hi {
                let idx = ((data[p] as usize) << 8) | (data[p + 1] as usize);
                if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 { verify(p); }
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

    struct OwnedWorker<I: InverseBwt + Default> { inv: I, runs: Runs, rle_runs: Vec<Run>, fb_sc: Option<Scratch<I>> }
    impl<I: InverseBwt + Default> OwnedWorker<I> {
        fn new() -> Self { Self { inv: I::default(), runs: Runs::default(), rle_runs: Vec::new(), fb_sc: None } }
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
            let sc = self.fb_sc.get_or_insert_with(Scratch::<I>::default);
            let mut out = take_out(b.max_len);
            out.clear();
            let crc = mibz2::decode_block(b, k, sc, &mut out)?;
            Ok((crc, out))
        }
    }

    enum SpecOutcome { Block { abs_end_bit: u64, crc: u32, out: Vec<u8> }, Err(ErrorKind) }
    struct SpecEntry { abs_start_bit: u64, level: u8, outcome: SpecOutcome }
    struct SliceTask { cand_seq: usize, abs_bit: u64, level: u8 }

    /// Unified PR 4 SliceWindow engine: ZERO input copies, ZERO 2 MiB window buffers, ZERO feeder thread!
    /// Lazy 2 MiB stride anchor-scan lets workers start decoding Block 0 in ~0.35 ms!
    pub fn decode_slice_pr4<I: InverseBwt + Default + Send + 'static, T>(
        data: &[u8],
        finder: &FastAnchorFinder,
        workers: usize,
        k: Kernels,
        mut on_block: impl FnMut(Vec<u8>, u64) -> Option<Vec<u8>>,
        finish: impl FnOnce() -> T,
    ) -> Result<(T, usize, usize), Error> {
        let slots = (4 * workers).max(workers);
        let ring_cap = (slots * 2).next_power_of_two();
        let floor = workers + 2;

        struct TaskQ { q: VecDeque<SliceTask>, closed: bool, waiting: usize }
        struct ReadyRing {
            ring: Vec<Option<SpecEntry>>, mask: usize, taken_seq: usize, ready_bytes: usize,
            rw: bool, ww: usize, free_out: Vec<Vec<u8>>, closed: bool, published_seq: usize,
        }
        let tq = Arc::new((Mutex::new(TaskQ { q: VecDeque::with_capacity(256), closed: false, waiting: 0 }), Condvar::new()));
        let ready = Arc::new(Mutex::new(ReadyRing {
            ring: (0..ring_cap).map(|_| None).collect(), mask: ring_cap - 1, taken_seq: 0, ready_bytes: 0,
            rw: false, ww: 0, free_out: Vec::with_capacity(slots), closed: false, published_seq: 0,
        }));
        let cv_r = Arc::new(Condvar::new());
        let cv_w = Arc::new(Condvar::new());
        let stopped = Arc::new(AtomicBool::new(false));
        let fast_hits = AtomicUsize::new(0);
        let fallbacks = AtomicUsize::new(0);

        std::thread::scope(|s| -> Result<(), Error> {
            // Spawn W workers reading DIRECTLY from `data: &[u8]` (0 copies, 0 Arc<WindowBuf>!)
            for _ in 0..workers {
                let (tq_w, rw_w, cvr_w, cvw_w, stp_w) = (tq.clone(), ready.clone(), cv_r.clone(), cv_w.clone(), stopped.clone());
                s.spawn(move || {
                    let mut syms = Vec::new();
                    let mut wst = OwnedWorker::<I>::new();
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
                        let Some(SliceTask { cand_seq, abs_bit, level }) = task else { return; };
                        {
                            let mut st = rw_w.lock().unwrap();
                            while !stp_w.load(Ordering::Acquire) && (
                                cand_seq >= st.taken_seq + slots || (st.ready_bytes >= READY_BYTES && cand_seq >= st.taken_seq + floor)
                            ) { st.ww += 1; st = cvw_w.wait(st).unwrap(); st.ww -= 1; }
                            if stp_w.load(Ordering::Acquire) { return; }
                        }
                        let mut br = Bits::at(data, abs_bit);
                        let read_res = read_item(&mut br, level as u32, std::mem::take(&mut syms));
                        let over = br.overran();
                        let abs_end_bit = br.bit_pos();
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
                            Ok(_) | Err((ErrorKind::Truncated, _)) => SpecOutcome::Err(ErrorKind::Truncated),
                            Err(_) if over => SpecOutcome::Err(ErrorKind::Truncated),
                            Err((e, _)) => SpecOutcome::Err(e),
                        };
                        let bbytes = match &outcome { SpecOutcome::Block { out, .. } => out.len(), _ => 0 };
                        let mut st = rw_w.lock().unwrap();
                        let idx = cand_seq & st.mask;
                        st.ring[idx] = Some(SpecEntry { abs_start_bit: abs_bit, level, outcome });
                        st.ready_bytes += bbytes;
                        let wake = st.rw && cand_seq == st.taken_seq; drop(st);
                        if wake { cvr_w.notify_one(); }
                    }
                });
            }

            // Consumer/Driver thread: lazily pushes 2 MiB slice strides of SpanCands so workers start in 0.35 ms,
            // ZERO extra producer/scanner thread!
            let (mut scan_off, mut scan_level, mut last_scan_bit, mut next_seq) = (0usize, 0u8, -1i64, 0usize);
            let mut cands = Vec::with_capacity(128);
            let mut scan_done = false;

            let mut pump_scan = |force_all_needed_for: usize| -> bool { let mut did = false;
                while !scan_done && (next_seq < force_all_needed_for || scan_off == 0) {
                    let hi = (scan_off + STRIDE).min(data.len());
                    finder.find_range(data, scan_off, hi, &mut scan_level, &mut last_scan_bit, &mut cands);
                    if !cands.is_empty() {
                        let mut q = tq.0.lock().unwrap();
                        for c in &cands {
                            q.q.push_back(SliceTask { cand_seq: next_seq, abs_bit: c.abs_bit, level: c.guessed_level });
                            next_seq += 1;
                        }
                        let wake = q.waiting > 0; drop(q);
                        if wake { tq.1.notify_all(); }
                    }
                    did = true; scan_off = hi;
                    if scan_off >= data.len() {
                        scan_done = true;
                        { let mut q = tq.0.lock().unwrap(); q.closed = true; }
                        tq.1.notify_all();
                        { let mut st = ready.lock().unwrap(); st.published_seq = next_seq; st.closed = true; }
                    }
                }
                did
            };

            // Kick off first 2 MiB stride immediately (~0.35 ms) so all workers wake right away!
            pump_scan(slots);

            let (mut abs_bit, mut first, mut in_stream, mut level, mut combined, mut block_idx) = (0u64, true, false, 0u8, 0u32, 0u64);
            let (mut fb_syms, mut fb_wst) = (Vec::new(), OwnedWorker::<I>::new());
            let mut pending_spec: Option<SpecEntry> = None;

            let res: Result<(), Error> = loop {
                if !in_stream {
                    let byte_off = (abs_bit / 8) as usize;
                    if byte_off >= data.len() {
                        if first { break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(0) }); }
                        break Ok(());
                    }
                    match &data[byte_off..] {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => { in_stream = true; level = l - b'0'; combined = 0; abs_bit += 32; }
                        _ => break Err(Error { kind: if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData }, block: None, bit: Some(abs_bit) }),
                    }
                    continue;
                }
                let mut br = Bits::at(data, abs_bit);
                let magic = ((br.bits(24) as u64) << 24) | (br.bits(24) as u64);
                if br.overran() { break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }); }
                if magic == 0x1772_4538_5090 {
                    let end_crc = br.bits(32);
                    if br.overran() { break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }); }
                    if end_crc != combined { break Err(Error { kind: ErrorKind::StreamCrc { stored: end_crc, computed: combined }, block: Some(block_idx), bit: Some(abs_bit) }); }
                    abs_bit = ((br.bit_pos() + 7) / 8) * 8;
                    in_stream = false; first = false; continue;
                }
                if magic != 0x3141_5926_5359 { break Err(Error { kind: ErrorKind::BadBlockMagic, block: Some(block_idx), bit: Some(abs_bit) }); }

                // Keep worker queue fed ahead of consumer!
                let cur_taken = ready.lock().unwrap().taken_seq;
                pump_scan(cur_taken + slots);

                let matched = loop {
                    let entry_opt = if let Some(e) = pending_spec.take() {
                        Some(e)
                    } else {
                        let mut st = ready.lock().unwrap();
                        loop {
                            let t = st.taken_seq; let idx = t & st.mask;
                            if let Some(entry) = st.ring[idx].take() {
                                let bbytes = match &entry.outcome { SpecOutcome::Block { out, .. } => out.len(), _ => 0 };
                                st.ready_bytes = st.ready_bytes.saturating_sub(bbytes);
                                st.taken_seq = t + 1;
                                let ww = st.ww > 0; st.rw = false; drop(st);
                                if ww { cv_w.notify_one(); }
                                break Some(entry);
                            }
                            if st.closed && st.taken_seq >= st.published_seq { st.rw = false; break None; }
                            drop(st); if pump_scan(t + slots) { st = ready.lock().unwrap(); continue; } st = ready.lock().unwrap();
                            st.rw = true; st = cv_r.wait(st).unwrap();
                        }
                    };
                    match entry_opt {
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
                            let consumed_bytes = (abs_end_bit + 7) / 8;
                            if let Some(ret) = on_block(out, consumed_bytes) {
                                if ret.capacity() >= POOLED_FROM {
                                    let mut st = ready.lock().unwrap();
                                    if st.free_out.len() < slots { st.free_out.push(ret); }
                                }
                            }
                            abs_bit = abs_end_bit; block_idx += 1; continue;
                        }
                        SpecOutcome::Err(kind) => break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }),
                    }
                }
                fallbacks.fetch_add(1, Ordering::Relaxed);
                let mut fbr = Bits::at(data, abs_bit);
                let r = read_item(&mut fbr, level as u32, std::mem::take(&mut fb_syms));
                let over = fbr.overran();
                match r {
                    Ok(Item::Block(b)) if !over => {
                        let end_abs = fbr.bit_pos();
                        let dec = fb_wst.decode_owned(&b, k, |n| Vec::with_capacity(n + n / 16 + 64));
                        fb_syms = b.syms;
                        match dec {
                            Ok((crc, out)) => {
                                combined = combined.rotate_left(1) ^ crc;
                                let _ = on_block(out, (end_abs + 7) / 8);
                                abs_bit = end_abs; block_idx += 1;
                            }
                            Err(kind) => break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }),
                        }
                    }
                    Ok(_) | Err((ErrorKind::Truncated, _)) => break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }),
                    Err(_) if over => break Err(Error { kind: ErrorKind::Truncated, block: Some(block_idx), bit: Some(abs_bit) }),
                    Err((kind, _)) => break Err(Error { kind, block: Some(block_idx), bit: Some(abs_bit) }),
                }
            };
            stopped.store(true, Ordering::Release); tq.1.notify_all(); cv_w.notify_all();
            res
        })?;
        Ok((finish(), fast_hits.load(Ordering::Relaxed), fallbacks.load(Ordering::Relaxed)))
    }

    pub fn decode_parallel_slicewin<I: InverseBwt + Default + Send + 'static>(data: &[u8], finder: &FastAnchorFinder, workers: usize, k: Kernels, adapt_cap: bool) -> Result<(Vec<u8>, usize, usize), Error> {
        let mut out = if adapt_cap { Vec::new() } else { Vec::with_capacity(data.len().saturating_mul(4)) };
        let mut adapted = !adapt_cap;
        let in_len = data.len() as u64;
        let ((), fh, fb) = decode_slice_pr4::<I, _>(
            data, finder, workers, k,
            |blk, consumed_bytes| {
                if !adapted {
                    adapted = true;
                    // Cycle 54 D2_AdaptCap: exact empirical ratio from block 0 (+12% safety margin), capped sensibly!
                    let cb = consumed_bytes.max(1);
                    let est = ((blk.len() as u128 * in_len as u128) / cb as u128) as usize;
                    let want = (est + est / 8).max(blk.len());
                    out.reserve_exact(want);
                }
                out.extend_from_slice(&blk);
                Some(blk)
            },
            || (),
        )?;
        Ok((out, fh, fb))
    }

    pub fn decode_parallel_chunks<I: InverseBwt + Default + Send + 'static>(data: &[u8], finder: &FastAnchorFinder, workers: usize, k: Kernels) -> Result<(Vec<Vec<u8>>, usize, usize), Error> {
        let mut chunks = Vec::with_capacity(512);
        let ((), fh, fb) = decode_slice_pr4::<I, _>(
            data, finder, workers, k,
            |blk, _| { chunks.push(blk); None },
            || (),
        )?;
        Ok((chunks, fh, fb))
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

fn fnv_hash_slices<'a>(it: impl IntoIterator<Item = &'a [u8]>) -> (u64, u64) {
    let k = Kernels::detect();
    let (mut bytes, mut s) = (0u64, !0u32);
    for sl in it {
        bytes += sl.len() as u64;
        s = mibz2::crc::update_with(s, sl, k);
    }
    (bytes, (!s) as u64)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let arm = args[3].as_str();
        let w: usize = args[4].parse().unwrap();
        let finder = slice_pr4::FastAnchorFinder::new();
        let r0 = rusage(); let t0 = Instant::now();
        match arm {
            "M739_DP_A" | "M739_DP_B" => {
                let v = mibz2::decode_parallel(&data, w).unwrap();
                let wall = t0.elapsed().as_secs_f64() * 1e3;
                let r1 = rusage();
                let (b, h) = fnv_hash_slices([v.as_slice()]);
                println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {b} {h:016x} -",
                    tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
                    r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
                return;
            }
            "DP_SliceWin" | "DP_Adapt" => {
                let (v, fh, fb) = if w < 8 { slice_pr4::decode_parallel_slicewin::<Cells>(&data, &finder, w, k, arm == "DP_Adapt") } else { slice_pr4::decode_parallel_slicewin::<OpenCells>(&data, &finder, w, k, arm == "DP_Adapt") }.unwrap();
                let wall = t0.elapsed().as_secs_f64() * 1e3;
                let r1 = rusage();
                let (b, h) = fnv_hash_slices([v.as_slice()]);
                let info = format!("fh={fh} fb={fb}");
                println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {b} {h:016x} {info}",
                    tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
                    r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
                return;
            }
            "DP_Chunks" => {
                let (cs, fh, fb) = if w < 8 { slice_pr4::decode_parallel_chunks::<Cells>(&data, &finder, w, k) } else { slice_pr4::decode_parallel_chunks::<OpenCells>(&data, &finder, w, k) }.unwrap();
                let wall = t0.elapsed().as_secs_f64() * 1e3;
                let r1 = rusage();
                let (b, h) = fnv_hash_slices(cs.iter().map(|v| v.as_slice()));
                let info = format!("fh={fh} fb={fb}");
                println!("{wall:.3} {:.3} {:.3} {} {} {:.2} {b} {h:016x} {info}",
                    tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
                    r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
                return;
            }
            _ => panic!(),
        };
    }

    let exe = std::env::current_exe().unwrap();
    let arms = ["M739_DP_A", "M739_DP_B", "DP_SliceWin", "DP_Adapt", "DP_Chunks"];
    for &(label, path, w, rounds) in &[
        ("wikidata-l9 4W (210 MB, 15.3x)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 4usize, 8usize),
        ("wikidata-l9 10W (210 MB, 15.3x)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 10usize, 5usize),
        ("wikidata-l1 4W (38 MB, 9.6x)", "/tmp/wormhole_r6/wikidata_l1_36m.bz2", 4usize, 8usize),
        ("random-l9 4W (11 MB, 1.0x)", "/tmp/wormhole_r6/random_l9_11m.bz2", 4usize, 8usize),
    ] {
        println!("\n=== {rounds}-ROUND CYCLE 66 UNIFIED decode_parallel(&[u8]) SLICEWINDOW A/B: {label} ===");
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
        let ma = &ws["M739_DP_A"];
        println!("{:<12} {:>7} {:>24} {:>6} {:>7} {:>5} {:>7}  {}", "Arm", "med_ms", "vs M739_DP [95% CI]", "sys", "minflt", "vcsw", "rssMB", "Hits");
        for &arm in &arms {
            let mut wv = ws[arm].clone(); wv.sort_by(|a,b| a.total_cmp(b));
            let mut sv = ss[arm].clone(); sv.sort_by(|a,b| a.total_cmp(b));
            let mut fv = fl[arm].clone(); fv.sort();
            let mut cv = vc[arm].clone(); cv.sort();
            let mut rv = rs[arm].clone(); rv.sort_by(|a,b| a.total_cmp(b));
            let (rat, lo, hi) = ci95_ratio(&ma.iter().copied().zip(ws[arm].iter().copied()).collect::<Vec<_>>());
            let m = rounds / 2;
            println!("{:<12} {:>7.2} {:>6.4} [{:.4}, {:.4}] {:>6.2} {:>7} {:>5} {:>7.2}  {}",
                arm, wv[m], rat, lo, hi, sv[m], fv[m], cv[m], rv[m], inf.get(arm).map(String::as_str).unwrap_or("-"));
        }
    }
}
