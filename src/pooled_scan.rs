//! Research only (`pooled-scan`): the pooled scan's mode, its counters and its finder
//! (docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md).
//!
//! With the mode set (`set(true)`), a parallel decoder of plain workers built from then on runs
//! the pooled scan in place of its scanner thread (`parallel/pooled.rs`):
//! - **the chain thread** holds the input, finds every block and stream-end magic in it as it
//!   arrives (`Finder`, #317's), and hands each block candidate's bytes to the workers in bit
//!   order, within the shipped window;
//! - **a worker** reads the item at its candidate (the symbols, at level 9's bound) and reports
//!   where it ended, then decodes the block end to end, as a worker does now;
//! - **the chain** walks the stream exactly as `stream::Scanner` does, through
//!   `Scanner::next_hinted`: at each block it takes a worker's report only where the worker
//!   read a block from exactly the chain's bit, in a stream of level 9, as the chain's next
//!   block (its candidate's ordinal its place). Anywhere else it reads the item itself, as the
//!   shipped scanner does, and hands the block to the workers as the shipped scanner does.
//!
//! So a worker's block is used only where the sequential scanner would have read the same item
//! from the same bit at the same level, and the finder is a hint: a false or missed magic costs
//! time, and can't change a byte or an error. Unset, the decoder is as shipped.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static ON: AtomicBool = AtomicBool::new(false);

/// Switch the pooled scan on or off for parallel decoders built from now on
pub fn set(on: bool) {
    ON.store(on, Ordering::SeqCst);
}

/// Whether the pooled scan is on
pub fn active() -> bool {
    ON.load(Ordering::SeqCst)
}

/// Counts across the decodes since the last `take`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// block candidates handed to the workers
    pub dispatched: u64,
    /// blocks the chain took from a worker's read
    pub accepted: u64,
    /// candidates whose read the chain did not use: stepped over (a false magic), read badly,
    /// or out of order, the worker's decode of them wasted
    pub rejected: u64,
    /// blocks the chain read itself and handed on as the shipped scanner does
    pub fallbacks: u64,
    /// decodes that left the pooled path for the rest of their input (a candidate's ordinal
    /// not its place, or a stream below level 9)
    pub degraded: u64,
    /// the bytes the chain copied into the workers' candidates
    pub copied: u64,
}

pub(crate) static DISPATCHED: AtomicU64 = AtomicU64::new(0);
pub(crate) static ACCEPTED: AtomicU64 = AtomicU64::new(0);
pub(crate) static REJECTED: AtomicU64 = AtomicU64::new(0);
pub(crate) static FALLBACKS: AtomicU64 = AtomicU64::new(0);
pub(crate) static DEGRADED: AtomicU64 = AtomicU64::new(0);
pub(crate) static COPIED: AtomicU64 = AtomicU64::new(0);

/// The counts since the last call (and reset)
pub fn take() -> Stats {
    Stats {
        dispatched: DISPATCHED.swap(0, Ordering::SeqCst),
        accepted: ACCEPTED.swap(0, Ordering::SeqCst),
        rejected: REJECTED.swap(0, Ordering::SeqCst),
        fallbacks: FALLBACKS.swap(0, Ordering::SeqCst),
        degraded: DEGRADED.swap(0, Ordering::SeqCst),
        copied: COPIED.swap(0, Ordering::SeqCst),
    }
}

pub(crate) fn count(c: &AtomicU64, n: u64) {
    c.fetch_add(n, Ordering::Relaxed);
}

/// (testing) Every `n`-th block candidate found gets a false one after it, inside its block,
/// and every `m`-th real one is dropped (missed): the chain's paths for a false and a missed
/// magic, which real input almost never reaches. 0: none (the default)
static FAKE_EVERY: AtomicU64 = AtomicU64::new(0);
static DROP_EVERY: AtomicU64 = AtomicU64::new(0);

#[doc(hidden)]
pub fn set_test_noise(fake_every: u64, drop_every: u64) {
    FAKE_EVERY.store(fake_every, Ordering::SeqCst);
    DROP_EVERY.store(drop_every, Ordering::SeqCst);
}

/// The test noise on a search's candidates (`set_test_noise`), in bit order; `seen` counts the
/// block candidates found so far in the decode
pub(crate) fn noise(v: &mut Vec<Cand>, seen: &mut u64) {
    let (fake, drop) = (
        FAKE_EVERY.load(Ordering::Relaxed),
        DROP_EVERY.load(Ordering::Relaxed),
    );
    if fake == 0 && drop == 0 {
        *seen += v.iter().filter(|c| !c.end).count() as u64;
        return;
    }
    let mut out = Vec::with_capacity(v.len() * 2);
    for &c in v.iter() {
        if c.end {
            out.push(c);
            continue;
        }
        *seen += 1;
        if drop > 0 && *seen % drop == 0 {
            continue;
        }
        out.push(c);
        if fake > 0 && *seen % fake == 0 {
            out.push(Cand {
                bit: c.bit + 8 * (1 + c.bit % 509) + c.bit % 7,
                end: false,
            });
        }
    }
    out.sort_by_key(|c| c.bit);
    *v = out;
}

// ---- the finder (#317's, `examples/par_scan.rs`) ----

use crate::block::{BLOCK_MAGIC, END_MAGIC};

const MASK48: u64 = (1 << 48) - 1;

/// A magic found: its first bit (absolute in the input), and whether it is the stream end's
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cand {
    pub bit: u64,
    pub end: bool,
}

/// Both magics at all 8 bit shifts, as 16 byte-aligned patterns. A magic that starts `s` bits
/// into byte `k` covers bytes k+1..=k+5 whole, whatever `s` is, so bytes k+1, k+2 and k+3 are
/// a byte-aligned fingerprint of it: a filter on them, then the 48 bits compared.
pub struct Finder {
    lo: [[u8; 16]; 3],
    hi: [[u8; 16]; 3],
    pair: Vec<u64>,
    avx2: bool,
}

impl Finder {
    pub fn new(avx2: bool) -> Finder {
        let (mut lo, mut hi) = ([[0u8; 16]; 3], [[0u8; 16]; 3]);
        let mut pair = vec![0u64; 1 << 10];
        for m in [BLOCK_MAGIC, END_MAGIC] {
            for s in 0..8u32 {
                let w = (m << (16 - s)).to_be_bytes();
                for j in 0..3 {
                    lo[j][(w[j + 1] & 15) as usize] |= 1 << s;
                    hi[j][(w[j + 1] >> 4) as usize] |= 1 << s;
                }
                let idx = (w[1] as usize) << 8 | w[2] as usize;
                pair[idx >> 6] |= 1 << (idx & 63);
            }
        }
        // (AVX2 only where asked for and the CPU has it, so no caller can reach `find_avx2` on
        // a CPU without it)
        #[cfg(target_arch = "x86_64")]
        let avx2 = avx2 && std::is_x86_feature_detected!("avx2");
        #[cfg(not(target_arch = "x86_64"))]
        let avx2 = {
            let _ = avx2;
            false
        };
        Finder { lo, hi, pair, avx2 }
    }

    pub fn detect() -> Finder {
        Finder::new(true)
    }

    /// Every magic whose first bit lies in bytes `from..to` of `data` (whose first byte is the
    /// input's byte `base`), in order, appended to `v` with absolute bits. It reads at most 7
    /// bytes past `to`: ranges cut anywhere need a 6-byte overlap and no more.
    pub fn find(&self, data: &[u8], base: u64, from: usize, to: usize, v: &mut Vec<Cand>) {
        let to = to.min(data.len().saturating_sub(5));
        if from >= to {
            return;
        }
        let mut k = from;
        #[cfg(target_arch = "x86_64")]
        if self.avx2 {
            // SAFETY: `avx2` is set only where the CPU has AVX2 (`Finder::new` checks it), and
            // `from < to <= data.len() - 5` here, so `find_avx2`'s bounds cannot wrap
            k = unsafe { self.find_avx2(data, base, k, to, v) };
        }
        while k < to {
            let idx = (data[k + 1] as usize) << 8 | data[k + 2] as usize;
            if self.pair[idx >> 6] >> (idx & 63) & 1 != 0 {
                verify(data, base, k, v);
            }
            k += 1;
        }
    }

    /// `find`'s whole steps of 32 window starts; returns the first start not covered
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn find_avx2(
        &self,
        data: &[u8],
        base: u64,
        from: usize,
        to: usize,
        v: &mut Vec<Cand>,
    ) -> usize {
        use std::arch::x86_64::*;
        let tbl = |t: &[u8; 16]| {
            _mm256_broadcastsi128_si256(_mm_loadu_si128(t.as_ptr() as *const __m128i))
        };
        let (l0, l1, l2) = (tbl(&self.lo[0]), tbl(&self.lo[1]), tbl(&self.lo[2]));
        let (h0, h1, h2) = (tbl(&self.hi[0]), tbl(&self.hi[1]), tbl(&self.hi[2]));
        let nib = _mm256_set1_epi8(0x0f);
        let zero = _mm256_setzero_si256();
        let class = |x: __m256i, l: __m256i, h: __m256i| {
            let a = _mm256_shuffle_epi8(l, _mm256_and_si256(x, nib));
            let b = _mm256_shuffle_epi8(h, _mm256_and_si256(_mm256_srli_epi16(x, 4), nib));
            _mm256_and_si256(a, b)
        };
        let mut k = from;
        // a step covers window starts k..k+32: it loads data[k+1..k+35], and a start's check
        // loads 8 bytes from it (through k+38)
        while k + 32 <= to && k + 40 <= data.len() {
            // SAFETY: k + 40 <= data.len(), so the three 32-byte loads from k+1, k+2 and k+3
            // stay inside `data`
            let p = data.as_ptr().add(k + 1);
            let c0 = class(_mm256_loadu_si256(p as *const __m256i), l0, h0);
            let c1 = class(_mm256_loadu_si256(p.add(1) as *const __m256i), l1, h1);
            let c2 = class(_mm256_loadu_si256(p.add(2) as *const __m256i), l2, h2);
            let c = _mm256_and_si256(_mm256_and_si256(c0, c1), c2);
            let mut m = !(_mm256_movemask_epi8(_mm256_cmpeq_epi8(c, zero)) as u32);
            while m != 0 {
                verify(data, base, k + m.trailing_zeros() as usize, v);
                m &= m - 1;
            }
            k += 32;
        }
        k
    }
}

/// The magics that start in byte `k` of `data`, at any of its 8 bits, appended to `v`
#[inline]
fn verify(data: &[u8], base: u64, k: usize, v: &mut Vec<Cand>) {
    let w = match data.get(k..k + 8) {
        Some(b) => u64::from_be_bytes(b.try_into().expect("8 bytes")),
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
                bit: base * 8 + bit,
                end: x == END_MAGIC,
            });
        }
    }
}

/// The reference finder: every bit tried (the finder's test)
pub fn find_naive(data: &[u8]) -> Vec<Cand> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_finder_finds_what_the_naive_one_does_at_every_cut() {
        // magics at every shift, both kinds, among noise
        let mut data = vec![0u8; 4000];
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        for b in data.iter_mut() {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *b = x as u8;
        }
        // plant each magic at bit `at`, by bits
        let mut set = |at: u64, m: u64| {
            for j in 0..48u64 {
                let bit = at + j;
                let (byte, sh) = ((bit / 8) as usize, 7 - (bit % 8));
                let v = (m >> (47 - j)) & 1;
                data[byte] = data[byte] & !(1 << sh) | ((v as u8) << sh);
            }
        };
        for i in 0..16u64 {
            let m = if i % 2 == 0 { BLOCK_MAGIC } else { END_MAGIC };
            set(800 + i * 1600 + i % 8, m);
        }
        let want = find_naive(&data);
        assert!(want.len() >= 16);
        for avx2 in [false, true] {
            let f = Finder::new(avx2);
            let mut got = Vec::new();
            f.find(&data, 0, 0, data.len(), &mut got);
            assert_eq!(got, want, "avx2 {avx2}");
            // cut into ranges with a 6-byte overlap, at a base
            for cut in [1usize, 7, 33, 500, 1999] {
                let mut got = Vec::new();
                let mut from = 0;
                while from < data.len() {
                    let to = (from + cut).min(data.len());
                    let lo = from.saturating_sub(0);
                    let hi = (to + 7).min(data.len());
                    f.find(&data[lo..hi], lo as u64, 0, to - lo, &mut got);
                    from = to;
                }
                assert_eq!(got, want, "avx2 {avx2} cut {cut}");
            }
        }
    }
}
