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

use std::sync::atomic::{AtomicUsize, Ordering};

/// The megabytes streamed through per admitted block: `MIBZ2_DOSE_MB` at compile time
pub const DOSE_MB: usize = parse_mb(option_env!("MIBZ2_DOSE_MB"));

/// A cache line
const LINE: usize = 64;

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
}

impl Pad {
    /// One dose of `DOSE_MB` MB
    #[inline(never)]
    pub(crate) fn apply(&mut self) {
        self.apply_bytes(DOSE_MB << 20);
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
