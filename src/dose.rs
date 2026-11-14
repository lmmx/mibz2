//! Research only, behind the `dose` feature (off by default): the Cells memory-dose
//! experiment (#145, LOG 6879466310). Each block Cells admits in `invert_runs` also streams
//! through `DOSE_MB` MB of its kernel's own scratch: one read and one write of every 64-byte
//! line, so a block costs extra memory traffic and nothing else. It decodes nothing and
//! writes nothing the decoder reads, so the output is byte-identical. Without the feature
//! this module is not compiled and nothing in the decoder refers to it.
//!
//! `DOSE_MB` is fixed when the crate is compiled, from the environment variable
//! `MIBZ2_DOSE_MB` (whole megabytes, 2^20 bytes; unset or empty is 0), so each arm of the
//! experiment is its own build.
//!
//! The dose's kind, `DOSE_KIND`, is fixed the same way, from `MIBZ2_DOSE_KIND` (the LOG's
//! amendment, registered before any data):
//! - `stream` (unset or empty): the streaming dose above;
//! - `chase`: a dependent pointer chase instead. The scratch holds a random single-cycle
//!   permutation of its 64-byte lines (Sattolo's, from a fixed seed, built once per kernel),
//!   each line's first 4 bytes the next line's index; each dose follows it from line 0 for
//!   one hop a line, every line visited once: `DOSE_MB` × 16384 dependent loads, the last
//!   index kept through `black_box`. Nothing is written.

use std::sync::atomic::{AtomicUsize, Ordering};

/// The megabytes streamed through per admitted block: `MIBZ2_DOSE_MB` at compile time
pub const DOSE_MB: usize = parse_mb(option_env!("MIBZ2_DOSE_MB"));

/// The dose's kind: `MIBZ2_DOSE_KIND` at compile time
pub const DOSE_KIND: Kind = parse_kind(option_env!("MIBZ2_DOSE_KIND"));

/// A dose's kind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// one read and one write of every line, in address order
    Stream,
    /// one dependent load a line, in a random single cycle's order
    Chase,
}

impl Kind {
    /// The name `MIBZ2_DOSE_KIND` takes
    pub const fn name(self) -> &'static str {
        match self {
            Kind::Stream => "stream",
            Kind::Chase => "chase",
        }
    }
}

/// A cache line
const LINE: usize = 64;
/// A line's u32 words
const WORDS: usize = LINE / 4;

const fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn parse_kind(s: Option<&str>) -> Kind {
    let b = match s {
        Some(s) => s.as_bytes(),
        None => return Kind::Stream,
    };
    if b.is_empty() || eq(b, b"stream") {
        Kind::Stream
    } else if eq(b, b"chase") {
        Kind::Chase
    } else {
        panic!("MIBZ2_DOSE_KIND is stream or chase")
    }
}

const fn parse_mb(s: Option<&str>) -> usize {
    let b = match s {
        Some(s) => s.as_bytes(),
        None => return 0,
    };
    let mut n = 0usize;
    let mut i = 0;
    while i < b.len() {
        let d = b[i];
        assert!(
            d.is_ascii_digit(),
            "MIBZ2_DOSE_MB must be a whole number of MB"
        );
        n = n * 10 + (d - b'0') as usize;
        i += 1;
    }
    assert!(n <= 4096, "MIBZ2_DOSE_MB is at most 4096");
    n
}

/// Blocks dosed in this process, at any `DOSE_MB` (0 included): whether Cells engaged
static BLOCKS: AtomicUsize = AtomicUsize::new(0);

/// The blocks this process has dosed so far (each a block Cells admitted in `invert_runs`)
pub fn blocks() -> usize {
    BLOCKS.load(Ordering::Relaxed)
}

/// A kernel's scratch: allocated (and every page touched) at its first dose, then reused,
/// as the kernel's own buffers are
#[derive(Default)]
pub(crate) struct Pad {
    buf: Vec<u8>,
    /// the chase's lines, `WORDS` words each, the first the next line's index
    chase: Vec<u32>,
}

impl Pad {
    /// One dose of `DOSE_MB` MB, of `DOSE_KIND`
    #[inline(never)]
    pub(crate) fn apply(&mut self) {
        match DOSE_KIND {
            Kind::Stream => self.apply_bytes(DOSE_MB << 20),
            Kind::Chase => {
                std::hint::black_box(self.chase_bytes(DOSE_MB << 20));
            }
        }
    }

    /// One chase over `bytes` (rounded down to whole lines): from line 0, one dependent
    /// load a line along a single cycle through every line, built at the first chase (a
    /// fixed seed) and reused. Returns where it ended (line 0, after a whole cycle)
    #[inline(never)]
    pub(crate) fn chase_bytes(&mut self, bytes: usize) -> u32 {
        BLOCKS.fetch_add(1, Ordering::Relaxed);
        let lines = bytes / LINE;
        if lines == 0 {
            return 0;
        }
        assert!(lines <= u32::MAX as usize);
        if self.chase.len() != lines * WORDS {
            self.chase = vec![0u32; lines * WORDS];
            // Sattolo's shuffle: a uniformly random permutation with one cycle
            let mut next: Vec<u32> = (0..lines as u32).collect();
            let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
            for i in (1..lines).rev() {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                next.swap(i, (x % i as u64) as usize);
            }
            for (line, &to) in next.iter().enumerate() {
                self.chase[line * WORDS] = to;
            }
        }
        let chase = std::hint::black_box(&self.chase[..]);
        let mut at = 0u32;
        for _ in 0..lines {
            at = chase[at as usize * WORDS];
        }
        at
    }

    /// One dose of `bytes` (rounded down to whole lines): each 64-byte line read once and
    /// written once, through volatile accesses the compiler must keep
    #[inline(never)]
    pub(crate) fn apply_bytes(&mut self, bytes: usize) {
        BLOCKS.fetch_add(1, Ordering::Relaxed);
        let lines = bytes / LINE;
        if lines == 0 {
            return;
        }
        if self.buf.len() < lines * LINE {
            self.buf = vec![1u8; lines * LINE];
        }
        let base = self.buf.as_mut_ptr();
        for i in 0..lines {
            // SAFETY: line i's first 8 bytes lie inside `buf`, which holds `lines` whole
            // lines of initialised bytes; nothing else refers to `buf` during the loop.
            unsafe {
                let p = base.add(i * LINE).cast::<u64>();
                let v = p.read_volatile();
                p.write_volatile(v.wrapping_add(1));
            }
        }
    }

    #[cfg(test)]
    fn sum(&self) -> u64 {
        self.buf.iter().map(|&b| u64::from(b)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_dose() {
        assert_eq!(parse_mb(None), 0);
        assert_eq!(parse_mb(Some("")), 0);
        assert_eq!(parse_mb(Some("0")), 0);
        assert_eq!(parse_mb(Some("8")), 8);
        assert_eq!(parse_mb(Some("16")), 16);
        assert_eq!(parse_kind(None), Kind::Stream);
        assert_eq!(parse_kind(Some("")), Kind::Stream);
        assert_eq!(parse_kind(Some("stream")), Kind::Stream);
        assert_eq!(parse_kind(Some("chase")), Kind::Chase);
    }

    #[test]
    fn a_chase_is_one_cycle_through_every_line_and_reuses_its_buffer() {
        let mut pad = Pad::default();
        let lines = 1000;
        assert_eq!(
            pad.chase_bytes(lines * LINE + 5),
            0,
            "a whole cycle ends where it began"
        );
        assert_eq!(pad.chase.len(), lines * WORDS);
        let mut seen = vec![false; lines];
        let mut at = 0usize;
        for _ in 0..lines {
            assert!(!seen[at], "line {at} visited twice");
            seen[at] = true;
            at = pad.chase[at * WORDS] as usize;
        }
        assert!(seen.iter().all(|&s| s) && at == 0);
        let ptr = pad.chase.as_ptr();
        assert_eq!(pad.chase_bytes(lines * LINE), 0);
        assert_eq!(pad.chase.as_ptr(), ptr, "reused, not rebuilt");
    }

    #[test]
    fn a_dose_touches_every_line_once_and_reuses_its_buffer() {
        let mut pad = Pad::default();
        let before = blocks();
        pad.apply_bytes(10 * LINE + 7);
        assert_eq!(pad.buf.len(), 10 * LINE);
        // each line's first byte read and written back one higher: 640 ones, +10
        assert_eq!(pad.sum(), 10 * LINE as u64 + 10);
        let ptr = pad.buf.as_ptr();
        pad.apply_bytes(10 * LINE);
        assert_eq!(pad.buf.as_ptr(), ptr, "reused, not reallocated");
        assert_eq!(pad.sum(), 10 * LINE as u64 + 20);
        pad.apply_bytes(0);
        assert!(blocks() >= before + 3);
    }
}
