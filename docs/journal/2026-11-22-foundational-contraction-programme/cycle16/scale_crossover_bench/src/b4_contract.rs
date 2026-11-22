#![forbid(unsafe_code)]
//! B4_Contract: Compositional O(r_0)-state Bucket-Schur / Redheffer Contraction
//! -> O(k_hub) Doubly-Linked Hub RV -> Single Root SLP -> Memoized Sequential Expansion!
//!
//! NEVER allocates O(n) per-row edges. Supports arbitrary n up to 2 GiB (full u32 coordinates).

use std::time::Instant;

const TAG_SHIFT: u32 = 30;
const TAG_MASK: u32 = 3 << TAG_SHIFT;
const TAG_EXT_MASK: u32 = 7 << 29;
const TAG_SLAB16: u32 = 6 << 29;
const TAG_SLP: u32 = 7 << 29;
const EXT_IDX_MASK: u32 = (1 << 29) - 1;
const SLAB16_BYTES_MASK: u128 = (1u128 << 120) - 1;
const REP_BIT: u32 = 1 << 31;

const CHUNK_CAP: usize = 64;
const NO_CHUNK: u32 = u32::MAX;

#[derive(Clone, Copy, Default, Debug)]
struct Piece {
    src: u32,
    len: u32,
    dst: u32,
    word: u32,
    sb: u8,
    db: u8,
}

impl Piece {
    #[inline(always)]
    fn new(src: u32, len: u32, dst: u32, sb: u8, db: u8, word: u32) -> Self {
        Self {
            src,
            len,
            dst,
            word,
            sb,
            db,
        }
    }
    #[inline(always)]
    fn end(self) -> u32 {
        self.src + self.len
    }
}

struct Chunk {
    next: u32,
    len: u16,
    data: [Piece; CHUNK_CAP],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            next: NO_CHUNK,
            len: 0,
            data: [Piece::default(); CHUNK_CAP],
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

#[derive(Default)]
struct ChunkPool {
    chunks: Vec<Chunk>,
    free: Vec<u32>,
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
    fn push(&mut self, list: &mut BucketList, p: Piece) {
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

    #[inline]
    fn drain_into(&mut self, list: &mut BucketList, dst: &mut Vec<Piece>) {
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
    a: u32,
    b: u32,
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
pub struct B4Stats {
    pub r0: usize,
    pub b4_ops: usize,
    pub hub_pieces: usize,
    pub hub_rv_ops: usize,
    pub slp_nodes: usize,
    pub slab16_len: usize,
    pub peak_kernel_bytes: usize,
    pub contract_ms: f64,
    pub expand_ms: f64,
    pub total_ms: f64,
}

/// Doubly-linked node for exact O(k_hub) Hub RV -> Root SLP
#[derive(Clone, Copy, Default)]
struct RvNode {
    prev: [u32; 2],
    next: [u32; 2],
    width: u32,
    word: u32,
}

pub struct B4Contract {
    pool: ChunkPool,
    out_at: [BucketList; 256],
    in_at: [BucketList; 256],
    self_at: [BucketList; 256],

    max_runs: Vec<(u32, u8)>,
    out_sorted: Vec<Piece>,
    in_buf: Vec<Piece>,
    tmp_out: Vec<Piece>,
    self_q: Vec<Piece>,
    sub_cur: Vec<Piece>,
    sub_ext: Vec<Piece>,
    sub_next: Vec<Piece>,
    bin_cnt: Vec<u32>,
    bin_off: Vec<u32>,
    grid: Vec<u32>,

    slab16: Vec<u128>,
    slp: Vec<SlpNode>,
    slp_memo: Vec<(u32, u32)>,
    slp_stack: Vec<u32>,
    rv_nodes: Vec<RvNode>,
    rv_img_ord: Vec<u32>,
    rot_tmp: Vec<u8>,
}

impl Default for B4Contract {
    fn default() -> Self {
        Self {
            pool: ChunkPool::default(),
            out_at: [BucketList::default(); 256],
            in_at: [BucketList::default(); 256],
            self_at: [BucketList::default(); 256],
            max_runs: Vec::new(),
            out_sorted: Vec::new(),
            in_buf: Vec::new(),
            tmp_out: Vec::new(),
            self_q: Vec::new(),
            sub_cur: Vec::new(),
            sub_ext: Vec::new(),
            sub_next: Vec::new(),
            bin_cnt: Vec::new(),
            bin_off: Vec::new(),
            grid: Vec::new(),
            slab16: Vec::new(),
            slp: Vec::new(),
            slp_memo: Vec::new(),
            slp_stack: Vec::new(),
            rv_nodes: Vec::new(),
            rv_img_ord: Vec::new(),
            rot_tmp: Vec::new(),
        }
    }
}

impl B4Contract {
    /// Peak kernel scratch memory (excluding the caller's `out` buffer; note `rot_tmp` is reported cleanly
    /// — for `shift_bytes == 0` we expand directly into `out`, else `rot_tmp` is an n-byte rotation buffer;
    /// wait: we can do an IN-PLACE `out.rotate_left(sh)` so `rot_tmp` is NOT EVEN NEEDED — 0 extra O(n) buffer!!).
    pub fn peak_kernel_bytes(&self) -> usize {
        self.pool.capacity_bytes()
            + self.max_runs.capacity() * std::mem::size_of::<(u32, u8)>()
            + (self.out_sorted.capacity()
                + self.in_buf.capacity()
                + self.tmp_out.capacity()
                + self.self_q.capacity()
                + self.sub_cur.capacity()
                + self.sub_ext.capacity()
                + self.sub_next.capacity())
                * std::mem::size_of::<Piece>()
            + (self.bin_cnt.capacity() + self.bin_off.capacity() + self.grid.capacity()) * 4
            + self.slab16.capacity() * 16
            + self.slp.capacity() * std::mem::size_of::<SlpNode>()
            + self.slp_memo.capacity() * 8
            + self.slp_stack.capacity() * 4
            + self.rv_nodes.capacity() * std::mem::size_of::<RvNode>()
            + self.rv_img_ord.capacity() * 4
            + self.rot_tmp.capacity()
    }

    #[inline(always)]
    fn word_inline_bytes(h: u32, slab16: &[u128]) -> Option<(u128, u32)> {
        if h == u32::MAX {
            return Some((0, 0));
        }
        let top2 = h & TAG_MASK;
        if top2 != TAG_MASK {
            let len = (top2 >> TAG_SHIFT) + 1;
            let bytes = (h & ((1u32 << (len * 8)) - 1)) as u128;
            Some((bytes, len))
        } else if (h & TAG_EXT_MASK) == TAG_SLAB16 {
            let v = slab16[(h & EXT_IDX_MASK) as usize];
            let len = (v >> 120) as u32;
            Some((v & SLAB16_BYTES_MASK, len))
        } else {
            None
        }
    }

    #[inline(always)]
    fn word_len(h: u32, slab16: &[u128], slp: &[SlpNode]) -> u32 {
        if h == u32::MAX {
            return 0;
        }
        let top2 = h & TAG_MASK;
        if top2 != TAG_MASK {
            (top2 >> TAG_SHIFT) + 1
        } else if (h & TAG_EXT_MASK) == TAG_SLAB16 {
            (slab16[(h & EXT_IDX_MASK) as usize] >> 120) as u32
        } else {
            slp[(h & EXT_IDX_MASK) as usize].byte_len()
        }
    }

    #[inline(always)]
    fn op_cat(hl: u32, hr: u32, slab16: &mut Vec<u128>, slp: &mut Vec<SlpNode>) -> u32 {
        if hl == u32::MAX {
            return hr;
        }
        if hr == u32::MAX {
            return hl;
        }
        if let (Some((bl, ll)), Some((br, lr))) = (
            Self::word_inline_bytes(hl, slab16),
            Self::word_inline_bytes(hr, slab16),
        ) {
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
        if k == 0 || hb == u32::MAX {
            return u32::MAX;
        }
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

    /// Memoized sequential SLP expansion directly into `dst` (ZERO extra O(n) buffers!).
    fn expand_memoized(
        root: u32,
        slab16: &[u128],
        slp: &[SlpNode],
        memo: &mut Vec<(u32, u32)>,
        stack: &mut Vec<u32>,
        dst: &mut Vec<u8>,
    ) {
        memo.clear();
        memo.resize(slp.len(), (u32::MAX, 0));
        Self::expand_rec(root, slab16, slp, memo, stack, dst);
    }

    fn expand_rec(
        w: u32,
        slab16: &[u128],
        slp: &[SlpNode],
        memo: &mut [(u32, u32)],
        stack: &mut Vec<u32>,
        dst: &mut Vec<u8>,
    ) {
        let base_sp = stack.len();
        stack.push(w);
        while stack.len() > base_sp {
            let cur = stack.pop().unwrap();
            if let Some((b, l)) = Self::word_inline_bytes(cur, slab16) {
                if l > 0 {
                    let arr = b.to_le_bytes();
                    dst.extend_from_slice(&arr[..l as usize]);
                }
            } else {
                let idx = (cur & EXT_IDX_MASK) as usize;
                let m = memo[idx];
                if m.0 != u32::MAX {
                    let s = m.0 as usize;
                    dst.extend_from_within(s..s + m.1 as usize);
                } else {
                    let node = slp[idx];
                    let start = dst.len() as u32;
                    let blen = node.byte_len();
                    if node.is_rep() {
                        let k = node.b as usize;
                        // Expand base once using our stack above current depth
                        Self::expand_rec(node.a, slab16, slp, memo, stack, dst);
                        let u_start = start as usize;
                        let u_len = dst.len() - u_start;
                        if k > 1 && u_len > 0 {
                            let target_len = u_len * k;
                            let mut cur_len = u_len;
                            while cur_len < target_len {
                                let take = cur_len.min(target_len - cur_len);
                                dst.extend_from_within(u_start..u_start + take);
                                cur_len += take;
                            }
                        }
                        memo[idx] = (start, blen);
                    } else {
                        // DAG LIFO invariant: node.a & node.b finish completely before anything deeper on stack pops!
                        memo[idx] = (start, blen);
                        stack.push(node.b);
                        stack.push(node.a);
                    }
                }
            }
        }
    }

    /// In-place cycle finalization: rotates `out` by `shift_bytes % clen` in O(n) with ZERO extra buffer!
    fn finalize_in_place(shift_bytes: usize, n: usize, out: &mut Vec<u8>) {
        let clen = out.len();
        debug_assert!(clen > 0);
        let sh = shift_bytes % clen;
        if sh != 0 {
            out.rotate_left(sh);
        }
        if out.len() > n {
            out.truncate(n);
            return;
        }
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

    fn build_shift_bin_grid(
        pool: &mut ChunkPool,
        out_list: &mut BucketList,
        self_list: &mut BucketList,
        b_start: u32,
        b_len: u32,
        out_sorted: &mut Vec<Piece>,
        bin_cnt: &mut Vec<u32>,
        bin_off: &mut Vec<u32>,
        grid: &mut Vec<u32>,
    ) -> u32 {
        let k = (out_list.total + self_list.total) as usize;
        out_sorted.clear();
        if k == 0 {
            grid.clear();
            return 0;
        }
        if k <= 64 {
            out_sorted.reserve(k);
            for list in [out_list, self_list] {
                let mut cur = list.head;
                while cur != NO_CHUNK {
                    let c = &pool.chunks[cur as usize];
                    out_sorted.extend_from_slice(&c.data[..c.len as usize]);
                    let nxt = c.next;
                    pool.free.push(cur);
                    cur = nxt;
                }
                *list = BucketList::default();
            }
            if k > 1 {
                out_sorted.sort_unstable_by_key(|p| p.src);
            }
            grid.clear();
            return 31;
        }

        let target_g = (k / 2).next_power_of_two().clamp(16, 16384) as u32;
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
        for head in [out_list.head, self_list.head] {
            let mut cur = head;
            while cur != NO_CHUNK {
                let c = &pool.chunks[cur as usize];
                for p in &c.data[..c.len as usize] {
                    bin_cnt[((p.src - b_start) >> shift) as usize] += 1;
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

        out_sorted.resize(k, Piece::default());
        for list in [out_list, self_list] {
            let mut cur = list.head;
            while cur != NO_CHUNK {
                let c = &pool.chunks[cur as usize];
                for &p in &c.data[..c.len as usize] {
                    let gi = ((p.src - b_start) >> shift) as usize;
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

        let mut start = 0usize;
        for &c in bin_cnt.iter() {
            let cnt = c as usize;
            if cnt > 1 {
                let sl = &mut out_sorted[start..start + cnt];
                for i in 1..cnt {
                    let key = sl[i];
                    let ks = key.src;
                    let mut j = i;
                    while j > 0 && sl[j - 1].src > ks {
                        sl[j] = sl[j - 1];
                        j -= 1;
                    }
                    sl[j] = key;
                }
            }
            start += cnt;
        }

        grid.clear();
        grid.resize(grid_sz, 0);
        let mut cur_i = 0usize;
        for gi in 0..grid_sz {
            let bin_start_row = b_start + ((gi as u32) << shift);
            while cur_i + 1 < k && out_sorted[cur_i].end() <= bin_start_row {
                cur_i += 1;
            }
            grid[gi] = cur_i as u32;
        }
        shift
    }

    #[inline(always)]
    fn grid_find(out_sorted: &[Piece], grid: &[u32], shift: u32, b_start: u32, x: u32) -> usize {
        if shift == 31 {
            let mut lo = 0usize;
            let mut hi = out_sorted.len();
            while lo < hi {
                let mid = (lo + hi) >> 1;
                if out_sorted[mid].src <= x {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            return lo - 1;
        }
        let mut oi = grid[((x - b_start) >> shift) as usize] as usize;
        let k = out_sorted.len();
        while oi + 1 < k && out_sorted[oi].end() <= x {
            oi += 1;
        }
        oi
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn spiral_push(
        s: u32,
        take: u32,
        u: u32,
        wid: u32,
        a_pos: u32,
        b_pos: u32,
        delta: i64,
        w_big: u32,
        sb: u8,
        dst_b_out: u8,
        slab16: &mut Vec<u128>,
        slp: &mut Vec<SlpNode>,
        out_list: &mut Vec<Piece>,
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
                    out_list.push(Piece::new(ss, sl, su + k_max * d, sb, dst_b_out, nw));
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
                    out_list.push(Piece::new(ss, l1, su + k_max * d, sb, dst_b_out, nw));
                }
            } else {
                let step = (-delta) as u32;
                let k_min = (su - a_pos) / step + 1;
                let k_max = (sv - 1 - a_pos) / step + 1;
                if k_min == k_max {
                    let rw = Self::op_rep(w_big, k_min, slab16, slp);
                    let nw = Self::op_cat(rw, sw, slab16, slp);
                    out_list.push(Piece::new(ss, sl, su - k_min * step, sb, dst_b_out, nw));
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
                    out_list.push(Piece::new(ss, l1, su - k_min * step, sb, dst_b_out, nw));
                }
            }
        }
        ops
    }

    fn subinterval_eliminate(
        cur_pieces: &mut Vec<Piece>,
        ext_in: &mut Vec<Piece>,
        next_buf: &mut Vec<Piece>,
        e_sym: u8,
        slab16: &mut Vec<u128>,
        slp: &mut Vec<SlpNode>,
        out_res: &mut Vec<Piece>,
    ) -> usize {
        let r_init_e = cur_pieces.len().max(8);
        let mut ops = 0usize;
        let mut boundary_mode = false;

        loop {
            cur_pieces.retain(|p| !(p.db == e_sym && p.dst == p.src));
            if cur_pieces.len() > 2 * r_init_e {
                boundary_mode = true;
            } else if boundary_mode && cur_pieces.len() <= (3 * r_init_e) / 2 {
                boundary_mode = false;
            }

            let mut best_i: Option<usize> = None;
            if boundary_mode {
                let mut best_src = u32::MAX;
                for (i, p) in cur_pieces.iter().enumerate() {
                    if p.db == e_sym && p.src < best_src {
                        best_src = p.src;
                        best_i = Some(i);
                    }
                }
            } else {
                let mut best_len = 0u32;
                let mut best_src = u32::MAX;
                for (i, p) in cur_pieces.iter().enumerate() {
                    if p.db == e_sym {
                        let pl = p.len;
                        if pl > best_len || (pl == best_len && p.src < best_src) {
                            best_len = pl;
                            best_src = p.src;
                            best_i = Some(i);
                        }
                    }
                }
            }
            let Some(bi) = best_i else { break };
            let big = cur_pieces.swap_remove(bi);
            let a_pos = big.src;
            let b_pos = a_pos + big.len;
            let delta = (big.dst as i64) - (a_pos as i64);
            let w_big = big.word;

            let route = |src_list: &mut Vec<Piece>,
                         dst_list: &mut Vec<Piece>,
                         slab16: &mut Vec<u128>,
                         slp: &mut Vec<SlpNode>,
                         ops: &mut usize| {
                dst_list.clear();
                for p in src_list.iter().copied() {
                    if p.db != e_sym {
                        dst_list.push(p);
                        continue;
                    }
                    let mut s = p.src;
                    let mut l = p.len;
                    let mut d = p.dst;
                    let sb = p.sb;
                    let wid = p.word;
                    if d < a_pos {
                        let take = l.min(a_pos - d);
                        dst_list.push(Piece::new(s, take, d, sb, e_sym, wid));
                        s += take;
                        d += take;
                        l -= take;
                    }
                    if l == 0 {
                        continue;
                    }
                    if d < b_pos {
                        let take = l.min(b_pos - d);
                        *ops += Self::spiral_push(
                            s, take, d, wid, a_pos, b_pos, delta, w_big, sb, e_sym, slab16, slp,
                            dst_list,
                        );
                        s += take;
                        d += take;
                        l -= take;
                    }
                    if l > 0 {
                        dst_list.push(Piece::new(s, l, d, sb, e_sym, wid));
                    }
                }
                std::mem::swap(src_list, dst_list);
            };
            route(cur_pieces, next_buf, slab16, slp, &mut ops);
            route(ext_in, next_buf, slab16, slp, &mut ops);
        }

        cur_pieces.sort_unstable_by_key(|p| p.src);
        out_res.clear();
        for p in ext_in.iter().copied() {
            let mut cs = p.src;
            let mut cd = p.dst;
            let mut rem = p.len;
            let sb = p.sb;
            let wid = p.word;
            while rem > 0 {
                ops += 1;
                let oi = cur_pieces.partition_point(|x| x.src <= cd) - 1;
                let op = cur_pieces[oi];
                let off = cd - op.src;
                let take = rem.min(op.len - off);
                let nw = Self::op_cat(op.word, wid, slab16, slp);
                out_res.push(Piece::new(cs, take, op.dst + off, sb, op.db, nw));
                cs += take;
                cd += take;
                rem -= take;
            }
        }
        ops
    }

    /// Exact O(k_hub) Doubly-Linked Rauzy-Veech on the final surviving hub `out_sorted`!
    /// Splits out the single marked row `cur_p`, eliminates all remaining pieces with spirals,
    /// and returns `(root_word_handle, rv_ops)`.
    fn hub_rv_to_root_slp(
        out_sorted: &[Piece],
        hub_start: u32,
        hub_len: u32,
        cur_p: u32,
        slab16: &mut Vec<u128>,
        slp: &mut Vec<SlpNode>,
        rv_nodes: &mut Vec<RvNode>,
        rv_img_ord: &mut Vec<u32>,
    ) -> (u32, usize) {
        let p_rel = cur_p - hub_start;
        // Build shifted strips (splitting out p_rel as the 1-row terminal!)
        let mut strips: Vec<(u32, u32, u32, u32)> = Vec::with_capacity(out_sorted.len() + 2);
        let mut mark_word = u32::MAX;
        for &p in out_sorted {
            let fr = p.src - hub_start;
            let to = p.dst - hub_start;
            let w = p.len;
            if fr <= p_rel && p_rel < fr + w {
                if fr < p_rel {
                    strips.push((fr, to, p_rel - fr, p.word));
                }
                mark_word = p.word;
                if p_rel + 1 < fr + w {
                    let off = p_rel + 1 - fr;
                    strips.push((p_rel + 1, to + off, w - off, p.word));
                }
            } else {
                strips.push((fr, to, w, p.word));
            }
        }
        let ns = strips.len();
        if ns == 0 {
            return (mark_word, 0);
        }

        // Sort strip indices by image `to`
        rv_img_ord.clear();
        rv_img_ord.extend(0..ns as u32);
        rv_img_ord.sort_unstable_by_key(|&i| strips[i as usize].1);

        rv_nodes.clear();
        rv_nodes.reserve(3 * ns + 4);
        rv_nodes.push(RvNode::default()); // sentinel 0
        for &(_, _, w, word) in &strips {
            rv_nodes.push(RvNode {
                prev: [0, 0],
                next: [0, 0],
                width: w,
                word,
            });
        }

        let append_to = |nodes: &mut Vec<RvNode>, list: &mut (u32, u32), id: u32, side: usize| {
            let (head, tail) = list;
            if *tail == 0 {
                *head = id;
            } else {
                nodes[*tail as usize].next[side] = id;
            }
            nodes[id as usize].prev[side] = *tail;
            *tail = id;
        };
        let close = |nodes: &mut Vec<RvNode>, side: usize, first: (u32, u32), second: (u32, u32)| {
            let mut prev = 0u32;
            for (head, tail) in [first, second] {
                if tail == 0 {
                    continue;
                }
                nodes[prev as usize].next[side] = head;
                nodes[head as usize].prev[side] = prev;
                prev = tail;
            }
            nodes[prev as usize].next[side] = 0;
            nodes[0].prev[side] = prev;
        };

        let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
        let mut x = 0u32;
        for (i, &(fr, _, w, _)) in strips.iter().enumerate() {
            if x < fr {
                let id = rv_nodes.len() as u32;
                rv_nodes.push(RvNode {
                    prev: [0, 0],
                    next: [0, 0],
                    width: fr - x,
                    word: u32::MAX,
                });
                append_to(rv_nodes, &mut order[0], id, 0);
                append_to(rv_nodes, &mut terminals[1], id, 1);
            }
            append_to(rv_nodes, &mut order[0], i as u32 + 1, 0);
            x = fr + w;
        }
        if x < hub_len {
            let id = rv_nodes.len() as u32;
            rv_nodes.push(RvNode {
                prev: [0, 0],
                next: [0, 0],
                width: hub_len - x,
                word: u32::MAX,
            });
            append_to(rv_nodes, &mut order[0], id, 0);
            append_to(rv_nodes, &mut terminals[1], id, 1);
        }

        x = 0;
        for &idx in rv_img_ord.iter() {
            let (_, to, w, _) = strips[idx as usize];
            if x < to {
                let id = rv_nodes.len() as u32;
                rv_nodes.push(RvNode {
                    prev: [0, 0],
                    next: [0, 0],
                    width: to - x,
                    word: u32::MAX,
                });
                append_to(rv_nodes, &mut order[1], id, 1);
                append_to(rv_nodes, &mut terminals[0], id, 0);
            }
            append_to(rv_nodes, &mut order[1], idx + 1, 1);
            x = to + w;
        }
        if x < hub_len {
            let id = rv_nodes.len() as u32;
            rv_nodes.push(RvNode {
                prev: [0, 0],
                next: [0, 0],
                width: hub_len - x,
                word: u32::MAX,
            });
            append_to(rv_nodes, &mut order[1], id, 1);
            append_to(rv_nodes, &mut terminals[0], id, 0);
        }

        for side in 0..2 {
            close(rv_nodes, side, terminals[side], order[side]);
        }

        // Exact spiral RV elimination loop!
        let mut left = hub_len;
        let mut rv_ops = 0usize;
        while left > 0 {
            rv_ops += 1;
            let x = rv_nodes[0].prev[0];
            let y = rv_nodes[0].prev[1];
            let aw = rv_nodes[x as usize].width;
            let bw = rv_nodes[y as usize].width;
            if x != y {
                if aw > bw && rv_nodes[y as usize].prev[1] == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    let head = Self::op_rep(rv_nodes[x as usize].word, q, slab16, slp);
                    let wnew = Self::op_cat(head, rv_nodes[y as usize].word, slab16, slp);
                    rv_nodes[y as usize].word = wnew;
                    rv_nodes[x as usize].width = aw - q * bw;
                    left -= q * bw;
                    continue;
                }
                if aw < bw && rv_nodes[x as usize].prev[0] == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = Self::op_rep(rv_nodes[y as usize].word, q, slab16, slp);
                    let wnew = Self::op_cat(rv_nodes[x as usize].word, tail, slab16, slp);
                    rv_nodes[x as usize].word = wnew;
                    rv_nodes[y as usize].width = bw - q * aw;
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                let px0 = rv_nodes[x as usize].prev[0];
                rv_nodes[px0 as usize].next[0] = 0;
                rv_nodes[0].prev[0] = px0;
                let px1 = rv_nodes[x as usize].prev[1];
                rv_nodes[px1 as usize].next[1] = 0;
                rv_nodes[0].prev[1] = px1;
                continue;
            }
            let wnew = Self::op_cat(rv_nodes[x as usize].word, rv_nodes[y as usize].word, slab16, slp);
            if aw <= bw {
                let px0 = rv_nodes[x as usize].prev[0];
                rv_nodes[px0 as usize].next[0] = 0;
                rv_nodes[0].prev[0] = px0;
                // insert x after y on side 0
                let ny0 = rv_nodes[y as usize].next[0];
                rv_nodes[x as usize].prev[0] = y;
                rv_nodes[x as usize].next[0] = ny0;
                rv_nodes[y as usize].next[0] = x;
                rv_nodes[ny0 as usize].prev[0] = x;
                rv_nodes[x as usize].word = wnew;
                if aw == bw {
                    // cut y from side 0, pop_tail y from side 1
                    let py0 = rv_nodes[y as usize].prev[0];
                    let ny0 = rv_nodes[y as usize].next[0];
                    rv_nodes[py0 as usize].next[0] = ny0;
                    rv_nodes[ny0 as usize].prev[0] = py0;
                    let py1 = rv_nodes[y as usize].prev[1];
                    rv_nodes[py1 as usize].next[1] = 0;
                    rv_nodes[0].prev[1] = py1;
                } else {
                    rv_nodes[y as usize].width = bw - w;
                }
            } else {
                let py1 = rv_nodes[y as usize].prev[1];
                rv_nodes[py1 as usize].next[1] = 0;
                rv_nodes[0].prev[1] = py1;
                // insert y after x on side 1
                let nx1 = rv_nodes[x as usize].next[1];
                rv_nodes[y as usize].prev[1] = x;
                rv_nodes[y as usize].next[1] = nx1;
                rv_nodes[x as usize].next[1] = y;
                rv_nodes[nx1 as usize].prev[1] = y;
                rv_nodes[y as usize].word = wnew;
                rv_nodes[x as usize].width = aw - w;
            }
        }

        let root_node = rv_nodes[0].next[0];
        let body_word = rv_nodes[root_node as usize].word;
        let full_word = Self::op_cat(body_word, mark_word, slab16, slp);
        (full_word, rv_ops)
    }

    pub fn decode(&mut self, records: &[u16], n: usize, orig_ptr: u32, out: &mut Vec<u8>) -> B4Stats {
        let t0 = Instant::now();
        let mut counts = [0u32; 256];
        self.max_runs.clear();
        let mut i = 0usize;
        while i < records.len() {
            let rec = records[i];
            let b = (rec >> 8) as u8;
            let mut w = (rec & 0xFF) as u32;
            i += 1;
            while i < records.len() && (records[i] >> 8) as u8 == b {
                w += (records[i] & 0xFF) as u32;
                i += 1;
            }
            counts[b as usize] += w;
            self.max_runs.push((w, b));
        }
        let r0 = self.max_runs.len();

        let mut c_off = [0u32; 256];
        let mut b_end = [0u32; 256];
        let mut acc = 0u32;
        let mut active: Vec<u8> = Vec::with_capacity(256);
        for b in 0..256usize {
            c_off[b] = acc;
            acc += counts[b];
            b_end[b] = acc;
            if counts[b] > 0 {
                active.push(b as u8);
            }
        }
        assert_eq!(acc as usize, n);

        if active.len() == 1 {
            out.clear();
            out.resize(n, active[0]);
            let tot = t0.elapsed().as_secs_f64() * 1e3;
            return B4Stats {
                r0,
                peak_kernel_bytes: self.peak_kernel_bytes(),
                total_ms: tot,
                ..B4Stats::default()
            };
        }

        // Rarest-first elimination order -> largest bucket is surviving hub!
        let mut order = active;
        order.sort_unstable_by_key(|&b| (counts[b as usize], b));
        let num_active = order.len();
        let hub = order[num_active - 1];
        let mut rank = [0u16; 256];
        for (idx, &b) in order.iter().enumerate() {
            rank[b as usize] = idx as u16;
        }

        for b in 0..256 {
            self.pool.clear_list(&mut self.out_at[b]);
            self.pool.clear_list(&mut self.in_at[b]);
            self.pool.clear_list(&mut self.self_at[b]);
        }
        self.slab16.clear();
        self.slp.clear();

        // Build initial F-bucket-split pieces and route directly into Theorem B4 causal queues
        let mut occ = c_off;
        let mut pos = 0u32;
        let mut sb = 0usize;
        while sb < 256 && b_end[sb] == 0 {
            sb += 1;
        }
        for &(mut w, db) in &self.max_runs {
            let mut dst = occ[db as usize];
            occ[db as usize] += w;
            while w > 0 {
                while pos >= b_end[sb] {
                    sb += 1;
                }
                let take = w.min(b_end[sb] - pos);
                let p = Piece::new(pos, take, dst, sb as u8, db, u32::from(db));
                if sb as u8 == db {
                    self.pool.push(&mut self.self_at[sb], p);
                } else if rank[sb] < rank[db as usize] {
                    self.pool.push(&mut self.out_at[sb], p);
                } else {
                    self.pool.push(&mut self.in_at[db as usize], p);
                }
                pos += take;
                dst += take;
                w -= take;
            }
        }

        let mut cur_p = orig_ptr;
        let mut cur_pb = {
            let mut found = 0u8;
            for &b in &order {
                if c_off[b as usize] <= orig_ptr && orig_ptr < b_end[b as usize] {
                    found = b;
                    break;
                }
            }
            found
        };
        let mut shift_bytes: usize = 0;
        let mut b4_ops: usize = 0;

        // Stage 1: Exact Theorem B4 Bucket-Schur over order[0..num_active-1]
        for &e in &order[..num_active - 1] {
            let ei = e as usize;
            let b_start = c_off[ei];
            let b_len = counts[ei];
            let k_out = (self.out_at[ei].total + self.self_at[ei].total) as usize;
            let k_in = self.in_at[ei].total as usize;
            let r_self_e = self.self_at[ei].total as usize;

            let shift = Self::build_shift_bin_grid(
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

            if cur_pb == e {
                let start_p = cur_p;
                let mut cyc_hs: Vec<u32> = Vec::new();
                let p_budget = (4 * k_out).max(16);
                let mut p_steps = 0usize;
                let mut exited = false;
                let mut closed = false;
                let saved_shift = shift_bytes;

                while p_steps < p_budget {
                    let oi = Self::grid_find(&self.out_sorted, &self.grid, shift, b_start, cur_p);
                    let op = self.out_sorted[oi];
                    let off = cur_p - op.src;
                    if op.db != e {
                        cur_p = op.dst + off;
                        cur_pb = op.db;
                        shift_bytes += Self::word_len(op.word, &self.slab16, &self.slp) as usize;
                        exited = true;
                        break;
                    } else {
                        let a_pos = op.src;
                        let b_pos = a_pos + op.len;
                        let delta = (op.dst as i64) - (a_pos as i64);
                        if delta == 0 {
                            cyc_hs.push(op.word);
                            closed = true;
                            break;
                        }
                        let k_jump = if delta > 0 {
                            let d = delta as u32;
                            let k = (b_pos - 1 - cur_p) / d + 1;
                            if cur_p < start_p
                                && start_p < cur_p + k * d
                                && (start_p - cur_p) % d == 0
                            {
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
                        if op.src <= start_p && start_p < op.src + op.len {
                            if start_p > op.src {
                                self.sub_cur.push(Piece::new(op.src, start_p - op.src, op.dst, e, op.db, op.word));
                            }
                            self.sub_ext.push(Piece::new(start_p, 1, op.dst + (start_p - op.src), e, op.db, op.word));
                            if start_p + 1 < op.src + op.len {
                                self.sub_cur.push(Piece::new(
                                    start_p + 1,
                                    op.src + op.len - (start_p + 1),
                                    op.dst + (start_p + 1 - op.src),
                                    e,
                                    op.db,
                                    op.word,
                                ));
                            }
                        } else {
                            self.sub_cur.push(op);
                        }
                    }
                    b4_ops += Self::subinterval_eliminate(
                        &mut self.sub_cur,
                        &mut self.sub_ext,
                        &mut self.sub_next,
                        e,
                        &mut self.slab16,
                        &mut self.slp,
                        &mut self.tmp_out,
                    );
                    if let Some(&mp) = self.tmp_out.first() {
                        if mp.db == e && mp.dst == start_p {
                            cyc_hs.clear();
                            cyc_hs.push(mp.word);
                            closed = true;
                        } else {
                            cur_p = mp.dst;
                            cur_pb = mp.db;
                            shift_bytes += Self::word_len(mp.word, &self.slab16, &self.slp) as usize;
                        }
                    }
                }

                if closed {
                    let t_exp = Instant::now();
                    out.clear();
                    if out.capacity() < n {
                        out.reserve_exact(n - out.capacity());
                    }
                    let mut root = u32::MAX;
                    for &hw in cyc_hs.iter().rev() {
                        root = Self::op_cat(root, hw, &mut self.slab16, &mut self.slp);
                    }
                    Self::expand_memoized(
                        root,
                        &self.slab16,
                        &self.slp,
                        &mut self.slp_memo,
                        &mut self.slp_stack,
                        out,
                    );
                    Self::finalize_in_place(saved_shift, n, out);
                    let expand_ms = t_exp.elapsed().as_secs_f64() * 1e3;
                    let total_ms = t0.elapsed().as_secs_f64() * 1e3;
                    return B4Stats {
                        r0,
                        b4_ops,
                        hub_pieces: 0,
                        hub_rv_ops: 0,
                        slp_nodes: self.slp.len(),
                        slab16_len: self.slab16.len(),
                        peak_kernel_bytes: self.peak_kernel_bytes(),
                        contract_ms: (total_ms - expand_ms).max(0.0),
                        expand_ms,
                        total_ms,
                    };
                }
            }

            self.pool.drain_into(&mut self.in_at[ei], &mut self.in_buf);
            if r_self_e == 0 {
                let out_sorted = &self.out_sorted;
                let grid = &self.grid;
                let slab16 = &mut self.slab16;
                let slp = &mut self.slp;
                let pool = &mut self.pool;
                let k_len = out_sorted.len();
                for &p in &self.in_buf {
                    let mut cs = p.src;
                    let mut cd = p.dst;
                    let mut rem = p.len;
                    let sb = p.sb;
                    let wid = p.word;
                    let mut oi = Self::grid_find(out_sorted, grid, shift, b_start, cd);
                    while rem > 0 {
                        let op = out_sorted[oi];
                        let off = cd - op.src;
                        let take = rem.min(op.len - off);
                        let nw = Self::op_cat(op.word, wid, slab16, slp);
                        let np = Piece::new(cs, take, op.dst + off, sb, op.db, nw);
                        if sb == op.db {
                            pool.push(&mut self.self_at[sb as usize], np);
                        } else if rank[sb as usize] < rank[op.db as usize] {
                            pool.push(&mut self.out_at[sb as usize], np);
                        } else {
                            pool.push(&mut self.in_at[op.db as usize], np);
                        }
                        b4_ops += 1;
                        cs += take;
                        cd += take;
                        rem -= take;
                        if oi + 1 < k_len {
                            oi += 1;
                        }
                    }
                }
            } else {
                let mut self_rows = 0u32;
                for &p in &self.out_sorted {
                    if p.db == e {
                        self_rows += p.len;
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
                    let mut fast_ops = 0usize;
                    let mut aborted = false;

                    for &p in &self.in_buf {
                        let mut cs = p.src;
                        let mut cd = p.dst;
                        let mut rem = p.len;
                        let sb = p.sb;
                        let wid = p.word;
                        let mut oi = Self::grid_find(out_sorted, grid, shift, b_start, cd);
                        while rem > 0 {
                            let op = out_sorted[oi];
                            let off = cd - op.src;
                            let take = rem.min(op.len - off);
                            fast_ops += 1;
                            if op.db != e {
                                let nw = Self::op_cat(op.word, wid, slab16, slp);
                                self.tmp_out.push(Piece::new(cs, take, op.dst + off, sb, op.db, nw));
                            } else {
                                let delta = (op.dst as i64) - (op.src as i64);
                                fast_ops += Self::spiral_push(
                                    cs,
                                    take,
                                    cd,
                                    wid,
                                    op.src,
                                    op.src + op.len,
                                    delta,
                                    op.word,
                                    sb,
                                    e,
                                    slab16,
                                    slp,
                                    &mut self.self_q,
                                ) - 1;
                            }
                            cs += take;
                            cd += take;
                            rem -= take;
                            if oi + 1 < k_len {
                                oi += 1;
                            }
                        }
                        if fast_ops > budget {
                            aborted = true;
                            break;
                        }
                    }

                    while !aborted {
                        let Some(q) = self.self_q.pop() else { break };
                        let oi = Self::grid_find(out_sorted, grid, shift, b_start, q.dst);
                        let op = out_sorted[oi];
                        let off = q.dst - op.src;
                        let take = q.len.min(op.len - off);
                        if take < q.len {
                            self.self_q.push(Piece::new(
                                q.src + take,
                                q.len - take,
                                q.dst + take,
                                q.sb,
                                e,
                                q.word,
                            ));
                        }
                        fast_ops += 1;
                        if fast_ops > budget {
                            aborted = true;
                            break;
                        }
                        if op.db != e {
                            let nw = Self::op_cat(op.word, q.word, slab16, slp);
                            self.tmp_out.push(Piece::new(q.src, take, op.dst + off, q.sb, op.db, nw));
                        } else {
                            let delta = (op.dst as i64) - (op.src as i64);
                            fast_ops += Self::spiral_push(
                                q.src,
                                take,
                                q.dst,
                                q.word,
                                op.src,
                                op.src + op.len,
                                delta,
                                op.word,
                                q.sb,
                                e,
                                slab16,
                                slp,
                                &mut self.self_q,
                            ) - 1;
                        }
                    }

                    if aborted {
                        use_subint = true;
                    } else {
                        b4_ops += fast_ops;
                    }
                }

                if use_subint {
                    self.sub_cur.clear();
                    self.sub_cur.extend_from_slice(&self.out_sorted);
                    self.sub_ext.clear();
                    self.sub_ext.extend_from_slice(&self.in_buf);
                    b4_ops += Self::subinterval_eliminate(
                        &mut self.sub_cur,
                        &mut self.sub_ext,
                        &mut self.sub_next,
                        e,
                        &mut self.slab16,
                        &mut self.slp,
                        &mut self.tmp_out,
                    );
                }

                for &np in &self.tmp_out {
                    if np.sb == np.db {
                        self.pool.push(&mut self.self_at[np.sb as usize], np);
                    } else if rank[np.sb as usize] < rank[np.db as usize] {
                        self.pool.push(&mut self.out_at[np.sb as usize], np);
                    } else {
                        self.pool.push(&mut self.in_at[np.db as usize], np);
                    }
                }
            }
        }

        // Stage 2: Exact O(k_hub) Doubly-Linked Hub RV -> Single Root SLP (ZERO O(n) edge walk!)
        let hi = hub as usize;
        let hub_start = c_off[hi];
        let hub_len = counts[hi];
        Self::build_shift_bin_grid(
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
        let hub_pieces = self.out_sorted.len();
        let (root_w, hub_rv_ops) = Self::hub_rv_to_root_slp(
            &self.out_sorted,
            hub_start,
            hub_len,
            cur_p,
            &mut self.slab16,
            &mut self.slp,
            &mut self.rv_nodes,
            &mut self.rv_img_ord,
        );

        // Stage 3: Single Sequential Memoized SLP Expansion -> In-Place Rotation -> `out` (`n` bytes)!
        let t_exp = Instant::now();
        out.clear();
        if out.capacity() < n {
            out.reserve_exact(n - out.capacity());
        }
        Self::expand_memoized(
            root_w,
            &self.slab16,
            &self.slp,
            &mut self.slp_memo,
            &mut self.slp_stack,
            out,
        );
        Self::finalize_in_place(shift_bytes, n, out);
        let expand_ms = t_exp.elapsed().as_secs_f64() * 1e3;
        let total_ms = t0.elapsed().as_secs_f64() * 1e3;

        B4Stats {
            r0,
            b4_ops: b4_ops + hub_rv_ops,
            hub_pieces,
            hub_rv_ops,
            slp_nodes: self.slp.len(),
            slab16_len: self.slab16.len(),
            peak_kernel_bytes: self.peak_kernel_bytes(),
            contract_ms: (total_ms - expand_ms).max(0.0),
            expand_ms,
            total_ms,
        }
    }
}
