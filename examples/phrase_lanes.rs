//! The phrase walk in lanes, a research prototype (docs/journal/2026-11-09-phrase-structure.md, "Correction and close"):
//! `examples/phrase_walk.rs`'s copy walk with its events overlapped. The text is cut into
//! segments at marker rows, as the room walk's is, and several lanes each walk a segment, one
//! event a turn, so one lane's cache misses are in flight while the others work. Nothing in the
//! decoder calls it.
//!
//! ```text
//! phrase_lanes check FILE.bz2 [LANES]            every block against bwt::walk, byte for byte
//! phrase_lanes time FILE.bz2 ROUNDS COPIES [LANES]
//! ```
//!
//! **Markers.** Each segment start is a one-row cell, flagged. Two rows of one cell stay
//! together, so no copy can pass through a one-row cell: a lane meets a marker only by landing
//! on it, and its segment ends there.
//!
//! **Sources in other segments.** A cell's remembered visits may lie in another lane's
//! segment. A copy reads that segment only up to its lane's frontier; there the source's state
//! is the lane's own current state, kept beside the frontier.
//!
//! **Research only.** Its `unsafe` walk has had no safety review, Miri, fuzzing or mutation
//! testing (RULES B5): it is checked for equal bytes on the corpora it is run on, no more.

use std::time::Instant;

use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const CAP: u32 = 127;
/// segments a block: positions are segment << 20 | offset, in 25 bits
const SEGS: usize = 32;
const REGION: usize = 1 << 20;
const NONE: u32 = u32::MAX;
const FLAG: u64 = 1 << 42;

/// A cell's record: idx (20 bits), d (7), len (7), byte (8), marker flag (bit 42)
#[inline(always)]
fn rec(len: u32, c: u8, flag: bool) -> u64 {
    (len as u64) << 27 | (c as u64) << 34 | if flag { FLAG } else { 0 }
}

#[inline(always)]
fn len_of(r: u64) -> u32 {
    (r >> 27) as u32 & 127
}

/// A state: cell << 14 | rows above << 7 | rows below
#[inline(always)]
fn pack(k: u32, o: u32, len: u32) -> u32 {
    k << 14 | o << 7 | (len - 1 - o)
}

#[derive(Clone, Copy)]
struct Lane {
    k: u32,
    o: u32,
    /// the next position to write: segment << 20 | offset
    t: u32,
    /// a chosen source and its row distance, when `copying`
    q: u32,
    d: i32,
    copying: bool,
}

#[derive(Default)]
pub struct Laned {
    cells: Vec<u64>,
    /// each cell's first and latest event starts: position (25 bits) << 7 | offset; a
    /// marker cell's first word is its segment
    memo: Vec<[u32; 2]>,
    runs: Vec<u16>,
    order: Vec<u32>,
    start: Vec<u32>,
    markers: Vec<u32>,
    /// each segment's marker cell
    seg_cell: Vec<u32>,
    st: Vec<u32>,
    bytes: Vec<u8>,
    /// each segment's frontier (offset), the state there, and the segment after it
    frontier: [u32; SEGS],
    state: [u32; SEGS],
    next: [u32; SEGS],
    pub real: usize,
    pub copies: usize,
    pub waits: usize,
    /// no copies: every step a real step through the table, with no memo and no state stream
    /// (a move-structure walk in lanes)
    pub plain: bool,
}

/// `bwt::random_rows` (pub(crate)), copied: the segment starts' generator
fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    for _ in 0..m {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        rows.push((x % n as u64) as u32);
    }
}

impl Laned {
    /// The table, with the markers' rows as one-row cells. Returns the segment that starts at
    /// `orig_ptr`, or None when the cells pass 2^18.
    pub fn build(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32) -> Option<usize> {
        let n = ll.len();
        self.markers.clear();
        random_rows(n, (SEGS - 1).min(n.saturating_sub(1)), &mut self.markers);
        self.markers.push(orig_ptr);
        self.markers.sort_unstable();
        self.markers.dedup();
        // each byte's first F row, and its markers as ranks within its bucket
        let mut first = [0u32; 257];
        for c in 0..256 {
            first[c + 1] = first[c] + counts[c];
        }
        let (mut ptr, mut end) = ([0usize; 256], [0usize; 256]);
        {
            let mut i = 0;
            for c in 0..256 {
                ptr[c] = i;
                while i < self.markers.len() && self.markers[i] < first[c + 1] {
                    i += 1;
                }
                end[c] = i;
            }
        }
        // pass 1: the runs, cut at CAP and at the markers, and each byte's count of them
        let mut per = [0u32; 256];
        let mut seen = [0u32; 256];
        self.runs.clear();
        let mut i = 0;
        while i < n {
            let c = ll[i];
            let cu = c as usize;
            let mut j = i + 1;
            let w = u64::from_ne_bytes([c; 8]);
            while j + 8 <= n && u64::from_ne_bytes(ll[j..j + 8].try_into().unwrap()) == w {
                j += 8;
            }
            while j < n && ll[j] == c {
                j += 1;
            }
            let mut left = (j - i) as u32;
            while left > 0 {
                let to_marker = if ptr[cu] < end[cu] {
                    self.markers[ptr[cu]] - first[cu] - seen[cu]
                } else {
                    u32::MAX
                };
                let (len, flag) = if to_marker == 0 {
                    ptr[cu] += 1;
                    (1, 1 << 7)
                } else {
                    (left.min(CAP).min(to_marker), 0)
                };
                self.runs.push((c as u16) << 8 | flag | len as u16);
                per[cu] += 1;
                seen[cu] += len;
                left -= len;
            }
            i = j;
        }
        let r = self.runs.len();
        if r >= 1 << 18 {
            return None;
        }
        let (mut nextj, mut nexta) = ([0u32; 256], [0u32; 256]);
        let mut j = 0;
        for c in 0..256 {
            nextj[c] = j;
            nexta[c] = first[c];
            j += per[c];
        }
        // pass 2: each run's F index, and the cells' F starts
        self.order.clear();
        self.order.reserve(r);
        self.start.clear();
        self.start.resize(r + 1, 0);
        self.cells.clear();
        self.cells.resize(r + 2, 0);
        for &run in &self.runs {
            let (c, len) = ((run >> 8) as usize, run as u32 & 127);
            let j = nextj[c];
            nextj[c] += 1;
            self.start[j as usize] = nexta[c];
            nexta[c] += len;
            self.cells[j as usize] = rec(len, c as u8, run & 1 << 7 != 0);
            self.order.push(j);
        }
        self.start[r] = n as u32;
        // pass 3: the image of run i is L rows p.., which lie in the cell jj with start ≤ p
        let (mut p, mut jj) = (0u32, 0usize);
        for &j in &self.order {
            while self.start[jj + 1] <= p {
                jj += 1;
            }
            let cell = &mut self.cells[j as usize];
            let len = len_of(*cell);
            *cell |= jj as u64 | ((p - self.start[jj]) as u64) << 20;
            p += len;
        }
        self.cells[r] = rec(127, 0, false);
        self.cells[r + 1] = rec(127, 0, false);
        self.memo.clear();
        self.memo.resize(r, [NONE; 2]);
        // each marker's cell, which knows its segment
        self.seg_cell.clear();
        let mut origin = 0;
        for (s, &m) in self.markers.iter().enumerate() {
            let k = self.start[..r].partition_point(|&a| a <= m) - 1;
            debug_assert!(self.start[k] == m && self.cells[k] & FLAG != 0);
            self.memo[k][0] = s as u32;
            self.seg_cell.push(k as u32);
            if m == orig_ptr {
                origin = s;
            }
        }
        Some(origin)
    }

    /// The walk: n bytes into `out`, the segments joined from `origin`'s
    pub fn walk(&mut self, n: usize, origin: usize, lanes: usize, out: &mut Vec<u8>) {
        #[cfg(target_arch = "x86_64")]
        assert!(
            is_x86_feature_detected!("avx2"),
            "the prototype's copy loop is AVX2"
        );
        // the streams: a region a segment, written before it is read, kept between blocks
        if self.st.capacity() < SEGS * REGION {
            self.st = Vec::with_capacity(SEGS * REGION);
            self.bytes = Vec::with_capacity(SEGS * REGION);
        }
        unsafe { self.walk_impl(lanes) };
        out.clear();
        out.reserve(n);
        let mut s = origin;
        while out.len() < n {
            let len = (self.frontier[s] as usize).min(n - out.len());
            // (a region's first `frontier` bytes were written by the walk)
            let from =
                unsafe { std::slice::from_raw_parts(self.bytes.as_ptr().add(s * REGION), len) };
            out.extend_from_slice(from);
            s = self.next[s] as usize;
        }
    }

    #[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx2"))]
    unsafe fn walk_impl(&mut self, lanes: usize) {
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        let cells = self.cells.as_ptr();
        let memo = self.memo.as_mut_ptr();
        let st = self.st.as_mut_ptr();
        let by = self.bytes.as_mut_ptr();
        let segs = self.seg_cell.len();
        let plain = self.plain;
        let (mut real, mut copies, mut waits) = (0usize, 0usize, 0usize);
        #[cfg(target_arch = "x86_64")]
        let v127 = _mm256_set1_epi32(127);
        macro_rules! fetch {
            ($p:expr) => {
                #[cfg(target_arch = "x86_64")]
                _mm_prefetch($p as *const i8, _MM_HINT_T0);
            };
        }
        // a segment's first step, from its marker cell: a real step
        macro_rules! start {
            ($s:expr) => {{
                let s = $s as u32;
                let k = *self.seg_cell.get_unchecked(s as usize);
                let r = *cells.add(k as usize);
                let t = s << 20;
                *st.add(t as usize) = pack(k, 0, 1);
                *by.add(t as usize) = (r >> 34) as u8;
                let (k, o) = (r as u32 & 0xfffff, (r >> 20) as u32 & 127);
                self.frontier[s as usize] = 1;
                self.state[s as usize] = pack(k, o, len_of(*cells.add(k as usize)));
                fetch!(cells.add(k as usize));
                fetch!(memo.add(k as usize));
                Lane {
                    k,
                    o,
                    t: t + 1,
                    q: 0,
                    d: 0,
                    copying: false,
                }
            }};
        }
        let mut lane: Vec<Lane> = Vec::with_capacity(lanes);
        let mut started = 0;
        while started < segs && lane.len() < lanes {
            lane.push(start!(started));
            started += 1;
        }
        while !lane.is_empty() {
            let mut li = 0;
            while li < lane.len() {
                let ln = lane.as_mut_ptr().add(li);
                let (mut k, mut o, t) = ((*ln).k, (*ln).o, (*ln).t as usize);
                let seg = t >> 20;
                if !(*ln).copying {
                    let r = *cells.add(k as usize);
                    if r & FLAG != 0 {
                        // landed on a marker: this segment ends, the marker's follows it
                        self.next[seg] = (*memo.add(k as usize))[0];
                        self.state[seg] = pack(k, 0, 1);
                        if started < segs {
                            *ln = start!(started);
                            started += 1;
                            li += 1;
                        } else {
                            lane.swap_remove(li);
                        }
                        continue;
                    }
                    let m = if plain {
                        [NONE; 2]
                    } else {
                        *memo.add(k as usize)
                    };
                    let me = (t as u32) << 7 | o;
                    if m[0] == NONE {
                        // the cell's first visit: a real step
                        if !plain {
                            *memo.add(k as usize) = [me, me];
                            *st.add(t) = pack(k, o, len_of(r));
                        }
                        *by.add(t) = (r >> 34) as u8;
                        real += 1;
                        k = r as u32 & 0xfffff;
                        o += (r >> 20) as u32 & 127;
                        let mut l;
                        loop {
                            l = len_of(*cells.add(k as usize));
                            if o < l {
                                break;
                            }
                            o -= l;
                            k += 1;
                        }
                        *self.frontier.get_unchecked_mut(seg) = (t + 1) as u32 & 0xfffff;
                        *self.state.get_unchecked_mut(seg) = pack(k, o, l);
                        (*ln).k = k;
                        (*ln).o = o;
                        (*ln).t = (t + 1) as u32;
                        fetch!(cells.add(k as usize));
                        fetch!(memo.add(k as usize));
                    } else {
                        // the nearer in rows of the cell's first and latest visits
                        let (d1, d2) = (
                            o as i32 - (m[0] & 127) as i32,
                            o as i32 - (m[1] & 127) as i32,
                        );
                        let (q, d) = if d1.abs() <= d2.abs() {
                            (m[0] >> 7, d1)
                        } else {
                            (m[1] >> 7, d2)
                        };
                        (*memo.add(k as usize))[1] = me;
                        (*ln).q = q;
                        (*ln).d = d;
                        (*ln).copying = true;
                        fetch!(st.add(q as usize));
                        fetch!(by.add(q as usize));
                    }
                    li += 1;
                    continue;
                }
                // a copy: while the source's trajectory has d rows of room on this side, and
                // (in another lane's segment) up to that lane's frontier
                let (q, d) = ((*ln).q as usize, (*ln).d);
                let sseg = q >> 20;
                let lim = if sseg == seg {
                    usize::MAX
                } else {
                    *self.frontier.get_unchecked(sseg) as usize - (q & 0xfffff)
                };
                let kd = (d << 7).wrapping_sub(d) as u32;
                let (shift, need) = if d > 0 {
                    (0, d as u32)
                } else {
                    (7, (-d) as u32)
                };
                let mut i = 0usize;
                #[cfg(target_arch = "x86_64")]
                if sseg != seg || t - q >= 8 {
                    let vneed = _mm256_set1_epi32(need as i32);
                    let vk = _mm256_set1_epi32(kd as i32);
                    let sh = _mm_cvtsi32_si128(shift);
                    while i + 8 <= lim {
                        let v = _mm256_loadu_si256(st.add(q + i) as *const __m256i);
                        let room = _mm256_and_si256(_mm256_srl_epi32(v, sh), v127);
                        let bad = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(
                            vneed, room,
                        )));
                        if bad != 0 {
                            break;
                        }
                        _mm256_storeu_si256(st.add(t + i) as *mut __m256i, _mm256_add_epi32(v, vk));
                        (by.add(t + i) as *mut u64)
                            .write_unaligned((by.add(q + i) as *const u64).read_unaligned());
                        i += 8;
                    }
                }
                let s;
                loop {
                    if i == lim {
                        // the source lane's own state, at its frontier
                        s = *self.state.get_unchecked(sseg);
                        break;
                    }
                    let v = *st.add(q + i);
                    if (v >> shift & 127) < need {
                        s = v;
                        break;
                    }
                    *st.add(t + i) = v.wrapping_add(kd);
                    *by.add(t + i) = *by.add(q + i);
                    i += 1;
                }
                // the source is at (cell, above, below): the walk is d rows from it
                k = s >> 14;
                let mut l;
                if (s >> shift & 127) >= need {
                    // (only at a frontier) still beside the source, in its cell
                    o = (s >> 7 & 127).wrapping_add(d as u32);
                    l = len_of(*cells.add(k as usize));
                    if i == 0 {
                        // nothing to copy yet: the source lane has not moved. Wait a turn
                        waits += 1;
                        li += 1;
                        continue;
                    }
                } else if d > 0 {
                    o = d as u32 - (s & 127) - 1;
                    k += 1;
                    loop {
                        l = len_of(*cells.add(k as usize));
                        if o < l {
                            break;
                        }
                        o -= l;
                        k += 1;
                    }
                } else {
                    let mut u = (-d) as u32 - (s >> 7 & 127) - 1;
                    loop {
                        k -= 1;
                        l = len_of(*cells.add(k as usize));
                        if u < l {
                            o = l - 1 - u;
                            break;
                        }
                        u -= l;
                    }
                }
                copies += 1;
                *self.frontier.get_unchecked_mut(seg) = (t + i) as u32 & 0xfffff;
                *self.state.get_unchecked_mut(seg) = pack(k, o, l);
                (*ln).k = k;
                (*ln).o = o;
                (*ln).t = (t + i) as u32;
                (*ln).copying = false;
                fetch!(cells.add(k as usize));
                fetch!(memo.add(k as usize));
                li += 1;
            }
        }
        self.real = real;
        self.copies = copies;
        self.waits = waits;
    }
}

struct Block {
    ll: Vec<u8>,
    counts: [u32; 256],
    orig_ptr: u32,
}

fn blocks(data: &[u8]) -> Vec<Block> {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let (mut syms, mut v) = (Vec::new(), Vec::new());
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        v.push(Block {
            ll,
            counts,
            orig_ptr: b.orig_ptr,
        });
        syms = b.syms;
        // (PHRASE_BLOCKS=N: the first N blocks only, for instruction counting)
        if std::env::var("PHRASE_BLOCKS").is_ok_and(|n| v.len() >= n.parse().unwrap()) {
            break;
        }
    }
    v
}

/// One pass over the blocks in decode order with one arm: (ns in build, ns in walk, bytes)
fn pass(bs: &[Block], arm: u8, lanes: usize) -> (u128, u128, usize) {
    let (mut tb, mut tw, mut bytes) = (0, 0, 0);
    let (mut tt, mut seg, mut out) = (TVector::new(), SegScratch::default(), Vec::new());
    let mut ph = Laned::default();
    for b in bs {
        let t0 = Instant::now();
        if arm == 0 {
            build(&b.ll, &b.counts, &mut tt);
            let t1 = Instant::now();
            out.clear();
            walk_segments_room(&mut tt, b.orig_ptr, &mut out, &mut seg);
            tb += (t1 - t0).as_nanos();
            tw += t1.elapsed().as_nanos();
        } else {
            ph.plain = arm == 2;
            let origin = ph
                .build(&b.ll, &b.counts, b.orig_ptr)
                .expect("a block of 2^18 cells or more");
            let t1 = Instant::now();
            ph.walk(b.ll.len(), origin, lanes, &mut out);
            tb += (t1 - t0).as_nanos();
            tw += t1.elapsed().as_nanos();
        }
        std::hint::black_box(&out);
        bytes += b.ll.len();
    }
    (tb, tw, bytes)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[2]).expect("read");
    let bs = blocks(&data);
    match args[1].as_str() {
        "check" => {
            let lanes: usize = args.get(3).map_or(8, |s| s.parse().unwrap());
            let (mut tt, mut want, mut got) = (TVector::new(), Vec::new(), Vec::new());
            let mut ph = Laned {
                plain: std::env::var("PHRASE_PLAIN").is_ok(),
                ..Default::default()
            };
            let (mut real, mut copies, mut waits, mut rows) = (0, 0, 0, 0);
            for (i, b) in bs.iter().enumerate() {
                if b.ll.is_empty() {
                    continue;
                }
                build(&b.ll, &b.counts, &mut tt);
                want.clear();
                walk(&tt, b.orig_ptr, &mut want);
                let origin = ph.build(&b.ll, &b.counts, b.orig_ptr).expect("cells");
                ph.walk(b.ll.len(), origin, lanes, &mut got);
                assert!(want == got, "block {i} differs");
                real += ph.real;
                copies += ph.copies;
                waits += ph.waits;
                rows += b.ll.len();
            }
            println!(
                "{} blocks, {rows} rows, {lanes} lanes: identical. real steps {:.4}/row, copies {:.4}/row, waits {:.5}/row",
                bs.len(),
                real as f64 / rows as f64,
                copies as f64 / rows as f64,
                waits as f64 / rows as f64
            );
        }
        "time" => {
            let rounds: usize = args[3].parse().unwrap();
            let copies: usize = args[4].parse().unwrap();
            let lanes: usize = args.get(5).map_or(8, |s| s.parse().unwrap());
            for round in 0..rounds {
                for a in 0..3u8 {
                    let arm = (a + round as u8) % 3;
                    let t0 = Instant::now();
                    let res: Vec<(u128, u128, usize)> = std::thread::scope(|s| {
                        let hs: Vec<_> = (0..copies)
                            .map(|_| s.spawn(|| pass(&bs, arm, lanes)))
                            .collect();
                        hs.into_iter().map(|h| h.join().unwrap()).collect()
                    });
                    let wall = t0.elapsed().as_nanos() as f64;
                    let bytes: usize = res.iter().map(|r| r.2).sum();
                    let tb: u128 = res.iter().map(|r| r.0).sum();
                    let tw: u128 = res.iter().map(|r| r.1).sum();
                    println!(
                        "round {round} {} copies {copies}: build {:.3} walk {:.3} total {:.3} ns/B a thread; {:.0} MB/s in all",
                        ["production", "lanes     ", "cells only"][arm as usize],
                        tb as f64 / bytes as f64,
                        tw as f64 / bytes as f64,
                        (tb + tw) as f64 / bytes as f64,
                        bytes as f64 / wall * 1e3
                    );
                }
            }
        }
        _ => panic!("check or time"),
    }
}
