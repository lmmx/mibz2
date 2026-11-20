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

/// One block's admission census line: (records, rows, LazyCells admitted, production's ns,
/// open's ns, the same bytes, LazyCells' decision), the fastest of `TIMED_REPS` each
pub type TimedBlock = (usize, usize, bool, u64, u64, bool, &'static str);
/// Every block's census line, in decode order
pub static TIMED: std::sync::Mutex<Vec<TimedBlock>> = std::sync::Mutex::new(Vec::new());
/// Reps a block for [`Timed`]
pub const TIMED_REPS: usize = 3;

/// The per-block admission census (research only): every run-route block decoded by
/// production's LazyCells route (its own fallback included) and by the open kernel, each
/// timed, the fastest of [`TIMED_REPS`], interleaved; production's output is the one kept
#[derive(Default)]
pub struct Timed {
    lazy: LazyCells,
    open: Open<16, true, true>,
    scratch: Vec<u8>,
}

impl InverseBwt for Timed {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        let (mut tl, mut to) = (u64::MAX, u64::MAX);
        for _ in 0..TIMED_REPS {
            let t = std::time::Instant::now();
            self.lazy.invert(ll, counts, p, out);
            tl = tl.min(t.elapsed().as_nanos() as u64);
            let t = std::time::Instant::now();
            self.open.invert(ll, counts, p, &mut self.scratch);
            to = to.min(t.elapsed().as_nanos() as u64);
        }
        let same = *out == self.scratch;
        TIMED
            .lock()
            .unwrap()
            .push((0, ll.len(), false, tl, to, same, "column"));
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        let (mut tl, mut to) = (u64::MAX, u64::MAX);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        let (mut ta, mut tb) = (false, false);
        for _ in 0..TIMED_REPS {
            a.clone_from(ll);
            let t = std::time::Instant::now();
            ta = self.lazy.invert_in_place(&mut a, counts, p);
            tl = tl.min(t.elapsed().as_nanos() as u64);
            b.clone_from(ll);
            let t = std::time::Instant::now();
            tb = self.open.invert_in_place(&mut b, counts, p);
            to = to.min(t.elapsed().as_nanos() as u64);
        }
        if ta || tb {
            let why = if ta && tb {
                "in-place"
            } else if ta {
                "in-place prod only"
            } else {
                "in-place open only"
            };
            TIMED
                .lock()
                .unwrap()
                .push((0, ll.len(), false, tl, to, ta == tb && a == b, why));
        }
        if ta {
            *ll = a;
        }
        ta
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let c0 = self.lazy.counts();
        let (mut tl, mut to) = (u64::MAX, u64::MAX);
        let mut admitted = false;
        let mut took = true;
        for _ in 0..TIMED_REPS {
            let before = self.lazy.counts().admitted;
            let t = std::time::Instant::now();
            took = self.lazy.invert_runs(r, n, p, out, k);
            tl = tl.min(t.elapsed().as_nanos() as u64);
            admitted = self.lazy.counts().admitted > before;
            let t = std::time::Instant::now();
            self.open.invert_runs(r, n, p, &mut self.scratch, k);
            to = to.min(t.elapsed().as_nanos() as u64);
        }
        let same = took && *out == self.scratch;
        let c1 = self.lazy.counts();
        let why = if c1.admitted > c0.admitted {
            "admitted"
        } else if c1.density > c0.density {
            "density"
        } else if c1.sparse > c0.sparse {
            "sparse"
        } else if c1.floor > c0.floor {
            "floor"
        } else if c1.budget > c0.budget {
            "budget"
        } else if c1.invalid > c0.invalid {
            "invalid"
        } else {
            "none"
        };
        TIMED
            .lock()
            .unwrap()
            .push((r.records().len(), n, admitted, tl, to, same, why));
        took
    }
}

/// One block kept for the thin-run limit sweep: (block index, its capped run records, rows,
/// primary, LazyCells' output)
pub type SweptBlock = (usize, Vec<u16>, usize, u32, Vec<u8>);
/// Every block LazyCells admits with at least [`SWEEP_MIN_ROWS`] rows, in decode order
pub static SWEPT: std::sync::Mutex<Vec<SweptBlock>> = std::sync::Mutex::new(Vec::new());
/// The sweep's row floor
pub const SWEEP_MIN_ROWS: usize = 65_536;

/// The thin-run limit sweep's collector (research only): LazyCells as shipped, keeping each
/// run-route block it admits with at least [`SWEEP_MIN_ROWS`] rows, with its output, for
/// `ab limit-sweep` to time the open kernel on afterwards at several limits
#[derive(Default)]
pub struct SweepCollect {
    lazy: LazyCells,
    block: usize,
}

impl InverseBwt for SweepCollect {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.block += 1;
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
        self.block += 1;
        self.lazy
            .invert_checkpointed(ll, counts, checkpoints, p, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], p: u32) -> bool {
        // (a refusal goes on to `invert`, which counts the block)
        let took = self.lazy.invert_in_place(ll, counts, p);
        self.block += took as usize;
        took
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let before = self.lazy.counts().admitted;
        let took = self.lazy.invert_runs(r, n, p, out, k);
        if took && self.lazy.counts().admitted > before && n >= SWEEP_MIN_ROWS {
            SWEPT
                .lock()
                .unwrap()
                .push((self.block, r.records().to_vec(), n, p, out.clone()));
        }
        self.block += 1;
        took
    }
}
