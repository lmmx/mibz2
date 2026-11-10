//! Can the sequential scan be removed? A research harness, not a decoder: block starts
//! found by a bit-aligned search for the two 48-bit magics, each block's header and
//! Huffman symbols then read by the worker that decodes it, and the blocks accepted only
//! along the chain the sequential decoder walks.
//!
//! ```text
//! par_scan FILE.bz2 [finder|check|adversarial|fuzz|time|shapes|all] [ROUNDS]
//! par_scan FILE.bz2 knee ROUNDS WORKERS      (WORKERS: a list, as 1,2,4,6,8,10)
//! par_scan - faults
//!   finder       the finder's rate on FILE (looped), its candidates against `mibz2::scan`
//!   check        `mibz2::decode`'s bytes against libbzip2's; FILE decoded by the harness
//!                at 1-4 and 10 workers, against `mibz2::decode`; the process's CPU mask
//!                and its threads' (sampled), and its peak resident memory
//!   faults       a panic injected at each role (a worker, the finder, the scanner, the
//!                joiner) of each arm, on a small input made in memory: each decode must
//!                end with an error or the joiner's panic, never Ok, never a hang (untimed)
//!   adversarial  magics inside valid blocks, a valid block spliced into a payload,
//!                a stream after trailing bytes: the harness against the sequential decoder
//!   fuzz         ROUNDS (at least 1500) mutated small files: the bytes before an error and
//!                the error, against the sequential decoder's
//!   time         ROUNDS (default 7) rounds, arms alternating, at 1-4 threads: MB/s, CPU s
//!   shapes       worst-case shapes at 4 workers: incompressible, tiny streams, one block,
//!                false magics in every block
//!   knee         the find and scan arms at each worker count, ROUNDS rounds, a fixed
//!                schedule: every cell run once, no waiting for quiet and no retries. One
//!                JSON line a cell on stdout and nothing else there (bench/par_scan_knee.py
//!                reads them): {"round","arm","workers","wall_ns","cpu_ns","bytes"}
//! ```
//!
//! It holds the input, the decoded output and one decode's output at once (about 0.8 GB
//! on the 210 MB corpus). Timings wait for the rest of the machine to be quiet and mark
//! the runs that were not; on a shared machine they are indicative only.
//!
//! **The design.** Three roles:
//! - **the finder** (one thread): every bit at which either magic occurs, in input order.
//!   A block magic that follows "BZh1-9" at a byte is also a guess at a stream's level.
//! - **the workers**: each takes the next block-magic candidate and decodes the one item
//!   that starts there (`block::read_item`, then `decode_block`), with a guessed level. It
//!   reports where the item ended. A worker never decides that its candidate is a block.
//! - **the joiner** (the caller's thread) walks the chain exactly as `stream::Scanner`
//!   does: a stream header, then items, each starting at the bit the last one ended at. At
//!   each chain position it takes the worker's result for the candidate at exactly that
//!   bit, decoded at exactly the stream's level, or else reads the item itself (the
//!   fallback: `one_item`, the same two calls). Candidates the chain steps over are dropped.
//!
//! So a worker's result is used only where the sequential decoder would have read the same
//! item from the same bit at the same level, and what it then holds is what that read gives.
//! The finder is a hint: a false candidate costs time, a missed one costs time (the joiner
//! reads the block itself), and neither can change a byte or an error.

use mibz2::bits::Bits;
use mibz2::block::{read_item, Item, Symbols, BLOCK_MAGIC, END_MAGIC};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, Error, ErrorKind, Kernels, Scratch};
use std::collections::{BTreeMap, VecDeque};
use std::io::Read;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Instant;

// ---- The finder ----

const MASK48: u64 = (1 << 48) - 1;

/// A magic found: its first bit, and whether it is the stream end's
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cand {
    bit: u64,
    end: bool,
}

/// Both magics at all 8 bit shifts, as 16 byte-aligned patterns. A magic that starts `s`
/// bits into byte `k` covers bytes k+1..=k+5 whole, whatever `s` is, so bytes k+1, k+2 and
/// k+3 are a byte-aligned fingerprint of it: a filter on them, then the 48 bits compared.
struct Finder {
    /// per fingerprint byte: for a low nibble, and for a high nibble, the shifts (a bit
    /// each) at which either magic has that nibble there
    lo: [[u8; 16]; 3],
    hi: [[u8; 16]; 3],
    /// the scalar filter: a bit for each value of bytes k+1 and k+2 that some pattern has
    pair: Vec<u64>,
    avx2: bool,
}

impl Finder {
    fn new(avx2: bool) -> Finder {
        let (mut lo, mut hi) = ([[0u8; 16]; 3], [[0u8; 16]; 3]);
        let mut pair = vec![0u64; 1 << 10];
        for m in [BLOCK_MAGIC, END_MAGIC] {
            for s in 0..8u32 {
                // the 8 bytes from byte k, the magic `s` bits in
                let w = (m << (16 - s)).to_be_bytes();
                for j in 0..3 {
                    lo[j][(w[j + 1] & 15) as usize] |= 1 << s;
                    hi[j][(w[j + 1] >> 4) as usize] |= 1 << s;
                }
                let idx = (w[1] as usize) << 8 | w[2] as usize;
                pair[idx >> 6] |= 1 << (idx & 63);
            }
        }
        // (a safe constructor: AVX2 only where asked for and the CPU has it, so no caller can
        // reach `find_avx2` on a CPU without it; R1 of the B5 review)
        #[cfg(target_arch = "x86_64")]
        let avx2 = avx2 && std::is_x86_feature_detected!("avx2");
        #[cfg(not(target_arch = "x86_64"))]
        let avx2 = {
            let _ = avx2;
            false
        };
        Finder { lo, hi, pair, avx2 }
    }

    fn detect() -> Finder {
        #[cfg(target_arch = "x86_64")]
        let avx2 = std::is_x86_feature_detected!("avx2");
        #[cfg(not(target_arch = "x86_64"))]
        let avx2 = false;
        Finder::new(avx2)
    }

    /// Every magic whose first bit lies in bytes `from..to` of `data`, in order, appended
    /// to `v`. It reads at most 7 bytes past `to`: a magic that starts in a range's last
    /// byte ends 6 bytes on, so ranges cut anywhere need a 6-byte overlap and no more.
    fn find(&self, data: &[u8], from: usize, to: usize, v: &mut Vec<Cand>) {
        // (a magic needs 6 bytes from its first)
        let to = to.min(data.len().saturating_sub(5));
        // (nothing to search: and `from` past the end must never reach the bounds below,
        // where `k + 40` could wrap; R2 of the B5 review)
        if from >= to {
            return;
        }
        let mut k = from;
        #[cfg(target_arch = "x86_64")]
        if self.avx2 {
            // SAFETY: `avx2` is set only where the CPU has AVX2 (`Finder::new` checks it), and
            // `from < to <= data.len() - 5` here, so `find_avx2`'s bounds cannot wrap
            k = unsafe { self.find_avx2(data, k, to, v) };
        }
        while k < to {
            let idx = (data[k + 1] as usize) << 8 | data[k + 2] as usize;
            if self.pair[idx >> 6] >> (idx & 63) & 1 != 0 {
                verify(data, k, v);
            }
            k += 1;
        }
    }

    /// `find`'s whole steps of 32 window starts; returns the first start not covered
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn find_avx2(&self, data: &[u8], from: usize, to: usize, v: &mut Vec<Cand>) -> usize {
        use std::arch::x86_64::*;
        let tbl = |t: &[u8; 16]| {
            _mm256_broadcastsi128_si256(_mm_loadu_si128(t.as_ptr() as *const __m128i))
        };
        let (l0, l1, l2) = (tbl(&self.lo[0]), tbl(&self.lo[1]), tbl(&self.lo[2]));
        let (h0, h1, h2) = (tbl(&self.hi[0]), tbl(&self.hi[1]), tbl(&self.hi[2]));
        let nib = _mm256_set1_epi8(0x0f);
        let zero = _mm256_setzero_si256();
        // the shifts each of 32 bytes can be the fingerprint's j-th byte of
        let class = |x: __m256i, l: __m256i, h: __m256i| {
            let a = _mm256_shuffle_epi8(l, _mm256_and_si256(x, nib));
            let b = _mm256_shuffle_epi8(h, _mm256_and_si256(_mm256_srli_epi16(x, 4), nib));
            _mm256_and_si256(a, b)
        };
        let mut k = from;
        // a step covers window starts k..k+32: it loads data[k+1..k+35], and a start's
        // check loads 8 bytes from it (through k+38)
        while k + 32 <= to && k + 40 <= data.len() {
            let p = data.as_ptr().add(k + 1);
            let c0 = class(_mm256_loadu_si256(p as *const __m256i), l0, h0);
            let c1 = class(_mm256_loadu_si256(p.add(1) as *const __m256i), l1, h1);
            let c2 = class(_mm256_loadu_si256(p.add(2) as *const __m256i), l2, h2);
            let c = _mm256_and_si256(_mm256_and_si256(c0, c1), c2);
            let mut m = !(_mm256_movemask_epi8(_mm256_cmpeq_epi8(c, zero)) as u32);
            while m != 0 {
                verify(data, k + m.trailing_zeros() as usize, v);
                m &= m - 1;
            }
            k += 32;
        }
        k
    }
}

/// The magics that start in byte `k`, at any of its 8 bits, appended to `v`
#[inline]
fn verify(data: &[u8], k: usize, v: &mut Vec<Cand>) {
    let w = match data.get(k..k + 8) {
        Some(b) => u64::from_be_bytes(b.try_into().unwrap()),
        None => {
            let mut b = [0u8; 8];
            b[..data.len() - k].copy_from_slice(&data[k..]);
            u64::from_be_bytes(b)
        }
    };
    for s in 0..8u32 {
        let x = w >> (16 - s) & MASK48;
        let bit = k as u64 * 8 + s as u64;
        // (wholly within the data: the zeros past its end are not input)
        if (x == BLOCK_MAGIC || x == END_MAGIC) && bit + 48 <= data.len() as u64 * 8 {
            v.push(Cand {
                bit,
                end: x == END_MAGIC,
            });
        }
    }
}

/// The reference finder: every bit tried
fn find_naive(data: &[u8]) -> Vec<Cand> {
    let mut v = Vec::new();
    let mut w = 0u64;
    for (i, &b) in data.iter().enumerate() {
        for j in 0..8 {
            w = w << 1 | (b >> (7 - j) & 1) as u64;
            let n = i as u64 * 8 + j + 1;
            let x = w & MASK48;
            if n >= 48 && (x == BLOCK_MAGIC || x == END_MAGIC) {
                v.push(Cand {
                    bit: n - 48,
                    end: x == END_MAGIC,
                });
            }
        }
    }
    v
}

// ---- One item, as the sequential decoder reads it ----

enum One {
    Block { end: u64, crc: u32 },
    End { end: u64, stored: u32 },
}

/// The item at `bit` of a stream of `level`: a block, its bytes appended to `out` and its
/// CRC checked, or the stream's end. What `stream::Scanner` on a slice and then
/// `decode_block` give for it, error for error (an item past the input's end is
/// `Truncated`, whatever else is wrong with it).
fn one_item(
    data: &[u8],
    bit: u64,
    level: u32,
    k: Kernels,
    sc: &mut Scratch,
    syms: &mut Vec<u16>,
    out: &mut Vec<u8>,
) -> Result<One, ErrorKind> {
    let mut br = Bits::at(data, bit);
    let item = read_item(&mut br, level, std::mem::take(syms));
    let (overran, end) = (br.overran(), br.bit_pos());
    match item {
        Err((kind, s)) => {
            *syms = s;
            Err(if overran { ErrorKind::Truncated } else { kind })
        }
        Ok(Item::End(_)) if overran => Err(ErrorKind::Truncated),
        Ok(Item::End(stored)) => Ok(One::End { end, stored }),
        Ok(Item::Block(b)) => {
            let r = if overran {
                Err(ErrorKind::Truncated)
            } else {
                decode_block(&b, k, sc, out)
            };
            let crc = b.crc;
            *syms = b.syms;
            r.map(|_| One::Block { end, crc })
        }
    }
}

// ---- The harness: finder, workers that read their own blocks, the chain ----

/// A block-magic candidate; `hdr`: the level of the stream header it follows, if the 4
/// bytes before it are one's
#[derive(Clone, Copy)]
struct Task {
    bit: u64,
    hdr: Option<u32>,
}

/// A worker's read of the item at a candidate: the level it assumed, and the block's end
/// bit, stored CRC and bytes, or what was wrong
struct Outcome {
    level: u32,
    res: Result<(u64, u32, Vec<u8>), ErrorKind>,
}

#[derive(Default, Clone, Copy, Debug)]
struct Stats {
    /// candidates found: block magics, end magics
    block_cands: u64,
    end_cands: u64,
    /// blocks the chain accepted from a worker, and those the joiner read itself
    accepted: u64,
    inline: u64,
    /// block candidates the chain stepped over (never a block's start)
    skipped: u64,
    /// workers' results thrown away, and those of them that were whole blocks with a
    /// matching CRC (valid blocks at places the chain never reached)
    dropped: u64,
    dropped_ok: u64,
    /// results decoded at a level the chain did not have there, read again by the joiner
    redone: u64,
    /// the finder thread's time, seconds
    find_s: f64,
}

#[derive(Default)]
struct FindState {
    tasks: Vec<Task>,
    /// every candidate that starts before this bit is in `tasks`
    found_upto: u64,
    /// the next task for a worker, and the chain's place: tasks before it are done with
    next: usize,
    base: usize,
    /// the level of the stream the chain is in
    level: u32,
    results: BTreeMap<usize, Outcome>,
    free: Vec<Vec<u8>>,
    stop: bool,
    /// a helper thread (the finder or a worker) panicked: nothing it owed will come
    failed: bool,
    stats: Stats,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// What a decode gave: the bytes before its end or its first error, and which
struct Run {
    out: Vec<u8>,
    res: Result<(), Error>,
    stats: Stats,
}

/// The input the finder searches at a time
const FIND_CHUNK: usize = 256 << 10;

// ---- Liveness: every thread's failure reaches every waiter ----
//
// Each role that another waits on (the finder, the scanner, the workers) holds a guard
// that, if the thread unwinds, marks the shared state failed and stopped and wakes every
// waiter; each wait checks that mark. The joiner (the caller's thread) holds a guard that
// stops the helpers and wakes them however it leaves, a panic included. Then the helpers'
// handles are joined, and a helper that panicked makes the decode an error, even where the
// chain had finished: a decode returns, or its panic propagates, and it never waits on a
// thread that has gone.

/// Runs its closure when dropped, on an ordinary exit and on unwinding alike
struct OnDrop<F: FnMut()>(F);

impl<F: FnMut()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        (self.0)()
    }
}

/// A panic injected at one role's boundary, for the fault tests (`faults`); `None` in
/// every decode that is measured or checked
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    None,
    /// a worker, after claiming its first job and before reporting it
    Worker,
    /// the finder, after searching its first chunk and before publishing it
    Finder,
    /// the scanner, after issuing its first block
    Scanner,
    /// the joiner, after accepting its first block
    Joiner,
}

fn inject(fault: Fault, here: Fault) {
    if fault == here && here != Fault::None {
        panic!("par_scan: injected fault: {here:?}");
    }
}

/// The error a decode returns when a helper thread panicked
fn helper_failed(who: &str) -> Error {
    Error {
        kind: ErrorKind::Internal(format!("par_scan: {who} panicked")),
        block: None,
        bit: None,
    }
}

/// `data` decoded by a finder thread and `threads` workers that each read and decode a
/// block, at most `slots` candidates ahead of the chain
fn decode_find(data: &[u8], threads: usize, slots: usize, k: Kernels, f: &Finder) -> Run {
    decode_find_with(data, threads, slots, k, f, Fault::None)
}

fn decode_find_with(
    data: &[u8],
    threads: usize,
    slots: usize,
    k: Kernels,
    f: &Finder,
    fault: Fault,
) -> Run {
    let shared = (Mutex::new(FindState::default()), Condvar::new());
    let (state, cv) = (&shared.0, &shared.1);
    let slots = slots.max(threads);
    let mut out = Vec::with_capacity(data.len().saturating_mul(4));
    // a helper that unwinds: the state failed and stopped, every waiter woken
    let on_panic = move || {
        if std::thread::panicking() {
            let mut st = lock(state);
            st.failed = true;
            st.stop = true;
            drop(st);
            cv.notify_all();
        }
    };
    let res = std::thread::scope(|s| {
        // the joiner, however it leaves: the helpers stopped and woken
        let stop_all = OnDrop(move || {
            lock(state).stop = true;
            cv.notify_all();
        });
        let mut helpers = Vec::with_capacity(threads + 1);
        helpers.push(s.spawn(move || {
            let _failed = OnDrop(on_panic);
            let t0 = Instant::now();
            let (mut v, mut from) = (Vec::new(), 0);
            let mut n = [0u64; 2];
            while from < data.len() {
                let to = (from + FIND_CHUNK).min(data.len());
                v.clear();
                f.find(data, from, to, &mut v);
                inject(fault, Fault::Finder);
                let mut st = lock(state);
                if st.stop {
                    return;
                }
                for c in &v {
                    n[c.end as usize] += 1;
                    if !c.end {
                        // a stream header just before it, at a byte: a guess at the level
                        let at = (c.bit / 8) as usize;
                        let hdr = match (c.bit % 8, data.get(at.wrapping_sub(4)..at)) {
                            (0, Some([b'B', b'Z', b'h', l @ b'1'..=b'9'])) => {
                                Some((l - b'0') as u32)
                            }
                            _ => None,
                        };
                        st.tasks.push(Task { bit: c.bit, hdr });
                    }
                }
                st.found_upto = to as u64 * 8;
                drop(st);
                cv.notify_all();
                from = to;
            }
            let mut st = lock(state);
            st.found_upto = u64::MAX;
            st.stats.block_cands = n[0];
            st.stats.end_cands = n[1];
            st.stats.find_s = t0.elapsed().as_secs_f64();
            drop(st);
            cv.notify_all();
        }));
        for _ in 0..threads {
            helpers.push(s.spawn(move || {
                let _failed = OnDrop(on_panic);
                let mut sc: Scratch = Scratch::default();
                let mut syms = Vec::new();
                loop {
                    let (i, bit, level, mut buf) = {
                        let mut st = lock(state);
                        loop {
                            if st.stop {
                                return;
                            }
                            st.next = st.next.max(st.base);
                            if st.next < st.tasks.len() && st.next < st.base + slots {
                                break;
                            }
                            st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
                        }
                        let i = st.next;
                        st.next += 1;
                        // the level: the last header guessed since the chain's place, else
                        // the chain's own (so a false header misleads only until the chain
                        // has passed it)
                        let level = (st.base..=i)
                            .rev()
                            .find_map(|j| st.tasks[j].hdr)
                            .unwrap_or(st.level);
                        (i, st.tasks[i].bit, level, st.free.pop().unwrap_or_default())
                    };
                    inject(fault, Fault::Worker);
                    buf.clear();
                    let level = if level == 0 { 9 } else { level };
                    let res = match one_item(data, bit, level, k, &mut sc, &mut syms, &mut buf) {
                        Ok(One::Block { end, crc }) => Ok((end, crc, buf)),
                        // (a candidate is a block magic: the item there is never an end)
                        Ok(One::End { .. }) => Err(ErrorKind::Internal("an end".into())),
                        Err(kind) => Err(kind),
                    };
                    let mut st = lock(state);
                    if i >= st.base {
                        st.results.insert(i, Outcome { level, res });
                    } else {
                        drop_result(&mut st, Outcome { level, res });
                    }
                    drop(st);
                    cv.notify_all();
                }
            }));
        }
        let res = chain(data, k, &mut out, state, cv, fault);
        drop(stop_all);
        // (every helper joined here: one that panicked makes the decode an error)
        let panicked = helpers
            .into_iter()
            .fold(false, |p, h| h.join().is_err() | p);
        if panicked {
            Err(helper_failed("the finder or a worker"))
        } else {
            res
        }
    });
    // (results the chain never came to, where it stopped at an error)
    let mut st = lock(state);
    for o in std::mem::take(&mut st.results).into_values() {
        drop_result(&mut st, o);
    }
    let stats = st.stats;
    drop(st);
    Run { out, res, stats }
}

/// A result nothing will take: counted, its buffer kept for reuse
fn drop_result(st: &mut FindState, o: Outcome) {
    st.stats.dropped += 1;
    if let Ok((_, _, buf)) = o.res {
        st.stats.dropped_ok += 1;
        st.free.push(buf);
    }
}

/// The worker's result for the candidate at exactly `at`, decoded at `level`; None where
/// there is none to use (the joiner then reads the item itself)
fn take(
    state: &Mutex<FindState>,
    cv: &Condvar,
    at: u64,
    level: u32,
) -> Result<Option<Outcome>, Error> {
    let mut st = lock(state);
    loop {
        if st.failed {
            return Err(helper_failed("the finder or a worker"));
        }
        // candidates before the chain's place: inside a block it accepted
        let before = st.base;
        while st.base < st.tasks.len() && st.tasks[st.base].bit < at {
            let b = st.base;
            if let Some(o) = st.results.remove(&b) {
                drop_result(&mut st, o);
            }
            st.base += 1;
            st.stats.skipped += 1;
        }
        if st.base != before {
            cv.notify_all();
        }
        let b = st.base;
        if b < st.tasks.len() {
            if st.tasks[b].bit != at {
                return Ok(None); // (the finder missed a magic: it never does)
            }
            if let Some(o) = st.results.remove(&b) {
                st.base += 1;
                cv.notify_all();
                if o.level != level {
                    st.stats.redone += 1;
                    if let Ok((_, _, buf)) = o.res {
                        st.free.push(buf);
                    }
                    return Ok(None);
                }
                st.stats.accepted += 1;
                return Ok(Some(o));
            }
        } else if st.found_upto > at {
            return Ok(None); // (as above)
        }
        st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
    }
}

/// The chain: `stream::Scanner`'s walk of the input (stream headers, items each from the
/// last one's end, each stream's combined CRC), with each block taken from the workers
/// where one read it from the same bit at the same level, and read here otherwise
fn chain(
    data: &[u8],
    k: Kernels,
    out: &mut Vec<u8>,
    state: &Mutex<FindState>,
    cv: &Condvar,
    fault: Fault,
) -> Result<(), Error> {
    let mut sc: Scratch = Scratch::default();
    let mut syms = Vec::new();
    let (mut bit, mut blocks, mut first) = (0u64, 0u64, true);
    loop {
        let rest = data.get((bit / 8) as usize..).unwrap_or(&[]);
        let level = match rest {
            [] if !first => return Ok(()),
            [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => (l - b'0') as u32,
            _ => {
                let kind = match (first, rest.is_empty()) {
                    (true, true) => ErrorKind::Empty,
                    (true, false) => ErrorKind::NotBz2,
                    _ => ErrorKind::TrailingData,
                };
                return Err(Error {
                    kind,
                    block: None,
                    bit: Some(bit),
                });
            }
        };
        bit += 32;
        lock(state).level = level;
        let mut combined = 0u32;
        loop {
            let at = bit;
            let index = blocks;
            let err = move |kind| Error {
                kind,
                block: Some(index),
                bit: Some(at),
            };
            let mut br = Bits::at(data, at);
            let magic = (br.bits(24) as u64) << 24 | br.bits(24) as u64;
            let got = if magic == BLOCK_MAGIC && !br.overran() {
                take(state, cv, at, level)?
            } else {
                None
            };
            let item = match got {
                Some(o) => {
                    let (end, crc, buf) = o.res.map_err(err)?;
                    out.extend_from_slice(&buf);
                    lock(state).free.push(buf);
                    One::Block { end, crc }
                }
                None => {
                    if magic == BLOCK_MAGIC {
                        lock(state).stats.inline += 1;
                    }
                    one_item(data, at, level, k, &mut sc, &mut syms, out).map_err(err)?
                }
            };
            match item {
                One::Block { end, crc } => {
                    combined = combined.rotate_left(1) ^ crc;
                    blocks += 1;
                    bit = end;
                    inject(fault, Fault::Joiner);
                }
                One::End { end, stored } => {
                    if stored != combined {
                        return Err(err(ErrorKind::StreamCrc {
                            stored,
                            computed: combined,
                        }));
                    }
                    bit = end.div_ceil(8) * 8;
                    first = false;
                    break;
                }
            }
        }
    }
}

// ---- The control: the same joiner and workers behind a sequential scanner thread ----

#[derive(Default)]
struct ScanState {
    jobs: VecDeque<(u64, u64, Symbols)>,
    results: BTreeMap<u64, Result<Vec<u8>, Error>>,
    issued: u64,
    taken: u64,
    end: Option<u64>,
    stop: bool,
    /// a helper thread (the scanner or a worker) panicked: nothing it owed will come
    failed: bool,
    free_out: Vec<Vec<u8>>,
    free_syms: Vec<Vec<u16>>,
}

/// `data` decoded as the library's parallel decoder does it (one scanner thread reading
/// every block's symbols in order, `threads` workers finishing a block each, at most
/// `slots` blocks in flight), in this harness's shape: the input a slice, the output
/// joined once. The control arm. Besides who reads the symbols, it differs from
/// `decode_find` in who owns the symbol buffers, in its locking and in having no candidates
/// or speculative work.
fn decode_scan(data: &[u8], threads: usize, slots: usize, k: Kernels) -> Run {
    decode_scan_with(data, threads, slots, k, Fault::None)
}

fn decode_scan_with(data: &[u8], threads: usize, slots: usize, k: Kernels, fault: Fault) -> Run {
    let shared = (Mutex::new(ScanState::default()), Condvar::new());
    let (state, cv) = (&shared.0, &shared.1);
    let slots = slots.max(threads) as u64;
    let mut out = Vec::with_capacity(data.len().saturating_mul(4));
    // (as in decode_find_with)
    let on_panic = move || {
        if std::thread::panicking() {
            let mut st = lock(state);
            st.failed = true;
            st.stop = true;
            drop(st);
            cv.notify_all();
        }
    };
    let res = std::thread::scope(|s| {
        let stop_all = OnDrop(move || {
            lock(state).stop = true;
            cv.notify_all();
        });
        let mut helpers = Vec::with_capacity(threads + 1);
        helpers.push(s.spawn(move || {
            let _failed = OnDrop(on_panic);
            let mut sc = Scanner::new(SliceSource(data));
            loop {
                let syms = {
                    let mut st = lock(state);
                    while !st.stop && st.issued - st.taken >= slots {
                        st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
                    }
                    if st.stop {
                        return;
                    }
                    st.free_syms.pop().unwrap_or_default()
                };
                let next = sc.next(syms);
                let mut st = lock(state);
                match next {
                    Ok(Event::Block {
                        index,
                        bit,
                        symbols,
                        ..
                    }) => {
                        st.jobs.push_back((index, bit, symbols));
                        st.issued += 1;
                        drop(st);
                        cv.notify_all();
                        inject(fault, Fault::Scanner);
                    }
                    other => {
                        if let Err(e) = other {
                            let at = st.issued;
                            st.results.insert(at, Err(e));
                            st.issued += 1;
                        }
                        st.end = Some(st.issued);
                        drop(st);
                        cv.notify_all();
                        return;
                    }
                }
            }
        }));
        for _ in 0..threads {
            helpers.push(s.spawn(move || {
                let _failed = OnDrop(on_panic);
                let mut sc: Scratch = Scratch::default();
                loop {
                    let ((index, bit, symbols), mut buf) = {
                        let mut st = lock(state);
                        loop {
                            if st.stop {
                                return;
                            }
                            if let Some(job) = st.jobs.pop_front() {
                                break (job, st.free_out.pop().unwrap_or_default());
                            }
                            st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
                        }
                    };
                    inject(fault, Fault::Worker);
                    buf.clear();
                    let res = decode_block(&symbols, k, &mut sc, &mut buf)
                        .map(|_| buf)
                        .map_err(|kind| Error {
                            kind,
                            block: Some(index),
                            bit: Some(bit),
                        });
                    let mut st = lock(state);
                    st.results.insert(index, res);
                    st.free_syms.push(symbols.syms);
                    drop(st);
                    cv.notify_all();
                }
            }));
        }
        let mut st = lock(state);
        let res = loop {
            let taken = st.taken;
            if st.failed {
                break Err(helper_failed("the scanner or a worker"));
            }
            if let Some(r) = st.results.remove(&taken) {
                st.taken += 1;
                drop(st);
                cv.notify_all();
                match r {
                    Ok(buf) => {
                        out.extend_from_slice(&buf);
                        inject(fault, Fault::Joiner);
                        st = lock(state);
                        st.free_out.push(buf);
                    }
                    Err(e) => {
                        st = lock(state);
                        break Err(e);
                    }
                }
            } else if st.end == Some(taken) {
                break Ok(());
            } else {
                st = cv.wait(st).unwrap_or_else(|e| e.into_inner());
            }
        };
        drop(st);
        drop(stop_all);
        let panicked = helpers
            .into_iter()
            .fold(false, |p, h| h.join().is_err() | p);
        if panicked {
            Err(helper_failed("the scanner or a worker"))
        } else {
            res
        }
    });
    Run {
        out,
        res,
        stats: Stats::default(),
    }
}

/// The sequential decoder as a reader: the bytes before its end or first error, and which
fn sequential(data: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    let mut out = Vec::new();
    let res = match mibz2::BzReader::new(data).read_to_end(&mut out) {
        Ok(_) => Ok(()),
        Err(e) => Err(e
            .get_ref()
            .and_then(|x| x.downcast_ref::<Error>())
            .cloned()
            .expect("the decoder's own error")),
    };
    (out, res)
}

// ---- Measuring ----

#[repr(C)]
#[derive(Default)]
struct Rusage {
    utime: [i64; 2],
    stime: [i64; 2],
    rest: [i64; 14],
}

extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
}

/// This process's CPU time so far (user and system, every thread), in microseconds. A
/// failed `getrusage` is a panic, never a figure: a cell without its CPU time is no cell.
fn cpu_micros() -> u64 {
    let mut r = Rusage::default();
    // SAFETY: RUSAGE_SELF (0) and a valid, exclusive pointer to a zeroed `Rusage`, which
    // has struct rusage's size and layout on x86_64 and aarch64 Linux (LP64: two
    // timevals of two i64 each, then 14 longs, 144 bytes; checked below at compile time).
    // The kernel writes only within it; nothing is retained after the call.
    let rc = unsafe { getrusage(0, &mut r) };
    if rc != 0 {
        panic!(
            "par_scan: getrusage failed ({}): no CPU time to record",
            std::io::Error::last_os_error()
        );
    }
    let t = |tv: [i64; 2]| {
        assert!(
            tv[0] >= 0 && (0..1_000_000).contains(&tv[1]),
            "par_scan: getrusage gave {tv:?}"
        );
        tv[0] as u64 * 1_000_000 + tv[1] as u64
    };
    t(r.utime) + t(r.stime)
}

#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
const _: () = assert!(std::mem::size_of::<Rusage>() == 144);

/// This process's CPU seconds so far
fn cpu_seconds() -> f64 {
    cpu_micros() as f64 * 1e-6
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split(' ').take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let n = s.len();
    if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

fn compress(data: &[u8], level: u32) -> Vec<u8> {
    use std::io::Write;
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

// ---- finder: its rate, and its candidates against the truth ----

/// A .bz2's true magics by the sequential scanner: each block's first bit, and each
/// stream's end marker (the bit its last block ends at)
fn truth(data: &[u8]) -> Vec<Cand> {
    let mut v = Vec::new();
    let mut last: Option<(u64, u64)> = None;
    for span in mibz2::scan(data) {
        let span = span.expect("a whole .bz2");
        if let Some((stream, end)) = last {
            if stream != span.stream {
                v.push(Cand {
                    bit: end,
                    end: true,
                });
            }
        }
        v.push(Cand {
            bit: span.start_bit,
            end: false,
        });
        last = Some((span.stream, span.end_bit));
    }
    if let Some((_, end)) = last {
        v.push(Cand {
            bit: end,
            end: true,
        });
    }
    v
}

fn cmd_finder(data: &[u8], decoded: usize) {
    let f = Finder::detect();
    let mut found = Vec::new();
    f.find(data, 0, data.len(), &mut found);
    let scalar = Finder::new(false);
    let mut found_scalar = Vec::new();
    scalar.find(data, 0, data.len(), &mut found_scalar);
    assert_eq!(found, found_scalar, "the scalar finder's candidates");
    assert_eq!(
        found,
        find_naive(data),
        "the bit-by-bit finder's candidates"
    );
    // cut into ranges anywhere, each read with its overlap: the same candidates
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..20 {
        let (mut v, mut from) = (Vec::new(), 0);
        while from < data.len() {
            let to = (from + 1 + rng.below(300_000) as usize).min(data.len());
            f.find(data, from, to, &mut v);
            from = to;
        }
        assert_eq!(found, v, "ranges cut at random");
    }
    let t = truth(data);
    let missed = t.iter().filter(|c| !found.contains(c)).count();
    let false_pos = found.iter().filter(|c| !t.contains(c)).count();
    let bits = data.len() as f64 * 8.0;
    println!(
        "finder ({}): {} block magics and {} end magics found in {:.1} Mbit; truth (mibz2::scan): \
         {} blocks, {} stream ends; missed {missed}, false positives {false_pos}",
        if f.avx2 { "AVX2" } else { "scalar" },
        found.iter().filter(|c| !c.end).count(),
        found.iter().filter(|c| c.end).count(),
        bits / 1e6,
        t.iter().filter(|c| !c.end).count(),
        t.iter().filter(|c| c.end).count(),
    );
    println!(
        "  model: uniform random bits match one of 2 magics at a given bit with p = 2^-47; \
         expected false positives here {:.1e}, in a 103.5 GB dump {:.1e}",
        bits / 2f64.powi(47),
        103_488_697_311f64 * 8.0 / 2f64.powi(47),
    );
    let ratio = decoded as f64 / data.len() as f64;
    let rate = |name: &str, f: &Finder, threads: usize| {
        // passes over the file for about a second, each pass timed
        let mut times = Vec::new();
        let t0 = Instant::now();
        while t0.elapsed().as_secs_f64() < 1.0 || times.len() < 20 {
            let t = Instant::now();
            let n: usize = std::thread::scope(|s| {
                let per = data.len().div_ceil(threads);
                let hs: Vec<_> = (0..threads)
                    .map(|i| {
                        s.spawn(move || {
                            let mut v = Vec::new();
                            let to = ((i + 1) * per).min(data.len());
                            f.find(data, i * per, to, &mut v);
                            v.len()
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).sum()
            });
            times.push(t.elapsed().as_secs_f64());
            assert_eq!(n, found.len());
        }
        let gbs = data.len() as f64 / median(&times) / 1e9;
        let best = data.len() as f64 / times.iter().cloned().fold(f64::INFINITY, f64::min) / 1e9;
        println!(
            "  {name:22} {gbs:6.2} GB/s of compressed input (median of {} passes; best {best:.2}) \
             = {:6.1} GB/s of decoded output at {ratio:.2}x",
            times.len(),
            gbs * ratio,
        );
        gbs
    };
    rate("AVX2 filter, 1 thread", &f, 1);
    rate("AVX2 filter, 2 threads", &f, 2);
    rate("AVX2 filter, 4 threads", &f, 4);
    rate("scalar, 1 thread", &scalar, 1);
    // the sequential scanner on the same input, for scale
    let mut times = Vec::new();
    for _ in 0..5 {
        let t = Instant::now();
        let n = mibz2::scan(data).count();
        times.push(t.elapsed().as_secs_f64());
        assert!(n > 0);
    }
    let gbs = data.len() as f64 / median(&times) / 1e9;
    println!(
        "  {:22} {gbs:6.3} GB/s of compressed input (median of 5 passes) = {:6.2} GB/s of decoded output",
        "mibz2::scan, 1 thread",
        gbs * ratio
    );
}

// ---- check: the harness against the decoders ----

/// A field of /proc/self/status (or of a thread's), as written there
fn status_field(path: &str, key: &str) -> String {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix(key).map(|v| v.trim().to_string()))
        })
        .unwrap_or_else(|| "?".into())
}

/// The CPUs each of this process's threads may run on (sched_getaffinity, as the kernel
/// reports it in /proc/self/task/*/status), and how many threads have each set
fn thread_masks() -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    if let Ok(dir) = std::fs::read_dir("/proc/self/task") {
        for e in dir.flatten() {
            let p = e.path().join("status");
            *m.entry(status_field(&p.to_string_lossy(), "Cpus_allowed_list:"))
                .or_insert(0) += 1;
        }
    }
    m
}

/// Where the process may run: its CPU mask, the CPUs Rust sees, its cgroup's CPU quota.
/// The harness never sets an affinity: every thread inherits this mask.
fn placement() -> String {
    format!(
        "process Cpus_allowed_list {}; available_parallelism {}; cgroup cpu.max {}",
        status_field("/proc/self/status", "Cpus_allowed_list:"),
        std::thread::available_parallelism().map_or(0, |n| n.get()),
        std::fs::read_to_string("/sys/fs/cgroup/cpu.max")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "?".into())
    )
}

/// This process's peak resident memory so far (VmHWM)
fn peak_rss() -> String {
    status_field("/proc/self/status", "VmHWM:")
}

/// `run` with a thread beside it sampling every thread's CPU mask until it returns: the
/// most threads seen at once, and each mask with the most threads seen holding it. The
/// sampler stops however `run` ends: a panic in it propagates, and never leaves the scope
/// waiting on the sampler.
fn sampled<T: Send>(run: impl FnOnce() -> T + Send) -> (T, usize, BTreeMap<String, usize>) {
    let done = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|s| {
        // (set on every exit, an unwinding `run` included)
        let _stop = OnDrop(|| done.store(true, std::sync::atomic::Ordering::Relaxed));
        let sampler = s.spawn(|| {
            let (mut most, mut masks) = (0, BTreeMap::new());
            while !done.load(std::sync::atomic::Ordering::Relaxed) {
                let m = thread_masks();
                most = most.max(m.values().sum::<usize>());
                for (k, n) in m {
                    let e = masks.entry(k).or_insert(0);
                    *e = n.max(*e);
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            (most, masks)
        });
        let v = run();
        done.store(true, std::sync::atomic::Ordering::Relaxed);
        let (most, masks) = sampler.join().unwrap();
        (v, most, masks)
    })
}

/// libbzip2's decode of `data` (every stream): a second decoder's bytes, by its own code
fn libbzip2_decode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    bzip2::read::MultiBzDecoder::new(data)
        .read_to_end(&mut out)
        .expect("libbzip2 decodes the input");
    out
}

fn cmd_check(data: &[u8], want: &[u8]) {
    let (k, f) = (Kernels::detect(), Finder::detect());
    // the reference itself against a second decoder: mibz2::decode's bytes are libbzip2's
    assert!(
        libbzip2_decode(data) == want,
        "mibz2::decode's bytes differ from libbzip2's"
    );
    println!(
        "check: mibz2::decode's bytes equal libbzip2's ({} bytes)",
        want.len()
    );
    println!("check: placement: {}", placement());
    for threads in 1..=4 {
        let r = decode_find(data, threads, 4 * threads, k, &f);
        assert_eq!(r.res, Ok(()), "find, {threads} workers");
        assert!(r.out == want, "find, {threads} workers: the output differs");
        let s = decode_scan(data, threads, 4 * threads, k);
        assert_eq!(s.res, Ok(()));
        assert!(s.out == want, "scan, {threads} workers: the output differs");
        println!(
            "check: {threads} workers: find and scan harness outputs equal mibz2::decode's \
             ({} bytes); find: {:?}",
            want.len(),
            r.stats
        );
    }
    // scalar finder and scalar kernels, and a window of one
    let r = decode_find(data, 3, 1, Kernels::scalar(), &Finder::new(false));
    assert!(r.res == Ok(()) && r.out == want);
    println!("check: scalar finder, scalar kernels, 3 workers: equal");
    // the grid's largest count: equal, and where its threads may run. Each arm's decode
    // is t workers, one finder or scanner thread and the caller's thread (the joiner),
    // so 10 workers are 12 threads of the arm, sharing the machine's CPUs with each
    // other (and here with the sampler)
    {
        let t = 10;
        let (r, most, masks) = sampled(|| decode_find(data, t, 4 * t, k, &f));
        assert!(r.res == Ok(()) && r.out == want, "find, {t} workers");
        println!(
            "check: {t} workers: find equal; placement: at most {most} threads at once \
             (the sampler's included); their CPU masks {masks:?}"
        );
        let (r, most, masks) = sampled(|| decode_scan(data, t, 4 * t, k));
        assert!(r.res == Ok(()) && r.out == want, "scan, {t} workers");
        println!(
            "check: {t} workers: scan equal; placement: at most {most} threads at once \
             (the sampler's included); their CPU masks {masks:?}"
        );
    }
    println!("check: peak resident memory {}", peak_rss());
}

// ---- faults: a panic at each role's boundary ends the decode, never hangs it ----

/// A small multi-stream input of many blocks, made here: 60 lines of text a stream, ten
/// streams at level 1
fn fault_input() -> Vec<u8> {
    let mut rng = Rng(0x2468_ace0_1357_9bdf);
    let words = [
        "scan", "find", "block", "magic", "chain", "worker", "joiner", "bit",
    ];
    let mut data = Vec::new();
    for _ in 0..10 {
        let mut text = Vec::new();
        while text.len() < 250_000 {
            text.extend_from_slice(words[rng.below(8) as usize].as_bytes());
            text.extend_from_slice(&rng.next().to_le_bytes()[..rng.below(4) as usize]);
            text.push(b' ');
        }
        data.extend_from_slice(&compress(&text, 1));
    }
    data
}

/// How a faulted decode ended
#[derive(Debug, PartialEq, Eq)]
enum Ended {
    /// it returned this error
    Error(String),
    /// its panic reached the caller
    Panic(String),
    /// it returned Ok: the fault went unnoticed
    Ok,
}

/// Each arm with a panic injected at each of its roles, at 1 and 4 workers, and the
/// joiner's panic again through `sampled` (the check's wrapper): each must end within
/// `deadline` seconds, with an error (a helper's panic) or a panic (the joiner's), never
/// Ok and never a hang. Untimed: the deadline only bounds the wait. Returns the lines it
/// reports, or the first failure.
fn run_faults(deadline: u64) -> Result<Vec<String>, String> {
    let data = std::sync::Arc::new(fault_input());
    let want = mibz2::decode(&data).unwrap();
    let mut lines = vec![format!(
        "faults: input {} bytes -> {} bytes, {} blocks",
        data.len(),
        want.len(),
        mibz2::scan(&data).count()
    )];
    let cases = [
        ("find", Fault::Worker),
        ("find", Fault::Finder),
        ("find", Fault::Joiner),
        ("scan", Fault::Worker),
        ("scan", Fault::Scanner),
        ("scan", Fault::Joiner),
        ("find, sampled", Fault::Joiner),
        ("scan, sampled", Fault::Joiner),
    ];
    for t in [1, 4] {
        for (arm, fault) in cases {
            let (tx, rx) = std::sync::mpsc::channel();
            let d = data.clone();
            // (detached: a hang is reported, and the process then exits)
            std::thread::spawn(move || {
                let r = std::panic::catch_unwind(|| {
                    let (k, f) = (Kernels::detect(), Finder::detect());
                    match arm {
                        "find" => decode_find_with(&d, t, 4 * t, k, &f, fault).res,
                        "scan" => decode_scan_with(&d, t, 4 * t, k, fault).res,
                        "find, sampled" => {
                            sampled(|| decode_find_with(&d, t, 4 * t, k, &f, fault).res).0
                        }
                        _ => sampled(|| decode_scan_with(&d, t, 4 * t, k, fault).res).0,
                    }
                });
                let ended = match r {
                    Ok(Ok(())) => Ended::Ok,
                    Ok(Err(e)) => Ended::Error(e.to_string()),
                    Err(p) => Ended::Panic(
                        p.downcast_ref::<String>()
                            .cloned()
                            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_default(),
                    ),
                };
                let _ = tx.send(ended);
            });
            let case = format!("{arm}, {t} workers, a panic in the {fault:?}");
            match rx.recv_timeout(std::time::Duration::from_secs(deadline)) {
                Err(_) => return Err(format!("faults: {case}: HANG (no end in {deadline} s)")),
                Ok(Ended::Ok) => return Err(format!("faults: {case}: returned Ok")),
                Ok(Ended::Error(e)) if fault != Fault::Joiner && e.contains("panicked") => {
                    lines.push(format!("faults: {case}: ended with an error: {e}"))
                }
                Ok(Ended::Panic(m)) if fault == Fault::Joiner && m.contains("injected") => {
                    lines.push(format!("faults: {case}: ended, its panic propagated: {m}"))
                }
                Ok(other) => return Err(format!("faults: {case}: ended unexpectedly: {other:?}")),
            }
        }
    }
    Ok(lines)
}

fn cmd_faults() {
    // (the injected panics' own messages are expected: kept off stderr)
    std::panic::set_hook(Box::new(|i| {
        if !i.to_string().contains("injected fault") {
            eprintln!("{i}");
        }
    }));
    match run_faults(120) {
        Ok(lines) => {
            for l in lines {
                println!("{l}");
            }
            println!("faults: every case ended, none hung");
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

// ---- adversarial inputs ----

/// The bytes whose in-use bitmap, read as 16-bit words from range `first` (16 byte values
/// a range), is `words`: a block that uses exactly these bytes (and any outside the
/// ranges) stores the words back to back in its header
fn bytes_for_map(first: usize, words: &[u16]) -> Vec<u8> {
    let mut v = Vec::new();
    for (i, w) in words.iter().enumerate() {
        assert!(*w != 0, "a range with no byte in use stores no word");
        for j in 0..16 {
            if w & (0x8000 >> j) != 0 {
                v.push(((first + i) * 16 + j) as u8);
            }
        }
    }
    v
}

/// `n` bytes drawn from `set`, low indexes likelier, every byte of the set in each 2,000,
/// and never 4 equal bytes in a row (bzip2 writes such a run's length as a byte of its
/// own, which would then be in use too)
fn drawn(rng: &mut Rng, set: &[u8], n: usize) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(n);
    for i in 0..n {
        let mut b = if i % 2000 < set.len() {
            set[i % 2000]
        } else {
            let (x, y) = (rng.below(set.len() as u64), rng.below(set.len() as u64));
            set[x.min(y) as usize]
        };
        if v.ends_with(&[b, b, b]) {
            b = set[(set.iter().position(|&x| x == b).unwrap() + 1) % set.len()];
        }
        v.push(b);
    }
    v
}

/// A valid .bz2 (libbzip2's own output) every block of which holds, in its header's
/// in-use bitmap: "BZh9" and the block magic, back to back, then the end magic
fn magic_in_map(rng: &mut Rng, n: usize, level: u32) -> Vec<u8> {
    let words = [
        0x425a, 0x6839, 0x3141, 0x5926, 0x5359, 0x1772, 0x4538, 0x5090,
    ];
    compress(&drawn(rng, &bytes_for_map(0, &words), n), level)
}

/// The harness on `data` (3 workers, a window of 12) against the sequential decoder: the
/// same bytes before the end or the error, and the same result
fn same_as_sequential(name: &str, data: &[u8], k: Kernels, f: &Finder) -> Stats {
    let (want, want_res) = sequential(data);
    let r = decode_find(data, 3, 12, k, f);
    let cands = find_naive(data);
    println!(
        "  {name}: {} bytes; magics found: {} block, {} end",
        data.len(),
        cands.iter().filter(|c| !c.end).count(),
        cands.iter().filter(|c| c.end).count()
    );
    println!(
        "    sequential: {} bytes, then {}",
        want.len(),
        match &want_res {
            Ok(()) => "the end".to_string(),
            Err(e) => format!("{e}"),
        }
    );
    println!(
        "    harness:    {} bytes, then {}; {:?}",
        r.out.len(),
        match &r.res {
            Ok(()) => "the end".to_string(),
            Err(e) => format!("{e}"),
        },
        r.stats
    );
    assert!(r.out == want, "{name}: the bytes differ");
    assert_eq!(r.res, want_res, "{name}: the result differs");
    assert_eq!(
        mibz2::decode(data).map(|_| ()),
        want_res,
        "{name}: decode's result"
    );
    r.stats
}

fn cmd_adversarial(text: &[u8]) {
    let (k, f) = (Kernels::detect(), Finder::detect());
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    println!("adversarial:");
    // 1. a valid file with both magics (and a false stream header) inside every block
    let a = magic_in_map(&mut rng, 3_000_000, 1);
    let blocks = mibz2::scan(&a).count() as u64;
    let s = same_as_sequential("1. magics in the in-use bitmap (valid, level 1)", &a, k, &f);
    assert_eq!(s.block_cands, 2 * blocks, "a false block magic a block");
    assert_eq!(s.accepted + s.inline, blocks);
    // (the last block's is past the last block the chain takes)
    assert_eq!(s.skipped, blocks - 1, "each stepped over");
    assert!(s.redone >= 1, "a false stream header misled a worker");
    // 2. a whole valid block (and a stream end) spliced over a block's payload
    let host = compress(&text[..1_500_000.min(text.len())], 3);
    let spans: Vec<_> = mibz2::scan(&host).map(|s| s.unwrap()).collect();
    let tiny = compress(b"a valid block where no block is", 9);
    let mut b = host.clone();
    let mid = ((spans[1].start_bit + spans[1].end_bit) / 16) as usize;
    b[mid..mid + tiny.len() - 4].copy_from_slice(&tiny[4..]);
    // (what a worker reads at the spliced magic: a whole block, its CRC matching)
    let mut part = Vec::new();
    let read = one_item(
        &b,
        mid as u64 * 8,
        9,
        k,
        &mut Scratch::default(),
        &mut Vec::new(),
        &mut part,
    );
    assert!(matches!(read, Ok(One::Block { .. })) && part == b"a valid block where no block is");
    println!(
        "  (2: the item at the spliced magic, bit {}, read alone: a block of {} bytes, its CRC matching)",
        mid * 8,
        part.len()
    );
    same_as_sequential("2. a valid block spliced into block 1's payload", &b, k, &f);
    // 3. bytes after a stream that begin no stream, then a whole stream
    let mut c = compress(&text[..200_000.min(text.len())], 9);
    c.extend_from_slice(b"junk");
    c.extend_from_slice(&host);
    same_as_sequential("3. a stream, 4 stray bytes, a stream", &c, k, &f);
    // 4. a randomised block: the bit after a block's CRC set
    let mut d = host.clone();
    let at = spans[2].start_bit + 80;
    d[(at / 8) as usize] |= 0x80 >> (at % 8);
    same_as_sequential("4. block 2 marked randomised", &d, k, &f);
    // 5. cut inside a block, and inside the end marker's CRC
    let cut = (spans[3].start_bit / 8 + 1000) as usize;
    same_as_sequential("5a. cut inside block 3", &host[..cut], k, &f);
    same_as_sequential(
        "5b. cut inside the stream CRC",
        &host[..host.len() - 2],
        k,
        &f,
    );
    // 6. two blocks exchanged: every block valid, the stream's combined CRC not
    let (s1, s2) = (&spans[1], &spans[2]);
    let mut w = BitWriter::default();
    let copy = |w: &mut BitWriter, from: u64, to: u64| {
        let mut br = Bits::at(&host, from);
        for _ in from..to {
            w.put(br.bits(1) as u64);
        }
    };
    copy(&mut w, 0, s1.start_bit);
    copy(&mut w, s2.start_bit, s2.end_bit);
    copy(&mut w, s1.start_bit, s1.end_bit);
    copy(&mut w, s2.end_bit, host.len() as u64 * 8);
    same_as_sequential("6. blocks 1 and 2 exchanged", &w.out, k, &f);
}

/// MSB-first bits
#[derive(Default)]
struct BitWriter {
    out: Vec<u8>,
    n: u64,
}

impl BitWriter {
    fn put(&mut self, bit: u64) {
        if self.n % 8 == 0 {
            self.out.push(0);
        }
        if bit & 1 == 1 {
            *self.out.last_mut().unwrap() |= 0x80 >> (self.n % 8);
        }
        self.n += 1;
    }
}

// ---- fuzz: mutated files, the harness against the sequential decoder ----

fn cmd_fuzz(text: &[u8], cases: usize) {
    let (k, f) = (Kernels::detect(), Finder::detect());
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    // a small file of every shape the chain meets: several streams at different levels,
    // several blocks, an empty stream, false magics in valid blocks
    let mut base = compress(&text[..260_000.min(text.len())], 1);
    base.extend_from_slice(&compress(b"", 9));
    // (one whose false "BZh9" falls at a byte in a block that is not its last: a false
    // stream header there, with blocks after it to mislead)
    let false_header = |d: &[u8]| {
        let starts: Vec<u64> = mibz2::scan(d).map(|s| s.unwrap().start_bit).collect();
        starts[..starts.len() - 1].iter().any(|b| b % 8 == 7)
    };
    let map = loop {
        let m = magic_in_map(&mut rng, 600_000, 1);
        if false_header(&m) {
            break m;
        }
    };
    base.extend_from_slice(&map);
    base.extend_from_slice(&compress(&text[..60_000.min(text.len())], 9));
    let tiny = compress(b"a valid block where no block is", 9);
    let magics = [BLOCK_MAGIC.to_be_bytes(), END_MAGIC.to_be_bytes()];
    let spans: Vec<_> = mibz2::scan(&base).map(|s| s.unwrap()).collect();
    let (want, res) = sequential(&base);
    assert!(res.is_ok());
    let whole = decode_find(&base, 3, 12, k, &f);
    assert!(whole.res.is_ok() && whole.out == want);
    println!("fuzz: the file whole, 3 workers: {:?}", whole.stats);
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let (mut dropped_ok, mut skipped, mut redone, mut inline) = (0, 0, 0, 0);
    for case in 0..cases {
        let mut d = base.clone();
        for _ in 0..1 + rng.below(2) {
            if d.is_empty() {
                break;
            }
            let at = rng.below(d.len() as u64) as usize;
            match rng.below(10) {
                0 | 1 => d[at] ^= 1 << rng.below(8),
                2 => d.truncate(at),
                // a magic written over the input, or put into it
                3 => {
                    let m = &magics[rng.below(2) as usize][2..];
                    let n = m.len().min(d.len() - at);
                    d[at..at + n].copy_from_slice(&m[..n]);
                }
                4 => {
                    let m = magics[rng.below(2) as usize];
                    d.splice(at..at, m[2..].iter().copied());
                }
                // a whole valid block and stream end written over it
                5 => {
                    let n = (tiny.len() - 4).min(d.len() - at);
                    d[at..at + n].copy_from_slice(&tiny[4..4 + n]);
                }
                // bytes removed, or a range repeated (a block's whole bytes, sometimes)
                6 => {
                    let n = (1 + rng.below(3000) as usize).min(d.len() - at);
                    d.drain(at..at + n);
                }
                7 => {
                    let s = &spans[rng.below(spans.len() as u64) as usize];
                    let (a, b) = ((s.start_bit / 8) as usize, (s.end_bit / 8) as usize);
                    let (a, b) = (a.min(d.len()), b.min(d.len()));
                    let part = d[a..b].to_vec();
                    d.splice(a..a, part);
                }
                // stray bytes, or a whole stream, after the end or between streams
                8 => {
                    let n = 1 + rng.below(8) as usize;
                    let junk: Vec<u8> = (0..n).map(|_| rng.next() as u8).collect();
                    if rng.below(2) == 0 {
                        d.extend_from_slice(&junk);
                    } else {
                        let s = &spans[rng.below(spans.len() as u64) as usize];
                        let a = ((s.start_bit / 8) as usize).min(d.len());
                        d.splice(a..a, junk);
                    }
                }
                // a stream header's level changed
                _ => {
                    let hs: Vec<usize> = (0..d.len().saturating_sub(4))
                        .filter(|&i| &d[i..i + 3] == b"BZh")
                        .collect();
                    if !hs.is_empty() {
                        let i = hs[rng.below(hs.len() as u64) as usize];
                        d[i + 3] = b'0' + rng.below(11) as u8;
                    }
                }
            }
        }
        let (want_out, want_res) = sequential(&d);
        let threads = 1 + rng.below(4) as usize;
        let slots = 1 + rng.below(12) as usize;
        let r = decode_find(&d, threads, slots, k, &f);
        assert!(r.out == want_out, "case {case}: the bytes differ");
        assert_eq!(r.res, want_res, "case {case}: the result differs");
        let whole = mibz2::decode(&d);
        assert_eq!(whole.as_ref().map(|_| ()).map_err(|e| e.clone()), want_res);
        if let Ok(o) = whole {
            assert!(o == want_out);
        }
        let name = match &want_res {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("{:?}", e.kind)
                .split([' ', '{', '('])
                .next()
                .unwrap()
                .to_string(),
        };
        *kinds.entry(name).or_default() += 1;
        dropped_ok += r.stats.dropped_ok;
        skipped += r.stats.skipped;
        redone += r.stats.redone;
        inline += r.stats.inline;
    }
    println!(
        "fuzz: {cases} mutated files ({} bytes -> {} bytes whole; 1-4 workers, windows 1-12): \
         the harness's bytes and result (Error {{ kind, block, bit }}) equal the sequential \
         decoder's in every case",
        base.len(),
        want.len()
    );
    println!("  results: {kinds:?}");
    println!(
        "  over the cases: {skipped} candidates stepped over, {dropped_ok} valid-CRC blocks \
         decoded at places the chain never reached (none emitted), {redone} read again for a \
         wrong level guess, {inline} read by the joiner"
    );
}

// ---- time ----

type Arm<'a> = (&'a str, Box<dyn Fn(usize) -> Vec<u8> + 'a>);

/// A timed run: seconds, this process's CPU seconds, and the cores other processes kept
/// busy meanwhile
#[derive(Clone, Copy)]
struct Timed {
    wall: f64,
    cpu: f64,
    others: f64,
}

/// A run beside which other processes kept at most this many cores busy counts as quiet
const QUIET: f64 = 0.25;

/// The machine's busy CPU seconds since boot (/proc/stat: every state but idle and iowait)
fn machine_busy() -> f64 {
    let stat = std::fs::read_to_string("/proc/stat").unwrap_or_default();
    let ticks: Vec<f64> = stat
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .take(8)
        .map(|x| x.parse().unwrap_or(0.0))
        .collect();
    // (user nice system idle iowait irq softirq steal, in 10 ms ticks)
    ticks
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 3 && *i != 4)
        .map(|x| x.1)
        .sum::<f64>()
        / 100.0
}

/// `f` timed, with the cores the rest of the machine kept busy while it ran
fn timed<T>(f: impl FnOnce() -> T) -> (T, Timed) {
    let (b0, c0, t0) = (machine_busy(), cpu_seconds(), Instant::now());
    let v = f();
    let wall = t0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    let others = ((machine_busy() - b0 - cpu) / wall).max(0.0);
    (v, Timed { wall, cpu, others })
}

/// Wait, for up to `patience` seconds, for the rest of the machine to be quiet
fn wait_quiet(patience: f64) {
    let t0 = Instant::now();
    while t0.elapsed().as_secs_f64() < patience {
        let ((), t) = timed(|| std::thread::sleep(std::time::Duration::from_millis(200)));
        if t.others <= QUIET {
            return;
        }
    }
}

type Runs = BTreeMap<(usize, usize), Vec<Timed>>;

/// Each arm at each thread count, `rounds` rounds, the arms' order rotated a round, each
/// run started once the rest of the machine is quiet (or after 2 seconds of waiting)
fn time_arms(arms: &[Arm], threads: &[usize], rounds: usize, want: &[u8]) -> Runs {
    let mut runs = Runs::new();
    for round in 0..rounds {
        let before = loadavg();
        for &t in threads {
            for i in 0..arms.len() {
                let a = (i + round) % arms.len();
                wait_quiet(2.0);
                let (out, run) = timed(|| arms[a].1(t));
                assert!(out == want, "{}: the output differs", arms[a].0);
                drop(out);
                runs.entry((t, a)).or_default().push(run);
            }
        }
        eprintln!("  round {round}: loadavg {before} -> {}", loadavg());
    }
    runs
}

/// The table: each arm's medians over its quiet runs (all of them, marked `*`, where
/// fewer than 3 were quiet), and its wall time against arms 0 and 1 as the median of the
/// per-round ratios, over the rounds in which both runs were quiet
fn report(arms: &[Arm], threads: &[usize], runs: &Runs, bytes: usize) {
    let mb = bytes as f64 / 1e6;
    println!(
        "| threads | arm | quiet runs | MB/s median | MB/s min-max | CPU s median | wall / {}'s | wall / {}'s |",
        arms[0].0.split(':').next().unwrap(),
        arms[1.min(arms.len() - 1)].0.split(':').next().unwrap()
    );
    println!("|--:|---|--:|--:|--:|--:|--:|--:|");
    for &t in threads {
        for (a, arm) in arms.iter().enumerate() {
            let all = &runs[&(t, a)];
            let quiet: Vec<Timed> = all.iter().copied().filter(|x| x.others <= QUIET).collect();
            let (used, mark) = if quiet.len() >= 3 {
                (&quiet, "")
            } else {
                (all, "*")
            };
            let walls: Vec<f64> = used.iter().map(|x| x.wall).collect();
            let cpus: Vec<f64> = used.iter().map(|x| x.cpu).collect();
            let paired = |b: usize| {
                let o = &runs[&(t, b)];
                let r: Vec<f64> = all
                    .iter()
                    .zip(o)
                    .filter(|(x, y)| x.others <= QUIET && y.others <= QUIET)
                    .map(|(x, y)| x.wall / y.wall)
                    .collect();
                if r.len() >= 3 {
                    format!("{:.3} ({})", median(&r), r.len())
                } else {
                    "-".to_string()
                }
            };
            let (lo, hi) = walls
                .iter()
                .fold((f64::INFINITY, 0f64), |(lo, hi), &w| (lo.min(w), hi.max(w)));
            println!(
                "| {t} | {} | {}/{} | {:.0}{mark} | {:.0}-{:.0} | {:.3} | {} | {} |",
                arm.0,
                quiet.len(),
                all.len(),
                mb / median(&walls),
                mb / hi,
                mb / lo,
                median(&cpus),
                paired(0),
                paired(1.min(arms.len() - 1)),
            );
        }
    }
}

fn cmd_time(data: &[u8], want: &[u8], rounds: usize) {
    let (k, f) = (Kernels::detect(), Finder::detect());
    let arms: Vec<Arm> = vec![
        (
            "lib: decode_parallel(t)",
            Box::new(|t| mibz2::decode_parallel(data, t).unwrap()),
        ),
        (
            "scan: harness, scanner thread + t workers",
            Box::new(|t| decode_scan(data, t, 4 * t, k).out),
        ),
        (
            "find: harness, finder thread + t workers",
            Box::new(|t| decode_find(data, t, 4 * t, k, &f).out),
        ),
        (
            "find+1: finder thread + t+1 workers",
            Box::new(|t| decode_find(data, t + 1, 4 * (t + 1), k, &f).out),
        ),
    ];
    let threads = [1, 2, 3, 4];
    println!(
        "time: {rounds} rounds, arms alternating (order rotated each round), {} MB decoded; \
         loadavg before {}",
        want.len() / 1_000_000,
        loadavg()
    );
    let runs = time_arms(&arms, &threads, rounds, want);
    println!("loadavg after {}", loadavg());
    report(&arms, &threads, &runs, want.len());
    // one thread, no helper threads: the sequential decoder, for the scale
    let mut seq = Vec::new();
    for _ in 0..rounds {
        wait_quiet(2.0);
        let (out, run) = timed(|| mibz2::decode(data).unwrap());
        assert!(out == want);
        seq.push(run);
    }
    let quiet: Vec<Timed> = seq.iter().copied().filter(|x| x.others <= QUIET).collect();
    let used = if quiet.len() >= 3 { &quiet } else { &seq };
    println!(
        "mibz2::decode (one thread, no helper threads): {:.0} MB/s, {:.3} CPU s (median of {} runs, {} quiet)",
        want.len() as f64 / 1e6 / median(&used.iter().map(|x| x.wall).collect::<Vec<_>>()),
        median(&used.iter().map(|x| x.cpu).collect::<Vec<_>>()),
        used.len(),
        quiet.len()
    );
}

// ---- shapes: the worst cases ----

fn cmd_shapes(text: &[u8], rounds: usize) {
    let (k, f) = (Kernels::detect(), Finder::detect());
    let mut rng = Rng(0x1357_9bdf_0246_8ace);
    let random: Vec<u8> = (0..24_000_000).map(|_| rng.next() as u8).collect();
    let mut tiny = Vec::new();
    for i in 0..20_000 {
        tiny.extend_from_slice(&compress(format!("stream {i}\n").as_bytes(), 1));
    }
    let shapes: Vec<(&str, Vec<u8>)> = vec![
        (
            "incompressible (24 MB random, level 9)",
            compress(&random, 9),
        ),
        ("tiny streams (20,000 of a line each)", tiny),
        (
            "one block (800 kB of text, level 9)",
            compress(&text[..800_000.min(text.len())], 9),
        ),
        (
            "false magics in every block (30 MB, level 9)",
            magic_in_map(&mut rng, 30_000_000, 9),
        ),
    ];
    for (name, data) in &shapes {
        let want = mibz2::decode(data).unwrap();
        let blocks = mibz2::scan(data).count();
        let arms: Vec<Arm> = vec![
            (
                "lib: decode_parallel(4)",
                Box::new(|t| mibz2::decode_parallel(data, t).unwrap()),
            ),
            (
                "scan: scanner thread + 4 workers",
                Box::new(|t| decode_scan(data, t, 4 * t, k).out),
            ),
            (
                "find: finder thread + 4 workers",
                Box::new(|t| decode_find(data, t, 4 * t, k, &f).out),
            ),
            (
                "seq: mibz2::decode",
                Box::new(|_| mibz2::decode(data).unwrap()),
            ),
        ];
        let stats = decode_find(data, 4, 16, k, &f).stats;
        println!(
            "shape: {name}: {} bytes -> {} bytes, {blocks} blocks; find: {stats:?}",
            data.len(),
            want.len()
        );
        let runs = time_arms(&arms, &[4], rounds, &want);
        report(&arms, &[4], &runs, want.len());
    }
}

// ---- knee: the two arms over a grid of worker counts, a JSON line a cell ----

/// The find and scan arms at each of `workers`, `rounds` rounds: a fixed schedule, every
/// cell once, which arm goes first alternating by round and by place in the grid. Each
/// cell's output is checked against `want` (a difference is a panic), outside its timing.
/// Stdout holds the cells' JSON lines and nothing else.
fn cmd_knee(data: &[u8], want: &[u8], rounds: usize, workers: &[usize]) {
    let (k, f) = (Kernels::detect(), Finder::detect());
    for round in 0..rounds {
        let before = loadavg();
        for (place, &t) in workers.iter().enumerate() {
            for i in 0..2 {
                let find = (round + place + i) % 2 == 1;
                let (c0, t0) = (cpu_micros(), Instant::now());
                let run = if find {
                    decode_find(data, t, 4 * t, k, &f)
                } else {
                    decode_scan(data, t, 4 * t, k)
                };
                let (wall_ns, cpu_ns) = (t0.elapsed().as_nanos(), (cpu_micros() - c0) * 1000);
                let arm = if find { "find" } else { "scan" };
                assert_eq!(run.res, Ok(()), "{arm}, {t} workers, round {round}");
                assert!(
                    run.out == want,
                    "{arm}, {t} workers, round {round}: the output differs from decode's"
                );
                println!(
                    "{{\"round\":{round},\"arm\":\"{arm}\",\"workers\":{t},\"wall_ns\":{wall_ns},\"cpu_ns\":{cpu_ns},\"bytes\":{}}}",
                    run.out.len()
                );
            }
        }
        eprintln!("  round {round}: loadavg {before} -> {}", loadavg());
    }
    eprintln!("knee: peak resident memory {}", peak_rss());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect(
        "usage: par_scan FILE.bz2 [finder|check|faults|adversarial|fuzz|time|shapes|all] [ROUNDS]",
    );
    let cmd = args.get(2).map_or("all", |s| s.as_str());
    let rounds: usize = args.get(3).map_or(7, |s| s.parse().unwrap());
    if cmd == "faults" {
        // (its own input, made in memory: FILE is not read)
        return cmd_faults();
    }
    let data = std::fs::read(path).unwrap();
    let want = mibz2::decode(&data).unwrap();
    if cmd == "knee" {
        let workers: Vec<usize> = args
            .get(4)
            .expect("usage: par_scan FILE.bz2 knee ROUNDS WORKERS")
            .split(',')
            .map(|w| w.parse().expect("WORKERS: whole numbers, as 1,2,4"))
            .collect();
        assert!(
            args.get(3).is_some() && rounds >= 1 && workers.iter().all(|&w| w >= 1),
            "usage: par_scan FILE.bz2 knee ROUNDS WORKERS (each at least 1)"
        );
        eprintln!(
            "{path}: {} bytes -> {} bytes; knee: {rounds} rounds, workers {workers:?}",
            data.len(),
            want.len()
        );
        // (each cell: t workers, a finder or scanner thread and this thread, the joiner,
        // none pinned)
        eprintln!("knee: placement: {}", placement());
        return cmd_knee(&data, &want, rounds, &workers);
    }
    println!(
        "{path}: {} bytes -> {} bytes ({:.2}x)",
        data.len(),
        want.len(),
        want.len() as f64 / data.len() as f64
    );
    let all = cmd == "all";
    if all || cmd == "finder" {
        cmd_finder(&data, want.len());
    }
    if all || cmd == "check" {
        cmd_check(&data, &want);
    }
    if all || cmd == "adversarial" {
        cmd_adversarial(&want);
    }
    if all || cmd == "fuzz" {
        cmd_fuzz(&want, if all { 1500 } else { rounds.max(1500) });
    }
    if all || cmd == "time" {
        cmd_time(&data, &want, rounds);
    }
    if all || cmd == "shapes" {
        cmd_shapes(&want, rounds.min(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A panic at every role of both arms ends the decode (natively: threads and libbzip2)
    #[cfg(not(miri))]
    #[test]
    fn faults_end_never_hang() {
        let lines = run_faults(120).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(lines.len(), 1 + 16, "{lines:#?}");
    }

    /// Bytes with both magics planted at chosen bits (any shift, near either end)
    fn planted(len: usize, rng: &mut Rng, at: &[(u64, bool)]) -> Vec<u8> {
        let mut d: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        for &(bit, end) in at {
            let m = if end { END_MAGIC } else { BLOCK_MAGIC };
            for j in 0..48u64 {
                let b = bit + j;
                if b / 8 >= len as u64 {
                    break;
                }
                let (byte, sh) = ((b / 8) as usize, 7 - (b % 8) as u32);
                let v = (m >> (47 - j) & 1) as u8;
                d[byte] = d[byte] & !(1 << sh) | v << sh;
            }
        }
        d
    }

    /// The finder's every path, the AVX2 filter's included, against the bit-by-bit
    /// reference, on every length up to 96 bytes and every cut of each into two ranges
    /// (the same 6-byte overlap rule the harness relies on): Miri checks each of the
    /// filter's unaligned 32-byte loads and its 8-byte verifications for bounds and
    /// initialisation. Run under Miri with AVX2 enabled for the target (the test fails
    /// rather than skip the filter where AVX2 was asked for and is absent).
    #[test]
    fn miri_finder_matches_naive() {
        let mut rng = Rng(0x0dd_ba11_cafe_f00d);
        let avx2 = Finder::detect().avx2;
        if cfg!(all(target_arch = "x86_64", target_feature = "avx2")) {
            assert!(
                avx2,
                "AVX2 is a target feature here, so the filter must be detected"
            );
        }
        let finders: Vec<Finder> = if avx2 {
            vec![Finder::new(true), Finder::new(false)]
        } else {
            vec![Finder::new(false)]
        };
        // (under Miri: the AVX2 loop's tightest lengths too, 32i + 34..=40, where its last
        // load ends within a few bytes of the input's end, and a loosened bound reads past)
        let lens: Vec<usize> = if cfg!(miri) {
            vec![
                0, 5, 6, 7, 34, 35, 36, 37, 38, 39, 40, 41, 45, 66, 69, 70, 72, 96,
            ]
        } else {
            (0..=96).collect()
        };
        for len in lens {
            let bits = len as u64 * 8;
            let mut at = Vec::new();
            for _ in 0..3 {
                if bits >= 48 {
                    at.push((rng.below(bits - 47), rng.below(2) == 1));
                }
            }
            // and at the last bit a magic fits, and one past it (cut short)
            if bits >= 48 {
                at.push((bits - 48, false));
            }
            if bits >= 47 {
                at.push((bits - 47, true));
            }
            let d = planted(len, &mut rng, &at);
            let want = find_naive(&d);
            for f in &finders {
                let mut v = Vec::new();
                f.find(&d, 0, len, &mut v);
                assert_eq!(v, want, "len {len}, avx2 {}", f.avx2);
                let cuts: Vec<usize> = if cfg!(miri) {
                    vec![len / 3, len / 2]
                } else {
                    (0..=len).collect()
                };
                for cut in cuts {
                    let mut v = Vec::new();
                    f.find(&d, 0, cut, &mut v);
                    f.find(&d, cut, len, &mut v);
                    assert_eq!(v, want, "len {len}, cut {cut}, avx2 {}", f.avx2);
                }
            }
        }
    }

    // ---- B5: fuzz and mutation tests aimed at the finder's `unsafe` (journal §8.4) ----

    /// The finders to compare: the AVX2 filter wherever the CPU has it, and the scalar
    fn finders() -> Vec<Finder> {
        if Finder::detect().avx2 {
            vec![Finder::new(true), Finder::new(false)]
        } else {
            vec![Finder::new(false)]
        }
    }

    /// Writable pages between two inaccessible ones, so a read one byte before or past a
    /// slice placed against either guard faults (SIGSEGV) natively, where an out-of-bounds
    /// load would otherwise read whatever follows the allocation and go unseen
    #[cfg(all(target_os = "linux", not(miri)))]
    struct Guarded {
        base: *mut u8,
        page: usize,
        pages: usize,
    }

    #[cfg(all(target_os = "linux", not(miri)))]
    mod sys {
        extern "C" {
            pub fn mmap(
                a: *mut u8,
                len: usize,
                prot: i32,
                flags: i32,
                fd: i32,
                off: i64,
            ) -> *mut u8;
            pub fn mprotect(a: *mut u8, len: usize, prot: i32) -> i32;
            pub fn munmap(a: *mut u8, len: usize) -> i32;
        }
    }

    #[cfg(all(target_os = "linux", not(miri)))]
    impl Guarded {
        fn new(max_len: usize) -> Guarded {
            let page = 4096;
            let pages = max_len.div_ceil(page).max(1);
            let total = (pages + 2) * page;
            // SAFETY: an anonymous private mapping of `total` bytes (PROT_READ|PROT_WRITE,
            // MAP_PRIVATE|MAP_ANONYMOUS); checked against MAP_FAILED; the first and last
            // pages are then made inaccessible, and nothing else maps or frees them
            unsafe {
                let base = sys::mmap(std::ptr::null_mut(), total, 3, 0x02 | 0x20, -1, 0);
                assert!(base as isize != -1, "mmap failed");
                assert_eq!(sys::mprotect(base, page, 0), 0);
                assert_eq!(sys::mprotect(base.add((pages + 1) * page), page, 0), 0);
                Guarded { base, page, pages }
            }
        }

        /// `d` copied against the trailing guard (`tail`) or the leading one
        fn place(&mut self, d: &[u8], tail: bool) -> &[u8] {
            assert!(d.len() <= self.pages * self.page);
            let rw = self.pages * self.page;
            let off = if tail { rw - d.len() } else { 0 };
            // SAFETY: `off..off + d.len()` lies within the writable pages, which only this
            // value refers to; the slice borrows `self`, so it ends before the next place
            unsafe {
                let p = self.base.add(self.page + off);
                std::ptr::copy_nonoverlapping(d.as_ptr(), p, d.len());
                std::slice::from_raw_parts(p, d.len())
            }
        }
    }

    #[cfg(all(target_os = "linux", not(miri)))]
    impl Drop for Guarded {
        fn drop(&mut self) {
            // SAFETY: the mapping made in `new`, unmapped once
            unsafe { sys::munmap(self.base, (self.pages + 2) * self.page) };
        }
    }

    /// Elsewhere (and under Miri, which checks every load's bounds itself): a plain copy
    #[cfg(not(all(target_os = "linux", not(miri))))]
    struct Guarded(Vec<u8>);

    #[cfg(not(all(target_os = "linux", not(miri))))]
    impl Guarded {
        fn new(_: usize) -> Guarded {
            Guarded(Vec::new())
        }
        fn place(&mut self, d: &[u8], _: bool) -> &[u8] {
            self.0 = d.to_vec();
            &self.0
        }
    }

    /// Counts of what a fuzz run compared
    #[derive(Default, Debug)]
    struct FuzzCounts {
        inputs: u64,
        bytes: u64,
        find_calls: u64,
        cands: u64,
        planted: u64,
        guarded: u64,
    }

    /// Every finder against the bit-by-bit reference on `d`: the whole input; a random
    /// sub-range (only the magics starting in its bytes); a random partition into 1-8
    /// ranges; and, natively, the whole input and the partition again with `d` placed
    /// against each guard page
    fn check_all(d: &[u8], fs: &[Finder], rng: &mut Rng, n: &mut FuzzCounts, g: &mut Guarded) {
        let want = find_naive(d);
        n.inputs += 1;
        n.bytes += d.len() as u64;
        n.cands += want.len() as u64;
        let len = d.len();
        let (a, b) = {
            let x = rng.below(len as u64 + 1) as usize;
            let y = rng.below(len as u64 + 1) as usize;
            (x.min(y), x.max(y))
        };
        let sub: Vec<Cand> = want
            .iter()
            .copied()
            .filter(|c| (a as u64 * 8..b as u64 * 8).contains(&c.bit))
            .collect();
        let mut cuts: Vec<usize> = (0..rng.below(8))
            .map(|_| rng.below(len as u64 + 1) as usize)
            .collect();
        cuts.push(0);
        cuts.push(len);
        cuts.sort_unstable();
        let run = |f: &Finder, d: &[u8], n: &mut FuzzCounts, what: &str| {
            let mut v = Vec::new();
            f.find(d, 0, len, &mut v);
            assert_eq!(
                v, want,
                "{what}: len {len}, avx2 {}, input {d:02x?}",
                f.avx2
            );
            let mut v = Vec::new();
            for w in cuts.windows(2) {
                f.find(d, w[0], w[1], &mut v);
            }
            assert_eq!(
                v, want,
                "{what}: cuts {cuts:?}, avx2 {}, input {d:02x?}",
                f.avx2
            );
            n.find_calls += cuts.len() as u64;
        };
        for f in fs {
            run(f, d, n, "plain");
            let mut v = Vec::new();
            f.find(d, a, b, &mut v);
            assert_eq!(v, sub, "range {a}..{b}, avx2 {}, input {d:02x?}", f.avx2);
            n.find_calls += 1;
            for tail in [true, false] {
                let gd = g.place(d, tail);
                run(f, gd, n, if tail { "guarded tail" } else { "guarded head" });
                n.guarded += 1;
            }
        }
    }

    /// One fuzz input: its length biased to the AVX2 loop's edges (the bound
    /// `k + 32 <= to && k + 40 <= len` and the scalar tail), its bytes one of several
    /// shapes, and magics (and near-misses) planted at random bits, at the tail, and across
    /// 32-byte step edges
    fn fuzz_input(rng: &mut Rng, n: &mut FuzzCounts) -> Vec<u8> {
        let len = match rng.below(10) {
            0..=5 => rng.below(140) as usize,
            6 | 7 => 32 * rng.below(8) as usize + rng.below(16) as usize + 30,
            8 => rng.below(1200) as usize,
            _ => rng.below(5000) as usize,
        };
        let bits = len as u64 * 8;
        let mut d: Vec<u8> = match rng.below(6) {
            0 => vec![0; len],
            1 => vec![0xff; len],
            2 => {
                // bytes of the magics, at random shifts: many filter hits
                let m = [BLOCK_MAGIC, END_MAGIC][rng.below(2) as usize];
                (0..len)
                    .map(|_| (m << (16 - rng.below(8))).to_be_bytes()[1 + rng.below(5) as usize])
                    .collect()
            }
            _ => (0..len).map(|_| rng.next() as u8).collect(),
        };
        let mut plant = |d: &mut Vec<u8>, bit: u64, m: u64, flip: Option<u64>| {
            n.planted += 1;
            for j in 0..48u64 {
                let b = bit + j;
                if b >= bits {
                    break;
                }
                let mut v = (m >> (47 - j) & 1) as u8;
                if flip == Some(j) {
                    v ^= 1;
                }
                let (byte, sh) = ((b / 8) as usize, 7 - (b % 8) as u32);
                d[byte] = d[byte] & !(1 << sh) | v << sh;
            }
        };
        if bits == 0 {
            return d;
        }
        for _ in 0..rng.below(6) {
            let m = [BLOCK_MAGIC, END_MAGIC][rng.below(2) as usize];
            let flip = (rng.below(4) == 0).then(|| rng.below(48));
            let bit = match rng.below(5) {
                // at or near the tail, including past where one fits
                0 => bits.saturating_sub(40 + rng.below(16)),
                // across a 32-byte step edge (window starts k..k+32 from 0)
                1 => {
                    (32 * rng.below(len as u64 / 32 + 1) + rng.below(3)).saturating_sub(1) * 8
                        + rng.below(8)
                }
                // a chain of magics, back to back
                2 => {
                    let mut b = rng.below(bits);
                    while b < bits {
                        plant(&mut d, b, m, None);
                        b += 48;
                    }
                    continue;
                }
                _ => rng.below(bits),
            };
            plant(&mut d, bit.min(bits - 1), m, flip);
        }
        d
    }

    /// The finder's randomised differential fuzz: the AVX2 filter and the scalar finder
    /// against the bit-by-bit reference (`check_all`) on `fuzz_input`s. The budget is
    /// PAR_SCAN_FUZZ_SECS seconds (the B5 run), else a fixed 3,000 inputs (CI); the seed
    /// PAR_SCAN_FUZZ_SEED (printed, so a failure replays)
    #[cfg_attr(miri, ignore)]
    #[test]
    fn fuzz_finder_differential() {
        let seed = std::env::var("PAR_SCAN_FUZZ_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0x5eed_f00d_u64);
        let secs: Option<f64> = std::env::var("PAR_SCAN_FUZZ_SECS")
            .ok()
            .and_then(|s| s.parse().ok());
        let mut rng = Rng(seed | 1);
        let fs = finders();
        let mut n = FuzzCounts::default();
        let mut g = Guarded::new(8192);
        let t0 = Instant::now();
        loop {
            let d = fuzz_input(&mut rng, &mut n);
            check_all(&d, &fs, &mut rng, &mut n, &mut g);
            let more = match secs {
                Some(s) => t0.elapsed().as_secs_f64() < s,
                None => n.inputs < 3000,
            };
            if !more {
                break;
            }
        }
        eprintln!(
            "fuzz_finder_differential: seed {seed:#x}, finders {} (avx2 {}): {n:?}",
            fs.len(),
            fs[0].avx2
        );
    }

    /// Exhaustive: one magic of each kind at every bit of every length 40-104 (the AVX2
    /// loop's first steps and its scalar tail), on a zero and a random background, each
    /// input also cut at every byte from 2 before the magic's to 7 after, and guarded
    #[cfg_attr(miri, ignore)]
    #[test]
    fn finder_every_bit_offset() {
        let fs = finders();
        let mut rng = Rng(0xb17_0ff5e7);
        let mut n = FuzzCounts::default();
        let mut g = Guarded::new(4096);
        for len in 40..=104usize {
            let bits = len as u64 * 8;
            for m in [BLOCK_MAGIC, END_MAGIC] {
                for bit in 0..bits {
                    for random in [false, true] {
                        let mut d: Vec<u8> = if random {
                            (0..len).map(|_| rng.next() as u8).collect()
                        } else {
                            vec![0; len]
                        };
                        for j in 0..48u64.min(bits - bit) {
                            let b = bit + j;
                            let (byte, sh) = ((b / 8) as usize, 7 - (b % 8) as u32);
                            let v = (m >> (47 - j) & 1) as u8;
                            d[byte] = d[byte] & !(1 << sh) | v << sh;
                        }
                        let want = find_naive(&d);
                        assert_eq!(
                            want.iter().any(|c| c.bit == bit),
                            bit + 48 <= bits,
                            "the reference itself"
                        );
                        n.inputs += 1;
                        let k = (bit / 8) as usize;
                        for f in &fs {
                            let mut v = Vec::new();
                            f.find(&d, 0, len, &mut v);
                            assert_eq!(v, want, "len {len}, bit {bit}, avx2 {}", f.avx2);
                            for cut in k.saturating_sub(2)..=(k + 7).min(len) {
                                let mut v = Vec::new();
                                f.find(&d, 0, cut, &mut v);
                                f.find(&d, cut, len, &mut v);
                                assert_eq!(v, want, "len {len}, bit {bit}, cut {cut}");
                                n.find_calls += 2;
                            }
                            for tail in [true, false] {
                                let gd = g.place(&d, tail);
                                let mut v = Vec::new();
                                f.find(gd, 0, len, &mut v);
                                assert_eq!(v, want, "guarded, len {len}, bit {bit}");
                                n.guarded += 1;
                            }
                        }
                    }
                }
            }
        }
        eprintln!("finder_every_bit_offset: {n:?}");
    }

    /// The harness's own chunking: a magic whose first bit is `r` bits before each
    /// FIND_CHUNK edge, for every r in 0..=48 (straddling the edge, ending at it, starting
    /// on it), found when the input is searched chunk by chunk as the finder thread does
    #[cfg_attr(miri, ignore)]
    #[test]
    fn finder_find_chunk_edges() {
        let fs = finders();
        let mut rng = Rng(0xc40e_d6e5);
        let len = 3 * FIND_CHUNK + 77;
        for r in 0..=48u64 {
            let at: Vec<(u64, bool)> = [FIND_CHUNK, 2 * FIND_CHUNK, 3 * FIND_CHUNK]
                .iter()
                .map(|&e| (e as u64 * 8 - r, r % 2 == 1))
                .collect();
            let d = planted(len, &mut rng, &at);
            let want = find_naive(&d);
            for &(bit, _) in &at {
                assert!(want.iter().any(|c| c.bit == bit), "planted at {bit}");
            }
            for f in &fs {
                let (mut v, mut from) = (Vec::new(), 0);
                while from < d.len() {
                    let to = (from + FIND_CHUNK).min(d.len());
                    f.find(&d, from, to, &mut v);
                    from = to;
                }
                assert_eq!(v, want, "chunked, r {r}, avx2 {}", f.avx2);
            }
        }
    }

    /// RUSAGE_SELF counts every thread's CPU: a second thread burns at least 1.1 s of its
    /// own CPU (by RUSAGE_THREAD, 1) and the process's figure grows by at least that.
    /// Catches the wrong `who` (RUSAGE_THREAD or RUSAGE_CHILDREN) and a misread timeval
    /// (past a whole second, so the seconds field is read as well as the microseconds).
    #[cfg(all(target_os = "linux", not(miri)))]
    #[test]
    fn getrusage_counts_every_thread() {
        let thread_micros = || {
            let mut r = Rusage::default();
            // SAFETY: as in `cpu_micros`; RUSAGE_THREAD (1) is Linux's
            assert_eq!(unsafe { getrusage(1, &mut r) }, 0);
            (r.utime[0] * 1_000_000 + r.utime[1] + r.stime[0] * 1_000_000 + r.stime[1]) as u64
        };
        let a = cpu_micros();
        let burnt = std::thread::spawn(move || {
            let (t0, mut x) = (thread_micros(), 0u64);
            while thread_micros() - t0 < 1_100_000 {
                for i in 0..100_000u64 {
                    x = x.wrapping_mul(31).wrapping_add(i);
                }
                std::hint::black_box(x);
            }
            thread_micros() - t0
        })
        .join()
        .unwrap();
        let grew = cpu_micros() - a;
        assert!(
            grew >= burnt,
            "the process's CPU grew {grew} us; the thread burnt {burnt} us"
        );
    }

    /// The FFI boundary: getrusage writes a whole `Rusage` and the times only grow.
    /// Natively only: not run under Miri, which does not execute this foreign call; the
    /// stated bound argument covers it (docs/journal/2026-11-10-parallel-scan.md, B5).
    #[cfg(not(miri))]
    #[test]
    fn getrusage_times_grow() {
        let a = cpu_micros();
        let mut x = 0u64;
        for i in 0..20_000_000u64 {
            x = x.wrapping_mul(31).wrapping_add(i);
        }
        std::hint::black_box(x);
        assert!(cpu_micros() >= a);
    }

    /// R1 (the B5 review, #320): `Finder::new` is safe, so it must not let a caller reach
    /// the AVX2 path on a CPU without AVX2. Under Miri without `+avx2` the request is
    /// refused, not obeyed (before the fix: "calling a function that requires unavailable
    /// target features: avx2").
    #[test]
    fn miri_r1_avx2_asked_for_without_avx2() {
        let f = Finder::new(true);
        let mut v = Vec::new();
        f.find(&[0u8; 64], 0, 64, &mut v);
        assert!(v.is_empty());
        #[cfg(target_arch = "x86_64")]
        assert_eq!(f.avx2, std::is_x86_feature_detected!("avx2"));
    }

    /// R2 (the B5 review, #320): a `from` at or past the clamped end searches nothing, and
    /// never wraps the bounds into out-of-range pointer arithmetic (before the fix, under Miri
    /// with `+avx2` and overflow checks off: "overflowing pointer arithmetic"; with overflow
    /// checks on, an overflow panic).
    #[test]
    fn miri_r2_from_past_the_end() {
        let d = [0u8; 64];
        for avx2 in [true, false] {
            let f = Finder::new(avx2);
            let mut v = Vec::new();
            f.find(&d[1..], usize::MAX - 1, 63, &mut v);
            f.find(&d[1..], usize::MAX, usize::MAX, &mut v);
            f.find(&d, 64, 64, &mut v);
            f.find(&d, 100, 200, &mut v);
            f.find(&d, 30, 10, &mut v);
            assert!(v.is_empty(), "avx2 {avx2}: {v:?}");
        }
    }
}
