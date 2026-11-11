//! The phrase walk, a research prototype (docs/journal/2026-11-10-phrase-census.md): an inverse
//! BWT that works on Ψ's translation cells and copies from text it has already decoded.
//! Nothing in the decoder calls it.
//!
//! ```text
//! phrase_walk check FILE.bz2              every block against bwt::walk, byte for byte
//! phrase_walk time FILE.bz2 ROUNDS COPIES in decode order, arms alternating: ns a byte
//! phrase_walk knee FILE.bz2 ROUNDS 1,2,4  the same at each count of concurrent copies, as
//!                                         one JSON line a (round, arm, copies) on stdout
//! ```
//!
//! **Research only.** Its `unsafe` walk has had no safety review, Miri, fuzzing or mutation
//! testing (RULES B5): it is checked for equal bytes on the corpora it is run on, no more.
//!
//! **The table.** One record a cell (a run of L, cut every 127 rows), in F order: the cell
//! holding the image of its first row, that image's offset there, its length and its byte.
//!
//! **The walk.** State (cell k, offset o) at text time t. If the cell was visited before, at
//! time q and offset o', the walk is now d = o − o' rows from that visit's trajectory, and stays
//! so while that trajectory keeps d rows of room in its cells. So it copies bytes and states
//! from q on (state + d), 8 a step, until the room runs out; there the source's state gives the
//! walk's own, exactly. With no earlier visit it takes one real step through the table.
//!
//! **The streams.** One byte and one u32 of state (cell, rows above, rows below) per text
//! position, written in order, read back only at copy sources.

use std::time::Instant;

use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const CAP: u32 = 127;

/// A cell's record: idx (20 bits), d (7), len (7), byte (8)
#[inline(always)]
fn rec(idx: u32, d: u32, len: u32, c: u8) -> u64 {
    idx as u64 | (d as u64) << 20 | (len as u64) << 27 | (c as u64) << 34
}

#[inline(always)]
fn len_of(r: u64) -> u32 {
    (r >> 27) as u32 & 127
}

#[derive(Default)]
pub struct Phrase {
    /// the cells, in F order, and two sentinels
    cells: Vec<u64>,
    /// each cell's two remembered visits: time (20 bits) and offset (7), twice
    memo: Vec<u64>,
    /// the runs in L order, cut at CAP: byte << 8 | length
    runs: Vec<u16>,
    /// the runs in L order: F index of each
    order: Vec<u32>,
    start: Vec<u32>,
    /// each text position's state: cell << 14 | rows above << 7 | rows below
    st: Vec<u32>,
    /// counters of the last walk: real steps, copies
    pub real: usize,
    pub copies: usize,
}

impl Phrase {
    /// The table, from the last column: a run scan, a counting sort into F order, and a merge
    /// of the runs' L starts against the cells' F starts. Returns the cell holding `orig_ptr`
    /// and its offset there, or None when the cells pass 2^18 (a block for the T-vector).
    pub fn build(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32) -> Option<(u32, u32)> {
        let n = ll.len();
        // pass 1: the runs, cut at CAP, and each byte's count of them
        let mut per = [0u32; 256];
        self.runs.clear();
        let mut i = 0;
        while i < n {
            let c = ll[i];
            let mut j = i + 1;
            // eight equal bytes at a time
            let w = u64::from_ne_bytes([c; 8]);
            while j + 8 <= n && u64::from_ne_bytes(ll[j..j + 8].try_into().unwrap()) == w {
                j += 8;
            }
            while j < n && ll[j] == c {
                j += 1;
            }
            let mut left = (j - i) as u32;
            while left > 0 {
                let len = left.min(CAP);
                self.runs.push((c as u16) << 8 | len as u16);
                per[c as usize] += 1;
                left -= len;
            }
            i = j;
        }
        let r = self.runs.len();
        if r >= 1 << 18 {
            return None;
        }
        // each byte's first F index and first F row
        let (mut nextj, mut nexta) = ([0u32; 256], [0u32; 256]);
        let (mut j, mut a) = (0, 0);
        for c in 0..256 {
            nextj[c] = j;
            nexta[c] = a;
            j += per[c];
            a += counts[c];
        }
        // pass 2: each run's F index, and the cells' F starts
        self.order.clear();
        self.order.reserve(r);
        self.start.clear();
        self.start.resize(r + 1, 0);
        self.cells.clear();
        self.cells.resize(r + 2, 0);
        for &run in &self.runs {
            let (c, len) = ((run >> 8) as usize, run as u32 & 255);
            let j = nextj[c];
            nextj[c] += 1;
            self.start[j as usize] = nexta[c];
            nexta[c] += len;
            self.cells[j as usize] = rec(0, 0, len, c as u8);
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
        // the sentinels: a cell no offset is past
        self.cells[r] = rec(0, 0, 127, 0);
        self.cells[r + 1] = rec(0, 0, 127, 0);
        self.memo.clear();
        self.memo.resize(r, u64::MAX);
        // the origin's cell
        let k = self.start[..r].partition_point(|&s| s <= orig_ptr) - 1;
        Some((k as u32, orig_ptr - self.start[k]))
    }

    /// The walk from (cell, offset): n bytes into `out`
    pub fn walk(&mut self, n: usize, origin: (u32, u32), out: &mut Vec<u8>) {
        #[cfg(target_arch = "x86_64")]
        assert!(
            is_x86_feature_detected!("avx2"),
            "the prototype's copy loop is AVX2"
        );
        unsafe { self.walk_impl(n, origin, out) }
    }

    #[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx2"))]
    unsafe fn walk_impl(&mut self, n: usize, origin: (u32, u32), out: &mut Vec<u8>) {
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        // the streams are written before they are read; their capacity is kept between blocks
        out.clear();
        out.reserve(n + 64);
        self.st.clear();
        self.st.reserve(n + 64);
        let (mut real, mut copies) = (0usize, 0usize);
        let cells = self.cells.as_ptr();
        let memo = self.memo.as_mut_ptr();
        let st = self.st.as_mut_ptr();
        let o_ = out.as_mut_ptr();
        let (mut k, mut o) = origin;
        let mut t = 0usize;
        #[cfg(target_arch = "x86_64")]
        let v127 = _mm256_set1_epi32(127);
        while t < n {
            let r = *cells.add(k as usize);
            let m = *memo.add(k as usize);
            let me = (t as u64) | (o as u64) << 20;
            if m == u64::MAX {
                // the cell's first visit: a real step
                *memo.add(k as usize) = me | me << 27;
                *st.add(t) = k << 14 | o << 7 | (len_of(r) - 1 - o);
                *o_.add(t) = (r >> 34) as u8;
                t += 1;
                real += 1;
                k = r as u32 & 0xfffff;
                o += (r >> 20) as u32 & 127;
                loop {
                    let l = len_of(*cells.add(k as usize));
                    if o < l {
                        break;
                    }
                    o -= l;
                    k += 1;
                }
                continue;
            }
            // the nearer in rows of the cell's first and latest visits
            let (q1, o1) = ((m & 0xfffff) as usize, (m >> 20) as u32 & 127);
            let (q2, o2) = ((m >> 27 & 0xfffff) as usize, (m >> 47) as u32 & 127);
            let (d1, d2) = (o as i32 - o1 as i32, o as i32 - o2 as i32);
            let (q, d) = if d1.abs() <= d2.abs() {
                (q1, d1)
            } else {
                (q2, d2)
            };
            *memo.add(k as usize) = (m & 0x7ff_ffff) | me << 27;
            copies += 1;
            // copy while the source's trajectory has d rows of room on the walk's side
            let kd = (d << 7).wrapping_sub(d) as u32;
            let (shift, need) = if d > 0 {
                (0, d as u32)
            } else {
                (7, (-d) as u32)
            };
            let mut i = 0usize;
            let s;
            #[cfg(target_arch = "x86_64")]
            if t - q >= 8 {
                let vneed = _mm256_set1_epi32(need as i32);
                let vk = _mm256_set1_epi32(kd as i32);
                let sh = _mm_cvtsi32_si128(shift);
                while t + i + 8 <= n {
                    let v = _mm256_loadu_si256(st.add(q + i) as *const __m256i);
                    let room = _mm256_and_si256(_mm256_srl_epi32(v, sh), v127);
                    let bad =
                        _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(vneed, room)));
                    if bad != 0 {
                        break;
                    }
                    _mm256_storeu_si256(st.add(t + i) as *mut __m256i, _mm256_add_epi32(v, vk));
                    (o_.add(t + i) as *mut u64)
                        .write_unaligned((o_.add(q + i) as *const u64).read_unaligned());
                    i += 8;
                }
            }
            loop {
                if t + i == n {
                    s = 0;
                    break;
                }
                let v = *st.add(q + i);
                if (v >> shift & 127) < need {
                    s = v;
                    break;
                }
                *st.add(t + i) = v.wrapping_add(kd);
                *o_.add(t + i) = *o_.add(q + i);
                i += 1;
            }
            t += i;
            if t == n {
                break;
            }
            // the source is at (cell, above, below): the walk is d rows from it
            k = s >> 14;
            if d > 0 {
                o = d as u32 - (s & 127) - 1;
                k += 1;
                loop {
                    let l = len_of(*cells.add(k as usize));
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
                    let l = len_of(*cells.add(k as usize));
                    if u < l {
                        o = l - 1 - u;
                        break;
                    }
                    u -= l;
                }
            }
        }
        out.set_len(n);
        self.real = real;
        self.copies = copies;
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
    }
    v
}

/// One pass over the blocks in decode order with one arm: (ns in build, ns in walk, bytes)
fn pass(bs: &[Block], arm: u8) -> (u128, u128, usize) {
    let (mut tb, mut tw, mut bytes) = (0, 0, 0);
    let (mut tt, mut seg, mut out) = (TVector::new(), SegScratch::default(), Vec::new());
    let mut ph = Phrase::default();
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
            let origin = ph
                .build(&b.ll, &b.counts, b.orig_ptr)
                .expect("a block of 2^18 cells or more");
            let t1 = Instant::now();
            ph.walk(b.ll.len(), origin, &mut out);
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
            let (mut tt, mut want, mut got) = (TVector::new(), Vec::new(), Vec::new());
            let mut ph = Phrase::default();
            let (mut real, mut copies, mut rows) = (0, 0, 0);
            for (i, b) in bs.iter().enumerate() {
                if b.ll.is_empty() {
                    continue;
                }
                build(&b.ll, &b.counts, &mut tt);
                want.clear();
                walk(&tt, b.orig_ptr, &mut want);
                let origin = ph.build(&b.ll, &b.counts, b.orig_ptr).expect("cells");
                ph.walk(b.ll.len(), origin, &mut got);
                assert!(want == got, "block {i} differs");
                real += ph.real;
                copies += ph.copies;
                rows += b.ll.len();
            }
            println!(
                "{} blocks, {rows} rows: identical. real steps {:.4}/row, copies {:.4}/row",
                bs.len(),
                real as f64 / rows as f64,
                copies as f64 / rows as f64
            );
        }
        "time" => {
            let rounds: usize = args[3].parse().unwrap();
            let copies: usize = args[4].parse().unwrap();
            for round in 0..rounds {
                for a in 0..2u8 {
                    let arm = (a + round as u8) % 2;
                    let t0 = Instant::now();
                    let res: Vec<(u128, u128, usize)> = std::thread::scope(|s| {
                        let hs: Vec<_> = (0..copies).map(|_| s.spawn(|| pass(&bs, arm))).collect();
                        hs.into_iter().map(|h| h.join().unwrap()).collect()
                    });
                    let wall = t0.elapsed().as_nanos() as f64;
                    let bytes: usize = res.iter().map(|r| r.2).sum();
                    let tb: u128 = res.iter().map(|r| r.0).sum();
                    let tw: u128 = res.iter().map(|r| r.1).sum();
                    println!(
                        "round {round} {} copies {copies}: build {:.3} walk {:.3} total {:.3} ns/B a thread; {:.0} MB/s in all",
                        if arm == 0 { "production" } else { "phrase    " },
                        tb as f64 / bytes as f64,
                        tw as f64 / bytes as f64,
                        (tb + tw) as f64 / bytes as f64,
                        bytes as f64 / wall * 1e3
                    );
                }
            }
        }
        "knee" => {
            let rounds: usize = args[3].parse().unwrap();
            let grid: Vec<usize> = args[4].split(',').map(|c| c.parse().unwrap()).collect();
            for round in 0..rounds {
                for (gi, &copies) in grid.iter().enumerate() {
                    for a in 0..2u8 {
                        // the arms alternate their order by round and by grid position
                        let arm = (a + round as u8 + gi as u8) % 2;
                        let t0 = Instant::now();
                        let res: Vec<(u128, u128, usize)> = std::thread::scope(|s| {
                            let hs: Vec<_> =
                                (0..copies).map(|_| s.spawn(|| pass(&bs, arm))).collect();
                            hs.into_iter().map(|h| h.join().unwrap()).collect()
                        });
                        let wall = t0.elapsed().as_nanos();
                        let bytes: usize = res.iter().map(|r| r.2).sum();
                        let tb: u128 = res.iter().map(|r| r.0).sum();
                        let tw: u128 = res.iter().map(|r| r.1).sum();
                        println!(
                            "{{\"round\":{round},\"arm\":\"{}\",\"copies\":{copies},\"wall_ns\":{wall},\"bytes\":{bytes},\"build_ns\":{tb},\"walk_ns\":{tw}}}",
                            if arm == 0 { "production" } else { "phrase" }
                        );
                    }
                }
            }
        }
        _ => panic!("check, time or knee"),
    }
}
