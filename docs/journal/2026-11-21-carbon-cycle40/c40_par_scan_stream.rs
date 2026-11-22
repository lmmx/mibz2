#![forbid(unsafe_code)]
//! Cycle 40 Push-Streaming Parallel Front End (`StreamDecoder::parscan` / `BzReader::parallel_parscan`,
//! `#![forbid(unsafe_code)]`):
//! Solves all four obligations of `docs/journal/2026-11-10-parallel-scan.md` §6:
//! 1. **Streaming input** over rolling `Arc<[u8]>` snapshots with `4`-byte-aligned `4 MiB` window
//!    compaction, `256 KiB` parallel tile-ordered Disjoint 4-Byte Anchor scanning, `64 MiB`
//!    `BlockTooLarge` cap, and worker wait-on-overrun before EOF (`Truncated` decided only at EOF).
//! 2. **Zero-Level-Guess Theorem** (`bound_level = 9`, $O(1)$ chain check `orig_ptr <= 10 + 100_000*L`
//!    and `n_rows <= 100_000*L`, zero re-decodes on valid blocks).
//! 3. **Bounded speculative work & memory** (`idx < base_idx + slots` and `issued - taken < slots`,
//!    deadlock-free head-of-line `idx == base_idx` progress).
//! 4. **1-thread path unchanged** (`BzReader::new` / `mibz2::decode`).

use crate::bits::Bits;
use crate::block::{read_item_with_scratch, Item, ScanScratch, BLOCK_MAGIC, END_MAGIC};
use crate::bwt::OpenCells;
use crate::par_scan::RawMagic;
use crate::parallel::Shared;
use crate::{decode_block_rows, Error, ErrorKind, Kernels, Scratch};
use std::any::Any;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

static LAST_COORD_WORK_NS: AtomicU64 = AtomicU64::new(0);

pub fn last_coord_work_ns() -> u64 {
    LAST_COORD_WORK_NS.load(Ordering::Relaxed)
}

const MASK48: u64 = (1u64 << 48) - 1;
const TILE_ANCHORS: u64 = 65_536; // 256 KiB per parallel anchor-scan tile
const FIRST_SNAP: usize = 256 << 10; // 256 KiB initial snapshot threshold
const GROW_STEP: usize = 2 << 20; // 2 MiB incremental snapshot threshold
const COMPACT_BYTES: u64 = 4 << 20; // 4 MiB compaction step (kept 4-byte aligned)
const MAX_ITEM: u64 = 64 << 20; // 64 MiB compressed block cap (stream.rs)

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

#[cold]
#[inline(never)]
fn verify_quad_rel(
    snap: &[u8],
    snap_base: u64,
    total_bits: u64,
    k_rel_base: usize,
    out: &mut Vec<RawMagic>,
) {
    for k_rel in k_rel_base..k_rel_base + 4 {
        let k_abs = snap_base + (k_rel as u64);
        if k_abs * 8 + 48 > total_bits || k_rel >= snap.len() {
            return;
        }
        let w64 = if let Some(s8) = snap.get(k_rel..k_rel + 8) {
            u64::from_be_bytes([s8[0], s8[1], s8[2], s8[3], s8[4], s8[5], s8[6], s8[7]])
        } else {
            let mut arr = [0u8; 8];
            let avail = snap.len() - k_rel;
            arr[..avail].copy_from_slice(&snap[k_rel..]);
            u64::from_be_bytes(arr)
        };
        for s in 0..8u32 {
            let bit = k_abs * 8 + (s as u64);
            if bit + 48 > total_bits {
                break;
            }
            let v = (w64 >> (16 - s)) & MASK48;
            if v == BLOCK_MAGIC || v == END_MAGIC {
                let hdr_level = if s == 0 && k_rel >= 4 {
                    match snap[k_rel - 4..k_rel] {
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

/// Scans absolute 4-byte anchors `m in m_start_abs..m_end_abs` inside `snap` (`snap_base % 4 == 0`).
fn scan_anchor_tile(
    snap: &[u8],
    snap_base: u64,
    total_bits: u64,
    m_start_abs: u64,
    m_end_abs: u64,
    out: &mut Vec<RawMagic>,
) {
    out.clear();
    if m_start_abs >= m_end_abs || snap.len() < 6 {
        return;
    }
    debug_assert_eq!(snap_base & 3, 0);
    let m_base = snap_base / 4;
    if m_end_abs <= m_base {
        return;
    }
    let r_start = m_start_abs.max(m_base + 1) - m_base;
    let r_end = m_end_abs - m_base;
    let mut p = (r_start as usize) * 4;
    let p_end = (r_end as usize) * 4;
    let len = snap.len();
    let bm = &ANCHOR_BITMAP;

    while p + 12 < p_end && p + 14 <= len {
        let s = &snap[p..p + 14];
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
                verify_quad_rel(snap, snap_base, total_bits, p - 4, out);
            }
            if h1 != 0 {
                verify_quad_rel(snap, snap_base, total_bits, p, out);
            }
            if h2 != 0 {
                verify_quad_rel(snap, snap_base, total_bits, p + 4, out);
            }
            if h3 != 0 {
                verify_quad_rel(snap, snap_base, total_bits, p + 8, out);
            }
        }
        p += 16;
    }

    while p < p_end && p + 2 <= len {
        let idx = ((snap[p] as usize) << 8) | (snap[p + 1] as usize);
        if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 {
            verify_quad_rel(snap, snap_base, total_bits, p - 4, out);
        }
        p += 4;
    }
}

#[derive(Clone, Copy, Debug)]
struct StreamCand {
    bit: u64,
    hint_level: u32,
}

enum StreamWorkerDone {
    Block {
        end_bit: u64,
        crc: u32,
        orig_ptr: u32,
        n_rows: usize,
        out: Vec<u8>,
    },
    Truncated,
    Failed(ErrorKind),
}

struct ParScanHub {
    snap_base: u64,
    snap_end: u64,
    snap: Arc<[u8]>,
    snap_eof: bool,

    // Parallel tile-ordered finder state
    avail_tiles: usize,
    m_avail_abs: u64,
    eof_tiles_set: bool,
    next_flush_tile: usize,
    done_tiles: BTreeMap<usize, Vec<RawMagic>>,
    cur_hint_lvl: u32,
    first_hdr_checked: bool,
    last_flushed_bit: Option<u64>,
    flushed_upto_bit: u64,
    cands_complete: bool,

    // Candidate & chain state
    cands: Vec<StreamCand>,
    base_idx: usize,
    done: BTreeMap<u64, StreamWorkerDone>,
    chain_bit: u64,
    released_byte: u64,
    need_more_bytes: bool,
    workers_left: usize,
    stop: bool,
}

impl ParScanHub {
    fn new(workers: usize) -> Self {
        Self {
            snap_base: 0,
            snap_end: 0,
            snap: Arc::from(&[][..]),
            snap_eof: false,
            avail_tiles: 0,
            m_avail_abs: 1,
            eof_tiles_set: false,
            next_flush_tile: 0,
            done_tiles: BTreeMap::new(),
            cur_hint_lvl: 0,
            first_hdr_checked: false,
            last_flushed_bit: None,
            flushed_upto_bit: 0,
            cands_complete: false,
            cands: Vec::new(),
            base_idx: 0,
            done: BTreeMap::new(),
            chain_bit: 0,
            released_byte: 0,
            need_more_bytes: false,
            workers_left: workers,
            stop: false,
        }
    }

    fn flush_ready_tiles(&mut self) {
        if !self.first_hdr_checked && self.snap_base == 0 && self.snap.len() >= 4 {
            self.first_hdr_checked = true;
            if let [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] = *self.snap {
                self.cur_hint_lvl = (l - b'0') as u32;
            }
        }
        while let Some(raws) = self.done_tiles.remove(&self.next_flush_tile) {
            for rm in raws {
                if self.last_flushed_bit == Some(rm.bit) {
                    continue;
                }
                self.last_flushed_bit = Some(rm.bit);
                if !rm.is_end {
                    if rm.hdr_level != 0 {
                        self.cur_hint_lvl = rm.hdr_level as u32;
                    }
                    let hint_level = if self.cur_hint_lvl == 0 {
                        9
                    } else {
                        self.cur_hint_lvl
                    };
                    self.cands.push(StreamCand {
                        bit: rm.bit,
                        hint_level,
                    });
                }
            }
            self.next_flush_tile += 1;
        }
        if self.eof_tiles_set && self.next_flush_tile >= self.avail_tiles {
            self.cands_complete = true;
            self.flushed_upto_bit = u64::MAX;
        } else {
            self.flushed_upto_bit = 32 * (self.next_flush_tile as u64) * TILE_ANCHORS;
        }
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn panic_msg(p: Box<dyn Any + Send>) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "a parallel front-end thread panicked".to_string()
    }
}

/// Try to claim and scan one available `256 KiB` anchor tile (`true` if a tile was scanned).
fn try_help_scan_tile(
    shared: &Shared,
    hub: &Mutex<ParScanHub>,
    tile_cursor: &AtomicUsize,
    raw_buf: &mut Vec<RawMagic>,
) -> bool {
    let (tile, snap, snap_base, total_bits, m_lo, m_hi) = {
        let ps = lock(hub);
        let cur = tile_cursor.load(Ordering::Relaxed);
        if cur >= ps.avail_tiles || ps.stop {
            return false;
        }
        let t = tile_cursor.fetch_add(1, Ordering::Relaxed);
        if t >= ps.avail_tiles {
            return false;
        }
        let m_lo = 1 + (t as u64) * TILE_ANCHORS;
        let m_hi = (1 + ((t + 1) as u64) * TILE_ANCHORS).min(ps.m_avail_abs);
        let total_bits = if ps.snap_eof {
            ps.snap_end * 8
        } else {
            u64::MAX
        };
        (t, Arc::clone(&ps.snap), ps.snap_base, total_bits, m_lo, m_hi)
    };

    scan_anchor_tile(&snap, snap_base, total_bits, m_lo, m_hi, raw_buf);

    let mut ps = lock(hub);
    ps.done_tiles.insert(tile, std::mem::take(raw_buf));
    ps.flush_ready_tiles();
    drop(ps);
    shared.cv.notify_all();
    true
}

#[allow(clippy::too_many_arguments)]
fn decode_cand_on_snap(
    snap: &[u8],
    snap_base: u64,
    bit: u64,
    bound_level: u32,
    hint_level: u32,
    k: Kernels,
    scan_scratch: &mut ScanScratch,
    bwt_scratch: &mut Scratch<OpenCells>,
    syms: &mut Vec<u16>,
    mut out: Vec<u8>,
) -> (StreamWorkerDone, Option<Vec<u8>>) {
    let cand_byte = bit / 8;
    if cand_byte < snap_base {
        return (StreamWorkerDone::Failed(ErrorKind::BadBlockMagic), Some(out));
    }
    let rel = (cand_byte - snap_base) as usize;
    let sub = snap.get(rel..).unwrap_or(&[]);
    let mut br = Bits::at(sub, bit % 8);
    let item = read_item_with_scratch(&mut br, bound_level, std::mem::take(syms), scan_scratch);
    let overran = br.overran();
    let end_bit = cand_byte * 8 + br.bit_pos();
    if overran {
        *syms = match item {
            Ok(Item::Block(b)) => b.syms,
            Ok(Item::End(_)) => Vec::new(),
            Err((_, s)) => s,
        };
        return (StreamWorkerDone::Truncated, Some(out));
    }
    match item {
        Ok(Item::Block(b)) => {
            let orig_ptr = b.orig_ptr;
            out.clear();
            let hint_max = 100_000 * (hint_level as usize);
            let res = decode_block_rows(&b, hint_max, k, bwt_scratch, &mut out);
            *syms = b.syms;
            match res {
                Ok((crc, n_rows)) => (
                    StreamWorkerDone::Block {
                        end_bit,
                        crc,
                        orig_ptr,
                        n_rows,
                        out,
                    },
                    None,
                ),
                Err(kind) => (StreamWorkerDone::Failed(kind), Some(out)),
            }
        }
        Ok(Item::End(_)) => (StreamWorkerDone::Failed(ErrorKind::BadBlockMagic), Some(out)),
        Err((kind, s)) => {
            *syms = s;
            (StreamWorkerDone::Failed(kind), Some(out))
        }
    }
}

pub(crate) struct ParScanHandles {
    pub handles: Vec<std::thread::JoinHandle<()>>,
    pub coord_work_ns: Arc<AtomicU64>,
}

pub(crate) fn spawn_parscan_team(
    shared: Arc<Shared>,
    in_rx: Receiver<Vec<u8>>,
    workers: usize,
    slots: usize,
    k: Kernels,
) -> ParScanHandles {
    let workers = workers.max(1);
    let slots = slots.max(workers);
    let hub = Arc::new(Mutex::new(ParScanHub::new(workers)));
    let tile_cursor = Arc::new(AtomicUsize::new(0));
    let cand_cursor = Arc::new(AtomicUsize::new(0));
    let chain_bit_atomic = Arc::new(AtomicU64::new(0));
    let coord_work_ns = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::with_capacity(workers + 1);

    for i in 0..workers {
        let sh = Arc::clone(&shared);
        let hb = Arc::clone(&hub);
        let tc = Arc::clone(&tile_cursor);
        let cc = Arc::clone(&cand_cursor);
        let cb = Arc::clone(&chain_bit_atomic);
        handles.push(
            std::thread::Builder::new()
                .name(format!("mibz2-ps-{i}"))
                .spawn(move || {
                    let ran = catch_unwind(AssertUnwindSafe(|| {
                        parscan_worker(&sh, &hb, &tc, &cc, &cb, slots, k);
                    }));
                    if let Err(p) = ran {
                        let msg = panic_msg(p);
                        {
                            let mut st = sh.lock();
                            if st.terminal.is_none() {
                                st.terminal = Some(Error::new(ErrorKind::Internal(msg)));
                            }
                            st.cancelled = true;
                        }
                        {
                            let mut ps = lock(&hb);
                            ps.stop = true;
                            ps.workers_left = ps.workers_left.saturating_sub(1);
                        }
                        sh.cv.notify_all();
                    }
                })
                .expect("spawn a bz2 parscan worker"),
        );
    }

    {
        let sh = Arc::clone(&shared);
        let hb = Arc::clone(&hub);
        let tc = Arc::clone(&tile_cursor);
        let cb = Arc::clone(&chain_bit_atomic);
        let cw = Arc::clone(&coord_work_ns);
        handles.push(
            std::thread::Builder::new()
                .name("mibz2-ps-coord".into())
                .spawn(move || {
                    let ran = catch_unwind(AssertUnwindSafe(|| {
                        parscan_coord(&sh, &hb, &tc, &cb, &cw, in_rx, k);
                    }));
                    if let Err(p) = ran {
                        let msg = panic_msg(p);
                        {
                            let mut st = sh.lock();
                            if st.terminal.is_none() {
                                st.terminal = Some(Error::new(ErrorKind::Internal(msg)));
                            }
                            st.cancelled = true;
                            if st.end.is_none() {
                                st.end = Some(st.issued);
                            }
                        }
                        lock(&hb).stop = true;
                        sh.cv.notify_all();
                    }
                })
                .expect("spawn the bz2 parscan coordinator"),
        );
    }

    ParScanHandles {
        handles,
        coord_work_ns,
    }
}

fn parscan_worker(
    shared: &Shared,
    hub: &Mutex<ParScanHub>,
    tile_cursor: &AtomicUsize,
    cand_cursor: &AtomicUsize,
    chain_bit_atomic: &AtomicU64,
    slots: usize,
    k: Kernels,
) {
    let mut raw_buf = Vec::with_capacity(32);
    let mut scan_scratch: Option<ScanScratch> = None;
    let mut bwt_scratch: Option<Scratch<OpenCells>> = None;
    let mut syms = Vec::new();
    let mut local_out: Option<Vec<u8>> = None;

    loop {
        // Help scan any newly arrived 256 KiB anchor tiles first
        while try_help_scan_tile(shared, hub, tile_cursor, &mut raw_buf) {}

        let idx = cand_cursor.fetch_add(1, Ordering::Relaxed);

        // Wait until `idx < cands.len()` (or all candidates finished / cancelled) AND `idx < base_idx + slots`
        let cand_opt = loop {
            if try_help_scan_tile(shared, hub, tile_cursor, &mut raw_buf) {
                continue;
            }
            let cancelled = shared.lock().cancelled;
            let mut ps = lock(hub);
            if cancelled || ps.stop {
                ps.workers_left -= 1;
                drop(ps);
                if let Some(b) = local_out.take() {
                    shared.lock().free_out.push(b);
                }
                shared.cv.notify_all();
                return;
            }
            if idx < ps.cands.len() {
                if idx < ps.base_idx || ps.cands[idx].bit < ps.chain_bit {
                    break None;
                }
                if idx < ps.base_idx + slots {
                    break Some((
                        ps.cands[idx],
                        ps.snap_base,
                        ps.snap_end,
                        Arc::clone(&ps.snap),
                        ps.snap_eof,
                    ));
                }
            } else if ps.cands_complete {
                ps.workers_left -= 1;
                drop(ps);
                if let Some(b) = local_out.take() {
                    shared.lock().free_out.push(b);
                }
                shared.cv.notify_all();
                return;
            } else {
                ps.need_more_bytes = true;
                shared.cv.notify_all();
            }
            // Park on `shared.cv` (must drop `ps` first and wait on `shared.state`!)
            drop(ps);
            let mut st = shared.lock();
            if st.cancelled {
                continue;
            }
            let ps_check = lock(hub);
            let ready_now = ps_check.stop
                || (idx < ps_check.cands.len()
                    && (idx < ps_check.base_idx || idx < ps_check.base_idx + slots))
                || (idx >= ps_check.cands.len() && ps_check.cands_complete)
                || tile_cursor.load(Ordering::Relaxed) < ps_check.avail_tiles;
            drop(ps_check);
            if !ready_now {
                st = shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
            }
            drop(st);
        };

        let Some((cand, mut snap_base, mut snap_end, mut snap, mut snap_eof)) = cand_opt else {
            continue;
        };

        if cand.bit < chain_bit_atomic.load(Ordering::Relaxed) {
            continue;
        }

        let mut out_buf = local_out
            .take()
            .or_else(|| shared.lock().free_out.pop())
            .unwrap_or_default();

        // Retry loop for wait-on-overrun before EOF (`2026-11-10-parallel-scan.md` §6.1)
        let final_res = loop {
            let sc_s = scan_scratch.get_or_insert_with(ScanScratch::default);
            let bw_s = bwt_scratch.get_or_insert_with(Scratch::default);
            let (res, ret_buf) = decode_cand_on_snap(
                &snap,
                snap_base,
                cand.bit,
                9,
                cand.hint_level,
                k,
                sc_s,
                bw_s,
                &mut syms,
                out_buf,
            );
            if let StreamWorkerDone::Truncated = res {
                out_buf = ret_buf.unwrap_or_default();
                let cand_byte = cand.bit / 8;
                if snap_eof {
                    local_out = Some(out_buf);
                    break Some(StreamWorkerDone::Truncated);
                }
                if snap_end.saturating_sub(cand_byte) >= MAX_ITEM {
                    local_out = Some(out_buf);
                    break Some(StreamWorkerDone::Failed(ErrorKind::BlockTooLarge));
                }
                // Wait for a larger snapshot (`snap_end > prev_end` or `snap_eof`)
                let prev_end = snap_end;
                let waited = loop {
                    if try_help_scan_tile(shared, hub, tile_cursor, &mut raw_buf) {
                        continue;
                    }
                    let cancelled = shared.lock().cancelled;
                    let mut ps = lock(hub);
                    if cancelled || ps.stop || idx < ps.base_idx || cand.bit < ps.chain_bit {
                        break None;
                    }
                    if ps.snap_end > prev_end || ps.snap_eof {
                        break Some((
                            ps.snap_base,
                            ps.snap_end,
                            Arc::clone(&ps.snap),
                            ps.snap_eof,
                        ));
                    }
                    ps.need_more_bytes = true;
                    drop(ps);
                    shared.cv.notify_all();
                    let mut st = shared.lock();
                    if st.cancelled {
                        continue;
                    }
                    let ps_chk = lock(hub);
                    let wake = ps_chk.stop
                        || idx < ps_chk.base_idx
                        || cand.bit < ps_chk.chain_bit
                        || ps_chk.snap_end > prev_end
                        || ps_chk.snap_eof;
                    drop(ps_chk);
                    if !wake {
                        st = shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
                    }
                    drop(st);
                };
                match waited {
                    Some((nb, ne, ns, neof)) => {
                        snap_base = nb;
                        snap_end = ne;
                        snap = ns;
                        snap_eof = neof;
                        continue;
                    }
                    None => {
                        local_out = Some(out_buf);
                        break None;
                    }
                }
            } else {
                local_out = ret_buf;
                break Some(res);
            }
        };

        let Some(res) = final_res else {
            continue;
        };

        let mut ps = lock(hub);
        if idx < ps.base_idx || cand.bit < ps.chain_bit || ps.stop {
            drop(ps);
            if let StreamWorkerDone::Block { out: b, .. } = res {
                shared.lock().free_out.push(b);
            }
        } else {
            ps.done.insert(cand.bit, res);
            drop(ps);
            shared.cv.notify_all();
        }
    }
}

fn parscan_coord(
    shared: &Shared,
    hub: &Mutex<ParScanHub>,
    tile_cursor: &AtomicUsize,
    chain_bit_atomic: &AtomicU64,
    coord_work_ns: &AtomicU64,
    rx: Receiver<Vec<u8>>,
    k: Kernels,
) {
    let t_coord0 = Instant::now();
    let parked_ns = std::cell::Cell::new(0u64);
    let mut buf: Vec<u8> = Vec::new();
    let mut buf_base: u64 = 0;
    let mut rx_eof = false;
    let mut raw_buf = Vec::with_capacity(32);

    let pull_input = |buf: &mut Vec<u8>,
                      buf_base: &mut u64,
                      rx_eof: &mut bool,
                      min_new_bytes: usize,
                      blocking: bool|
     -> bool {
        if *rx_eof {
            return false;
        }
        {
            let ps = lock(hub);
            let tile_safe_byte = 4 * (1 + (ps.next_flush_tile as u64) * TILE_ANCHORS);
            let safe_upto = ps.released_byte.min(tile_safe_byte).saturating_sub(16) & !3;
            if safe_upto >= *buf_base + COMPACT_BYTES {
                let drop_n = ((safe_upto - *buf_base) as usize).min(buf.len() & !3);
                if drop_n > 0 {
                    buf.drain(..drop_n);
                    *buf_base += drop_n as u64;
                }
            }
        }

        let start_len = buf.len();
        let mut got_any = false;
        while !*rx_eof {
            if shared.lock().cancelled {
                return false;
            }
            let chunk_res = if !got_any && blocking {
                let tw = Instant::now();
                let r = rx.recv().map_err(|_| ());
                parked_ns.set(parked_ns.get() + tw.elapsed().as_nanos() as u64);
                r
            } else if buf.len() - start_len < min_new_bytes && blocking {
                let tw = Instant::now();
                let r = rx.recv().map_err(|_| ());
                parked_ns.set(parked_ns.get() + tw.elapsed().as_nanos() as u64);
                r
            } else {
                match rx.try_recv() {
                    Ok(c) => Ok(c),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(()),
                }
            };
            match chunk_res {
                Ok(c) => {
                    if !c.is_empty() {
                        buf.extend_from_slice(&c);
                        got_any = true;
                        if buf.len() - start_len >= GROW_STEP {
                            break;
                        }
                    }
                }
                Err(()) => {
                    *rx_eof = true;
                    break;
                }
            }
        }

        if !got_any && !*rx_eof {
            return false;
        }

        let snap: Arc<[u8]> = Arc::from(&buf[..]);
        let snap_end = *buf_base + (buf.len() as u64);
        let mut ps = lock(hub);
        ps.snap_base = *buf_base;
        ps.snap_end = snap_end;
        ps.snap = snap;
        ps.snap_eof = *rx_eof;
        ps.need_more_bytes = false;

        if *rx_eof {
            let max_anchor = if snap_end >= 6 { (snap_end - 2) / 4 } else { 0 };
            ps.m_avail_abs = if max_anchor >= 1 { max_anchor + 1 } else { 1 };
            ps.avail_tiles = (ps.m_avail_abs - 1).div_ceil(TILE_ANCHORS) as usize;
            ps.eof_tiles_set = true;
        } else {
            let max_anchor = if snap_end >= 11 {
                (snap_end - 7) / 4
            } else {
                0
            };
            let m_avail = if max_anchor >= 1 { max_anchor + 1 } else { 1 };
            let full_tiles = ((m_avail - 1) / TILE_ANCHORS) as usize;
            ps.avail_tiles = full_tiles;
            ps.m_avail_abs = 1 + (full_tiles as u64) * TILE_ANCHORS;
        }
        ps.flush_ready_tiles();
        drop(ps);
        shared.cv.notify_all();
        true
    };

    let finish = |err: Option<Error>| {
        let active = (t_coord0.elapsed().as_nanos() as u64).saturating_sub(parked_ns.get());
        coord_work_ns.store(active, Ordering::Relaxed);
        LAST_COORD_WORK_NS.store(active, Ordering::Relaxed);
        {
            let mut st = shared.lock();
            let at = st.issued;
            match err {
                Some(e) if e.kind != ErrorKind::Cancelled => {
                    st.ready.insert(at, Err(e));
                    st.issued += 1;
                    st.end = Some(at + 1);
                }
                _ => st.end = Some(at),
            }
        }
        lock(hub).stop = true;
        shared.cv.notify_all();
    };

    // Initial input pull (`FIRST_SNAP` or EOF)
    pull_input(&mut buf, &mut buf_base, &mut rx_eof, FIRST_SNAP, true);

    let mut bit = 0u64;
    let mut first = true;
    let mut in_stream = false;
    let mut level = 0u32;
    let mut combined = 0u32;
    let mut blocks = 0u64;

    let mut fallback_scan: Option<ScanScratch> = None;
    let mut fallback_bwt: Option<Scratch<OpenCells>> = None;
    let mut fallback_syms = Vec::new();

    // Ensure at least `want_bytes` from `at_byte` are present in `buf` (or `rx_eof`).
    macro_rules! ensure_bytes {
        ($at_byte:expr, $want:expr) => {{
            while !rx_eof && (buf_base + buf.len() as u64) < ($at_byte) + ($want as u64) {
                if shared.lock().cancelled {
                    return finish(None);
                }
                let need = (($at_byte + ($want as u64)) - (buf_base + buf.len() as u64))
                    .max(FIRST_SNAP as u64) as usize;
                pull_input(&mut buf, &mut buf_base, &mut rx_eof, need, true);
            }
            if shared.lock().cancelled {
                return finish(None);
            }
        }};
    }

    loop {
        if shared.lock().cancelled {
            return finish(None);
        }
        // Opportunistically drain queued chunks to keep workers fed ahead of the chain
        if !rx_eof {
            let chain_byte = bit / 8;
            let ahead = (buf_base + buf.len() as u64).saturating_sub(chain_byte);
            if ahead < (GROW_STEP as u64) {
                pull_input(&mut buf, &mut buf_base, &mut rx_eof, 0, false);
            }
        }
        while try_help_scan_tile(shared, hub, tile_cursor, &mut raw_buf) {}

        if !in_stream {
            let at = bit;
            let at_byte = at / 8;
            ensure_bytes!(at_byte, 4);
            let rel = (at_byte - buf_base) as usize;
            let rem = buf.get(rel..).unwrap_or(&[]);
            if rem.is_empty() && rx_eof {
                if first {
                    return finish(Some(Error {
                        kind: ErrorKind::Empty,
                        block: None,
                        bit: Some(at),
                    }));
                }
                return finish(None);
            }
            match rem {
                [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                    in_stream = true;
                    level = (l - b'0') as u32;
                    combined = 0;
                    bit += 32;
                    chain_bit_atomic.store(bit, Ordering::Relaxed);
                }
                _ => {
                    return finish(Some(Error {
                        kind: if first {
                            ErrorKind::NotBz2
                        } else {
                            ErrorKind::TrailingData
                        },
                        block: None,
                        bit: Some(at),
                    }));
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
            let at_byte = at / 8;
            ensure_bytes!(at_byte, 12);
            let rel = (at_byte - buf_base) as usize;
            let sub = buf.get(rel..).unwrap_or(&[]);
            let mut r_peek = Bits::at(sub, at % 8);
            let magic = (r_peek.bits(24) as u64) << 24 | r_peek.bits(24) as u64;
            if magic == END_MAGIC {
                let stored = r_peek.bits(32);
                if r_peek.overran() {
                    return finish(Some(err_at(ErrorKind::Truncated)));
                }
                if stored != combined {
                    return finish(Some(err_at(ErrorKind::StreamCrc {
                        stored,
                        computed: combined,
                    })));
                }
                bit = (at_byte * 8 + r_peek.bit_pos()).div_ceil(8) * 8;
                chain_bit_atomic.store(bit, Ordering::Relaxed);
                lock(hub).released_byte = bit / 8;
                in_stream = false;
                first = false;
                continue;
            }
            if magic != BLOCK_MAGIC {
                if r_peek.overran() {
                    return finish(Some(err_at(ErrorKind::Truncated)));
                }
                return finish(Some(err_at(ErrorKind::BadBlockMagic)));
            }

            // Wait for slot in `st.ready` (`issued - taken < slots`) before retiring block `idx`
            {
                let mut st = shared.lock();
                while !st.cancelled && st.issued - st.taken >= shared.slots {
                    let tw = Instant::now();
                    st = shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
                    parked_ns.set(parked_ns.get() + tw.elapsed().as_nanos() as u64);
                }
                if st.cancelled {
                    return finish(None);
                }
            }

            // Wait for worker result at `at` (while helping scan tiles & pulling input if needed!)
            let taken_worker: Option<StreamWorkerDone> = loop {
                if try_help_scan_tile(shared, hub, tile_cursor, &mut raw_buf) {
                    continue;
                }
                if shared.lock().cancelled {
                    return finish(None);
                }
                let need_pull = {
                    let mut ps = lock(hub);
                    ps.chain_bit = at;
                    ps.released_byte = at / 8;
                    chain_bit_atomic.store(at, Ordering::Relaxed);

                    let before = ps.base_idx;
                    let mut freed = Vec::new();
                    while ps.base_idx < ps.cands.len() && ps.cands[ps.base_idx].bit < at {
                        let k_bit = ps.cands[ps.base_idx].bit;
                        if let Some(StreamWorkerDone::Block { out: b, .. }) = ps.done.remove(&k_bit)
                        {
                            freed.push(b);
                        }
                        ps.base_idx += 1;
                    }
                    let base_moved = ps.base_idx != before;
                    if ps.flushed_upto_bit > at {
                        if ps.base_idx >= ps.cands.len() || ps.cands[ps.base_idx].bit != at {
                            drop(ps);
                            if !freed.is_empty() {
                                shared.lock().free_out.extend(freed);
                            }
                            if base_moved {
                                shared.cv.notify_all();
                            }
                            break None;
                        }
                        if let Some(w_res) = ps.done.remove(&at) {
                            ps.base_idx += 1;
                            drop(ps);
                            if !freed.is_empty() {
                                shared.lock().free_out.extend(freed);
                            }
                            shared.cv.notify_all();
                            break Some(w_res);
                        }
                    }
                    let wp = ps.workers_left == 0 && ps.cands_complete;
                    let want_more = ps.need_more_bytes || ps.flushed_upto_bit <= at;
                    drop(ps);
                    if !freed.is_empty() {
                        shared.lock().free_out.extend(freed);
                    }
                    if base_moved {
                        shared.cv.notify_all();
                    }
                    if wp {
                        break None;
                    }
                    want_more && !rx_eof
                };

                if need_pull {
                    if pull_input(&mut buf, &mut buf_base, &mut rx_eof, FIRST_SNAP, true) {
                        continue;
                    }
                } else if !rx_eof {
                    if pull_input(&mut buf, &mut buf_base, &mut rx_eof, 0, false) {
                        continue;
                    }
                }

                let mut st = shared.lock();
                if st.cancelled {
                    return finish(None);
                }
                let ps_chk = lock(hub);
                let wake_now = ps_chk.done.contains_key(&at)
                    || (ps_chk.flushed_upto_bit > at
                        && (ps_chk.base_idx >= ps_chk.cands.len()
                            || ps_chk.cands[ps_chk.base_idx].bit != at))
                    || (ps_chk.need_more_bytes && !rx_eof)
                    || tile_cursor.load(Ordering::Relaxed) < ps_chk.avail_tiles
                    || (ps_chk.workers_left == 0 && ps_chk.cands_complete);
                drop(ps_chk);
                if !wake_now {
                    let tw = Instant::now();
                    st = shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
                    parked_ns.set(parked_ns.get() + tw.elapsed().as_nanos() as u64);
                }
                drop(st);
            };

            let actual_max = 100_000 * (level as usize);
            let resolved = match taken_worker {
                Some(StreamWorkerDone::Block {
                    end_bit,
                    crc,
                    orig_ptr,
                    n_rows,
                    out,
                }) => {
                    if (orig_ptr as usize) > 10 + actual_max {
                        shared.lock().free_out.push(out);
                        return finish(Some(err_at(ErrorKind::BadOrigPtr)));
                    }
                    if n_rows > actual_max {
                        shared.lock().free_out.push(out);
                        return finish(Some(err_at(ErrorKind::BlockTooLong)));
                    }
                    Some((end_bit, crc, out))
                }
                Some(StreamWorkerDone::Truncated) => {
                    return finish(Some(err_at(ErrorKind::Truncated)));
                }
                Some(StreamWorkerDone::Failed(kind)) if level == 9 => {
                    return finish(Some(err_at(kind)));
                }
                _ => None,
            };

            let (end_bit, crc, out) = match resolved {
                Some(v) => v,
                None => {
                    // Inline read on `buf` (growing lookahead up to `MAX_ITEM` on overrun before EOF)
                    let mut lookahead = GROW_STEP;
                    let out_buf = shared.lock().free_out.pop().unwrap_or_default();
                    let mut cur_out = out_buf;
                    loop {
                        ensure_bytes!(at_byte, lookahead);
                        let sc = fallback_scan.get_or_insert_with(ScanScratch::default);
                        let bw = fallback_bwt.get_or_insert_with(Scratch::default);
                        let (w_res, ret_buf) = decode_cand_on_snap(
                            &buf,
                            buf_base,
                            at,
                            level,
                            level,
                            k,
                            sc,
                            bw,
                            &mut fallback_syms,
                            cur_out,
                        );
                        match w_res {
                            StreamWorkerDone::Block {
                                end_bit, crc, out, ..
                            } => break (end_bit, crc, out),
                            StreamWorkerDone::Truncated => {
                                cur_out = ret_buf.unwrap_or_default();
                                if rx_eof {
                                    shared.lock().free_out.push(cur_out);
                                    return finish(Some(err_at(ErrorKind::Truncated)));
                                }
                                if (lookahead as u64) >= MAX_ITEM {
                                    shared.lock().free_out.push(cur_out);
                                    return finish(Some(err_at(ErrorKind::BlockTooLarge)));
                                }
                                lookahead = (lookahead * 2).min(MAX_ITEM as usize);
                            }
                            StreamWorkerDone::Failed(kind) => {
                                if let Some(b) = ret_buf {
                                    shared.lock().free_out.push(b);
                                }
                                return finish(Some(err_at(kind)));
                            }
                        }
                    }
                }
            };

            {
                let mut st = shared.lock();
                let place = st.issued;
                debug_assert_eq!(place, idx);
                st.ready.insert(place, Ok(out));
                st.issued += 1;
            }
            combined = combined.rotate_left(1) ^ crc;
            blocks += 1;
            bit = end_bit;
            chain_bit_atomic.store(bit, Ordering::Relaxed);
            lock(hub).released_byte = bit / 8;
            shared.cv.notify_all();
        }
    }
    // `rx` drops on return, immediately waking any blocked `StreamDecoder::write` with `Err(Stopped)`!
}
