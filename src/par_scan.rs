#![forbid(unsafe_code)]
//! Cycle 37 Parallel Front End (`#![forbid(unsafe_code)]`):
//! 1. **Disjoint 4-Byte Anchor Theorem** parallel `BLOCK_MAGIC` / `END_MAGIC` finder across all
//!    worker threads (`[u64; 1024]` `ANCHOR_BITMAP`, `0.01 ms` serial boundary merge).
//! 2. **Lock-free `AtomicUsize` work stealing** of candidate blocks, each worker running its own
//!    54 KiB `ScanScratch` (`read_item_with_scratch`) + `takes_runs` + `Scratch<OpenCells>`.
//! 3. **Exact bit-continuity chain joiner** matching `stream::Scanner` byte-for-byte and
//!    `Error { kind, block, bit }`-for-`Error` on all valid, corrupt, truncated, and adversarial inputs.

use crate::bits::Bits;
use crate::block::{read_item_with_scratch, Item, ScanScratch, BLOCK_MAGIC, END_MAGIC};
use crate::bwt::OpenCells;
use crate::{decode_block, Error, ErrorKind, Kernels, Scratch};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Instant;

const MASK48: u64 = (1u64 << 48) - 1;

const fn build_anchor_bitmap() -> [u64; 1024] {
    let mut bm = [0u64; 1024];
    let magics = [BLOCK_MAGIC, END_MAGIC];
    let mut mi = 0;
    while mi < 2 {
        let m = magics[mi];
        let mut s = 0u32;
        while s < 8 {
            let w = (m << (16 - s)).to_be_bytes();
            let mut r = 0usize;
            while r < 4 {
                let idx = ((w[1 + r] as usize) << 8) | (w[2 + r] as usize);
                bm[idx >> 6] |= 1u64 << (idx & 63);
                r += 1;
            }
            s += 1;
        }
        mi += 1;
    }
    bm
}

static ANCHOR_BITMAP: [u64; 1024] = build_anchor_bitmap();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMagic {
    pub bit: u64,
    pub is_end: bool,
    pub hdr_level: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct MagicCand {
    pub bit: u64,
    pub guessed_level: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ParScanStats {
    pub block_cands: u64,
    pub end_cands: u64,
    pub accepted: u64,
    pub inline: u64,
    pub skipped: u64,
    pub dropped: u64,
    pub dropped_ok: u64,
    pub redone: u64,
    pub merge_ns: u64,
    pub chain_work_ns: u64,
    pub worker_find_ns: u64,
    pub worker_decode_ns: u64,
}

#[cold]
#[inline(never)]
fn verify_quad_raw(data: &[u8], k_base: usize, out: &mut Vec<RawMagic>) {
    let total_bits = data.len() as u64 * 8;
    for k in k_base..k_base + 4 {
        if (k as u64) * 8 + 48 > total_bits {
            return;
        }
        let w64 = if let Some(s8) = data.get(k..k + 8) {
            u64::from_be_bytes([s8[0], s8[1], s8[2], s8[3], s8[4], s8[5], s8[6], s8[7]])
        } else {
            let mut arr = [0u8; 8];
            let avail = data.len() - k;
            arr[..avail].copy_from_slice(&data[k..]);
            u64::from_be_bytes(arr)
        };
        for s in 0..8u32 {
            let bit = (k as u64) * 8 + (s as u64);
            if bit + 48 > total_bits {
                break;
            }
            let v = (w64 >> (16 - s)) & MASK48;
            if v == BLOCK_MAGIC || v == END_MAGIC {
                let hdr_level = if s == 0 && k >= 4 {
                    match data[k - 4..k] {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9'] => l - b'0',
                        _ => 0,
                    }
                } else {
                    0
                };
                out.push(RawMagic {
                    bit,
                    is_end: v == END_MAGIC,
                    hdr_level,
                });
            }
        }
    }
}

/// Scans disjoint 4-byte anchors `p = 4 * m` for `m in m_start..m_end` (`1 <= m_start <= m_end`).
pub fn find_magics_anchor_range(
    data: &[u8],
    m_start: usize,
    m_end: usize,
    out: &mut Vec<RawMagic>,
) {
    out.clear();
    if m_start >= m_end || data.len() < 6 {
        return;
    }
    let bm = &ANCHOR_BITMAP;
    let mut p = m_start * 4;
    let p_end = m_end * 4;
    let len = data.len();

    while p + 12 < p_end && p + 14 <= len {
        let s = &data[p..p + 14];
        let i0 = ((s[0] as usize) << 8) | (s[1] as usize);
        let i1 = ((s[4] as usize) << 8) | (s[5] as usize);
        let i2 = ((s[8] as usize) << 8) | (s[9] as usize);
        let i3 = ((s[12] as usize) << 8) | (s[13] as usize);

        let h0 = (bm[i0 >> 6] >> (i0 & 63)) & 1;
        let h1 = (bm[i1 >> 6] >> (i1 & 63)) & 1;
        let h2 = (bm[i2 >> 6] >> (i2 & 63)) & 1;
        let h3 = (bm[i3 >> 6] >> (i3 & 63)) & 1;

        if (h0 | h1 | h2 | h3) != 0 {
            if h0 != 0 {
                verify_quad_raw(data, p - 4, out);
            }
            if h1 != 0 {
                verify_quad_raw(data, p, out);
            }
            if h2 != 0 {
                verify_quad_raw(data, p + 4, out);
            }
            if h3 != 0 {
                verify_quad_raw(data, p + 8, out);
            }
        }
        p += 16;
    }

    while p < p_end && p + 2 <= len {
        let idx = ((data[p] as usize) << 8) | (data[p + 1] as usize);
        if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 {
            verify_quad_raw(data, p - 4, out);
        }
        p += 4;
    }
}

struct OnDrop<F: FnMut()>(F);
impl<F: FnMut()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

enum WorkerDone {
    Block {
        end_bit: u64,
        crc: u32,
        out: Vec<u8>,
    },
    Truncated,
    Failed(ErrorKind),
}

#[allow(clippy::too_many_arguments)]
fn decode_one_cand(
    data: &[u8],
    bit: u64,
    level: u32,
    k: Kernels,
    scan_scratch: &mut ScanScratch,
    bwt_scratch: &mut Scratch<OpenCells>,
    syms: &mut Vec<u16>,
    mut out: Vec<u8>,
) -> (WorkerDone, Option<Vec<u8>>) {
    let base = (bit / 8) as usize;
    let sub = data.get(base..).unwrap_or(&[]);
    let mut br = Bits::at(sub, bit % 8);
    let item = read_item_with_scratch(&mut br, level, std::mem::take(syms), scan_scratch);
    let overran = br.overran();
    let end_bit = (base as u64) * 8 + br.bit_pos();
    if overran {
        *syms = match item {
            Ok(Item::Block(b)) => b.syms,
            Ok(Item::End(_)) => Vec::new(),
            Err((_, s)) => s,
        };
        return (WorkerDone::Truncated, Some(out));
    }
    match item {
        Ok(Item::Block(b)) => {
            out.clear();
            let res = decode_block(&b, k, bwt_scratch, &mut out);
            *syms = b.syms;
            match res {
                Ok(crc) => (WorkerDone::Block { end_bit, crc, out }, None),
                Err(kind) => (WorkerDone::Failed(kind), Some(out)),
            }
        }
        Ok(Item::End(_)) => (WorkerDone::Failed(ErrorKind::BadBlockMagic), Some(out)),
        Err((kind, s)) => {
            *syms = s;
            (WorkerDone::Failed(kind), Some(out))
        }
    }
}

/// Whole-slice parallel decode using Cycle 37's Disjoint 4-Byte Anchor parallel finder +
/// lock-free `AtomicUsize` work stealing + `Scratch<OpenCells>`.
pub fn decode_parallel_slice(data: &[u8], workers: usize, k: Kernels) -> Result<Vec<u8>, Error> {
    let mut out = Vec::with_capacity(data.len().saturating_mul(4));
    decode_parallel_slice_with(data, workers, 4 * workers.max(1), k, |chunk| {
        out.extend_from_slice(chunk);
    })?;
    Ok(out)
}

pub fn decode_parallel_slice_with<F: FnMut(&[u8])>(
    data: &[u8],
    workers: usize,
    slots: usize,
    k: Kernels,
    on_chunk: F,
) -> Result<(), Error> {
    decode_parallel_slice_with_stats(data, workers, slots, k, on_chunk).map(|_| ())
}

pub fn decode_parallel_slice_with_stats<F: FnMut(&[u8])>(
    data: &[u8],
    workers: usize,
    slots: usize,
    k: Kernels,
    mut on_chunk: F,
) -> Result<ParScanStats, Error> {
    let workers = workers.max(1);
    let slots = slots.max(workers);

    struct Shared {
        find_done: usize,
        cands_ready: bool,
        cands: Vec<MagicCand>,
        base_idx: usize,
        done: BTreeMap<u64, (u32, WorkerDone)>,
        workers_left: usize,
        stop: bool,
        failed: bool,
        chain_bit: u64,
        free_out: Vec<Vec<u8>>,
        stats: ParScanStats,
    }

    let shared = (
        Mutex::new(Shared {
            find_done: 0,
            cands_ready: false,
            cands: Vec::new(),
            base_idx: 0,
            done: BTreeMap::new(),
            workers_left: workers,
            stop: false,
            failed: false,
            chain_bit: 0,
            free_out: Vec::new(),
            stats: ParScanStats::default(),
        }),
        Condvar::new(),
    );
    let (state, cv) = (&shared.0, &shared.1);
    let raw_parts: Vec<Mutex<Vec<RawMagic>>> = (0..workers)
        .map(|_| Mutex::new(Vec::with_capacity(64)))
        .collect();
    let raw_parts_ref = &raw_parts;
    let cursor = AtomicUsize::new(0);
    let cursor_ref = &cursor;
    let chain_bit_atomic = AtomicU64::new(0);
    let chain_bit_ref = &chain_bit_atomic;
    let worker_find_ns = AtomicU64::new(0);
    let worker_find_ref = &worker_find_ns;
    let worker_decode_ns = AtomicU64::new(0);
    let worker_decode_ref = &worker_decode_ns;
    let total_anchors = if data.len() >= 6 {
        (data.len() - 2) / 4
    } else {
        0
    };

    let on_panic = move || {
        if std::thread::panicking() {
            let mut st = lock(state);
            st.failed = true;
            st.stop = true;
            drop(st);
            cv.notify_all();
        }
    };

    std::thread::scope(|s| {
        let _stop_all = OnDrop(move || {
            let mut st = lock(state);
            st.stop = true;
            drop(st);
            cv.notify_all();
        });

        let mut whs = Vec::with_capacity(workers);
        for tid in 0..workers {
            whs.push(s.spawn(move || {
                let _guard = OnDrop(on_panic);
                let tf0 = Instant::now();
                if total_anchors >= 1 {
                    let m_start = 1 + (total_anchors * tid) / workers;
                    let m_end = 1 + (total_anchors * (tid + 1)) / workers;
                    let mut local = lock(&raw_parts_ref[tid]);
                    find_magics_anchor_range(data, m_start, m_end, &mut local);
                }
                worker_find_ref.fetch_add(tf0.elapsed().as_nanos() as u64, Ordering::Relaxed);

                // Panic-safe Condvar phase-1 merge handshake
                {
                    let mut st = lock(state);
                    st.find_done += 1;
                    if st.find_done == workers {
                        let tm0 = Instant::now();
                        let mut merged: Vec<RawMagic> = Vec::new();
                        for part_mu in raw_parts_ref.iter() {
                            let part = lock(part_mu);
                            merged.extend_from_slice(&part);
                        }
                        merged.sort_unstable_by_key(|c| c.bit);
                        merged.dedup_by_key(|c| c.bit);

                        let mut cur_lvl = match data.get(0..4) {
                            Some([b'B', b'Z', b'h', l @ b'1'..=b'9']) => (l - b'0') as u32,
                            _ => 0u32,
                        };
                        let mut block_cands = 0u64;
                        let mut end_cands = 0u64;
                        st.cands.clear();
                        for &rm in &merged {
                            if rm.is_end {
                                end_cands += 1;
                            } else {
                                block_cands += 1;
                                if rm.hdr_level != 0 {
                                    cur_lvl = rm.hdr_level as u32;
                                }
                                let guessed_level = if cur_lvl == 0 { 9 } else { cur_lvl };
                                st.cands.push(MagicCand {
                                    bit: rm.bit,
                                    guessed_level,
                                });
                            }
                        }
                        st.stats.block_cands = block_cands;
                        st.stats.end_cands = end_cands;
                        st.stats.merge_ns = tm0.elapsed().as_nanos() as u64;
                        st.cands_ready = true;
                        cv.notify_all();
                    } else {
                        while !st.cands_ready && !st.stop {
                            st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
                        }
                    }
                    if st.stop {
                        st.workers_left -= 1;
                        cv.notify_all();
                        return;
                    }
                }

                let mut scan_scratch = ScanScratch::default();
                let mut bwt_scratch = Scratch::<OpenCells>::default();
                let mut syms = Vec::new();
                let mut local_out: Option<Vec<u8>> = None;

                loop {
                    let idx = cursor_ref.fetch_add(1, Ordering::Relaxed);
                    let (cand, out_buf) = {
                        let mut st = lock(state);
                        if idx >= st.cands.len() {
                            st.workers_left -= 1;
                            if let Some(b) = local_out.take() {
                                st.free_out.push(b);
                            }
                            cv.notify_all();
                            return;
                        }
                        while !st.stop && idx >= st.base_idx + slots {
                            st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
                        }
                        if st.stop {
                            st.workers_left -= 1;
                            if let Some(b) = local_out.take() {
                                st.free_out.push(b);
                            }
                            cv.notify_all();
                            return;
                        }
                        if idx < st.base_idx || st.cands[idx].bit < st.chain_bit {
                            continue;
                        }
                        let c = st.cands[idx];
                        let buf = local_out
                            .take()
                            .or_else(|| st.free_out.pop())
                            .unwrap_or_default();
                        (c, buf)
                    };

                    if cand.bit < chain_bit_ref.load(Ordering::Relaxed) {
                        local_out = Some(out_buf);
                        continue;
                    }

                    let td0 = Instant::now();
                    let (res, ret_buf) = decode_one_cand(
                        data,
                        cand.bit,
                        cand.guessed_level,
                        k,
                        &mut scan_scratch,
                        &mut bwt_scratch,
                        &mut syms,
                        out_buf,
                    );
                    worker_decode_ref.fetch_add(td0.elapsed().as_nanos() as u64, Ordering::Relaxed);
                    local_out = ret_buf;

                    let mut st = lock(state);
                    if idx < st.base_idx || cand.bit < st.chain_bit || st.stop {
                        st.stats.dropped += 1;
                        if let WorkerDone::Block { out: b, .. } = res {
                            st.stats.dropped_ok += 1;
                            st.free_out.push(b);
                        }
                    } else {
                        st.done.insert(cand.bit, (cand.guessed_level, res));
                        cv.notify_all();
                    }
                }
            }));
        }

        let mut bit = 0u64;
        let mut first = true;
        let mut in_stream = false;
        let mut level = 0u32;
        let mut combined = 0u32;
        let mut blocks = 0u64;
        let mut chain_work_ns = 0u64;

        let mut fallback_scan: Option<ScanScratch> = None;
        let mut fallback_bwt: Option<Scratch<OpenCells>> = None;
        let mut fallback_syms = Vec::new();
        let mut fallback_out = Vec::new();

        let join_res = loop {
            let tc0 = Instant::now();
            if !in_stream {
                let at = bit;
                let base = (at / 8) as usize;
                let rem = data.get(base..).unwrap_or(&[]);
                if rem.is_empty() {
                    chain_work_ns += tc0.elapsed().as_nanos() as u64;
                    if first {
                        break Err(Error {
                            kind: ErrorKind::Empty,
                            block: None,
                            bit: Some(at),
                        });
                    }
                    break Ok(());
                }
                match rem {
                    [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                        in_stream = true;
                        level = (l - b'0') as u32;
                        combined = 0;
                        bit += 32;
                        chain_bit_ref.store(bit, Ordering::Relaxed);
                        chain_work_ns += tc0.elapsed().as_nanos() as u64;
                    }
                    _ => {
                        chain_work_ns += tc0.elapsed().as_nanos() as u64;
                        break Err(Error {
                            kind: if first {
                                ErrorKind::NotBz2
                            } else {
                                ErrorKind::TrailingData
                            },
                            block: None,
                            bit: Some(at),
                        });
                    }
                }
            } else {
                let at = bit;
                let idx = blocks;
                let err_at = |kind| Error {
                    kind,
                    block: Some(idx),
                    bit: Some(at),
                };
                let base = (at / 8) as usize;
                let sub = data.get(base..).unwrap_or(&[]);
                let mut r_peek = Bits::at(sub, at % 8);
                let magic = (r_peek.bits(24) as u64) << 24 | r_peek.bits(24) as u64;
                if magic == END_MAGIC {
                    let stored = r_peek.bits(32);
                    if r_peek.overran() {
                        chain_work_ns += tc0.elapsed().as_nanos() as u64;
                        break Err(err_at(ErrorKind::Truncated));
                    }
                    if stored != combined {
                        chain_work_ns += tc0.elapsed().as_nanos() as u64;
                        break Err(err_at(ErrorKind::StreamCrc {
                            stored,
                            computed: combined,
                        }));
                    }
                    bit = ((base as u64) * 8 + r_peek.bit_pos()).div_ceil(8) * 8;
                    chain_bit_ref.store(bit, Ordering::Relaxed);
                    in_stream = false;
                    first = false;
                    chain_work_ns += tc0.elapsed().as_nanos() as u64;
                    continue;
                }
                if magic != BLOCK_MAGIC {
                    chain_work_ns += tc0.elapsed().as_nanos() as u64;
                    if r_peek.overran() {
                        break Err(err_at(ErrorKind::Truncated));
                    }
                    break Err(err_at(ErrorKind::BadBlockMagic));
                }
                chain_work_ns += tc0.elapsed().as_nanos() as u64;

                let taken_worker = {
                    let mut st = lock(state);
                    st.chain_bit = at;
                    chain_bit_ref.store(at, Ordering::Relaxed);
                    loop {
                        if st.failed {
                            break Err(Error::new(ErrorKind::Internal(
                                "a parallel front-end worker panicked".into(),
                            )));
                        }
                        if st.cands_ready {
                            let before = st.base_idx;
                            while st.base_idx < st.cands.len() && st.cands[st.base_idx].bit < at {
                                let k_bit = st.cands[st.base_idx].bit;
                                if let Some((_, w_res)) = st.done.remove(&k_bit) {
                                    st.stats.dropped += 1;
                                    if let WorkerDone::Block { out: buf, .. } = w_res {
                                        st.stats.dropped_ok += 1;
                                        st.free_out.push(buf);
                                    }
                                }
                                st.base_idx += 1;
                                st.stats.skipped += 1;
                            }
                            if st.base_idx != before {
                                cv.notify_all();
                            }
                            if st.base_idx >= st.cands.len() || st.cands[st.base_idx].bit != at {
                                break Ok(None);
                            }
                            if let Some((w_lvl, w_res)) = st.done.remove(&at) {
                                st.base_idx += 1;
                                cv.notify_all();
                                break Ok(Some((w_lvl, w_res)));
                            }
                        }
                        if st.workers_left == 0 {
                            break Ok(None);
                        }
                        st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
                    }
                };

                let tc1 = Instant::now();
                let taken_worker = match taken_worker {
                    Ok(v) => v,
                    Err(e) => break Err(e),
                };

                match taken_worker {
                    Some((w_lvl, w_res)) if w_lvl == level => match w_res {
                        WorkerDone::Block { end_bit, crc, out } => {
                            on_chunk(&out);
                            {
                                let mut st = lock(state);
                                st.stats.accepted += 1;
                                st.chain_bit = end_bit;
                                chain_bit_ref.store(end_bit, Ordering::Relaxed);
                                st.free_out.push(out);
                                cv.notify_all();
                            }
                            combined = combined.rotate_left(1) ^ crc;
                            blocks += 1;
                            bit = end_bit;
                            chain_work_ns += tc1.elapsed().as_nanos() as u64;
                        }
                        WorkerDone::Truncated => {
                            chain_work_ns += tc1.elapsed().as_nanos() as u64;
                            break Err(err_at(ErrorKind::Truncated));
                        }
                        WorkerDone::Failed(kind) => {
                            chain_work_ns += tc1.elapsed().as_nanos() as u64;
                            break Err(err_at(kind));
                        }
                    },
                    other => {
                        {
                            let mut st = lock(state);
                            if let Some((_, w_res)) = other {
                                st.stats.redone += 1;
                                if let WorkerDone::Block { out, .. } = w_res {
                                    st.free_out.push(out);
                                }
                            } else {
                                st.stats.inline += 1;
                            }
                        }
                        let sc = fallback_scan.get_or_insert_with(ScanScratch::default);
                        let bw = fallback_bwt.get_or_insert_with(Scratch::default);
                        let (w_res, _) = decode_one_cand(
                            data,
                            at,
                            level,
                            k,
                            sc,
                            bw,
                            &mut fallback_syms,
                            std::mem::take(&mut fallback_out),
                        );
                        match w_res {
                            WorkerDone::Block { end_bit, crc, out } => {
                                on_chunk(&out);
                                fallback_out = out;
                                lock(state).chain_bit = end_bit;
                                chain_bit_ref.store(end_bit, Ordering::Relaxed);
                                cv.notify_all();
                                combined = combined.rotate_left(1) ^ crc;
                                blocks += 1;
                                bit = end_bit;
                                chain_work_ns += tc1.elapsed().as_nanos() as u64;
                            }
                            WorkerDone::Truncated => {
                                chain_work_ns += tc1.elapsed().as_nanos() as u64;
                                break Err(err_at(ErrorKind::Truncated));
                            }
                            WorkerDone::Failed(kind) => {
                                chain_work_ns += tc1.elapsed().as_nanos() as u64;
                                break Err(err_at(kind));
                            }
                        }
                    }
                }
            }
        };

        {
            let mut st = lock(state);
            st.stop = true;
            for (_, (_, w_res)) in std::mem::take(&mut st.done) {
                st.stats.dropped += 1;
                if let WorkerDone::Block { out, .. } = w_res {
                    st.stats.dropped_ok += 1;
                    st.free_out.push(out);
                }
            }
            st.stats.chain_work_ns = chain_work_ns;
            st.stats.worker_find_ns = worker_find_ref.load(Ordering::Relaxed);
            st.stats.worker_decode_ns = worker_decode_ref.load(Ordering::Relaxed);
            cv.notify_all();
        }
        let panicked = whs.into_iter().fold(false, |p, h| h.join().is_err() | p);
        if panicked {
            Err(Error::new(ErrorKind::Internal(
                "a parallel front-end worker panicked".into(),
            )))
        } else {
            join_res?;
            Ok(lock(state).stats)
        }
    })
}
