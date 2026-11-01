//! Research only (the minimal real-T-vector walker diagnostic; reports 132, 134, 135;
//! docs/journal/2026-11-01-walker-ladder-protocol.md): one real block's T-vector, walked by a
//! ladder of kernels, each adding one cost of the shipped walk to the one below it, so the
//! counters can say where a row's cycles go. Not the decoder.
//!
//! ```text
//! walker_ladder FILE.bz2 RUNG LANES REPS [BLOCK]    one rung, REPS timed walks of one block
//! walker_ladder FILE.bz2 check [BLOCK]              every rung at every lane count, checked
//! ```
//!
//! The block is the first full one (at least 899,000 rows) unless BLOCK names another. Its
//! last column is made by the library's scanner and MTF; its T-vector is built twice: by the
//! library (`bwt::build`, for the shipped rung) and by this file's copy of the same build
//! (for the others), whose entry is the library's, `link << 8 | byte`. The segment starts
//! are the shipped walk's: 511 xorshift rows (`bwt::random_rows`, copied) and the row the
//! output begins in.
//!
//! The rungs, each adding one cost to the one before:
//! - `A` the bare chase: k lanes, each following T from a segment's start for the
//!   segment's length (known in advance, a counter), summing the bytes; no prefetch, no
//!   flags, no stores;
//! - `B` A, with each lane's next entry prefetched into L1 (the shipped prefetch, one lane
//!   turn ahead);
//! - `C` B, ending a segment at a flagged row (the shipped START bit) instead of a counter;
//! - `D` C, with each byte stored: into the output itself, at the segment's place in it
//!   (known in advance), so no buffer grows and nothing is assembled;
//! - `P` the shipped walk itself: `bwt::walk_segments_room_with_prefetch::<k, PF_L1>`, its
//!   lanes, room passes, buffers, growth and assembly;
//! - `S` the shipped walk's assembly alone (`SegScratch::assemble`), to take from P.
//!
//! Every rung's every walk is checked: A–C by the rows stepped (n) and the bytes' sum (L's,
//! when each row is visited once), D and P by the output's hash (the library's output).
//!
//! A run prints one JSON line: the rung, lanes, reps, rows, seconds, ns a row and the
//! checks. With REPS 0 it does the setup only, so a counter run at REPS less one at 0 is the
//! walks' own count (bench/walker_ladder.py).

use std::time::Instant;

use mibz2::bwt::{build, walk_segments_room_with_prefetch, SegScratch, TVector, PF_L1};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const START: u32 = 1 << 31;
const SEGMENTS: usize = 512;
const FULL: usize = 899_000;

/// production's `random_rows` (src/bwt.rs), copied: `m` xorshift rows below `n`
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

/// The T-vector production builds (the entry `link << 8 | byte`, the link of row k being the
/// row of L whose byte is F's k-th): its plain two-pass form
fn build_t(ll: &[u8], counts: &[u32; 256]) -> Vec<u32> {
    let mut t: Vec<u32> = ll.iter().map(|&c| c as u32).collect();
    let mut ftab = [0u32; 256];
    let mut cum = 0;
    for c in 0..256 {
        ftab[c] = cum;
        cum += counts[c];
    }
    for (i, &c) in ll.iter().enumerate() {
        let k = ftab[c as usize] as usize;
        t[k] |= (i as u32) << 8;
        ftab[c as usize] += 1;
    }
    t
}

fn fnv(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| {
        (h ^ x as u64).wrapping_mul(0x100_0000_01b3)
    })
}

/// The block's segments, the shipped walk's: their starts (sorted), lengths in rows, and
/// places in the output
struct Segs {
    starts: Vec<u32>,
    len: Vec<u32>,
    at: Vec<usize>,
}

fn segments(t: &[u32], orig_ptr: u32) -> Segs {
    let n = t.len();
    let r0 = t[orig_ptr as usize] >> 8;
    let mut starts = Vec::new();
    random_rows(n, SEGMENTS - 1, &mut starts);
    starts.push(r0);
    starts.sort_unstable();
    starts.dedup();
    let is_start = |r: u32| starts.binary_search(&r).is_ok();
    let (mut len, mut next) = (vec![0u32; starts.len()], vec![0usize; starts.len()]);
    for (s, &st) in starts.iter().enumerate() {
        let (mut r, mut c) = (t[st as usize] >> 8, 1u32);
        while !is_start(r) {
            r = t[r as usize] >> 8;
            c += 1;
        }
        len[s] = c;
        next[s] = starts.binary_search(&r).unwrap();
    }
    // the output begins in r0's segment and follows each segment's successor
    let mut at = vec![usize::MAX; starts.len()];
    let (mut s, mut pos) = (starts.binary_search(&r0).unwrap(), 0usize);
    while at[s] == usize::MAX {
        at[s] = pos;
        pos += len[s] as usize;
        s = next[s];
    }
    assert_eq!(pos, n, "the segments cover the block");
    Segs { starts, len, at }
}

#[inline(always)]
fn prefetch(t: *const u32, row: u32) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch never faults
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(
            t.wrapping_add(row as usize) as *const i8,
        )
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = (t, row);
}

/// One walk of the block by rung A, B, C or D with K lanes: the rows stepped and the bytes'
/// sum (A–C), or the output written (D). `t` carries START flags at the starts for C and D.
///
/// Every lane takes the next segment as it ends one; when no segment is left, the lanes
/// still walking finish theirs one turn each, as the shipped walk's tail does.
#[inline(never)]
fn ladder<const K: usize, const PF: bool, const FLAG: bool, const STORE: bool>(
    t: &[u32],
    sg: &Segs,
    out: &mut [u8],
) -> (u64, u64) {
    let tp = t.as_ptr();
    let ns = sg.starts.len();
    let (mut row, mut rem, mut ptr) = ([0u32; K], [0u32; K], [0usize; K]);
    let (mut steps, mut sum) = (0u64, 0u64);
    let mut queued = 0;
    // a lane's segment taken: its first step, from the start's own entry
    let take = |s: usize, row: &mut u32, rem: &mut u32, ptr: &mut usize, out: &mut [u8]| -> u8 {
        let q = t[sg.starts[s] as usize] & !START;
        *row = q >> 8;
        *rem = sg.len[s] - 1;
        if STORE {
            out[sg.at[s]] = q as u8;
            *ptr = sg.at[s] + 1;
        }
        q as u8
    };
    for k in 0..K.min(ns) {
        sum += take(queued, &mut row[k], &mut rem[k], &mut ptr[k], out) as u64;
        steps += 1;
        if PF {
            prefetch(tp, row[k]);
        }
        queued += 1;
    }
    let mut live = K.min(ns);
    if live == K {
        'full: loop {
            for k in 0..K {
                // SAFETY: rows are below n (links from the build, starts are rows)
                let q = unsafe { *tp.add(row[k] as usize) };
                let end = if FLAG { q & START != 0 } else { rem[k] == 0 };
                if end {
                    if queued == ns {
                        // this lane is done; the others finish in the tail
                        row[k] = u32::MAX;
                        break 'full;
                    }
                    sum += take(queued, &mut row[k], &mut rem[k], &mut ptr[k], out) as u64;
                    steps += 1;
                    if PF {
                        prefetch(tp, row[k]);
                    }
                    queued += 1;
                    continue;
                }
                if !FLAG {
                    rem[k] -= 1;
                }
                row[k] = q >> 8;
                if PF {
                    prefetch(tp, row[k]);
                }
                if STORE {
                    // SAFETY: ptr is within the segment's place in the output
                    unsafe { *out.get_unchecked_mut(ptr[k]) = q as u8 };
                    ptr[k] += 1;
                } else {
                    sum += (q & 0xff) as u64;
                }
                steps += 1;
            }
        }
        live = K - 1;
    }
    // the tail: the lanes still walking, one turn each until each reaches its end
    let mut ks: Vec<usize> = (0..K.min(ns)).filter(|&k| row[k] != u32::MAX).collect();
    debug_assert_eq!(ks.len(), live);
    while !ks.is_empty() {
        let mut i = 0;
        while i < ks.len() {
            let k = ks[i];
            let q = t[row[k] as usize];
            let end = if FLAG { q & START != 0 } else { rem[k] == 0 };
            if end {
                ks.swap_remove(i);
                continue;
            }
            if !FLAG {
                rem[k] -= 1;
            }
            row[k] = q >> 8;
            if PF {
                prefetch(tp, row[k]);
            }
            if STORE {
                out[ptr[k]] = q as u8;
                ptr[k] += 1;
            } else {
                sum += (q & 0xff) as u64;
            }
            steps += 1;
            i += 1;
        }
    }
    (steps, sum)
}

/// A rung at a lane count, as a function
type Kernel = fn(&[u32], &Segs, &mut [u8]) -> (u64, u64);

fn kernel(rung: char, lanes: usize) -> Kernel {
    macro_rules! k {
        ($($n:literal),*) => {
            match (rung, lanes) {
                $(
                    ('A', $n) => ladder::<$n, false, false, false>,
                    ('B', $n) => ladder::<$n, true, false, false>,
                    ('C', $n) => ladder::<$n, true, true, false>,
                    ('D', $n) => ladder::<$n, true, true, true>,
                )*
                _ => panic!("rungs A-D take lanes 1, 8, 16, 32"),
            }
        };
    }
    k!(1, 8, 16, 32)
}

fn shipped(lanes: usize) -> fn(&mut TVector, u32, &mut Vec<u8>, &mut SegScratch, usize) {
    match lanes {
        1 => walk_segments_room_with_prefetch::<1, PF_L1>,
        8 => walk_segments_room_with_prefetch::<8, PF_L1>,
        16 => walk_segments_room_with_prefetch::<16, PF_L1>,
        32 => walk_segments_room_with_prefetch::<32, PF_L1>,
        _ => panic!("rung P takes lanes 1, 8, 16, 32"),
    }
}

/// The block: its last column, counts and primary index
fn block(data: &[u8], want: Option<usize>) -> (usize, Vec<u8>, [u32; 256], u32) {
    let kern = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut i = 0;
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&symbols, &mut ll, &mut counts, kern).unwrap();
        if want.map_or(ll.len() >= FULL, |w| w == i) {
            return (i, ll, counts, symbols.orig_ptr);
        }
        i += 1;
    }
    panic!("no such block");
}

struct Setup {
    index: usize,
    t: Vec<u32>,
    tf: Vec<u32>,
    tv: TVector,
    orig_ptr: u32,
    sg: Segs,
    sum: u64,
    want: u64,
}

fn setup(data: &[u8], want: Option<usize>) -> Setup {
    let (index, ll, counts, orig_ptr) = block(data, want);
    let t = build_t(&ll, &counts);
    let mut tv = TVector::new();
    build(&ll, &counts, &mut tv);
    let sg = segments(&t, orig_ptr);
    let mut tf = t.clone();
    for &r in &sg.starts {
        tf[r as usize] |= START;
    }
    // the shipped walk's output, the reference
    let mut out = Vec::new();
    walk_segments_room_with_prefetch::<32, PF_L1>(
        &mut tv,
        orig_ptr,
        &mut out,
        &mut SegScratch::default(),
        SEGMENTS,
    );
    let sum = ll.iter().map(|&b| b as u64).sum();
    Setup {
        index,
        t,
        tf,
        tv,
        orig_ptr,
        sg,
        sum,
        want: fnv(&out),
    }
}

/// One rung's REPS walks, each checked after the clock stops: (seconds, every check passed)
fn run(s: &mut Setup, rung: char, lanes: usize, reps: usize) -> (f64, bool) {
    let n = s.t.len();
    let mut out = vec![0u8; n];
    let mut ok = true;
    let mut secs = 0.0;
    match rung {
        'A' | 'B' | 'C' | 'D' => {
            let f = kernel(rung, lanes);
            let t = if matches!(rung, 'C' | 'D') {
                &s.tf
            } else {
                &s.t
            };
            for _ in 0..reps {
                let c = Instant::now();
                let (steps, sum) = f(t, &s.sg, &mut out);
                secs += c.elapsed().as_secs_f64();
                ok &= steps == n as u64;
                ok &= if rung == 'D' {
                    fnv(&out) == s.want
                } else {
                    sum == s.sum
                };
            }
        }
        'P' => {
            let f = shipped(lanes);
            let (mut o, mut sc) = (Vec::new(), SegScratch::default());
            for _ in 0..reps {
                let c = Instant::now();
                f(&mut s.tv, s.orig_ptr, &mut o, &mut sc, SEGMENTS);
                secs += c.elapsed().as_secs_f64();
                ok &= fnv(&o) == s.want;
            }
        }
        'S' => {
            let (mut o, mut sc) = (Vec::new(), SegScratch::default());
            walk_segments_room_with_prefetch::<32, PF_L1>(
                &mut s.tv, s.orig_ptr, &mut o, &mut sc, SEGMENTS,
            );
            for _ in 0..reps {
                let c = Instant::now();
                sc.assemble(&mut o);
                secs += c.elapsed().as_secs_f64();
                ok &= fnv(&o) == s.want;
            }
        }
        _ => panic!("rungs are A, B, C, D, P and S"),
    }
    (secs, ok)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(args.get(1).expect("walker_ladder FILE.bz2 ...")).unwrap();
    let mode = args.get(2).map(String::as_str).unwrap_or("check");
    if mode == "check" {
        let mut s = setup(&data, args.get(3).map(|b| b.parse().unwrap()));
        let n = s.t.len();
        for rung in ['A', 'B', 'C', 'D', 'P'] {
            for lanes in [1, 8, 16, 32] {
                let (_, ok) = run(&mut s, rung, lanes, 2);
                assert!(ok, "rung {rung} at {lanes} lanes fails its check");
            }
        }
        assert!(run(&mut s, 'S', 32, 2).1, "S fails its check");
        println!(
            "check: block {} ({n} rows, {} segments): rungs A-D and P at 1, 8, 16 and 32 lanes, and S: every walk visits every row once and D's and P's outputs are the shipped walk's (fnv {:016x})",
            s.index,
            s.sg.starts.len(),
            s.want
        );
        return;
    }
    let rung = mode.chars().next().unwrap();
    let lanes: usize = args.get(3).expect("LANES").parse().unwrap();
    let reps: usize = args.get(4).expect("REPS").parse().unwrap();
    let mut s = setup(&data, args.get(5).map(|b| b.parse().unwrap()));
    let n = s.t.len();
    let (secs, ok) = run(&mut s, rung, lanes, reps);
    assert!(ok, "rung {rung} at {lanes} lanes fails its check");
    println!(
        "{{\"rung\": \"{rung}\", \"lanes\": {lanes}, \"reps\": {reps}, \"block\": {}, \"rows\": {n}, \"segments\": {}, \"seconds\": {secs:.6}, \"ns_a_row\": {:.4}, \"checked\": true, \"output_fnv\": \"{:016x}\"}}",
        s.index,
        s.sg.starts.len(),
        if reps > 0 { 1e9 * secs / (reps * n) as f64 } else { 0.0 },
        s.want
    );
}
