//! The interval engine (docs/journal/2026-11-10-interval-exchange.md): an inverse BWT that
//! moves intervals of rows through a tower of first-return maps, timed against the
//! production build and room walk on the same blocks. A prototype: nothing in the decoder
//! calls it, and its timing here is indicative only (RULES C5).
//!
//! ```text
//! iet_engine FILE.bz2 [EVERY] [RATIO] [LEVELS] [ROUNDS]
//! ```
//!
//! **What is timed, a block at a time, the fastest of ROUNDS:**
//! - `walk`: `bwt::build` from the last column, then `bwt::walk_segments_room`;
//! - `engine`: from the block's runs (start, byte: what a run-record front hands over; made
//!   from the column outside the clock), the LF exchange's table, the tower, the orbit of its
//!   top map, and the block's bytes in text order.
//!
//! Both end with the block's bytes; they are compared.

use mibz2::bwt::{build, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::time::Instant;

/// An interval exchange with a word on each piece. Piece k is the points start[k]..start[k +
/// 1], sent to land[k] + (x - start[k]), emitting words[wfrom[k]..wfrom[k + 1]] (in the order
/// the walk emits: the text reversed). `start` and `wfrom` end with a sentinel.
#[derive(Default)]
struct Map {
    size: u32,
    /// bit x set: x starts a piece; and the set bits before each word
    bits: Vec<u64>,
    cum: Vec<u32>,
    start: Vec<u32>,
    land: Vec<u32>,
    wfrom: Vec<u32>,
    words: Vec<u8>,
}

impl Map {
    fn clear(&mut self, size: u32) {
        self.size = size;
        self.start.clear();
        self.land.clear();
        self.wfrom.clear();
        self.words.clear();
    }

    /// The sentinels and the piece index, once the pieces are in
    fn seal(&mut self) {
        self.start.push(self.size);
        self.wfrom.push(self.words.len() as u32);
        let words = self.size as usize / 64 + 1;
        self.bits.clear();
        self.bits.resize(words, 0);
        for &s in &self.start[..self.start.len() - 1] {
            self.bits[(s >> 6) as usize] |= 1 << (s & 63);
        }
        self.cum.clear();
        let mut c = 0u32;
        for &b in &self.bits {
            self.cum.push(c);
            c += b.count_ones();
        }
    }

    #[inline(always)]
    fn piece(&self, x: u32) -> usize {
        let w = (x >> 6) as usize;
        let m = self.bits[w] & (u64::MAX >> (63 - (x & 63)));
        (self.cum[w] + m.count_ones()) as usize - 1
    }

    #[inline(always)]
    fn word(&self, k: usize) -> &[u8] {
        &self.words[self.wfrom[k] as usize..self.wfrom[k + 1] as usize]
    }
}

struct Work {
    x: u32,
    w: u32,
    origin: u32,
    /// the path's length when this piece was cut off
    top: u32,
    /// it has returned: x is where it landed
    done: bool,
}

#[derive(Default)]
struct Scratch {
    stack: Vec<Work>,
    path: Vec<u8>,
}

/// The first-return map of `m` to J = [a, a + big_w), into `out` (an exchange on [0, big_w)
/// whose words are the return words). `BYTES`: `m`'s words are one byte each.
fn induce<const BYTES: bool>(m: &Map, a: u32, big_w: u32, out: &mut Map, s: &mut Scratch) {
    out.clear(big_w);
    let (ja, jb) = (a, a + big_w);
    let (stack, path) = (&mut s.stack, &mut s.path);
    stack.clear();
    path.clear();
    stack.push(Work {
        x: a,
        w: big_w,
        origin: a,
        top: 0,
        done: false,
    });
    // (pieces are taken leftmost first, and a cut's parts pushed rightmost first: the leaves
    // come out in J's order, and the path below a piece's `top` is its ancestors' bytes)
    while let Some(p) = stack.pop() {
        path.truncate(p.top as usize);
        let (mut x, w) = (p.x, p.w);
        if !p.done {
            loop {
                let k = m.piece(x);
                let e = x + w;
                if e > m.start[k + 1] {
                    // not rigid: cut at the pieces' starts inside it
                    let kl = m.piece(e - 1);
                    let top = path.len() as u32;
                    for kk in (k..=kl).rev() {
                        let from = x.max(m.start[kk]);
                        let to = e.min(m.start[kk + 1]);
                        stack.push(Work {
                            x: from,
                            w: to - from,
                            origin: p.origin + (from - x),
                            top,
                            done: false,
                        });
                    }
                    break;
                }
                if BYTES {
                    path.push(m.words[k]);
                } else {
                    path.extend_from_slice(m.word(k));
                }
                let y = m.land[k] + (x - m.start[k]);
                let e = y + w;
                if e <= ja || y >= jb {
                    x = y;
                    continue;
                }
                if y >= ja && e <= jb {
                    x = y;
                    out.start.push(p.origin - ja);
                    out.land.push(x - ja);
                    out.wfrom.push(out.words.len() as u32);
                    out.words.extend_from_slice(path);
                    break;
                }
                // it straddles an edge of J: cut there; the part inside has returned
                let top = path.len() as u32;
                let mut cuts = [y, ja.clamp(y, e), jb.clamp(y, e), e];
                cuts.sort_unstable();
                for i in (0..3).rev() {
                    let (from, to) = (cuts[i], cuts[i + 1]);
                    if to > from {
                        stack.push(Work {
                            x: from,
                            w: to - from,
                            origin: p.origin + (from - y),
                            top,
                            done: from >= ja && to <= jb,
                        });
                    }
                }
                break;
            }
        } else {
            out.start.push(p.origin - ja);
            out.land.push(x - ja);
            out.wfrom.push(out.words.len() as u32);
            out.words.extend_from_slice(path);
        }
    }
    out.seal();
}

#[derive(Default)]
struct Engine {
    base: Map,
    maps: Vec<Map>,
    offs: Vec<u32>,
    scratch: Scratch,
    /// the block, last byte first
    back: Vec<u8>,
    /// seconds in: the base map, each induction, the orbit, the reversal
    phase: [f64; 8],
}

impl Engine {
    /// The block of `n` rows whose runs of L start at `starts` with bytes `bytes`, walked from
    /// `orig_ptr`: its bytes in text order into `out`
    #[allow(clippy::too_many_arguments)]
    fn decode(
        &mut self,
        n: usize,
        starts: &[u32],
        bytes: &[u8],
        orig_ptr: u32,
        ratio: u32,
        levels: usize,
        out: &mut Vec<u8>,
    ) {
        let t0 = Instant::now();
        // LF as an exchange: a run's first row goes to its byte's cursor
        let base = &mut self.base;
        base.clear(n as u32);
        let mut cnt = [0u32; 256];
        for (k, &s) in starts.iter().enumerate() {
            let end = starts.get(k + 1).copied().unwrap_or(n as u32);
            cnt[bytes[k] as usize] += end - s;
        }
        let mut next = [0u32; 256];
        let mut cum = 0;
        for c in 0..256 {
            next[c] = cum;
            cum += cnt[c];
        }
        base.start.extend_from_slice(starts);
        base.words.extend_from_slice(bytes);
        for (k, &s) in starts.iter().enumerate() {
            let end = starts.get(k + 1).copied().unwrap_or(n as u32);
            let c = bytes[k] as usize;
            base.land.push(next[c]);
            base.wfrom.push(k as u32);
            next[c] += end - s;
        }
        base.seal();
        self.phase[0] += t0.elapsed().as_secs_f64();
        // the tower
        while self.maps.len() < levels {
            self.maps.push(Map::default());
        }
        self.offs.clear();
        for lv in 0..levels {
            let (below, above) = self.maps.split_at_mut(lv);
            let m = if lv == 0 { &self.base } else { &below[lv - 1] };
            let big_w = (m.size / ratio).max(1);
            let a = (m.size - big_w) / 2;
            let t1 = Instant::now();
            if lv == 0 {
                induce::<true>(m, a, big_w, &mut above[0], &mut self.scratch);
            } else {
                induce::<false>(m, a, big_w, &mut above[0], &mut self.scratch);
            }
            self.phase[1 + lv.min(3)] += t1.elapsed().as_secs_f64();
            self.offs.push(a);
        }
        let t2 = Instant::now();
        // the orbit: down the tower until the walk is inside each J, then the top map
        let back = &mut self.back;
        back.clear();
        back.reserve(n + (1 << 16));
        let mut x = orig_ptr;
        for lv in 0..levels {
            let m = if lv == 0 {
                &self.base
            } else {
                &self.maps[lv - 1]
            };
            let (ja, jb) = (self.offs[lv], self.offs[lv] + self.maps[lv].size);
            while !(x >= ja && x < jb) && back.len() < n {
                let k = m.piece(x);
                back.extend_from_slice(m.word(k));
                x = m.land[k] + (x - m.start[k]);
            }
            x = x.wrapping_sub(ja);
        }
        let top = if levels == 0 {
            &self.base
        } else {
            &self.maps[levels - 1]
        };
        while back.len() < n {
            let k = top.piece(x);
            back.extend_from_slice(top.word(k));
            x = top.land[k] + (x - top.start[k]);
        }
        back.truncate(n);
        self.phase[5] += t2.elapsed().as_secs_f64();
        let t3 = Instant::now();
        out.clear();
        out.extend_from_slice(back);
        out.reverse();
        self.phase[6] += t3.elapsed().as_secs_f64();
    }

    fn live_bytes(&self) -> usize {
        let m = |m: &Map| {
            m.bits.len() * 8
                + m.cum.len() * 4
                + m.start.len() * 4
                + m.land.len() * 4
                + m.wfrom.len() * 4
                + m.words.len()
        };
        m(&self.base) + self.maps.iter().map(m).sum::<usize>()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).expect("read");
    let arg = |i: usize, d: usize| args.get(i).map_or(d, |s| s.parse().unwrap());
    let (every, ratio, levels, rounds) = (arg(2, 1), arg(3, 8) as u32, arg(4, 2), arg(5, 5));
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut ll, mut counts, mut tt) = (Vec::new(), [0u32; 256], TVector::new());
    let mut seg = SegScratch::default();
    let mut syms = Vec::new();
    let (mut walked, mut got) = (Vec::new(), Vec::new());
    let mut engine = Engine::default();
    let (mut rows, mut runs, mut t_walk, mut t_engine, mut live) = (0u64, 0u64, 0f64, 0f64, 0usize);
    let (mut starts, mut bytes) = (Vec::new(), Vec::new());
    let mut i = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        if ll.len() >= 65536 && i % every == 0 {
            let n = ll.len();
            starts.clear();
            bytes.clear();
            for j in 0..n {
                if j == 0 || ll[j] != ll[j - 1] {
                    starts.push(j as u32);
                    bytes.push(ll[j]);
                }
            }
            let (mut best_w, mut best_e) = (f64::MAX, f64::MAX);
            for r in 0..rounds {
                for which in [r % 2, 1 - r % 2] {
                    if which == 0 {
                        let t = Instant::now();
                        build(&ll, &counts, &mut tt);
                        walk_segments_room(&mut tt, b.orig_ptr, &mut walked, &mut seg);
                        best_w = best_w.min(t.elapsed().as_secs_f64());
                    } else {
                        let t = Instant::now();
                        engine.decode(n, &starts, &bytes, b.orig_ptr, ratio, levels, &mut got);
                        best_e = best_e.min(t.elapsed().as_secs_f64());
                    }
                }
            }
            assert!(got == walked, "the engine's bytes differ from the walk's");
            rows += n as u64;
            runs += starts.len() as u64;
            t_walk += best_w;
            t_engine += best_e;
            live = live.max(engine.live_bytes());
        }
        i += 1;
        syms = b.syms;
    }
    let ph: Vec<String> = engine
        .phase
        .iter()
        .map(|p| format!("{:.3}", p * 1e9 / rows as f64 / rounds as f64))
        .collect();
    println!("  engine phases, ns/row averaged over all rounds: base {} | induce {} {} {} {} | orbit {} | reverse {}", ph[0], ph[1], ph[2], ph[3], ph[4], ph[5], ph[6]);
    println!(
        "rows {rows} runs/row {:.4} tower {ratio}x{levels} | build+walk {:.3} ns/row | engine {:.3} ns/row | engine/walk {:.3} | engine live set at most {} kB (table: {} kB a 900 kB block)",
        runs as f64 / rows as f64,
        t_walk * 1e9 / rows as f64,
        t_engine * 1e9 / rows as f64,
        t_engine / t_walk,
        live / 1000,
        3600
    );
}
