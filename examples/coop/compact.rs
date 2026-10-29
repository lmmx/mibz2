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
            _ => unreachable!("compact arms are o, v and u"),
        }
    }

    /// The block's output into `out`, from the table `build` made
    pub fn walk(&mut self, kind: char, ll: &[u8], orig_ptr: u32, out: &mut Vec<u8>) {
        match kind {
            'o' => self.walk_k::<O>(ll, orig_ptr, out),
            'v' => self.walk_k::<V>(ll, orig_ptr, out),
            'u' => self.walk_k::<U>(ll, orig_ptr, out),
            _ => unreachable!("compact arms are o, v and u"),
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
