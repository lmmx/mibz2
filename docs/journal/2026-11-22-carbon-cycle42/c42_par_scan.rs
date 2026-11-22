#![forbid(unsafe_code)]
//! Push-streaming parallel front end (`StreamDecoder::parscan`, `BzReader::parscan`,
//! `decode_parscan`):
//!
//! 1. **Zero-copy `SlabRope`**: input arrives in bounded `Vec<u8>` chunks, wrapped in
//!    `Arc<Vec<u8>>` without copying (coalescing tiny `< 64 KiB` writes in place via
//!    `Arc::get_mut`), and recycled back to `StreamDecoder::spare` via `Arc::try_unwrap`
//!    as soon as the verified bit-chain passes each slab's end.
//! 2. **Fast `S = 1` `SlabScanner` (`workers <= 3`)**: at 1-3 workers, a single `C37`
//!    scanner has `>= 1.6x-4x` throughput headroom even on `-1` streams, so `mibz2-scan`
//!    streams directly over `SlabRope` + `read_item_tight` with zero magic-scan or
//!    coordinator-lock overhead.
//! 3. **Demand-driven `S`-Scanner Pool (`workers >= 4`: `mibz2-scan` + `1..2` `54 KiB`
//!    L1d-resident helper scanners)**: scans global 4-byte anchors (`ANCHOR_BITMAP`),
//!    decodes `Symbols` in parallel under a tight `workers + 4` front-end cap (with
//!    Zero-Level-Guess `C39a` across multi-stream boundaries), and feeds `#739`'s `Jobs` ->
//!    `decode_owned` -> `Ready` ring + `12 MiB` `READY_BYTES` cap, keeping BWT workers
//!    100% dedicated to BWT inversion with zero worker-side scan scratch.

use crate::bits::Bits;
use crate::block::{
    self, read_item_tight, Item, ScanScratch, Symbols, BLOCK_MAGIC, END_MAGIC,
};
use crate::parallel::{Jobs, JobsTx, Shared, SLOTS_PER_THREAD};
use crate::{Error, ErrorKind, Kernels, StreamDecoder};
use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

const MASK48: u64 = (1u64 << 48) - 1;
const INIT_NEED_MORE: usize = 256 << 10;
const MAX_STITCH_RETAIN: usize = 256 << 10;
const MAX_ITEM: usize = 64 << 20;
const COALESCE_MIN: usize = 64 << 10;
const COALESCE_TARGET: usize = 256 << 10;
/// Proactively stage the next `4 MiB` slab once live bytes in `SlabRope` drop to `<= 4.5 MiB`,
/// capping normal rope residency at `1-2` slabs (`4-8 MiB`) with zero chunk-seam bubbles.
const PRESTAGE_ROPE_BYTES: u64 = (4 << 20) + (512 << 10);
const MAX_ROPE_LIVE: u64 = 8 << 20;

/// Input chunks queued in `StreamDecoder::parscan`'s input channel.
pub(crate) const PARSCAN_INPUT_QUEUE: usize = 2;

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

/// A candidate 48-bit `BLOCK_MAGIC` or `END_MAGIC` found in the compressed stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMagic {
    pub bit: u64,
    pub is_end: bool,
    pub hdr_level: u8,
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

/// Scans disjoint 4-byte anchors `p = 4 * m` for `m in m_start..m_end` (`1 <= m_start <= m_end`)
/// on a contiguous slice `data`.
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

#[cold]
#[inline(never)]
fn verify_quad_global(win: &[u8], global_k_base: u64, total_bits: u64, out: &mut Vec<(u64, bool)>) {
    for rel_k in 0..4usize {
        let k = global_k_base + rel_k as u64;
        if k * 8 + 48 > total_bits || rel_k >= win.len() {
            return;
        }
        let w64 = if let Some(s8) = win.get(rel_k..rel_k + 8) {
            u64::from_be_bytes([s8[0], s8[1], s8[2], s8[3], s8[4], s8[5], s8[6], s8[7]])
        } else {
            let mut arr = [0u8; 8];
            let avail = win.len() - rel_k;
            arr[..avail].copy_from_slice(&win[rel_k..]);
            u64::from_be_bytes(arr)
        };
        for s in 0..8u32 {
            let bit = k * 8 + (s as u64);
            if bit + 48 > total_bits {
                break;
            }
            let v = (w64 >> (16 - s)) & MASK48;
            if v == BLOCK_MAGIC || v == END_MAGIC {
                out.push((bit, v == END_MAGIC));
            }
        }
    }
}

fn scan_slab_interior(
    data: &[u8],
    slab_base: u64,
    m_start: u64,
    m_end: u64,
    total_bits: u64,
    out: &mut Vec<(u64, bool)>,
) {
    if m_start >= m_end {
        return;
    }
    let bm = &ANCHOR_BITMAP;
    let mut p = (4 * m_start - slab_base) as usize;
    let p_end = (4 * m_end - slab_base) as usize;
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
                let k0 = p - 4;
                verify_quad_global(
                    &data[k0..(k0 + 12).min(len)],
                    slab_base + k0 as u64,
                    total_bits,
                    out,
                );
            }
            if h1 != 0 {
                let k1 = p;
                verify_quad_global(
                    &data[k1..(k1 + 12).min(len)],
                    slab_base + k1 as u64,
                    total_bits,
                    out,
                );
            }
            if h2 != 0 {
                let k2 = p + 4;
                verify_quad_global(
                    &data[k2..(k2 + 12).min(len)],
                    slab_base + k2 as u64,
                    total_bits,
                    out,
                );
            }
            if h3 != 0 {
                let k3 = p + 8;
                verify_quad_global(
                    &data[k3..(k3 + 12).min(len)],
                    slab_base + k3 as u64,
                    total_bits,
                    out,
                );
            }
        }
        p += 16;
    }

    while p < p_end && p + 2 <= len {
        let idx = ((data[p] as usize) << 8) | (data[p + 1] as usize);
        if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 {
            let k0 = p - 4;
            verify_quad_global(
                &data[k0..(k0 + 12).min(len)],
                slab_base + k0 as u64,
                total_bits,
                out,
            );
        }
        p += 4;
    }
}

struct Slab {
    base: u64,
    data: Arc<Vec<u8>>,
}

impl Slab {
    #[inline]
    fn end(&self) -> u64 {
        self.base + self.data.len() as u64
    }
}

/// Zero-copy bounded rope of compressed input chunks.
pub(crate) struct SlabRope {
    slabs: VecDeque<Slab>,
    floor_byte: u64,
    end_byte: u64,
    next_anchor_m: u64,
    pub(crate) eof: bool,
}

enum RopeView {
    ZeroCopy {
        slab: Arc<Vec<u8>>,
        offset: usize,
        len: usize,
        to_eof: bool,
    },
    Stitched {
        len: usize,
        to_eof: bool,
    },
}

impl SlabRope {
    pub(crate) fn new() -> Self {
        Self {
            slabs: VecDeque::with_capacity(4),
            floor_byte: 0,
            end_byte: 0,
            next_anchor_m: 1,
            eof: false,
        }
    }

    #[inline]
    pub(crate) fn total_bytes(&self) -> u64 {
        self.end_byte
    }

    #[inline]
    pub(crate) fn live_bytes(&self) -> u64 {
        self.end_byte.saturating_sub(self.floor_byte)
    }

    fn push_slab_raw(&mut self, v: Vec<u8>) {
        if v.is_empty() {
            return;
        }
        let base = self.end_byte;
        self.end_byte += v.len() as u64;
        self.slabs.push_back(Slab {
            base,
            data: Arc::new(v),
        });
    }

    pub(crate) fn push_chunk(&mut self, mut chunk: Vec<u8>) {
        if chunk.is_empty() {
            return;
        }
        if chunk.len() < COALESCE_MIN {
            if let Some(back) = self.slabs.back_mut() {
                if back.data.len() + chunk.len() <= COALESCE_TARGET {
                    if let Some(vec) = Arc::get_mut(&mut back.data) {
                        vec.extend_from_slice(&chunk);
                        self.end_byte += chunk.len() as u64;
                        return;
                    }
                }
            }
            chunk.reserve(COALESCE_TARGET.saturating_sub(chunk.len()));
        }
        self.push_slab_raw(chunk);
    }

    pub(crate) fn set_eof(&mut self) {
        self.eof = true;
    }

    pub(crate) fn copy_at(&self, global_off: u64, dst: &mut [u8]) -> usize {
        if global_off >= self.end_byte || dst.is_empty() {
            return 0;
        }
        let mut idx = match self
            .slabs
            .binary_search_by(|s| {
                if s.end() <= global_off {
                    std::cmp::Ordering::Less
                } else if s.base > global_off {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }) {
            Ok(i) => i,
            Err(_) => return 0,
        };
        let mut cur = global_off;
        let mut written = 0usize;
        while written < dst.len() && idx < self.slabs.len() {
            let s = &self.slabs[idx];
            let off = (cur - s.base) as usize;
            let take = (dst.len() - written).min(s.data.len() - off);
            dst[written..written + take].copy_from_slice(&s.data[off..off + take]);
            written += take;
            cur += take as u64;
            idx += 1;
        }
        written
    }

    pub(crate) fn scan_new_anchors(&mut self, out: &mut Vec<(u64, bool)>) {
        let m_limit = if self.eof {
            if self.end_byte >= 6 {
                (self.end_byte - 2) / 4 + 1
            } else {
                1
            }
        } else if self.end_byte >= 8 {
            (self.end_byte - 8) / 4 + 1
        } else {
            1
        };
        if self.next_anchor_m >= m_limit {
            return;
        }
        let total_bits = self.end_byte * 8;
        let mut m = self.next_anchor_m;
        while m < m_limit {
            let k_base = 4 * m - 4;
            let Some(s_idx) = self
                .slabs
                .binary_search_by(|s| {
                    if s.end() <= k_base {
                        std::cmp::Ordering::Less
                    } else if s.base > k_base {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .ok()
            else {
                m += 1;
                continue;
            };
            let slab_base = self.slabs[s_idx].base;
            let slab_end = self.slabs[s_idx].end();
            let m_int_end = if slab_end >= 8 {
                ((slab_end - 8) / 4 + 1).min(m_limit)
            } else {
                m
            };
            if m < m_int_end {
                let slab_arc = Arc::clone(&self.slabs[s_idx].data);
                scan_slab_interior(&slab_arc, slab_base, m, m_int_end, total_bits, out);
                m = m_int_end;
                continue;
            }
            let mut win = [0u8; 12];
            let n = self.copy_at(k_base, &mut win);
            if n >= 6 {
                let idx = ((win[4] as usize) << 8) | (win[5] as usize);
                if ((ANCHOR_BITMAP[idx >> 6] >> (idx & 63)) & 1) != 0 {
                    verify_quad_global(&win[..n], k_base, total_bits, out);
                }
            }
            m += 1;
        }
        self.next_anchor_m = m_limit;
    }

    fn view_for_cand(
        &self,
        start_byte: u64,
        min_want: usize,
        next_cand_byte: Option<u64>,
        stitch: &mut Vec<u8>,
    ) -> RopeView {
        if start_byte >= self.end_byte {
            return RopeView::Stitched {
                len: 0,
                to_eof: self.eof,
            };
        }
        let s_idx = self
            .slabs
            .binary_search_by(|s| {
                if s.end() <= start_byte {
                    std::cmp::Ordering::Less
                } else if s.base > start_byte {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .expect("candidate start_byte >= floor_byte");
        let slab = &self.slabs[s_idx];
        let offset = (start_byte - slab.base) as usize;
        let in_slab = slab.data.len() - offset;
        let in_rope = (self.end_byte - start_byte) as usize;
        let last_slab = s_idx + 1 == self.slabs.len();
        let fits_zero_copy = (last_slab && min_want <= in_slab)
            || (min_want <= in_slab
                && next_cand_byte.is_some_and(|nb| nb.saturating_add(16) <= slab.end()));
        if fits_zero_copy {
            return RopeView::ZeroCopy {
                slab: Arc::clone(&slab.data),
                offset,
                len: in_slab,
                to_eof: last_slab && self.eof,
            };
        }
        let hint = next_cand_byte
            .map_or(INIT_NEED_MORE, |nb| {
                (nb.saturating_sub(start_byte) as usize).saturating_add(64)
            })
            .max(min_want)
            .min(MAX_ITEM);
        let want = hint.min(in_rope);
        if stitch.len() < want {
            stitch.resize(want, 0);
        }
        let got = self.copy_at(start_byte, &mut stitch[..want]);
        RopeView::Stitched {
            len: got,
            to_eof: got == in_rope && self.eof,
        }
    }

    pub(crate) fn release_before(&mut self, upto_byte: u64, spare_out: &mut Option<Vec<u8>>) {
        while let Some(front) = self.slabs.front() {
            if front.end() <= upto_byte {
                let popped = self.slabs.pop_front().unwrap();
                self.floor_byte = popped.end();
                if spare_out.is_none() && popped.data.capacity() >= COALESCE_MIN {
                    if let Ok(vec) = Arc::try_unwrap(popped.data) {
                        *spare_out = Some(vec);
                    }
                }
            } else {
                break;
            }
        }
    }
}

/// Zero-overhead `S = 1` scanner over `SlabRope` + `read_item_tight` (`workers <= 3`).
pub(crate) fn single_slab_scanner(shared: &Shared, rx: Receiver<Vec<u8>>, jobs_tx: JobsTx) {
    let jobs = jobs_tx.jobs();
    let max_syms_pool = (shared.slots as usize).min(6);
    let mut rope = SlabRope::new();
    let mut scan_scratch = ScanScratch::default();
    let mut stitch = Vec::new();
    let mut phase = ChainPhase::Header { first: true };
    let mut bit = 0u64;
    let mut blocks = 0u64;
    let mut min_want = 0usize;

    let finish = |res: Option<Error>| {
        let mut st = shared.lock();
        let at = st.issued;
        match res {
            Some(e) if e.kind != ErrorKind::Cancelled => {
                st.ready.insert(at, Err(e));
                st.issued += 1;
                st.end = Some(at + 1);
            }
            _ => st.end = Some(at),
        }
        drop(st);
        shared.cv.notify_one();
    };

    let recv_until = |rope: &mut SlabRope, need_end: u64| -> Result<(), ErrorKind> {
        while !rope.eof && rope.total_bytes() < need_end {
            match rx.recv() {
                Ok(chunk) => {
                    rope.push_chunk(chunk);
                    while rope.live_bytes() < MAX_ROPE_LIVE {
                        match rx.try_recv() {
                            Ok(more) => rope.push_chunk(more),
                            Err(std::sync::mpsc::TryRecvError::Empty) => break,
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                rope.set_eof();
                                break;
                            }
                        }
                    }
                }
                Err(_) => rope.set_eof(),
            }
            if shared.stopped.load(Ordering::Acquire) {
                return Err(ErrorKind::Cancelled);
            }
        }
        Ok(())
    };

    loop {
        if shared.stopped.load(Ordering::Acquire) {
            return finish(None);
        }
        match phase {
            ChainPhase::Finished => return finish(None),
            ChainPhase::Header { first } => {
                let byte_at = bit / 8;
                if let Err(k) = recv_until(&mut rope, byte_at + 4) {
                    return finish(Some(Error {
                        kind: k,
                        block: None,
                        bit: Some(bit),
                    }));
                }
                let avail = rope.total_bytes().saturating_sub(byte_at);
                if avail == 0 && rope.eof {
                    if first {
                        return finish(Some(Error {
                            kind: ErrorKind::Empty,
                            block: None,
                            bit: Some(bit),
                        }));
                    }
                    return finish(None);
                }
                if avail < 4 {
                    return finish(Some(Error {
                        kind: if first {
                            ErrorKind::NotBz2
                        } else {
                            ErrorKind::TrailingData
                        },
                        block: None,
                        bit: Some(bit),
                    }));
                }
                let mut hdr = [0u8; 4];
                rope.copy_at(byte_at, &mut hdr);
                match hdr {
                    [b'B', b'Z', b'h', l @ b'1'..=b'9'] => {
                        phase = ChainPhase::Stream {
                            level: (l - b'0') as u32,
                            combined: 0,
                        };
                        bit += 32;
                    }
                    _ => {
                        return finish(Some(Error {
                            kind: if first {
                                ErrorKind::NotBz2
                            } else {
                                ErrorKind::TrailingData
                            },
                            block: None,
                            bit: Some(bit),
                        }));
                    }
                }
            }
            ChainPhase::Stream { level, combined } => {
                let syms = {
                    let mut st = shared.lock();
                    while !st.cancelled && shared.full(&st) {
                        st.record_capped_wait_if_any(shared);
                        st.scanner_waiting = true;
                        st = shared.scan_cv.wait(st).unwrap_or_else(|e| e.into_inner());
                    }
                    st.scanner_waiting = false;
                    if st.cancelled {
                        drop(st);
                        return finish(None);
                    }
                    st.free_syms.pop().unwrap_or_default()
                };

                let start_byte = bit / 8;
                let want_bytes = min_want.max(INIT_NEED_MORE);
                if rope.total_bytes() < start_byte + want_bytes as u64 && !rope.eof {
                    if let Err(k) = recv_until(&mut rope, start_byte + want_bytes as u64) {
                        return finish(Some(Error {
                            kind: k,
                            block: Some(blocks),
                            bit: Some(bit),
                        }));
                    }
                }

                let view = rope.view_for_cand(start_byte, min_want, None, &mut stitch);
                let (slice, to_eof): (&[u8], bool) = match &view {
                    RopeView::ZeroCopy {
                        slab,
                        offset,
                        len,
                        to_eof,
                    } => (&slab[*offset..*offset + *len], *to_eof),
                    RopeView::Stitched { len, to_eof } => (&stitch[..*len], *to_eof),
                };
                let slice_len = slice.len();
                let bit_mod = bit % 8;
                let r = catch_unwind(AssertUnwindSafe(|| {
                    let mut br = Bits::at(slice, bit_mod);
                    let item = read_item_tight(&mut br, level, syms, &mut scan_scratch);
                    (item, br.overran(), br.bit_pos())
                }));
                drop(view);
                if stitch.capacity() > MAX_STITCH_RETAIN {
                    stitch = Vec::new();
                }

                match r {
                    Err(p) => {
                        return finish(Some(Error {
                            kind: ErrorKind::Internal(panic_message(p)),
                            block: Some(blocks),
                            bit: Some(bit),
                        }));
                    }
                    Ok((item, overran, bit_pos)) => {
                        if overran {
                            let back_syms = match item {
                                Ok(Item::Block(b)) => b.syms,
                                Ok(Item::End(_)) => Vec::new(),
                                Err((_, s)) => s,
                            };
                            if back_syms.capacity() > 0 {
                                let mut st = shared.lock();
                                if st.free_syms.len() < max_syms_pool {
                                    st.free_syms.push(back_syms);
                                }
                            }
                            if to_eof {
                                return finish(Some(Error {
                                    kind: ErrorKind::Truncated,
                                    block: Some(blocks),
                                    bit: Some(bit),
                                }));
                            }
                            if slice_len >= MAX_ITEM || min_want >= MAX_ITEM {
                                return finish(Some(Error {
                                    kind: ErrorKind::BlockTooLarge,
                                    block: Some(blocks),
                                    bit: Some(bit),
                                }));
                            }
                            min_want = slice_len
                                .saturating_mul(2)
                                .max(min_want.saturating_mul(2))
                                .max(INIT_NEED_MORE * 2)
                                .min(MAX_ITEM);
                            continue;
                        }
                        min_want = 0;
                        let end_bit = start_byte * 8 + bit_pos;
                        match item {
                            Err((kind, _)) => {
                                return finish(Some(Error {
                                    kind,
                                    block: Some(blocks),
                                    bit: Some(bit),
                                }));
                            }
                            Ok(Item::End(stored)) => {
                                if stored != combined {
                                    return finish(Some(Error {
                                        kind: ErrorKind::StreamCrc {
                                            stored,
                                            computed: combined,
                                        },
                                        block: Some(blocks),
                                        bit: Some(bit),
                                    }));
                                }
                                bit = end_bit.div_ceil(8) * 8;
                                phase = ChainPhase::Header { first: false };
                                let mut spare = None;
                                rope.release_before(bit / 8, &mut spare);
                                if let Some(chunk) = spare {
                                    let mut st = shared.lock();
                                    if st.free_in.is_empty() {
                                        st.free_in.push(chunk);
                                    }
                                }
                            }
                            Ok(Item::Block(symbols)) => {
                                let at = bit;
                                let place = blocks;
                                blocks += 1;
                                bit = end_bit;
                                phase = ChainPhase::Stream {
                                    level,
                                    combined: combined.rotate_left(1) ^ symbols.crc,
                                };
                                let mut spare = None;
                                rope.release_before(bit / 8, &mut spare);
                                {
                                    let mut st = shared.lock();
                                    if let Some(chunk) = spare {
                                        if st.free_in.is_empty() {
                                            st.free_in.push(chunk);
                                        }
                                    }
                                    st.issued = blocks;
                                }
                                jobs.push((place, at, symbols));
                            }
                        }
                    }
                }
            }
        }
    }
}

enum CandDone {
    Block { end_bit: u64, symbols: Symbols },
    End { end_bit: u64, stored_crc: u32 },
    Err { kind: ErrorKind },
}

enum CandState {
    Unclaimed { min_want: usize },
    Claimed,
    NeedMore { min_want: usize },
    Done(CandDone),
}

struct CandSlot {
    id: u64,
    bit: u64,
    is_end: bool,
    state: CandState,
}

#[derive(Clone, Copy)]
enum ChainPhase {
    Header { first: bool },
    Stream { level: u32, combined: u32 },
    Finished,
}

struct HubState {
    rope: SlabRope,
    cands: VecDeque<CandSlot>,
    next_cand_id: u64,
    phase: ChainPhase,
    chain_bit: u64,
    blocks_issued: u64,
    pending_publishes: usize,
    lead_exited: bool,
    terminal_err: Option<Error>,
}

/// Shared parallel-front-end coordinator between `mibz2-scan` and `1..2` lightweight
/// `54 KiB` L1d-resident helper scanner threads (`mibz2-scan-1`, `mibz2-scan-2`).
pub(crate) struct ParScanHub {
    mu: Mutex<HubState>,
    pub(crate) helper_cv: Condvar,
    max_front_ahead: usize,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn panic_message(p: Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| p.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic".into())
}

struct ClaimedWork {
    cand_id: u64,
    bit: u64,
    bound_level: u32,
    min_want: usize,
    view: RopeView,
}

struct CommitBatch {
    jobs: Vec<(u64, u64, Symbols)>,
    recycled_syms: Vec<Vec<u16>>,
    spare_chunk: Option<Vec<u8>>,
    finished: Option<Option<Error>>,
    blocks_issued: u64,
    tracked_publish: bool,
}

impl ParScanHub {
    pub(crate) fn new(workers: usize, slots: usize) -> Self {
        let max_front_ahead = (workers + 4).min(slots).max(2);
        Self {
            mu: Mutex::new(HubState {
                rope: SlabRope::new(),
                cands: VecDeque::with_capacity(32),
                next_cand_id: 0,
                phase: ChainPhase::Header { first: true },
                chain_bit: 0,
                blocks_issued: 0,
                pending_publishes: 0,
                lead_exited: false,
                terminal_err: None,
            }),
            helper_cv: Condvar::new(),
            max_front_ahead,
        }
    }

    fn advance_chain_locked(st: &mut HubState) -> CommitBatch {
        let mut batch = CommitBatch {
            jobs: Vec::new(),
            recycled_syms: Vec::new(),
            spare_chunk: None,
            finished: None,
            blocks_issued: st.blocks_issued,
            tracked_publish: false,
        };
        loop {
            match st.phase {
                ChainPhase::Finished => break,
                ChainPhase::Header { first } => {
                    let at = st.chain_bit;
                    let byte_at = at / 8;
                    let avail = st.rope.total_bytes().saturating_sub(byte_at);
                    if avail == 0 && st.rope.eof {
                        st.phase = ChainPhase::Finished;
                        if first {
                            let e = Error {
                                kind: ErrorKind::Empty,
                                block: None,
                                bit: Some(at),
                            };
                            st.terminal_err = Some(e.clone());
                            batch.finished = Some(Some(e));
                        } else {
                            batch.finished = Some(None);
                        }
                        break;
                    }
                    if avail < 4 {
                        if st.rope.eof {
                            st.phase = ChainPhase::Finished;
                            let e = Error {
                                kind: if first {
                                    ErrorKind::NotBz2
                                } else {
                                    ErrorKind::TrailingData
                                },
                                block: None,
                                bit: Some(at),
                            };
                            st.terminal_err = Some(e.clone());
                            batch.finished = Some(Some(e));
                        }
                        break;
                    }
                    let mut hdr = [0u8; 4];
                    let got = st.rope.copy_at(byte_at, &mut hdr);
                    debug_assert_eq!(got, 4);
                    match hdr {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9'] => {
                            st.phase = ChainPhase::Stream {
                                level: (l - b'0') as u32,
                                combined: 0,
                            };
                            st.chain_bit += 32;
                            Self::drop_before_chain(st, &mut batch.recycled_syms);
                            st.rope
                                .release_before(st.chain_bit / 8, &mut batch.spare_chunk);
                        }
                        _ => {
                            st.phase = ChainPhase::Finished;
                            let e = Error {
                                kind: if first {
                                    ErrorKind::NotBz2
                                } else {
                                    ErrorKind::TrailingData
                                },
                                block: None,
                                bit: Some(at),
                            };
                            st.terminal_err = Some(e.clone());
                            batch.finished = Some(Some(e));
                            break;
                        }
                    }
                }
                ChainPhase::Stream { level, combined } => {
                    Self::drop_before_chain(st, &mut batch.recycled_syms);
                    let at = st.chain_bit;
                    let index = st.blocks_issued;
                    let head_matches = st.cands.front().is_some_and(|c| c.bit == at);
                    if !head_matches {
                        let byte_at = at / 8;
                        let avail = st.rope.total_bytes().saturating_sub(byte_at);
                        if !st.rope.eof && avail < 16 {
                            break;
                        }
                        let mut tmp = [0u8; 16];
                        let n = st.rope.copy_at(byte_at, &mut tmp);
                        let mut br = Bits::at(&tmp[..n], at % 8);
                        let item = block::read_item(&mut br, level, Vec::new());
                        let kind = if br.overran() {
                            ErrorKind::Truncated
                        } else {
                            match item {
                                Err((k, _)) => k,
                                Ok(_) => ErrorKind::BadBlockMagic,
                            }
                        };
                        st.phase = ChainPhase::Finished;
                        let e = Error {
                            kind,
                            block: Some(index),
                            bit: Some(at),
                        };
                        st.terminal_err = Some(e.clone());
                        batch.finished = Some(Some(e));
                        break;
                    }

                    let is_done = matches!(st.cands.front().unwrap().state, CandState::Done(_));
                    if !is_done {
                        break;
                    }
                    let slot = st.cands.pop_front().unwrap();
                    let CandState::Done(done) = slot.state else {
                        unreachable!()
                    };
                    match done {
                        CandDone::End {
                            end_bit,
                            stored_crc,
                        } => {
                            if stored_crc != combined {
                                st.phase = ChainPhase::Finished;
                                let e = Error {
                                    kind: ErrorKind::StreamCrc {
                                        stored: stored_crc,
                                        computed: combined,
                                    },
                                    block: Some(index),
                                    bit: Some(at),
                                };
                                st.terminal_err = Some(e.clone());
                                batch.finished = Some(Some(e));
                                break;
                            }
                            st.chain_bit = end_bit.div_ceil(8) * 8;
                            st.phase = ChainPhase::Header { first: false };
                            Self::drop_before_chain(st, &mut batch.recycled_syms);
                            st.rope
                                .release_before(st.chain_bit / 8, &mut batch.spare_chunk);
                        }
                        CandDone::Err { kind } => {
                            st.phase = ChainPhase::Finished;
                            let e = Error {
                                kind,
                                block: Some(index),
                                bit: Some(at),
                            };
                            st.terminal_err = Some(e.clone());
                            batch.finished = Some(Some(e));
                            break;
                        }
                        CandDone::Block {
                            end_bit,
                            mut symbols,
                        } => {
                            let max_len = 100_000 * (level as usize);
                            symbols.max_len = max_len;
                            if symbols.orig_ptr as usize > 10 + max_len {
                                batch.recycled_syms.push(symbols.syms);
                                st.phase = ChainPhase::Finished;
                                let e = Error {
                                    kind: ErrorKind::BadOrigPtr,
                                    block: Some(index),
                                    bit: Some(at),
                                };
                                st.terminal_err = Some(e.clone());
                                batch.finished = Some(Some(e));
                                break;
                            }
                            st.phase = ChainPhase::Stream {
                                level,
                                combined: combined.rotate_left(1) ^ symbols.crc,
                            };
                            st.blocks_issued += 1;
                            st.chain_bit = end_bit;
                            Self::drop_before_chain(st, &mut batch.recycled_syms);
                            st.rope
                                .release_before(st.chain_bit / 8, &mut batch.spare_chunk);
                            batch.jobs.push((index, at, symbols));
                        }
                    }
                }
            }
        }
        batch.blocks_issued = st.blocks_issued;
        if !batch.jobs.is_empty() || batch.finished.is_some() {
            st.pending_publishes += 1;
            batch.tracked_publish = true;
        }
        batch
    }

    fn drop_before_chain(st: &mut HubState, recycled: &mut Vec<Vec<u16>>) {
        while st.cands.front().is_some_and(|c| c.bit < st.chain_bit) {
            let dropped = st.cands.pop_front().unwrap();
            if let CandState::Done(CandDone::Block { symbols, .. }) = dropped.state {
                if symbols.syms.capacity() > 0 {
                    recycled.push(symbols.syms);
                }
            }
        }
    }

    fn publish_batch(&self, shared: &Shared, jobs: &Jobs, batch: CommitBatch) {
        if batch.jobs.is_empty()
            && batch.recycled_syms.is_empty()
            && batch.spare_chunk.is_none()
            && batch.finished.is_none()
        {
            return;
        }
        let max_pool = self.max_front_ahead;
        let mut wake_reader = false;
        {
            let mut sst = shared.lock();
            if let Some(chunk) = batch.spare_chunk {
                if sst.free_in.is_empty() {
                    sst.free_in.push(chunk);
                }
            }
            for s in batch.recycled_syms {
                if s.capacity() > 0 && sst.free_syms.len() < max_pool {
                    sst.free_syms.push(s);
                }
            }
            sst.issued = sst.issued.max(batch.blocks_issued);
            if let Some(fin) = &batch.finished {
                let at = batch.blocks_issued;
                match fin {
                    Some(e) if e.kind != ErrorKind::Cancelled => {
                        sst.ready.insert(at, Err(e.clone()));
                        sst.issued = sst.issued.max(at + 1);
                        sst.end = Some(at + 1);
                    }
                    _ => sst.end = Some(at),
                }
                wake_reader = true;
            }
        }
        for job in batch.jobs {
            jobs.push(job);
        }
        if batch.tracked_publish {
            let wake_lead = {
                let mut st = lock(&self.mu);
                st.pending_publishes -= 1;
                st.pending_publishes == 0 && matches!(st.phase, ChainPhase::Finished)
            };
            if wake_lead {
                shared.scan_cv.notify_all();
            }
        }
        if wake_reader {
            shared.cv.notify_one();
            shared.scan_cv.notify_all();
        }
    }

    pub(crate) fn scan_step(
        &self,
        shared: &Shared,
        jobs: &Jobs,
        scan_scratch: &mut ScanScratch,
        stitch_buf: &mut Vec<u8>,
        local_syms: &mut Vec<u16>,
    ) -> bool {
        if shared.stopped.load(Ordering::Acquire) {
            return false;
        }
        let allow_spec = {
            let mut sst = shared.lock();
            if sst.cancelled || shared.full(&sst) {
                return false;
            }
            let ahead = (sst.issued - sst.taken) as usize;
            if ahead >= self.max_front_ahead {
                return false;
            }
            if local_syms.capacity() == 0 {
                if let Some(s) = sst.free_syms.pop() {
                    *local_syms = s;
                }
            }
            (self.max_front_ahead - ahead).max(1)
        };

        let claimed = {
            let mut st = lock(&self.mu);
            let batch = Self::advance_chain_locked(&mut st);
            let has_batch = !batch.jobs.is_empty()
                || !batch.recycled_syms.is_empty()
                || batch.spare_chunk.is_some()
                || batch.finished.is_some();
            if has_batch {
                drop(st);
                self.publish_batch(shared, jobs, batch);
                return true;
            }
            if matches!(st.phase, ChainPhase::Finished) {
                None
            } else {
                Self::claim_cand_locked(&mut st, allow_spec, stitch_buf)
            }
        };

        let Some(work) = claimed else {
            return false;
        };

        let outcome = Self::run_scan(&work, stitch_buf, scan_scratch, local_syms);
        drop(work.view);
        if stitch_buf.capacity() > MAX_STITCH_RETAIN {
            *stitch_buf = Vec::new();
        }

        let (batch, wake_lead) = {
            let mut st = lock(&self.mu);
            let mut wake = false;
            if let Some(pos) = st.cands.iter().position(|c| c.id == work.cand_id) {
                st.cands[pos].state = outcome;
                if pos == 0 {
                    wake = true;
                }
            }
            let batch = Self::advance_chain_locked(&mut st);
            if matches!(st.phase, ChainPhase::Finished)
                || st
                    .cands
                    .front()
                    .is_some_and(|c| matches!(c.state, CandState::NeedMore { .. }))
                || !st.cands.iter().any(|c| matches!(c.state, CandState::Claimed))
            {
                wake = true;
            }
            (batch, wake)
        };
        self.publish_batch(shared, jobs, batch);
        if wake_lead {
            shared.scan_cv.notify_one();
        }
        true
    }

    fn claim_cand_locked(
        st: &mut HubState,
        allow_spec: usize,
        stitch_buf: &mut Vec<u8>,
    ) -> Option<ClaimedWork> {
        let limit = allow_spec.min(st.cands.len());
        let mut saw_end = matches!(st.phase, ChainPhase::Header { .. });
        let stream_level = match st.phase {
            ChainPhase::Stream { level, .. } => level,
            _ => 9,
        };
        for idx in 0..limit {
            let has_next_horizon = idx + 1 < st.cands.len() || st.rope.eof;
            if !has_next_horizon {
                break;
            }
            let min_want = match st.cands[idx].state {
                CandState::Unclaimed { min_want } => min_want,
                _ => {
                    if st.cands[idx].is_end {
                        saw_end = true;
                    }
                    continue;
                }
            };
            let bit = st.cands[idx].bit;
            let cand_id = st.cands[idx].id;
            let is_end = st.cands[idx].is_end;
            let next_cand_byte = st.cands.get(idx + 1).map(|c| c.bit / 8);
            let bound_level = if is_end || saw_end { 9 } else { stream_level };
            let view = st
                .rope
                .view_for_cand(bit / 8, min_want, next_cand_byte, stitch_buf);
            st.cands[idx].state = CandState::Claimed;
            return Some(ClaimedWork {
                cand_id,
                bit,
                bound_level,
                min_want,
                view,
            });
        }
        None
    }

    fn run_scan(
        work: &ClaimedWork,
        stitch_buf: &[u8],
        scratch: &mut ScanScratch,
        syms: &mut Vec<u16>,
    ) -> CandState {
        let (slice, to_eof): (&[u8], bool) = match &work.view {
            RopeView::ZeroCopy {
                slab,
                offset,
                len,
                to_eof,
            } => (&slab[*offset..*offset + *len], *to_eof),
            RopeView::Stitched { len, to_eof } => (&stitch_buf[..*len], *to_eof),
        };
        let slice_len = slice.len();
        let bit_mod = work.bit % 8;
        let taken_syms = std::mem::take(syms);
        let res = catch_unwind(AssertUnwindSafe(|| {
            let mut br = Bits::at(slice, bit_mod);
            let item = read_item_tight(&mut br, work.bound_level, taken_syms, scratch);
            (item, br.overran(), br.bit_pos())
        }));
        match res {
            Ok((item, overran, bit_pos)) => {
                if overran {
                    *syms = match item {
                        Ok(Item::Block(b)) => b.syms,
                        Ok(Item::End(_)) => Vec::new(),
                        Err((_, s)) => s,
                    };
                    if to_eof {
                        return CandState::Done(CandDone::Err {
                            kind: ErrorKind::Truncated,
                        });
                    }
                    if slice_len >= MAX_ITEM || work.min_want >= MAX_ITEM {
                        return CandState::Done(CandDone::Err {
                            kind: ErrorKind::BlockTooLarge,
                        });
                    }
                    let next_want = slice_len
                        .saturating_mul(2)
                        .max(work.min_want.saturating_mul(2))
                        .max(INIT_NEED_MORE * 2)
                        .min(MAX_ITEM);
                    return CandState::NeedMore {
                        min_want: next_want,
                    };
                }
                let end_bit = (work.bit / 8) * 8 + bit_pos;
                match item {
                    Ok(Item::Block(b)) => CandState::Done(CandDone::Block {
                        end_bit,
                        symbols: b,
                    }),
                    Ok(Item::End(stored_crc)) => CandState::Done(CandDone::End {
                        end_bit,
                        stored_crc,
                    }),
                    Err((kind, s)) => {
                        *syms = s;
                        CandState::Done(CandDone::Err { kind })
                    }
                }
            }
            Err(p) => {
                *scratch = ScanScratch::default();
                CandState::Done(CandDone::Err {
                    kind: ErrorKind::Internal(panic_message(p)),
                })
            }
        }
    }
}

/// Primary coordinator + parallel scanner thread (`mibz2-scan`).
pub(crate) fn par_scanner_lead(
    shared: &Shared,
    hub: &ParScanHub,
    rx: Receiver<Vec<u8>>,
    jobs_tx: JobsTx,
) {
    struct OnLeadExit<'a> {
        shared: &'a Shared,
        hub: &'a ParScanHub,
    }
    impl Drop for OnLeadExit<'_> {
        fn drop(&mut self) {
            {
                let mut st = lock(&self.hub.mu);
                st.lead_exited = true;
            }
            self.hub.helper_cv.notify_all();
            self.shared.scan_cv.notify_all();
        }
    }
    let _on_exit = OnLeadExit { shared, hub };
    let jobs = jobs_tx.jobs();
    let mut scan_scratch = ScanScratch::default();
    let mut stitch_buf: Vec<u8> = Vec::new();
    let mut local_syms: Vec<u16> = Vec::new();
    let mut raw_magics: Vec<(u64, bool)> = Vec::with_capacity(64);

    loop {
        if shared.stopped.load(Ordering::Acquire) {
            return;
        }

        let (need_input_blocking, want_prestage, is_finished, promoted) = {
            let mut st = lock(&hub.mu);
            let batch = ParScanHub::advance_chain_locked(&mut st);
            let fin = matches!(st.phase, ChainPhase::Finished);

            let (rope_total, rope_eof) = (st.rope.total_bytes(), st.rope.eof);
            let mut promoted = false;
            for c in st.cands.iter_mut() {
                if let CandState::NeedMore { min_want } = c.state {
                    let avail = rope_total.saturating_sub(c.bit / 8) as usize;
                    if rope_eof || avail >= min_want {
                        c.state = CandState::Unclaimed { min_want };
                        promoted = true;
                    }
                }
            }

            let head_needs_bytes = !fin
                && !st.rope.eof
                && (match st.phase {
                    ChainPhase::Header { .. } => {
                        st.rope.total_bytes().saturating_sub(st.chain_bit / 8) < 4
                    }
                    ChainPhase::Stream { .. } => {
                        if let Some(front) = st.cands.front() {
                            if front.bit > st.chain_bit {
                                st.rope.total_bytes().saturating_sub(st.chain_bit / 8) < 16
                            } else if matches!(front.state, CandState::NeedMore { .. }) {
                                true
                            } else {
                                st.cands.len() < 2
                            }
                        } else {
                            true
                        }
                    }
                    ChainPhase::Finished => false,
                });

            let want_prestage =
                !fin && !st.rope.eof && st.rope.live_bytes() <= PRESTAGE_ROPE_BYTES;

            drop(st);
            hub.publish_batch(shared, jobs, batch);
            let done_and_drained = fin && lock(&hub.mu).pending_publishes == 0;
            (head_needs_bytes, want_prestage, done_and_drained, promoted)
        };

        if is_finished {
            return;
        }
        if promoted && jobs.queued_len() < 3 {
            hub.helper_cv.notify_all();
        }

        if want_prestage && !need_input_blocking {
            let mut got_any = false;
            let mut hit_eof = false;
            {
                let mut st = lock(&hub.mu);
                while st.rope.live_bytes() < MAX_ROPE_LIVE {
                    match rx.try_recv() {
                        Ok(chunk) => {
                            st.rope.push_chunk(chunk);
                            got_any = true;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            st.rope.set_eof();
                            hit_eof = true;
                            break;
                        }
                    }
                }
                if got_any || hit_eof {
                    ingest_magics_locked(&mut st, &mut raw_magics);
                }
            }
            if (got_any || hit_eof) && jobs.queued_len() < 3 {
                hub.helper_cv.notify_all();
            }
        }

        if need_input_blocking {
            match rx.recv() {
                Ok(chunk) => {
                    let mut st = lock(&hub.mu);
                    st.rope.push_chunk(chunk);
                    while st.rope.live_bytes() < MAX_ROPE_LIVE {
                        match rx.try_recv() {
                            Ok(more) => st.rope.push_chunk(more),
                            Err(std::sync::mpsc::TryRecvError::Empty) => break,
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                st.rope.set_eof();
                                break;
                            }
                        }
                    }
                    ingest_magics_locked(&mut st, &mut raw_magics);
                }
                Err(_) => {
                    let mut st = lock(&hub.mu);
                    st.rope.set_eof();
                    ingest_magics_locked(&mut st, &mut raw_magics);
                }
            }
            if jobs.queued_len() < 3 {
                hub.helper_cv.notify_all();
            }
            continue;
        }

        if hub.scan_step(
            shared,
            jobs,
            &mut scan_scratch,
            &mut stitch_buf,
            &mut local_syms,
        ) {
            continue;
        }

        let can_pull_more = {
            let st = lock(&hub.mu);
            if matches!(st.phase, ChainPhase::Finished) && st.pending_publishes == 0 {
                return;
            }
            if matches!(st.phase, ChainPhase::Finished) {
                false
            } else {
                let any_unclaimed_without_horizon = !st.rope.eof
                    && st.cands.len() == 1
                    && matches!(st.cands[0].state, CandState::Unclaimed { .. });
                any_unclaimed_without_horizon
                    || (!st.rope.eof
                        && st.rope.live_bytes() < MAX_ROPE_LIVE
                        && st
                            .cands
                            .iter()
                            .all(|c| !matches!(c.state, CandState::Unclaimed { .. })))
            }
        };
        if can_pull_more && !shared.lock().cancelled {
            let full_now = {
                let sst = shared.lock();
                shared.full(&sst) || (sst.issued - sst.taken) as usize >= hub.max_front_ahead
            };
            if !full_now {
                match rx.recv() {
                    Ok(chunk) => {
                        let mut st = lock(&hub.mu);
                        st.rope.push_chunk(chunk);
                        ingest_magics_locked(&mut st, &mut raw_magics);
                    }
                    Err(_) => {
                        let mut st = lock(&hub.mu);
                        st.rope.set_eof();
                        ingest_magics_locked(&mut st, &mut raw_magics);
                    }
                }
                if jobs.queued_len() < 3 {
                    hub.helper_cv.notify_all();
                }
                continue;
            }
        }

        let mut sst = shared.lock();
        if sst.cancelled {
            return;
        }
        if shared.full(&sst) {
            sst.record_capped_wait_if_any(shared);
        }
        sst.scanner_waiting = true;
        sst = shared.scan_cv.wait(sst).unwrap_or_else(|e| e.into_inner());
        sst.scanner_waiting = false;
        drop(sst);
        if jobs.queued_len() < 3 {
            hub.helper_cv.notify_all();
        }
    }
}

/// Demand-driven lightweight `54 KiB` L1d-resident helper scanner thread (`mibz2-scan-1`,
/// `mibz2-scan-2`).
pub(crate) fn par_scanner_helper(shared: &Shared, hub: &ParScanHub, jobs: &Jobs) {
    let mut scan_scratch = ScanScratch::default();
    let mut stitch_buf: Vec<u8> = Vec::new();
    let mut local_syms: Vec<u16> = Vec::new();
    loop {
        if shared.stopped.load(Ordering::Acquire) {
            return;
        }
        if jobs.queued_len() < 3
            && hub.scan_step(
                shared,
                jobs,
                &mut scan_scratch,
                &mut stitch_buf,
                &mut local_syms,
            )
        {
            continue;
        }
        let st = lock(&hub.mu);
        if st.lead_exited || matches!(st.phase, ChainPhase::Finished) || jobs.is_closed() {
            return;
        }
        let _st = hub.helper_cv.wait(st).unwrap_or_else(|e| e.into_inner());
    }
}

fn ingest_magics_locked(st: &mut HubState, raw_magics: &mut Vec<(u64, bool)>) {
    raw_magics.clear();
    st.rope.scan_new_anchors(raw_magics);
    for &(bit, is_end) in raw_magics.iter() {
        if bit < st.chain_bit {
            continue;
        }
        let id = st.next_cand_id;
        st.next_cand_id += 1;
        st.cands.push_back(CandSlot {
            id,
            bit,
            is_end,
            state: CandState::Unclaimed { min_want: 0 },
        });
    }
}

/// Whole-slice convenience wrapper delegating through bounded `StreamDecoder::parscan`
/// (`4 MiB` recycled chunks + `READY_BYTES` output cap).
pub fn decode_parallel_slice(data: &[u8], threads: usize, k: Kernels) -> Result<Vec<u8>, Error> {
    if threads <= 1 {
        return crate::decode_with::<crate::bwt::Cells>(data, k);
    }
    let mut out = Vec::with_capacity(data.len().saturating_mul(4));
    decode_parallel_slice_with(data, threads, SLOTS_PER_THREAD * threads.max(1), k, |b| {
        out.extend_from_slice(b);
    })?;
    Ok(out)
}

/// Streaming block-callback wrapper over `StreamDecoder::parscan`.
pub fn decode_parallel_slice_with<F: FnMut(&[u8])>(
    data: &[u8],
    threads: usize,
    slots: usize,
    k: Kernels,
    mut on_block: F,
) -> Result<(), Error> {
    let dec = StreamDecoder::parscan(threads, slots, k);
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in data.chunks(4 << 20) {
                let mut chunk = dec.spare();
                chunk.clear();
                chunk.extend_from_slice(c);
                if dec.write(chunk).is_err() {
                    return;
                }
            }
            dec.close();
        });
        struct CancelGuard<'a>(&'a StreamDecoder);
        impl Drop for CancelGuard<'_> {
            fn drop(&mut self) {
                self.0.cancel();
            }
        }
        let _cancel = CancelGuard(&dec);
        let mut spent = Vec::new();
        loop {
            match dec.read_block(spent) {
                Ok(None) => break Ok(()),
                Ok(Some(b)) => {
                    on_block(&b);
                    spent = b;
                }
                Err(e) => break Err(e),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slab_rope_seam_invariance_across_chunk_sizes() {
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let len = 2048usize;
        let mut base = vec![0u8; len];
        for b in &mut base {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *b = x as u8;
        }
        let plants = [
            (0u64, false),
            (51, false),
            (509, true),
            (1021, false),
            (4093, true),
            ((len as u64 * 8) - 48, true),
        ];
        for &(bit, is_end) in &plants {
            let m = if is_end { END_MAGIC } else { BLOCK_MAGIC };
            for i in 0..48u64 {
                let pos = bit + i;
                let bit_val = ((m >> (47 - i)) & 1) as u8;
                let byte_idx = (pos / 8) as usize;
                let mask = 1u8 << (7 - (pos % 8));
                if bit_val != 0 {
                    base[byte_idx] |= mask;
                } else {
                    base[byte_idx] &= !mask;
                }
            }
        }
        let mut ref_raw = Vec::new();
        find_magics_anchor_range(&base, 1, 1 + (len - 2) / 4, &mut ref_raw);
        let want: Vec<(u64, bool)> = ref_raw.iter().map(|r| (r.bit, r.is_end)).collect();
        for &(bit, is_end) in &plants {
            assert!(want.contains(&(bit, is_end)), "missing planted ({bit}, {is_end})");
        }

        for chunk_sz in [1usize, 2, 3, 4, 5, 7, 8, 13, 64, 513, len] {
            let mut rope = SlabRope::new();
            let mut got = Vec::new();
            for c in base.chunks(chunk_sz) {
                rope.push_chunk(c.to_vec());
                rope.scan_new_anchors(&mut got);
            }
            rope.set_eof();
            rope.scan_new_anchors(&mut got);
            assert_eq!(got, want, "mismatch at chunk_sz={chunk_sz}");
        }
    }
}
