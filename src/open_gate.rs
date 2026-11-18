//! Research only (`open-return`): the open-return comparison's two wrappers around Astra's
//! open decoder (#565, `docs/journal/2026-11-18-open-return/`, compiled from the journal as
//! `open_return`), for `examples/ab.rs` (#145 QUEUED 6978711740). Neither is on a production
//! path.
//!
//! - [`OpenGated`]: the open decoder on the blocks under LazyCells' density line, LazyCells as
//!   shipped on every other block (it refuses them on that same line, by its own fallback). The
//!   line is LazyCells' first test: it also refuses a block under the line whose induction
//!   exceeds its cell budget, which the line cannot see, and there the open decoder runs.
//! - [`Census`]: LazyCells as shipped, counting each block's route: on the run route, admitted
//!   by LazyCells (its counts; it handles every run-route block, refusals by its own fallback),
//!   under the density line, both (and the column route's blocks).
use crate::bwt::{InverseBwt, LazyCells};
use crate::open_return::Open;
use crate::runs::Runs;
use crate::Kernels;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// Whether a block of `records` capped run records in `n` rows is under LazyCells' density line
/// (`cells_lazy.rs`' `LAZY_DENSITY`, 3 records in 20 rows)
pub fn under_density(records: usize, n: usize) -> bool {
    records * 20 < n * 3
}

/// Astra's queued open endpoint at width 16 under LazyCells' density line, LazyCells as shipped
/// over it and on the column route
#[derive(Default)]
pub struct OpenGated {
    open: Open<16, true, true>,
    lazy: LazyCells,
}

impl InverseBwt for OpenGated {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(ll, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy
            .invert_checkpointed(ll, counts, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(ll, counts, p)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if under_density(r.records().len(), n) {
            self.open.invert_runs(r, n, p, out, k)
        } else {
            self.lazy.invert_runs(r, n, p, out, k)
        }
    }
}

/// The census's counts so far: [run-route blocks, admitted by LazyCells, under the density
/// line, both, column-route blocks]
pub static CENSUS: [AtomicU64; 5] = [const { AtomicU64::new(0) }; 5];

/// LazyCells as shipped, counting each block's route into [`CENSUS`]
#[derive(Default)]
pub struct Census {
    lazy: LazyCells,
}

impl InverseBwt for Census {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        CENSUS[4].fetch_add(1, Relaxed);
        self.lazy.invert(ll, counts, p, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        CENSUS[4].fetch_add(1, Relaxed);
        self.lazy
            .invert_checkpointed(ll, counts, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        // (a refusal goes on to `invert`, which counts it)
        let took = self.lazy.invert_in_place(ll, counts, p);
        CENSUS[4].fetch_add(took as u64, Relaxed);
        took
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let under = under_density(r.records().len(), n);
        // (LazyCells handles every run-route block, its refusals by its own fallback: its
        // admission is read from its counts)
        let before = self.lazy.counts().admitted;
        let took = self.lazy.invert_runs(r, n, p, out, k);
        let admitted = self.lazy.counts().admitted > before;
        CENSUS[0].fetch_add(1, Relaxed);
        CENSUS[1].fetch_add(admitted as u64, Relaxed);
        CENSUS[2].fetch_add(under as u64, Relaxed);
        CENSUS[3].fetch_add((under && admitted) as u64, Relaxed);
        took
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// 100 kB blocks: Wikidata-like records (repetitive, under the density line) and this
    /// file's own source (prose-like, over it), then bytes from an LCG (the column route)
    fn stream() -> Vec<u8> {
        let mut src = Vec::new();
        for i in 0..6000u32 {
            let q = i % 37;
            writeln!(src, "{{\"id\":\"Q{q}\",\"type\":\"item\",\"labels\":{{\"en\":{{\"language\":\"en\",\"value\":\"item {q}\"}}}}}}").unwrap();
        }
        for _ in 0..4 {
            src.extend_from_slice(include_bytes!("open_gate.rs"));
            src.extend_from_slice(include_bytes!("bwt.rs"));
        }
        let mut x = 0x2545_f491_4f6c_dd1du64;
        for _ in 0..150_000 {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            src.push((x >> 56) as u8);
        }
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(1));
        e.write_all(&src).unwrap();
        e.finish().unwrap()
    }

    #[test]
    fn open_arms_decode_as_the_default_and_the_census_sees_each_route() {
        let data = stream();
        let want = crate::decode(&data).unwrap();
        let k = Kernels::detect();
        assert!(crate::decode_with::<Open<16, true, true>>(&data, k).unwrap() == want);
        assert!(crate::decode_with::<OpenGated>(&data, k).unwrap() == want);
        let before: Vec<u64> = CENSUS.iter().map(|c| c.load(Relaxed)).collect();
        assert!(crate::decode_with::<Census>(&data, k).unwrap() == want);
        let d: Vec<u64> = CENSUS
            .iter()
            .zip(&before)
            .map(|(c, b)| c.load(Relaxed) - b)
            .collect();
        // run route, admitted, under the line, both, column route: here [8, 1, 5, 1, 3], the
        // line's blocks over LazyCells' budget among them
        eprintln!("census {d:?}");
        assert!(d[2] > 0 && d[2] < d[0], "{d:?}");
        assert!(d[3] <= d[1].min(d[2]) && d[4] > 0, "{d:?}");
    }
}
