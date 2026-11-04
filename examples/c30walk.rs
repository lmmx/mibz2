//! Refined compact30: one experimental walker (external reports 165–166). Compile and
//! correctness only: untimed, oracle-built records, no direct constructor, not wired into
//! the decoder.
//!
//!     cargo run --release --example c30walk -- check OUT.jsonl FILE.bz2 [FILE.bz2 ...]
//!     cargo run --release --example c30walk -- adversarial OUT.jsonl
//!     cargo run --release --example c30walk -- audit OUT.jsonl FILE.bz2 [FILE.bz2 ...]
//!     cargo run --release --example c30walk -- audit-adversarial OUT.jsonl
//!     cargo run --release --example c30walk -- grid OUT.jsonl CORPUS.bz2 --rounds R --cpus A,B,C,D [--dry-run]
//!     cargo run --release --example c30walk -- barrier-test [--cpus A,B,C,D] [--steps N]
//!     cargo run --release --example c30walk -- ctor-check OUT.jsonl FILE.bz2 [FILE.bz2 ...]
//!     cargo run --release --example c30walk -- ctor-adversarial OUT.jsonl
//!     cargo run --release --example c30walk -- ctor-synthetic [--quick]
//!     cargo run --release --example c30walk -- cbuild OUT.jsonl CORPUS.bz2 --rounds R --cpus A,B,C,D [--dry-run]
//!     cargo run --release --example c30walk -- cbuild-rss OUT.json CORPUS.bz2 --arm P|C --cpus A,B,C,D
//!     cargo run --release --example c30walk -- cbuild-v2 OUT.jsonl CORPUS.bz2 --rounds R --cpus A,B,C,D [--dry-run]
//!     cargo run --release --example c30walk -- cbuild-rss-v2 OUT.json CORPUS.bz2 --arm P|C --cpus A,B,C,D
//!
//! The `ctor-` modes check the direct constructor (`c30walk/ctor.rs`, report 188) against
//! the oracle.
//!
//! **The representation:**
//! - **Records:** one 64-byte, 64-aligned record per 30 rows in F order (`Rec`): 30 `u16`
//!   fields and a `u32` header at byte 60.
//! - **Labels:** row r is labelled phi(r) = 32⌊r/30⌋ + r mod 30. Its field is at byte
//!   2·phi and its record's header at 2·(phi & !31) + 60. Successors are stored as labels.
//! - **An ordinary record:** header H = (B << 8) | F, where B is the least successor label
//!   of its 30 rows and F their common first-column byte. A field holds an offset ≤ 65,533,
//!   so the successor is (H >> 8) + offset and the row's output byte is H's low byte.
//! - **A RAW record:** header RAW_BIT | group index; every field 0xFFFF. Its group's 32
//!   `u32` entries are (successor label << 8) | F[row] (`RawG`, 128 bytes).
//! - **Markers:** a field of 0xFFFE marks a segment start; the walk saves the field and
//!   restores it at the end.
//! - **Parking:** one record of 0xFFFF fields after the last, at label PARK = 32·G.
//!
//! **The emission convention (report 165):** F[r] = L[Ψ(r)], so production's output
//! L[Ψ(p)], L[Ψ²(p)], … from orig_ptr = p equals F[p], F[Ψ(p)], …. A lane at row x emits
//! F[x] and moves to Ψ(x). Its state is the plain encoded row, with no carried byte. The
//! first segment starts at phi(orig_ptr).
//!
//! **The walk:** production's room walk (32 lanes, 511 random starts plus the first, room
//! passes, replacement at a start, the tail), as the FOR-14 k arm copies it, with L1
//! prefetch of the successor's record.
//! - **An ordinary step:** load the field and the header (both addressed from x alone);
//!   if field < 0xFFFE, write H's low byte, set row = (H >> 8) + field, and prefetch.
//! - **A slow step:**
//!   - START: the segment ends, emitting nothing for the marked row;
//!   - PARK: the lane resumes, writing the pending entry's low byte and moving to its
//!     successor;
//!   - otherwise a RAW row: it saves the entry's index, prefetches it, parks and emits
//!     nothing this turn.
//!
//! **`room::<false>`** is the walk audited in the disassembly. **`room::<true>`** is the
//! same walk with transition counters. Both are checked byte for byte against production
//! (`bwt::Segmented`).

use mibz2::bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The direct constructor and its correctness harness (report 188)
#[path = "c30walk/ctor.rs"]
mod ctor;

/// The construction-cost collector (docs/journal/2026-11-03-compact30-construction-protocol.md)
#[path = "c30walk/cbuild.rs"]
mod cbuild;

/// Constructor v2, deferred offsets (report 205), and its correctness harness
#[path = "c30walk/ctor2.rs"]
mod ctor2;

/// Charter prototype 2: v2 with its descriptor split into hot pointer arrays
#[path = "c30walk/ctor2d.rs"]
mod ctor2d;

/// Constructor v3: v2 with the epoch-tagged ordinary finaliser (reports 227, 235, 236)
#[path = "c30walk/ctor3.rs"]
mod ctor3;

const LANES: usize = 32;
const SEGMENTS: usize = 512;
const MIN_SEGMENTED: usize = 1 << 16;
const ROWS: usize = 30;
const START: u16 = 0xFFFE;
const RAWF: u16 = 0xFFFF;
const MAX_OFF: u32 = 65_533;
const RAW_BIT: u32 = 1 << 28;
const M20: u32 = (1 << 20) - 1;

/// One record: 30 fields, the header at byte 60
#[repr(C, align(64))]
#[derive(Clone, Copy, PartialEq, Eq)]
struct Rec {
    f: [u16; 30],
    h: u32,
}

/// A RAW group's entries: 30 used, padded to 32
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct RawG([u32; 32]);

const _: () = {
    assert!(std::mem::size_of::<Rec>() == 64);
    assert!(std::mem::align_of::<Rec>() == 64);
    assert!(std::mem::offset_of!(Rec, h) == 60);
    assert!(std::mem::offset_of!(Rec, f) == 0);
    assert!(std::mem::size_of::<RawG>() == 128);
};

fn phi(r: usize) -> u32 {
    (32 * (r / ROWS) + r % ROWS) as u32
}

fn phi_inv(x: u32) -> usize {
    ROWS * (x >> 5) as usize + (x & 31) as usize
}

/// Production's segment starts (examples/coop.rs, `random_rows`)
fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
    rows.clear();
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    rows.extend((0..m).map(|_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x % n as u64) as u32
    }));
}

#[inline(always)]
fn prefetch(p: *const u8) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch never faults
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(p as *const i8)
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
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

/// A lane: its encoded row, the RAW entry it waits on, its segment and output pointers
/// (32 bytes)
#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    pend: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

const _: () = assert!(std::mem::size_of::<Lane>() == 32);

/// The walk's transition counts (`room::<true>`)
#[derive(Default, Clone, Copy, Debug)]
struct Counts {
    raw_arrivals: u64,
    resumptions: u64,
    wait_turns_full: u64,
    wait_turns_tail: u64,
    starts_hit: u64,
    replacements: u64,
    grows_full: u64,
    grows_tail: u64,
    tail_lanes: u64,
    midround_ends: u64,
    one_byte_segments: u64,
    raw_starts: u64,
    segments: u64,
    emitted: u64,
}

impl Counts {
    fn add(&mut self, o: &Counts) {
        self.raw_arrivals += o.raw_arrivals;
        self.resumptions += o.resumptions;
        self.wait_turns_full += o.wait_turns_full;
        self.wait_turns_tail += o.wait_turns_tail;
        self.starts_hit += o.starts_hit;
        self.replacements += o.replacements;
        self.grows_full += o.grows_full;
        self.grows_tail += o.grows_tail;
        self.tail_lanes += o.tail_lanes;
        self.midround_ends += o.midround_ends;
        self.one_byte_segments += o.one_byte_segments;
        self.raw_starts += o.raw_starts;
        self.segments += o.segments;
        self.emitted += o.emitted;
    }
}

/// What a slow step found
enum Slow {
    Byte(u8),
    Start,
    Wait,
}

/// The candidate's storage (records and RAW groups) and its walk scratch
#[derive(Default)]
struct C30 {
    n: usize,
    rec: Vec<Rec>,
    raw: Vec<RawG>,
    s: Scratch,
    /// the marked starts' saved fields, and their (successor << 8) | F
    saved: Vec<u16>,
    succ: Vec<u32>,
}

#[derive(Default, Clone, Copy, Debug)]
struct Build {
    groups: usize,
    ordinary: usize,
    raw_crossing: usize,
    raw_wide: usize,
    raw_forced: usize,
    logical_bytes: usize,
    reserved_bytes: usize,
    oracle_temp_bytes: usize,
    bound: usize,
}

impl C30 {
    /// The records from Ψ and F (an oracle: Ψ by counting scatter, as `bwt::build`
    /// orders it). `permille` of the groups that would be ordinary are forced RAW
    /// instead, for the exceptional paths' tests.
    fn build(&mut self, ll: &[u8], counts: &[u32; 256], permille: u32) -> Build {
        let n = ll.len();
        self.n = n;
        // oracle-only data, dropped at the end of this function
        let mut ftab = [0usize; 256];
        let mut cum = 0;
        for c in 0..256 {
            ftab[c] = cum;
            cum += counts[c] as usize;
        }
        assert_eq!(cum, n);
        let mut psi = vec![0u32; n];
        let mut f = vec![0u8; n];
        for (i, &c) in ll.iter().enumerate() {
            let s = ftab[c as usize];
            psi[s] = phi(i);
            f[s] = c;
            ftab[c as usize] += 1;
        }
        let g_count = n.div_ceil(ROWS);
        let park = 32 * g_count as u32;
        assert!(park <= M20, "PARK fits 20 bits");
        // classify first, so the RAW area is reserved once
        let mut kind = vec![0u8; g_count]; // 0 ordinary, 1 crossing, 2 wide, 3 forced
        let mut b = Build {
            groups: g_count,
            oracle_temp_bytes: 5 * n + g_count,
            ..Default::default()
        };
        for (g, k) in kind.iter_mut().enumerate() {
            let (a, e) = (g * ROWS, ((g + 1) * ROWS).min(n));
            *k = if f[a..e].iter().any(|&x| x != f[a]) {
                1
            } else if psi[e - 1] - psi[a] > MAX_OFF {
                2
            } else if permille > 0 && (mix(g as u64 ^ (n as u64) << 32) % 1000) < permille as u64 {
                3
            } else {
                0
            };
        }
        let r_count = kind.iter().filter(|&&k| k != 0).count();
        self.rec.clear();
        self.rec.reserve_exact(g_count + 1);
        self.raw.clear();
        self.raw.reserve_exact(r_count);
        for (g, &k) in kind.iter().enumerate() {
            let (a, e) = (g * ROWS, ((g + 1) * ROWS).min(n));
            let mut r = Rec {
                f: [RAWF; 30],
                h: 0,
            };
            for &x in &psi[a..e] {
                // no successor is a header position or a row past the block
                assert!(x & 31 < 30 && phi_inv(x) < n, "successor label {x}");
            }
            if k == 0 {
                let base = psi[a];
                assert!(base <= M20);
                r.h = base << 8 | f[a] as u32;
                for j in 0..e - a {
                    let o = psi[a + j] - base;
                    assert!(o <= MAX_OFF);
                    r.f[j] = o as u16;
                    assert_eq!((r.h >> 8) + o, psi[a + j]);
                    assert_eq!(r.h as u8, f[a + j]);
                }
                b.ordinary += 1;
            } else {
                let idx = self.raw.len() as u32;
                r.h = RAW_BIT | idx;
                let mut gr = RawG([0; 32]);
                for j in 0..e - a {
                    gr.0[j] = psi[a + j] << 8 | f[a + j] as u32;
                }
                self.raw.push(gr);
                match k {
                    1 => b.raw_crossing += 1,
                    2 => b.raw_wide += 1,
                    _ => b.raw_forced += 1,
                }
            }
            self.rec.push(r);
        }
        // parking: 0xFFFF fields, a dummy RAW header the walk never dereferences
        self.rec.push(Rec {
            f: [RAWF; 30],
            h: RAW_BIT | M20,
        });
        assert_eq!(
            self.rec.as_ptr() as usize % 64,
            0,
            "records 64-byte aligned"
        );
        assert_eq!(
            self.raw.as_ptr() as usize % 64,
            0,
            "RAW groups 64-byte aligned"
        );
        b.logical_bytes = 64 * (g_count + 1) + 128 * r_count;
        b.reserved_bytes = 64 * self.rec.capacity() + 128 * self.raw.capacity();
        let sigma = counts.iter().filter(|&&c| c > 0).count();
        b.bound = g_count.min(sigma * (phi(n - 1) as usize / 65_534) + sigma.saturating_sub(1));
        b
    }

    fn park(&self) -> u32 {
        32 * (self.rec.len() as u32 - 1)
    }

    /// Ψ(x) and F[x] from the records, no start marked
    fn next(&self, x: u32) -> (u32, u8) {
        let r = &self.rec[(x >> 5) as usize];
        let o = r.f[(x & 31) as usize];
        if o < START {
            ((r.h >> 8) + o as u32, r.h as u8)
        } else {
            debug_assert!(r.h & RAW_BIT != 0);
            let e = self.raw[(r.h & M20) as usize].0[(x & 31) as usize];
            (e >> 8, e as u8)
        }
    }

    fn is_raw(&self, x: u32) -> bool {
        self.rec[(x >> 5) as usize].f[(x & 31) as usize] == RAWF
    }

    /// The plain walk, for a block under MIN_SEGMENTED rows
    fn plain(&self, orig_ptr: u32, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(self.n);
        let mut x = phi(orig_ptr as usize);
        for _ in 0..self.n {
            let (s, b) = self.next(x);
            out.push(b);
            x = s;
        }
    }

    /// The segments' starts, as labels: production's random rows and the first, and with
    /// `extra` the adversarial ones:
    /// - fixed rows;
    /// - chain-consecutive triples (one-byte segments);
    /// - the first rows of RAW groups (RAW starts);
    /// - rows whose successor is RAW.
    fn starts(&mut self, orig_ptr: u32, extra: bool) {
        let n = self.n;
        let mut rows = Vec::new();
        random_rows(n, SEGMENTS - 1, &mut rows);
        let mut st: Vec<u32> = rows.iter().map(|&r| phi(r as usize)).collect();
        st.push(phi(orig_ptr as usize));
        if extra {
            let n32 = n as u32;
            for r in [0, 1, n32 - 1, n32 / 2, n32 / 2 + 1, n32 / 3, n32 / 3 + 1] {
                st.push(phi(r as usize));
            }
            st.extend((0..40).map(|i| phi(((n32 / 5 + i).min(n32 - 1)) as usize)));
            for i in 1..9 {
                let mut x = phi(n * i / 9);
                for _ in 0..3 {
                    st.push(x);
                    x = self.next(x).0;
                }
            }
            let mut found = 0;
            for g in 0..self.rec.len() - 1 {
                if self.rec[g].h & RAW_BIT != 0 {
                    st.push(32 * g as u32);
                    if phi_inv(32 * g as u32 + 1) < n {
                        st.push(32 * g as u32 + 1);
                    }
                    found += 1;
                    if found == 16 {
                        break;
                    }
                }
            }
            let mut found = 0;
            for r in (0..n).step_by(97) {
                let x = phi(r);
                if self.is_raw(self.next(x).0) {
                    st.push(x);
                    found += 1;
                    if found == 16 {
                        break;
                    }
                }
            }
        }
        st.sort_unstable();
        st.dedup();
        self.s.starts = st;
    }

    /// The segmented walk into the segments, then their assembly
    fn walk<const COUNT: bool>(
        &mut self,
        orig_ptr: u32,
        extra: bool,
        out: &mut Vec<u8>,
        c: &mut Counts,
    ) {
        if self.n < MIN_SEGMENTED {
            self.plain(orig_ptr, out);
            if COUNT {
                c.emitted += out.len() as u64;
            }
            return;
        }
        self.starts(orig_ptr, extra);
        let before: Vec<Rec> = if COUNT { self.rec.clone() } else { Vec::new() };
        self.room::<COUNT>(orig_ptr, c);
        if COUNT {
            assert!(self.rec == before, "a marker not restored");
            c.emitted += self.s.bufs[..self.s.starts.len()]
                .iter()
                .map(|b| b.len() as u64)
                .sum::<u64>();
            c.segments += self.s.starts.len() as u64;
            for (i, b) in self.s.bufs[..self.s.starts.len()].iter().enumerate() {
                c.one_byte_segments += (b.len() == 1) as u64;
                c.raw_starts += self.is_raw(self.s.starts[i]) as u64;
            }
        }
        self.s.assemble(out);
    }

    /// The room walk. Starts are in `self.s.starts`; each is marked 0xFFFE (its field
    /// saved) and restored after the tail.
    #[inline(never)]
    fn room<const COUNT: bool>(&mut self, orig_ptr: u32, c: &mut Counts) {
        let n = self.n;
        let park = self.park();
        let ns = self.s.starts.len();
        // each start's (successor << 8) | F and its field, before marking
        self.succ.clear();
        self.saved.clear();
        for i in 0..ns {
            let x = self.s.starts[i];
            let (s, b) = self.next(x);
            self.succ.push(s << 8 | b as u32);
            self.saved
                .push(self.rec[(x >> 5) as usize].f[(x & 31) as usize]);
        }
        for i in 0..ns {
            let x = self.s.starts[i];
            self.rec[(x >> 5) as usize].f[(x & 31) as usize] = START;
        }
        let rec = self.rec.as_ptr() as *const u8;
        let raw = self.raw.as_ptr() as *const u32;
        // the physical-access audit (report 167), checked in debug builds: every field and
        // header read is inside the records' initialised storage, and every RAW entry read
        // is inside the RAW area
        let (rec_bytes, raw_entries) = (64 * self.rec.len(), 32 * self.raw.len());
        let in_rec = move |x: u32| {
            debug_assert!(x & 31 < 30, "label {x} on a header position");
            debug_assert!(
                2 * (x as usize) + 2 <= rec_bytes,
                "field of {x} past the records"
            );
            debug_assert!(
                2 * (x & !31) as usize + 64 <= rec_bytes,
                "header of {x} past the records"
            );
        };
        let s = &mut self.s;
        let succ = &self.succ;
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = |x: u32| starts.binary_search(&x).unwrap_or(usize::MAX);
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        // SAFETY (every record and RAW access below): a lane's row is a live label or
        // PARK, so its field, its record's header and any RAW index are in bounds
        let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            // SAFETY: the buffer has `cap` > 0 spare bytes
            unsafe { *base = succ[seg] as u8 };
            Lane {
                row: succ[seg] >> 8,
                pend: 0,
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
        // the sentinel's three cases; START is tested first, PARK before any RAW index
        let slow = |l: &mut Lane, o: u16, h: u32, c: &mut Counts| -> Slow {
            let x = l.row;
            if o == START {
                if COUNT {
                    c.starts_hit += 1;
                }
                Slow::Start
            } else if x == park {
                debug_assert!(
                    (l.pend as usize) < raw_entries,
                    "a pending entry past the RAW area"
                );
                let e = unsafe { *raw.add(l.pend as usize) };
                l.row = e >> 8;
                if COUNT {
                    c.resumptions += 1;
                }
                Slow::Byte(e as u8)
            } else {
                debug_assert!(h & RAW_BIT != 0 && o == RAWF);
                // a RAW row's own header, never the parking record's dummy index
                debug_assert!(x != park);
                l.pend = 32 * (h & M20) + (x & 31);
                debug_assert!(
                    (l.pend as usize) < raw_entries,
                    "a RAW index past the RAW area"
                );
                prefetch(unsafe { raw.add(l.pend as usize) } as *const u8);
                l.row = park;
                if COUNT {
                    c.raw_arrivals += 1;
                }
                Slow::Wait
            }
        };

        let mut queued = 0;
        let mut lanes: [Lane; LANES] = std::array::from_fn(|_| Lane {
            row: 0,
            pend: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0;
        while active < LANES && queued < ns {
            lanes[active] = take(queued, bufs);
            prefetch(unsafe { rec.add(2 * lanes[active].row as usize) });
            queued += 1;
            active += 1;
        }
        if active == LANES {
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                        if COUNT {
                            c.grows_full += 1;
                        }
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                debug_assert!(room > 0);
                let mut base: [*mut u8; LANES] = std::array::from_fn(|k| lanes[k].p);
                for off in 0..room {
                    for k in 0..LANES {
                        let l = &mut lanes[k];
                        let x = l.row;
                        in_rec(x);
                        // the ordinary step: field and header, both addressed from x
                        let o = unsafe { *(rec.add(2 * x as usize) as *const u16) };
                        let h = unsafe { *(rec.add(2 * (x & !31) as usize + 60) as *const u32) };
                        let byte = if o < START {
                            l.row = (h >> 8) + o as u32;
                            h as u8
                        } else {
                            match slow(l, o, h, c) {
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
                                        if COUNT {
                                            c.midround_ends += 1;
                                        }
                                        break 'full;
                                    }
                                    *l = take(queued, bufs);
                                    prefetch(unsafe { rec.add(2 * l.row as usize) });
                                    queued += 1;
                                    base[k] = l.p.wrapping_sub(off + 1);
                                    if COUNT {
                                        c.replacements += 1;
                                    }
                                    continue;
                                }
                                Slow::Wait => {
                                    base[k] = base[k].wrapping_sub(1);
                                    if COUNT {
                                        c.wait_turns_full += 1;
                                    }
                                    continue;
                                }
                            }
                        };
                        prefetch(unsafe { rec.add(2 * l.row as usize) });
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
        if COUNT {
            c.tail_lanes += live.len() as u64;
        }
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
                if COUNT {
                    c.grows_tail += 1;
                }
            }
            debug_assert!(l.p < l.end);
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let x = l.row;
                in_rec(x);
                let o = unsafe { *(rec.add(2 * x as usize) as *const u16) };
                let h = unsafe { *(rec.add(2 * (x & !31) as usize + 60) as *const u32) };
                let byte = if o < START {
                    l.row = (h >> 8) + o as u32;
                    h as u8
                } else {
                    match slow(l, o, h, c) {
                        Slow::Byte(b) => b,
                        Slow::Start => {
                            finish(l, bufs, next);
                            live.swap_remove(i);
                            continue;
                        }
                        Slow::Wait => {
                            if COUNT {
                                c.wait_turns_tail += 1;
                            }
                            i += 1;
                            continue;
                        }
                    }
                };
                prefetch(unsafe { rec.add(2 * l.row as usize) });
                unsafe {
                    *l.p = byte;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                    if COUNT {
                        c.grows_tail += 1;
                    }
                }
                i += 1;
            }
        }
        // the markers stay until every segment has ended; restore them now
        for i in 0..ns {
            let x = self.s.starts[i];
            self.rec[(x >> 5) as usize].f[(x & 31) as usize] = self.saved[i];
        }
        self.s.first = self
            .s
            .starts
            .binary_search(&phi(orig_ptr as usize))
            .expect("the first segment's start");
        self.s.n = n;
    }
}

fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^ (x >> 33)
}

/// A block as the harness holds it: its last column, counts, primary index and
/// production's output
type Block = (Vec<u8>, [u32; 256], u32, Vec<u8>);

/// Every block of `data`
fn blocks(data: &[u8]) -> Vec<Block> {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let mut inv = Segmented::default();
    let mut v = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        let (mut ll, mut counts, mut want) = (Vec::new(), [0u32; 256], Vec::new());
        mibz2::mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
        inv.invert(&ll, &counts, symbols.orig_ptr, &mut want);
        v.push((ll, counts, symbols.orig_ptr, want));
        syms = symbols.syms;
    }
    v
}

/// The configurations each input is checked under: (forced-RAW permille, adversarial
/// starts)
const CONFIGS: [(u32, bool); 4] = [(0, false), (0, true), (50, true), (1000, true)];

/// Check every block under every configuration, both walks; one JSON line per
/// configuration
fn check(name: &str, data: &[u8], jl: &mut std::fs::File) {
    let bs = blocks(data);
    for &(pm, extra) in &CONFIGS {
        let next = AtomicUsize::new(0);
        let threads = 3usize;
        let results: Vec<(Counts, Build, usize, usize, bool)> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..threads)
                .map(|_| {
                    sc.spawn(|| {
                        let mut w = C30::default();
                        let mut out = Vec::new();
                        let (mut c, mut b) = (Counts::default(), Build::default());
                        let (mut seg_blocks, mut rows) = (0, 0);
                        let mut bound_ok = true;
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some((ll, counts, op, want)) = bs.get(i) else {
                                break;
                            };
                            let bb = w.build(ll, counts, pm);
                            if pm == 0 {
                                bound_ok &= bb.raw_crossing + bb.raw_wide <= bb.bound;
                            }
                            for (x, y) in [
                                (&mut b.groups, bb.groups),
                                (&mut b.ordinary, bb.ordinary),
                                (&mut b.raw_crossing, bb.raw_crossing),
                                (&mut b.raw_wide, bb.raw_wide),
                                (&mut b.raw_forced, bb.raw_forced),
                                (&mut b.logical_bytes, bb.logical_bytes),
                                (&mut b.reserved_bytes, bb.reserved_bytes),
                                (&mut b.oracle_temp_bytes, bb.oracle_temp_bytes),
                            ] {
                                *x += y;
                            }
                            let mut ci = Counts::default();
                            w.walk::<true>(*op, extra, &mut out, &mut ci);
                            assert!(
                                &out == want,
                                "{name} pm {pm} extra {extra} block {i}: counted walk differs"
                            );
                            c.add(&ci);
                            let mut dummy = Counts::default();
                            w.walk::<false>(*op, extra, &mut out, &mut dummy);
                            assert!(
                                &out == want,
                                "{name} pm {pm} extra {extra} block {i}: audited walk differs"
                            );
                            seg_blocks += (ll.len() >= MIN_SEGMENTED) as usize;
                            rows += ll.len();
                        }
                        (c, b, seg_blocks, rows, bound_ok)
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let (mut c, mut b) = (Counts::default(), Build::default());
        let (mut segb, mut rows, mut bound_ok) = (0, 0, true);
        for (ci, bi, s, r, ok) in results {
            c.add(&ci);
            for (x, y) in [
                (&mut b.groups, bi.groups),
                (&mut b.ordinary, bi.ordinary),
                (&mut b.raw_crossing, bi.raw_crossing),
                (&mut b.raw_wide, bi.raw_wide),
                (&mut b.raw_forced, bi.raw_forced),
                (&mut b.logical_bytes, bi.logical_bytes),
                (&mut b.reserved_bytes, bi.reserved_bytes),
                (&mut b.oracle_temp_bytes, bi.oracle_temp_bytes),
            ] {
                *x += y;
            }
            segb += s;
            rows += r;
            bound_ok &= ok;
        }
        assert!(
            bound_ok,
            "{name}: the RAW-count bound fails at natural storage"
        );
        writeln!(
            jl,
            "{{\"input\":{name:?},\"permille\":{pm},\"extra_starts\":{extra},\"blocks\":{},\"segmented_blocks\":{segb},\"rows\":{rows},\"emitted\":{},\"groups\":{},\"ordinary\":{},\"raw_crossing\":{},\"raw_wide\":{},\"raw_forced\":{},\"logical_bytes\":{},\"reserved_bytes\":{},\"oracle_temp_bytes\":{},\"segments\":{},\"starts_hit\":{},\"replacements\":{},\"midround_ends\":{},\"raw_arrivals\":{},\"resumptions\":{},\"wait_turns_full\":{},\"wait_turns_tail\":{},\"grows_full\":{},\"grows_tail\":{},\"tail_lanes\":{},\"one_byte_segments\":{},\"raw_starts\":{}}}",
            bs.len(),
            c.emitted,
            b.groups,
            b.ordinary,
            b.raw_crossing,
            b.raw_wide,
            b.raw_forced,
            b.logical_bytes,
            b.reserved_bytes,
            b.oracle_temp_bytes,
            c.segments,
            c.starts_hit,
            c.replacements,
            c.midround_ends,
            c.raw_arrivals,
            c.resumptions,
            c.wait_turns_full,
            c.wait_turns_tail,
            c.grows_full,
            c.grows_tail,
            c.tail_lanes,
            c.one_byte_segments,
            c.raw_starts
        )
        .unwrap();
        println!(
            "{name}: permille {pm}, extra starts {extra}: {} blocks ({segb} segmented), {rows} rows, emitted {}; RAW arrivals {}, resumptions {}, wait turns {} + {} (tail); starts hit {}, replacements {}, grows {} + {} (tail), one-byte segments {}, RAW starts {}: both walks exact",
            bs.len(),
            c.emitted,
            c.raw_arrivals,
            c.resumptions,
            c.wait_turns_full,
            c.wait_turns_tail,
            c.starts_hit,
            c.replacements,
            c.grows_full,
            c.grows_tail,
            c.one_byte_segments,
            c.raw_starts
        );
    }
}

/// The audited walk's own validation (reports 172–173), separate from `check`:
/// - for every block, a **fresh** `C30` (fresh records and scratch) runs `walk::<false>`;
/// - its output is compared with production's, and its records with a snapshot taken
///   after the build (every marker restored);
/// - it then runs `walk::<false>` **again on the same records, without rebuilding**, with
///   both checks repeated.
///
/// Segment-buffer growth inside the audited walk is read from the scratch afterwards,
/// with no counter in the walk: a fresh segment buffer is reserved exactly `cap` bytes
/// (2n/ns + 64, the walk's own reserve on an empty `Vec`), so a capacity above `cap`
/// after the cold walk means the audited walk grew it. In the repeated walk, a capacity
/// above its value before the walk means it grew again.
///
/// The counted transition audit runs on a **separate** instance per thread (as in
/// `check`), so it can't prepare the audited instance's scratch.
fn audit(name: &str, data: &[u8], jl: &mut std::fs::File) {
    let bs = blocks(data);
    for &(pm, extra) in &CONFIGS {
        let next = AtomicUsize::new(0);
        let threads = 3usize;
        // (segmented blocks, cold-grown segments, cold blocks with growth, warm-grown
        // segments, restoration checks, output checks, counted RAW arrivals)
        let results: Vec<[u64; 7]> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..threads)
                .map(|_| {
                    sc.spawn(|| {
                        let mut counted = C30::default();
                        let mut out = Vec::new();
                        let mut t = [0u64; 7];
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some((ll, counts, op, want)) = bs.get(i) else { break };
                            // the audited walk, on fresh records and scratch
                            let mut a = C30::default();
                            a.build(ll, counts, pm);
                            let snap = a.rec.clone();
                            let mut d = Counts::default();
                            a.walk::<false>(*op, extra, &mut out, &mut d);
                            assert!(&out == want, "{name} pm {pm} extra {extra} block {i}: cold audited walk differs");
                            assert!(a.rec == snap, "{name} pm {pm} extra {extra} block {i}: cold audited walk left a marker");
                            t[4] += 1;
                            t[5] += 1;
                            let segmented = ll.len() >= MIN_SEGMENTED;
                            let ns = a.s.starts.len();
                            let caps: Vec<usize> = if segmented {
                                let cap = 2 * ll.len() / ns + 64;
                                let caps: Vec<usize> =
                                    a.s.bufs[..ns].iter().map(|b| b.capacity()).collect();
                                let grown = caps.iter().filter(|&&c| c > cap).count() as u64;
                                t[0] += 1;
                                t[1] += grown;
                                t[2] += (grown > 0) as u64;
                                caps
                            } else {
                                Vec::new()
                            };
                            // again on the same records, without rebuilding
                            a.walk::<false>(*op, extra, &mut out, &mut d);
                            assert!(&out == want, "{name} pm {pm} extra {extra} block {i}: repeated audited walk differs");
                            assert!(a.rec == snap, "{name} pm {pm} extra {extra} block {i}: repeated audited walk left a marker");
                            t[4] += 1;
                            t[5] += 1;
                            if segmented {
                                assert_eq!(a.s.starts.len(), ns, "the same starts");
                                t[3] += a.s.bufs[..ns]
                                    .iter()
                                    .zip(&caps)
                                    .filter(|(b, &c)| b.capacity() > c)
                                    .count() as u64;
                            }
                            // the counted audit, on its own instance
                            counted.build(ll, counts, pm);
                            let mut c = Counts::default();
                            counted.walk::<true>(*op, extra, &mut out, &mut c);
                            assert!(&out == want, "{name} pm {pm} extra {extra} block {i}: counted walk differs");
                            t[6] += c.raw_arrivals;
                        }
                        t
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut t = [0u64; 7];
        for r in results {
            for (x, y) in t.iter_mut().zip(r) {
                *x += y;
            }
        }
        writeln!(
            jl,
            "{{\"input\":{name:?},\"permille\":{pm},\"extra_starts\":{extra},\"blocks\":{},\"segmented_blocks\":{},\"audited_cold_grown_segments\":{},\"audited_cold_blocks_with_growth\":{},\"audited_warm_grown_segments\":{},\"restoration_checks\":{},\"output_checks\":{},\"counted_raw_arrivals\":{}}}",
            bs.len(),
            t[0],
            t[1],
            t[2],
            t[3],
            t[4],
            t[5],
            t[6]
        )
        .unwrap();
        println!(
            "{name}: permille {pm}, extra starts {extra}: {} blocks ({} segmented); audited walk on fresh scratch then repeated: {} outputs and {} restorations checked; cold growth in {} segments ({} blocks), warm growth in {} segments: exact",
            bs.len(),
            t[0],
            t[5],
            t[4],
            t[1],
            t[2],
            t[3]
        );
    }
}

fn compress(data: &[u8], level: u32) -> Vec<u8> {
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

/// The adversarial inputs: tiny and partial-record sizes, the MIN_SEGMENTED boundary,
/// periodic and multi-cycle data, one byte value, every byte value, runs, random
fn adversarial() -> Vec<(String, Vec<u8>)> {
    let mut x = 0x87_c30u64;
    let mut rnd = |n: usize| -> Vec<u8> {
        (0..n)
            .map(|_| {
                x = mix(x.wrapping_add(0x9e37_79b9_7f4a_7c15));
                x as u8
            })
            .collect()
    };
    let mut v: Vec<(String, Vec<u8>)> = Vec::new();
    for n in [1usize, 2, 3, 29, 30, 31, 59, 60, 61, 1000] {
        v.push((format!("random-{n}"), rnd(n)));
    }
    for n in [65_535usize, 65_536, 65_537, 99_999, 300_000, 899_990] {
        v.push((format!("random-{n}"), rnd(n)));
    }
    v.push(("zeros-200k".into(), vec![0u8; 200_000]));
    v.push(("ab-600k".into(), b"ab".repeat(300_000)));
    v.push(("abc-1.2M".into(), b"abc".repeat(400_000)));
    v.push(("period7-700k".into(), b"abcdefg".repeat(100_000)));
    v.push((
        "period30-900k".into(),
        (0..30u8).cycle().take(900_000).collect(),
    ));
    v.push((
        "period31-900k".into(),
        (0..31u8).cycle().take(900_000).collect(),
    ));
    v.push((
        "allbytes-1M".into(),
        (0..=255u8).cycle().take(1_000_000).collect(),
    ));
    let mut runs = Vec::new();
    let mut y = 7u64;
    while runs.len() < 1_200_000 {
        y = mix(y + 1);
        let n = [1, 2, 3, 4, 5, 255, 259, 1000, 70_000][(y % 9) as usize];
        runs.extend(std::iter::repeat((y >> 8) as u8).take(n));
    }
    v.push(("runs-1.2M".into(), runs));
    let mut few = Vec::new();
    let mut z = 11u64;
    while few.len() < 1_000_000 {
        z = mix(z + 1);
        few.push(b"ab"[(z & 1) as usize]);
    }
    v.push(("few-1M".into(), few));
    v
}

// ---------------------------------------------------------------------------------------
// The traversal comparison's harness (docs/journal/2026-11-02-compact30-traversal-protocol.md)
//
//     c30walk grid OUT.jsonl CORPUS.bz2 --rounds R --cpus A,B,C,D [--dry-run]
//
// Two arms walk every block of the corpus, both in this one binary:
// - **P**, production's `bwt::walk_segments_room`, its table written by `bwt::build` just
//   before;
// - **C**, compact30's `walk::<false>`, its records copied from the block's oracle image
//   into the worker's arena just before.
// Only the walk, from table to assembled output, is inside the timer. Every walk's output
// is compared with production's (`bwt::Segmented`), outside the timer.
//
// - **W1:** one worker, pinned to the first CPU, walks the blocks in corpus order.
// - **W4:** four workers, each pinned to its own CPU, walk in lockstep steps. At each step,
//   all four prepare their tables, meet at a barrier, walk one block each, and meet again.
//   The blocks are dealt from the corpus sorted by rows, four to a step, snaking
//   (worker 0→3, then 3→0), so each step's four blocks are of nearly equal size and the
//   per-worker rows balance. The assignment is deterministic and recorded.
// - **Rounds:** round −1 is untimed (warm and check); rounds 0..R alternate the arm order
//   (even: P then C; odd: C then P), with W1 before W4 in each.
//
// Every walk is one JSON line: its round, mode, arm, worker, step, block and rows; the CPU
// it ran on; its preparation's start and end; its walk's start and end (ns from a shared
// monotonic epoch). With --dry-run (two rounds), the records are marked as a dry run, and
// bench/c30_traversal.py checks the measurement path and suppresses every timing figure.
// ---------------------------------------------------------------------------------------

/// The calling thread pinned to `cpu` (sched_setaffinity on itself): false if refused
fn pin(cpu: usize) -> bool {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
        }
        let mut mask = [0u64; 16];
        mask[cpu / 64] |= 1 << (cpu % 64);
        unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) == 0 }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cpu;
        false
    }
}

fn cpu_now() -> i64 {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_getcpu() -> i32;
        }
        unsafe { sched_getcpu() as i64 }
    }
    #[cfg(not(target_os = "linux"))]
    {
        -1
    }
}

/// The calling thread's consumed CPU time, in ns (`CLOCK_THREAD_CPUTIME_ID`): it advances
/// only while the thread runs
fn thread_cpu_ns() -> u64 {
    #[cfg(target_os = "linux")]
    {
        #[repr(C)]
        struct Timespec {
            sec: i64,
            nsec: i64,
        }
        extern "C" {
            fn clock_gettime(clock: i32, tp: *mut Timespec) -> i32;
        }
        const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
        let mut t = Timespec { sec: 0, nsec: 0 };
        if unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut t) } != 0 {
            fail(
                "clock",
                "clock_gettime(CLOCK_THREAD_CPUTIME_ID) failed".into(),
            );
        }
        t.sec as u64 * 1_000_000_000 + t.nsec as u64
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

/// Where a failure's evidence goes: the run's records file
static FAIL_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
static FAIL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Ends the run at once, from any thread. The failure is appended to the records file as
/// its last line and printed, then the process exits with status 3, so no worker is left
/// waiting at a barrier for one that has stopped. The records of every finished round are
/// already in the file.
fn fail(what: &str, detail: String) -> ! {
    let _g = FAIL_LOCK.lock();
    let line = format!(
        "{{\"kind\":\"failure\",\"what\":{},\"detail\":{}}}",
        json_str(what),
        json_str(&detail)
    );
    eprintln!("grid: FAILED: {line}");
    if let Some(p) = FAIL_PATH.get() {
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(p) {
            let _ = writeln!(f, "{line}");
            let _ = f.sync_all();
        }
    }
    std::process::exit(3)
}

/// A block's compact30 oracle image, copied into a worker's arena before C's walk
struct Image {
    rec: Vec<Rec>,
    raw: Vec<RawG>,
}

/// A worker's state, kept across rounds: each arm's table arena and walk scratch
#[derive(Default)]
struct Worker {
    tt: mibz2::bwt::TVector,
    seg: mibz2::bwt::SegScratch,
    c: C30,
    out: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq)]
enum Arm {
    P,
    C,
}

/// One walk's record. Every field is filled in after the step's completion barrier.
#[derive(Clone, Copy)]
struct Walk {
    round: i64,
    w4: bool,
    arm: Arm,
    worker: usize,
    step: usize,
    block: usize,
    rows: usize,
    /// sched_getcpu before the start barrier and after the completion barrier
    cpu: (i64, i64),
    /// monotonic ns since the epoch: the table's preparation
    prep: (u64, u64),
    /// monotonic ns since the epoch: the traversal window
    wall: (u64, u64),
    /// the thread's CPU ns: the traversal window, nested inside `wall`
    tcpu: (u64, u64),
    /// monotonic ns since the epoch: the output's check
    check: (u64, u64),
}

/// The traversal window around `f`. The clock calls, in order: the monotonic clock, the
/// thread CPU clock, `f`, the thread CPU clock, the monotonic clock. So the CPU window
/// lies inside the monotonic one, and each holds the part of its clock calls between the
/// two readings, which the calibration's empty windows measure.
#[inline(always)]
fn timed(epoch: std::time::Instant, f: impl FnOnce()) -> ((u64, u64), (u64, u64)) {
    let w0 = std::time::Instant::now();
    let c0 = thread_cpu_ns();
    f();
    let c1 = thread_cpu_ns();
    let w1 = std::time::Instant::now();
    let ns = |t: std::time::Instant| t.duration_since(epoch).as_nanos() as u64;
    ((ns(w0), ns(w1)), (c0, c1))
}

impl Worker {
    /// Write block's table for `arm` (untimed)
    fn prepare(&mut self, arm: Arm, b: &Block, img: &Image) {
        let (ll, counts, _, _) = b;
        match arm {
            Arm::P => mibz2::bwt::build(ll, counts, &mut self.tt),
            Arm::C => {
                self.c.rec.clear();
                self.c.rec.extend_from_slice(&img.rec);
                self.c.raw.clear();
                self.c.raw.extend_from_slice(&img.raw);
                self.c.n = ll.len();
            }
        }
    }

    /// Walk the prepared table: the traversal and assembly, timed
    fn walk(&mut self, arm: Arm, op: u32, epoch: std::time::Instant) -> ((u64, u64), (u64, u64)) {
        let mut d = Counts::default();
        let (tt, seg, c, out) = (&mut self.tt, &mut self.seg, &mut self.c, &mut self.out);
        timed(epoch, || match arm {
            Arm::P => mibz2::bwt::walk_segments_room(tt, op, out, seg),
            Arm::C => c.walk::<false>(op, false, out, &mut d),
        })
    }

    /// The first byte where the output differs from production's, if any (untimed)
    fn check(&self, want: &[u8]) -> Option<usize> {
        if self.out == want {
            return None;
        }
        Some(
            self.out
                .iter()
                .zip(want)
                .position(|(a, b)| a != b)
                .unwrap_or(self.out.len().min(want.len())),
        )
    }
}

/// A fault injected on purpose, to test the failure path (dry runs only)
#[derive(Clone, Copy, PartialEq)]
enum Inject {
    None,
    /// flip the first output byte of this worker's first walk in round −1, after the
    /// completion barrier and before the check
    Output {
        w4: bool,
        arm: Arm,
        worker: usize,
    },
    /// pin this worker to a CPU that does not exist, in round −1
    Pin {
        w4: bool,
        worker: usize,
    },
}

fn parse_inject(s: &str) -> Inject {
    let p: Vec<&str> = s.split(':').collect();
    let w4 = |m: &str| match m {
        "w1" => false,
        "w4" => true,
        _ => panic!("--inject: mode w1 or w4"),
    };
    match p.as_slice() {
        ["output", m, a, k] => Inject::Output {
            w4: w4(m),
            arm: if *a == "P" { Arm::P } else { Arm::C },
            worker: k.parse().unwrap(),
        },
        ["pin", m, k] => Inject::Pin {
            w4: w4(m),
            worker: k.parse().unwrap(),
        },
        _ => panic!("--inject output:MODE:ARM:WORKER or pin:MODE:WORKER"),
    }
}

const NO_SUCH_CPU: usize = 1023;

/// Pin the calling thread to `cpu`, or end the run
fn pin_or_fail(cpu: usize, what: &str) {
    if !pin(cpu) {
        fail("pin", format!("{what}: could not pin to CPU {cpu}"));
    }
}

/// The clock calls' own cost inside a traversal window: `n` empty windows on each CPU,
/// pinned, with the same calls in the same order as a walk's. Untimed by the analysis's
/// figures; it reads them only to bound the overhead (G5).
fn calibrate(cpus: &[usize], n: usize, epoch: std::time::Instant) -> String {
    let mut parts = Vec::new();
    for &cpu in cpus {
        let (mut cw, mut ww): (Vec<u64>, Vec<u64>) = std::thread::scope(|sc| {
            sc.spawn(|| {
                pin_or_fail(cpu, "calibration");
                let mut cw = Vec::with_capacity(n);
                let mut ww = Vec::with_capacity(n);
                for _ in 0..n {
                    let (w, c) = timed(epoch, || {});
                    cw.push(c.1 - c.0);
                    ww.push(w.1 - w.0);
                }
                (cw, ww)
            })
            .join()
            .unwrap()
        });
        cw.sort_unstable();
        ww.sort_unstable();
        let q = |v: &[u64], f: f64| v[((v.len() - 1) as f64 * f) as usize];
        parts.push(format!(
            "{{\"cpu\":{cpu},\"n\":{n},\"cpu_ns\":[{},{},{},{},{}],\"wall_ns\":[{},{},{},{},{}]}}",
            q(&cw, 0.0),
            q(&cw, 0.5),
            q(&cw, 0.9),
            q(&cw, 0.99),
            q(&cw, 1.0),
            q(&ww, 0.0),
            q(&ww, 0.5),
            q(&ww, 0.9),
            q(&ww, 0.99),
            q(&ww, 1.0),
        ));
    }
    parts.join(",")
}

fn write_walks(jl: &mut std::fs::File, recs: &[Walk]) {
    for r in recs {
        writeln!(
            jl,
            "{{\"kind\":\"walk\",\"round\":{},\"mode\":\"{}\",\"arm\":\"{}\",\"worker\":{},\"step\":{},\"block\":{},\"rows\":{},\"cpu_before\":{},\"cpu_after\":{},\"prep_start\":{},\"prep_end\":{},\"start\":{},\"end\":{},\"cpu_start\":{},\"cpu_end\":{},\"check_start\":{},\"check_end\":{},\"exact\":true}}",
            r.round,
            if r.w4 { "w4" } else { "w1" },
            if r.arm == Arm::P { "P" } else { "C" },
            r.worker,
            r.step,
            r.block,
            r.rows,
            r.cpu.0,
            r.cpu.1,
            r.prep.0,
            r.prep.1,
            r.wall.0,
            r.wall.1,
            r.tcpu.0,
            r.tcpu.1,
            r.check.0,
            r.check.1
        )
        .unwrap();
    }
}

/// The registered comparison (docs/journal/2026-11-02-compact30-traversal-protocol.md)
/// The W4 start barrier (protocol revision 5, report 184): four participants, reusable
/// across steps, spinning instead of sleeping.
///
/// The third i9 capture's start skew was the operating system waking workers that
/// `std::sync::Barrier` had put to sleep, not the barrier's release (report 183). Here a
/// worker that arrives early spins on the generation until the last arrives. The last
/// arrival resets the count, then publishes the next generation, which releases the others.
///
/// **Correctness:**
/// - A waiter reads the generation before it counts itself in. The generation can't
///   change before that waiter's own arrival completes the count, so its snapshot is the
///   generation it waits on.
/// - While it waits, the generation advances exactly once: the next release needs all four
///   participants, itself included. So there is no ABA, and no wrap-around in practice
///   (usize).
/// - The count is reset before the release store. A thread leaving the barrier has
///   acquired that store, so when it arrives for the next step it sees the reset count.
///
/// **Where it spins:**
/// - Only inside `wait`, before a walk, after the worker's own check and preparation.
/// - The completion barrier (still `std::sync::Barrier`) holds every worker until all four
///   walks of the step have ended. So no worker can be spinning here while a peer's timed
///   walk runs.
/// - The walk reads its clocks only after `wait` returns, as before.
///
/// **On failure:** `fail` exits the process, which ends spinning workers with it.
struct SpinBarrier {
    n: usize,
    count: AtomicUsize,
    generation: AtomicUsize,
}

impl SpinBarrier {
    fn new(n: usize) -> Self {
        SpinBarrier {
            n,
            count: AtomicUsize::new(0),
            generation: AtomicUsize::new(0),
        }
    }

    /// Returns once all `n` participants have arrived; returns the generation passed.
    fn wait(&self) -> usize {
        let g = self.generation.load(Ordering::Acquire);
        if self.count.fetch_add(1, Ordering::AcqRel) + 1 == self.n {
            self.count.store(0, Ordering::Relaxed);
            self.generation.store(g.wrapping_add(1), Ordering::Release);
        } else {
            while self.generation.load(Ordering::Acquire) == g {
                std::hint::spin_loop();
            }
        }
        g
    }
}

/// `barrier-test [--cpus A,B,C,D] [--steps N]`: the W4 synchronisation, without walks or
/// timing. Four workers, the spinning start barrier and the sleeping completion barrier, in
/// grid's order. Each step:
/// - the workers arrive unevenly: one rotating worker is delayed by a long busy wait, the
///   others by a short one;
/// - the workers' lists are uneven, so the final steps are partial;
/// - a fresh pair of barriers is used for each of two arms per round, as grid does.
///
/// It checks, and panics (exit 101) on any violation:
/// - no worker enters the start barrier while a peer is inside its "walk";
/// - no worker leaves the start barrier before all four have arrived for that step;
/// - every worker passes the same generation at the same step;
/// - no worker leaves the completion barrier while a peer is still walking.
///
/// It prints the steps and generations checked.
///
/// `--fault early-release` (a start barrier that releases at three arrivals) and
/// `--fault no-completion` (no completion barrier) are negative controls: each must fail
/// its check. A violation prints and exits with status 101 at once, so no worker is left
/// at a barrier.
fn barrier_test(args: &[String]) {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("barrier-test: FAILED: {info}");
        std::process::exit(101)
    }));
    let mut cpus: Vec<usize> = Vec::new();
    let mut steps = 2_000usize;
    let mut fault = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--cpus" => {
                cpus = args[i + 1].split(',').map(|c| c.parse().unwrap()).collect();
                i += 1;
            }
            "--steps" => {
                steps = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--fault" => {
                fault = args[i + 1].clone();
                assert!(
                    fault == "early-release" || fault == "no-completion",
                    "--fault is early-release or no-completion"
                );
                i += 1;
            }
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    // uneven lists: the last two workers have three and seven fewer items, so the final
    // steps are partial, as in grid's snake (whose lists differ by at most one)
    let lens = [steps, steps, steps - 3, steps - 7];
    let spin = |n: u64| {
        let t = std::time::Instant::now();
        while (t.elapsed().as_nanos() as u64) < n {
            std::hint::spin_loop();
        }
    };
    let mut checked = 0usize;
    for round in 0..3usize {
        for arm in 0..2usize {
            let start = SpinBarrier::new(if fault == "early-release" { 3 } else { 4 });
            let done = std::sync::Barrier::new(4);
            let skip_done = fault == "no-completion";
            let arrived = AtomicUsize::new(0);
            let walking = AtomicUsize::new(0);
            let gens: Vec<Vec<usize>> = std::thread::scope(|sc| {
                let hs: Vec<_> = (0..4)
                    .map(|k| {
                        let (start, done, arrived, walking, cpus) =
                            (&start, &done, &arrived, &walking, &cpus);
                        sc.spawn(move || {
                            if let Some(&c) = cpus.get(k) {
                                assert!(pin(c), "barrier-test: could not pin to CPU {c}");
                            }
                            let mut seen = Vec::with_capacity(steps);
                            for step in 0..steps {
                                // the uneven arrival: a rotating latecomer, a short wait otherwise
                                let late = (step + round + arm) % 4 == k;
                                spin(if late { 20_000 + 997 * (step as u64 % 13) } else { 300 * k as u64 });
                                assert_eq!(
                                    walking.load(Ordering::SeqCst),
                                    0,
                                    "worker {k} step {step}: entered the start barrier during a peer's walk"
                                );
                                arrived.fetch_add(1, Ordering::SeqCst);
                                let g = start.wait();
                                let a = arrived.load(Ordering::SeqCst);
                                assert!(
                                    a >= 4 * (step + 1),
                                    "worker {k} step {step}: left the start barrier after {a} arrivals"
                                );
                                seen.push(g);
                                if step < lens[k] {
                                    walking.fetch_add(1, Ordering::SeqCst);
                                    spin(2_000 + 50 * k as u64);
                                    walking.fetch_sub(1, Ordering::SeqCst);
                                }
                                if !skip_done {
                                    done.wait();
                                }
                                assert_eq!(
                                    walking.load(Ordering::SeqCst),
                                    0,
                                    "worker {k} step {step}: left the completion barrier during a peer's walk"
                                );
                            }
                            seen
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for step in 0..steps {
                let g = gens[0][step];
                assert_eq!(
                    g, step,
                    "round {round} arm {arm} step {step}: generation {g}"
                );
                for (k, v) in gens.iter().enumerate() {
                    assert_eq!(
                        v[step], g,
                        "round {round} arm {arm} step {step}: worker {k} passed generation {}",
                        v[step]
                    );
                }
            }
            assert_eq!(arrived.load(Ordering::SeqCst), 4 * steps);
            checked += steps;
        }
    }
    println!(
        "barrier-test: {checked} steps in 3 rounds x 2 arms, 4 workers, uneven arrivals and lists (partial final steps), cpus {:?}: every check passed",
        cpus
    );
}

fn grid(args: &[String]) {
    let out_path = &args[0];
    let file = &args[1];
    let mut rounds = 20i64;
    let mut cpus: Vec<usize> = Vec::new();
    let mut dry = false;
    let mut inject = Inject::None;
    let mut inject_arg = String::new();
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--rounds" => {
                rounds = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--cpus" => {
                cpus = args[i + 1].split(',').map(|c| c.parse().unwrap()).collect();
                i += 1;
            }
            "--dry-run" => dry = true,
            "--inject" => {
                inject = parse_inject(&args[i + 1]);
                inject_arg = args[i + 1].clone();
                i += 1;
            }
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    assert!(
        inject == Inject::None || dry,
        "--inject is for dry runs only"
    );
    if dry {
        rounds = 2;
    }
    assert!(
        cpus.len() == 4,
        "--cpus needs four CPUs, each a physical core"
    );
    let data = std::fs::read(file).unwrap();
    let bs = blocks(&data);
    // the oracle images (untimed, once)
    let imgs: Vec<Image> = bs
        .iter()
        .map(|(ll, counts, _, _)| {
            let mut w = C30::default();
            w.build(ll, counts, 0);
            Image {
                rec: w.rec,
                raw: w.raw,
            }
        })
        .collect();
    // the four-worker assignment: sorted by rows (descending, then index), dealt four to a
    // step, snaking
    let mut order: Vec<usize> = (0..bs.len()).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(bs[i].0.len()), i));
    let mut lists: [Vec<usize>; 4] = Default::default();
    for (k, chunk) in order.chunks(4).enumerate() {
        for (j, &b) in chunk.iter().enumerate() {
            let w = if k % 2 == 0 { j } else { 3 - j };
            lists[w].push(b);
        }
    }
    let wrows: Vec<usize> = lists
        .iter()
        .map(|l| l.iter().map(|&b| bs[b].0.len()).sum())
        .collect();
    let mut jl = std::fs::File::create(out_path).unwrap();
    FAIL_PATH.set(out_path.clone()).unwrap();
    std::panic::set_hook(Box::new(|info| fail("panic", info.to_string())));
    let name = std::path::Path::new(file)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let total: usize = bs.iter().map(|b| b.0.len()).sum();
    let brows: Vec<usize> = bs.iter().map(|b| b.0.len()).collect();
    let braw: Vec<usize> = imgs.iter().map(|m| m.raw.len()).collect();
    writeln!(
        jl,
        "{{\"kind\":\"meta\",\"corpus\":{name:?},\"blocks\":{},\"rows\":{total},\"full_blocks\":{},\"rounds\":{rounds},\"dry_run\":{dry},\"inject\":{},\"cpus\":{cpus:?},\"w4_lists\":{lists:?},\"w4_rows\":{wrows:?},\"block_rows\":{brows:?},\"block_raw_groups\":{braw:?}}}",
        bs.len(),
        bs.iter().filter(|b| b.0.len() >= 899_000).count(),
        if inject == Inject::None { "null".to_string() } else { json_str(&inject_arg) },
    )
    .unwrap();
    let epoch = std::time::Instant::now();
    let calib = calibrate(&cpus, 20_000, epoch);
    writeln!(
        jl,
        "{{\"kind\":\"calibration\",\"when\":\"before\",\"cpus\":[{calib}]}}"
    )
    .unwrap();
    let mut ws: Vec<Worker> = (0..4).map(|_| Worker::default()).collect();
    let steps = lists.iter().map(|l| l.len()).max().unwrap();
    let blank = Walk {
        round: 0,
        w4: false,
        arm: Arm::P,
        worker: 0,
        step: 0,
        block: 0,
        rows: 0,
        cpu: (0, 0),
        prep: (0, 0),
        wall: (0, 0),
        tcpu: (0, 0),
        check: (0, 0),
    };
    let ns = |t: std::time::Instant| t.duration_since(epoch).as_nanos() as u64;
    let mut recs: Vec<Walk> = Vec::with_capacity(4 * bs.len());
    for round in -1..rounds {
        let arms = if round.rem_euclid(2) == 0 {
            [Arm::P, Arm::C]
        } else {
            [Arm::C, Arm::P]
        };
        recs.clear();
        // W1: one worker, every block in order: prepare, walk, check
        for &arm in &arms {
            let w = &mut ws[0];
            let mut v: Vec<Walk> = Vec::with_capacity(bs.len());
            let v = &mut v;
            std::thread::scope(|sc| {
                sc.spawn(|| {
                    let cpu = match inject {
                        Inject::Pin { w4: false, .. } if round == -1 => NO_SUCH_CPU,
                        _ => cpus[0],
                    };
                    pin_or_fail(cpu, "W1 worker 0");
                    for (i, b) in bs.iter().enumerate() {
                        let c0 = cpu_now();
                        let p0 = std::time::Instant::now();
                        w.prepare(arm, b, &imgs[i]);
                        let p1 = std::time::Instant::now();
                        let (wall, tcpu) = w.walk(arm, b.2, epoch);
                        if inject == (Inject::Output { w4: false, arm, worker: 0 })
                            && round == -1
                            && i == 0
                        {
                            w.out[0] ^= 1;
                        }
                        let k0 = std::time::Instant::now();
                        if let Some(at) = w.check(&b.3) {
                            fail("output", format!("round {round} w1 arm {} worker 0 step {i} block {i}: the output differs from production's at byte {at}", if arm == Arm::P { "P" } else { "C" }));
                        }
                        let k1 = std::time::Instant::now();
                        v.push(Walk {
                            round,
                            w4: false,
                            arm,
                            worker: 0,
                            step: i,
                            block: i,
                            rows: b.0.len(),
                            cpu: (c0, cpu_now()),
                            prep: (ns(p0), ns(p1)),
                            wall,
                            tcpu,
                            check: (ns(k0), ns(k1)),
                        });
                    }
                })
                .join()
                .unwrap()
            });
            recs.extend_from_slice(v);
        }
        // W4: four workers in lockstep by block. Each step: prepare (untimed), the start
        // barrier, the walk (timed), the completion barrier, then the check and the record.
        // So no check, record or next preparation runs while any walk of the step does.
        for &arm in &arms {
            // the start barrier spins (protocol revision 5); the completion barrier sleeps
            let start = SpinBarrier::new(4);
            let done = std::sync::Barrier::new(4);
            let r: Vec<Vec<Walk>> = std::thread::scope(|sc| {
                let hs: Vec<_> = ws
                    .iter_mut()
                    .enumerate()
                    .map(|(k, w)| {
                        let (list, start, done, cpus, bs, imgs) =
                            (&lists[k], &start, &done, &cpus, &bs, &imgs);
                        sc.spawn(move || {
                            let mut v: Vec<Walk> = vec![blank; list.len()];
                            let cpu = match inject {
                                Inject::Pin { w4: true, worker } if worker == k && round == -1 => {
                                    NO_SUCH_CPU
                                }
                                _ => cpus[k],
                            };
                            pin_or_fail(cpu, &format!("W4 worker {k}"));
                            for step in 0..steps {
                                let item = list.get(step).map(|&i| {
                                    let c0 = cpu_now();
                                    let p0 = std::time::Instant::now();
                                    w.prepare(arm, &bs[i], &imgs[i]);
                                    let p1 = std::time::Instant::now();
                                    (i, c0, (ns(p0), ns(p1)))
                                });
                                start.wait();
                                let walked = item.map(|(i, ..)| w.walk(arm, bs[i].2, epoch));
                                done.wait();
                                if let (Some((i, c0, prep)), Some((wall, tcpu))) = (item, walked) {
                                    if inject == (Inject::Output { w4: true, arm, worker: k })
                                        && round == -1
                                        && step == 0
                                    {
                                        w.out[0] ^= 1;
                                    }
                                    let k0 = std::time::Instant::now();
                                    if let Some(at) = w.check(&bs[i].3) {
                                        fail("output", format!("round {round} w4 arm {} worker {k} step {step} block {i}: the output differs from production's at byte {at}", if arm == Arm::P { "P" } else { "C" }));
                                    }
                                    let k1 = std::time::Instant::now();
                                    *v.get_mut(step).unwrap() = Walk {
                                        round,
                                        w4: true,
                                        arm,
                                        worker: k,
                                        step,
                                        block: i,
                                        rows: bs[i].0.len(),
                                        cpu: (c0, cpu_now()),
                                        prep,
                                        wall,
                                        tcpu,
                                        check: (ns(k0), ns(k1)),
                                    };
                                }
                            }
                            v
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for v in &r {
                recs.extend_from_slice(v);
            }
        }
        write_walks(&mut jl, &recs);
        println!(
            "grid: round {round} done{}",
            if dry { " (dry run)" } else { "" }
        );
    }
    let calib = calibrate(&cpus, 20_000, epoch);
    writeln!(
        jl,
        "{{\"kind\":\"calibration\",\"when\":\"after\",\"cpus\":[{calib}]}}"
    )
    .unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "grid" {
        return grid(&args[2..]);
    }
    if args[1] == "barrier-test" {
        return barrier_test(&args[2..]);
    }
    if let Some(m) = args[1].strip_prefix("ctor-") {
        return ctor::main(m, &args[2..]);
    }
    if let Some(m) = args[1].strip_prefix("ctor2-") {
        return ctor2::main(m, &args[2..]);
    }
    if let Some(m) = args[1].strip_prefix("ctor2d-") {
        return ctor2d::main(m, &args[2..]);
    }
    if let Some(m) = args[1].strip_prefix("ctor3-") {
        return ctor3::main(m, &args[2..]);
    }
    if args[1] == "cbuild" {
        return cbuild::main(&args[2..]);
    }
    if args[1] == "cbuild-rss" {
        return cbuild::rss(&args[2..]);
    }
    if args[1] == "cbuild-v2" {
        return cbuild::main_v2(&args[2..]);
    }
    if args[1] == "cbuild-rss-v2" {
        return cbuild::rss_v2(&args[2..]);
    }
    let mut jl = std::fs::File::create(&args[2]).unwrap();
    match args[1].as_str() {
        "check" => {
            for file in &args[3..] {
                let data = std::fs::read(file).unwrap();
                let name = std::path::Path::new(file)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                check(&name, &data, &mut jl);
            }
        }
        "adversarial" => {
            for (name, data) in adversarial() {
                for level in [1, 9] {
                    let z = compress(&data, level);
                    check(&format!("{name} -{level}"), &z, &mut jl);
                }
            }
        }
        "audit" => {
            for file in &args[3..] {
                let data = std::fs::read(file).unwrap();
                let name = std::path::Path::new(file)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                audit(&name, &data, &mut jl);
            }
        }
        "audit-adversarial" => {
            for (name, data) in adversarial() {
                for level in [1, 9] {
                    let z = compress(&data, level);
                    audit(&format!("{name} -{level}"), &z, &mut jl);
                }
            }
        }
        m => panic!("unknown mode {m}"),
    }
}
