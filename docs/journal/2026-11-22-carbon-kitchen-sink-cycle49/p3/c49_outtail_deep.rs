#![forbid(unsafe_code)]

//! Cycle 49 P3 — Part B (Strictly Modern `2020s` CS Systems Engineering — Separate from Classical Share!):
//! Root-Causing `I3+OT64`'s `+0.018` `instr` Overhead & Multi-Buffer `OutTail-Deep`
//!
//! Investigates and benchmarks 7 arms apples-to-apples over `cand_r5_head` (`25be0973d`):
//!   - Arm 0 (`R5-Head`): verbatim `cand_r5_head` (`ID_BITS=22`, persistent `bands`, default pre-reserves)
//!   - Arm 1 (`F-Inline3`): Cycle 47 `#729` `instr=0.920` baseline (`ID_BITS=24`, persistent `bands`)
//!   - Arm 2 (`I3+OT64-C48`): Cycle 48 `I3+OT64` reproduction (`chunks_exact(8)` + checked S1 + `TrimWS`)
//!   - Arm 3 (`I3+OT64-Clean`): Fixes ALL `+0.018` `instr` causes:
//!       (a) single `nrec > n` upfront guard eliminating per-record `width == 0` & `checked_add` in `S1`,
//!       (b) Rust `slice::as_chunks_mut::<8>()` / `as_chunks::<8>()` eliminating `Option::unwrap` &
//!           `try_into().unwrap()` in `S3a`/`S4`,
//!       (c) `#[cold] #[inline(never)]` fallbacks & cold `TrimWS` growth helpers,
//!       (d) stack-local `next_inline: [u32; 896]` & `stack_inline: [u32; 64]` (zero persistent `spill.next`
//!           & `stack` heap allocations on all non-overflow blocks!).
//!   - Arm 4 (`OT-Nodes`): Places `open_nodes` (`&mut [[u8; 16]]`) + `image_order` (reused by `bands`
//!       in `S3a`) + `bands` (`&mut [[u8; 8]]`) in `out_tail`, while `rules` lives in `self.edges[0..]`
//!       (`base = 0`!), eliminating `self.edges[m..arena_peak]` touches and `S2a` node WS!
//!   - Arm 5 (`OT-FullArena`): Places ALL pre-`S5`-dead `S2a/S2b/S3a` buffers (`open_nodes` + `rules` +
//!       `image_order` + `bands` + `S3a` cache) inside `out_tail` (`&mut [[u8; 16]]` + `&mut [[u8; 8]]`)
//!       whenever they fit, with seamless spill/fallback to `self.edges` / `self.bands` if `out_tail`
//!       is insufficient — zero `self.edges` writes during `S2a..S3a`!
//!   - Arm 6 (`OT-Full+TinyFL`): Arm 5 + zero-WS `PlainRunWalk` for `n < 32_768` reusing `self.edges`
//!       (`+0 B` persistent WS, fixing `C48-Fused`'s `+128 KiB` `plain_fl` RSS tax!).
//!
//! Strictly `#![forbid(unsafe_code)]`, `-D warnings`, forward-`FL` exact on malformed multi-cycle columns.

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS; // 255

#[inline(always)]
const fn literal(byte: u8) -> u32 {
    (1u32 << ID_BITS) | (byte as u32)
}

#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 {
        (self.0 & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn dst(self) -> u32 {
        ((self.0 >> 20) & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn code(self) -> u32 {
        (self.0 >> 40) as u32
    }
}

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    mark_byte: u8,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            mark_byte: 0,
        }
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

#[inline(always)]
fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

#[inline(always)]
fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
}

#[cold]
#[inline(never)]
fn grow_u32_vec(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

#[cold]
#[inline(never)]
fn grow_step_u8(v: &mut Vec<u8>, need: usize, step: usize) {
    let target = need.div_ceil(step) * step;
    v.reserve_exact(target - v.len());
}

#[inline(always)]
fn ensure_step_u8(v: &mut Vec<u8>, add: usize, step: usize) {
    let need = v.len() + add;
    if need > v.capacity() {
        grow_step_u8(v, need, step);
    }
}

#[cold]
#[inline(never)]
fn grow_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    let target = (lens.len() + 1).div_ceil(step) * step;
    inline16.reserve_exact(target - inline16.len());
    lens.reserve_exact(target - lens.len());
}

#[inline(always)]
fn ensure_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    if lens.len() == lens.capacity() {
        grow_step_inline(inline16, lens, step);
    }
}

fn begin_inline_default(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    if words > 0 && inline16.capacity() < 256 + words {
        inline16.reserve_exact(256 + words - inline16.len());
        lens.reserve_exact(256 + words - lens.len());
    }
}

fn begin_inline_trim(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.reserve_exact(256);
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.reserve_exact(256);
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
}

#[inline(always)]
fn rd_u32_slot(slot: &[u8; 16], w: usize) -> u32 {
    let off = w * 4;
    u32::from_le_bytes([slot[off], slot[off + 1], slot[off + 2], slot[off + 3]])
}

#[inline(always)]
fn wr_u32_slot(slot: &mut [u8; 16], w: usize, val: u32) {
    let off = w * 4;
    let b = val.to_le_bytes();
    slot[off] = b[0];
    slot[off + 1] = b[1];
    slot[off + 2] = b[2];
    slot[off + 3] = b[3];
}

// ============================================================================
// OutQ16 with Optional Stack-Local `next` Buffer (0 B Persistent Heap on Non-Ovf!)
// ============================================================================

mod outq_local {
    const CHUNK: usize = 512;
    pub(super) const BYTES: usize = 2 * CHUNK;
    const NONE: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;
    pub(super) const INLINE_NEXT: usize = 896; // covers all n <= 917,504 with 0 heap!

    #[derive(Default)]
    pub(super) struct Spill {
        pub(super) next: Vec<u32>,
        pub(super) ovf: Vec<[u8; BYTES]>,
        pub(super) cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub(super) fn scratch_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
    }

    pub(super) struct NextLinks<'a> {
        inline: &'a mut [u32; INLINE_NEXT],
        spill_next: &'a mut Vec<u32>,
        use_inline: bool,
    }

    impl<'a> NextLinks<'a> {
        #[inline(always)]
        pub(super) fn new(
            inline: &'a mut [u32; INLINE_NEXT],
            spill_next: &'a mut Vec<u32>,
            use_inline: bool,
        ) -> Self {
            Self {
                inline,
                spill_next,
                use_inline,
            }
        }

        #[inline(always)]
        fn set(&mut self, id: usize, val: u32) {
            if self.use_inline && id < INLINE_NEXT {
                self.inline[id] = val;
            } else {
                let idx = if self.use_inline { id - INLINE_NEXT } else { id };
                if self.spill_next.len() <= idx {
                    self.spill_next.resize(idx + 1, NONE);
                }
                self.spill_next[idx] = val;
            }
        }
    }

    #[derive(Clone, Copy)]
    pub(super) struct NextView<'a> {
        inline: &'a [u32; INLINE_NEXT],
        spill_next: &'a [u32],
        use_inline: bool,
    }

    impl<'a> NextView<'a> {
        #[inline(always)]
        pub(super) fn new(
            inline: &'a [u32; INLINE_NEXT],
            spill_next: &'a [u32],
            use_inline: bool,
        ) -> Self {
            Self {
                inline,
                spill_next,
                use_inline,
            }
        }

        #[inline(always)]
        fn get(&self, id: usize) -> u32 {
            if self.use_inline && id < INLINE_NEXT {
                self.inline[id]
            } else {
                let idx = if self.use_inline { id - INLINE_NEXT } else { id };
                self.spill_next[idx]
            }
        }
    }

    pub(super) struct Queues<'a> {
        out: &'a mut [u8],
        ovf: &'a mut Vec<[u8; BYTES]>,
        cuts: &'a mut Vec<[u32; 256]>,
        next: NextLinks<'a>,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        pt: [u64; 256],
        limit: u32,
        over: bool,
    }

    pub(super) struct Heads {
        heads: [u32; 256],
        in_out: u32,
        pub(super) chunks: u32,
    }

    impl<'a> Queues<'a> {
        pub(super) fn new(
            out: &'a mut [u8],
            spill: &'a mut Spill,
            inline_next: &'a mut [u32; INLINE_NEXT],
            use_inline_next: bool,
            limit: u32,
        ) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            let next = NextLinks::new(inline_next, &mut spill.next, use_inline_next);
            Queues {
                in_out,
                out,
                ovf: &mut spill.ovf,
                cuts: &mut spill.cuts,
                next,
                chunks: 0,
                heads: [NONE; 256],
                pt: [u32::MAX as u64; 256],
                limit,
                over: false,
            }
        }

        pub(super) fn over(&self) -> bool {
            self.over
        }

        pub(super) fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
                chunks: self.chunks,
            }
        }

        #[inline(always)]
        pub(super) fn thin_of(&self, a: usize) -> u32 {
            (self.pt[a] >> 32) as u32
        }

        pub(super) fn extract_thin(&self) -> [u32; 256] {
            std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
        }

        #[inline(always)]
        pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & 1023) as usize;
            if (w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
                let at = p as usize + f;
                if let Some(dst) = self.out.get_mut(at..at + 32) {
                    let lo = m as u16;
                    let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
                    for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&x.to_le_bytes());
                    }
                    let wu = u64::from(w);
                    self.pt[a] = v + ((wu << 32) | wu);
                    return;
                }
            }
            self.push_slow_pt(a, m, w);
        }

        #[cold]
        #[inline(never)]
        fn push_slow_pt(&mut self, a: usize, mut m: u32, w: u32) {
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                let p = self.pt[a] as u32;
                if p == u32::MAX || (p & 1023) as usize == CHUNK {
                    self.take_chunk_pt(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(p >> 10);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                let tu = u64::from(take);
                self.pt[a] = v + ((tu << 32) | tu);
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                    self.cuts.push(cut);
                }
            }
        }

        fn take_chunk_pt(&mut self, a: usize) {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.ovf.len() <= (id - self.in_out) as usize {
                self.ovf.push([0; BYTES]);
            }
            self.next.set(id as usize, NONE);
            let hi = self.pt[a] & !0xFFFF_FFFFu64;
            if self.heads[a] == NONE {
                self.heads[a] = id;
            } else {
                self.next.set(((self.pt[a] as u32) >> 10) as usize, id);
            }
            self.pt[a] = hi | u64::from(id << 10);
        }

        #[cold]
        #[inline(never)]
        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * BYTES;
                &mut self.out[at..at + BYTES]
            } else {
                &mut self.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    pub(super) struct Reader<'a> {
        out: &'a [u8],
        spill: &'a Spill,
        next: NextView<'a>,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read: u32,
        epoch: u32,
        cut: u32,
    }

    impl<'a> Reader<'a> {
        #[inline(always)]
        pub(super) fn new(
            out: &'a [u8],
            spill: &'a Spill,
            next: NextView<'a>,
            heads: &Heads,
            a: usize,
        ) -> Self {
            Reader {
                out,
                spill,
                next,
                in_out: heads.in_out,
                a,
                id: heads.heads[a],
                at: 0,
                read: 0,
                epoch: 0,
                cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
            }
        }

        #[inline(always)]
        pub(super) fn wire_xor_nomark(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
        ) {
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut i = 0usize;
                while i + 16 <= span {
                    let b = &bytes[2 * i..2 * i + 32];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                    let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let v8 = hi | (q2 as u32 & 0xFFFF);
                    let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                    let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF);
                    let v11 = hi | ((q2 >> 48) as u32);
                    let v12 = hi | (q3 as u32 & 0xFFFF);
                    let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                    let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF);
                    let v15 = hi | ((q3 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 16];
                    eu[0] ^= pack | v0;   eu[1] ^= pack | v1;   eu[2] ^= pack | v2;   eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4;   eu[5] ^= pack | v5;   eu[6] ^= pack | v6;   eu[7] ^= pack | v7;
                    eu[8] ^= pack | v8;   eu[9] ^= pack | v9;   eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                    eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                    edges[v0 as usize] ^= u0 as u32;        edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32;  edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32;  edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32;  edges[v7 as usize] ^= (u0 + 7) as u32;
                    edges[v8 as usize] ^= (u0 + 8) as u32;  edges[v9 as usize] ^= (u0 + 9) as u32;
                    edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                    edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                    edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                    i += 16;
                }
                if i + 8 <= span {
                    let b = &bytes[2 * i..2 * i + 16];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 8];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                    i += 8;
                }
                if i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let v0 = hi | (q as u32 & 0xFFFF);
                    let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 4];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    i += 4;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    i += 1;
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
            }
        }

        #[inline(always)]
        pub(super) fn wire_xor(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_slow(edges, dst_start, span, pack, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.next.get(self.id as usize);
                    self.at = 0;
                }
                let take = left
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)]
                    .chunks_exact(2)
                    .enumerate()
                {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        #[inline(always)]
        pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * BYTES + 2 * self.at;
                for (e, s) in dst
                    .iter_mut()
                    .zip(self.out[at..at + 2 * span].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_slow(dst, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_slow(&mut self, mut dst: &mut [u32], pack: u32) {
            while !dst.is_empty() {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.next.get(self.id as usize);
                    self.at = 0;
                }
                let take = dst
                    .len()
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = pack | (self.epoch << 16);
                let (now, rest) = dst.split_at_mut(take);
                for (e, s) in now
                    .iter_mut()
                    .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += take;
                self.read += take as u32;
                dst = rest;
            }
        }
    }
}

// ============================================================================
// Stack-Local Append Stack (`[u32; 64]` covers 100% of real blocks, 0 B heap!)
// ============================================================================

struct SmallStack<'a> {
    buf: [u32; 64],
    len: usize,
    spill: &'a mut Vec<u32>,
}

impl<'a> SmallStack<'a> {
    #[inline(always)]
    fn new(spill: &'a mut Vec<u32>) -> Self {
        Self {
            buf: [0; 64],
            len: 0,
            spill,
        }
    }
    #[inline(always)]
    fn clear(&mut self) {
        self.len = 0;
        self.spill.clear();
    }
    #[inline(always)]
    fn push(&mut self, v: u32) {
        if self.len < 64 {
            self.buf[self.len] = v;
            self.len += 1;
        } else {
            self.spill.push(v);
        }
    }
    #[inline(always)]
    fn pop(&mut self) -> Option<u32> {
        if let Some(v) = self.spill.pop() {
            return Some(v);
        }
        if self.len > 0 {
            self.len -= 1;
            Some(self.buf[self.len])
        } else {
            None
        }
    }
}

// ============================================================================
// Traditional u32-Arena Links & Elim (for Arms 1, 2, 3 and fallback)
// ============================================================================

trait Links {
    const STRIDE: usize;
    const IS_NARROW: bool;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
}

struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
    const IS_NARROW: bool = true;
    const BOTTOM: usize = 1;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] & 0xFFFF
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] >> 16
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF_0000) | to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const IS_NARROW: bool = false;
    const BOTTOM: usize = 2;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side]
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side + 1]
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side] = to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links> Elim<'_, L> {
    #[inline(always)]
    fn width(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }

    fn assign_bottom(&mut self, rows: u32) {
        let mut first = L::next(self.a, 0, 1);
        let mut last = L::prev(self.a, 0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = L::next(self.a, first, 1);
            let prev = L::prev(self.a, last, 1);
            let width = self.width(first);
            hi -= self.width(last);
            self.set_bottom(first, lo);
            self.set_bottom(last, hi);
            lo += width;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            self.set_bottom(first, lo);
            lo += self.width(first);
        }
        assert_eq!(lo, hi);
    }

    #[inline(always)]
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        self.set_width(id, width);
        self.set_word(id, word);
        id
    }

    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            L::set_next(self.a, *tail, side, id);
        }
        L::set_prev(self.a, id, side, *tail);
        *tail = id;
    }

    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.a, prev, side, head);
            L::set_prev(self.a, head, side, prev);
            prev = tail;
        }
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }

    #[inline(always)]
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }

    #[inline(always)]
    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side);
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }

    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }

    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + id as usize * RULE
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }

    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 3 {
            return u128::from(id);
        }
        let r = self.rule(id);
        let sl = &self.a[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum <= 3 {
                return (sum << ID_BITS)
                    | (left & ID_MASK)
                    | ((right & ID_MASK) << (8 * left_len));
            }
            let id = ((self.top - self.base) / RULE) as u32;
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << ID_BITS) | id
        } else {
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = ((self.top - self.base) / RULE) as u32;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }

    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base);
        }
    }

    #[inline(never)]
    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                continue;
            }
            let ny = &self.a[yi..yi + 4];
            let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                self.a[px as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.a[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.a[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.a[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
            }
        };
        *remaining = left;
        done
    }

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        if !SPIRALS && L::IS_NARROW {
            return self.eliminate_narrow_plain(remaining);
        }
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        *remaining = left;
        done
    }

    fn inline_word<const TRIM: bool>(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut SmallStack<'_>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append::<TRIM>(word, stack, spelling);
            if TRIM {
                ensure_step_u8(spelling, 1, 8192);
            }
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    fn append<const TRIM: bool>(
        &mut self,
        handle: u32,
        stack: &mut SmallStack<'_>,
        dest: &mut Vec<u8>,
    ) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                if TRIM {
                    ensure_step_u8(dest, g as usize, 8192);
                }
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                let rule = &mut self.a[r..r + 4];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = rule[2] as usize;
                    if TRIM {
                        ensure_step_u8(dest, rlen, 8192);
                    }
                    dest.extend_from_within(start..start + rlen);
                }
            }
        }
    }
}

// ============================================================================
// Split-Storage Narrow Elimination: `nodes` in `out_tail` (`&mut [[u8; 16]]`),
// `rules` in EITHER `self.edges[0..]` (`OT-Nodes`, Arm 4) OR `out_tail` (`OT-FullArena`, Arm 5)!
// ============================================================================

struct SplitNodesElim<'a> {
    nodes_buf: &'a mut [[u8; 16]], // lives in out_tail!
    rules: &'a mut [u32],          // lives at self.edges[0..] (base == 0!)
    rules_top: usize,              // word offset in `rules`
    nodes: u32,
}

impl SplitNodesElim<'_> {
    #[inline(always)]
    fn node_w(&self, id: u32, w: usize) -> u32 {
        rd_u32_slot(&self.nodes_buf[id as usize], w)
    }
    #[inline(always)]
    fn set_node_w(&mut self, id: u32, w: usize, val: u32) {
        wr_u32_slot(&mut self.nodes_buf[id as usize], w, val);
    }
    #[inline(always)]
    fn prev(&self, id: u32, side: usize) -> u32 {
        self.node_w(id, side) & 0xFFFF
    }
    #[inline(always)]
    fn next(&self, id: u32, side: usize) -> u32 {
        self.node_w(id, side) >> 16
    }
    #[inline(always)]
    fn set_prev(&mut self, id: u32, side: usize, to: u32) {
        let cur = self.node_w(id, side);
        self.set_node_w(id, side, (cur & 0xFFFF_0000) | to);
    }
    #[inline(always)]
    fn set_next(&mut self, id: u32, side: usize, to: u32) {
        let cur = self.node_w(id, side);
        self.set_node_w(id, side, (cur & 0xFFFF) | (to << 16));
    }
    #[inline(always)]
    fn band_node(&self, id: u32) -> (u32, u32, u32, u32) {
        let sl = &self.nodes_buf[id as usize];
        (
            rd_u32_slot(sl, 0) >> 16,
            rd_u32_slot(sl, 1),
            rd_u32_slot(sl, 2),
            rd_u32_slot(sl, 3),
        )
    }
    #[inline(always)]
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        let sl = &mut self.nodes_buf[id as usize];
        wr_u32_slot(sl, 2, width);
        wr_u32_slot(sl, 3, word);
        id
    }
    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            self.set_next(*tail, side, id);
        }
        self.set_prev(id, side, *tail);
        *tail = id;
    }
    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            self.set_next(prev, side, head);
            self.set_prev(head, side, prev);
            prev = tail;
        }
        self.set_next(prev, side, 0);
        self.set_prev(0, side, prev);
    }
    fn assign_bottom(&mut self, rows: u32) {
        let mut first = self.next(0, 1);
        let mut last = self.prev(0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = self.next(first, 1);
            let prev = self.prev(last, 1);
            let width = self.node_w(first, 2);
            hi -= self.node_w(last, 2);
            self.set_node_w(first, 1, lo);
            self.set_node_w(last, 1, hi);
            lo += width;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            self.set_node_w(first, 1, lo);
            lo += self.node_w(first, 2);
        }
        assert_eq!(lo, hi);
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.rules[(h & ID_MASK) as usize * RULE + 2]
        }
    }
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 3 {
            return u128::from(id);
        }
        let r = id as usize * RULE;
        let sl = &self.rules[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }
    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum <= 3 {
                return (sum << ID_BITS)
                    | (left & ID_MASK)
                    | ((right & ID_MASK) << (8 * left_len));
            }
            let id = (self.rules_top / RULE) as u32;
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            let out = &mut self.rules[self.rules_top..self.rules_top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.rules_top += RULE;
            (sum << ID_BITS) | id
        } else {
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = (self.rules_top / RULE) as u32;
            let out = &mut self.rules[self.rules_top..self.rules_top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.rules_top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }

    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base);
        }
    }

    #[inline(never)]
    fn eliminate_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rules_top + RULE > self.rules.len() {
                break false;
            }
            let sent = self.nodes_buf[0];
            let s0 = rd_u32_slot(&sent, 0);
            let s1 = rd_u32_slot(&sent, 1);
            let x = s0 & 0xFFFF;
            let y = s1 & 0xFFFF;
            let nx = self.nodes_buf[x as usize];
            let (x0, x1, aw, wx) = (
                rd_u32_slot(&nx, 0),
                rd_u32_slot(&nx, 1),
                rd_u32_slot(&nx, 2),
                rd_u32_slot(&nx, 3),
            );
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                let cur0 = rd_u32_slot(&self.nodes_buf[px0 as usize], 0);
                wr_u32_slot(&mut self.nodes_buf[px0 as usize], 0, cur0 & 0xFFFF);
                wr_u32_slot(&mut self.nodes_buf[0], 0, (s0 & 0xFFFF_0000) | px0);
                let cur1 = rd_u32_slot(&self.nodes_buf[px1 as usize], 1);
                wr_u32_slot(&mut self.nodes_buf[px1 as usize], 1, cur1 & 0xFFFF);
                wr_u32_slot(&mut self.nodes_buf[0], 1, (s1 & 0xFFFF_0000) | px1);
                continue;
            }
            let ny = self.nodes_buf[y as usize];
            let (y0, y1, bw, wy) = (
                rd_u32_slot(&ny, 0),
                rd_u32_slot(&ny, 1),
                rd_u32_slot(&ny, 2),
                rd_u32_slot(&ny, 3),
            );
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                let curp = rd_u32_slot(&self.nodes_buf[px as usize], 0);
                wr_u32_slot(&mut self.nodes_buf[px as usize], 0, curp & 0xFFFF);
                let cs0 = rd_u32_slot(&self.nodes_buf[0], 0);
                wr_u32_slot(&mut self.nodes_buf[0], 0, (cs0 & 0xFFFF_0000) | px);
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.nodes_buf[x as usize];
                    wr_u32_slot(nxm, 0, y | (y_next << 16));
                    wr_u32_slot(nxm, 3, word);
                    let nym = &mut self.nodes_buf[y as usize];
                    wr_u32_slot(nym, 0, (y0 & 0xFFFF) | (x << 16));
                    wr_u32_slot(nym, 2, bw - aw);
                    let cur_yn = rd_u32_slot(&self.nodes_buf[y_next as usize], 0);
                    wr_u32_slot(&mut self.nodes_buf[y_next as usize], 0, (cur_yn & 0xFFFF_0000) | x);
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.nodes_buf[x as usize];
                    wr_u32_slot(nxm, 0, y_prev | (y_next << 16));
                    wr_u32_slot(nxm, 3, word);
                    let cur_yp = rd_u32_slot(&self.nodes_buf[y_prev as usize], 0);
                    wr_u32_slot(&mut self.nodes_buf[y_prev as usize], 0, (cur_yp & 0xFFFF) | (x << 16));
                    let cur_yn = rd_u32_slot(&self.nodes_buf[y_next as usize], 0);
                    wr_u32_slot(&mut self.nodes_buf[y_next as usize], 0, (cur_yn & 0xFFFF_0000) | x);
                    let py = y1 & 0xFFFF;
                    let cur_py = rd_u32_slot(&self.nodes_buf[py as usize], 1);
                    wr_u32_slot(&mut self.nodes_buf[py as usize], 1, cur_py & 0xFFFF);
                    let cs1 = rd_u32_slot(&self.nodes_buf[0], 1);
                    wr_u32_slot(&mut self.nodes_buf[0], 1, (cs1 & 0xFFFF_0000) | py);
                }
            } else {
                let py = y1 & 0xFFFF;
                let cur_py = rd_u32_slot(&self.nodes_buf[py as usize], 1);
                wr_u32_slot(&mut self.nodes_buf[py as usize], 1, cur_py & 0xFFFF);
                let cs1 = rd_u32_slot(&self.nodes_buf[0], 1);
                wr_u32_slot(&mut self.nodes_buf[0], 1, (cs1 & 0xFFFF_0000) | py);
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.nodes_buf[y as usize];
                wr_u32_slot(nym, 1, x | (x_next << 16));
                wr_u32_slot(nym, 3, word);
                let nxm = &mut self.nodes_buf[x as usize];
                wr_u32_slot(nxm, 1, (x1 & 0xFFFF) | (y << 16));
                wr_u32_slot(nxm, 2, aw - bw);
                let cur_xn = rd_u32_slot(&self.nodes_buf[x_next as usize], 1);
                wr_u32_slot(&mut self.nodes_buf[x_next as usize], 1, (cur_xn & 0xFFFF_0000) | y);
            }
        };
        *remaining = left;
        done
    }

    #[cold]
    #[inline(never)]
    fn eliminate_spirals(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let room = SPIRAL_RULES * RULE;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.rules_top + room > self.rules.len() {
                break false;
            }
            let (x, y) = (self.prev(0, 0), self.prev(0, 1));
            let (aw, bw) = (self.node_w(x, 2), self.node_w(y, 2));
            if x != y {
                if aw > bw && self.prev(y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.node_w(x, 3), q);
                    let word = self.join(head, self.node_w(y, 3));
                    self.set_node_w(y, 3, word);
                    self.set_node_w(x, 2, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && self.prev(x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.node_w(y, 3), q);
                    let word = self.join(self.node_w(x, 3), tail);
                    self.set_node_w(x, 3, word);
                    self.set_node_w(y, 2, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            if x == y {
                let p0 = self.prev(x, 0);
                self.set_next(p0, 0, 0);
                self.set_prev(0, 0, p0);
                let p1 = self.prev(x, 1);
                self.set_next(p1, 1, 0);
                self.set_prev(0, 1, p1);
                continue;
            }
            let word = self.join(self.node_w(x, 3), self.node_w(y, 3));
            if aw <= bw {
                let p0 = self.prev(x, 0);
                self.set_next(p0, 0, 0);
                self.set_prev(0, 0, p0);
                let yn = self.next(y, 0);
                self.set_prev(x, 0, y);
                self.set_next(x, 0, yn);
                self.set_next(y, 0, x);
                self.set_prev(yn, 0, x);
                self.set_node_w(x, 3, word);
                if aw == bw {
                    let yp = self.prev(y, 0);
                    let yn2 = self.next(y, 0);
                    self.set_next(yp, 0, yn2);
                    self.set_prev(yn2, 0, yp);
                    let py1 = self.prev(y, 1);
                    self.set_next(py1, 1, 0);
                    self.set_prev(0, 1, py1);
                } else {
                    self.set_node_w(y, 2, bw - w);
                }
            } else {
                let py1 = self.prev(y, 1);
                self.set_next(py1, 1, 0);
                self.set_prev(0, 1, py1);
                let xn = self.next(x, 1);
                self.set_prev(y, 1, x);
                self.set_next(y, 1, xn);
                self.set_next(x, 1, y);
                self.set_prev(xn, 1, y);
                self.set_node_w(y, 3, word);
                self.set_node_w(x, 2, aw - w);
            }
        };
        *remaining = left;
        done
    }

    fn append(&mut self, handle: u32, stack: &mut SmallStack<'_>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                ensure_step_u8(dest, g as usize, 8192);
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = (h & ID_MASK) as usize * RULE;
                let tag = self.rules[r + 3];
                let rule = &mut self.rules[r..r + 4];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = rule[2] as usize;
                    ensure_step_u8(dest, rlen, 8192);
                    dest.extend_from_within(start..start + rlen);
                }
            }
        }
    }
}

// ============================================================================
// Full-OutTail Narrow Elimination (`nodes` AND `rules` BOTH in `out_tail` `&mut [[u8; 16]]`!)
// ============================================================================

struct FullOutTailElim<'a> {
    slots: &'a mut [[u8; 16]], // [0..nodes] = nodes, [base..top] = rules — 100% in out_tail!
    base: usize,               // in 16-byte slots (= nodes)
    top: usize,                // in 16-byte slots
    nodes: u32,
}

impl FullOutTailElim<'_> {
    #[inline(always)]
    fn w(&self, slot: usize, k: usize) -> u32 {
        rd_u32_slot(&self.slots[slot], k)
    }
    #[inline(always)]
    fn set_w(&mut self, slot: usize, k: usize, val: u32) {
        wr_u32_slot(&mut self.slots[slot], k, val);
    }
    #[inline(always)]
    fn prev(&self, id: u32, side: usize) -> u32 {
        self.w(id as usize, side) & 0xFFFF
    }
    #[inline(always)]
    fn next(&self, id: u32, side: usize) -> u32 {
        self.w(id as usize, side) >> 16
    }
    #[inline(always)]
    fn set_prev(&mut self, id: u32, side: usize, to: u32) {
        let cur = self.w(id as usize, side);
        self.set_w(id as usize, side, (cur & 0xFFFF_0000) | to);
    }
    #[inline(always)]
    fn set_next(&mut self, id: u32, side: usize, to: u32) {
        let cur = self.w(id as usize, side);
        self.set_w(id as usize, side, (cur & 0xFFFF) | (to << 16));
    }
    #[inline(always)]
    fn band_node(&self, id: u32) -> (u32, u32, u32, u32) {
        let sl = &self.slots[id as usize];
        (
            rd_u32_slot(sl, 0) >> 16,
            rd_u32_slot(sl, 1),
            rd_u32_slot(sl, 2),
            rd_u32_slot(sl, 3),
        )
    }
    #[inline(always)]
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        let sl = &mut self.slots[id as usize];
        wr_u32_slot(sl, 2, width);
        wr_u32_slot(sl, 3, word);
        id
    }
    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            self.set_next(*tail, side, id);
        }
        self.set_prev(id, side, *tail);
        *tail = id;
    }
    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            self.set_next(prev, side, head);
            self.set_prev(head, side, prev);
            prev = tail;
        }
        self.set_next(prev, side, 0);
        self.set_prev(0, side, prev);
    }
    fn assign_bottom(&mut self, rows: u32) {
        let mut first = self.next(0, 1);
        let mut last = self.prev(0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = self.next(first, 1);
            let prev = self.prev(last, 1);
            let width = self.w(first as usize, 2);
            hi -= self.w(last as usize, 2);
            self.set_w(first as usize, 1, lo);
            self.set_w(last as usize, 1, hi);
            lo += width;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            self.set_w(first as usize, 1, lo);
            lo += self.w(first as usize, 2);
        }
        assert_eq!(lo, hi);
    }
    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.w(self.base + (h & ID_MASK) as usize, 2)
        }
    }
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 3 {
            return u128::from(id);
        }
        let raw = u128::from_le_bytes(self.slots[self.base + id as usize]);
        raw & ((1u128 << 120) - 1)
    }
    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum <= 3 {
                return (sum << ID_BITS)
                    | (left & ID_MASK)
                    | ((right & ID_MASK) << (8 * left_len));
            }
            let id = (self.top - self.base) as u32;
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            self.slots[self.top] = bytes.to_le_bytes();
            self.top += 1;
            (sum << ID_BITS) | id
        } else {
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = (self.top - self.base) as u32;
            let sl = &mut self.slots[self.top];
            wr_u32_slot(sl, 0, left);
            wr_u32_slot(sl, 1, right);
            wr_u32_slot(sl, 2, len);
            wr_u32_slot(sl, 3, u32::MAX);
            self.top += 1;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }
    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base);
        }
    }

    #[inline(never)]
    fn eliminate_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let cap = self.slots.len();
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top >= cap {
                break false;
            }
            let sent = self.slots[0];
            let s0 = rd_u32_slot(&sent, 0);
            let s1 = rd_u32_slot(&sent, 1);
            let x = s0 & 0xFFFF;
            let y = s1 & 0xFFFF;
            let nx = self.slots[x as usize];
            let (x0, x1, aw, wx) = (
                rd_u32_slot(&nx, 0),
                rd_u32_slot(&nx, 1),
                rd_u32_slot(&nx, 2),
                rd_u32_slot(&nx, 3),
            );
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                let cur0 = rd_u32_slot(&self.slots[px0 as usize], 0);
                wr_u32_slot(&mut self.slots[px0 as usize], 0, cur0 & 0xFFFF);
                wr_u32_slot(&mut self.slots[0], 0, (s0 & 0xFFFF_0000) | px0);
                let cur1 = rd_u32_slot(&self.slots[px1 as usize], 1);
                wr_u32_slot(&mut self.slots[px1 as usize], 1, cur1 & 0xFFFF);
                wr_u32_slot(&mut self.slots[0], 1, (s1 & 0xFFFF_0000) | px1);
                continue;
            }
            let ny = self.slots[y as usize];
            let (y0, y1, bw, wy) = (
                rd_u32_slot(&ny, 0),
                rd_u32_slot(&ny, 1),
                rd_u32_slot(&ny, 2),
                rd_u32_slot(&ny, 3),
            );
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                let curp = rd_u32_slot(&self.slots[px as usize], 0);
                wr_u32_slot(&mut self.slots[px as usize], 0, curp & 0xFFFF);
                let cs0 = rd_u32_slot(&self.slots[0], 0);
                wr_u32_slot(&mut self.slots[0], 0, (cs0 & 0xFFFF_0000) | px);
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.slots[x as usize];
                    wr_u32_slot(nxm, 0, y | (y_next << 16));
                    wr_u32_slot(nxm, 3, word);
                    let nym = &mut self.slots[y as usize];
                    wr_u32_slot(nym, 0, (y0 & 0xFFFF) | (x << 16));
                    wr_u32_slot(nym, 2, bw - aw);
                    let cur_yn = rd_u32_slot(&self.slots[y_next as usize], 0);
                    wr_u32_slot(&mut self.slots[y_next as usize], 0, (cur_yn & 0xFFFF_0000) | x);
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.slots[x as usize];
                    wr_u32_slot(nxm, 0, y_prev | (y_next << 16));
                    wr_u32_slot(nxm, 3, word);
                    let cur_yp = rd_u32_slot(&self.slots[y_prev as usize], 0);
                    wr_u32_slot(&mut self.slots[y_prev as usize], 0, (cur_yp & 0xFFFF) | (x << 16));
                    let cur_yn = rd_u32_slot(&self.slots[y_next as usize], 0);
                    wr_u32_slot(&mut self.slots[y_next as usize], 0, (cur_yn & 0xFFFF_0000) | x);
                    let py = y1 & 0xFFFF;
                    let cur_py = rd_u32_slot(&self.slots[py as usize], 1);
                    wr_u32_slot(&mut self.slots[py as usize], 1, cur_py & 0xFFFF);
                    let cs1 = rd_u32_slot(&self.slots[0], 1);
                    wr_u32_slot(&mut self.slots[0], 1, (cs1 & 0xFFFF_0000) | py);
                }
            } else {
                let py = y1 & 0xFFFF;
                let cur_py = rd_u32_slot(&self.slots[py as usize], 1);
                wr_u32_slot(&mut self.slots[py as usize], 1, cur_py & 0xFFFF);
                let cs1 = rd_u32_slot(&self.slots[0], 1);
                wr_u32_slot(&mut self.slots[0], 1, (cs1 & 0xFFFF_0000) | py);
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.slots[y as usize];
                wr_u32_slot(nym, 1, x | (x_next << 16));
                wr_u32_slot(nym, 3, word);
                let nxm = &mut self.slots[x as usize];
                wr_u32_slot(nxm, 1, (x1 & 0xFFFF) | (y << 16));
                wr_u32_slot(nxm, 2, aw - bw);
                let cur_xn = rd_u32_slot(&self.slots[x_next as usize], 1);
                wr_u32_slot(&mut self.slots[x_next as usize], 1, (cur_xn & 0xFFFF_0000) | y);
            }
        };
        *remaining = left;
        done
    }

    #[cold]
    #[inline(never)]
    fn eliminate_spirals(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let cap = self.slots.len();
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + SPIRAL_RULES > cap {
                break false;
            }
            let (x, y) = (self.prev(0, 0), self.prev(0, 1));
            let (aw, bw) = (self.w(x as usize, 2), self.w(y as usize, 2));
            if x != y {
                if aw > bw && self.prev(y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.w(x as usize, 3), q);
                    let word = self.join(head, self.w(y as usize, 3));
                    self.set_w(y as usize, 3, word);
                    self.set_w(x as usize, 2, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && self.prev(x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.w(y as usize, 3), q);
                    let word = self.join(self.w(x as usize, 3), tail);
                    self.set_w(x as usize, 3, word);
                    self.set_w(y as usize, 2, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            if x == y {
                let p0 = self.prev(x, 0);
                self.set_next(p0, 0, 0);
                self.set_prev(0, 0, p0);
                let p1 = self.prev(x, 1);
                self.set_next(p1, 1, 0);
                self.set_prev(0, 1, p1);
                continue;
            }
            let word = self.join(self.w(x as usize, 3), self.w(y as usize, 3));
            if aw <= bw {
                let p0 = self.prev(x, 0);
                self.set_next(p0, 0, 0);
                self.set_prev(0, 0, p0);
                let yn = self.next(y, 0);
                self.set_prev(x, 0, y);
                self.set_next(x, 0, yn);
                self.set_next(y, 0, x);
                self.set_prev(yn, 0, x);
                self.set_w(x as usize, 3, word);
                if aw == bw {
                    let yp = self.prev(y, 0);
                    let yn2 = self.next(y, 0);
                    self.set_next(yp, 0, yn2);
                    self.set_prev(yn2, 0, yp);
                    let py1 = self.prev(y, 1);
                    self.set_next(py1, 1, 0);
                    self.set_prev(0, 1, py1);
                } else {
                    self.set_w(y as usize, 2, bw - w);
                }
            } else {
                let py1 = self.prev(y, 1);
                self.set_next(py1, 1, 0);
                self.set_prev(0, 1, py1);
                let xn = self.next(x, 1);
                self.set_prev(y, 1, x);
                self.set_next(y, 1, xn);
                self.set_next(x, 1, y);
                self.set_prev(xn, 1, y);
                self.set_w(y as usize, 3, word);
                self.set_w(x as usize, 2, aw - w);
            }
        };
        *remaining = left;
        done
    }

    fn append(&mut self, handle: u32, stack: &mut SmallStack<'_>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                ensure_step_u8(dest, g as usize, 8192);
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.base + (h & ID_MASK) as usize;
                let tag = rd_u32_slot(&self.slots[r], 3);
                if tag == u32::MAX {
                    let c0 = rd_u32_slot(&self.slots[r], 0);
                    let c1 = rd_u32_slot(&self.slots[r], 1);
                    wr_u32_slot(&mut self.slots[r], 3, 0x8000_0000 | dest.len() as u32);
                    stack.push(c1);
                    stack.push(c0);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = rd_u32_slot(&self.slots[r], 2) as usize;
                    ensure_step_u8(dest, rlen, 8192);
                    dest.extend_from_within(start..start + rlen);
                }
            }
        }
    }
}

// ============================================================================
// Dictionary (Generic Over Rule Source)
// ============================================================================

trait RuleSource {
    fn short_payload(&self, handle: u32) -> u128;
    fn is_memoized(&self, id: u32) -> bool;
    fn append_rule<const TRIM: bool>(
        &mut self,
        handle: u32,
        stack: &mut SmallStack<'_>,
        dest: &mut Vec<u8>,
    );
}

impl<L: Links> RuleSource for Elim<'_, L> {
    #[inline(always)]
    fn short_payload(&self, handle: u32) -> u128 {
        self.short_payload_h(handle)
    }
    #[inline(always)]
    fn is_memoized(&self, id: u32) -> bool {
        self.a[self.rule(id) + 3] != u32::MAX
    }
    #[inline(always)]
    fn append_rule<const TRIM: bool>(
        &mut self,
        handle: u32,
        stack: &mut SmallStack<'_>,
        dest: &mut Vec<u8>,
    ) {
        self.append::<TRIM>(handle, stack, dest);
    }
}

impl RuleSource for SplitNodesElim<'_> {
    #[inline(always)]
    fn short_payload(&self, handle: u32) -> u128 {
        self.short_payload_h(handle)
    }
    #[inline(always)]
    fn is_memoized(&self, id: u32) -> bool {
        self.rules[id as usize * RULE + 3] != u32::MAX
    }
    #[inline(always)]
    fn append_rule<const TRIM: bool>(
        &mut self,
        handle: u32,
        stack: &mut SmallStack<'_>,
        dest: &mut Vec<u8>,
    ) {
        self.append(handle, stack, dest);
    }
}

impl RuleSource for FullOutTailElim<'_> {
    #[inline(always)]
    fn short_payload(&self, handle: u32) -> u128 {
        self.short_payload_h(handle)
    }
    #[inline(always)]
    fn is_memoized(&self, id: u32) -> bool {
        rd_u32_slot(&self.slots[self.base + id as usize], 3) != u32::MAX
    }
    #[inline(always)]
    fn append_rule<const TRIM: bool>(
        &mut self,
        handle: u32,
        stack: &mut SmallStack<'_>,
        dest: &mut Vec<u8>,
    ) {
        self.append(handle, stack, dest);
    }
}

fn spelling_hash_long(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8)
            .rotate_left(29)
            .wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

struct Dictionary<'a, const TRIM: bool> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut SmallStack<'a>,
}

impl<const TRIM: bool> Dictionary<'_, TRIM> {
    #[inline(always)]
    fn code<R: RuleSource>(&mut self, e: &mut R, handle: u32, byte: u8) -> u32 {
        let bucket = ((handle.wrapping_mul(0x9e37_79b9)
            ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b)))
            >> 24) as usize;
        let slot = bucket * 2;
        let val = self.pairs[slot + 1];
        if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) {
            return val & 0x00FF_FFFF;
        }
        let code = self.code_miss(e, handle, handle & ID_MASK, byte);
        self.pairs[slot] = handle;
        self.pairs[slot + 1] = (u32::from(byte) << 24) | code;
        code
    }

    fn code_miss<R: RuleSource>(&mut self, e: &mut R, handle: u32, id: u32, byte: u8) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, len);
            self.contents[index] = tag | c;
            return c;
        }
        let start = self.spelling.len();
        let memoized = e.is_memoized(id);
        e.append_rule::<TRIM>(handle, self.stack, self.spelling);
        if TRIM {
            ensure_step_u8(self.spelling, 1, 8192);
        }
        self.spelling.push(byte);
        let entry = long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if memoized {
                self.spelling.truncate(start);
            } else {
                self.spelling.pop();
            }
            return ent & 0x00FF_FFFF;
        }
        let c = self.insert(entry, LONG);
        self.contents[index] = tag | c;
        c
    }

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        if TRIM {
            ensure_step_inline(self.inline16, self.lens, 256);
        }
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

// ============================================================================
// Unified Benchmarkable Kernel Across Arms 1..6
// ============================================================================

const ARM_I3_BASE: u8 = 1;     // Cycle 47 F-Inline3 (persistent bands, default reserves)
const ARM_OT64_C48: u8 = 2;    // Cycle 48 I3+OT64 reproduction
const ARM_OT64_CLEAN: u8 = 3;  // Fixed instr: clean S1 + as_chunks::<8> + stack next/stack + cold TrimWS
const ARM_OT_NODES: u8 = 4;    // open_nodes + image_order + bands in out_tail, rules in edges[0..]
const ARM_OT_FULL: u8 = 5;     // open_nodes + rules + image_order + bands ALL in out_tail (seamless fallback)
const ARM_OT_FULL_TINY: u8 = 6;// Arm 5 + zero-WS PlainRunWalk reusing self.edges for n < 32_768

#[derive(Default, Clone, Copy, Debug)]
struct StageTimes {
    s1_us: f64,
    s2a_us: f64,
    s2b_us: f64,
    s3a_us: f64,
    s4_us: f64,
    s5_us: f64,
}

impl StageTimes {
    fn ind_us(&self) -> f64 {
        self.s2a_us + self.s2b_us + self.s3a_us
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct Telemetry {
    used_ot_bands: bool,
    used_ot_nodes: bool,
    used_ot_full: bool,
    spilled_rules: bool,
    m: usize,
    ns: usize,
    open_nodes: usize,
    rules_count: usize,
    bands_count: usize,
    out_tail_free: usize,
    edges_touched_bytes: usize,
    non_out_touched_bytes: usize,
}

struct Pass<'a> {
    queues: outq_local::Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        if width <= LIMIT {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        let end = pos + width;
        if pos < p {
            self.piece(c, pos, p - pos, false, p);
        }
        self.piece(c, p, 1, true, p);
        if p + 1 < end {
            self.piece(c, p + 1, end - p - 1, false, p);
        }
    }

    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
            }
            self.queues.push_thin(a, f.m, w);
            f.m += w;
        } else {
            if self.ns < self.cap_strips {
                let at = 3 * self.ns;
                if self.edges.len() < at + 3 {
                    let need = (at + 3)
                        .max(2 * self.edges.len())
                        .max(1024)
                        .min(self.cap_strips.saturating_mul(3));
                    self.edges.reserve_exact(need - self.edges.len());
                    self.edges.resize(need, 0);
                }
                let thin_a = self.queues.thin_of(a);
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    thin_a + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
            }
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

#[derive(Default)]
struct DeepKernel {
    spill: outq_local::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    predecessor: Option<u32>,
    // Test overrides (all false by default)
    force_wide: bool,
    force_spirals: bool,
    force_no_outtail: bool,
    force_small_ot_rules: Option<usize>, // forces OT-Full rules spill during S2b to verify exact migration!
}

impl DeepKernel {
    fn ws_breakdown(&self) -> (usize, usize, usize, usize, usize, usize, usize) {
        let edges_b = 4 * self.edges.capacity();
        let bands_b = 8 * self.bands.capacity();
        let spelling_b = self.spelling.capacity();
        let inline_b = 16 * self.inline16.capacity() + self.lens.capacity();
        let spill_b = self.spill.scratch_bytes();
        let other_b = 4 * self.extra.capacity() + 4 * self.stack.capacity();
        let total = edges_b + bands_b + spelling_b + inline_b + spill_b + other_b;
        (total, edges_b, bands_b, spelling_b, inline_b, spill_b, other_b)
    }

    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn arena_words(&self, ns: usize, m: usize) -> usize {
        let most = Self::nodes_most(ns);
        let stride = if most > NARROW_NODES || self.force_wide {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * most).max(m)
    }

    /// Zero-WS `PlainRunWalk` forward-`FL` reusing `self.edges` (`+0 B` persistent WS!).
    #[inline(never)]
    fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n {
            return false;
        }
        if records.len() == 1 {
            if usize::from(records[0] & 255) != n {
                return false;
            }
            let b = (records[0] >> 8) as u8;
            out.clear();
            out.resize(n, b);
            return true;
        }
        let mut counts = [0u32; 256];
        for &rec in records {
            counts[(rec >> 8) as usize] += u32::from(rec & 255);
        }
        let mut st = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            st[c] = tot;
            tot += counts[c];
        }
        if tot as usize != n {
            return false;
        }
        if self.edges.len() < n {
            let add = n - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(n, 0);
        }
        let fl = &mut self.edges[..n];
        let mut pos = 0usize;
        for &rec in records {
            let b = (rec >> 8) as usize;
            let len = (rec & 255) as usize;
            let s0 = st[b] as usize;
            st[b] += len as u32;
            let mut i = pos as u32;
            let bu = b as u32;
            for slot in &mut fl[s0..s0 + len] {
                *slot = (i << 8) | bu;
                i += 1;
            }
            pos += len;
        }
        out.resize(n, 0);
        let mut x = p as usize;
        for slot in &mut out[..n] {
            let v = fl[x];
            *slot = v as u8;
            x = (v >> 8) as usize;
        }
        true
    }

    fn run_arm<const ARM: u8, const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (bool, StageTimes, Telemetry) {
        let mut tm = StageTimes::default();
        let mut tel = Telemetry::default();

        if ARM == ARM_OT_FULL_TINY && n < 32_768 && !self.force_wide && !self.force_spirals {
            let t0 = if TIMED { Some(Instant::now()) } else { None };
            let ok = self.invert_tiny_fl(records, n, p, out);
            if let Some(t) = t0 {
                tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }
            tel.edges_touched_bytes = 4 * n;
            tel.non_out_touched_bytes = 4 * n;
            return (ok, tm, tel);
        }

        let use_inline_next = ARM >= ARM_OT64_CLEAN;
        let use_clean_s1 = ARM != ARM_OT64_C48;
        let use_trim = ARM >= ARM_OT64_C48;

        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        let nrec = records.len();
        // Clean S1 upfront overflow guard: when nrec <= n <= 900_000, sum(rec & 255) <= 900_000 * 255
        // = 229.5M << u32::MAX, so `pos += width` can NEVER overflow! Malformed `nrec > n` falls back cleanly.
        if use_clean_s1 && nrec > n {
            return (false, tm, tel);
        }

        out.resize(n + INLINE, 0);
        let mut inline_next = [u32::MAX; outq_local::INLINE_NEXT];
        let (ns, f, heads, over, tot_pos) = {
            let mut pass = Pass {
                queues: outq_local::Queues::new(
                    &mut out[..],
                    &mut self.spill,
                    &mut inline_next,
                    use_inline_next,
                    u32::MAX,
                ),
                edges: &mut self.edges,
                ns: 0,
                cap_strips: usize::MAX,
                f: First::default(),
            };
            let mut pos = 0u32;
            let mut i = 0usize;
            if use_clean_s1 {
                while i < nrec {
                    let rec = records[i];
                    let c = (rec >> 8) as u8;
                    let mut width = u32::from(rec & 255);
                    i += 1;
                    if width <= LIMIT {
                        pass.piece(c, pos, width, true, p);
                        pos += width;
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c {
                                width += u32::from(records[i] & 255);
                                i += 1;
                            }
                        }
                        pass.take(c, pos, width, p);
                        pos += width;
                    }
                }
            } else {
                // Exact Cycle 48 I3+OT64 S1 loop (with per-record width==0 and checked_add)
                while i < nrec {
                    let rec = records[i];
                    let c = (rec >> 8) as u8;
                    let mut width = u32::from(rec & 255);
                    i += 1;
                    if width == 0 {
                        continue;
                    }
                    if width <= LIMIT {
                        if let Some(next_pos) = pos.checked_add(width) {
                            pass.piece(c, pos, width, true, p);
                            pos = next_pos;
                        } else {
                            return (false, tm, tel);
                        }
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c {
                                width = width.saturating_add(u32::from(records[i] & 255));
                                i += 1;
                            }
                        }
                        if let Some(next_pos) = pos.checked_add(width) {
                            pass.take(c, pos, width, p);
                            pos = next_pos;
                        } else {
                            return (false, tm, tel);
                        }
                    }
                }
            }
            let over = pass.queues.over();
            let thin = pass.queues.extract_thin();
            let heads = pass.queues.heads();
            let mut f = pass.f;
            f.thin = thin;
            (pass.ns, f, heads, over, pos)
        };
        if let Some(t) = t_s1 {
            tm.s1_us = t.elapsed().as_secs_f64() * 1e6;
        }
        if over || tot_pos as usize != n || f.m == 0 {
            return (false, tm, tel);
        }
        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        if all.iter().sum::<u32>() != n as u32 {
            return (false, tm, tel);
        }

        let first = starts(all);
        let mut write = starts(f.thick);
        let m = f.m as usize;
        let in_out_chunks = out.len() / outq_local::BYTES;
        let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq_local::BYTES;
        let out_tail_free = out.len() - tail_offset;
        let most = Self::nodes_most(ns);
        let is_narrow = most <= NARROW_NODES && !self.force_wide;

        tel.m = m;
        tel.ns = ns;
        tel.out_tail_free = out_tail_free;

        self.bands.clear();
        self.spelling.clear();
        if !use_trim && n >= 65_536 {
            if self.bands.capacity() < 8192 {
                self.bands.reserve_exact(8192);
            }
            if self.spelling.capacity() < 48 * 1024 {
                self.spelling.reserve_exact(48 * 1024);
            }
        }

        // Dispatch S2a + S2b + S3a
        let (cursors, used_tail_bands, tail_band_start, tail_band_count) =
            if !self.force_no_outtail && is_narrow && (ARM == ARM_OT_FULL || ARM == ARM_OT_FULL_TINY) {
                self.induced_outtail_full::<TIMED>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    &mut out[tail_offset..],
                    &mut tm,
                    &mut tel,
                )
            } else if !self.force_no_outtail && is_narrow && ARM == ARM_OT_NODES {
                self.induced_outtail_nodes::<TIMED>(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    &mut out[tail_offset..],
                    &mut tm,
                    &mut tel,
                )
            } else {
                let allow_tail_bands = !self.force_no_outtail && ARM >= ARM_OT64_C48;
                let use_c48_band_encode = ARM == ARM_OT64_C48;
                let need = self.arena_words(ns, m);
                if self.edges.len() < need {
                    let add = need - self.edges.len();
                    self.edges.reserve_exact(add);
                    self.edges.resize(need, 0);
                }
                if is_narrow {
                    if use_trim {
                        self.induced_std::<Narrow, true, TIMED>(
                            ns,
                            n as u32,
                            f.m,
                            &f.thin,
                            &first,
                            &mut write,
                            most,
                            &mut out[tail_offset..],
                            allow_tail_bands,
                            use_c48_band_encode,
                            &mut tm,
                            &mut tel,
                        )
                    } else {
                        self.induced_std::<Narrow, false, TIMED>(
                            ns,
                            n as u32,
                            f.m,
                            &f.thin,
                            &first,
                            &mut write,
                            most,
                            &mut out[tail_offset..],
                            allow_tail_bands,
                            use_c48_band_encode,
                            &mut tm,
                            &mut tel,
                        )
                    }
                } else {
                    self.induced_std_wide::<TIMED>(
                        ns,
                        n as u32,
                        f.m,
                        &f.thin,
                        &first,
                        &mut write,
                        most,
                        &mut out[tail_offset..],
                        allow_tail_bands,
                        use_trim,
                        &mut tm,
                        &mut tel,
                    )
                }
            };

        // S4: Wiring
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            let add = m - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(m, 0);
        }
        tel.edges_touched_bytes = tel.edges_touched_bytes.max(4 * m);
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        self.predecessor = None;
        let use_xor = !wide && f.m >= 65_536;
        if use_xor {
            self.edges[..m].fill(0);
        }
        let mut predecessor = f.mark;
        let mb = usize::from(f.mark_byte);
        let (out_q, out_tail) = out.split_at(tail_offset);

        if used_tail_bands {
            if ARM == ARM_OT64_C48 {
                self.wire_outtail_c48(
                    &cursors,
                    tail_band_count,
                    &out_tail[tail_band_start..],
                    &heads,
                    out_q,
                    &inline_next,
                    use_inline_next,
                    m,
                    bits,
                    wide,
                    use_xor,
                    f.mark,
                    mb,
                    &mut predecessor,
                );
            } else {
                self.wire_outtail_clean(
                    &cursors,
                    tail_band_count,
                    &out_tail[tail_band_start..],
                    &heads,
                    out_q,
                    &inline_next,
                    use_inline_next,
                    m,
                    bits,
                    wide,
                    use_xor,
                    f.mark,
                    mb,
                    &mut predecessor,
                );
            }
        } else {
            self.wire_from_bands_vec(
                &cursors,
                &heads,
                out_q,
                &inline_next,
                use_inline_next,
                m,
                bits,
                wide,
                use_xor,
                f.mark,
                mb,
                &mut predecessor,
            );
        }
        if use_xor {
            self.predecessor = Some(predecessor);
        }
        if let Some(t) = t_s4 {
            tm.s4_us = t.elapsed().as_secs_f64() * 1e6;
        }

        // Compute distinct non-`out` bytes touched during this block!
        let bands_non_out = if used_tail_bands { 0 } else { 8 * tel.bands_count };
        let dict_touched = self.spelling.len() + 17 * self.lens.len();
        let spill_touched = if use_inline_next {
            self.spill.ovf.len() * outq_local::BYTES + self.spill.cuts.len() * 1024
        } else {
            4 * (heads.chunks as usize)
                + self.spill.ovf.len() * outq_local::BYTES
                + self.spill.cuts.len() * 1024
        };
        tel.non_out_touched_bytes =
            tel.edges_touched_bytes + bands_non_out + dict_touched + spill_touched + 4 * self.extra.len();

        // S5: Emit
        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        if let Some(before) = self.predecessor {
            emit_bidirectional(
                &self.edges[..m],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                before,
                bits,
                out,
            );
        } else {
            emit(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
        }
        if let Some(t) = t_s5 {
            tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
        }
        (true, tm, tel)
    }

    // ------------------------------------------------------------------------
    // Standard u32-Arena Induced (`Arms 1, 2, 3` & `Wide` / tiny-tail fallback)
    // ------------------------------------------------------------------------
    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn induced_std_wide<const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        allow_tail_bands: bool,
        use_trim: bool,
        tm: &mut StageTimes,
        tel: &mut Telemetry,
    ) -> ([usize; 256], bool, usize, usize) {
        if use_trim {
            self.induced_std::<Wide, true, TIMED>(
                ns,
                n,
                m,
                thin,
                first,
                write,
                most,
                out_tail,
                allow_tail_bands,
                false,
                tm,
                tel,
            )
        } else {
            self.induced_std::<Wide, false, TIMED>(
                ns,
                n,
                m,
                thin,
                first,
                write,
                most,
                out_tail,
                allow_tail_bands,
                false,
                tm,
                tel,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_std<L: Links, const TRIM: bool, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        allow_tail_bands: bool,
        use_c48_band_encode: bool,
        tm: &mut StageTimes,
        tel: &mut Telemetry,
    ) -> ([usize; 256], bool, usize, usize) {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let nodes_words = L::STRIDE * most;
        assert!(self.edges.len() >= 4 * ns + nodes_words);

        let mut widest = 0u32;
        let (base, mut top, nodes, open_nodes) = {
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let mut e = Elim::<L> {
                a: arena,
                base: 0,
                top: 0,
                nodes: 0,
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0u32;
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let from = sw[0];
                let wbyte = sw[2];
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                sw[1] += first[a];
                let dst_idx = write[a] as usize;
                image_order[dst_idx] = i as u32;
                write[a] = dst_idx as u32 + 1;
                let sid = i as u32 + 1;
                e.set_width(sid, width);
                e.set_word(sid, literal(byte));
                widest = widest.max(width);
                if x < from {
                    let gw = from - x;
                    widest = widest.max(gw);
                    let gid = e.node(gw, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                }
                e.append_to(&mut order[0], sid, 0);
                x = from + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for &i in image_order.iter() {
                let idx = i as usize;
                let sw = &strips_raw[3 * idx..3 * idx + 3];
                let to = sw[1];
                let width = sw[2] >> 8;
                if x < to {
                    widest = widest.max(to - x);
                    let id = e.node(to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = to + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            let open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, base, e.nodes, open_nodes)
        };
        if let Some(t) = t_s2a {
            tm.s2a_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = widest > CERTIFIED_WIDTH || self.force_spirals;
        let mut remaining = n;
        loop {
            let mut e = Elim::<L> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            };
            top = e.top;
            if done {
                break;
            }
            grow_u32_vec(&mut self.edges, 4 * ns + top + SPIRAL_RULES * RULE);
        }
        if let Some(t) = t_s2b {
            tm.s2b_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        tel.open_nodes = open_nodes;
        tel.rules_count = (top - base) / RULE;
        tel.edges_touched_bytes = 4 * (4 * ns + top);
        {
            let mut e = Elim::<L> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            e.assign_bottom(m);
        }
        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail_arena) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail_arena
        };
        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        if TRIM {
            begin_inline_trim(&mut self.inline16, &mut self.lens);
        } else {
            begin_inline_default(if n >= 65_536 { 1536 } else { 0 }, &mut self.inline16, &mut self.lens);
        }

        let max_bands = open_nodes + 256;
        let use_tail = allow_tail_bands && out_tail.len() >= max_bands * 8;
        tel.used_ot_bands = use_tail;
        let mut cursors = [0usize; 256];
        let mut nbands = 0usize;
        let mut small_stack = SmallStack::new(&mut self.stack);

        if cache.len() < CACHE_WORDS {
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            let (band_slots, _) = out_tail.as_chunks_mut::<8>();
            for a in 0..256 {
                cursors[a] = if use_tail {
                    if use_c48_band_encode { nbands * 8 } else { nbands }
                } else {
                    self.bands.len()
                };
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let (entry, len) =
                            e.inline_word::<TRIM>(node_word, a as u8, &mut small_stack, &mut self.spelling);
                        if TRIM {
                            ensure_step_inline(&mut self.inline16, &mut self.lens, 256);
                        }
                        let c = self.lens.len() as u32;
                        self.inline16.push(entry);
                        self.lens.push(len);
                        c
                    };
                    if use_tail {
                        band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                        nbands += 1;
                    } else {
                        self.bands.push(Band::new(width, dst, code));
                    }
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
        } else {
            cache[..CACHE_WORDS].fill(0);
            let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
            let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
            let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
            let mut dictionary = Dictionary::<TRIM> {
                pairs,
                contents,
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                stack: &mut small_stack,
            };
            let mut id = L::next(e.a, 0, 0);
            let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                L::band_node(e.a, id)
            } else {
                (0, 0, 0, 0)
            };
            let mut in_node = 0u32;
            let mut k = 0u32;
            if use_tail {
                if use_c48_band_encode {
                    let mut chunks = out_tail[..max_bands * 8].chunks_exact_mut(8);
                    let mut tail_pos = 0usize;
                    for a in 0..256 {
                        cursors[a] = tail_pos;
                        let end = k + thin[a];
                        let mut last_w = 0u32;
                        let mut last_c = 0u32;
                        while k < end {
                            let width = (end - k).min(node_width - in_node);
                            assert!(width > 0);
                            let dst = node_bottom + in_node;
                            let code = if node_word == 0 {
                                a as u32
                            } else if node_word == last_w {
                                last_c
                            } else {
                                let c = dictionary.code(&mut e, node_word, a as u8);
                                last_w = node_word;
                                last_c = c;
                                c
                            };
                            let slot = chunks.next().unwrap();
                            slot.copy_from_slice(&Band::new(width, dst, code).0.to_le_bytes());
                            tail_pos += 8;
                            nbands += 1;
                            in_node += width;
                            k += width;
                            if in_node == node_width {
                                id = next_id;
                                in_node = 0;
                                if id != 0 {
                                    (next_id, node_bottom, node_width, node_word) =
                                        L::band_node(e.a, id);
                                }
                            }
                        }
                    }
                } else {
                    let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                    for a in 0..256 {
                        cursors[a] = nbands;
                        let end = k + thin[a];
                        let mut last_w = 0u32;
                        let mut last_c = 0u32;
                        while k < end {
                            let width = (end - k).min(node_width - in_node);
                            assert!(width > 0);
                            let dst = node_bottom + in_node;
                            let code = if node_word == 0 {
                                a as u32
                            } else if node_word == last_w {
                                last_c
                            } else {
                                let c = dictionary.code(&mut e, node_word, a as u8);
                                last_w = node_word;
                                last_c = c;
                                c
                            };
                            band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                            nbands += 1;
                            in_node += width;
                            k += width;
                            if in_node == node_width {
                                id = next_id;
                                in_node = 0;
                                if id != 0 {
                                    (next_id, node_bottom, node_width, node_word) =
                                        L::band_node(e.a, id);
                                }
                            }
                        }
                    }
                }
            } else {
                for a in 0..256 {
                    cursors[a] = self.bands.len();
                    let end = k + thin[a];
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        assert!(width > 0);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else if node_word == last_w {
                            last_c
                        } else {
                            let c = dictionary.code(&mut e, node_word, a as u8);
                            last_w = node_word;
                            last_c = c;
                            c
                        };
                        self.bands.push(Band::new(width, dst, code));
                        in_node += width;
                        k += width;
                        if in_node == node_width {
                            id = next_id;
                            in_node = 0;
                            if id != 0 {
                                (next_id, node_bottom, node_width, node_word) =
                                    L::band_node(e.a, id);
                            }
                        }
                    }
                }
            }
            assert_eq!(id, 0);
        }
        tel.bands_count = if use_tail { nbands } else { self.bands.len() };
        if let Some(t) = t_s3a {
            tm.s3a_us = t.elapsed().as_secs_f64() * 1e6;
        }
        let end_val = if use_tail && use_c48_band_encode {
            nbands * 8
        } else {
            nbands
        };
        (cursors, use_tail, 0, end_val)
    }

    // ------------------------------------------------------------------------
    // Arm 4 (`OT-Nodes`): `open_nodes` + `image_order` (overwritten by `bands`)
    // in `out_tail`; `rules` in `self.edges[0..]` (`base = 0`!).
    // ------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn induced_outtail_nodes<const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
        tel: &mut Telemetry,
    ) -> ([usize; 256], bool, usize, usize) {
        let s2a_need = most * 16 + ns * 4;
        if out_tail.len() < s2a_need {
            // Exact clean fallback to standard u32-arena + OutTail64 bands
            let need = self.arena_words(ns, m as usize);
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }
            return self.induced_std::<Narrow, true, TIMED>(
                ns, n, m, thin, first, write, most, out_tail, true, false, tm, tel,
            );
        }

        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let mut widest = 0u32;
        let e_nodes = {
            let (nodes_raw, img_raw) = out_tail.split_at_mut(most * 16);
            let (nodes_slots, _) = nodes_raw.as_chunks_mut::<16>();
            let (img_slots, _) = img_raw[..ns * 4].as_chunks_mut::<4>();
            let strips_raw = &mut self.edges[..3 * ns];
            let mut dummy_rules: [u32; 0] = [];
            let mut e = SplitNodesElim {
                nodes_buf: nodes_slots,
                rules: &mut dummy_rules,
                rules_top: 0,
                nodes: 0,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0u32;
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let from = sw[0];
                let wbyte = sw[2];
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                sw[1] += first[a];
                let dst_idx = write[a] as usize;
                img_slots[dst_idx] = (i as u32).to_le_bytes();
                write[a] = dst_idx as u32 + 1;
                let sid = i as u32 + 1;
                e.set_node_w(sid, 2, width);
                e.set_node_w(sid, 3, literal(byte));
                widest = widest.max(width);
                if x < from {
                    let gw = from - x;
                    widest = widest.max(gw);
                    let gid = e.node(gw, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                }
                e.append_to(&mut order[0], sid, 0);
                x = from + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for raw_i in img_slots.iter() {
                let i = u32::from_le_bytes(*raw_i);
                let idx = i as usize;
                let sw = &strips_raw[3 * idx..3 * idx + 3];
                let to = sw[1];
                let width = sw[2] >> 8;
                if x < to {
                    widest = widest.max(to - x);
                    let id = e.node(to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = to + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            e.nodes
        };
        if let Some(t) = t_s2a {
            tm.s2a_us = t.elapsed().as_secs_f64() * 1e6;
        }

        // Size self.edges ONLY to `max(m, 4 * e_nodes)` (rules start at self.edges[0], base == 0!)
        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let open_nodes = e_nodes as usize - 1;
        let init_rules_words = (m as usize).max(RULE * e_nodes as usize);
        if self.edges.len() < init_rules_words {
            let add = init_rules_words - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(init_rules_words, 0);
        }

        let live_nodes_bytes = e_nodes as usize * 16;
        let (nodes_raw, bands_raw) = out_tail.split_at_mut(live_nodes_bytes);
        let (nodes_slots, _) = nodes_raw.as_chunks_mut::<16>();

        let spirals = widest > CERTIFIED_WIDTH || self.force_spirals;
        let mut remaining = n;
        let mut rules_top = 0usize;
        loop {
            let mut e = SplitNodesElim {
                nodes_buf: nodes_slots,
                rules: &mut self.edges[..],
                rules_top,
                nodes: e_nodes,
            };
            let done = if spirals {
                e.eliminate_spirals(&mut remaining)
            } else {
                e.eliminate_plain(&mut remaining)
            };
            rules_top = e.rules_top;
            if done {
                break;
            }
            grow_u32_vec(&mut self.edges, rules_top + SPIRAL_RULES * RULE);
        }
        if let Some(t) = t_s2b {
            tm.s2b_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        tel.used_ot_nodes = true;
        tel.open_nodes = open_nodes;
        tel.rules_count = rules_top / RULE;
        tel.edges_touched_bytes = 4 * (3 * ns).max(rules_top);

        let mut e = SplitNodesElim {
            nodes_buf: nodes_slots,
            rules: &mut self.edges[..rules_top],
            rules_top,
            nodes: e_nodes,
        };
        e.assign_bottom(m);
        begin_inline_trim(&mut self.inline16, &mut self.lens);

        let max_bands = open_nodes + 256;
        let use_tail_bands = bands_raw.len() >= max_bands * 8;
        tel.used_ot_bands = use_tail_bands;

        // Stack-local 10 KiB S3a dedup cache (0 B heap, 0 B self.edges touched!)
        let mut pairs = [0u32; 512];
        let mut contents = [0u32; 2048];
        let mut small_stack = SmallStack::new(&mut self.stack);
        let mut dictionary = Dictionary::<true> {
            pairs: &mut pairs,
            contents: &mut contents,
            inline16: &mut self.inline16,
            lens: &mut self.lens,
            spelling: &mut self.spelling,
            stack: &mut small_stack,
        };

        let mut cursors = [0usize; 256];
        let mut nbands = 0usize;
        let mut id = e.next(0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            e.band_node(id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        if use_tail_bands {
            let (band_slots, _) = bands_raw[..max_bands * 8].as_chunks_mut::<8>();
            for a in 0..256 {
                cursors[a] = nbands;
                let end = k + thin[a];
                let mut last_w = 0u32;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(&mut e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                    nbands += 1;
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) = e.band_node(id);
                        }
                    }
                }
            }
        } else {
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                let mut last_w = 0u32;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(&mut e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    self.bands.push(Band::new(width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) = e.band_node(id);
                        }
                    }
                }
            }
        }
        assert_eq!(id, 0);
        tel.bands_count = if use_tail_bands {
            nbands
        } else {
            self.bands.len()
        };
        if let Some(t) = t_s3a {
            tm.s3a_us = t.elapsed().as_secs_f64() * 1e6;
        }
        (cursors, use_tail_bands, live_nodes_bytes, nbands)
    }

    // ------------------------------------------------------------------------
    // Arm 5 / 6 (`OT-FullArena`): `open_nodes` + `rules` + `bands` ALL in `out_tail`!
    // Layout inside `out_tail`:
    //   S2a: [0 .. most*16] = nodes | [most*16 .. most*16 + ns*4] = image_order
    //   S2b: [0 .. e_nodes*16] = nodes | [e_nodes*16 .. end - bands_reserve] = rules!
    //   S3a: [end - bands_reserve .. end] = bands! (disjoint from BOTH nodes and rules!)
    // If S2b rules ever fill `end - bands_reserve`, seamlessly migrates existing rules
    // into `self.edges[0..]` and finishes via `SplitNodesElim` — 100% exact, 0 panic!
    // ------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn induced_outtail_full<const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
        tel: &mut Telemetry,
    ) -> ([usize; 256], bool, usize, usize) {
        let s2a_need = most * 16 + ns * 4;
        if out_tail.len() < s2a_need {
            let need = self.arena_words(ns, m as usize);
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }
            return self.induced_std::<Narrow, true, TIMED>(
                ns, n, m, thin, first, write, most, out_tail, true, false, tm, tel,
            );
        }

        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let mut widest = 0u32;
        let e_nodes = {
            let (nodes_raw, img_raw) = out_tail.split_at_mut(most * 16);
            let (nodes_slots, _) = nodes_raw.as_chunks_mut::<16>();
            let (img_slots, _) = img_raw[..ns * 4].as_chunks_mut::<4>();
            let strips_raw = &mut self.edges[..3 * ns];
            let mut e = FullOutTailElim {
                slots: nodes_slots,
                base: 0,
                top: 0,
                nodes: 0,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0u32;
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let from = sw[0];
                let wbyte = sw[2];
                let byte = (wbyte & 0xFF) as u8;
                let width = wbyte >> 8;
                let a = usize::from(byte);
                sw[1] += first[a];
                let dst_idx = write[a] as usize;
                img_slots[dst_idx] = (i as u32).to_le_bytes();
                write[a] = dst_idx as u32 + 1;
                let sid = i as u32 + 1;
                e.set_w(sid as usize, 2, width);
                e.set_w(sid as usize, 3, literal(byte));
                widest = widest.max(width);
                if x < from {
                    let gw = from - x;
                    widest = widest.max(gw);
                    let gid = e.node(gw, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                }
                e.append_to(&mut order[0], sid, 0);
                x = from + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for raw_i in img_slots.iter() {
                let i = u32::from_le_bytes(*raw_i);
                let idx = i as usize;
                let sw = &strips_raw[3 * idx..3 * idx + 3];
                let to = sw[1];
                let width = sw[2] >> 8;
                if x < to {
                    widest = widest.max(to - x);
                    let id = e.node(to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = to + width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            e.nodes
        };
        if let Some(t) = t_s2a {
            tm.s2a_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let open_nodes = e_nodes as usize - 1;
        let max_bands = open_nodes + 256;
        let bands_bytes = max_bands * 8;
        let nodes_bytes = e_nodes as usize * 16;

        // Reserve bands at the END of `out_tail` (aligned down to 16B boundary for arena)
        let arena_bytes_avail = out_tail.len().saturating_sub(bands_bytes) & !15usize;
        let avail_slots = arena_bytes_avail / 16;
        let cap_slots = if let Some(forced) = self.force_small_ot_rules {
            avail_slots.min(e_nodes as usize + forced)
        } else {
            avail_slots
        };
        let spirals = widest > CERTIFIED_WIDTH || self.force_spirals;
        let min_rules_room = if spirals { SPIRAL_RULES } else { 1 };

        // Try running S2b 100% inside `out_tail[0..cap_slots*16]`!
        let mut remaining = n;
        let (done_in_tail, top_slots) = if cap_slots >= e_nodes as usize + min_rules_room {
            let (arena_raw, _) = out_tail.split_at_mut(cap_slots * 16);
            let (slots, _) = arena_raw.as_chunks_mut::<16>();
            let mut e = FullOutTailElim {
                slots,
                base: e_nodes as usize,
                top: e_nodes as usize,
                nodes: e_nodes,
            };
            let done = if spirals {
                e.eliminate_spirals(&mut remaining)
            } else {
                e.eliminate_plain(&mut remaining)
            };
            (done, e.top)
        } else {
            (false, e_nodes as usize)
        };

        if done_in_tail {
            if let Some(t) = t_s2b {
                tm.s2b_us = t.elapsed().as_secs_f64() * 1e6;
            }
            let t_s3a = if TIMED { Some(Instant::now()) } else { None };
            tel.used_ot_full = true;
            tel.used_ot_nodes = true;
            tel.used_ot_bands = true;
            tel.open_nodes = open_nodes;
            tel.rules_count = top_slots - e_nodes as usize;
            tel.edges_touched_bytes = 4 * (3 * ns); // ONLY strips_raw in S1! Zero S2b/S3a touches in edges!

            let band_start = arena_bytes_avail;
            let (arena_raw, bands_raw) = out_tail.split_at_mut(band_start);
            let (slots, _) = arena_raw[..top_slots * 16].as_chunks_mut::<16>();
            let mut e = FullOutTailElim {
                slots,
                base: e_nodes as usize,
                top: top_slots,
                nodes: e_nodes,
            };
            e.assign_bottom(m);
            begin_inline_trim(&mut self.inline16, &mut self.lens);

            let mut pairs = [0u32; 512];
            let mut contents = [0u32; 2048];
            let mut small_stack = SmallStack::new(&mut self.stack);
            let mut dictionary = Dictionary::<true> {
                pairs: &mut pairs,
                contents: &mut contents,
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                stack: &mut small_stack,
            };

            let (band_slots, _) = bands_raw[..bands_bytes].as_chunks_mut::<8>();
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut id = e.next(0, 0);
            let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                e.band_node(id)
            } else {
                (0, 0, 0, 0)
            };
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256 {
                cursors[a] = nbands;
                let end = k + thin[a];
                let mut last_w = 0u32;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(&mut e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                    nbands += 1;
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) = e.band_node(id);
                        }
                    }
                }
            }
            assert_eq!(id, 0);
            tel.bands_count = nbands;
            if let Some(t) = t_s3a {
                tm.s3a_us = t.elapsed().as_secs_f64() * 1e6;
            }
            return (cursors, true, band_start, nbands);
        }

        // Seamless cold migration: rules outgrew `out_tail`! Copy already-built rules to `self.edges[0..]`
        // and finish S2b + S3a with `SplitNodesElim` (`open_nodes` stays in `out_tail[..nodes_bytes]`!).
        tel.spilled_rules = true;
        let existing_rules = top_slots - e_nodes as usize;
        let existing_words = existing_rules * RULE;
        let init_words = (existing_words + SPIRAL_RULES * RULE)
            .max(m as usize)
            .max(RULE * e_nodes as usize);
        if self.edges.len() < init_words {
            let add = init_words - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(init_words, 0);
        }
        {
            let (old_slots, _) = out_tail[nodes_bytes..top_slots * 16].as_chunks::<16>();
            for (ri, sl) in old_slots.iter().enumerate() {
                let dst = &mut self.edges[ri * 4..ri * 4 + 4];
                dst[0] = rd_u32_slot(sl, 0);
                dst[1] = rd_u32_slot(sl, 1);
                dst[2] = rd_u32_slot(sl, 2);
                dst[3] = rd_u32_slot(sl, 3);
            }
        }
        let (nodes_raw, bands_raw) = out_tail.split_at_mut(nodes_bytes);
        let (nodes_slots, _) = nodes_raw.as_chunks_mut::<16>();
        let mut rules_top = existing_words;
        loop {
            let mut e = SplitNodesElim {
                nodes_buf: nodes_slots,
                rules: &mut self.edges[..],
                rules_top,
                nodes: e_nodes,
            };
            let done = if spirals {
                e.eliminate_spirals(&mut remaining)
            } else {
                e.eliminate_plain(&mut remaining)
            };
            rules_top = e.rules_top;
            if done {
                break;
            }
            grow_u32_vec(&mut self.edges, rules_top + SPIRAL_RULES * RULE);
        }
        if let Some(t) = t_s2b {
            tm.s2b_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        tel.used_ot_nodes = true;
        tel.open_nodes = open_nodes;
        tel.rules_count = rules_top / RULE;
        tel.edges_touched_bytes = 4 * (3 * ns).max(rules_top);

        let mut e = SplitNodesElim {
            nodes_buf: nodes_slots,
            rules: &mut self.edges[..rules_top],
            rules_top,
            nodes: e_nodes,
        };
        e.assign_bottom(m);
        begin_inline_trim(&mut self.inline16, &mut self.lens);

        let use_tail_bands = bands_raw.len() >= bands_bytes;
        tel.used_ot_bands = use_tail_bands;
        let mut pairs = [0u32; 512];
        let mut contents = [0u32; 2048];
        let mut small_stack = SmallStack::new(&mut self.stack);
        let mut dictionary = Dictionary::<true> {
            pairs: &mut pairs,
            contents: &mut contents,
            inline16: &mut self.inline16,
            lens: &mut self.lens,
            spelling: &mut self.spelling,
            stack: &mut small_stack,
        };
        let mut cursors = [0usize; 256];
        let mut nbands = 0usize;
        let mut id = e.next(0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            e.band_node(id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        if use_tail_bands {
            let (band_slots, _) = bands_raw[..bands_bytes].as_chunks_mut::<8>();
            for a in 0..256 {
                cursors[a] = nbands;
                let end = k + thin[a];
                let mut last_w = 0u32;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(&mut e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                    nbands += 1;
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) = e.band_node(id);
                        }
                    }
                }
            }
        } else {
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                let mut last_w = 0u32;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(&mut e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    self.bands.push(Band::new(width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) = e.band_node(id);
                        }
                    }
                }
            }
        }
        assert_eq!(id, 0);
        tel.bands_count = if use_tail_bands {
            nbands
        } else {
            self.bands.len()
        };
        if let Some(t) = t_s3a {
            tm.s3a_us = t.elapsed().as_secs_f64() * 1e6;
        }
        (cursors, use_tail_bands, nodes_bytes, nbands)
    }

    // ------------------------------------------------------------------------
    // S4 Wiring Helpers (`as_chunks::<8>()` Clean vs C48 `chunks_exact(8)` vs `Vec<Band>`)
    // ------------------------------------------------------------------------
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_outtail_clean(
        &mut self,
        cursors: &[usize; 256],
        nbands: usize,
        bands_bytes: &[u8],
        heads: &outq_local::Heads,
        out_q: &[u8],
        inline_next: &[u32; outq_local::INLINE_NEXT],
        use_inline_next: bool,
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        let next_view = outq_local::NextView::new(inline_next, &self.spill.next, use_inline_next);
        let (band_slots, _) = bands_bytes.as_chunks::<8>();
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut queue = outq_local::Reader::new(out_q, &self.spill, next_view, heads, a);
                if a != mb {
                    for &raw in &band_slots[cursors[a]..end] {
                        let band = Band(u64::from_le_bytes(raw));
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &raw in &band_slots[cursors[a]..end] {
                        let band = Band(u64::from_le_bytes(raw));
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut queue = outq_local::Reader::new(out_q, &self.spill, next_view, heads, a);
                for &raw in &band_slots[cursors[a]..end] {
                    let b = Band(u64::from_le_bytes(raw));
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_outtail_c48(
        &mut self,
        cursors: &[usize; 256],
        tail_end_bytes: usize,
        out_tail: &[u8],
        heads: &outq_local::Heads,
        out_q: &[u8],
        inline_next: &[u32; outq_local::INLINE_NEXT],
        use_inline_next: bool,
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        let next_view = outq_local::NextView::new(inline_next, &self.spill.next, use_inline_next);
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end_bytes);
                let mut queue = outq_local::Reader::new(out_q, &self.spill, next_view, heads, a);
                let sl = &out_tail[cursors[a]..end];
                if a != mb {
                    for ch in sl.chunks_exact(8) {
                        let band = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for ch in sl.chunks_exact(8) {
                        let band = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end_bytes);
                let mut queue = outq_local::Reader::new(out_q, &self.spill, next_view, heads, a);
                for ch in out_tail[cursors[a]..end].chunks_exact(8) {
                    let b = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn wire_from_bands_vec(
        &mut self,
        cursors: &[usize; 256],
        heads: &outq_local::Heads,
        out_q: &[u8],
        inline_next: &[u32; outq_local::INLINE_NEXT],
        use_inline_next: bool,
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        let next_view = outq_local::NextView::new(inline_next, &self.spill.next, use_inline_next);
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq_local::Reader::new(out_q, &self.spill, next_view, heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq_local::Reader::new(out_q, &self.spill, next_view, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }
}

// (NextView is Copy)

// ============================================================================
// S5 Emit (Verbatim cand_r5_head)
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bidirectional(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single {
            break;
        }
        let eb = edges[back as usize];
        let cb = (eb >> bits) as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

// ============================================================================
// Reference Forward-FL, Corpus Loading, Exactness & Benchmarks
// ============================================================================

fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[usize::from(b)] += 1;
    }
    let mut st = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        st[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let s = st[usize::from(b)] as usize;
        st[usize::from(b)] += 1;
        fl[s] = ((i as u32) << 8) | u32::from(b);
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for slot in &mut out {
        let v = fl[x];
        *slot = v as u8;
        x = (v >> 8) as usize;
    }
    out
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let b = l[i];
        let mut w = 1usize;
        while i + w < l.len() && l[i + w] == b && w < 127 {
            w += 1;
        }
        recs.push((u16::from(b) << 8) | (w as u16));
        i += w;
    }
    recs
}

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    route_class: u8,
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let manifest = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest");
    let ids = [0usize, 2, 85, 143, 207, 210];
    let mut blocks = Vec::new();
    for &id in &ids {
        let key = format!("\"block-{id}.L\"");
        let pos = manifest.find(&key).expect("find block");
        let sub = &manifest[pos..];
        let pkey = "\"primary_index\":";
        let ppos = sub.find(pkey).expect("find primary_index");
        let rest = sub[ppos + pkey.len()..].trim_start();
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        let p: u32 = rest[..end].parse().expect("parse primary_index");
        let l = fs::read(format!("{dir}/block-{id}.L")).expect("read .L");
        let n = l.len();
        blocks.push(Block {
            name: format!("block-{id}.L"),
            n,
            p,
            records: records_of(&l),
            route_class: 0,
        });
    }
    blocks
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && route_class > 1 {
            continue;
        }
        let name = names
            .and_then(|ns| ns.get(b_idx).map(|s| (*s).to_string()))
            .unwrap_or_else(|| format!("w431-{idx}"));
        blocks.push(Block {
            name,
            n,
            p,
            records,
            route_class,
        });
    }
    blocks
}

fn resolve_path(cands: &[&str]) -> String {
    for &p in cands {
        if Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("could not find corpus in {:?}", cands);
}

const ARM_NAMES: [&str; 7] = [
    "R5-Head",
    "F-Inline3",
    "I3+OT64-C48",
    "I3+OT64-Cln",
    "OT-Nodes",
    "OT-FullAr",
    "OT-Full+Tny",
];

fn run_one_arm<const TIMED: bool>(
    arm: usize,
    r5: &mut cand_r5_head::Kernel,
    dk: &mut DeepKernel,
    blk: &Block,
    out: &mut Vec<u8>,
) -> (StageTimes, Telemetry) {
    match arm {
        0 => {
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, out)
                .unwrap();
            (StageTimes::default(), Telemetry::default())
        }
        1 => {
            let (ok, tm, tel) = dk.run_arm::<ARM_I3_BASE, TIMED>(&blk.records, blk.n, blk.p, out);
            assert!(ok);
            (tm, tel)
        }
        2 => {
            let (ok, tm, tel) = dk.run_arm::<ARM_OT64_C48, TIMED>(&blk.records, blk.n, blk.p, out);
            assert!(ok);
            (tm, tel)
        }
        3 => {
            let (ok, tm, tel) = dk.run_arm::<ARM_OT64_CLEAN, TIMED>(&blk.records, blk.n, blk.p, out);
            assert!(ok);
            (tm, tel)
        }
        4 => {
            let (ok, tm, tel) = dk.run_arm::<ARM_OT_NODES, TIMED>(&blk.records, blk.n, blk.p, out);
            assert!(ok);
            (tm, tel)
        }
        5 => {
            let (ok, tm, tel) = dk.run_arm::<ARM_OT_FULL, TIMED>(&blk.records, blk.n, blk.p, out);
            assert!(ok);
            (tm, tel)
        }
        6 => {
            let (ok, tm, tel) = dk.run_arm::<ARM_OT_FULL_TINY, TIMED>(&blk.records, blk.n, blk.p, out);
            assert!(ok);
            (tm, tel)
        }
        _ => unreachable!(),
    }
}

fn run_exactness_suite(wiki6: &[Block], nonwiki10: &[Block], wiki431: &[Block]) {
    println!("================================================================================");
    println!("PART B HARD EXACTNESS, POISON, SPILL & 431-BLOCK VERIFICATION SUITE");
    println!("================================================================================");
    let mut r5 = cand_r5_head::Kernel::default();
    let mut r5_arm = cand_r5_head::Kernel::default();
    let mut dks: Vec<DeepKernel> = (0..7).map(|_| DeepKernel::default()).collect();
    let mut out_ref = Vec::new();
    let mut out_got = Vec::new();
    let mut out_pad = Vec::new();

    let mut check_all = |recs: &[u16], n: usize, p: u32, expected: &[u8], tag: &str| {
        for arm in 1..7usize {
            // Poison `out` with alternating 0xAA / 0x55 before EVERY call to prove zero stale reads!
            let poison = if arm % 2 == 0 { 0xAA } else { 0x55 };
            out_got.clear();
            out_got.resize(n + 37, poison);
            let blk = Block {
                name: tag.to_string(),
                n,
                p,
                records: recs.to_vec(),
                route_class: 0,
            };
            run_one_arm::<false>(arm, &mut r5_arm, &mut dks[arm], &blk, &mut out_got);
            assert_eq!(out_got.as_slice(), expected, "MISMATCH arm={} tag={tag}", ARM_NAMES[arm]);
        }
    };

    // 1. 5,700 small valid + exhaustive binary + 0x00/0x7F/non-ASCII + thick-strip cases
    let mut small_cnt = 0usize;
    for n in 1..=11usize {
        for mask in 0..(1u32 << n) {
            let l: Vec<u8> = (0..n).map(|i| if (mask >> i) & 1 == 1 { 0xFF } else { 0x00 }).collect();
            let p = (mask as usize % n) as u32;
            let expected = forward_fl_invert(&l, p);
            let recs = records_of(&l);
            check_all(&recs, n, p, &expected, "bin-exhaust");
            small_cnt += 1;
        }
    }
    let mut rng = 0x1234_5678_9abc_def0u64;
    let mut next_u64 = || -> u64 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for idx in 0..1_650 {
        let n = 1 + (next_u64() as usize % 600);
        let mode = idx % 4;
        let mut l = Vec::with_capacity(n);
        while l.len() < n {
            let b = match mode {
                0 => (next_u64() as u8) % 3,
                1 => 0x80 | ((next_u64() as u8) % 16),
                2 => [0x00, 0x7F, 0x80, 0xFF][(next_u64() as usize) % 4],
                _ => next_u64() as u8,
            };
            let w = 1 + (next_u64() as usize % 45);
            let take = w.min(n - l.len());
            l.resize(l.len() + take, b);
        }
        let p = (next_u64() as usize % n) as u32;
        let expected = forward_fl_invert(&l, p);
        let recs = records_of(&l);
        check_all(&recs, n, p, &expected, "small-rand");
        small_cnt += 1;
    }
    println!("[PASS] {small_cnt} small/exhaustive/0x00/0xFF/non-ASCII/thick-strip cases 100% BYTE-EXACT across all 6 arms (with 0xAA/0x55 poison).");

    // 2. 330 malformed multi-cycle columns (incl. 10 large m >= 70k bi-XOR + zero-width malformed records!)
    let mut mal_cnt = 0usize;
    for idx in 0..320 {
        let n = if idx < 10 {
            72_000 + (next_u64() as usize % 15_000)
        } else {
            20 + (next_u64() as usize % 1_200)
        };
        let sigma = if idx < 10 { 32 } else { 2 + (next_u64() as usize % 8) };
        let mut l = Vec::with_capacity(n);
        while l.len() < n {
            let b = (next_u64() as u8) % (sigma as u8);
            let w = if (next_u64() & 7) == 0 {
                17 + (next_u64() as usize % 40)
            } else {
                1 + (next_u64() as usize % 4)
            };
            let take = w.min(n - l.len());
            l.resize(l.len() + take, b);
        }
        let p = (next_u64() as usize % n) as u32;
        let expected = forward_fl_invert(&l, p);
        let recs = records_of(&l);
        r5.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        assert_eq!(out_ref, expected, "cand_r5_head vs FL sanity");
        check_all(&recs, n, p, &expected, "malformed");
        mal_cnt += 1;
    }
    // Zero-width malformed records test (`nrec <= n` with embedded width==0, AND `nrec > n` clean reject!)
    {
        let l = b"banana_malformed_zero_width_test_aaaaaaaaaaaaaaaaaaaa_bbbbbbbbbbbbbbbbbbbb".to_vec();
        let n = l.len();
        let p = 5u32;
        let expected = forward_fl_invert(&l, p);
        let mut recs = records_of(&l);
        recs.insert(0, (u16::from(b'z') << 8) | 0);
        recs.push((u16::from(b'q') << 8) | 0);
        assert!(recs.len() <= n);
        check_all(&recs, n, p, &expected, "zero-width-valid-nrec");
        mal_cnt += 1;

        // `nrec > n` via zero-width padding: clean arms must return `ok == false` (fallback to LazyCells)
        let mut pad_recs = recs.clone();
        while pad_recs.len() <= n + 5 {
            pad_recs.push((u16::from(b'x') << 8) | 0);
        }
        for arm in [ARM_OT64_CLEAN, ARM_OT_NODES, ARM_OT_FULL] {
            let mut dk = DeepKernel::default();
            let (ok, _, _) = match arm {
                ARM_OT64_CLEAN => dk.run_arm::<ARM_OT64_CLEAN, false>(&pad_recs, n, p, &mut out_pad),
                ARM_OT_NODES => dk.run_arm::<ARM_OT_NODES, false>(&pad_recs, n, p, &mut out_pad),
                _ => dk.run_arm::<ARM_OT_FULL, false>(&pad_recs, n, p, &mut out_pad),
            };
            assert!(!ok, "nrec > n must cleanly return false for LazyCells fallback");
        }
        mal_cnt += 1;
    }
    println!("[PASS] {mal_cnt} malformed multi-cycle (incl. 10 large m>=70k bi-XOR + zero-width/nrec>n) 100% BYTE-EXACT vs forward-FL & cand_r5_head.");

    // 3. Adversarial stress: spirals (SATURATED=255), force_wide, force_spirals, force_no_outtail,
    //    FORCED OT-Full rules spill during S2b, and tiny<->900K resized-`out` buffer reuse with 0xAA/0x55 poison!
    {
        // Spiral 200k
        let mut l = vec![b'b'; 76_394];
        l.resize(200_000, b'a');
        let recs = records_of(&l);
        let p = 123_456u32;
        let expected = forward_fl_invert(&l, p);
        check_all(&recs, l.len(), p, &expected, "spiral-200k");

        // Forced OT-Full rules spill during S2b (`force_small_ot_rules = Some(0), Some(1), Some(100), Some(2000)`)
        // + force_wide + force_spirals + force_no_outtail on block-0.L & block-143.L!
        for blk in [&wiki6[0], &wiki6[3]] {
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
            for &cap_r in &[Some(0), Some(1), Some(64), Some(1500), None] {
                let mut dk = DeepKernel {
                    force_small_ot_rules: cap_r,
                    ..Default::default()
                };
                out_got.clear();
                out_got.resize(blk.n + 64, 0xAA);
                let (ok, _, tel) = dk.run_arm::<ARM_OT_FULL, false>(&blk.records, blk.n, blk.p, &mut out_got);
                assert!(ok);
                assert_eq!(out_got, out_ref, "OT-Full forced spill cap={cap_r:?}");
                if cap_r.is_some() {
                    assert!(tel.spilled_rules, "expected forced spill to trigger");
                } else {
                    assert!(tel.used_ot_full && !tel.spilled_rules, "expected full out_tail hit on {}", blk.name);
                }
            }
            for &(fw, fs, fno) in &[(true, false, false), (false, true, false), (false, false, true)] {
                let mut dk = DeepKernel {
                    force_wide: fw,
                    force_spirals: fs,
                    force_no_outtail: fno,
                    ..Default::default()
                };
                out_got.clear();
                out_got.resize(blk.n + 64, 0x55);
                let (ok, _, _) = dk.run_arm::<ARM_OT_FULL, false>(&blk.records, blk.n, blk.p, &mut out_got);
                assert!(ok);
                assert_eq!(out_got, out_ref, "OT-Full override ({fw},{fs},{fno})");
            }
        }

        // Resized-out buffer reuse: alternating tiny (n=300) <-> 900K (block-0.L) <-> medium (n=20000) <-> block-143.L
        let tiny_l = b"hello world! ".repeat(25);
        let tiny_blk = Block {
            name: "tiny".to_string(),
            n: tiny_l.len(),
            p: 7,
            records: records_of(&tiny_l),
            route_class: 2,
        };
        let tiny_exp = forward_fl_invert(&tiny_l, 7);
        let mut shared_out = Vec::new();
        for arm in 1..7usize {
            let mut dk = DeepKernel::default();
            for step in 0..6 {
                let blk = if step % 2 == 0 { &tiny_blk } else { &wiki6[step % wiki6.len()] };
                run_one_arm::<false>(arm, &mut r5, &mut dk, blk, &mut shared_out);
                if step % 2 == 0 {
                    assert_eq!(shared_out, tiny_exp);
                } else {
                    r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
                    assert_eq!(shared_out, out_ref);
                }
            }
        }
    }
    println!("[PASS] Adversarial stress (spiral-200k, forced OT-Full S2b spill=0/1/64/1500, force_wide/spirals/no_outtail, tiny<->900K buffer reuse) 100% BYTE-EXACT.");

    // 4. All 6 Admitted + 10 Non-Wikidata/CodeDocs + ALL 431 wikidata.bz2 blocks!
    let mut full_hits = 0usize;
    let mut nodes_hits = 0usize;
    let mut bands_hits = 0usize;
    let mut spill_hits = 0usize;
    for (corp_name, list) in [
        ("6 Admitted", wiki6),
        ("10 Non-Wikidata+CodeDocs", nonwiki10),
        ("431 ALL wikidata.bz2", wiki431),
    ] {
        let mut arm_dks: Vec<DeepKernel> = (0..7).map(|_| DeepKernel::default()).collect();
        let mut out_a = Vec::new();
        for blk in list {
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
            for arm in 1..7usize {
                let (_, tel) = run_one_arm::<false>(arm, &mut r5, &mut arm_dks[arm], blk, &mut out_a);
                assert_eq!(
                    out_a, out_ref,
                    "REAL MISMATCH corpus={corp_name} blk={} arm={}",
                    blk.name, ARM_NAMES[arm]
                );
                if corp_name == "431 ALL wikidata.bz2" && arm == 5 && blk.route_class <= 1 {
                    if tel.used_ot_full { full_hits += 1; }
                    if tel.used_ot_nodes { nodes_hits += 1; }
                    if tel.used_ot_bands { bands_hits += 1; }
                    if tel.spilled_rules { spill_hits += 1; }
                }
            }
        }
        println!("[PASS] {corp_name} ({} blocks) 100% BYTE-EXACT across ALL 6 arms!", list.len());
    }
    println!("  (On 233 Full Wikidata under OT-FullArena: full_in_tail={full_hits}/233, nodes_in_tail={nodes_hits}/233, bands_in_tail={bands_hits}/233, rules_spilled={spill_hits}/233)");
}

fn audit_ws_and_touched(blocks: &[Block], label: &str) {
    println!("\n--- Persistent WS & Per-Block Distinct Non-`out` Bytes Touched — {label} ({} blocks) ---", blocks.len());
    println!(
        "{:<12} | {:>9} {:>9} | {:>8} {:>7} {:>7} {:>7} {:>7} | {:>10} {:>10} | {:>6}",
        "Arm", "TotWS(KB)", "dWS(KB)", "EdgesKB", "BandKB", "SpelKB", "InlKB", "SpilKB", "AvgTch(KB)", "MaxTch(KB)", "OTFull"
    );
    let mut base_ws = 0usize;
    for arm in 0..7usize {
        let mut r5 = cand_r5_head::Kernel::default();
        let mut dk = DeepKernel::default();
        let mut out = Vec::new();
        let mut sum_tch = 0u64;
        let mut max_tch = 0usize;
        let mut full_cnt = 0usize;
        let mut nodes_cnt = 0usize;
        let mut bands_cnt = 0usize;
        for blk in blocks {
            let (_, tel) = run_one_arm::<false>(arm, &mut r5, &mut dk, blk, &mut out);
            sum_tch += tel.non_out_touched_bytes as u64;
            max_tch = max_tch.max(tel.non_out_touched_bytes);
            if tel.used_ot_full { full_cnt += 1; }
            if tel.used_ot_nodes { nodes_cnt += 1; }
            if tel.used_ot_bands { bands_cnt += 1; }
        }
        if arm == 0 {
            let (tot, _, _) = r5.scratch_report();
            base_ws = tot;
            println!(
                "{:<12} | {:>9.2} {:>+9.2} | {:>8} {:>7} {:>7} {:>7} {:>7} | {:>10} {:>10} | {:>6}",
                ARM_NAMES[arm],
                tot as f64 / 1024.0,
                0.0,
                "-", "-", "-", "-", "-", "-", "-", "-"
            );
        } else {
            let (tot, e_b, b_b, s_b, i_b, sp_b, _) = dk.ws_breakdown();
            let d_ws = (tot as f64 - base_ws as f64) / 1024.0;
            let avg_tch = (sum_tch as f64 / blocks.len().max(1) as f64) / 1024.0;
            let hit_str = if arm >= 5 {
                format!("{full_cnt}/{}", blocks.len())
            } else if arm == 4 {
                format!("{nodes_cnt}/{}", blocks.len())
            } else if arm >= 2 {
                format!("{bands_cnt}/{}", blocks.len())
            } else {
                "-".to_string()
            };
            println!(
                "{:<12} | {:>9.2} {:>+9.2} | {:>8.2} {:>7.2} {:>7.2} {:>7.2} {:>7.2} | {:>10.2} {:>10.2} | {:>6}",
                ARM_NAMES[arm],
                tot as f64 / 1024.0,
                d_ws,
                e_b as f64 / 1024.0,
                b_b as f64 / 1024.0,
                s_b as f64 / 1024.0,
                i_b as f64 / 1024.0,
                sp_b as f64 / 1024.0,
                avg_tch,
                max_tch as f64 / 1024.0,
                hit_str
            );
        }
    }
}

fn bench_stage_1t(blocks: &[Block], label: &str, rounds: usize) {
    println!("\n--- Stage-by-Stage 1T Timings (p50 summed us, {rounds} paired rounds) — {label} ---");
    let mut r5 = cand_r5_head::Kernel::default();
    let mut dks: Vec<DeepKernel> = (0..7).map(|_| DeepKernel::default()).collect();
    let mut out = Vec::new();

    let mut s1 = [0.0f64; 7];
    let mut s2a = [0.0f64; 7];
    let mut s2b = [0.0f64; 7];
    let mut s3a = [0.0f64; 7];
    let mut s4 = [0.0f64; 7];
    let mut s5 = [0.0f64; 7];
    let mut ind = [0.0f64; 7];
    let mut tot = [0.0f64; 7];

    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    };

    for blk in blocks {
        for arm in 0..7usize {
            run_one_arm::<false>(arm, &mut r5, &mut dks[arm], blk, &mut out);
        }
        let mut vs1 = vec![Vec::with_capacity(rounds); 7];
        let mut vs2a = vec![Vec::with_capacity(rounds); 7];
        let mut vs2b = vec![Vec::with_capacity(rounds); 7];
        let mut vs3a = vec![Vec::with_capacity(rounds); 7];
        let mut vs4 = vec![Vec::with_capacity(rounds); 7];
        let mut vs5 = vec![Vec::with_capacity(rounds); 7];
        let mut vind = vec![Vec::with_capacity(rounds); 7];
        let mut vtot = vec![Vec::with_capacity(rounds); 7];

        for _ in 0..rounds {
            for arm in 0..7usize {
                if arm == 0 {
                    let (c, e) = r5.run_stages(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
                    vs5[0].push(e);
                    vtot[0].push(c + e);
                } else {
                    let (tm, _) = run_one_arm::<true>(arm, &mut r5, &mut dks[arm], blk, &mut out);
                    vs1[arm].push(tm.s1_us);
                    vs2a[arm].push(tm.s2a_us);
                    vs2b[arm].push(tm.s2b_us);
                    vs3a[arm].push(tm.s3a_us);
                    vs4[arm].push(tm.s4_us);
                    vs5[arm].push(tm.s5_us);
                    vind[arm].push(tm.ind_us());
                }
            }
            for arm in 1..7usize {
                let t0 = Instant::now();
                run_one_arm::<false>(arm, &mut r5, &mut dks[arm], blk, &mut out);
                vtot[arm].push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        for arm in 0..7usize {
            if arm == 0 {
                s5[0] += p50(&mut vs5[0]);
                tot[0] += p50(&mut vtot[0]);
            } else {
                s1[arm] += p50(&mut vs1[arm]);
                s2a[arm] += p50(&mut vs2a[arm]);
                s2b[arm] += p50(&mut vs2b[arm]);
                s3a[arm] += p50(&mut vs3a[arm]);
                s4[arm] += p50(&mut vs4[arm]);
                s5[arm] += p50(&mut vs5[arm]);
                ind[arm] += p50(&mut vind[arm]);
                tot[arm] += p50(&mut vtot[arm]);
            }
        }
    }

    println!(
        "{:<12} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>8} {:>7} | {:>8} {:>7} {:>7}",
        "Arm", "S1", "S2a", "S2b", "S3a", "S4", "S5", "Ind(us)", "vsI3", "Tot(us)", "vsR5H", "vsI3"
    );
    for arm in 0..7usize {
        if arm == 0 {
            println!(
                "{:<12} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7.1} | {:>8} {:>7} | {:>8.1}  1.000x  {:.3}x",
                ARM_NAMES[arm], "-", "-", "-", "-", "-", s5[0], "-", "-", tot[0], tot[0] / tot[1]
            );
        } else {
            println!(
                "{:<12} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>8.1}  {:.3}x | {:>8.1}  {:.3}x  {:.3}x",
                ARM_NAMES[arm],
                s1[arm], s2a[arm], s2b[arm], s3a[arm], s4[arm], s5[arm],
                ind[arm], ind[arm] / ind[1],
                tot[arm], tot[arm] / tot[0], tot[arm] / tot[1]
            );
        }
    }
}

fn bench_multiworker(
    blocks: &[Block],
    label: &str,
    rounds: usize,
    iters_1w: usize,
    iters_mw: usize,
) {
    println!("\n--- Multi-Worker (1W / 2W / 4W, paired {rounds} rds, p50 wall ms [vsR5H/vsI3]) — {label} ---");
    let warm_n = blocks.len().min(16);
    for &workers in &[1usize, 2, 4] {
        let mut res: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); 7];
        let iters = if workers == 1 { iters_1w } else { iters_mw };
        for _ in 0..rounds {
            for arm in 0..7usize {
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for _ in 0..workers {
                        handles.push(s.spawn(|| {
                            let mut r5 = cand_r5_head::Kernel::default();
                            let mut dk = DeepKernel::default();
                            let mut out = Vec::new();
                            for blk in &blocks[..warm_n] {
                                run_one_arm::<false>(arm, &mut r5, &mut dk, blk, &mut out);
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    run_one_arm::<false>(arm, &mut r5, &mut dk, blk, &mut out);
                                    std::hint::black_box(&out);
                                }
                            }
                            t0.elapsed().as_secs_f64() * 1e3 / iters as f64
                        }));
                    }
                    let mut max_ms = 0.0f64;
                    for h in handles {
                        max_ms = max_ms.max(h.join().unwrap());
                    }
                    max_ms
                });
                res[arm].push(t_wall);
            }
        }
        let p50_s = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            s[s.len() / 2]
        };
        print!("  {workers}W: ");
        for arm in 0..7usize {
            let m = p50_s(&res[arm]);
            if arm == 0 {
                print!("{}={:.2}ms ", ARM_NAMES[arm], m);
            } else {
                let mut pr0: Vec<f64> = (0..rounds).map(|r| res[arm][r] / res[0][r]).collect();
                let mut pr1: Vec<f64> = (0..rounds).map(|r| res[arm][r] / res[1][r]).collect();
                pr0.sort_by(|a, b| a.total_cmp(b));
                pr1.sort_by(|a, b| a.total_cmp(b));
                print!("| {}={:.2}({:.3}/{:.3}) ", ARM_NAMES[arm], m, pr0[rounds / 2], pr1[rounds / 2]);
            }
        }
        println!();
    }
}

fn main() {
    let wiki6 = load_six_admitted();
    let nonwiki_path = resolve_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ]);
    let codedocs_path = resolve_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]);
    let wiki431_path = resolve_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);

    let nw_names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nw_names), false);
    let cd_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
    let code4 = load_bin_blocks(&codedocs_path, Some(&cd_names), false);
    let wiki233 = load_bin_blocks(&wiki431_path, None, true);
    let wiki431 = load_bin_blocks(&wiki431_path, None, false);

    let nonwiki10: Vec<Block> = nonwiki6
        .iter()
        .chain(code4.iter())
        .map(|b| Block {
            name: b.name.clone(),
            n: b.n,
            p: b.p,
            records: b.records.clone(),
            route_class: b.route_class,
        })
        .collect();

    run_exactness_suite(&wiki6, &nonwiki10, &wiki431);

    audit_ws_and_touched(&wiki6, "6 Admitted Wikidata Blocks");
    audit_ws_and_touched(&nonwiki6, "6 Real Non-Wikidata Blocks (JSONL + Syslog)");
    audit_ws_and_touched(&code4, "4 Real Code & Docs Blocks");
    audit_ws_and_touched(&wiki233, "233 Full wikidata.bz2 Blocks");
    audit_ws_and_touched(&wiki431, "431 ALL wikidata.bz2 Blocks");

    bench_stage_1t(&wiki6, "6 Admitted Wikidata Blocks", 15);
    bench_stage_1t(&nonwiki6, "6 Real Non-Wikidata Blocks (JSONL + Syslog)", 15);
    bench_stage_1t(&code4, "4 Real Code & Docs Blocks", 9);

    bench_multiworker(&wiki6, "6 Admitted Wikidata Blocks", 9, 4, 4);
    bench_multiworker(&nonwiki6, "6 Real Non-Wikidata Blocks (JSONL + Syslog)", 9, 6, 6);
    bench_multiworker(&code4, "4 Real Code & Docs Blocks", 5, 2, 2);
    bench_multiworker(&wiki233, "233 Full wikidata.bz2 Blocks", 5, 1, 1);
    bench_multiworker(&wiki431, "431 ALL wikidata.bz2 Blocks", 5, 1, 1);
}
