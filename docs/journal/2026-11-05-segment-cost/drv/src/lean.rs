//! The lean per-segment path (scratch prototype, not for commit), for counting only.
//!
//! - **Starts set by the build, at no cost:** row r starts a segment iff r & (K - 1) == 0
//!   (K = 2^bits). Nothing is flagged, sorted or searched: a lane tests the link it just
//!   read (`q >> 8`), so a segment ends on the step that reads its last byte, with no
//!   wasted read of the next start's entry (master reads it, marks, and rereads it at the
//!   take). Segment id = start row >> bits; the walk's own first row r0 is a head segment
//!   (id ns) when it is not a start.
//! - **Takes in row order** (ids 0, 1, 2, ...): the dense, row-sorted starts the collision
//!   found walk in cross-chain lockstep.
//! - **Branch-free steps:** every lane stores its byte and its next row every step; the
//!   offset's marks come from one pass over the 32 rows (a mask compare).
//! - **No per-segment allocation, one arena:** the staged tile (32 x 256) is dumped whole
//!   into the arena at each chunk's end (one 8 KiB copy), so lane k's bytes at global
//!   offset g sit at arena[(g / TILE) * L * TILE + k * TILE + g % TILE].
//! - **Deferred joins:** a take writes pos[id] = g0 << 5 | lane (sequential in id); a
//!   finish writes fin[id] = succ << 32 | g1 (near-sequential: ids finish within a window
//!   of the 32 lanes). No search: succ = link >> bits.
//! - **The join as a second-level walk:** L2 lanes walk the segment list from level-2
//!   starts (ids with id % K2 == 0, plus the chain's head), each step copying one segment's
//!   bytes from the arena into the lane's own buffer; a level-3 chain over the level-2
//!   segments then copies them into the output.
const L: usize = 32;
const TILE: usize = 256;
const L2: usize = 16;
const K2BITS: u32 = 6;

#[derive(Default)]
pub struct Lean {
    arena: Vec<u8>,
    /// per segment, 8 bytes: the low half (g0 << 5 | lane) written at its take, the high
    /// half (succ | len << 20, len 4095 meaning "in `long`") at its finish
    rec: Vec<u32>,
    long: Vec<(u32, u32)>,
    // level 2: each lane's bytes, and per level-2 segment (lane, offset in it, length, successor)
    out2: Vec<Vec<u8>>,
    rec2: Vec<(u32, u32, u32, u32)>,
    ns: usize,
}

impl Lean {
    pub fn segs(&self) -> usize {
        self.ns
    }
}

#[inline(always)]
fn prefetch(t: *const u32, row: u32) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        _mm_prefetch::<_MM_HINT_T0>(t.wrapping_add(row as usize) as *const i8);
    }
}

#[inline(always)]
fn prefetch_p<T>(p: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        _mm_prefetch::<_MM_HINT_T0>(p as *const i8);
    }
}

/// The walk (level 1): each segment's bytes into the arena, pos and fin per segment
#[inline(never)]
pub fn lean_walk(t: &[u32], r0: u32, bits: u32, s: &mut Lean) -> (u32, usize) {
    let n = t.len();
    let mask = (1u32 << bits) - 1;
    let ns = n.div_ceil(1 << bits);
    let head = r0 & mask != 0;
    let total = ns + head as usize;
    s.ns = total;
    // ids: 0..ns the start rows j << bits; ns the head (r0), taken first
    s.rec.clear();
    s.rec.resize(2 * total, 0);
    s.long.clear();
    // the arena holds whole chunks (grown a chunk at a time, kept from block to block),
    // plus 16 bytes of slack for the join's 16-byte copies
    if s.arena.len() < (n / L / TILE + 2) * L * TILE + 16 {
        s.arena.resize((n / L / TILE + 2) * L * TILE + 16, 0);
    }
    let tp = t.as_ptr();
    // the take order: the head (if any), then ids 0, 1, 2, ...
    let mut taken = 0usize;
    let hd = head as usize;
    let mut next_id = move || -> Option<usize> {
        if taken == total {
            return None;
        }
        let id = if taken < hd { ns } else { taken - hd };
        taken += 1;
        Some(id)
    };
    let start_row = |id: usize| if id == ns { r0 } else { (id as u32) << bits };
    let mut rows = [0u32; L];
    let mut cur = [0u32; L];
    let mut g0s = [0u32; L];
    let mut live: u64 = 0;
    for k in 0..L {
        if let Some(id) = next_id() {
            cur[k] = id as u32;
            rows[k] = start_row(id);
            s.rec[2 * id] = k as u32; // g0 = 0
            prefetch(tp, rows[k]);
            live |= 1 << k;
        }
    }
    let mut tile = [[0u8; TILE]; L];
    let mut g: usize = 0; // global offset of the chunk's first offset
    let mut chunk = 0usize;
    let rec = s.rec.as_mut_ptr();
    'walk: loop {
        let tl = tile.as_mut_ptr().cast::<u8>();
        let mut j = 0;
        while j < TILE {
            macro_rules! step {
                ($k:expr) => {{
                    // SAFETY: rows are below n; k * TILE + j inside the tile
                    unsafe {
                        let r = rows.get_unchecked_mut($k);
                        let q = *tp.add(*r as usize);
                        *tl.add($k * TILE + j) = q as u8;
                        *r = q >> 8;
                        prefetch(tp, q >> 8);
                    }
                }};
            }
            let mut c = 0;
            while c < L {
                step!(c);
                step!(c + 1);
                step!(c + 2);
                step!(c + 3);
                step!(c + 4);
                step!(c + 5);
                step!(c + 6);
                step!(c + 7);
                c += 8;
            }
            // the offset's marks: lanes whose next row is a start
            let mut m: u64 = 0;
            for k in 0..L {
                m |= ((rows[k] & mask == 0) as u64) << k;
            }
            m &= live;
            while m != 0 {
                let k = m.trailing_zeros() as usize;
                m &= m - 1;
                let gj = (g + j) as u64;
                // SAFETY: ids below total
                let len = gj as u32 + 1 - g0s[k];
                let l12 = if len >= 4095 {
                    s.long.push((cur[k], len));
                    4095
                } else {
                    len
                };
                unsafe {
                    *rec.add(2 * cur[k] as usize + 1) = (rows[k] >> bits) | l12 << 20;
                }
                match next_id() {
                    Some(id) => {
                        cur[k] = id as u32;
                        rows[k] = start_row(id);
                        g0s[k] = gj as u32 + 1;
                        unsafe { *rec.add(2 * id) = ((gj as u32 + 1) << 5) | k as u32 };
                        prefetch(tp, rows[k]);
                    }
                    None => {
                        live &= !(1 << k);
                    }
                }
            }
            j += 1;
            if live == 0 {
                break;
            }
        }
        // the chunk's tile into the arena, whole
        if (chunk + 1) * L * TILE + 16 > s.arena.len() {
            s.arena.resize((chunk + 2) * L * TILE + 16, 0);
        }
        // SAFETY: the arena holds chunk + 1 chunks and the slack
        unsafe {
            std::ptr::copy_nonoverlapping(
                tile.as_ptr().cast::<u8>(),
                s.arena.as_mut_ptr().add(chunk * L * TILE),
                L * TILE,
            );
        }
        chunk += 1;
        g += TILE;
        if live == 0 {
            break 'walk;
        }
        // a head segment on a cycle with no start row never ends (a block that is a short
        // string repeated): the prototype refuses it, a real walk would fall back to `walk`
        assert!(g <= 2 * n + 2 * TILE, "head segment on a start-free cycle");
    }
    let first = if head { ns } else { (r0 >> bits) as usize };
    (first as u32, total)
}

/// One segment's bytes, from its lane's stripes in the arena to dst; returns its length
#[inline(always)]
unsafe fn copy_seg(arena: *const u8, posv: u32, len: usize, dst: *mut u8) -> usize {
    let k = (posv & 31) as usize;
    let g0 = (posv >> 5) as usize;
    let g1 = g0 + len - 1;
    let mut g = g0;
    let mut d = dst;
    let end = g1 + 1;
    while g < end {
        let run = (TILE - g % TILE).min(end - g);
        let src = arena.add((g / TILE) * L * TILE + k * TILE + g % TILE);
        // 16 bytes at a time, the last copy overlapping past the run (dst has slack)
        let mut i = 0;
        while i < run {
            std::ptr::copy_nonoverlapping(src.add(i), d.add(i), 16);
            i += 16;
        }
        d = d.add(run);
        g += run;
    }
    len
}

/// The join (levels 2 and 3): out = the segments in output order, n bytes
#[inline(never)]
pub fn lean_join(s: &mut Lean, first: u32, total: usize, n: usize, out: &mut Vec<u8>) {
    let k2mask = (1u32 << K2BITS) - 1;
    // level-2 ids: segment id >> K2BITS for ids with the low bits zero; the chain's head
    // (if not such an id) gets level-2 id n2
    let n2 = total.div_ceil(1 << K2BITS);
    let head2 = first & k2mask != 0;
    let total2 = n2 + head2 as usize;
    s.rec2.clear();
    s.rec2.resize(total2, (0, 0, 0, u32::MAX));
    s.out2.resize_with(L2, Vec::new);
    // each lane's buffer can hold the whole block (untouched pages cost nothing)
    for b in &mut s.out2 {
        b.clear();
        b.reserve(n + 64);
    }
    let arena = s.arena.as_ptr();
    let rec = s.rec.as_ptr() as *const u64;
    // level-2 starts that exist: ids j << K2BITS below total
    let mut taken = 0usize;
    let hd2 = head2 as usize;
    let mut next2 = move || -> Option<usize> {
        if taken == total2 {
            return None;
        }
        let id2 = if taken < hd2 { n2 } else { taken - hd2 };
        taken += 1;
        Some(id2)
    };
    let seg_of = |id2: usize| if id2 == n2 { first } else { (id2 as u32) << K2BITS };
    let mut seg = [0u32; L2];
    let mut cur2 = [0u32; L2];
    let mut st = [0usize; L2];
    let mut live = 0u32;
    for k in 0..L2 {
        if let Some(id2) = next2() {
            cur2[k] = id2 as u32;
            seg[k] = seg_of(id2);
            st[k] = 0;
            live |= 1 << k;
            unsafe {
                prefetch_p(rec.add(seg[k] as usize));
            }
        }
    }
    let mut dp: [*mut u8; L2] = std::array::from_fn(|k| s.out2[k].as_mut_ptr());
    let base: [*mut u8; L2] = dp;
    // A segment off r0's cycle is never reached from a level-2 start on it only when its
    // cycle has none: those cycles are never joined, as master's starts off r0's cycle
    while live != 0 {
        for k in 0..L2 {
            if live >> k & 1 == 0 {
                continue;
            }
            let sg = seg[k] as usize;
            // SAFETY: sg below total; the lane's buffer has 2n / L2 + 64 KiB, more than any
            // lane's share (checked below)
            unsafe {
                let rv = rec.add(sg).read_unaligned();
                let (posv, hi) = (rv as u32, (rv >> 32) as u32);
                let mut ln = (hi >> 20) as usize;
                if ln == 4095 {
                    ln = s.long.iter().find(|e| e.0 as usize == sg).unwrap().1 as usize;
                }
                let len = copy_seg(arena, posv, ln, dp[k]);
                dp[k] = dp[k].add(len);
                let nx = hi & 0xf_ffff;
                prefetch_p(rec.add(nx as usize));
                // a level-2 segment ends at a level-2 start, or back at the chain's head
                // (a cycle of r0's with no level-2 start)
                if nx & k2mask == 0 || nx == first {
                    // the level-2 segment ends: record it, take the next start
                    let off = st[k];
                    let now = dp[k].offset_from(base[k]) as usize;
                    let nx2 = if nx == first { if head2 { n2 as u32 } else { nx >> K2BITS } } else { nx >> K2BITS };
                    s.rec2[cur2[k] as usize] = (k as u32, off as u32, (now - off) as u32, nx2);
                    match next2() {
                        Some(id2) => {
                            cur2[k] = id2 as u32;
                            seg[k] = seg_of(id2);
                            st[k] = now;
                        }
                        None => live &= !(1 << k),
                    }
                } else {
                    seg[k] = nx;
                }
            }
        }
    }
    for k in 0..L2 {
        let len = unsafe { dp[k].offset_from(base[k]) } as usize;
        assert!(len <= s.out2[k].capacity());
        unsafe { s.out2[k].set_len(len) };
    }
    // level 3: the level-2 segments in order
    out.clear();
    out.reserve(n);
    let mut id2 = if head2 { n2 } else { (first >> K2BITS) as usize };
    while out.len() < n {
        let (k, off, len, nx) = s.rec2[id2];
        let b = &s.out2[k as usize][off as usize..(off + len) as usize];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        id2 = nx as usize;
    }
}

pub fn walk_lean(t: &[u32], orig: u32, bits: u32, s: &mut Lean, out: &mut Vec<u8>) {
    let r0 = t[orig as usize] >> 8;
    let (first, total) = lean_walk(t, r0, bits, s);
    lean_join(s, first, total, t.len(), out);
}

#[inline(never)]
pub fn measured_lean(t: &[u32], orig: u32, bits: u32, s: &mut Lean, out: &mut Vec<u8>) {
    walk_lean(t, orig, bits, s, out)
}
