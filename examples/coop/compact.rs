//! The compact-Ψ prototype, stage A (docs/journal/2026-10-29-compact-psi.md): one core a
//! block, production's build and room walk (`mibz2::bwt::build`, `walk_segments_room`)
//! copied here and made generic over what a row's entry holds, so that an arm differs from
//! production only in its table. The library keeps its walk's scratch and table private,
//! hence the copy; the `o` arm is the copy's check.
//!
//! - `o` (the control): production's entry, the link above the row's byte (link << 8 |
//!   byte, 4 bytes a row), the START flag at bit 31. It should run as `p` does.
//! - `v`: a 32-bit entry holding the link alone, the byte recovered from F (below). Against
//!   `o`: the byte's recovery, and a build without production's init pass and
//!   read-modify-write.
//! - `u`: a 24-bit entry holding the link alone (rows < 2^20, START at bit 23), 3 bytes a
//!   row: 2.7 MB for a 900 K-row block, against 3.6. Read as one unaligned 4-byte load,
//!   masked (a load crosses a cache line for 3 rows in 64). Built with 4-byte stores, each
//!   spilling a zero into the next entry's first byte, which the build writes after it
//!   (a bucket's rows are filled in order), except at a bucket's last row, whose next entry
//!   is another bucket's and may be written already: that store is narrow (2 + 1 bytes).
//!   Against `v`: the packing alone.
//!
//! **The byte from F, off the link chain.** At row r the walk writes L[r] = F[ψ⁻¹(r)], and
//! ψ⁻¹(r) is the row the lane came from, so a lane carries its next byte: at row r it
//! writes the byte it carries, takes F[r] (L of the row it moves to, as L[ψ(r)] = F[r]),
//! and moves to ψ(r). F[r] is its 32-row window's first byte (`win`, n/32 bytes) stepped
//! past any bucket boundary inside the window (`cstart`). Its address is the current row,
//! known before the row's own entry is loaded, so the lookup is not on the chain. A
//! segment's first byte, L of its start, is read from the last column itself, once a
//! segment.

use super::{random_rows, read_u64};

/// The arms' tables, as the walk's const parameter
pub const O: u8 = 0;
pub const V: u8 = 1;
pub const U: u8 = 2;

/// production's (`bwt::FLAGGED_LANES`, `FLAGGED_SEGMENTS`, `MIN_SEGMENTED`, `PREFETCH`
/// = `PF_L1`)
const LANES: usize = 32;
const SEGMENTS: usize = 512;
const MIN_SEGMENTED: usize = 1 << 16;

/// A compact arm's tables and walk scratch, kept from block to block
#[derive(Default)]
pub struct Compact {
    /// `o`'s and `v`'s entries; `v` grows it once and never clears it, as its build writes
    /// every entry below n
    t32: Vec<u32>,
    /// `u`'s entries, 3 bytes a row and one byte more, so the last row's 4-byte load and
    /// store are in bounds; grown once, as `t32` is
    t24: Vec<u8>,
    n: usize,
    /// F's byte at each 32-row window's first row
    win: Vec<u8>,
    /// each byte's first row in F, and n (257 entries)
    cstart: Vec<u32>,
    s: Scratch,
    /// `h`'s records and raw entries, the worker's own copy of a block's (`load_h`)
    hrec: Vec<Line>,
    hraw: Vec<Line>,
}

/// The room walk's segments (production's `SegScratch`)
#[derive(Default)]
struct Scratch {
    starts: Vec<u32>,
    bufs: Vec<Vec<u8>>,
    next: Vec<usize>,
    n: usize,
    first: usize,
}

impl Scratch {
    fn assemble(&self, out: &mut Vec<u8>) {
        let n = self.n;
        out.clear();
        out.reserve(n);
        let mut seg = self.first;
        while out.len() < n {
            let b = &self.bufs[seg];
            out.extend_from_slice(&b[..b.len().min(n - out.len())]);
            seg = self.next[seg];
        }
    }
}

#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    /// the byte the lane writes at its next step (`v`, `u`): L of `row`
    b: u8,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

#[inline(always)]
const fn flag<const K: u8>() -> u32 {
    if K == U {
        1 << 23
    } else {
        1 << 31
    }
}

/// Row `row`'s entry, its flag included
///
/// SAFETY: `row` is below n, and `t` is the arm's table
#[inline(always)]
unsafe fn load<const K: u8>(t: *const u8, row: u32) -> u32 {
    if K == U {
        u32::from_le(unsafe { (t.add(3 * row as usize) as *const u32).read_unaligned() })
            & 0xff_ffff
    } else {
        unsafe { *(t as *const u32).add(row as usize) }
    }
}

/// The next row, from an unflagged entry
#[inline(always)]
fn link<const K: u8>(q: u32) -> u32 {
    if K == O {
        q >> 8
    } else {
        q
    }
}

#[inline(always)]
fn prefetch<const K: u8>(t: *const u8, row: u32) {
    let at = if K == U {
        3 * row as usize
    } else {
        4 * row as usize
    };
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch never faults
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(
            t.wrapping_add(at) as *const i8
        )
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = at;
}

/// SAFETY (both): `row` is below n
#[inline(always)]
unsafe fn set_flag<const K: u8>(t: *mut u8, row: u32) {
    unsafe {
        if K == U {
            *t.add(3 * row as usize + 2) |= 0x80;
        } else {
            *(t as *mut u32).add(row as usize) |= 1 << 31;
        }
    }
}

#[inline(always)]
unsafe fn clear_flag<const K: u8>(t: *mut u8, row: u32) {
    unsafe {
        if K == U {
            *t.add(3 * row as usize + 2) &= 0x7f;
        } else {
            *(t as *mut u32).add(row as usize) &= !(1 << 31);
        }
    }
}

/// F[row]: its window's first byte, stepped past the bucket boundaries in the window
///
/// SAFETY: `row` is below n = cstart[256], so the window is in `win` and the loop stops at
/// a byte below 256
#[inline(always)]
unsafe fn f_byte(win: &[u8], cstart: &[u32], row: u32) -> u8 {
    unsafe {
        let mut c = *win.get_unchecked((row >> 5) as usize) as usize;
        while row >= *cstart.get_unchecked(c + 1) {
            c += 1;
        }
        c as u8
    }
}

impl Compact {
    /// The table of the last column `ll`, whose bytes `counts` counts (`kind` o, v or u)
    pub fn build(&mut self, kind: char, ll: &[u8], counts: &[u32; 256]) {
        self.n = ll.len();
        match kind {
            'o' => self.build_o(ll, counts),
            'v' => {
                self.build_v(ll, counts);
                self.f_tables(counts);
            }
            'u' => {
                self.build_u(ll, counts);
                self.f_tables(counts);
            }
            _ => unreachable!("compact arms are o, v and u (h: `load_h`)"),
        }
    }

    /// The block's output into `out`, from the table `build` made
    pub fn walk(&mut self, kind: char, ll: &[u8], orig_ptr: u32, out: &mut Vec<u8>) {
        match kind {
            'o' => self.walk_k::<O>(ll, orig_ptr, out),
            'v' => self.walk_k::<V>(ll, orig_ptr, out),
            'u' => self.walk_k::<U>(ll, orig_ptr, out),
            'h' => self.walk_h(ll, orig_ptr, out),
            _ => unreachable!("compact arms are o, v, u and h"),
        }
    }

    /// production's `build`, line for line
    fn build_o(&mut self, ll: &[u8], counts: &[u32; 256]) {
        let tt = &mut self.t32;
        tt.clear();
        tt.extend(ll.iter().map(|&c| c as u32));
        let mut ftab = [0u32; 256];
        let mut cum = 0u32;
        for i in 0..256 {
            ftab[i] = cum;
            cum += counts[i];
        }
        let mut i = 0;
        while i + 16 <= ll.len() {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                let c = ll[i] as usize;
                let k = ftab[c] as usize;
                let base = (i as u32) << 8;
                for (j, e) in tt[k..k + 16].iter_mut().enumerate() {
                    *e = (*e & 0xff) | (base + ((j as u32) << 8));
                }
                ftab[c] += 16;
            } else {
                for i in i..i + 16 {
                    let c = ll[i];
                    let k = ftab[c as usize] as usize;
                    tt[k] = (tt[k] & 0xff) | (i as u32) << 8;
                    ftab[c as usize] += 1;
                }
            }
            i += 16;
        }
        for i in i..ll.len() {
            let c = ll[i];
            let k = ftab[c as usize] as usize;
            tt[k] = (tt[k] & 0xff) | (i as u32) << 8;
            ftab[c as usize] += 1;
        }
    }

    /// production's build storing the link alone: no init pass, no read-modify-write
    fn build_v(&mut self, ll: &[u8], counts: &[u32; 256]) {
        let n = ll.len();
        if self.t32.len() < n {
            self.t32.resize(n, 0);
        }
        let tt = &mut self.t32[..n];
        let mut ftab = [0u32; 256];
        let mut cum = 0u32;
        for i in 0..256 {
            ftab[i] = cum;
            cum += counts[i];
        }
        let mut i = 0;
        while i + 16 <= n {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                let c = ll[i] as usize;
                let k = ftab[c] as usize;
                for (j, e) in tt[k..k + 16].iter_mut().enumerate() {
                    *e = i as u32 + j as u32;
                }
                ftab[c] += 16;
            } else {
                for i in i..i + 16 {
                    let c = ll[i] as usize;
                    tt[ftab[c] as usize] = i as u32;
                    ftab[c] += 1;
                }
            }
            i += 16;
        }
        for i in i..n {
            let c = ll[i] as usize;
            tt[ftab[c] as usize] = i as u32;
            ftab[c] += 1;
        }
    }

    /// `build_v` into 3-byte entries: a 4-byte store each, narrow at a bucket's last row
    fn build_u(&mut self, ll: &[u8], counts: &[u32; 256]) {
        let n = ll.len();
        assert!(n < 1 << 20, "u's entries hold 20-bit links");
        if self.t24.len() < 3 * n + 1 {
            self.t24.resize(3 * n + 1, 0);
        }
        let t = &mut self.t24[..3 * n + 1];
        let (mut ftab, mut end) = ([0u32; 256], [0u32; 256]);
        let mut cum = 0u32;
        for i in 0..256 {
            ftab[i] = cum;
            cum += counts[i];
            end[i] = cum;
        }
        // (checked slices, as production's stores are checked indexes)
        #[inline(always)]
        fn wide(t: &mut [u8], k: usize, v: u32) {
            t[3 * k..3 * k + 4].copy_from_slice(&v.to_le_bytes());
        }
        #[inline(always)]
        fn narrow(t: &mut [u8], k: usize, v: u32) {
            t[3 * k..3 * k + 3].copy_from_slice(&v.to_le_bytes()[..3]);
        }
        let mut i = 0;
        while i + 16 <= n {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                let c = ll[i] as usize;
                let k = ftab[c] as usize;
                for j in 0..15 {
                    wide(t, k + j, (i + j) as u32);
                }
                if k + 16 == end[c] as usize {
                    narrow(t, k + 15, (i + 15) as u32);
                } else {
                    wide(t, k + 15, (i + 15) as u32);
                }
                ftab[c] += 16;
            } else {
                for i in i..i + 16 {
                    let c = ll[i] as usize;
                    let k = ftab[c];
                    if k + 1 == end[c] {
                        narrow(t, k as usize, i as u32);
                    } else {
                        wide(t, k as usize, i as u32);
                    }
                    ftab[c] += 1;
                }
            }
            i += 16;
        }
        for i in i..n {
            let c = ll[i] as usize;
            let k = ftab[c];
            if k + 1 == end[c] {
                narrow(t, k as usize, i as u32);
            } else {
                wide(t, k as usize, i as u32);
            }
            ftab[c] += 1;
        }
    }

    /// F's tables: each byte's first row, and each 32-row window's first byte
    fn f_tables(&mut self, counts: &[u32; 256]) {
        let n = self.n;
        self.cstart.clear();
        self.cstart.push(0);
        for c in 0..256 {
            let s = self.cstart[c] + counts[c];
            self.cstart.push(s);
        }
        self.win.clear();
        self.win.resize(n.div_ceil(32), 0);
        for c in 0..256 {
            let (a, b) = (self.cstart[c] as usize, self.cstart[c + 1] as usize);
            // window w's first row 32 w is in [a, b) for w in [ceil(a/32), ceil(b/32))
            for w in a.div_ceil(32)..b.div_ceil(32) {
                self.win[w] = c as u8;
            }
        }
    }

    fn walk_k<const K: u8>(&mut self, ll: &[u8], orig_ptr: u32, out: &mut Vec<u8>) {
        if self.n < MIN_SEGMENTED {
            self.plain::<K>(ll, orig_ptr, out);
        } else {
            self.room::<K>(ll, orig_ptr);
            self.s.assemble(out);
        }
    }

    /// production's plain walk, for blocks too short to segment
    fn plain<const K: u8>(&mut self, _ll: &[u8], orig_ptr: u32, out: &mut Vec<u8>) {
        let n = self.n;
        let t = if K == U {
            self.t24.as_ptr()
        } else {
            self.t32.as_ptr() as *const u8
        };
        out.clear();
        out.reserve(n);
        // SAFETY: every link is below n, as is orig_ptr (checked by the stream's parser)
        unsafe {
            if K == O {
                let mut p = load::<K>(t, orig_ptr);
                for _ in 0..n {
                    p = load::<K>(t, p >> 8);
                    out.push(p as u8);
                }
            } else {
                let (win, cstart) = (&self.win[..], &self.cstart[..]);
                let mut r = load::<K>(t, orig_ptr);
                let mut b = f_byte(win, cstart, orig_ptr);
                for _ in 0..n {
                    out.push(b);
                    b = f_byte(win, cstart, r);
                    r = load::<K>(t, r);
                }
            }
        }
    }

    /// production's `room_segments`, its step made generic over the table: the same
    /// starts, passes, handoffs and tail, and the contract its unchecked writes rely on
    /// (see `mibz2::bwt::walk_segments_room_with_prefetch`)
    #[inline(never)]
    fn room<const K: u8>(&mut self, ll: &[u8], orig_ptr: u32) {
        let n = self.n;
        let tm = if K == U {
            self.t24.as_mut_ptr()
        } else {
            self.t32.as_mut_ptr() as *mut u8
        };
        let t = tm as *const u8;
        let (win, cstart) = (&self.win[..], &self.cstart[..]);
        let s = &mut self.s;
        // SAFETY (the loads, flags and F lookups below): every row is below n, the starts
        // being rows and the links coming from the build
        let r0 = link::<K>(unsafe { load::<K>(t, orig_ptr) });
        random_rows(n, SEGMENTS - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            unsafe { set_flag::<K>(tm, r) };
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let st = starts[seg];
            let q = unsafe { load::<K>(t, st) } & !flag::<K>();
            // SAFETY: the buffer has `cap` > 0 spare bytes
            unsafe { *base = if K == O { q as u8 } else { ll[st as usize] } };
            Lane {
                row: link::<K>(q),
                b: if K == O {
                    0
                } else {
                    unsafe { f_byte(win, cstart, st) }
                },
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
            }
        };
        let finish = |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            // SAFETY: p is within b's allocation, every byte below it written
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };
        // a lane's step from its row's entry `q` (unflagged): the byte it writes, its row moved
        #[inline(always)]
        fn step<const K: u8>(l: &mut Lane, q: u32, win: &[u8], cstart: &[u32]) -> u8 {
            if K == O {
                l.row = q >> 8;
                q as u8
            } else {
                let b = l.b;
                l.b = unsafe { f_byte(win, cstart, l.row) };
                l.row = q;
                b
            }
        }

        let mut queued = 0;
        let mut lanes: [Lane; LANES] = std::array::from_fn(|_| Lane {
            row: 0,
            b: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < LANES && queued < ns {
            lanes[active] = take(queued, bufs);
            prefetch::<K>(t, lanes[active].row);
            queued += 1;
            active += 1;
        }
        if active == LANES {
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                debug_assert!(room > 0);
                let mut base: [*mut u8; LANES] = std::array::from_fn(|k| lanes[k].p);
                for off in 0..room {
                    for k in 0..LANES {
                        let l = &mut lanes[k];
                        let q = unsafe { load::<K>(t, l.row) };
                        if q & flag::<K>() != 0 {
                            l.p = base[k].wrapping_add(off);
                            finish(l, bufs, next);
                            if queued == ns {
                                for (j, lj) in lanes.iter_mut().enumerate() {
                                    if j < k {
                                        lj.p = base[j].wrapping_add(off + 1);
                                    } else if j > k {
                                        lj.p = base[j].wrapping_add(off);
                                    }
                                }
                                break 'full;
                            }
                            *l = take(queued, bufs);
                            prefetch::<K>(t, l.row);
                            queued += 1;
                            base[k] = l.p.wrapping_sub(off + 1);
                            continue;
                        }
                        let byte = step::<K>(l, q, win, cstart);
                        prefetch::<K>(t, l.row);
                        debug_assert!(base[k].wrapping_add(off) < l.end);
                        unsafe { *base[k].wrapping_add(off) = byte };
                    }
                }
                for (l, b) in lanes.iter_mut().zip(base) {
                    l.p = b.wrapping_add(room);
                }
            }
        }
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| l.seg != usize::MAX && next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
            debug_assert!(l.p < l.end);
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = unsafe { load::<K>(t, l.row) };
                if q & flag::<K>() != 0 {
                    finish(l, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                let byte = step::<K>(l, q, win, cstart);
                prefetch::<K>(t, l.row);
                unsafe {
                    *l.p = byte;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        for &r in starts {
            unsafe { clear_flag::<K>(tm, r) };
        }
        s.first = seg_of(r0);
        s.n = n;
    }
}

// ---------------------------------------------------------------------------------------
// `h`: FOR-14/raw records (P40/P38/P84 stage 1; docs/journal/2026-11-01-for14-stage1.md)
//
// One 64-byte record, cache-line aligned, holds 32 consecutive rows j = 32 g .. 32 g + 31:
// - bytes 0..56: the payload, 32 offsets of 14 bits, offset i at bits 14 i .. 14 i + 13
//   (little-endian bit order across the 56 bytes);
// - bytes 56..64: the header, a little-endian u64:
//   - bits 0..20: FOR, the group's base (its smallest ψ); RAW, the group's index in the raw
//     area;
//   - bits 20..28: F, the group's one byte (FOR; zero for RAW);
//   - bit 28: RAW;
//   - bits 29..32: zero;
//   - bits 32..64: the start mask, bit 32 + i set while row 32 g + i starts a segment.
//
// A group is FOR when its 32 rows lie in one F bucket and its ψ span (max − min) is at most
// 2^14 − 2: ψ(j) = base + offset, F[j] = F. RUN groups (ψ consecutive) are FOR with offsets
// 0..31: one format, so the common path has no format branch. Every other group is RAW:
// its 32 entries are in the raw area, 128 bytes (two lines) a raw group, entry i a u32:
// ψ(j) in bits 0..20, F[j] in bits 20..28.
//
// **The sentinel.** Offset 2^14 − 1 (S) means "not an ordinary step": every offset of a RAW
// record is S, and the walk writes S over a start row's offset while it runs (restoring it
// after). So an ordinary step tests one value, the offset it needs anyway; only at S does
// it read the start mask, to tell a start from a raw row.
//
// **The parking record.** One more record, after the block's, whose offsets are all S and
// whose mask is empty. A lane at a raw row prefetches the row's raw entry, writes nothing,
// and moves to the parking record's first row (PARK); at its next turn the ordinary step
// lands on S again (an L1 hit), sees PARK, and takes ψ and F from the entry. So no
// ordinary step tests whether its lane is waiting on a raw entry.
//
// The header is at the end of the line so that every offset's 4-byte load, at byte
// floor(14 i / 8) <= 54, ends inside it: an ordinary step reads one line, at an address
// known from the row alone (header and offset loads independent of each other).
// ---------------------------------------------------------------------------------------

/// A cache line
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Line(pub [u8; 64]);

const H_RAW: u64 = 1 << 28;
const H_MASK20: u64 = 0xf_ffff;
/// the sentinel offset
const H_S: u32 = 0x3fff;
/// a record whose every offset is S, mask empty, RAW: the parking record, and a RAW
/// record's payload before its header is written
const H_ALL_S: Line = {
    let mut l = [0xffu8; 64];
    let mut i = 56;
    while i < 64 {
        l[i] = 0;
        i += 1;
    }
    Line(l)
};

/// A block's FOR-14/raw records, made offline from its full T-vector (the oracle)
#[derive(Default)]
pub struct Records {
    /// the block's records and, last, the parking record
    pub rec: Vec<Line>,
    pub raw: Vec<Line>,
    pub n: usize,
    /// the groups by class: RUN (a FOR group with offsets 0..31), FOR (other), RAW, and of
    /// the RAW ones those crossing a bucket boundary, and those forced (`force_permille`)
    pub run: usize,
    pub for14: usize,
    pub raw_groups: usize,
    pub crossing: usize,
    pub forced: usize,
}

impl Records {
    /// The records of the block whose last column is `ll` (its byte counts `counts`), from
    /// the T-vector production builds (the `o` arm's copy of `bwt::build`): ψ(j) = T[j] >>
    /// 8. `force_permille` of the FOR groups (chosen by a hash of the group's index) are
    /// made RAW too: a check of the raw path, 0 for every timed run.
    pub fn from_block(ll: &[u8], counts: &[u32; 256], force_permille: u32) -> Records {
        let n = ll.len();
        assert!(n < 1 << 20, "ψ and the raw index are 20 bits");
        let mut c = Compact {
            n,
            ..Default::default()
        };
        c.build_o(ll, counts);
        let t = &c.t32[..n];
        // F and ψ, checked: T's byte is L's, and L[ψ(j)] = F[j] (the carried byte's
        // orientation: a lane that reaches ψ(j) from j writes F[j] there)
        let mut f = vec![0u8; n];
        let mut j = 0;
        for (b, &k) in counts.iter().enumerate() {
            f[j..j + k as usize].fill(b as u8);
            j += k as usize;
        }
        assert_eq!(j, n, "the counts are the block's");
        let psi: Vec<u32> = t.iter().map(|&q| q >> 8).collect();
        for j in 0..n {
            assert_eq!(t[j] as u8, ll[j], "T's byte is L's");
            assert!((psi[j] as usize) < n);
            assert_eq!(ll[psi[j] as usize], f[j], "L[ψ(j)] = F[j]");
        }
        let groups = n.div_ceil(32);
        let mut r = Records {
            rec: vec![Line([0; 64]); groups],
            n,
            ..Default::default()
        };
        for g in 0..groups {
            let (lo, hi) = (32 * g, (32 * g + 32).min(n));
            let one = f[lo] == f[hi - 1];
            let (mn, mx) = psi[lo..hi]
                .iter()
                .fold((u32::MAX, 0), |(a, b), &x| (a.min(x), b.max(x)));
            let mut x = (g as u64 ^ 0x9e37_79b9_7f4a_7c15).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            x ^= x >> 31;
            let force = (x % 1000) < force_permille as u64;
            let fits = one && mx - mn < H_S;
            let h = if fits && !force {
                let line = &mut r.rec[g].0;
                for (i, &p) in psi[lo..hi].iter().enumerate() {
                    put14(line, i, p - mn);
                }
                if (lo..hi).all(|j| psi[j] == mn + (j - lo) as u32) {
                    r.run += 1;
                } else {
                    r.for14 += 1;
                }
                mn as u64 | (f[lo] as u64) << 20
            } else {
                let idx = r.raw_groups;
                r.rec[g] = H_ALL_S;
                r.raw.push(Line([0; 64]));
                r.raw.push(Line([0; 64]));
                for (i, j) in (lo..hi).enumerate() {
                    let e = psi[j] | (f[j] as u32) << 20;
                    let line = &mut r.raw[2 * idx + i / 16].0;
                    line[4 * (i % 16)..4 * (i % 16) + 4].copy_from_slice(&e.to_le_bytes());
                }
                r.raw_groups += 1;
                r.crossing += !one as usize;
                r.forced += fits as usize;
                idx as u64 | H_RAW
            };
            r.rec[g].0[56..64].copy_from_slice(&h.to_le_bytes());
        }
        let mut park = H_ALL_S;
        park.0[56..64].copy_from_slice(&H_RAW.to_le_bytes());
        r.rec.push(park);
        // every row decodes to its ψ and F (the decoder below, against the oracle)
        for j in 0..n {
            let (p, b) = unsafe { h_next(r.rec.as_ptr(), r.raw.as_ptr(), j as u32) };
            assert!(
                p == psi[j] && b == f[j],
                "row {j}'s record decodes to its ψ and F"
            );
        }
        r
    }

    /// The records' bytes (the parking record's line included) and raw area's
    pub fn bytes(&self) -> (usize, usize) {
        (64 * self.rec.len(), 64 * self.raw.len())
    }
}

/// Offset i's 14 bits in the payload, set to `v` (the others kept)
fn put14(line: &mut [u8; 64], i: usize, v: u32) {
    debug_assert!(v <= H_S);
    let at = 14 * i;
    let mut w = u32::from_le_bytes(line[at / 8..at / 8 + 4].try_into().unwrap());
    w = (w & !(H_S << (at % 8))) | v << (at % 8);
    line[at / 8..at / 8 + 4].copy_from_slice(&w.to_le_bytes());
}

/// Row `row`'s header
///
/// SAFETY (all three): `row` is below 32 × the records' count, `rec` a block's records
#[inline(always)]
unsafe fn h_header(rec: *const Line, row: u32) -> u64 {
    unsafe {
        u64::from_le(((rec.add((row >> 5) as usize) as *const u8).add(56) as *const u64).read())
    }
}

/// Row `row`'s 14-bit offset (S in a RAW record, at a start row while the walk runs, and in
/// the parking record)
#[inline(always)]
unsafe fn h_offset(rec: *const Line, row: u32) -> u32 {
    let at = 14 * (row & 31);
    let w = unsafe {
        ((rec.add((row >> 5) as usize) as *const u8).add((at >> 3) as usize) as *const u32)
            .read_unaligned()
    };
    (u32::from_le(w) >> (at & 7)) & H_S
}

/// ψ(row) and F[row], either format, from the records with no start marked: the walk's
/// starts (made before it marks them) and the plain walk
#[inline(always)]
unsafe fn h_next(rec: *const Line, raw: *const Line, row: u32) -> (u32, u8) {
    unsafe {
        let h = h_header(rec, row);
        if h & H_RAW != 0 {
            let e = u32::from_le(*(raw as *const u32).add(h_raw_index(h, row) as usize));
            (e & 0xf_ffff, (e >> 20) as u8)
        } else {
            ((h & H_MASK20) as u32 + h_offset(rec, row), (h >> 20) as u8)
        }
    }
}

/// Row `row`'s raw entry's index (in u32s) in the raw area, from its RAW header
#[inline(always)]
fn h_raw_index(h: u64, row: u32) -> u32 {
    32 * (h & H_MASK20) as u32 + (row & 31)
}

#[inline(always)]
fn prefetch_at(p: *const u8) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch never faults
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(p as *const i8)
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
}

/// `h`'s lane: `Lane` and the raw entry it waits on
#[derive(Clone, Copy)]
struct LaneH {
    /// a row, or PARK while the lane waits on its raw entry
    row: u32,
    /// the raw entry (u32 index) the lane waits on (read only while `row` is PARK)
    pend: u32,
    /// the byte the lane writes at its next step: L of `row`
    b: u8,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

/// What a lane's step found
enum Slow {
    /// a byte to write (the lane moved on)
    Byte(u8),
    /// the lane reached a start: its segment ends
    Start,
    /// the lane reached a raw row: it waits a turn, writing nothing
    Wait,
}

impl Compact {
    /// `h`'s build: a copy of the block's precomputed records into the worker's own,
    /// reused buffers, as a direct builder would leave them (written just before the
    /// walk). The optimistic stand-in for a builder (stage 1 measures traversal only).
    pub fn load_h(&mut self, r: &Records) {
        self.n = r.n;
        self.hrec.clear();
        self.hrec.extend_from_slice(&r.rec);
        self.hraw.clear();
        self.hraw.extend_from_slice(&r.raw);
    }

    fn walk_h(&mut self, ll: &[u8], orig_ptr: u32, out: &mut Vec<u8>) {
        assert_eq!(self.hrec.len(), self.n.div_ceil(32) + 1, "load_h first");
        if self.n < MIN_SEGMENTED {
            self.plain_h(orig_ptr, out);
        } else {
            self.room_h(ll, orig_ptr);
            self.s.assemble(out);
        }
    }

    /// production's plain walk, on records
    fn plain_h(&mut self, orig_ptr: u32, out: &mut Vec<u8>) {
        let n = self.n;
        let (rec, raw) = (self.hrec.as_ptr(), self.hraw.as_ptr());
        out.clear();
        out.reserve(n);
        // SAFETY: every ψ is below n, as is orig_ptr (checked by the stream's parser)
        unsafe {
            let (mut r, mut b) = h_next(rec, raw, orig_ptr);
            for _ in 0..n {
                out.push(b);
                let (r2, b2) = h_next(rec, raw, r);
                r = r2;
                b = b2;
            }
        }
    }

    /// `room` on records: the same starts, passes, handoffs and tail. An ordinary step
    /// reads its record's line and tests its offset against S; the sentinel's three cases
    /// (`slow`) are a start, a raw row (the lane prefetches the entry, writes nothing this
    /// turn and parks: its base moves back a byte) and a parked lane (it takes ψ and F
    /// from the entry). So a raw successor costs one turn and one line.
    #[inline(never)]
    fn room_h(&mut self, ll: &[u8], orig_ptr: u32) {
        let n = self.n;
        let park = 32 * (self.hrec.len() as u32 - 1);
        let recm = self.hrec.as_mut_ptr();
        let (rec, raw) = (recm as *const Line, self.hraw.as_ptr());
        let rawu = raw as *const u32;
        let s = &mut self.s;
        // SAFETY (every record and raw access below): every row is below n or is PARK, the
        // starts being rows and every ψ coming from the oracle's records
        let r0 = unsafe { h_next(rec, raw, orig_ptr) }.0;
        random_rows(n, SEGMENTS - 1, &mut s.starts);
        s.starts.push(r0);
        h_extra_starts(n, rec, &mut s.starts);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        // each start's ψ and F (read before it is marked), and its offset, to restore
        let succ: Vec<(u32, u8, u32)> = s
            .starts
            .iter()
            .map(|&r| unsafe {
                let (p, b) = h_next(rec, raw, r);
                (p, b, h_offset(rec, r))
            })
            .collect();
        let mark = |r: u32, v: u32, on: bool| unsafe {
            let line = &mut (*recm.add((r >> 5) as usize)).0;
            put14(line, (r & 31) as usize, v);
            let bit = 1u64 << (32 + (r & 31));
            let h = u64::from_le_bytes(line[56..64].try_into().unwrap());
            let h = if on { h | bit } else { h & !bit };
            line[56..64].copy_from_slice(&h.to_le_bytes());
        };
        for &r in &s.starts {
            mark(r, H_S, true);
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let bufs = &mut s.bufs;
        let next = &mut s.next;
        let line = |row: u32| unsafe { rec.add((row >> 5) as usize) as *const u8 };

        let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> LaneH {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let (r, f, _) = succ[seg];
            // SAFETY: the buffer has `cap` > 0 spare bytes
            unsafe { *base = ll[starts[seg] as usize] };
            LaneH {
                row: r,
                pend: 0,
                b: f,
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
            }
        };
        let finish = |l: &LaneH, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            // SAFETY: p is within b's allocation, every byte below it written
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut LaneH, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };
        // the sentinel's cases, off the common path
        let slow = |l: &mut LaneH, h: u64| -> Slow {
            if l.row == park {
                let e = u32::from_le(unsafe { *rawu.add(l.pend as usize) });
                let byte = l.b;
                l.b = (e >> 20) as u8;
                l.row = e & 0xf_ffff;
                Slow::Byte(byte)
            } else if (h >> (32 + (l.row & 31))) & 1 != 0 {
                Slow::Start
            } else {
                debug_assert!(h & H_RAW != 0);
                l.pend = h_raw_index(h, l.row);
                prefetch_at(unsafe { rawu.add(l.pend as usize) } as *const u8);
                l.row = park;
                Slow::Wait
            }
        };

        let mut queued = 0;
        let mut lanes: [LaneH; LANES] = std::array::from_fn(|_| LaneH {
            row: 0,
            pend: 0,
            b: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < LANES && queued < ns {
            lanes[active] = take(queued, bufs);
            prefetch_at(line(lanes[active].row));
            queued += 1;
            active += 1;
        }
        if active == LANES {
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                debug_assert!(room > 0);
                let mut base: [*mut u8; LANES] = std::array::from_fn(|k| lanes[k].p);
                for off in 0..room {
                    for k in 0..LANES {
                        let l = &mut lanes[k];
                        let h = unsafe { h_header(rec, l.row) };
                        let o = unsafe { h_offset(rec, l.row) };
                        let byte = if o != H_S {
                            let byte = l.b;
                            l.b = (h >> 20) as u8;
                            l.row = (h & H_MASK20) as u32 + o;
                            byte
                        } else {
                            match slow(l, h) {
                                Slow::Byte(b) => b,
                                Slow::Start => {
                                    l.p = base[k].wrapping_add(off);
                                    finish(l, bufs, next);
                                    if queued == ns {
                                        for (j, lj) in lanes.iter_mut().enumerate() {
                                            if j < k {
                                                lj.p = base[j].wrapping_add(off + 1);
                                            } else if j > k {
                                                lj.p = base[j].wrapping_add(off);
                                            }
                                        }
                                        break 'full;
                                    }
                                    *l = take(queued, bufs);
                                    prefetch_at(line(l.row));
                                    queued += 1;
                                    base[k] = l.p.wrapping_sub(off + 1);
                                    continue;
                                }
                                Slow::Wait => {
                                    // nothing written this turn: the lane's next byte
                                    // goes where this one would have
                                    base[k] = base[k].wrapping_sub(1);
                                    continue;
                                }
                            }
                        };
                        prefetch_at(line(l.row));
                        debug_assert!(base[k].wrapping_add(off) < l.end);
                        unsafe { *base[k].wrapping_add(off) = byte };
                    }
                }
                for (l, b) in lanes.iter_mut().zip(base) {
                    l.p = b.wrapping_add(room);
                }
            }
        }
        let mut live: Vec<LaneH> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| l.seg != usize::MAX && next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
            debug_assert!(l.p < l.end);
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let h = unsafe { h_header(rec, l.row) };
                let o = unsafe { h_offset(rec, l.row) };
                let byte = if o != H_S {
                    let byte = l.b;
                    l.b = (h >> 20) as u8;
                    l.row = (h & H_MASK20) as u32 + o;
                    byte
                } else {
                    match slow(l, h) {
                        Slow::Byte(b) => b,
                        Slow::Start => {
                            finish(l, bufs, next);
                            live.swap_remove(i);
                            continue;
                        }
                        Slow::Wait => {
                            i += 1;
                            continue;
                        }
                    }
                };
                prefetch_at(line(l.row));
                unsafe {
                    *l.p = byte;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        for (&r, &(_, _, o)) in starts.iter().zip(&succ) {
            mark(r, o, false);
        }
        s.first = seg_of(r0);
        s.n = n;
    }
}

/// COOP_H_STARTS=1 (checks only, never a timed run): segment starts the random ones rarely
/// give, added to them: rows 0, 1, n − 1, adjacent pairs, a run of 40 consecutive rows
/// (one-byte segments), and the first two rows of the first 16 RAW groups (a segment
/// that starts, or ends, at a raw row)
fn h_extra_starts(n: usize, rec: *const Line, starts: &mut Vec<u32>) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("COOP_H_STARTS").is_ok_and(|v| v == "1")) {
        return;
    }
    let n32 = n as u32;
    starts.extend([0, 1, n32 - 1, n32 / 2, n32 / 2 + 1, n32 / 3, n32 / 3 + 1]);
    starts.extend((0..40).map(|i| (n32 / 5 + i).min(n32 - 1)));
    let mut found = 0;
    for g in 0..n.div_ceil(32) {
        // SAFETY: g is a record of the block's
        let h = unsafe { h_header(rec, 32 * g as u32) };
        if h & H_RAW != 0 {
            starts.push(32 * g as u32);
            if 32 * g + 1 < n {
                starts.push(32 * g as u32 + 1);
            }
            found += 1;
            if found == 16 {
                break;
            }
        }
    }
}
