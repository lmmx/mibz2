#![forbid(unsafe_code)]
//! B4-Grid-Flat16: Compositional Bucket IET-Schur Contraction with:
//!   - Theorem B4 (Zero-Tombstone Causal Routing, 0.00% stale, single-copy pieces)
//!   - Shared 1 KiB ChunkPool allocator (reclaims bucket chunks immediately into L2!)
//!   - Single-Pass Shift-Bin-Grid (O(k) linear sort + L1d lookup, zero binary search)
//!   - Sub-Interval 1-Piece Schur (delta==0 guard + hysteresis Boundary-RV fallback)
//!   - Inline3 (u32 handle) + slab16 (len 4..=15) + compact 12B SLP (len >= 16)
//!
//! Conforms to mibz2's `bench/moonshot` `crate::candidate` contract (`ee7fbf0`).

use crate::bwt::{InverseBwt, OpenCells};
use crate::runs::Runs;
use crate::Kernels;

/// The scorer reads this name.
pub type Candidate = B4GridFlat16;

/// Whether a block of `records` capped run records in `n` rows is under LazyCells' density line.
#[inline]
pub fn under_density(records: usize, n: usize) -> bool {
    records * 20 < n * 3
}

// ============================================================================
// Compact 16-Byte Hot Piece & Shared 1 KiB ChunkPool (Zero Vec-Capacity Bloat!)
// ============================================================================

const TAG_SHIFT: u32 = 30;
const TAG_MASK: u32 = 3 << TAG_SHIFT;
const TAG_LEN1: u32 = 0 << TAG_SHIFT;
const TAG_EXT_MASK: u32 = 7 << 29;
const TAG_SLAB16: u32 = 6 << 29; // len in 4..=15 -> slab16[word & EXT_IDX_MASK]
const TAG_SLP: u32 = 7 << 29;    // len >= 16 -> slp[word & EXT_IDX_MASK]
const EXT_IDX_MASK: u32 = (1 << 29) - 1;
const SLAB16_BYTES_MASK: u128 = (1u128 << 120) - 1;
const REP_BIT: u32 = 1 << 31;
const SUB_THICK_TAG: u8 = 255;

const CHUNK_CAP: usize = 64; // 64 * 16B = 1 KiB per chunk!
const NO_CHUNK: u32 = u32::MAX;

#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
struct Piece16 {
    src_sb: u32, // low 24: src, high 8: src_b
    len_db: u32, // low 24: len, high 8: dst_b
    dst: u32,    // dst row
    word: u32,   // tagged word handle
}

impl Piece16 {
    #[inline(always)]
    fn new(src: u32, len: u32, dst: u32, sb: u8, db: u8, word: u32) -> Self {
        Self {
            src_sb: src | ((sb as u32) << 24),
            len_db: len | ((db as u32) << 24),
            dst,
            word,
        }
    }
    #[inline(always)]
    fn src(self) -> u32 {
        self.src_sb & 0x00FF_FFFF
    }
    #[inline(always)]
    fn sb(self) -> u8 {
        (self.src_sb >> 24) as u8
    }
    #[inline(always)]
    fn len(self) -> u32 {
        self.len_db & 0x00FF_FFFF
    }
    #[inline(always)]
    fn db(self) -> u8 {
        (self.len_db >> 24) as u8
    }
    #[inline(always)]
    fn end(self) -> u32 {
        self.src() + self.len()
    }
}

struct Chunk {
    next: u32,
    len: u16,
    data: [Piece16; CHUNK_CAP],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            next: NO_CHUNK,
            len: 0,
            data: [Piece16::default(); CHUNK_CAP],
        }
    }
}

#[derive(Clone, Copy)]
struct BucketList {
    head: u32,
    tail: u32,
    total: u32,
}

impl Default for BucketList {
    fn default() -> Self {
        Self {
            head: NO_CHUNK,
            tail: NO_CHUNK,
            total: 0,
        }
    }
}

struct ChunkPool {
    chunks: Vec<Chunk>,
    free: Vec<u32>,
}

impl Default for ChunkPool {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl ChunkPool {
    #[inline]
    fn alloc_chunk(&mut self) -> u32 {
        if let Some(id) = self.free.pop() {
            let c = &mut self.chunks[id as usize];
            c.next = NO_CHUNK;
            c.len = 0;
            id
        } else {
            let id = self.chunks.len() as u32;
            self.chunks.push(Chunk::default());
            id
        }
    }

    #[inline(always)]
    fn push(&mut self, list: &mut BucketList, p: Piece16) {
        if list.tail != NO_CHUNK {
            let tc = &mut self.chunks[list.tail as usize];
            let l = tc.len as usize;
            if l < CHUNK_CAP {
                tc.data[l] = p;
                tc.len = (l + 1) as u16;
                list.total += 1;
                return;
            }
        }
        let cid = self.alloc_chunk();
        let c = &mut self.chunks[cid as usize];
        c.data[0] = p;
        c.len = 1;
        if list.tail != NO_CHUNK {
            self.chunks[list.tail as usize].next = cid;
        } else {
            list.head = cid;
        }
        list.tail = cid;
        list.total += 1;
    }

    /// Reclaim all chunks of `list` immediately back to the hot LIFO free stack!
    #[inline]
    fn clear_list(&mut self, list: &mut BucketList) {
        let mut cur = list.head;
        while cur != NO_CHUNK {
            let nxt = self.chunks[cur as usize].next;
            self.free.push(cur);
            cur = nxt;
        }
        list.head = NO_CHUNK;
        list.tail = NO_CHUNK;
        list.total = 0;
    }

    /// Drain all pieces of `list` into `dst` and immediately recycle its chunks!
    #[inline]
    fn drain_into(&mut self, list: &mut BucketList, dst: &mut Vec<Piece16>) {
        dst.clear();
        dst.reserve(list.total as usize);
        let mut cur = list.head;
        while cur != NO_CHUNK {
            let c = &self.chunks[cur as usize];
            let l = c.len as usize;
            dst.extend_from_slice(&c.data[..l]);
            let nxt = c.next;
            self.free.push(cur);
            cur = nxt;
        }
        list.head = NO_CHUNK;
        list.tail = NO_CHUNK;
        list.total = 0;
    }

    fn capacity_bytes(&self) -> usize {
        self.chunks.capacity() * std::mem::size_of::<Chunk>()
            + self.free.capacity() * std::mem::size_of::<u32>()
    }
}

#[derive(Clone, Copy, Default, Debug)]
struct SlpNode {
    a: u32,   // left handle (or base handle if len has REP_BIT)
    b: u32,   // right handle (or repeat count k if len has REP_BIT)
    len: u32, // low 31 bits: total expanded byte length; bit 31: REP_BIT
}

impl SlpNode {
    #[inline(always)]
    fn byte_len(self) -> u32 {
        self.len & !REP_BIT
    }
    #[inline(always)]
    fn is_rep(self) -> bool {
        (self.len & REP_BIT) != 0
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct RunStats {
    pub r0: usize,
    pub r_init: usize,
    pub appends: usize,
    pub scans: usize,
    pub stale: usize,
    pub s1_1st_hop: usize,
    pub s1_self_lkp: usize,
    pub s1_subint_buckets: usize,
    pub s2_ops: usize,
    pub m_core: usize,
    pub slab16_len: usize,
    pub slp_len: usize,
    pub flat_q_bytes: usize,
    pub peak_scratch_bytes: usize,
    pub ns_s0: u64,
    pub ns_s1_bin: u64,
    pub ns_s1_ops: u64,
    pub ns_s2: u64,
    pub ns_s3_flat: u64,
    pub ns_s3_emit: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExecMode {
    DirectHubP16,
    CoreRvP16,
}

pub struct B4GridFlat16 {
    shipped: OpenCells,
    mode: ExecMode,

    // Stage 0 scratch
    max_runs: Vec<u32>,

    // Theorem B4 zero-tombstone single-copy buckets backed by ONE shared 1 KiB ChunkPool!
    pool: ChunkPool,
    in_at: [BucketList; 256],
    out_at: [BucketList; 256],
    self_at: [BucketList; 256],

    // Reusable per-bucket local buffers (L1d/L2 resident)
    in_buf: Vec<Piece16>,
    out_sorted: Vec<Piece16>,
    bin_cnt: Vec<u32>,
    bin_off: Vec<u32>,
    grid: Vec<u16>,
    self_q: Vec<Piece16>,
    tmp_out: Vec<Piece16>,
    sub_cur: Vec<Piece16>,
    sub_ext: Vec<Piece16>,
    sub_next: Vec<Piece16>,

    // Word operad tables
    slab16: Vec<u128>,
    slp: Vec<SlpNode>,

    // Stage 2 Core-RV scratch (used only in CoreRvP16)
    rv_thick: Vec<Piece16>,
    rv_thick_next: Vec<Piece16>,
    rv_thin: Vec<Piece16>,
    rv_wired: Vec<Piece16>,
    edges_core: Vec<u32>,
    words_core: Vec<u32>,
    row_to_core: Vec<u32>,

    // Stage 3 flat overflow buffer (len >= 16 only!)
    flat_q: Vec<u8>,
    flat_spans: Vec<(u32, u32)>,
    slp_stack: Vec<u32>,
    rot_tmp: Vec<u8>,

    pub last_stats: RunStats,
}

impl Default for B4GridFlat16 {
    fn default() -> Self {
        Self::with_mode(ExecMode::CoreRvP16)
    }
}

impl B4GridFlat16 {
    pub fn with_mode(mode: ExecMode) -> Self {
        Self {
            shipped: OpenCells::default(),
            mode,
            max_runs: Vec::new(),
            pool: ChunkPool::default(),
            in_at: [BucketList::default(); 256],
            out_at: [BucketList::default(); 256],
            self_at: [BucketList::default(); 256],
            in_buf: Vec::new(),
            out_sorted: Vec::new(),
            bin_cnt: Vec::new(),
            bin_off: Vec::new(),
            grid: Vec::new(),
            self_q: Vec::new(),
            tmp_out: Vec::new(),
            sub_cur: Vec::new(),
            sub_ext: Vec::new(),
            sub_next: Vec::new(),
            slab16: Vec::new(),
            slp: Vec::new(),
            rv_thick: Vec::new(),
            rv_thick_next: Vec::new(),
            rv_thin: Vec::new(),
            rv_wired: Vec::new(),
            edges_core: Vec::new(),
            words_core: Vec::new(),
            row_to_core: Vec::new(),
            flat_q: Vec::new(),
            flat_spans: Vec::new(),
            slp_stack: Vec::new(),
            rot_tmp: Vec::new(),
            last_stats: RunStats::default(),
        }
    }

    pub fn allocated_scratch_bytes(&self) -> usize {
        let mut tot = self.pool.capacity_bytes();
        tot += self.max_runs.capacity() * 4;
        tot += self.in_buf.capacity() * 16;
        tot += self.out_sorted.capacity() * 16;
        tot += self.bin_cnt.capacity() * 4;
        tot += self.bin_off.capacity() * 4;
        tot += self.grid.capacity() * 2;
        tot += self.self_q.capacity() * 16;
        tot += self.tmp_out.capacity() * 16;
        tot += self.sub_cur.capacity() * 16;
        tot += self.sub_ext.capacity() * 16;
        tot += self.sub_next.capacity() * 16;
        tot += self.slab16.capacity() * 16;
        tot += self.slp.capacity() * std::mem::size_of::<SlpNode>();
        tot += self.rv_thick.capacity() * 16;
        tot += self.rv_thick_next.capacity() * 16;
        tot += self.rv_thin.capacity() * 16;
        tot += self.rv_wired.capacity() * 16;
        tot += self.edges_core.capacity() * 4;
        tot += self.words_core.capacity() * 4;
        tot += self.row_to_core.capacity() * 4;
        tot += self.flat_q.capacity();
        tot += self.flat_spans.capacity() * 8;
        tot += self.slp_stack.capacity() * 4;
        tot += self.rot_tmp.capacity();
        tot
    }

    #[inline(always)]
    fn word_len(w: u32, slab16: &[u128], slp: &[SlpNode]) -> u32 {
        let top2 = w & TAG_MASK;
        if top2 != (3 << TAG_SHIFT) {
            (top2 >> TAG_SHIFT) + 1
        } else if (w & TAG_EXT_MASK) == TAG_SLAB16 {
            (slab16[(w & EXT_IDX_MASK) as usize] >> 120) as u32
        } else {
            slp[(w & EXT_IDX_MASK) as usize].byte_len()
        }
    }

    #[inline(always)]
    fn word_inline_bytes(w: u32, slab16: &[u128]) -> Option<(u128, u32)> {
        let top2 = w & TAG_MASK;
        if top2 != (3 << TAG_SHIFT) {
            Some(((w & 0x00FF_FFFF) as u128, (top2 >> TAG_SHIFT) + 1))
        } else if (w & TAG_EXT_MASK) == TAG_SLAB16 {
            let v = slab16[(w & EXT_IDX_MASK) as usize];
            Some((v & SLAB16_BYTES_MASK, (v >> 120) as u32))
        } else {
            None
        }
    }

    #[inline(always)]
    fn op_cat(hl: u32, hr: u32, slab16: &mut Vec<u128>, slp: &mut Vec<SlpNode>) -> u32 {
        if let (Some((bl, ll)), Some((br, lr))) =
            (Self::word_inline_bytes(hl, slab16), Self::word_inline_bytes(hr, slab16))
        {
            let tot = ll + lr;
            if tot <= 15 {
                let comb = bl | (br << (ll * 8));
                if tot <= 3 {
                    return ((tot - 1) << TAG_SHIFT) | (comb as u32);
                } else {
                    let idx = slab16.len() as u32;
                    slab16.push(comb | ((tot as u128) << 120));
                    return TAG_SLAB16 | idx;
                }
            }
        }
        let tot = Self::word_len(hl, slab16, slp) + Self::word_len(hr, slab16, slp);
        let idx = slp.len() as u32;
        slp.push(SlpNode {
            a: hl,
            b: hr,
            len: tot,
        });
        TAG_SLP | idx
    }

    #[inline(always)]
    fn op_rep(hb: u32, k: u32, slab16: &mut Vec<u128>, slp: &mut Vec<SlpNode>) -> u32 {
        if k == 1 {
            return hb;
        }
        if let Some((bb, lb)) = Self::word_inline_bytes(hb, slab16) {
            let tot = lb * k;
            if tot <= 15 {
                let mut comb = 0u128;
                let mut sh = 0u32;
                for _ in 0..k {
                    comb |= bb << sh;
                    sh += lb * 8;
                }
                if tot <= 3 {
                    return ((tot - 1) << TAG_SHIFT) | (comb as u32);
                } else {
                    let idx = slab16.len() as u32;
                    slab16.push(comb | ((tot as u128) << 120));
                    return TAG_SLAB16 | idx;
                }
            }
        }
        let tot = Self::word_len(hb, slab16, slp) * k;
        let idx = slp.len() as u32;
        slp.push(SlpNode {
            a: hb,
            b: k,
            len: tot | REP_BIT,
        });
        TAG_SLP | idx
    }

    fn expand_word_to_vec(
        w: u32,
        slab16: &[u128],
        slp: &[SlpNode],
        stack: &mut Vec<u32>,
        dst: &mut Vec<u8>,
    ) {
        stack.clear();
        stack.push(w);
        while let Some(cur) = stack.pop() {
            if let Some((b, l)) = Self::word_inline_bytes(cur, slab16) {
                let arr = b.to_le_bytes();
                dst.extend_from_slice(&arr[..l as usize]);
            } else {
                let node = slp[(cur & EXT_IDX_MASK) as usize];
                if node.is_rep() {
                    let k = node.b as usize;
                    let start = dst.len();
                    let mut sub_stack = Vec::new();
                    Self::expand_word_to_vec(node.a, slab16, slp, &mut sub_stack, dst);
                    let unit_len = dst.len() - start;
                    if k > 1 && unit_len > 0 {
                        dst.reserve(unit_len * (k - 1));
                        for _ in 1..k {
                            dst.extend_from_within(start..start + unit_len);
                        }
                    }
                } else {
                    stack.push(node.b);
                    stack.push(node.a);
                }
            }
        }
    }

    fn materialize_slp_word(
        slp_idx: usize,
        slab16: &[u128],
        slp: &[SlpNode],
        flat_q: &mut Vec<u8>,
        flat_spans: &mut [(u32, u32)],
        stack: &mut Vec<u32>,
    ) -> (u32, u32) {
        let existing = flat_spans[slp_idx];
        if existing.0 != u32::MAX {
            return existing;
        }
        let start = flat_q.len() as u32;
        stack.clear();
        stack.push(TAG_SLP | (slp_idx as u32));
        while let Some(cur) = stack.pop() {
            if let Some((b, l)) = Self::word_inline_bytes(cur, slab16) {
                let arr = b.to_le_bytes();
                flat_q.extend_from_slice(&arr[..l as usize]);
            } else {
                let idx = (cur & EXT_IDX_MASK) as usize;
                let sp = flat_spans[idx];
                if sp.0 != u32::MAX {
                    let s = sp.0 as usize;
                    flat_q.extend_from_within(s..s + sp.1 as usize);
                } else {
                    let node = slp[idx];
                    if node.is_rep() {
                        let k = node.b as usize;
                        let u_start = flat_q.len();
                        let mut sub = Vec::new();
                        Self::expand_word_to_vec(node.a, slab16, slp, &mut sub, flat_q);
                        let u_len = flat_q.len() - u_start;
                        if k > 1 && u_len > 0 {
                            flat_q.reserve(u_len * (k - 1));
                            for _ in 1..k {
                                flat_q.extend_from_within(u_start..u_start + u_len);
                            }
                        }
                    } else {
                        stack.push(node.b);
                        stack.push(node.a);
                    }
                }
            }
        }
        let len = (flat_q.len() as u32) - start;
        let span = (start, len);
        flat_spans[slp_idx] = span;
        span
    }

    /// Tight Shift-Bin-Grid: small buckets (k <= 64) sort in-place without grid overhead;
    /// larger buckets use L1d-compact G <= 4096 (<= 8 KiB grid!) with single-pass counting bin!
    fn build_shift_bin_grid_from_lists(
        pool: &mut ChunkPool,
        out_list: &mut BucketList,
        self_list: &mut BucketList,
        b_start: u32,
        b_len: u32,
        out_sorted: &mut Vec<Piece16>,
        bin_cnt: &mut Vec<u32>,
        bin_off: &mut Vec<u32>,
        grid: &mut Vec<u16>,
    ) -> u32 {
        let k = (out_list.total + self_list.total) as usize;
        out_sorted.clear();
        if k == 0 {
            grid.clear();
            return 0;
        }
        if k <= 64 {
            out_sorted.reserve(k);
            let mut cur = out_list.head;
            while cur != NO_CHUNK {
                let c = &pool.chunks[cur as usize];
                out_sorted.extend_from_slice(&c.data[..c.len as usize]);
                let nxt = c.next;
                pool.free.push(cur);
                cur = nxt;
            }
            *out_list = BucketList::default();
            let mut cur = self_list.head;
            while cur != NO_CHUNK {
                let c = &pool.chunks[cur as usize];
                out_sorted.extend_from_slice(&c.data[..c.len as usize]);
                let nxt = c.next;
                pool.free.push(cur);
                cur = nxt;
            }
            *self_list = BucketList::default();
            if k > 1 {
                out_sorted.sort_unstable_by_key(|p| p.src());
            }
            grid.clear();
            return 31; // signals tiny-bucket direct binary/linear lookup
        }

        let target_g = (k / 2).next_power_of_two().clamp(16, 4096) as u32;
        let mut shift = 0u32;
        if b_len > 1 {
            let max_off = b_len - 1;
            while (max_off >> shift) >= target_g {
                shift += 1;
            }
        }
        let grid_sz = (((b_len - 1) >> shift) + 1) as usize;

        bin_cnt.clear();
        bin_cnt.resize(grid_sz, 0);
        // Count pass over chunks
        for head in [out_list.head, self_list.head] {
            let mut cur = head;
            while cur != NO_CHUNK {
                let c = &pool.chunks[cur as usize];
                for p in &c.data[..c.len as usize] {
                    bin_cnt[((p.src() - b_start) >> shift) as usize] += 1;
                }
                cur = c.next;
            }
        }

        bin_off.clear();
        bin_off.reserve(grid_sz + 1);
        let mut acc = 0u32;
        for &c in bin_cnt.iter() {
            bin_off.push(acc);
            acc += c;
        }
        bin_off.push(acc);

        out_sorted.resize(k, Piece16::default());
        // Scatter pass + immediate chunk recycling!
        for list in [out_list, self_list] {
            let mut cur = list.head;
            while cur != NO_CHUNK {
                let c = &pool.chunks[cur as usize];
                for &p in &c.data[..c.len as usize] {
                    let gi = ((p.src() - b_start) >> shift) as usize;
                    let pos = bin_off[gi] as usize;
                    out_sorted[pos] = p;
                    bin_off[gi] = (pos + 1) as u32;
                }
                let nxt = c.next;
                pool.free.push(cur);
                cur = nxt;
            }
            *list = BucketList::default();
        }

        // In-bin insertion sort
        let mut start = 0usize;
        for &c in bin_cnt.iter() {
            let cnt = c as usize;
            if cnt > 1 {
                let end = start + cnt;
                for i in (start + 1)..end {
                    let val = out_sorted[i];
                    let vsrc = val.src();
                    let mut j = i;
                    while j > start && out_sorted[j - 1].src() > vsrc {
                        out_sorted[j] = out_sorted[j - 1];
                        j -= 1;
                    }
                    out_sorted[j] = val;
                }
            }
            start += cnt;
        }

        // Build L1d lookup grid (<= 8 KiB!)
        grid.clear();
        grid.resize(grid_sz, 0);
        let mut oi = 0usize;
        for gi in 0..grid_sz {
            let pos = b_start + ((gi as u32) << shift);
            while oi + 1 < k && out_sorted[oi].end() <= pos {
                oi += 1;
            }
            grid[gi] = oi as u16;
        }
        shift
    }

    #[inline(always)]
    fn grid_find_p16(out_sorted: &[Piece16], grid: &[u16], shift: u32, b_start: u32, pos: u32) -> usize {
        let k = out_sorted.len();
        if k <= 1 {
            return 0;
        }
        if grid.is_empty() {
            // k <= 64 tiny bucket: fast binary search in L1d!
            return out_sorted.partition_point(|x| x.src() <= pos) - 1;
        }
        let gi = ((pos - b_start) >> shift) as usize;
        let mut oi = grid[gi] as usize;
        while oi + 1 < k && out_sorted[oi].end() <= pos {
            oi += 1;
        }
        oi
    }

    #[inline]
    fn spiral_push_p16(
        s: u32,
        take: u32,
        u: u32,
        wid: u32,
        a_pos: u32,
        b_pos: u32,
        delta: i32,
        w_big: u32,
        sb: u8,
        dst_b_out: u8,
        slab16: &mut Vec<u128>,
        slp: &mut Vec<SlpNode>,
        out_list: &mut Vec<Piece16>,
    ) -> usize {
        debug_assert!(delta != 0);
        let mut ops = 0usize;
        let mut st_buf = [(s, take, u, wid); 16];
        let mut sp = 1usize;
        let mut heap_st: Vec<(u32, u32, u32, u32)> = Vec::new();

        while sp > 0 || !heap_st.is_empty() {
            let (ss, sl, su, sw) = if !heap_st.is_empty() {
                heap_st.pop().unwrap()
            } else {
                sp -= 1;
                st_buf[sp]
            };
            ops += 1;
            let sv = su + sl;
            if delta > 0 {
                let d = delta as u32;
                let k_max = (b_pos - 1 - su) / d + 1;
                let k_min = (b_pos - 1 - (sv - 1)) / d + 1;
                if k_max == k_min {
                    let rw = Self::op_rep(w_big, k_max, slab16, slp);
                    let nw = Self::op_cat(rw, sw, slab16, slp);
                    out_list.push(Piece16::new(ss, sl, su + k_max * d, sb, dst_b_out, nw));
                } else {
                    let split_x = b_pos - (k_max - 1) * d;
                    let l1 = split_x - su;
                    if l1 < sl {
                        let rem_item = (ss + l1, sl - l1, split_x, sw);
                        if sp < st_buf.len() {
                            st_buf[sp] = rem_item;
                            sp += 1;
                        } else {
                            heap_st.push(rem_item);
                        }
                    }
                    let rw = Self::op_rep(w_big, k_max, slab16, slp);
                    let nw = Self::op_cat(rw, sw, slab16, slp);
                    out_list.push(Piece16::new(ss, l1, su + k_max * d, sb, dst_b_out, nw));
                }
            } else {
                let step = (-delta) as u32;
                let k_min = (su - a_pos) / step + 1;
                let k_max = (sv - 1 - a_pos) / step + 1;
                if k_min == k_max {
                    let rw = Self::op_rep(w_big, k_min, slab16, slp);
                    let nw = Self::op_cat(rw, sw, slab16, slp);
                    out_list.push(Piece16::new(ss, sl, su - k_min * step, sb, dst_b_out, nw));
                } else {
                    let split_x = a_pos + k_min * step;
                    let l1 = split_x - su;
                    if l1 < sl {
                        let rem_item = (ss + l1, sl - l1, split_x, sw);
                        if sp < st_buf.len() {
                            st_buf[sp] = rem_item;
                            sp += 1;
                        } else {
                            heap_st.push(rem_item);
                        }
                    }
                    let rw = Self::op_rep(w_big, k_min, slab16, slp);
                    let nw = Self::op_cat(rw, sw, slab16, slp);
                    out_list.push(Piece16::new(ss, l1, su - k_min * step, sb, dst_b_out, nw));
                }
            }
        }
        ops
    }

    fn subinterval_eliminate_p16(
        cur_pieces: &mut Vec<Piece16>,
        ext_in: &mut Vec<Piece16>,
        next_buf: &mut Vec<Piece16>,
        e_sym: u8,
        slab16: &mut Vec<u128>,
        slp: &mut Vec<SlpNode>,
        out_res: &mut Vec<Piece16>,
    ) -> usize {
        let r_init_e = cur_pieces.len().max(8);
        let mut ops = 0usize;
        let mut boundary_mode = false;

        loop {
            cur_pieces.retain(|p| !(p.db() == e_sym && p.dst == p.src()));
            if cur_pieces.len() > 2 * r_init_e {
                boundary_mode = true;
            } else if boundary_mode && cur_pieces.len() <= (3 * r_init_e) / 2 {
                boundary_mode = false;
            }

            let mut best_i: Option<usize> = None;
            if boundary_mode {
                let mut best_src = u32::MAX;
                for (i, p) in cur_pieces.iter().enumerate() {
                    if p.db() == e_sym && p.src() < best_src {
                        best_src = p.src();
                        best_i = Some(i);
                    }
                }
            } else {
                let mut best_len = 0u32;
                let mut best_src = u32::MAX;
                for (i, p) in cur_pieces.iter().enumerate() {
                    if p.db() == e_sym {
                        let pl = p.len();
                        if pl > best_len || (pl == best_len && p.src() < best_src) {
                            best_len = pl;
                            best_src = p.src();
                            best_i = Some(i);
                        }
                    }
                }
            }
            let Some(bi) = best_i else { break };
            let big = cur_pieces.swap_remove(bi);
            let a_pos = big.src();
            let b_pos = a_pos + big.len();
            let delta = (big.dst as i32) - (a_pos as i32);
            let w_big = big.word;

            let route = |src_list: &mut Vec<Piece16>,
                         dst_list: &mut Vec<Piece16>,
                         slab16: &mut Vec<u128>,
                         slp: &mut Vec<SlpNode>,
                         ops: &mut usize| {
                dst_list.clear();
                for p in src_list.iter().copied() {
                    if p.db() != e_sym {
                        dst_list.push(p);
                        continue;
                    }
                    let mut s = p.src();
                    let mut l = p.len();
                    let mut d = p.dst;
                    let sb = p.sb();
                    let wid = p.word;
                    if d < a_pos {
                        let take = l.min(a_pos - d);
                        dst_list.push(Piece16::new(s, take, d, sb, e_sym, wid));
                        s += take;
                        d += take;
                        l -= take;
                    }
                    if l == 0 { continue; }
                    if d < b_pos {
                        let take = l.min(b_pos - d);
                        *ops += Self::spiral_push_p16(
                            s, take, d, wid, a_pos, b_pos, delta, w_big, sb, e_sym, slab16, slp,
                            dst_list,
                        );
                        s += take;
                        d += take;
                        l -= take;
                    }
                    if l > 0 {
                        dst_list.push(Piece16::new(s, l, d, sb, e_sym, wid));
                    }
                }
                std::mem::swap(src_list, dst_list);
            };
            route(cur_pieces, next_buf, slab16, slp, &mut ops);
            route(ext_in, next_buf, slab16, slp, &mut ops);
        }

        cur_pieces.sort_unstable_by_key(|p| p.src());
        out_res.clear();
        for p in ext_in.iter().copied() {
            let mut cs = p.src();
            let mut cd = p.dst;
            let mut rem = p.len();
            let sb = p.sb();
            let wid = p.word;
            while rem > 0 {
                ops += 1;
                let oi = cur_pieces.partition_point(|x| x.src() <= cd) - 1;
                let op = cur_pieces[oi];
                let off = cd - op.src();
                let take = rem.min(op.len() - off);
                let nw = Self::op_cat(op.word, wid, slab16, slp);
                out_res.push(Piece16::new(cs, take, op.dst + off, sb, op.db(), nw));
                cs += take;
                cd += take;
                rem -= take;
            }
        }
        ops
    }

    fn finalize_cycle_to_n(rot_out: &[u8], shift_bytes: usize, n: usize, out: &mut Vec<u8>) {
        let clen = rot_out.len();
        debug_assert!(clen > 0);
        let sh = shift_bytes % clen;
        out.clear();
        out.reserve(n);
        if clen == n {
            out.extend_from_slice(&rot_out[sh..]);
            out.extend_from_slice(&rot_out[..sh]);
            return;
        }
        let rem = n % clen;
        let first_off = if rem == 0 { 0 } else { clen - rem };
        if rem != 0 {
            for j in first_off..clen {
                out.push(rot_out[(sh + j) % clen]);
            }
        }
        let full = n / clen;
        for _ in 0..full {
            out.extend_from_slice(&rot_out[sh..]);
            out.extend_from_slice(&rot_out[..sh]);
        }
    }

    pub fn invert_runs_raw(&mut self, records: &[u16], n: usize, orig_ptr: u32, out: &mut Vec<u8>) -> bool {
        self.invert_runs_inner::<false>(records, n, orig_ptr, out)
    }

    pub fn invert_runs_profiled(&mut self, records: &[u16], n: usize, orig_ptr: u32, out: &mut Vec<u8>) -> bool {
        self.invert_runs_inner::<true>(records, n, orig_ptr, out)
    }

    fn invert_runs_inner<const PROFILE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        use std::time::Instant;
        let t0 = if PROFILE { Some(Instant::now()) } else { None };

        if n == 0 || (orig_ptr as usize) >= n || n > 900_000 {
            return false;
        }

        // Stage 0a: Merge capped records into maximal runs
        self.max_runs.clear();
        let mut counts = [0u32; 256];
        let mut run_in = [0u32; 256];
        let mut total_rows = 0usize;
        let mut cur_b: u8 = 0;
        let mut cur_l: u32 = 0;
        for &rec in records {
            let b = (rec >> 8) as u8;
            let l = (rec & 0xFF) as u32;
            if l == 0 {
                return false;
            }
            total_rows += l as usize;
            if cur_l == 0 {
                cur_b = b;
                cur_l = l;
            } else if b == cur_b {
                cur_l += l;
            } else {
                self.max_runs.push(((cur_b as u32) << 24) | cur_l);
                counts[cur_b as usize] += cur_l;
                run_in[cur_b as usize] += 1;
                cur_b = b;
                cur_l = l;
            }
        }
        if cur_l > 0 {
            self.max_runs.push(((cur_b as u32) << 24) | cur_l);
            counts[cur_b as usize] += cur_l;
            run_in[cur_b as usize] += 1;
        }
        if total_rows != n {
            return false;
        }
        let r0 = self.max_runs.len();

        let mut alpha = [0u8; 256];
        let mut num_active = 0usize;
        let mut c_off = [0u32; 256];
        let mut b_end = [0u32; 256];
        let mut acc = 0u32;
        for sym in 0..256usize {
            let cnt = counts[sym];
            if cnt > 0 {
                alpha[num_active] = sym as u8;
                num_active += 1;
                c_off[sym] = acc;
                acc += cnt;
                b_end[sym] = acc;
            }
        }
        if num_active == 1 {
            out.clear();
            out.resize(n, alpha[0]);
            return true;
        }

        // Static Theorem B4 elimination order: keep highest-frequency hub LAST
        let mut order = [0u8; 256];
        order[..num_active].copy_from_slice(&alpha[..num_active]);
        let mut hub_idx = 0usize;
        for i in 1..num_active {
            let c_i = counts[order[i] as usize];
            let c_h = counts[order[hub_idx] as usize];
            if c_i > c_h || (c_i == c_h && order[i] > order[hub_idx]) {
                hub_idx = i;
            }
        }
        order.swap(hub_idx, num_active - 1);
        order[..num_active - 1].sort_unstable_by_key(|&b| {
            let bi = b as usize;
            (run_in[bi], counts[bi], b)
        });
        let mut rank = [0u8; 256];
        for (rk, &b) in order[..num_active].iter().enumerate() {
            rank[b as usize] = rk as u8;
        }
        let hub = order[num_active - 1];

        // Clear any leftover chunks & word tables
        for b in 0..256 {
            if self.in_at[b].head != NO_CHUNK {
                self.pool.clear_list(&mut self.in_at[b]);
            }
            if self.out_at[b].head != NO_CHUNK {
                self.pool.clear_list(&mut self.out_at[b]);
            }
            if self.self_at[b].head != NO_CHUNK {
                self.pool.clear_list(&mut self.self_at[b]);
            }
        }
        self.slab16.clear();
        self.slp.clear();

        // Stage 0b: Route initial pieces ONCE via Theorem B4 into shared 1 KiB ChunkPool!
        let mut occ = c_off;
        let mut bi_ptr = 0usize;
        let mut pos = 0u32;
        let mut r_init = 0usize;
        for &mr in &self.max_runs {
            let db = (mr >> 24) as u8;
            let mut rem = mr & 0x00FF_FFFF;
            let word = TAG_LEN1 | (db as u32);
            while rem > 0 {
                let sb = alpha[bi_ptr];
                let avail = b_end[sb as usize] - pos;
                let take = rem.min(avail);
                let dst = occ[db as usize];
                occ[db as usize] = dst + take;
                let p = Piece16::new(pos, take, dst, sb, db, word);
                if sb == db {
                    self.pool.push(&mut self.self_at[sb as usize], p);
                } else if rank[sb as usize] < rank[db as usize] {
                    self.pool.push(&mut self.out_at[sb as usize], p);
                } else {
                    self.pool.push(&mut self.in_at[db as usize], p);
                }
                r_init += 1;
                pos += take;
                rem -= take;
                if pos == b_end[sb as usize] && bi_ptr + 1 < num_active {
                    bi_ptr += 1;
                }
            }
        }

        let mut stats = RunStats {
            r0,
            r_init,
            appends: r_init,
            ns_s0: if let Some(t) = t0 { t.elapsed().as_nanos() as u64 } else { 0 },
            ..RunStats::default()
        };

        let mut cur_p = orig_ptr;
        let mut cur_pb = {
            let mut found = alpha[0];
            for &b in &alpha[..num_active] {
                if c_off[b as usize] <= orig_ptr && orig_ptr < b_end[b as usize] {
                    found = b;
                    break;
                }
            }
            found
        };
        let mut shift_bytes: usize = 0;

        // Stage 1: Causal Theorem B4 Contraction
        for step_idx in 0..(num_active - 1) {
            let e = order[step_idx];
            let ei = e as usize;
            let b_start = c_off[ei];
            let b_len = counts[ei];

            let t_b0 = if PROFILE { Some(Instant::now()) } else { None };
            let k_out = (self.out_at[ei].total + self.self_at[ei].total) as usize;
            let k_in = self.in_at[ei].total as usize;
            let r_self_e = self.self_at[ei].total as usize;
            if PROFILE {
                stats.scans += k_out + k_in;
            }

            let shift = Self::build_shift_bin_grid_from_lists(
                &mut self.pool,
                &mut self.out_at[ei],
                &mut self.self_at[ei],
                b_start,
                b_len,
                &mut self.out_sorted,
                &mut self.bin_cnt,
                &mut self.bin_off,
                &mut self.grid,
            );
            if let Some(t) = t_b0 {
                stats.ns_s1_bin += t.elapsed().as_nanos() as u64;
            }

            // Track primary row cur_p if in bucket e
            if cur_pb == e {
                let start_p = cur_p;
                let mut cyc_hs: Vec<u32> = Vec::new();
                let p_budget = (4 * k_out).max(16);
                let mut p_steps = 0usize;
                let mut exited = false;
                let mut closed = false;
                let saved_shift = shift_bytes;

                while p_steps < p_budget {
                    let oi = Self::grid_find_p16(&self.out_sorted, &self.grid, shift, b_start, cur_p);
                    let op = self.out_sorted[oi];
                    let off = cur_p - op.src();
                    let odb = op.db();
                    if odb != e {
                        cur_p = op.dst + off;
                        cur_pb = odb;
                        shift_bytes += Self::word_len(op.word, &self.slab16, &self.slp) as usize;
                        exited = true;
                        break;
                    } else {
                        let a_pos = op.src();
                        let b_pos = a_pos + op.len();
                        let delta = (op.dst as i32) - (a_pos as i32);
                        if delta == 0 {
                            cyc_hs.push(op.word);
                            closed = true;
                            break;
                        }
                        let k_jump = if delta > 0 {
                            let d = delta as u32;
                            let k = (b_pos - 1 - cur_p) / d + 1;
                            if cur_p < start_p && start_p < cur_p + k * d && (start_p - cur_p) % d == 0 {
                                let kh = (start_p - cur_p) / d;
                                cyc_hs.push(Self::op_rep(op.word, kh, &mut self.slab16, &mut self.slp));
                                closed = true;
                                break;
                            }
                            cur_p += k * d;
                            k
                        } else {
                            let st = (-delta) as u32;
                            let k = (cur_p - a_pos) / st + 1;
                            if cur_p > start_p
                                && start_p > cur_p.saturating_sub(k * st)
                                && (cur_p - start_p) % st == 0
                            {
                                let kh = (cur_p - start_p) / st;
                                cyc_hs.push(Self::op_rep(op.word, kh, &mut self.slab16, &mut self.slp));
                                closed = true;
                                break;
                            }
                            cur_p -= k * st;
                            k
                        };
                        let rw = Self::op_rep(op.word, k_jump, &mut self.slab16, &mut self.slp);
                        cyc_hs.push(rw);
                        shift_bytes += Self::word_len(rw, &self.slab16, &self.slp) as usize;
                        p_steps += 1;
                        if cur_p == start_p {
                            closed = true;
                            break;
                        }
                    }
                }

                if !exited && !closed {
                    shift_bytes = saved_shift;
                    cur_p = start_p;
                    self.sub_cur.clear();
                    self.sub_ext.clear();
                    for &op in &self.out_sorted {
                        let s = op.src();
                        let l = op.len();
                        if s <= start_p && start_p < s + l {
                            if start_p > s {
                                self.sub_cur.push(Piece16::new(s, start_p - s, op.dst, e, op.db(), op.word));
                            }
                            self.sub_ext.push(Piece16::new(
                                start_p,
                                1,
                                op.dst + (start_p - s),
                                e,
                                op.db(),
                                op.word,
                            ));
                            if start_p + 1 < s + l {
                                self.sub_cur.push(Piece16::new(
                                    start_p + 1,
                                    s + l - (start_p + 1),
                                    op.dst + (start_p + 1 - s),
                                    e,
                                    op.db(),
                                    op.word,
                                ));
                            }
                        } else {
                            self.sub_cur.push(op);
                        }
                    }
                    let m_ops = Self::subinterval_eliminate_p16(
                        &mut self.sub_cur,
                        &mut self.sub_ext,
                        &mut self.sub_next,
                        e,
                        &mut self.slab16,
                        &mut self.slp,
                        &mut self.tmp_out,
                    );
                    if PROFILE {
                        stats.s1_self_lkp += m_ops;
                    }
                    if let Some(&mp) = self.tmp_out.first() {
                        if mp.db() == e && mp.dst == start_p {
                            cyc_hs.clear();
                            cyc_hs.push(mp.word);
                            closed = true;
                        } else {
                            cur_p = mp.dst;
                            cur_pb = mp.db();
                            shift_bytes += Self::word_len(mp.word, &self.slab16, &self.slp) as usize;
                        }
                    }
                }

                if closed {
                    self.rot_tmp.clear();
                    for &hw in cyc_hs.iter().rev() {
                        Self::expand_word_to_vec(hw, &self.slab16, &self.slp, &mut self.slp_stack, &mut self.rot_tmp);
                    }
                    Self::finalize_cycle_to_n(&self.rot_tmp, shift_bytes, n, out);
                    if PROFILE {
                        self.last_stats = stats;
                    }
                    return true;
                }
            }

            // Eliminate bucket e (draining in_at[e] into in_buf & recycling chunks immediately!)
            let t_o0 = if PROFILE { Some(Instant::now()) } else { None };
            self.pool.drain_into(&mut self.in_at[ei], &mut self.in_buf);

            if r_self_e == 0 {
                let out_sorted = &self.out_sorted;
                let grid = &self.grid;
                let slab16 = &mut self.slab16;
                let slp = &mut self.slp;
                let pool = &mut self.pool;
                let k_len = out_sorted.len();
                let mut hops = 0usize;

                for &p in &self.in_buf {
                    let mut cs = p.src();
                    let mut cd = p.dst;
                    let mut rem = p.len();
                    let sb = p.sb();
                    let wid = p.word;
                    let mut oi = Self::grid_find_p16(out_sorted, grid, shift, b_start, cd);
                    while rem > 0 {
                        let op = out_sorted[oi];
                        let off = cd - op.src();
                        let take = rem.min(op.len() - off);
                        let odb = op.db();
                        let nw = Self::op_cat(op.word, wid, slab16, slp);
                        let np = Piece16::new(cs, take, op.dst + off, sb, odb, nw);
                        if sb == odb {
                            pool.push(&mut self.self_at[sb as usize], np);
                        } else if rank[sb as usize] < rank[odb as usize] {
                            pool.push(&mut self.out_at[sb as usize], np);
                        } else {
                            pool.push(&mut self.in_at[odb as usize], np);
                        }
                        hops += 1;
                        cs += take;
                        cd += take;
                        rem -= take;
                        if oi + 1 < k_len {
                            oi += 1;
                        }
                    }
                }
                if PROFILE {
                    stats.s1_1st_hop += hops;
                    stats.appends += hops;
                }
            } else {
                let mut self_rows = 0u32;
                for &p in &self.out_sorted {
                    if p.db() == e {
                        self_rows += p.len();
                    }
                }
                let in_rows = b_len.saturating_sub(self_rows);
                let mut use_subint = r_self_e >= 2 && (in_rows == 0 || b_len > 4 * in_rows);
                let budget = 4 * (k_out + k_in).max(8);

                self.tmp_out.clear();
                self.self_q.clear();

                if !use_subint {
                    let out_sorted = &self.out_sorted;
                    let grid = &self.grid;
                    let slab16 = &mut self.slab16;
                    let slp = &mut self.slp;
                    let k_len = out_sorted.len();
                    let mut fast_1st = 0usize;
                    let mut fast_slf = 0usize;
                    let mut aborted = false;

                    for &p in &self.in_buf {
                        let mut cs = p.src();
                        let mut cd = p.dst;
                        let mut rem = p.len();
                        let sb = p.sb();
                        let wid = p.word;
                        let mut oi = Self::grid_find_p16(out_sorted, grid, shift, b_start, cd);
                        while rem > 0 {
                            let op = out_sorted[oi];
                            let off = cd - op.src();
                            let take = rem.min(op.len() - off);
                            let odb = op.db();
                            fast_1st += 1;
                            if odb != e {
                                let nw = Self::op_cat(op.word, wid, slab16, slp);
                                self.tmp_out.push(Piece16::new(cs, take, op.dst + off, sb, odb, nw));
                            } else {
                                let a_pos = op.src();
                                let b_pos = a_pos + op.len();
                                let delta = (op.dst as i32) - (a_pos as i32);
                                fast_slf += Self::spiral_push_p16(
                                    cs, take, cd, wid, a_pos, b_pos, delta, op.word, sb, e, slab16,
                                    slp, &mut self.self_q,
                                ) - 1;
                            }
                            cs += take;
                            cd += take;
                            rem -= take;
                            if oi + 1 < k_len {
                                oi += 1;
                            }
                        }
                        if fast_1st + fast_slf > budget {
                            aborted = true;
                            break;
                        }
                    }

                    while !aborted {
                        let Some(q) = self.self_q.pop() else { break };
                        let orig_s = q.src();
                        let l = q.len();
                        let cur_pos = q.dst;
                        let sb = q.sb();
                        let wid = q.word;
                        let oi = Self::grid_find_p16(out_sorted, grid, shift, b_start, cur_pos);
                        let op = out_sorted[oi];
                        let off = cur_pos - op.src();
                        let take = l.min(op.len() - off);
                        if take < l {
                            self.self_q.push(Piece16::new(
                                orig_s + take,
                                l - take,
                                cur_pos + take,
                                sb,
                                e,
                                wid,
                            ));
                        }
                        fast_slf += 1;
                        if fast_1st + fast_slf > budget {
                            aborted = true;
                            break;
                        }
                        let odb = op.db();
                        if odb != e {
                            let nw = Self::op_cat(op.word, wid, slab16, slp);
                            self.tmp_out.push(Piece16::new(orig_s, take, op.dst + off, sb, odb, nw));
                        } else {
                            let a_pos = op.src();
                            let b_pos = a_pos + op.len();
                            let delta = (op.dst as i32) - (a_pos as i32);
                            fast_slf += Self::spiral_push_p16(
                                orig_s, take, cur_pos, wid, a_pos, b_pos, delta, op.word, sb, e,
                                slab16, slp, &mut self.self_q,
                            ) - 1;
                        }
                    }

                    if aborted {
                        use_subint = true;
                    } else if PROFILE {
                        stats.s1_1st_hop += fast_1st;
                        stats.s1_self_lkp += fast_slf;
                    }
                }

                if use_subint {
                    if PROFILE {
                        stats.s1_subint_buckets += 1;
                    }
                    self.sub_cur.clear();
                    self.sub_cur.extend_from_slice(&self.out_sorted);
                    self.sub_ext.clear();
                    self.sub_ext.extend_from_slice(&self.in_buf);
                    let sub_ops = Self::subinterval_eliminate_p16(
                        &mut self.sub_cur,
                        &mut self.sub_ext,
                        &mut self.sub_next,
                        e,
                        &mut self.slab16,
                        &mut self.slp,
                        &mut self.tmp_out,
                    );
                    if PROFILE {
                        stats.s1_self_lkp += sub_ops;
                    }
                }

                for &np in &self.tmp_out {
                    let sb = np.sb();
                    let odb = np.db();
                    if sb == odb {
                        self.pool.push(&mut self.self_at[sb as usize], np);
                    } else if rank[sb as usize] < rank[odb as usize] {
                        self.pool.push(&mut self.out_at[sb as usize], np);
                    } else {
                        self.pool.push(&mut self.in_at[odb as usize], np);
                    }
                }
                if PROFILE {
                    stats.appends += self.tmp_out.len();
                }
            }

            if let Some(t) = t_o0 {
                stats.ns_s1_ops += t.elapsed().as_nanos() as u64;
            }
        }

        // ====================================================================
        // Stage 2 & Stage 3 on Surviving Hub `hub` (self_at[hub])!
        // ====================================================================
        let t_s2_0 = if PROFILE { Some(Instant::now()) } else { None };
        let hi = hub as usize;
        let hub_start = c_off[hi];
        let hub_len = counts[hi];
        let k_hub = self.self_at[hi].total as usize;
        if PROFILE {
            stats.scans += k_hub;
        }

        let hub_shift = Self::build_shift_bin_grid_from_lists(
            &mut self.pool,
            &mut self.out_at[hi],
            &mut self.self_at[hi],
            hub_start,
            hub_len,
            &mut self.out_sorted,
            &mut self.bin_cnt,
            &mut self.bin_off,
            &mut self.grid,
        );

        let has_thin = self.out_sorted.iter().any(|p| p.len() < 16);
        let has_thick = self.out_sorted.iter().any(|p| p.len() >= 16);

        if !has_thin && hub_len as usize > 8 * k_hub.max(16) {
            // Pure-thick low-r hub killer guard (AltRing, Low-r Padded-Ring, Fib, SymMidRun!)
            let exit_tag = if hub == 0 { 1u8 } else { 0u8 };
            self.sub_cur.clear();
            self.sub_ext.clear();
            for &p in &self.out_sorted {
                let s = p.src();
                let l = p.len();
                let d = p.dst;
                let w = p.word;
                if s <= cur_p && cur_p < s + l {
                    if cur_p > s {
                        self.sub_cur.push(Piece16::new(s, cur_p - s, d, hub, hub, w));
                    }
                    self.sub_ext.push(Piece16::new(cur_p, 1, d + (cur_p - s), hub, hub, w));
                    if cur_p + 1 < s + l {
                        self.sub_cur.push(Piece16::new(
                            cur_p + 1,
                            s + l - (cur_p + 1),
                            d + (cur_p + 1 - s),
                            hub,
                            hub,
                            w,
                        ));
                    }
                } else {
                    self.sub_cur.push(p);
                }
            }
            let tag_exit = |src_l: &mut Vec<Piece16>, dst_l: &mut Vec<Piece16>| {
                dst_l.clear();
                for &p in src_l.iter() {
                    let s = p.src();
                    let l = p.len();
                    let d = p.dst;
                    let sb = p.sb();
                    let w = p.word;
                    if d <= cur_p && cur_p < d + l {
                        if cur_p > d {
                            dst_l.push(Piece16::new(s, cur_p - d, d, sb, hub, w));
                        }
                        dst_l.push(Piece16::new(s + (cur_p - d), 1, cur_p, sb, exit_tag, w));
                        if cur_p + 1 < d + l {
                            dst_l.push(Piece16::new(
                                s + (cur_p + 1 - d),
                                d + l - (cur_p + 1),
                                cur_p + 1,
                                sb,
                                hub,
                                w,
                            ));
                        }
                    } else {
                        dst_l.push(p);
                    }
                }
                std::mem::swap(src_l, dst_l);
            };
            tag_exit(&mut self.sub_cur, &mut self.sub_next);
            tag_exit(&mut self.sub_ext, &mut self.sub_next);
            let root_w = if self.sub_ext[0].db() == exit_tag {
                self.sub_ext[0].word
            } else {
                let s2_ops = Self::subinterval_eliminate_p16(
                    &mut self.sub_cur,
                    &mut self.sub_ext,
                    &mut self.sub_next,
                    hub,
                    &mut self.slab16,
                    &mut self.slp,
                    &mut self.tmp_out,
                );
                stats.s2_ops += s2_ops;
                self.tmp_out[0].word
            };
            if let Some(t) = t_s2_0 {
                stats.ns_s2 = t.elapsed().as_nanos() as u64;
            }
            let t_e0 = if PROFILE { Some(Instant::now()) } else { None };
            self.rot_tmp.clear();
            Self::expand_word_to_vec(root_w, &self.slab16, &self.slp, &mut self.slp_stack, &mut self.rot_tmp);
            Self::finalize_cycle_to_n(&self.rot_tmp, shift_bytes, n, out);
            if PROFILE {
                stats.m_core = 1;
                stats.slab16_len = self.slab16.len();
                stats.slp_len = self.slp.len();
                stats.ns_s3_emit = t_e0.unwrap().elapsed().as_nanos() as u64;
                stats.peak_scratch_bytes = self.allocated_scratch_bytes();
                self.last_stats = stats;
            }
            return true;
        }

        if self.mode == ExecMode::CoreRvP16 && has_thick && has_thin {
            return self.finish_core_rv_p16::<PROFILE>(
                hub,
                hub_start,
                hub_len,
                cur_p,
                shift_bytes,
                n,
                out,
                t_s2_0,
                stats,
            );
        }

        if let Some(t) = t_s2_0 {
            stats.ns_s2 = t.elapsed().as_nanos() as u64;
        }

        // Stage 3a: Materialize ONLY overflow len >= 16 words referenced by out_sorted!
        let t_f0 = if PROFILE { Some(Instant::now()) } else { None };
        self.flat_q.clear();
        self.flat_spans.clear();
        self.flat_spans.resize(self.slp.len(), (u32::MAX, 0));
        for p in &self.out_sorted {
            let w = p.word;
            if (w & TAG_EXT_MASK) == TAG_SLP {
                let idx = (w & EXT_IDX_MASK) as usize;
                Self::materialize_slp_word(
                    idx,
                    &self.slab16,
                    &self.slp,
                    &mut self.flat_q,
                    &mut self.flat_spans,
                    &mut self.slp_stack,
                );
            }
        }
        let t_f1 = if PROFILE {
            let now = Instant::now();
            stats.ns_s3_flat = (now - t_f0.unwrap()).as_nanos() as u64;
            stats.flat_q_bytes = self.flat_q.len();
            stats.slab16_len = self.slab16.len();
            stats.slp_len = self.slp.len();
            stats.m_core = hub_len as usize;
            Some(now)
        } else {
            None
        };

        // Stage 3b: Direct Single-Pass Backward Emit into out[0..n]!
        out.resize(n, 0);
        let init_wp = {
            let sh = shift_bytes % n;
            if sh == 0 { n } else { n - sh }
        };
        let mut wp = init_wp;
        let mut cur = cur_p;
        let mut bytes_written = 0usize;
        let out_sorted = &self.out_sorted;
        let grid = &self.grid;
        let slab16 = &self.slab16;
        let flat_q = &self.flat_q;
        let flat_spans = &self.flat_spans;
        let k_len = out_sorted.len();
        let has_grid = !grid.is_empty();

        loop {
            let oi = if k_len <= 1 {
                0
            } else if has_grid {
                let mut i = grid[((cur - hub_start) >> hub_shift) as usize] as usize;
                while i + 1 < k_len && out_sorted[i].end() <= cur {
                    i += 1;
                }
                i
            } else {
                out_sorted.partition_point(|x| x.src() <= cur) - 1
            };
            let p = out_sorted[oi];
            let next_cur = p.dst + (cur - p.src());
            let w = p.word;

            let top2 = w & TAG_MASK;
            if top2 != (3 << TAG_SHIFT) {
                let wl = ((top2 >> TAG_SHIFT) + 1) as usize;
                let arr = w.to_le_bytes();
                write_backward_wrap(out, &mut wp, &arr[..wl], n);
                bytes_written += wl;
            } else if (w & TAG_EXT_MASK) == TAG_SLAB16 {
                let v = slab16[(w & EXT_IDX_MASK) as usize];
                let wl = (v >> 120) as usize;
                let arr = v.to_le_bytes();
                write_backward_wrap(out, &mut wp, &arr[..wl], n);
                bytes_written += wl;
            } else {
                let (foff, flen) = flat_spans[(w & EXT_IDX_MASK) as usize];
                let s = foff as usize;
                let wl = flen as usize;
                write_backward_wrap(out, &mut wp, &flat_q[s..s + wl], n);
                bytes_written += wl;
            }

            cur = next_cur;
            if cur == cur_p {
                break;
            }
        }

        if bytes_written < n {
            self.rot_tmp.clear();
            self.rot_tmp.reserve(bytes_written);
            if wp < init_wp {
                self.rot_tmp.extend_from_slice(&out[wp..init_wp]);
            } else {
                self.rot_tmp.extend_from_slice(&out[wp..n]);
                self.rot_tmp.extend_from_slice(&out[..init_wp]);
            }
            Self::finalize_cycle_to_n(&self.rot_tmp, shift_bytes, n, out);
        }

        if let Some(t) = t_f1 {
            stats.ns_s3_emit = t.elapsed().as_nanos() as u64;
            stats.peak_scratch_bytes = self.allocated_scratch_bytes();
            self.last_stats = stats;
        }
        true
    }

    fn finish_core_rv_p16<const PROFILE: bool>(
        &mut self,
        hub: u8,
        hub_start: u32,
        hub_len: u32,
        mut cur_p: u32,
        mut shift_bytes: usize,
        n: usize,
        out: &mut Vec<u8>,
        t_s2_0: Option<std::time::Instant>,
        mut stats: RunStats,
    ) -> bool {
        use std::time::Instant;
        const CORE_LIMIT: u32 = 16;

        self.rv_thick.clear();
        self.rv_thin.clear();
        for &p in &self.out_sorted {
            if p.len() >= CORE_LIMIT {
                self.rv_thick.push(p);
            } else {
                self.rv_thin.push(p);
            }
        }

        let budget_s2 = (8 * (self.rv_thick.len() + self.rv_thin.len())).max(64);
        let mut dbl_ops = 0usize;
        let mut completed = false;
        self.sub_cur.clear();
        self.sub_cur.extend_from_slice(&self.rv_thick);

        for _ in 0..12 {
            // rv_thick is ALREADY sorted on round 0; sort on subsequent rounds!
            if dbl_ops > 0 {
                self.rv_thick.sort_unstable_by_key(|p| p.src());
            }
            // Build compact Shift-Grid over rv_thick (L1d!) to avoid binary search!
            let tk = self.rv_thick.len();
            let t_shift = if tk > 16 && hub_len > 1 {
                let target_g = (tk / 2).next_power_of_two().clamp(16, 2048) as u32;
                let mut sh = 0u32;
                let max_off = hub_len - 1;
                while (max_off >> sh) >= target_g {
                    sh += 1;
                }
                let gsz = (((hub_len - 1) >> sh) + 1) as usize;
                self.grid.clear();
                self.grid.resize(gsz, 0);
                let mut oi = 0usize;
                for gi in 0..gsz {
                    let pos = hub_start + ((gi as u32) << sh);
                    while oi < tk && self.rv_thick[oi].end() <= pos {
                        oi += 1;
                    }
                    self.grid[gi] = oi as u16;
                }
                sh
            } else {
                self.grid.clear();
                31
            };
            let has_tg = !self.grid.is_empty();

            self.rv_thick_next.clear();
            let mut any_moved = false;
            for &p in &self.rv_thick {
                let mut cs = p.src();
                let mut cd = p.dst;
                let mut rem = p.len();
                let w = p.word;
                while rem > 0 {
                    let mut ti = if has_tg {
                        let mut i = self.grid[((cd - hub_start) >> t_shift) as usize] as usize;
                        while i < tk && self.rv_thick[i].end() <= cd {
                            i += 1;
                        }
                        i
                    } else {
                        let pi = self.rv_thick.partition_point(|x| x.src() <= cd);
                        if pi > 0 && cd < self.rv_thick[pi - 1].end() { pi - 1 } else { pi }
                    };
                    if ti < tk && self.rv_thick[ti].src() <= cd && cd < self.rv_thick[ti].end() {
                        let tp = self.rv_thick[ti];
                        let off = cd - tp.src();
                        let take = rem.min(tp.len() - off);
                        if tp.dst == tp.src() {
                            dbl_ops = budget_s2 + 1;
                            break;
                        }
                        let nw = Self::op_cat(tp.word, w, &mut self.slab16, &mut self.slp);
                        self.rv_thick_next.push(Piece16::new(cs, take, tp.dst + off, hub, hub, nw));
                        any_moved = true;
                        dbl_ops += 1;
                        cs += take;
                        cd += take;
                        rem -= take;
                    } else {
                        while ti < tk && self.rv_thick[ti].src() <= cd {
                            ti += 1;
                        }
                        let take = if ti < tk {
                            rem.min(self.rv_thick[ti].src() - cd)
                        } else {
                            rem
                        };
                        self.rv_thick_next.push(Piece16::new(cs, take, cd, hub, hub, w));
                        cs += take;
                        cd += take;
                        rem -= take;
                    }
                    if dbl_ops > budget_s2 {
                        break;
                    }
                }
                if dbl_ops > budget_s2 {
                    break;
                }
            }
            if dbl_ops > budget_s2 {
                break;
            }
            std::mem::swap(&mut self.rv_thick, &mut self.rv_thick_next);
            if !any_moved {
                completed = true;
                break;
            }
        }

        self.rv_wired.clear();
        if completed {
            if PROFILE {
                stats.s2_ops += dbl_ops;
            }
            self.rv_thick.sort_unstable_by_key(|p| p.src());
            let pos_p = self.rv_thick.partition_point(|x| x.src() <= cur_p);
            if pos_p > 0 && cur_p < self.rv_thick[pos_p - 1].end() {
                let tp = self.rv_thick[pos_p - 1];
                cur_p = tp.dst + (cur_p - tp.src());
                shift_bytes += Self::word_len(tp.word, &self.slab16, &self.slp) as usize;
            }
            // Build grid on final rv_thick to wire rv_thin fast!
            let tk = self.rv_thick.len();
            let t_shift = if tk > 16 && hub_len > 1 {
                let target_g = (tk / 2).next_power_of_two().clamp(16, 2048) as u32;
                let mut sh = 0u32;
                let max_off = hub_len - 1;
                while (max_off >> sh) >= target_g {
                    sh += 1;
                }
                let gsz = (((hub_len - 1) >> sh) + 1) as usize;
                self.grid.clear();
                self.grid.resize(gsz, 0);
                let mut oi = 0usize;
                for gi in 0..gsz {
                    let pos = hub_start + ((gi as u32) << sh);
                    while oi < tk && self.rv_thick[oi].end() <= pos {
                        oi += 1;
                    }
                    self.grid[gi] = oi as u16;
                }
                sh
            } else {
                self.grid.clear();
                31
            };
            let has_tg = !self.grid.is_empty();

            for &p in &self.rv_thin {
                let mut cs = p.src();
                let mut cd = p.dst;
                let mut rem = p.len();
                let w = p.word;
                while rem > 0 {
                    let mut ti = if has_tg {
                        let mut i = self.grid[((cd - hub_start) >> t_shift) as usize] as usize;
                        while i < tk && self.rv_thick[i].end() <= cd {
                            i += 1;
                        }
                        i
                    } else {
                        let pi = self.rv_thick.partition_point(|x| x.src() <= cd);
                        if pi > 0 && cd < self.rv_thick[pi - 1].end() { pi - 1 } else { pi }
                    };
                    if ti < tk && self.rv_thick[ti].src() <= cd && cd < self.rv_thick[ti].end() {
                        let tp = self.rv_thick[ti];
                        let off = cd - tp.src();
                        let take = rem.min(tp.len() - off);
                        let nw = Self::op_cat(tp.word, w, &mut self.slab16, &mut self.slp);
                        self.rv_wired.push(Piece16::new(cs, take, tp.dst + off, hub, hub, nw));
                        if PROFILE { stats.s2_ops += 1; }
                        cs += take;
                        cd += take;
                        rem -= take;
                    } else {
                        while ti < tk && self.rv_thick[ti].src() <= cd {
                            ti += 1;
                        }
                        let take = if ti < tk { rem.min(self.rv_thick[ti].src() - cd) } else { rem };
                        self.rv_wired.push(Piece16::new(cs, take, cd, hub, hub, w));
                        if PROFILE { stats.s2_ops += 1; }
                        cs += take;
                        cd += take;
                        rem -= take;
                    }
                }
            }
        } else {
            // Fallback: Sub-Interval 1-Piece Schur with cur_p demoted to thin!
            let thin_tag = 0u8;
            let thick_tag = SUB_THICK_TAG;
            let mut dem_thick: Vec<Piece16> = Vec::with_capacity(self.sub_cur.len() + 1);
            let mut dem_thin: Vec<Piece16> = self.rv_thin.clone();
            for &p in &self.sub_cur {
                let s = p.src();
                let l = p.len();
                if s <= cur_p && cur_p < s + l {
                    if cur_p > s {
                        dem_thick.push(Piece16::new(s, cur_p - s, p.dst, thick_tag, thick_tag, p.word));
                    }
                    dem_thin.push(Piece16::new(cur_p, 1, p.dst + (cur_p - s), thin_tag, thin_tag, p.word));
                    if cur_p + 1 < s + l {
                        dem_thick.push(Piece16::new(
                            cur_p + 1,
                            s + l - (cur_p + 1),
                            p.dst + (cur_p + 1 - s),
                            thick_tag,
                            thick_tag,
                            p.word,
                        ));
                    }
                } else {
                    dem_thick.push(Piece16::new(s, l, p.dst, thick_tag, thick_tag, p.word));
                }
            }
            dem_thick.sort_unstable_by_key(|p| p.src());
            let tag_list = |plist: &[Piece16], sb_tag: u8, out_l: &mut Vec<Piece16>| {
                out_l.clear();
                for &p in plist {
                    let mut cs = p.src();
                    let mut cd = p.dst;
                    let mut rem = p.len();
                    let w = p.word;
                    while rem > 0 {
                        let pi = dem_thick.partition_point(|x| x.src() <= cd);
                        if pi > 0 && cd < dem_thick[pi - 1].end() {
                            let take = rem.min(dem_thick[pi - 1].end() - cd);
                            out_l.push(Piece16::new(cs, take, cd, sb_tag, thick_tag, w));
                            cs += take;
                            cd += take;
                            rem -= take;
                        } else {
                            let take = if pi < dem_thick.len() { rem.min(dem_thick[pi].src() - cd) } else { rem };
                            out_l.push(Piece16::new(cs, take, cd, sb_tag, thin_tag, w));
                            cs += take;
                            cd += take;
                            rem -= take;
                        }
                    }
                }
            };
            tag_list(&dem_thick, thick_tag, &mut self.sub_cur);
            let mut ext_all = Vec::new();
            tag_list(&dem_thin, thin_tag, &mut ext_all);
            self.sub_ext.clear();
            for p in ext_all {
                if p.db() == thick_tag {
                    self.sub_ext.push(p);
                } else {
                    self.rv_wired.push(p);
                }
            }
            let sub2_ops = Self::subinterval_eliminate_p16(
                &mut self.sub_cur,
                &mut self.sub_ext,
                &mut self.sub_next,
                thick_tag,
                &mut self.slab16,
                &mut self.slp,
                &mut self.tmp_out,
            );
            if PROFILE { stats.s2_ops += sub2_ops; }
            self.rv_wired.extend_from_slice(&self.tmp_out);
        }

        // Dense-index m_core into L1d/L2 edges_core + words_core!
        self.rv_wired.sort_unstable_by_key(|p| p.src());
        let m_core: usize = self.rv_wired.iter().map(|p| p.len() as usize).sum();
        self.row_to_core.clear();
        self.row_to_core.resize(hub_len as usize, 0);
        let mut idx_acc = 0u32;
        for &p in &self.rv_wired {
            let s0 = (p.src() - hub_start) as usize;
            let l = p.len() as usize;
            for off in 0..l {
                self.row_to_core[s0 + off] = idx_acc;
                idx_acc += 1;
            }
        }
        self.edges_core.clear();
        self.edges_core.reserve(m_core);
        self.words_core.clear();
        self.words_core.reserve(m_core);
        for &p in &self.rv_wired {
            let d0 = (p.dst - hub_start) as usize;
            let l = p.len() as usize;
            let w = p.word;
            for off in 0..l {
                self.edges_core.push(self.row_to_core[d0 + off]);
                self.words_core.push(w);
            }
        }
        let p_core = self.row_to_core[(cur_p - hub_start) as usize];
        if let Some(t) = t_s2_0 {
            stats.ns_s2 = t.elapsed().as_nanos() as u64;
        }

        // Stage 3a: Flatten overflow words referenced by rv_wired
        let t_f0 = if PROFILE { Some(Instant::now()) } else { None };
        self.flat_q.clear();
        self.flat_spans.clear();
        self.flat_spans.resize(self.slp.len(), (u32::MAX, 0));
        for &p in &self.rv_wired {
            let w = p.word;
            if (w & TAG_EXT_MASK) == TAG_SLP {
                let idx = (w & EXT_IDX_MASK) as usize;
                Self::materialize_slp_word(
                    idx,
                    &self.slab16,
                    &self.slp,
                    &mut self.flat_q,
                    &mut self.flat_spans,
                    &mut self.slp_stack,
                );
            }
        }
        let t_f1 = if PROFILE {
            let now = Instant::now();
            stats.ns_s3_flat = (now - t_f0.unwrap()).as_nanos() as u64;
            stats.flat_q_bytes = self.flat_q.len();
            stats.slab16_len = self.slab16.len();
            stats.slp_len = self.slp.len();
            stats.m_core = m_core;
            Some(now)
        } else {
            None
        };

        // Stage 3b: Tiny L1d/L2 Walk on edges_core (10.7k-29.1k steps)!
        out.resize(n, 0);
        let init_wp = {
            let sh = shift_bytes % n;
            if sh == 0 { n } else { n - sh }
        };
        let mut wp = init_wp;
        let mut cur = p_core;
        let mut bytes_written = 0usize;
        let edges = &self.edges_core;
        let words = &self.words_core;
        let slab16 = &self.slab16;
        let flat_q = &self.flat_q;
        let flat_spans = &self.flat_spans;

        loop {
            let u = cur as usize;
            let next_cur = edges[u];
            let w = words[u];
            let top2 = w & TAG_MASK;
            if top2 != (3 << TAG_SHIFT) {
                let wl = ((top2 >> TAG_SHIFT) + 1) as usize;
                let arr = w.to_le_bytes();
                write_backward_wrap(out, &mut wp, &arr[..wl], n);
                bytes_written += wl;
            } else if (w & TAG_EXT_MASK) == TAG_SLAB16 {
                let v = slab16[(w & EXT_IDX_MASK) as usize];
                let wl = (v >> 120) as usize;
                let arr = v.to_le_bytes();
                write_backward_wrap(out, &mut wp, &arr[..wl], n);
                bytes_written += wl;
            } else {
                let (foff, flen) = flat_spans[(w & EXT_IDX_MASK) as usize];
                let s = foff as usize;
                let wl = flen as usize;
                write_backward_wrap(out, &mut wp, &flat_q[s..s + wl], n);
                bytes_written += wl;
            }
            cur = next_cur;
            if cur == p_core {
                break;
            }
        }

        if bytes_written < n {
            self.rot_tmp.clear();
            self.rot_tmp.reserve(bytes_written);
            if wp < init_wp {
                self.rot_tmp.extend_from_slice(&out[wp..init_wp]);
            } else {
                self.rot_tmp.extend_from_slice(&out[wp..n]);
                self.rot_tmp.extend_from_slice(&out[..init_wp]);
            }
            Self::finalize_cycle_to_n(&self.rot_tmp, shift_bytes, n, out);
        }

        if let Some(t) = t_f1 {
            stats.ns_s3_emit = t.elapsed().as_nanos() as u64;
            stats.peak_scratch_bytes = self.allocated_scratch_bytes();
            self.last_stats = stats;
        }
        true
    }
}

#[inline(always)]
fn write_backward_wrap(out: &mut [u8], wp: &mut usize, src: &[u8], n: usize) {
    let wl = src.len();
    if *wp >= wl {
        let start = *wp - wl;
        out[start..*wp].copy_from_slice(src);
        *wp = if start == 0 { n } else { start };
    } else {
        let head = wl - *wp;
        if *wp > 0 {
            out[0..*wp].copy_from_slice(&src[head..]);
        }
        let tail_start = n - head;
        out[tail_start..n].copy_from_slice(&src[..head]);
        *wp = tail_start;
    }
}

impl InverseBwt for B4GridFlat16 {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        self.shipped.invert(ll, counts, orig_ptr, out)
    }

    fn wants_checkpoints(&self) -> bool {
        self.shipped.wants_checkpoints()
    }

    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        orig_ptr: u32,
        out: &mut Vec<u8>,
    ) {
        self.shipped
            .invert_checkpointed(ll, counts, checkpoints, orig_ptr, out)
    }

    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        self.shipped.invert_in_place(ll, counts, orig_ptr)
    }

    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(
        &mut self,
        r: &Runs,
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> bool {
        let recs = r.records();
        if n == 0 || n > 900_000 || !under_density(recs.len(), n) {
            return self.shipped.invert_runs(r, n, orig_ptr, out, k);
        }
        self.invert_runs_raw(recs, n, orig_ptr, out)
    }
}
