//! Research, opt-in (the `run-fed` feature): the pipelined team with a run-fed leader
//! (docs/journal/2026-11-09-run-fed-leader.md). Nothing in the default decoder calls it,
//! with or without the feature: only `StreamDecoder::pipelined_run_fed` builds it.
//!
//! It is the shipped pipelined team (`parallel::pipelined_worker` at `PIPELINED_LEVERS`:
//! park and small) with its leader's move-to-front and its helpers' build changed, and
//! nothing else: the same runs, hand-offs, walk, RLE1 and CRC.
//! - **The leader** runs `mtf::mtf_runs` (run records and counts, no last column and no
//!   count checkpoints) instead of `mtf::mtf_checkpoints`, on every block `run_stage`'s
//!   rule (`takes_runs`, #269's) sends to records. Every other block takes today's path:
//!   `mtf_checkpoints`, then `coop::FusedBuild` on the helpers.
//! - **The sort** into F order, by `RunFedSort`:
//!   - `Leader`: on the leader, `run_stage::sort`, right after the emitter;
//!   - `Helpers`: on the helpers, each counting its share of the records by byte, a prefix
//!     over the shares and the bytes (the last helper to count), each scattering its share
//!     into its own slots, then a check that every slot was written;
//!   - `Control`: no records at all: every block takes today's path, through this module's
//!     own worker and timers, so the instrumented worker can be compared with the shipped.
//! - **The helpers** fill their row shares of the table from the F-order records
//!   (`fill_share`: `run_stage::fill_rows`'s sweep, each share finding its iterators by
//!   binary search, into its own rows only), then walk exactly as today.
//!
//! The checks: the records' tilings are checked before any fill (by `run_stage::sort` on
//! the leader, or by the helpers' merge and scatter checks, which are its checks split by
//! share); the table's length is set only once every share filled its rows exactly. A
//! refused tiling, or a share that failed its check, is a bug (in the emitter or here), and
//! the block is rebuilt from its symbols by the column's path, so the output is right
//! whatever (counted: the phases' `fallbacks`, 0 in every valid run).
//!
//! Every call into `run_stage` is in `api` below, so a change to that module's API (#269's
//! fix) touches only it.

use super::{
    finish_and_publish, finish_block, min_coop, panic_message, publish, take_out, try_job,
    Inverted, Job, Lead, Shared, Solo, Staged, Staging, StreamDecoder,
};
#[cfg(test)]
use super::{inject, Fault};
use crate::block::Symbols;
use crate::bwt::{self, SegScratch, TVector, FLAGGED_LANES, FLAGGED_SEGMENTS};
use crate::coop::{FusedBuild, Pool, RoomWalk, BARRIER_SPINS, MAX_SHARES};
use crate::{mtf, ErrorKind, Kernels};
use std::any::Any;
use std::mem::MaybeUninit;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, MutexGuard};
use std::thread::{self, Thread};
use std::time::Instant;

/// Where the run-fed team sorts its records (`StreamDecoder::pipelined_run_fed`)
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunFedSort {
    /// no records: today's path through this module's worker (the instrumented control)
    Control,
    /// the sort on the leader, after the emitter
    Leader,
    /// the sort on the helpers: count, prefix, scatter (C30)
    Helpers,
    /// no records: every block's column by `mtf::mtf_checkpoints_reg`, the register-list
    /// kernel `mtf_runs` shares, then today's build (the matched control, C29)
    RegControl,
    /// no F-order array (C31): the helpers count their shares' rows a byte, a prefix, then
    /// each fills its runs' F intervals directly, reading L by predecessor lookup; its
    /// record shares cut by estimated work, rows plus `RECORD_WEIGHT` a record (C35,
    /// `row_cuts`), not by equal records
    Direct,
}

const CTL: u8 = 0;
const LS: u8 = 1;
const HS: u8 = 2;
const CTLR: u8 = 3;
const DIR: u8 = 4;

/// Every call into `run_stage`, in one place (#269's checked API: a `Sorted` is the only
/// way to its fills)
mod api {
    use crate::block::Symbols;
    use crate::bwt::TVector;
    use crate::run_stage::{self, Scratch};
    use crate::Kernels;

    pub(super) fn takes_runs(b: &Symbols) -> bool {
        run_stage::takes_runs(b)
    }

    /// A small block's whole table on this thread from its records: `run_stage::sort`,
    /// then its `fill` on the checked `Sorted` it returns; false (the table untouched) if
    /// the sort refused the records
    pub(super) fn build_one_core(
        recs: &[u32],
        counts: &[u32; 256],
        n: usize,
        s: &mut Scratch,
        tt: &mut TVector,
        k: Kernels,
    ) -> bool {
        match run_stage::sort(recs, counts, n, s) {
            Some(t) => {
                run_stage::fill(&t, tt, k);
                true
            }
            None => false,
        }
    }

    /// `run_stage::sort`'s F order, copied (the tests' oracle for the shares' sort)
    #[cfg(test)]
    pub(super) fn sorted_fo(
        recs: &[u32],
        counts: &[u32; 256],
        n: usize,
        s: &mut Scratch,
    ) -> Option<Vec<[u32; 2]>> {
        run_stage::sort(recs, counts, n, s).map(|t| t.fo().to_vec())
    }
}

// ---- The phases: per-process totals, for the harness (`run_fed_phases`) ----

macro_rules! phases {
    ($($name:ident: $doc:literal,)*) => {
        struct Phases {
            $($name: AtomicU64,)*
        }
        static PH: Phases = Phases {
            $($name: AtomicU64::new(0),)*
        };
        /// The run-fed teams' phases so far, in this process: each a count or nanoseconds
        /// summed over every team (research only; `examples/ab.rs` reports them)
        #[doc(hidden)]
        pub fn run_fed_phases() -> Vec<(&'static str, &'static str, u64)> {
            vec![$((stringify!($name), $doc, PH.$name.load(Ordering::Relaxed)),)*]
        }
    };
}

phases! {
    runs: "team runs",
    team_records: "team blocks by records",
    team_column: "team blocks by the column",
    small_records: "small blocks by records",
    small_column: "small blocks by the column",
    fallbacks: "blocks rebuilt from the column after a refused check (a bug: 0)",
    lead_ns: "the leader's own work in runs",
    lead_front_ns: "its MTF or emitter, in runs",
    lead_sort_ns: "its sort, in runs",
    lead_rle_ns: "its RLE1 and CRC of the block before, in runs",
    lead_small_ns: "its small blocks' build, walk, RLE1 and CRC, in runs",
    join_wait_ns: "the leader waiting for the build after its work (scheduling and wake-up included)",
    lead_start_ns: "a run's start (t0) to its leader's start (d), summed",
    lead_end_ns: "t0 to its leader's work's end (d + L), summed",
    build_end_ns: "t0 to its build's end, the walk set up (B), summed",
    run_ns: "a run's whole span",
    leader_late: "runs whose leader ended its work after the build: d + L > B, one origin",
    helper_sort_ns: "the helpers' count, prefix and scatter (summed, waits included)",
    helper_fill_ns: "the helpers' fills from records (summed)",
    helper_fused_ns: "the helpers' column builds (summed)",
    drained_front_ns: "the MTF or emitter and sort with no run in flight",
    records: "team blocks' run records (r, summed)",
    rows: "team blocks' rows by records (n, summed)",
    stores: "table entries the record fills stored (W, summed: n <= W)",
    lookups: "the direct fill's predecessor lookups (C31)",
    cells: "the record fills' cells (m, summed: one store loop an L-run and F-interval piece)",
    shares: "the record fills' shares (summed over team runs by records)",
    share_records_max: "a run's largest share's records (r_h; summed over runs)",
    share_rows_max: "a run's largest share's rows (n_h, its distinct words written)",
    share_cells_max: "a run's largest share's cells (m_h)",
    share_stores_max: "a run's largest share's entries stored (W_h)",
    ready_last_ns: "t0 to a run's last share's fill end (summed)",
    ready_skew_ns: "a run's last share's fill end less its first's (summed: the readiness skew)",
    rule_blocks: "blocks the records' rule takes, staged in runs (in every arm, the controls too)",
    rule_front_ns: "their MTF or emitter, in runs (the release's like-for-like blocks)",
    rule_sort_ns: "their sort on the leader, in runs",
}

fn add(c: &AtomicU64, v: u64) {
    c.fetch_add(v, Ordering::Relaxed);
}

fn since(t: Instant) -> u64 {
    t.elapsed().as_nanos() as u64
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ---- The front set ----

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Path {
    /// `ll`, `counts` and `checkpoints` hold the block (today's path)
    Column,
    /// `recs` and `counts`, not yet sorted (the helpers sort)
    Records,
    /// `recs`, `counts` and `fo`, sorted and checked on the leader
    Sorted,
}

/// A block's front set: the column's buffers, and the records' with their F-order ones
struct Front {
    ll: Vec<u8>,
    counts: [u32; 256],
    checkpoints: Vec<[u32; 256]>,
    recs: Vec<u32>,
    /// `run_stage::sort`'s scratch (the one-core path's)
    fo: crate::run_stage::Scratch,
    /// the helpers' sort's F-order records and its shares' counts
    fo_hs: Vec<[u32; 2]>,
    hist: Vec<[[u32; 256]; 3]>,
    path: Path,
    n: usize,
}

impl Default for Front {
    fn default() -> Self {
        Front {
            ll: Vec::new(),
            counts: [0; 256],
            checkpoints: Vec::new(),
            recs: Vec::new(),
            fo: Default::default(),
            fo_hs: Vec::new(),
            hist: Vec::new(),
            path: Path::Column,
            n: 0,
        }
    }
}

/// The front stage (`parallel::front_stage`'s): the records where `MODE` takes them and
/// `takes_runs` says, else today's MTF with checkpoints; the primary index checked. Adds
/// its time to `front` and `sort` (nanoseconds).
fn front_stage<const MODE: u8>(
    b: &Symbols,
    f: &mut Front,
    k: Kernels,
    front: &mut u64,
    sort: &mut u64,
    front_rule: &mut bool,
) -> Result<usize, ErrorKind> {
    let t = Instant::now();
    let rule = api::takes_runs(b);
    *front_rule = rule;
    let n = if matches!(MODE, LS | HS | DIR) && rule {
        let n = mtf::mtf_runs(b, &mut f.recs, &mut f.counts, k)?;
        f.path = Path::Records;
        *front += since(t);
        if MODE == LS {
            let t = Instant::now();
            if sort_one_share(&f.recs, &f.counts, n, &mut f.hist, &mut f.fo_hs) {
                f.path = Path::Sorted;
                *sort += since(t);
            } else {
                // a refused tiling is a bug in the emitter: the column's path
                *sort += since(t);
                add(&PH.fallbacks, 1);
                let t = Instant::now();
                mtf::mtf_checkpoints(b, &mut f.ll, &mut f.counts, &mut f.checkpoints, k)?;
                f.path = Path::Column;
                *front += since(t);
            }
        }
        n
    } else {
        let n = if MODE == CTLR {
            mtf::mtf_checkpoints_reg(b, &mut f.ll, &mut f.counts, &mut f.checkpoints, k)?
        } else {
            mtf::mtf_checkpoints(b, &mut f.ll, &mut f.counts, &mut f.checkpoints, k)?
        };
        f.path = Path::Column;
        *front += since(t);
        n
    };
    f.n = n;
    if b.orig_ptr as usize >= n {
        return Err(ErrorKind::BadOrigPtr);
    }
    Ok(n)
}

/// A job's front stage into `f` (`parallel::stage`'s). A panic resets `f`.
fn stage<const MODE: u8>(
    shared: &Shared,
    job: Job,
    f: &mut Front,
    front: usize,
    k: Kernels,
    in_run: bool,
) -> Staging {
    let (place, bit, symbols) = job;
    let (mut tf, mut ts, mut rule) = (0, 0, false);
    let r = catch_unwind(AssertUnwindSafe(|| {
        front_stage::<MODE>(&symbols, f, k, &mut tf, &mut ts, &mut rule)
    }));
    if in_run {
        add(&PH.lead_front_ns, tf);
        add(&PH.lead_sort_ns, ts);
        if rule {
            add(&PH.rule_blocks, 1);
            add(&PH.rule_front_ns, tf);
            add(&PH.rule_sort_ns, ts);
        }
    } else {
        add(&PH.drained_front_ns, tf + ts);
    }
    let s = Staged {
        place,
        bit,
        symbols,
        front,
    };
    match r {
        Ok(Ok(n)) if n >= min_coop(shared) => Staging::Team(s),
        Ok(Ok(_)) => Staging::Small(s),
        Ok(Err(kind)) => {
            publish(shared, place, bit, Err(kind), s.symbols.syms);
            Staging::Failed
        }
        Err(p) => {
            *f = Front::default();
            let kind = ErrorKind::Internal(panic_message(p));
            publish(shared, place, bit, Err(kind), s.symbols.syms);
            Staging::Failed
        }
    }
}

/// The column's path from the symbols, for a block whose records were refused (a bug)
fn rebuild_from_column(
    b: &Symbols,
    f: &mut Front,
    tt: &mut TVector,
    k: Kernels,
) -> Result<(), ErrorKind> {
    add(&PH.fallbacks, 1);
    mtf::mtf(b, &mut f.ll, &mut f.counts, k)?;
    bwt::build(&f.ll, &f.counts, tt);
    f.path = Path::Column;
    Ok(())
}

/// A small block's table on this thread, by its front's path
fn table_one_core(
    b: &Symbols,
    f: &mut Front,
    tt: &mut TVector,
    k: Kernels,
) -> Result<(), ErrorKind> {
    match f.path {
        Path::Column => {
            add(&PH.small_column, 1);
            bwt::build(&f.ll, &f.counts, tt);
        }
        Path::Sorted => {
            add(&PH.small_records, 1);
            // (the leader's sort checked these records: `sort_one_share`)
            if !fill_whole(&f.recs, &f.fo_hs, f.n, tt, k) {
                rebuild_from_column(b, f, tt, k)?;
            }
        }
        Path::Records => {
            add(&PH.small_records, 1);
            if !api::build_one_core(&f.recs, &f.counts, f.n, &mut f.fo, tt, k) {
                rebuild_from_column(b, f, tt, k)?;
            }
        }
    }
    Ok(())
}

/// `parallel::solo` on this module's front: the table by the front's path
fn solo(shared: &Shared, f: &mut Front, sc: &mut Solo, s: Staged, k: Kernels) -> bool {
    let Some(mut out) = take_out(shared) else {
        return false;
    };
    let r = catch_unwind(AssertUnwindSafe(|| {
        table_one_core(&s.symbols, f, &mut sc.tt, k)?;
        bwt::walk_segments_room(&mut sc.tt, s.symbols.orig_ptr, &mut sc.buf, &mut sc.seg);
        finish_block(&sc.buf, s.symbols.crc, k, &mut out)
    }));
    let (res, panicked) = match r {
        Ok(Ok(())) => (Ok(out), false),
        Ok(Err(kind)) => (Err(kind), false),
        Err(p) => (Err(ErrorKind::Internal(panic_message(p))), true),
    };
    publish(shared, s.place, s.bit, res, s.symbols.syms);
    panicked
}

// ---- The share kernels (the helpers' sort and fill) ----

/// Share `j` of `h` of `len` items: [len·j/h, len·(j+1)/h)
fn share(len: usize, h: usize, j: usize) -> (usize, usize) {
    (len * j / h, len * (j + 1) / h)
}

/// The helpers' sort's first step, share [lo, hi) of the r records (`recs` holds r + 1,
/// the closing one last): into `hist`, each byte's runs ([0]) and rows ([1]). Returns
/// whether every run's L start is below the next's, the next share's first included.
fn count_share(recs: &[u32], lo: usize, hi: usize, hist: &mut [[u32; 256]; 3]) -> bool {
    *hist = [[0; 256]; 3];
    let mut ok = true;
    for w in recs[lo..=hi].windows(2) {
        let (x, next) = (w[0], w[1] >> 8);
        let (c, p) = ((x & 0xff) as usize, x >> 8);
        ok &= next > p;
        hist[0][c] += 1;
        hist[1][c] = hist[1][c].wrapping_add(next.wrapping_sub(p));
    }
    ok
}

/// The prefix over the shares' counts (`count_share`'s, in share order), each share's
/// entry made [its first slot, its slots' end, its first F row] a byte, and `run_stage::
/// sort`'s other checks: the first run starts at 0, the closing record is n, each byte's
/// runs' rows sum to its count, the counts to n, and the slots to r. The slots' ranges, a
/// byte and a share, are disjoint and tile [0, r) by construction.
fn merge_shares(
    recs: &[u32],
    counts: &[u32; 256],
    n: usize,
    hists: &mut [[[u32; 256]; 3]],
) -> bool {
    if recs.len() < 2 || n >= 1 << 24 {
        return false;
    }
    let r = recs.len() - 1;
    if recs[0] >> 8 != 0 || recs[r] >> 8 != n as u32 {
        return false;
    }
    let (mut slot, mut start) = (0u64, 0u64);
    for c in 0..256 {
        let mut a = start;
        for h in hists.iter_mut() {
            let (runs, rows) = (h[0][c] as u64, h[1][c] as u64);
            h[0][c] = slot as u32;
            slot += runs;
            h[2][c] = slot as u32;
            h[1][c] = a as u32;
            a += rows;
        }
        if a - start != counts[c] as u64 {
            return false;
        }
        start = a;
    }
    slot == r as u64 && start == n as u64
}

/// The helpers' sort's scatter, share [lo, hi) of the records into `fo`: each run's
/// `[a, p]` at its byte's next slot of this share's range ([hist[0][c], hist[2][c])), its a
/// advanced by its length. Returns whether every range was filled exactly; writes nothing
/// outside them, whatever the records hold.
///
/// SAFETY: `hist` is `merge_shares`'s entry for this share, and `fo` is valid for writes
/// at every slot of its ranges, which no other thread writes or reads meanwhile
unsafe fn scatter_share(
    recs: &[u32],
    lo: usize,
    hi: usize,
    hist: &mut [[u32; 256]; 3],
    fo: *mut [u32; 2],
) -> bool {
    for w in recs[lo..=hi].windows(2) {
        let (x, next) = (w[0], w[1] >> 8);
        let (c, p) = ((x & 0xff) as usize, x >> 8);
        let s = hist[0][c];
        if s >= hist[2][c] {
            return false;
        }
        // SAFETY: s is inside this share's range of byte c (checked just above)
        unsafe { fo.add(s as usize).write([hist[1][c], p]) };
        hist[0][c] = s + 1;
        hist[1][c] = hist[1][c].wrapping_add(next.wrapping_sub(p));
    }
    hist[0] == hist[2]
}

/// A fill's counts (the census): C31's lookups, the entries stored (W, repeats counted),
/// the cells (m: `cell` calls) and the rows written (n: the distinct words, every one inside
/// the rows the thread owns)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Tally {
    lookups: u64,
    stores: u64,
    cells: u64,
    rows: u64,
}

impl Tally {
    /// One cell's: rows [q, stop), `stored` entries stored
    #[inline(always)]
    fn cell(&mut self, q: u32, stop: u32, stored: u64) {
        self.stores += stored;
        self.cells += 1;
        self.rows += (stop - q) as u64;
    }
}

/// One cell, rows [q, stop): `T[q + i] = base + 256·i`; with `AVX`, 8 entries a store
/// while a store ends at or before `end` (the rows this thread owns), then single entries.
/// A store past `stop` writes rows of this thread's next cell, which that cell writes
/// again (C32: never past `end`, another thread's). Returns the entries stored.
///
/// **The physical write footprint (Guardian D044).** Every word this writes is in
/// [q, end): w starts at q and only rises; a vector store covers [w, w + 8) and is issued
/// only while w + 8 <= end; a scalar one writes w < stop <= end. So the tail at an
/// ownership edge is exact (scalar), not an 8-word overstore: a share of one row (or any
/// range shorter than 8) gets no vector store at all, and no store reaches the table's
/// n-th row (no n + 8 slack is relied on). The callers pass as `end` the end of the rows
/// the thread owns: `fill_rows_at` its share's q1; `fill_direct_at` the end of byte c's
/// F range in its share (`merge_rows`' hist[2][c]), checked to contain the run before
/// any store. Those ranges are disjoint across threads by the prefix's construction, so
/// no store from one share's fill lands in another's rows. Inside its own range, a vector
/// store's words past `stop` are rows the same thread writes again later, because it
/// fills that range in rising order to its end (checked: a fill that stops short
/// returns false, and the table is then never exposed). Tests: `fills_write_their_own_
/// rows_only` (each share alone into a table of sentinels: exactly its rows changed, its
/// distinct words its rows), `miri_parallel_footprint` (every share at once, one-row and
/// one-record shares among them, under Miri's race detector with AVX2).
///
/// SAFETY: q <= stop <= end, and rows [q, end) of `t` are valid for writes and written by
/// no other thread meanwhile (with `AVX`, AVX2 is available)
#[inline(always)]
unsafe fn cell<const AVX: bool>(t: *mut u32, q: u32, stop: u32, end: u32, base: u32) -> u64 {
    let mut w = q;
    let mut stored = 0u64;
    #[cfg(target_arch = "x86_64")]
    if AVX {
        use std::arch::x86_64::*;
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut v = _mm256_add_epi32(_mm256_set1_epi32(base as i32), lanes);
        while w < stop && w + 8 <= end {
            // SAFETY: rows [w, w + 8) are inside [q, end)
            unsafe { _mm256_storeu_si256(t.add(w as usize) as *mut __m256i, v) };
            v = _mm256_add_epi32(v, step);
            w += 8;
            stored += 8;
        }
    }
    while w < stop {
        // SAFETY: w is inside [q, stop)
        unsafe { t.add(w as usize).write(base + ((w - q) << 8)) };
        w += 1;
        stored += 1;
    }
    stored
}

/// Rows [q0, q1) of the table at `t` from L-order records and F-order ones whose tilings
/// were checked: `run_stage::fill_rows`'s sweep, its iterators found by binary search, cut
/// where either order's run ends. Returns whether every row was written (false only for
/// records that don't tile, which the checks before it refuse); adds its counts to `tally`
/// (C32's W among them). A bad index panics.
///
/// SAFETY: rows [q0, q1) of `t` are valid for writes and this thread's alone (with
/// `AVX`, AVX2 is available)
#[inline(always)]
unsafe fn fill_rows_at<const AVX: bool>(
    recs: &[u32],
    fo: &[[u32; 2]],
    q0: u32,
    q1: u32,
    t: *mut u32,
    tally: &mut Tally,
) -> bool {
    if q0 >= q1 {
        return true;
    }
    let (Some(mut f), Some(mut l)) = (
        fo.partition_point(|e| e[0] <= q0).checked_sub(1),
        recs.partition_point(|&x| x >> 8 <= q0).checked_sub(1),
    ) else {
        return false;
    };
    let mut q = q0;
    while q < q1 {
        let [af, pf] = fo[f];
        let fe = fo[f + 1][0];
        let (x, le) = (recs[l], recs[l + 1] >> 8);
        let end = fe.min(le);
        if end <= q || q < af {
            return false;
        }
        let base = (x & 0xff) | (pf + (q - af)) << 8;
        let stop = end.min(q1);
        // SAFETY: q < stop <= q1, the rows this thread owns
        tally.cell(q, stop, unsafe { cell::<AVX>(t, q, stop, q1, base) });
        f += (end == fe) as usize;
        l += (end == le) as usize;
        q = stop;
    }
    true
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_rows_avx2(
    recs: &[u32],
    fo: &[[u32; 2]],
    q0: u32,
    q1: u32,
    t: *mut u32,
    tally: &mut Tally,
) -> bool {
    // SAFETY: the caller's contract
    unsafe { fill_rows_at::<true>(recs, fo, q0, q1, t, tally) }
}

/// `fill_rows_at` on the kernels given: rows [q0, q1) into `out`, its q1 − q0 slots
fn fill_share(
    recs: &[u32],
    fo: &[[u32; 2]],
    q0: u32,
    q1: u32,
    out: &mut [MaybeUninit<u32>],
    k: Kernels,
    tally: &mut Tally,
) -> bool {
    assert_eq!(out.len(), q1.saturating_sub(q0) as usize);
    // (the rows' pointer, offset back by q0: only rows [q0, q1) are written through it)
    let t = (out.as_mut_ptr() as *mut u32).wrapping_sub(q0 as usize);
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected; rows [q0, q1) of t are
            // `out`'s, held mutably here
            return unsafe { fill_rows_avx2(recs, fo, q0, q1, t, tally) };
        }
    }
    let _ = k;
    // SAFETY: as above
    unsafe { fill_rows_at::<false>(recs, fo, q0, q1, t, tally) }
}

/// The leader's sort (`RunFedSort::Leader`): the helpers' sort as one share, into `fo`
/// (`r + 1` entries, the closing `[n, 0]` last), with all of `run_stage::sort`'s checks;
/// false, `fo` emptied, unless both tilings hold. The same kernels as the helpers', so the
/// two sub-arms differ in where they run alone.
fn sort_one_share(
    recs: &[u32],
    counts: &[u32; 256],
    n: usize,
    hist: &mut Vec<[[u32; 256]; 3]>,
    fo: &mut Vec<[u32; 2]>,
) -> bool {
    fo.clear();
    hist.clear();
    hist.push([[0; 256]; 3]);
    let Some(r) = recs.len().checked_sub(1) else {
        return false;
    };
    if !(count_share(recs, 0, r, &mut hist[0]) && merge_shares(recs, counts, n, hist)) {
        return false;
    }
    fo.reserve(r + 1);
    // SAFETY: merge_shares' ranges for the one share tile [0, r), inside the r + 1 reserved
    if !unsafe { scatter_share(recs, 0, r, &mut hist[0], fo.as_mut_ptr()) } {
        return false;
    }
    // SAFETY: every slot below r written once (each range filled exactly), and r here
    unsafe {
        fo.as_mut_ptr().add(r).write([n as u32, 0]);
        fo.set_len(r + 1);
    }
    true
}

/// The whole table on this thread from checked records (`sort_one_share`'s): false, the
/// table empty, if the fill found they don't tile
fn fill_whole(recs: &[u32], fo: &[[u32; 2]], n: usize, tt: &mut TVector, k: Kernels) -> bool {
    tt.0.clear();
    tt.0.reserve(n);
    let ok = fill_share(
        recs,
        fo,
        0,
        n as u32,
        &mut tt.0.spare_capacity_mut()[..n],
        k,
        &mut Tally::default(),
    );
    if ok {
        // SAFETY: every row of [0, n) written (the fill's check)
        unsafe { tt.0.set_len(n) };
    }
    ok
}

/// C31's first step (no F-order array): share [lo, hi) of the records' rows a byte, into
/// `hist[1]` (no run counts). Returns whether every run's L start is below the next's.
fn count_rows(recs: &[u32], lo: usize, hi: usize, hist: &mut [[u32; 256]; 3]) -> bool {
    hist[1] = [0; 256];
    let mut ok = true;
    for w in recs[lo..=hi].windows(2) {
        let (x, next) = (w[0], w[1] >> 8);
        let p = x >> 8;
        ok &= next > p;
        let c = (x & 0xff) as usize;
        hist[1][c] = hist[1][c].wrapping_add(next.wrapping_sub(p));
    }
    ok
}

/// C31's prefix: each share's entry made [_, its first F row, its rows' end] a byte, and
/// `run_stage::sort`'s checks but the run counts' (none are kept). Byte c's rows of share
/// j are the F rows [C(c) + the rows of c in the shares before j, + its own): disjoint, a
/// byte and a share, and tiling [0, n) once the rows sum to the counts and they to n.
fn merge_rows(recs: &[u32], counts: &[u32; 256], n: usize, hists: &mut [[[u32; 256]; 3]]) -> bool {
    if recs.len() < 2 || n >= 1 << 24 {
        return false;
    }
    let r = recs.len() - 1;
    if recs[0] >> 8 != 0 || recs[r] >> 8 != n as u32 {
        return false;
    }
    let mut start = 0u64;
    for c in 0..256 {
        let mut a = start;
        for h in hists.iter_mut() {
            let rows = h[1][c] as u64;
            h[1][c] = a as u32;
            a += rows;
            h[2][c] = a as u32;
        }
        if a - start != counts[c] as u64 {
            return false;
        }
        start = a;
    }
    start == n as u64
}

/// C35's record weight: a share's work estimated as its rows plus this a record, so the cut
/// equalises the entries stored (W ≈ n + about 7.6 r on the bench corpus: each cell's
/// vector store rounds up to 8, and a record makes about 1.2 cells), not records alone
/// (C31 as first built) nor rows alone. An estimate from an untimed census, not a
/// measured cost: the registration reports the shares' census and readiness to check it.
const RECORD_WEIGHT: u64 = 8;

/// C35's partition for C31: the records cut into `shares` contiguous ranges of about equal
/// work, rows + `weight` a record: cut j the first record i whose key, its L start plus
/// weight · i, is at least the total's (n + weight · r) j/shares share. The key rises with
/// i on records that tile, so a binary search finds it (O(shares · log r), on the thread
/// that sets the run up). The cuts rise, cuts[0] is 0 and cuts[shares] is r whatever the
/// records hold (each search starts at the cut before), so they are a contiguous
/// partition of [0, r) for `merge_rows`' byte-then-share prefix, whose ownership argument
/// holds for any such partition; records that don't rise are refused by `count_rows`
/// before any fill. With weight 0 it is the equal-rows cut.
fn row_cuts(recs: &[u32], n: usize, shares: usize, weight: u64, cuts: &mut [usize]) {
    let r = recs.len().saturating_sub(1);
    let total = n as u64 + weight * r as u64;
    cuts[0] = 0;
    let mut i = 0;
    for (j, cut) in cuts.iter_mut().enumerate().take(shares).skip(1) {
        let target = total * j as u64 / shares as u64;
        let key = |k: usize| (recs[k] >> 8) as u64 + weight * k as u64;
        // (a binary search over [i, r) for the first key at least the target)
        let (mut lo, mut hi) = (i, r);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if key(mid) < target {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        i = lo;
        *cut = i;
    }
    cuts[shares] = r;
}

/// C31's fill: share [lo, hi) of the records, each run's F interval [a, a + ℓ) written
/// directly (a from `merge_rows`' cursor, advanced by ℓ), its bytes L[q] found by one
/// predecessor lookup a byte (the first of its runs in the share: the byte's F rows in the
/// share are consecutive, so the L rows they read are too) and then by walking the L runs
/// on. Writes only inside each byte's range ([hist[1][c], hist[2][c]), the share's own);
/// returns whether every range was filled exactly, and adds its counts to `tally`.
///
/// SAFETY: `hist` is `merge_rows`' entry for this share, `t` valid for writes at every row
/// of its ranges, which no other thread writes meanwhile (with `AVX`, AVX2 is available)
#[inline(always)]
unsafe fn fill_direct_at<const AVX: bool>(
    recs: &[u32],
    lo: usize,
    hi: usize,
    hist: &mut [[u32; 256]; 3],
    t: *mut u32,
    tally: &mut Tally,
) -> bool {
    let mut lcur = [u32::MAX; 256];
    for i in lo..hi {
        let (x, next) = (recs[i], recs[i + 1] >> 8);
        let (c, p) = ((x & 0xff) as usize, x >> 8);
        let a = hist[1][c];
        let end = hist[2][c];
        let Some(len) = next.checked_sub(p) else {
            return false;
        };
        if len > end.saturating_sub(a) {
            return false;
        }
        let mut l = if lcur[c] == u32::MAX {
            tally.lookups += 1;
            match recs.partition_point(|&y| y >> 8 <= a).checked_sub(1) {
                Some(l) => l,
                None => return false,
            }
        } else {
            lcur[c] as usize
        };
        let (mut q, qe) = (a, a + len);
        while q < qe {
            let (y, le) = (recs[l], recs[l + 1] >> 8);
            if le <= q || y >> 8 > q {
                return false;
            }
            let stop = le.min(qe);
            let base = (y & 0xff) | (p + (q - a)) << 8;
            // SAFETY: a <= q < stop <= qe <= end: this share's range of byte c
            tally.cell(q, stop, unsafe { cell::<AVX>(t, q, stop, end, base) });
            l += (stop == le) as usize;
            q = stop;
        }
        lcur[c] = l as u32;
        hist[1][c] = qe;
    }
    hist[1] == hist[2]
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_direct_avx2(
    recs: &[u32],
    lo: usize,
    hi: usize,
    hist: &mut [[u32; 256]; 3],
    t: *mut u32,
    tally: &mut Tally,
) -> bool {
    // SAFETY: the caller's contract
    unsafe { fill_direct_at::<true>(recs, lo, hi, hist, t, tally) }
}

/// `fill_direct_at` on the kernels given
///
/// SAFETY: as `fill_direct_at`'s, but for AVX2's availability, which `k` says
unsafe fn fill_direct(
    recs: &[u32],
    lo: usize,
    hi: usize,
    hist: &mut [[u32; 256]; 3],
    t: *mut u32,
    k: Kernels,
    tally: &mut Tally,
) -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: the caller's, and AVX2 was detected
            return unsafe { fill_direct_avx2(recs, lo, hi, hist, t, tally) };
        }
    }
    let _ = k;
    // SAFETY: the caller's
    unsafe { fill_direct_at::<false>(recs, lo, hi, hist, t, tally) }
}

// ---- A run of the team ----

/// A one-shot gate for the helpers: each counts itself in (`pass`); the last runs its
/// step, then opens it; the others wait for it, spinning `BARRIER_SPINS` and then parking
/// (lever A's wait, `coop::Relay::wait_walk_ready`, whose argument it shares: the waiter
/// lists itself under the lock, then loads the flag; the opener stores the flag, then
/// unparks everyone listed, so no wake-up is lost). Each count's AcqRel and the flag's
/// release and acquire carry every thread's writes before its count to every thread past
/// the gate.
struct Gate {
    arrived: AtomicUsize,
    open: AtomicBool,
    sleepers: Mutex<Vec<Thread>>,
}

impl Gate {
    fn new() -> Self {
        Gate {
            arrived: AtomicUsize::new(0),
            open: AtomicBool::new(false),
            sleepers: Mutex::new(Vec::new()),
        }
    }

    fn wait(&self) {
        let ready = || self.open.load(Ordering::Acquire);
        for _ in 0..BARRIER_SPINS {
            if ready() {
                return;
            }
            std::hint::spin_loop();
        }
        lock(&self.sleepers).push(thread::current());
        while !ready() {
            thread::park();
        }
    }

    fn set(&self) {
        self.open.store(true, Ordering::Release);
        for t in lock(&self.sleepers).drain(..) {
            t.unpark();
        }
    }

    /// Count this thread in, of `of`; the last runs `last` (which must not unwind), then
    /// opens the gate; every other waits until it is open
    fn pass(&self, of: usize, last: impl FnOnce()) {
        if self.arrived.fetch_add(1, Ordering::AcqRel) + 1 == of {
            last();
            self.set();
        } else {
            self.wait();
        }
    }
}

/// A panic injected in a helper's phase, for the tests (0: none)
const INJECT_COUNT: u8 = 1;
const INJECT_MERGE: u8 = 2;
const INJECT_SCATTER: u8 = 3;
const INJECT_FILL: u8 = 4;
/// the leader's start delayed (`LEAD_DELAY`), for the common-origin timeline's test
const INJECT_LEAD_DELAY: u8 = 5;
const LEAD_DELAY: std::time::Duration = std::time::Duration::from_millis(200);

/// A run's moments, each in nanoseconds from the same origin, the run's start t0 (before
/// its build is set up and the pool dispatched): the leader's start (d) and its work's end
/// (d + L), and the build's end, its walk set up (B). The leader ended after the build iff
/// d + L > B: comparing its duration L with B, as before, missed every run with
/// L <= B < d + L (the Optimiser's and the Researcher's finding).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Timeline {
    lead_start: u64,
    lead_end: u64,
    build_end: u64,
}

impl Timeline {
    fn leader_late(&self) -> bool {
        self.lead_end > self.build_end
    }
}

/// The run-fed build's shared state: the records and their F order, the table being
/// filled by shares, and the helpers' sort's gates
struct RunFill<'a> {
    recs: &'a [u32],
    counts: &'a [u32; 256],
    n: usize,
    r: usize,
    shares: usize,
    /// the F-order records, r + 1: the leader's (sorted, read only) or the helpers' to
    /// scatter (`fo_hs`'s buffer, reserved)
    fo: *mut [u32; 2],
    /// the shares' counts (the helpers' sort: `Front::hist`'s, `shares` of them)
    hist: *mut [[u32; 256]; 3],
    tt_p: *mut u32,
    tt: &'a mut TVector,
    _bufs: std::marker::PhantomData<&'a mut ()>,
    counted: Gate,
    scattered_gate: Gate,
    /// a share's records didn't rise
    unrisen: AtomicBool,
    /// the F order is whole and checked: from the start (the leader's sort), or once the
    /// merge and every scatter passed
    sorted: AtomicBool,
    scattered: AtomicU64,
    claimed: AtomicU64,
    filled: AtomicU64,
    /// C31: no F-order array (`count_rows`, `merge_rows`, `fill_direct`)
    direct: bool,
    /// C31's record shares, by rows (C35, `row_cuts`): share j is [cuts[j], cuts[j + 1])
    cuts: [usize; MAX_SHARES + 1],
    /// the run's start, the census's readiness origin (`Relay::t0`)
    t0: Instant,
    /// each share's census: its records, rows, cells, entries stored, and its fill's end
    /// from t0; written by that share alone, read after the built gate
    census: [[AtomicU64; 5]; MAX_SHARES],
    inject: u8,
}
// SAFETY: the raw pointers are into buffers the run holds (`_bufs`, `tt`): each share
// writes only its own hist entry, its own slots of `fo` (`scatter_share`) and its own rows
// of the table (`fill`); every read of another's writes is after a gate or the build's end
unsafe impl Sync for RunFill<'_> {}

impl<'a> RunFill<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        recs: &'a [u32],
        counts: &'a [u32; 256],
        n: usize,
        presorted: bool,
        fo_hs: &'a mut Vec<[u32; 2]>,
        hist: &'a mut Vec<[[u32; 256]; 3]>,
        shares: usize,
        tt: &'a mut TVector,
        direct: bool,
        t0: Instant,
        inject: u8,
    ) -> RunFill<'a> {
        // (`invert`'s: 1 to MAX_SHARES shares, a bit each in `scattered`, `claimed`, `filled`)
        debug_assert!((1..=MAX_SHARES).contains(&shares));
        let r = recs.len().saturating_sub(1);
        // the leader's sorted F order (read only), or the helpers' to scatter into
        let sorted = presorted && fo_hs.len() == r + 1;
        if !presorted {
            fo_hs.clear();
            if !direct {
                fo_hs.reserve(r + 1);
            }
        }
        let fo = fo_hs.as_mut_ptr();
        hist.clear();
        hist.resize(shares, [[0; 256]; 3]);
        tt.0.clear();
        tt.0.reserve(n);
        let mut cuts = [0; MAX_SHARES + 1];
        if direct {
            row_cuts(recs, n, shares, RECORD_WEIGHT, &mut cuts);
        }
        RunFill {
            recs,
            counts,
            n,
            r,
            shares,
            fo,
            hist: hist.as_mut_ptr(),
            tt_p: tt.0.as_mut_ptr(),
            tt,
            _bufs: std::marker::PhantomData,
            counted: Gate::new(),
            scattered_gate: Gate::new(),
            unrisen: AtomicBool::new(false),
            sorted: AtomicBool::new(sorted),
            scattered: AtomicU64::new(0),
            claimed: AtomicU64::new(0),
            filled: AtomicU64::new(0),
            direct,
            cuts,
            t0,
            census: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            inject,
        }
    }

    /// Whether helper j has a share: a team of more than `MAX_SHARES` helpers has helpers
    /// past the shares (`invert`'s `shares` is `helpers.min(MAX_SHARES)`), and theirs is no
    /// share's work. Each share phase (`count`, `scatter`, `fill`) asks it first, before any
    /// index into the shares' entries (`hist`, `cuts`, `census`) or bits (`1 << j`), as
    /// `FusedBuild::share` returns for `j >= shares`; a surplus helper still passes every
    /// gate (`Relay::build_share` calls the phases and the gates for every helper), so the
    /// gates' counts, of all the helpers, stay whole (Guardian D049, B6).
    fn owns(&self, j: usize) -> bool {
        j < self.shares
    }

    /// Share j's records: C35's row cuts for C31, else equal records (j a share: `owns`)
    fn records_of(&self, j: usize) -> (usize, usize) {
        debug_assert!(self.owns(j));
        if self.direct {
            (self.cuts[j], self.cuts[j + 1])
        } else {
            share(self.r, self.shares, j)
        }
    }

    /// Share j's census, once its fill has ended
    fn tally(&self, j: usize, records: usize, t: &Tally) {
        let c = &self.census[j];
        for (i, v) in [records as u64, t.rows, t.cells, t.stores, since(self.t0)]
            .into_iter()
            .enumerate()
        {
            c[i].store(v, Ordering::Relaxed);
        }
    }

    /// The run's census into the phases: each count's largest share, the readiness skew
    /// (on the build's last helper, past the built gate, every share's census written)
    fn add_census(&self) {
        let get = |j: usize, i: usize| self.census[j][i].load(Ordering::Relaxed);
        let js = 0..self.shares;
        let max = |i: usize| js.clone().map(|j| get(j, i)).max().unwrap_or(0);
        let first = js.clone().map(|j| get(j, 4)).min().unwrap_or(0);
        add(&PH.shares, self.shares as u64);
        add(&PH.share_records_max, max(0));
        add(&PH.share_rows_max, max(1));
        add(&PH.share_cells_max, max(2));
        add(&PH.share_stores_max, max(3));
        add(&PH.ready_last_ns, max(4));
        add(&PH.ready_skew_ns, max(4) - first);
    }

    fn all(&self) -> u64 {
        if self.shares == 64 {
            u64::MAX
        } else {
            (1u64 << self.shares) - 1
        }
    }

    fn injected(&self, at: u8, j: usize) {
        if self.inject == at && j == 0 {
            panic!("injected");
        }
    }

    /// Share j's count (the helpers' sort); nothing for a helper past the shares
    fn count(&self, j: usize) {
        if !self.owns(j) {
            return;
        }
        self.injected(INJECT_COUNT, j);
        let (lo, hi) = self.records_of(j);
        // SAFETY: j < shares (`owns`, above), so entry j is one of the `shares` allocated by
        // `new`, and this share's alone until the counted gate
        let h = unsafe { &mut *self.hist.add(j) };
        let ok = if self.direct {
            count_rows(self.recs, lo, hi, h)
        } else {
            count_share(self.recs, lo, hi, h)
        };
        if !ok {
            self.unrisen.store(true, Ordering::Relaxed);
        }
    }

    /// The prefix, on the last helper to count (after the counted gate's acquire)
    fn merge(&self) {
        self.injected(INJECT_MERGE, 0);
        // SAFETY: every share has counted (the gate) and none touches the entries until
        // the gate opens
        let hists = unsafe { std::slice::from_raw_parts_mut(self.hist, self.shares) };
        if self.unrisen.load(Ordering::Relaxed) {
            return;
        }
        if self.direct {
            if merge_rows(self.recs, self.counts, self.n, hists) {
                self.sorted.store(true, Ordering::Relaxed);
            }
        } else if merge_shares(self.recs, self.counts, self.n, hists) {
            // SAFETY: r + 1 slots were reserved; slot r is no share's (theirs tile [0, r))
            unsafe { self.fo.add(self.r).write([self.n as u32, 0]) };
            self.sorted.store(true, Ordering::Relaxed);
        }
    }

    /// Share j's scatter, if the merge passed; nothing for a helper past the shares
    fn scatter(&self, j: usize) {
        if !self.owns(j) || !self.sorted.load(Ordering::Relaxed) {
            return;
        }
        self.injected(INJECT_SCATTER, j);
        let (lo, hi) = self.records_of(j);
        // SAFETY: entry j (j < shares: `owns`) is this share's; its ranges are merge_shares', disjoint from
        // every other share's and inside the r + 1 slots reserved
        let ok = unsafe { scatter_share(self.recs, lo, hi, &mut *self.hist.add(j), self.fo) };
        if ok {
            self.scattered.fetch_or(1 << j, Ordering::Relaxed);
        }
    }

    /// On the last helper to scatter: the F order whole only if every share filled its
    /// ranges exactly (so every slot below r written once, and r by the merge)
    fn check_scattered(&self) {
        if self.scattered.load(Ordering::Relaxed) != self.all() {
            self.sorted.store(false, Ordering::Relaxed);
        }
    }

    /// Share j's rows of the table from the F order, if it is whole and checked; nothing
    /// for a helper past the shares
    fn fill(&self, j: usize, k: Kernels) {
        if !self.owns(j) || !self.sorted.load(Ordering::Relaxed) {
            return;
        }
        self.injected(INJECT_FILL, j);
        let bit = 1u64 << j;
        if self.claimed.fetch_or(bit, Ordering::Relaxed) & bit != 0 {
            return;
        }
        let mut tally = Tally::default();
        if self.direct {
            let (lo, hi) = self.records_of(j);
            // SAFETY: entry j (j < shares: `owns`) is this share's, merge_rows': its ranges are disjoint from
            // every other share's and inside [0, n), the n rows reserved, the buffer held
            let ok = unsafe {
                fill_direct(
                    self.recs,
                    lo,
                    hi,
                    &mut *self.hist.add(j),
                    self.tt_p,
                    k,
                    &mut tally,
                )
            };
            add(&PH.lookups, tally.lookups);
            add(&PH.stores, tally.stores);
            add(&PH.cells, tally.cells);
            self.tally(j, hi - lo, &tally);
            if ok {
                self.filled.fetch_or(bit, Ordering::Release);
            }
            return;
        }
        let (q0, q1) = share(self.n, self.shares, j);
        // SAFETY: all r + 1 slots are initialised (the leader's sorted Vec, or every share's
        // ranges filled and slot r by the merge, published by the scattered gate), and
        // nothing writes them now
        let fo = unsafe { std::slice::from_raw_parts(self.fo as *const [u32; 2], self.r + 1) };
        // SAFETY: rows [q0, q1) are this share's alone (claimed above), inside the n
        // reserved, and the table's buffer is held, not moved, until `finish`
        let out = unsafe {
            std::slice::from_raw_parts_mut(self.tt_p.add(q0) as *mut MaybeUninit<u32>, q1 - q0)
        };
        let ok = fill_share(self.recs, fo, q0 as u32, q1 as u32, out, k, &mut tally);
        add(&PH.stores, tally.stores);
        add(&PH.cells, tally.cells);
        let (lo, hi) = self.records_of(j);
        self.tally(j, hi - lo, &tally);
        if ok {
            self.filled.fetch_or(bit, Ordering::Release);
        }
    }

    /// The table, its length set only if every share filled its rows; else the block
    /// rebuilt by the column's path from its symbols (a bug's path, counted)
    fn finish(self, b: &Symbols, k: Kernels) -> Result<&'a mut TVector, ErrorKind> {
        let ok = self.sorted.load(Ordering::Relaxed)
            && self.filled.load(Ordering::Acquire) == self.all();
        if ok {
            self.add_census();
        }
        let tt = self.tt;
        if ok {
            // SAFETY: every share wrote every row of its range (its check), and the ranges
            // tile [0, n); their writes were released by `filled` and the built gate
            unsafe { tt.0.set_len(self.n) };
            return Ok(tt);
        }
        let mut f = Front::default();
        rebuild_from_column(b, &mut f, tt, k)?;
        Ok(tt)
    }
}

/// Where a run is (`coop::Stage`, with the run-fed build)
#[allow(clippy::large_enum_variant)]
enum Stage<'a> {
    Fused(Result<FusedBuild<'a>, &'a mut TVector>, &'a mut SegScratch),
    Runs(RunFill<'a>, &'a mut SegScratch),
    Walk(RoomWalk<'a>),
    Over,
}

/// A run's shared state (`coop::Relay`'s protocol, its build's waits lever A's)
struct Relay<'a> {
    stage: std::cell::UnsafeCell<Stage<'a>>,
    /// the column, for a fused build that refused its checkpoints
    col: Option<(&'a [u8], &'a [u32; 256])>,
    syms: &'a Symbols,
    k: Kernels,
    orig_ptr: u32,
    segments: usize,
    out: std::cell::UnsafeCell<&'a mut Vec<u8>>,
    /// the build's end: the last helper to count in ends it and opens it
    built: Gate,
    walkers: AtomicUsize,
    failed: AtomicBool,
    panic: Mutex<Option<Box<dyn Any + Send>>>,
    t0: Instant,
    /// t0 to the build's end (B)
    build_end: AtomicU64,
}
// SAFETY: as `coop::Relay`: the stage is shared read only while its shares run, and changed
// only by the thread that ends the build (the built gate's last) or the walk (the thread
// that takes `walkers` to 0), each change published by the gate or the pool's run's end
unsafe impl Sync for Relay<'_> {}

impl Relay<'_> {
    fn caught(&self, r: std::thread::Result<()>) {
        if let Err(p) = r {
            self.failed.store(true, Ordering::Relaxed);
            let mut slot = lock(&self.panic);
            if slot.is_none() {
                *slot = Some(p);
            }
        }
    }

    fn failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    /// One helper's build: its share of the fused build, or of the run-fed sort and fill.
    /// Every helper, j past the shares too, passes both sort gates (counted of `helpers`);
    /// the share phases themselves do nothing for such a j (`RunFill::owns`).
    fn build_share(&self, j: usize, helpers: usize) {
        // SAFETY: the stage is shared read only until the built gate's last helper
        match unsafe { &*self.stage.get() } {
            Stage::Fused(Ok(b), _) => {
                let t = Instant::now();
                self.caught(catch_unwind(AssertUnwindSafe(|| b.share(j))));
                add(&PH.helper_fused_ns, since(t));
            }
            Stage::Runs(rf, _) => {
                if !rf.sorted.load(Ordering::Relaxed) {
                    // the helpers' sort (a leader-sorted block starts sorted)
                    let t = Instant::now();
                    if !self.failed() {
                        self.caught(catch_unwind(AssertUnwindSafe(|| rf.count(j))));
                    }
                    rf.counted.pass(helpers, || {
                        if !self.failed() {
                            self.caught(catch_unwind(AssertUnwindSafe(|| rf.merge())));
                        }
                    });
                    if !rf.direct {
                        if !self.failed() {
                            self.caught(catch_unwind(AssertUnwindSafe(|| rf.scatter(j))));
                        }
                        rf.scattered_gate.pass(helpers, || rf.check_scattered());
                    }
                    add(&PH.helper_sort_ns, since(t));
                }
                let t = Instant::now();
                if !self.failed() {
                    self.caught(catch_unwind(AssertUnwindSafe(|| rf.fill(j, self.k))));
                }
                add(&PH.helper_fill_ns, since(t));
            }
            _ => {}
        }
    }

    /// The build's end, on the built gate's last helper (`coop::Relay::end_build`)
    ///
    /// SAFETY: every helper has counted in, so none reads the stage until the gate opens
    unsafe fn end_build(&self) {
        let stage = unsafe { &mut *self.stage.get() };
        let old = std::mem::replace(stage, Stage::Over);
        if self.failed() {
            self.build_end.store(since(self.t0), Ordering::Relaxed);
            return;
        }
        let r = catch_unwind(AssertUnwindSafe(|| {
            let (tt, seg) = match old {
                Stage::Fused(b, seg) => {
                    let tt = match b {
                        Ok(b) => match b.finish() {
                            (Ok(()), tt) => tt,
                            (Err(_), tt) => {
                                add(&PH.fallbacks, 1);
                                let (ll, counts) = self.col.expect("the column");
                                bwt::build(ll, counts, tt);
                                tt
                            }
                        },
                        Err(tt) => tt,
                    };
                    (tt, seg)
                }
                Stage::Runs(rf, seg) => match rf.finish(self.syms, self.k) {
                    Ok(tt) => (tt, seg),
                    Err(kind) => panic!("the column's path refused the block: {kind}"),
                },
                _ => return None,
            };
            Some(RoomWalk::new(tt, self.orig_ptr, seg, self.segments))
        }));
        self.build_end.store(since(self.t0), Ordering::Relaxed);
        match r {
            Ok(Some(w)) => *stage = Stage::Walk(w),
            Ok(None) => {}
            Err(p) => self.caught(Err(p)),
        }
    }

    fn walk_and_leave(&self) {
        // SAFETY: the built gate was passed, and the stage changes again only when the
        // last walker leaves, which this thread has not
        if let Stage::Walk(w) = unsafe { &*self.stage.get() } {
            self.caught(catch_unwind(AssertUnwindSafe(|| {
                w.share::<FLAGGED_LANES>()
            })));
        }
        if self.walkers.fetch_sub(1, Ordering::AcqRel) == 1 {
            // SAFETY: every other walker has left, and none can join (walkers is 0)
            let stage = unsafe { &mut *self.stage.get() };
            if let Stage::Walk(w) = std::mem::replace(stage, Stage::Over) {
                let r = catch_unwind(AssertUnwindSafe(|| {
                    let (_, s) = w.finish();
                    if !self.failed() {
                        // SAFETY: only the thread ending the walk touches `out`
                        s.assemble(unsafe { &mut *self.out.get() });
                    }
                }));
                self.caught(r);
            }
        }
    }

    fn join(&self) {
        let t = Instant::now();
        self.built.wait();
        add(&PH.join_wait_ns, since(t));
        let mut v = self.walkers.load(Ordering::Acquire);
        loop {
            if v == 0 {
                return;
            }
            match self
                .walkers
                .compare_exchange_weak(v, v + 1, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(now) => v = now,
            }
        }
        self.walk_and_leave();
    }
}

/// One run of a run-fed team (`coop::invert_pipelined_with` at lever A's waits): the
/// helpers build block `f` (by its path) and walk it into `out`, while the caller runs
/// `lead`, then joins the walk if it is still going. Exactly `bwt::walk`'s output; its
/// timeline, each moment from the run's start.
#[allow(clippy::too_many_arguments)]
fn invert(
    pool: &Pool,
    f: &mut Front,
    syms: &Symbols,
    k: Kernels,
    tt: &mut TVector,
    seg: &mut SegScratch,
    out: &mut Vec<u8>,
    lead: &(dyn Fn() + Sync),
    direct: bool,
    inject: u8,
) -> Result<Timeline, Box<dyn Any + Send>> {
    let helpers = pool.threads().saturating_sub(1).max(1);
    let shares = helpers.min(MAX_SHARES);
    let t0 = Instant::now();
    add(&PH.runs, 1);
    let Front {
        ll,
        counts,
        checkpoints,
        recs,
        fo: _,
        fo_hs,
        hist,
        path,
        n,
    } = f;
    let (stage, col) = match *path {
        Path::Column => {
            add(&PH.team_column, 1);
            let b = match FusedBuild::new(ll, counts, checkpoints, mtf::CHECKPOINT, shares, tt) {
                Ok(b) => Ok(b),
                Err((_, tt)) => {
                    add(&PH.fallbacks, 1);
                    bwt::build(ll, counts, tt);
                    Err(tt)
                }
            };
            (Stage::Fused(b, seg), Some((&ll[..], &*counts)))
        }
        Path::Sorted | Path::Records => {
            add(&PH.team_records, 1);
            add(&PH.records, recs.len().saturating_sub(1) as u64);
            add(&PH.rows, *n as u64);
            let rf = RunFill::new(
                recs,
                counts,
                *n,
                *path == Path::Sorted,
                fo_hs,
                hist,
                shares,
                tt,
                direct,
                t0,
                inject,
            );
            (Stage::Runs(rf, seg), None)
        }
    };
    let relay = Relay {
        stage: std::cell::UnsafeCell::new(stage),
        col,
        syms,
        k,
        orig_ptr: syms.orig_ptr,
        segments: FLAGGED_SEGMENTS * helpers,
        out: std::cell::UnsafeCell::new(out),
        built: Gate::new(),
        walkers: AtomicUsize::new(helpers),
        failed: AtomicBool::new(false),
        panic: Mutex::new(None),
        t0,
        build_end: AtomicU64::new(0),
    };
    let relay = &relay;
    let (lead_start, lead_end) = (AtomicU64::new(0), AtomicU64::new(0));
    let helper = |j: usize| {
        relay.build_share(j, helpers);
        // SAFETY: the gate's last helper is the last to count in: every share has ended
        relay.built.pass(helpers, || unsafe { relay.end_build() });
        relay.walk_and_leave();
    };
    let leader = || {
        if inject == INJECT_LEAD_DELAY {
            thread::sleep(LEAD_DELAY);
        }
        // (both moments from t0, as the build's end is)
        let start = since(t0);
        let t = Instant::now();
        lead();
        add(&PH.lead_ns, since(t));
        lead_start.store(start, Ordering::Relaxed);
        lead_end.store(since(t0), Ordering::Relaxed);
    };
    if pool.threads() < 2 {
        leader();
        helper(0);
    } else {
        pool.run(&|j| {
            if j == 0 {
                leader();
                relay.join();
            } else {
                helper(j - 1);
            }
        });
    }
    add(&PH.run_ns, since(t0));
    // (the pool's run's end publishes every thread's stores to these)
    let tl = Timeline {
        lead_start: lead_start.load(Ordering::Relaxed),
        lead_end: lead_end.load(Ordering::Relaxed),
        build_end: relay.build_end.load(Ordering::Relaxed),
    };
    add(&PH.lead_start_ns, tl.lead_start);
    add(&PH.lead_end_ns, tl.lead_end);
    add(&PH.build_end_ns, tl.build_end);
    if tl.leader_late() {
        add(&PH.leader_late, 1);
    }
    let caught = lock(&relay.panic).take();
    caught.map_or(Ok(tl), Err)
}

// ---- The worker ----

/// A run-fed team's buffers (`parallel::Team`'s, with this module's fronts)
#[derive(Default)]
struct Team {
    fronts: [Front; 2],
    bufs: [Vec<u8>; 2],
    tt: TVector,
    seg: SegScratch,
    pool: Option<std::io::Result<Pool>>,
    small: Solo,
}

fn solo_on(shared: &Shared, t: &mut Team, s: Staged, k: Kernels) {
    let i = s.front;
    if solo(shared, &mut t.fronts[i], &mut t.small, s, k) {
        t.fronts[i] = Front::default();
        t.small = Solo::default();
    }
}

/// `parallel::pipelined_worker::<true, true, BARRIER_SPINS>` (the shipped levers, park and
/// small) with this module's front stage, build and timers; `MODE` its sort
fn worker<const MODE: u8>(shared: &Shared, jobs: &Mutex<Receiver<Job>>, k: Kernels, team_k: usize) {
    let mut t = Team::default();
    let mut staged: Option<Staged> = None;
    let mut inverted: Option<Inverted> = None;
    loop {
        let Some(s) = staged.take() else {
            if let Some(b) = inverted.take() {
                let i = b.buf;
                if finish_and_publish(shared, b, &t.bufs[i], k) {
                    t.bufs[i] = Vec::new();
                }
                continue;
            }
            let job = {
                let rx = jobs.lock().unwrap_or_else(|e| e.into_inner());
                rx.recv()
            };
            let Ok(job) = job else {
                return;
            };
            if shared.lock().cancelled {
                continue;
            }
            match stage::<MODE>(shared, job, &mut t.fronts[0], 0, k, false) {
                Staging::Team(s) => staged = Some(s),
                Staging::Small(small) => solo_on(shared, &mut t, small, k),
                Staging::Failed => {}
            }
            continue;
        };
        if t.pool
            .get_or_insert_with(|| Pool::pipelined(team_k))
            .is_err()
        {
            if let Some(b) = inverted.take() {
                let i = b.buf;
                if finish_and_publish(shared, b, &t.bufs[i], k) {
                    t.bufs[i] = Vec::new();
                }
            }
            solo_on(shared, &mut t, s, k);
            continue;
        }
        let a = inverted.as_ref().map_or(0, |b| 1 - b.buf);
        let other = 1 - s.front;
        let lead = Mutex::new(Lead {
            inverted: inverted.take(),
            ..Lead::default()
        });
        let Team {
            fronts,
            bufs,
            tt,
            seg,
            pool,
            small,
        } = &mut t;
        let Some(Ok(pool)) = pool.as_ref() else {
            unreachable!("the pool was spawned above")
        };
        let [f0, f1] = fronts;
        let (hf, lf) = if s.front == 0 { (f0, f1) } else { (f1, f0) };
        let [b0, b1] = bufs;
        let (hb, lb) = if a == 0 { (b0, &*b1) } else { (b1, &*b0) };
        let lf = Mutex::new(lf);
        let small = Mutex::new(small);
        #[cfg(test)]
        let place = s.place;
        let leader = || {
            let mut l = lock(&lead);
            #[cfg(test)]
            inject(shared, place, Fault::LeadStart, &l);
            if let Some(b) = l.inverted.take() {
                let t = Instant::now();
                l.rle_panicked = finish_and_publish(shared, b, lb, k);
                add(&PH.lead_rle_ns, since(t));
            }
            let mut next_job = try_job(jobs);
            while let Some(job) = next_job.take() {
                shared.overlapped.fetch_add(1, Ordering::Relaxed);
                if shared.lock().cancelled {
                    return;
                }
                let mut f = lock(&lf);
                match stage::<MODE>(shared, job, &mut f, other, k, true) {
                    Staging::Team(next) => l.next = Some(next),
                    Staging::Small(b) => {
                        let t = Instant::now();
                        let mut sc = lock(&small);
                        if solo(shared, &mut f, &mut sc, b, k) {
                            **f = Front::default();
                            **sc = Solo::default();
                        }
                        add(&PH.lead_small_ns, since(t));
                        // (lever B, shipped: stage on after a small block)
                        next_job = try_job(jobs);
                    }
                    Staging::Failed => {
                        shared.overlap_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            #[cfg(test)]
            inject(shared, place, Fault::LeadEnd, &l);
        };
        let r = catch_unwind(AssertUnwindSafe(|| {
            invert(
                pool,
                hf,
                &s.symbols,
                k,
                tt,
                seg,
                hb,
                &leader,
                MODE == DIR,
                0,
            )
        }));
        let mut l = lead.into_inner().unwrap_or_else(|e| e.into_inner());
        match r {
            Ok(Ok(_)) => {
                inverted = Some(Inverted {
                    place: s.place,
                    bit: s.bit,
                    symbols: s.symbols,
                    buf: a,
                });
                if l.rle_panicked {
                    t.bufs[1 - a] = Vec::new();
                }
            }
            Ok(Err(p)) => {
                let kind = ErrorKind::Internal(panic_message(p));
                publish(shared, s.place, s.bit, Err(kind), s.symbols.syms);
                t.tt = Default::default();
                t.seg = Default::default();
                t.bufs = Default::default();
                t.fronts[s.front] = Front::default();
            }
            Err(p) => {
                #[cfg(test)]
                {
                    shared.faults.lock().unwrap().recovered += 1;
                }
                // the run itself panicked, outside its stages' catches: every block it held
                // fails at its own place and bit, and the team is reset
                let msg = panic_message(p);
                let held = [Some(s), l.next.take()];
                for b in held.into_iter().flatten() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                if let Some(b) = l.inverted.take() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                t = Team::default();
                continue;
            }
        }
        staged = l.next.take();
    }
}

impl StreamDecoder {
    /// Research, opt-in (`run-fed`), not the default: `teams` pipelined teams of `k`
    /// threads (at least 2) with the shipped levers (park and small), their leaders
    /// emitting run records instead of the last column, and the sort where `sort` says
    /// (docs/journal/2026-11-09-run-fed-leader.md). Byte for byte the default decoder's
    /// output and errors.
    #[doc(hidden)]
    pub fn pipelined_run_fed(
        teams: usize,
        k: usize,
        slots: usize,
        kernels: Kernels,
        sort: RunFedSort,
    ) -> StreamDecoder {
        let (teams, team_k) = (teams.max(1), k.max(2));
        let mut dec = match sort {
            RunFedSort::Control => Self::launch(teams, slots, move |s, j| {
                worker::<CTL>(s, j, kernels, team_k)
            }),
            RunFedSort::Leader => Self::launch(teams, slots, move |s, j| {
                worker::<LS>(s, j, kernels, team_k)
            }),
            RunFedSort::Helpers => Self::launch(teams, slots, move |s, j| {
                worker::<HS>(s, j, kernels, team_k)
            }),
            RunFedSort::RegControl => Self::launch(teams, slots, move |s, j| {
                worker::<CTLR>(s, j, kernels, team_k)
            }),
            RunFedSort::Direct => Self::launch(teams, slots, move |s, j| {
                worker::<DIR>(s, j, kernels, team_k)
            }),
        };
        dec.threads = teams * team_k;
        dec.pipelined = true;
        dec
    }
}

#[cfg(test)]
mod unwind_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bwt::MIN_COOP;
    use crate::run_fed;

    fn xorshift(mut x: u64) -> impl FnMut() -> u64 {
        move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }
    }

    /// A column of runs (lengths from `lens`) over `sigma` bytes
    fn runs_column(len: usize, sigma: u64, lens: &[usize], seed: u64) -> Vec<u8> {
        let mut r = xorshift(seed);
        let mut ll = Vec::new();
        while ll.len() < len {
            let c = (r() % sigma) as u8;
            let run = lens[(r() % lens.len() as u64) as usize];
            ll.extend(std::iter::repeat(c).take(run));
        }
        ll.truncate(len);
        ll
    }

    /// A column's block symbols (every byte in use, in order), bzip2's MTF and RLE2
    fn symbols_of(ll: &[u8], orig_ptr: u32) -> Symbols {
        let mut list: Vec<u8> = (0..=255).collect();
        let mut syms = Vec::new();
        let mut run = 0u32;
        let flush = |run: &mut u32, syms: &mut Vec<u16>| {
            if *run > 0 {
                let mut z = *run - 1;
                loop {
                    syms.push((z & 1) as u16);
                    if z < 2 {
                        break;
                    }
                    z = (z - 2) / 2;
                }
                *run = 0;
            }
        };
        for &c in ll {
            let j = list.iter().position(|&x| x == c).unwrap();
            if j == 0 {
                run += 1;
                continue;
            }
            flush(&mut run, &mut syms);
            list.remove(j);
            list.insert(0, c);
            syms.push(j as u16 + 1);
        }
        flush(&mut run, &mut syms);
        syms.push(257);
        Symbols {
            crc: 0,
            orig_ptr,
            seq_to_unseq: std::array::from_fn(|i| i as u8),
            syms,
            eob: 257,
            max_len: 900_000,
        }
    }

    fn kernels() -> Vec<Kernels> {
        let mut v = vec![Kernels::scalar()];
        if Kernels::detect() != Kernels::scalar() {
            v.push(Kernels::detect());
        }
        v
    }

    fn recs_of(ll: &[u8]) -> Vec<u32> {
        let mut v: Vec<u32> = run_fed::runs_of(ll, 0)
            .iter()
            .map(|r| r.p << 8 | r.c as u32)
            .collect();
        v.push((ll.len() as u32) << 8);
        v
    }

    /// The helpers' sort, in `h` shares run one after another: the F order, or None if
    /// refused
    fn sort_by_shares(
        recs: &[u32],
        counts: &[u32; 256],
        n: usize,
        h: usize,
    ) -> Option<Vec<[u32; 2]>> {
        let r = recs.len().checked_sub(1)?;
        let mut hists = vec![[[0u32; 256]; 3]; h];
        let mut rise = true;
        for (j, hist) in hists.iter_mut().enumerate() {
            let (lo, hi) = share(r, h, j);
            rise &= count_share(recs, lo, hi, hist);
        }
        if !(rise && merge_shares(recs, counts, n, &mut hists)) {
            return None;
        }
        let mut fo: Vec<[u32; 2]> = Vec::with_capacity(r + 1);
        let mut all = true;
        for (j, hist) in hists.iter_mut().enumerate() {
            let (lo, hi) = share(r, h, j);
            // SAFETY: merge_shares' ranges, inside the r + 1 reserved, one share at a time
            all &= unsafe { scatter_share(recs, lo, hi, hist, fo.as_mut_ptr()) };
        }
        if !all {
            return None;
        }
        // SAFETY: every slot below r written once (each range filled exactly), and r here
        unsafe {
            fo.as_mut_ptr().add(r).write([n as u32, 0]);
            fo.set_len(r + 1);
        }
        Some(fo)
    }

    /// The record cuts of `h` shares: equal records (`by_rows` false, C31 as first built),
    /// or C35's (`row_cuts` at `RECORD_WEIGHT`, the team's)
    fn cuts_of(recs: &[u32], n: usize, h: usize, by_rows: bool) -> Vec<usize> {
        cuts_weighted(recs, n, h, by_rows.then_some(RECORD_WEIGHT))
    }

    /// Equal records (None), or `row_cuts` at a record weight (0: equal rows)
    fn cuts_weighted(recs: &[u32], n: usize, h: usize, weight: Option<u64>) -> Vec<usize> {
        let r = recs.len().saturating_sub(1);
        let Some(w) = weight else {
            return (0..=h).map(|j| r * j / h).collect();
        };
        let mut cuts = vec![0; h + 1];
        row_cuts(recs, n, h, w, &mut cuts);
        cuts
    }

    /// C31's count and prefix in the shares `cuts` gives: each share's `merge_rows` entry,
    /// or None if refused
    fn direct_hists(
        recs: &[u32],
        counts: &[u32; 256],
        n: usize,
        cuts: &[usize],
    ) -> Option<Vec<[[u32; 256]; 3]>> {
        let h = cuts.len() - 1;
        let mut hists = vec![[[0u32; 256]; 3]; h];
        let mut rise = true;
        for (j, hist) in hists.iter_mut().enumerate() {
            rise &= count_rows(recs, cuts[j], cuts[j + 1], hist);
        }
        (rise && merge_rows(recs, counts, n, &mut hists)).then_some(hists)
    }

    /// C31 in the shares `cuts` gives, run one after another: the table, or None if refused
    fn direct_by_cuts(
        recs: &[u32],
        counts: &[u32; 256],
        n: usize,
        cuts: &[usize],
        k: Kernels,
    ) -> Option<Vec<u32>> {
        recs.len().checked_sub(1)?;
        let h = cuts.len() - 1;
        let mut hists = direct_hists(recs, counts, n, cuts)?;
        let mut tt: Vec<u32> = Vec::with_capacity(n);
        let (mut all, mut tally) = (true, Tally::default());
        for (j, hist) in hists.iter_mut().enumerate() {
            // SAFETY: merge_rows' ranges, inside the n reserved, one share at a time
            all &= unsafe {
                fill_direct(
                    recs,
                    cuts[j],
                    cuts[j + 1],
                    hist,
                    tt.as_mut_ptr(),
                    k,
                    &mut tally,
                )
            };
        }
        assert!(tally.lookups <= 256 * h as u64);
        if !all {
            return None;
        }
        assert_eq!(tally.rows, n as u64, "the distinct words written");
        // SAFETY: every row written (each range filled exactly, the ranges tiling [0, n))
        unsafe { tt.set_len(n) };
        Some(tt)
    }

    /// C31 in `h` shares of equal records, and of equal rows (C35): the table, or None if
    /// either refused
    fn direct_by_shares(
        recs: &[u32],
        counts: &[u32; 256],
        n: usize,
        h: usize,
        k: Kernels,
    ) -> Option<Vec<u32>> {
        let a = direct_by_cuts(recs, counts, n, &cuts_of(recs, n, h, false), k);
        let b = direct_by_cuts(recs, counts, n, &cuts_of(recs, n, h, true), k);
        assert_eq!(a.is_some(), b.is_some());
        assert!(a == b);
        a
    }

    /// The share kernels on a column: the helpers' sort is `run_stage::sort`'s F order, in
    /// 1-5 shares; the fill in 1-5 shares, each kernel, is the production table; and every
    /// tiling broken by a row refused
    fn check_kernels(ll: &[u8]) {
        let n = ll.len();
        let counts = run_fed::counts_of(ll);
        let want = run_fed::production(ll, &counts);
        let recs = recs_of(ll);
        let mut s = crate::run_stage::Scratch::default();
        let oracle = api::sorted_fo(&recs, &counts, n, &mut s).expect("run_stage::sort");
        let (mut hist, mut one) = (Vec::new(), Vec::new());
        assert!(sort_one_share(&recs, &counts, n, &mut hist, &mut one));
        assert!(one == oracle, "n {n}: the leader's sort");
        for h in 1..=5 {
            let fo = sort_by_shares(&recs, &counts, n, h).expect("the shares' sort");
            assert!(fo == oracle, "n {n}, {h} shares: the F order");
            for k in kernels() {
                let mut tt: Vec<u32> = Vec::with_capacity(n);
                let mut tally = Tally::default();
                for j in 0..h {
                    let (q0, q1) = share(n, h, j);
                    let out = &mut tt.spare_capacity_mut()[q0..q1];
                    assert!(fill_share(
                        &recs, &fo, q0 as u32, q1 as u32, out, k, &mut tally
                    ));
                }
                // SAFETY: the shares tile [0, n), each filled whole (asserted)
                unsafe { tt.set_len(n) };
                assert!(tt == want, "n {n}, {h} shares, {k:?}: the table");
                let stores = tally.stores;
                assert!(stores >= n as u64 && stores <= n as u64 + 7 * 2 * recs.len() as u64);
                assert_eq!(tally.rows, n as u64, "the distinct words written");
                let direct = direct_by_shares(&recs, &counts, n, h, k).expect("C31");
                assert!(direct == want, "n {n}, {h} shares, {k:?}: the direct table");
            }
        }
        if n > 0 {
            let mut c = counts;
            c[ll[0] as usize] -= 1;
            c[(ll[0] as usize + 1) % 256] += 1;
            for h in 1..=3 {
                assert!(sort_by_shares(&recs, &c, n, h).is_none(), "a wrong count");
                assert!(direct_by_shares(&recs, &c, n, h, Kernels::scalar()).is_none());
                let mut r = recs.clone();
                *r.last_mut().unwrap() += 256;
                assert!(sort_by_shares(&r, &counts, n, h).is_none(), "a wrong end");
                if recs.len() > 2 {
                    let mut r = recs.clone();
                    r[1] += 256;
                    assert!(sort_by_shares(&r, &counts, n, h).is_none(), "a run moved");
                    assert!(direct_by_shares(&r, &counts, n, h, Kernels::detect()).is_none());
                    let mut r = recs.clone();
                    r[1] = (r[1] & 0xff) | (r[2] >> 8) << 8;
                    assert!(sort_by_shares(&r, &counts, n, h).is_none(), "a run emptied");
                }
            }
        }
    }

    #[test]
    fn share_kernels() {
        for len in [1usize, 2, 7, 8, 9, 17, 100, 4096, 70_001] {
            for sigma in [1u64, 2, 5, 256] {
                check_kernels(&runs_column(
                    len,
                    sigma,
                    &[1, 2, 7, 8, 9, 16, 40],
                    len as u64,
                ));
                let mut r = xorshift(sigma + len as u64);
                let ll: Vec<u8> = (0..len).map(|_| (r() % sigma) as u8).collect();
                check_kernels(&ll);
            }
        }
    }

    /// The share kernels' unsafe writes under Miri (the AVX2 fill too, with
    /// `-C target-feature=+avx2`), on small columns
    #[test]
    fn miri_share_kernels() {
        // (CI's AVX2 step asks for the AVX2 kernels to be reached, as run_stage's does)
        if std::env::var_os("MIBZ2_EXPECT_AVX2").is_some() {
            assert!(Kernels::detect().avx2, "AVX2 not detected");
        }
        for ll in [
            runs_column(300, 3, &[1, 2, 9, 17], 5),
            (0..64u8).map(|i| i % 3).collect(),
            vec![7u8; 40],
        ] {
            check_kernels(&ll);
        }
    }

    const MODES: [u8; 5] = [CTL, LS, HS, CTLR, DIR];

    /// A column through a run's whole path: the front stage by `MODE`, then `invert` on a
    /// pool of k; its output (or the injected panic)
    fn run_column<const MODE: u8>(
        pool: &Pool,
        ll: &[u8],
        orig_ptr: u32,
        tamper: bool,
        inject: u8,
        late: bool,
    ) -> Ran {
        run_column_on::<MODE>(pool, ll, orig_ptr, tamper, inject, late, Kernels::detect())
    }

    /// `run_column` on kernels `k`
    fn run_column_on<const MODE: u8>(
        pool: &Pool,
        ll: &[u8],
        orig_ptr: u32,
        tamper: bool,
        inject: u8,
        late: bool,
        k: Kernels,
    ) -> Ran {
        let b = symbols_of(ll, orig_ptr);
        let mut f = Front::default();
        let n = front_stage::<MODE>(&b, &mut f, k, &mut 0, &mut 0, &mut false).unwrap();
        assert_eq!(n, ll.len());
        assert_eq!(
            f.path,
            match (MODE, crate::run_stage::takes_runs(&b)) {
                (_, false) | (CTL | CTLR, _) => Path::Column,
                (LS, true) => Path::Sorted,
                _ => Path::Records,
            }
        );
        if tamper && f.path == Path::Records {
            // a run's start moved a row after the emitter: the helpers' checks refuse it,
            // and the column's path runs (a leader-sorted block was checked by its sort)
            f.recs[1] += 256;
        }
        let (mut tt, mut seg, mut out) = (TVector::new(), SegScratch::default(), Vec::new());
        let lead = || {
            if late {
                thread::sleep(std::time::Duration::from_millis(20));
            }
        };
        invert(
            pool,
            &mut f,
            &b,
            k,
            &mut tt,
            &mut seg,
            &mut out,
            &lead,
            MODE == DIR,
            inject,
        )?;
        // (the table too: a random column is no BWT, so its walk visits only one cycle)
        Ok((out, tt.0))
    }

    fn want(ll: &[u8], orig_ptr: u32) -> (Vec<u8>, Vec<u32>) {
        let mut tt = TVector::new();
        bwt::build(ll, &run_fed::counts_of(ll), &mut tt);
        let mut want = Vec::new();
        bwt::walk(&tt, orig_ptr, &mut want);
        (want, tt.0)
    }

    type Ran = Result<(Vec<u8>, Vec<u32>), Box<dyn Any + Send>>;

    fn each_mode(
        pool: &Pool,
        ll: &[u8],
        orig_ptr: u32,
        tamper: bool,
        inject: u8,
        late: bool,
    ) -> [Ran; 5] {
        MODES.map(|m| match m {
            CTL => run_column::<CTL>(pool, ll, orig_ptr, tamper, inject, late),
            LS => run_column::<LS>(pool, ll, orig_ptr, tamper, inject, late),
            HS => run_column::<HS>(pool, ll, orig_ptr, tamper, inject, late),
            CTLR => run_column::<CTLR>(pool, ll, orig_ptr, tamper, inject, late),
            _ => run_column::<DIR>(pool, ll, orig_ptr, tamper, inject, late),
        })
    }

    #[test]
    fn runs_are_walk() {
        for k in [2, 3, 4, 5] {
            let p = Pool::new(k).unwrap();
            for (i, ll) in [
                runs_column(MIN_COOP + 777, 9, &[8, 9, 16, 33, 64], 1),
                runs_column(50_000, 200, &[1, 2, 3], 2),
                runs_column(40_000, 1, &[40], 3),
                runs_column(9, 2, &[1], 4),
            ]
            .iter()
            .enumerate()
            {
                let w = want(ll, ll.len() as u32 / 3);
                for late in [false, true] {
                    for r in each_mode(&p, ll, ll.len() as u32 / 3, false, 0, late) {
                        assert!(r.unwrap() == w, "k {k}, column {i}, late {late}");
                    }
                }
            }
        }
    }

    /// A tiling broken after the front stage (a moved run start): every mode's checks
    /// refuse it, and the column's path rebuilds the block, the output still right
    #[test]
    fn a_refused_tiling_rebuilds_from_the_column() {
        let ll = runs_column(30_000, 5, &[8, 16, 20], 9);
        let w = want(&ll, 11);
        let p = Pool::new(3).unwrap();
        let before = PH.fallbacks.load(Ordering::Relaxed);
        for r in each_mode(&p, &ll, 11, true, 0, false) {
            assert!(r.unwrap() == w);
        }
        // (the helpers' sort and the direct fill each refused theirs)
        assert!(PH.fallbacks.load(Ordering::Relaxed) >= before + 2);
    }

    /// A panic in each phase of a helper's share: the run's Err, no thread left waiting,
    /// and the pool's next run right; and the leader's own panic resumed
    #[test]
    fn panics_in_each_phase() {
        let ll = runs_column(30_000, 5, &[8, 16, 20], 12);
        let w = want(&ll, 5);
        for k in [2, 3] {
            let p = Pool::new(k).unwrap();
            for inject in [INJECT_COUNT, INJECT_MERGE, INJECT_SCATTER, INJECT_FILL] {
                let r = run_column::<HS>(&p, &ll, 5, false, inject, false);
                let e = r.unwrap_err();
                assert_eq!(*e.downcast_ref::<&str>().unwrap(), "injected");
                assert!(run_column::<HS>(&p, &ll, 5, false, 0, false).unwrap() == w);
                if inject != INJECT_SCATTER {
                    let e = run_column::<DIR>(&p, &ll, 5, false, inject, false).unwrap_err();
                    assert_eq!(*e.downcast_ref::<&str>().unwrap(), "injected");
                    assert!(run_column::<DIR>(&p, &ll, 5, false, 0, false).unwrap() == w);
                }
            }
            let r = run_column::<LS>(&p, &ll, 5, false, INJECT_FILL, false);
            assert!(r.is_err());
            // the leader's panic, resumed once the helpers end
            let b = symbols_of(&ll, 5);
            let mut f = Front::default();
            front_stage::<HS>(&b, &mut f, Kernels::detect(), &mut 0, &mut 0, &mut false).unwrap();
            let (mut tt, mut seg, mut out) = (TVector::new(), SegScratch::default(), Vec::new());
            let r = catch_unwind(AssertUnwindSafe(|| {
                invert(
                    &p,
                    &mut f,
                    &b,
                    Kernels::detect(),
                    &mut tt,
                    &mut seg,
                    &mut out,
                    &|| panic!("lead"),
                    false,
                    0,
                )
            }));
            assert_eq!(*r.unwrap_err().downcast_ref::<&str>().unwrap(), "lead");
            assert!(run_column::<HS>(&p, &ll, 5, false, 0, false).unwrap() == w);
        }
    }

    // ---- Helpers past the shares (Guardian D049, B6) ----

    /// A share entry no share may touch: the sentinel in the shares' buffer past `shares`
    const MARK: [[u32; 256]; 3] = [[0xa5a5_a5a5; 256]; 3];
    /// The sentinel entries past the shares
    const SPARE: usize = 4;

    /// A run-fed build's phases by hand, on this thread, in the gates' order (every
    /// helper's count, the merge, every helper's scatter, the check, every helper's fill),
    /// for `helpers` helpers of `shares` shares (`invert`'s `helpers.min(MAX_SHARES)`), by
    /// `mode` (LS: the leader's sort; HS: the helpers'; DIR: the direct fill): the table.
    /// Asserts that the sort and fill took every share and no more (the bits), that no
    /// helper past the shares counted itself in the census, and, with `exact` false, that
    /// the shares' buffer past its `shares` entries still holds the sentinel. With `exact`,
    /// the buffer is fresh (`shares` entries, nothing past them: Guardian D049's witness, a
    /// fresh 64-entry histogram), where Miri reports any reference past it.
    fn by_hand(
        ll: &[u8],
        mode: u8,
        shares: usize,
        helpers: usize,
        k: Kernels,
        exact: bool,
    ) -> Vec<u32> {
        let n = ll.len();
        let counts = run_fed::counts_of(ll);
        let recs = recs_of(ll);
        let b = symbols_of(ll, 0);
        let presorted = mode == LS;
        let mut fo_hs = Vec::new();
        if presorted {
            assert!(sort_one_share(
                &recs,
                &counts,
                n,
                &mut Vec::new(),
                &mut fo_hs
            ));
        }
        let mut hist: Vec<[[u32; 256]; 3]> = if exact {
            Vec::new()
        } else {
            vec![MARK; shares + SPARE]
        };
        let mut tt = TVector::new();
        let what = format!("mode {mode}, {shares} shares, {helpers} helpers, {k:?}");
        let table = {
            let rf = RunFill::new(
                &recs,
                &counts,
                n,
                presorted,
                &mut fo_hs,
                &mut hist,
                shares,
                &mut tt,
                mode == DIR,
                Instant::now(),
                0,
            );
            assert_eq!(rf.sorted.load(Ordering::Relaxed), presorted);
            if !presorted {
                (0..helpers).for_each(|j| rf.count(j));
                rf.merge();
                if !rf.direct {
                    (0..helpers).for_each(|j| rf.scatter(j));
                    assert_eq!(rf.scattered.load(Ordering::Relaxed), rf.all(), "{what}");
                    rf.check_scattered();
                }
            }
            assert!(rf.sorted.load(Ordering::Relaxed), "{what}: the sort");
            (0..helpers).for_each(|j| rf.fill(j, k));
            assert_eq!(rf.claimed.load(Ordering::Relaxed), rf.all(), "{what}");
            assert_eq!(rf.filled.load(Ordering::Relaxed), rf.all(), "{what}");
            for (j, c) in rf.census.iter().enumerate() {
                let tallied = c.iter().any(|v| v.load(Ordering::Relaxed) != 0);
                assert!(j < shares || !tallied, "{what}: helper {j}'s census");
            }
            rf.finish(&b, k).expect("the table").0.clone()
        };
        assert_eq!(hist.len(), shares, "{what}: the shares' entries");
        if !exact {
            // SAFETY: the SPARE entries past `shares` were written (the sentinel) before
            // `RunFill::new` cut the vector to `shares`, which kept its capacity
            let past = unsafe { std::slice::from_raw_parts(hist.as_ptr().add(shares), SPARE) };
            assert!(
                past.iter().all(|h| *h == MARK),
                "{what}: an entry past the shares"
            );
        }
        table
    }

    /// Every helper count about the shares': fewer (none past), as many, one and two past
    fn surplus(ll: &[u8], shares: &[usize], kernels: &[Kernels], exact: bool) {
        let want = run_fed::production(ll, &run_fed::counts_of(ll));
        for &s in shares {
            for helpers in [s, s + 1, s + 2] {
                for mode in [LS, HS, DIR] {
                    for &k in kernels {
                        let got = by_hand(ll, mode, s, helpers, k, exact);
                        assert!(got == want, "mode {mode}, {s} shares, {helpers} helpers");
                    }
                }
            }
        }
    }

    /// Helpers past the shares (a team of more than `MAX_SHARES` + 1 threads) do no share
    /// work: every phase, at 1, 2, 63 and `MAX_SHARES` shares (k 2, 3, 64, 65; and their
    /// first and second surplus helpers, k = 66 and 67 at the most shares), each kernel
    #[test]
    fn helpers_past_the_shares_share_nothing() {
        let ll = runs_column(20_000, 9, &[1, 2, 8, 9, 40], 31);
        surplus(&ll, &[1, 2, 63, MAX_SHARES], &kernels(), false);
        surplus(&ll, &[MAX_SHARES], &kernels(), true);
    }

    /// The same under Miri, on a small column: two shares and their two surplus helpers, in
    /// a fresh buffer of exactly the shares' entries (Miri reports a reference past it) and
    /// in one with sentinels past it
    #[test]
    fn miri_helpers_past_the_shares() {
        let ll = runs_column(300, 3, &[1, 2, 9, 17], 6);
        surplus(&ll, &[2], &kernels(), true);
        surplus(&ll, &[2], &[Kernels::scalar()], false);
    }

    /// Whole runs on pools of k = 64 (63 helpers), 65 (`MAX_SHARES` helpers, the most
    /// shares), 66 (D049's witness: helper 64 past the shares), 67 and 80, every mode, each
    /// kernel: the walk's output and table, the gates passed by every helper
    #[test]
    fn teams_past_the_shares_are_walk() {
        let ll = runs_column(20_000, 9, &[1, 2, 8, 9, 40], 32);
        let w = want(&ll, 77);
        for k in [64, 65, 66, 67, 80] {
            let p = Pool::new(k).unwrap();
            assert_eq!(p.threads(), k);
            for kern in kernels() {
                let r = [
                    run_column_on::<CTL>(&p, &ll, 77, false, 0, false, kern),
                    run_column_on::<LS>(&p, &ll, 77, false, 0, false, kern),
                    run_column_on::<HS>(&p, &ll, 77, false, 0, false, kern),
                    run_column_on::<CTLR>(&p, &ll, 77, false, 0, false, kern),
                    run_column_on::<DIR>(&p, &ll, 77, false, 0, false, kern),
                ];
                for (m, r) in r.into_iter().enumerate() {
                    assert!(r.unwrap() == w, "k {k}, mode {m}, {kern:?}");
                }
            }
        }
    }

    /// C29's control, `mtf_checkpoints_reg`, is `mtf_checkpoints` on every column and
    /// error: its result, column, counts and checkpoints
    #[test]
    fn register_checkpoints_are_checkpoints() {
        let cp = mtf::CHECKPOINT;
        let mut cases: Vec<Symbols> = Vec::new();
        for (len, sigma, lens) in [
            (3 * cp, 5u64, &[1usize, 2, 3, 64, 65, 200][..]),
            (2 * cp + 17, 200, &[1, 2][..]),
            (cp - 1, 3, &[40, 1000][..]),
            (cp, 2, &[cp / 2 + 3][..]),
            (5 * cp + 9, 1, &[cp + 5][..]),
            (100, 7, &[1][..]),
        ] {
            let ll = runs_column(len, sigma, lens, len as u64);
            cases.push(symbols_of(&ll, 0));
            // the bound at, just above and below the length: BlockTooLong where it fails
            for d in [0isize, -1, -64, 1] {
                let mut b = symbols_of(&ll, 0);
                b.max_len = (len as isize + d).max(1) as usize;
                cases.push(b);
            }
        }
        let mut long = symbols_of(&[1, 2], 0);
        long.syms = [vec![0u16; 22], vec![257]].concat();
        cases.push(long);
        for b in &cases {
            for k in kernels() {
                let (mut ll, mut c, mut p) = (Vec::new(), [0; 256], Vec::new());
                let want = mtf::mtf_checkpoints(b, &mut ll, &mut c, &mut p, k);
                let (mut ll2, mut c2, mut p2) = (vec![1], [9; 256], vec![[3; 256]]);
                let got = mtf::mtf_checkpoints_reg(b, &mut ll2, &mut c2, &mut p2, k);
                assert_eq!(got, want);
                if want.is_ok() {
                    assert!(ll == ll2 && c == c2 && p == p2);
                }
            }
        }
    }

    // ---- Malformed record tilings (D049 §4: unreachable from valid emission) ----

    /// The column a tiling describes, if it is one: `run_stage::sort`'s contract, by an
    /// independent model (the first run at 0, starts strictly rising, the closing record
    /// at n, below 2^24, and the counts the column's), else None
    fn tiled(recs: &[u32], counts: &[u32; 256], n: usize) -> Option<Vec<u8>> {
        if n == 0 || n >= 1 << 24 || recs.len() < 2 {
            return None;
        }
        let r = recs.len() - 1;
        if recs[0] >> 8 != 0 || recs[r] >> 8 != n as u32 {
            return None;
        }
        let mut ll = Vec::with_capacity(n);
        for w in recs.windows(2) {
            let (p, next) = (w[0] >> 8, w[1] >> 8);
            if next <= p {
                return None;
            }
            ll.extend(std::iter::repeat((w[0] & 0xff) as u8).take((next - p) as usize));
        }
        (run_fed::counts_of(&ll) == *counts).then_some(ll)
    }

    /// The tilings' mutations, by name: each changes the records or the counts after the
    /// emitter. The last two leave a valid tiling (another column; the same column), to be
    /// taken; the others leave one that isn't, to be refused (or, now and then, by chance,
    /// one that is: the model decides)
    const TAMPERS: [&str; 10] = [
        "a start moved",
        "a byte changed",
        "two neighbours' bytes swapped",
        "a record dropped",
        "a record doubled",
        "the closing record moved",
        "the first start moved",
        "a count moved",
        "two equal runs' bytes swapped (another column)",
        "a run split in two (the same column)",
    ];

    fn tamper(
        recs: &mut Vec<u32>,
        counts: &mut [u32; 256],
        kind: usize,
        r: &mut impl FnMut() -> u64,
    ) {
        let rr = recs.len() - 1;
        let pick = |r: &mut dyn FnMut() -> u64, lo: usize, hi: usize| {
            lo + (r() % (hi - lo).max(1) as u64) as usize
        };
        match kind {
            0 if rr > 1 => {
                let i = pick(r, 1, rr);
                let d = [-3i64, -2, -1, 1, 2, 3][(r() % 6) as usize];
                let p = ((recs[i] >> 8) as i64 + d).max(0) as u32;
                recs[i] = p << 8 | (recs[i] & 0xff);
            }
            1 => {
                let i = pick(r, 0, rr);
                recs[i] = (recs[i] & !0xff) | (r() % 256) as u32;
            }
            2 if rr > 1 => {
                let i = pick(r, 0, rr - 1);
                let (a, b) = (recs[i] & 0xff, recs[i + 1] & 0xff);
                recs[i] = (recs[i] & !0xff) | b;
                recs[i + 1] = (recs[i + 1] & !0xff) | a;
            }
            3 if rr > 1 => {
                recs.remove(pick(r, 1, rr));
            }
            4 => {
                let i = pick(r, 0, rr);
                recs.insert(i, recs[i]);
            }
            5 => {
                let d = [-1i64, 1, 256, -((r() % 9 + 2) as i64)][(r() % 4) as usize];
                recs[rr] = (((recs[rr] >> 8) as i64 + d).max(0) as u32) << 8;
            }
            6 => recs[0] += (1 + r() % 3) as u32 * 256,
            7 => {
                let a = (recs[pick(r, 0, rr)] & 0xff) as usize;
                if counts[a] > 0 {
                    counts[a] -= 1;
                    counts[(a + 1 + (r() % 255) as usize) % 256] += 1;
                }
            }
            8 => {
                // two runs of one length and different bytes, their bytes swapped
                let len = |i: usize, recs: &[u32]| (recs[i + 1] >> 8) - (recs[i] >> 8);
                let i = pick(r, 0, rr);
                if let Some(j) = (0..rr).find(|&j| {
                    j != i && len(j, recs) == len(i, recs) && (recs[j] ^ recs[i]) & 0xff != 0
                }) {
                    let (a, b) = (recs[i] & 0xff, recs[j] & 0xff);
                    recs[i] = (recs[i] & !0xff) | b;
                    recs[j] = (recs[j] & !0xff) | a;
                }
            }
            _ => {
                // a run of 2 or more split: a record at its middle, its byte
                if let Some(i) = (0..rr).find(|&i| (recs[i + 1] >> 8) - (recs[i] >> 8) >= 2) {
                    let mid = (recs[i] >> 8) + ((recs[i + 1] >> 8) - (recs[i] >> 8)) / 2;
                    recs.insert(i + 1, mid << 8 | (recs[i] & 0xff));
                }
            }
        }
    }

    /// One tampered tiling through every check: the leader's sort, `run_stage::sort`
    /// (independent of this module), the helpers' sort and the direct fill in 1, 2, 3 and
    /// 5 shares, each kernel: each takes it iff the model says it tiles, and then makes
    /// exactly the column's table; and, with `pool`, whole runs (the helpers' sort and the
    /// direct fill on the pool, from the front stage of the untampered block): the walk of
    /// that table if taken, else the untampered block's, rebuilt from its symbols. Returns
    /// whether it was taken.
    fn check_tiling(
        ll: &[u8],
        recs: &[u32],
        counts: &[u32; 256],
        pool: Option<&Pool>,
        what: &str,
    ) -> bool {
        let n = ll.len();
        let model = tiled(recs, counts, n);
        let taken = model.is_some();
        let mut s = crate::run_stage::Scratch::default();
        let oracle = api::sorted_fo(recs, counts, n, &mut s);
        assert_eq!(oracle.is_some(), taken, "{what}: run_stage::sort");
        let (mut hist, mut one) = (Vec::new(), Vec::new());
        assert_eq!(
            sort_one_share(recs, counts, n, &mut hist, &mut one),
            taken,
            "{what}: the leader's sort"
        );
        let want = model.as_ref().map(|m| run_fed::production(m, counts));
        if taken {
            assert!(
                Some(&one) == oracle.as_ref(),
                "{what}: the leader's F order"
            );
        }
        for h in [1, 2, 3, 5] {
            let fo = sort_by_shares(recs, counts, n, h);
            assert_eq!(fo.is_some(), taken, "{what}, {h} shares: the helpers' sort");
            for k in kernels() {
                if let Some(fo) = &fo {
                    assert!(Some(fo) == oracle.as_ref(), "{what}, {h}: the F order");
                    let mut tt: Vec<u32> = Vec::with_capacity(n);
                    let mut tally = Tally::default();
                    for j in 0..h {
                        let (q0, q1) = share(n, h, j);
                        let out = &mut tt.spare_capacity_mut()[q0..q1];
                        assert!(fill_share(
                            recs, fo, q0 as u32, q1 as u32, out, k, &mut tally
                        ));
                    }
                    // SAFETY: the shares tile [0, n), each filled whole (asserted)
                    unsafe { tt.set_len(n) };
                    assert!(Some(&tt) == want.as_ref(), "{what}, {h}, {k:?}: the table");
                }
                let d = direct_by_shares(recs, counts, n, h, k);
                assert_eq!(d.is_some(), taken, "{what}, {h}, {k:?}: the direct fill");
                assert!(d == want, "{what}, {h}, {k:?}: the direct table");
            }
        }
        if let Some(pool) = pool {
            let b = symbols_of(ll, 1);
            let expect = match &want {
                Some(tt) => {
                    let mut out = Vec::new();
                    bwt::walk(&TVector(tt.clone()), 1, &mut out);
                    (out, tt.clone())
                }
                None => self::want(ll, 1),
            };
            for direct in [false, true] {
                let mut f = Front::default();
                let mode = |f: &mut Front| {
                    if direct {
                        front_stage::<DIR>(&b, f, Kernels::detect(), &mut 0, &mut 0, &mut false)
                    } else {
                        front_stage::<HS>(&b, f, Kernels::detect(), &mut 0, &mut 0, &mut false)
                    }
                };
                mode(&mut f).unwrap();
                assert_eq!(f.path, Path::Records);
                f.recs = recs.to_vec();
                f.counts = *counts;
                let (mut tt, mut seg, mut out) =
                    (TVector::new(), SegScratch::default(), Vec::new());
                let k = Kernels::detect();
                invert(
                    pool,
                    &mut f,
                    &b,
                    k,
                    &mut tt,
                    &mut seg,
                    &mut out,
                    &|| {},
                    direct,
                    0,
                )
                .unwrap();
                assert!((out, tt.0) == expect, "{what}, direct {direct}: the run");
            }
        }
        taken
    }

    /// Tilings tampered after the emitter, each check against the model, seeded and
    /// bounded (`cases` cases of columns of `lens` rows, every `whole`th also as whole runs
    /// on a pool of 3): each tamper reached, the refusals and the valid ones taken
    fn tilings(seed: u64, cases: usize, lens: &[usize], whole: usize) {
        let pool = Pool::new(3).unwrap();
        let mut r = xorshift(seed);
        let mut seen = [[0u64; 2]; TAMPERS.len()];
        for c in 0..cases {
            let len = lens[c % lens.len()];
            let sigma = [2u64, 3, 5, 9][c % 4];
            let ll = runs_column(len, sigma, &[1, 2, 3, 4, 8], r());
            let (mut recs, mut counts) = (recs_of(&ll), run_fed::counts_of(&ll));
            let kind = c % TAMPERS.len();
            tamper(&mut recs, &mut counts, kind, &mut r);
            let what = format!("case {c} (seed {seed}): {}", TAMPERS[kind]);
            let taken = check_tiling(
                &ll,
                &recs,
                &counts,
                (c % whole == 0).then_some(&pool),
                &what,
            );
            seen[kind][taken as usize] += 1;
        }
        eprintln!("tilings (seed {seed}, {cases} cases): tamper, refused, taken");
        for (t, [no, yes]) in TAMPERS.iter().zip(seen) {
            eprintln!("{no:>7} {yes:>7}  {t}");
        }
        for (i, [no, yes]) in seen.iter().enumerate() {
            if i < 8 {
                assert!(*no > 0, "never refused: {}", TAMPERS[i]);
            } else {
                assert!(*yes > 0, "never taken: {}", TAMPERS[i]);
            }
        }
    }

    #[test]
    fn tampered_tilings_are_refused_or_tile() {
        tilings(0xd049, 6_000, &[40, 301, 4_099, 20_011], 6);
    }

    /// The same under Miri: the helpers' sort's scatter and the fills' writes on tilings
    /// that are not, and the runs' refusals and rebuilds on the pool
    #[test]
    fn miri_tampered_tilings() {
        tilings(0x5afe, 20, &[24, 37], 4);
    }

    // ---- C29's control against `mtf_checkpoints`: its checkpoints and raw stores (D049 §4) ----

    /// The input classes a symbol stream reaches in `mtf`'s semantics (`classify`), each
    /// one a branch of `mtf_checkpoints_reg_avx2`: a run split at a checkpoint, a run
    /// ending at one, a byte written at one, a block ending at one; a run of 64 or fewer
    /// by the two 32-byte stores, or by the byte store at the block's bound (n + 64 past
    /// `max_len`), or a longer run; and each error and where it fires
    #[derive(Debug, Default)]
    struct Reach {
        run_split: u64,
        run_to_checkpoint: u64,
        byte_at_checkpoint: u64,
        end_at_checkpoint: u64,
        run_vector: u64,
        run_bytes_at_bound: u64,
        run_long: u64,
        ok: u64,
        too_long_in_run: u64,
        too_long_at_byte: u64,
        run_too_long: u64,
        no_eob: u64,
    }

    impl Reach {
        fn each(&self) -> [(&'static str, u64); 12] {
            [
                ("run split at a checkpoint", self.run_split),
                ("run ending at a checkpoint", self.run_to_checkpoint),
                ("byte at a checkpoint", self.byte_at_checkpoint),
                ("block ending at a checkpoint", self.end_at_checkpoint),
                ("run by the vector stores", self.run_vector),
                (
                    "run of <= 64 by bytes at the bound",
                    self.run_bytes_at_bound,
                ),
                ("run of > 64", self.run_long),
                ("Ok", self.ok),
                ("BlockTooLong in a run", self.too_long_in_run),
                ("BlockTooLong at a byte", self.too_long_at_byte),
                ("RunTooLong", self.run_too_long),
                ("no end-of-block symbol", self.no_eob),
            ]
        }
    }

    /// `b`'s classes in `mtf`'s semantics (an independent model of the input, not of the
    /// kernel): the length so far, each run and byte as it lands, the bound checked
    fn classify(b: &Symbols, r: &mut Reach) {
        // (mtf.rs's MAX_RUN_WEIGHT)
        const MAX_RUN_WEIGHT: u64 = 2 * 1024 * 1024;
        let (cp, max) = (mtf::CHECKPOINT, b.max_len);
        let (mut n, mut run, mut w) = (0usize, 0u64, 1u64);
        for &s in &b.syms {
            if s <= 1 {
                if w >= MAX_RUN_WEIGHT {
                    r.run_too_long += 1;
                    return;
                }
                run += (s as u64 + 1) * w;
                w <<= 1;
                continue;
            }
            if run != 0 {
                let len = run as usize;
                if n + len > max {
                    r.too_long_in_run += 1;
                    return;
                }
                let end = n + len;
                r.run_split += ((n / cp + 1) * cp < end) as u64;
                r.run_to_checkpoint += (end % cp == 0) as u64;
                if len > 64 {
                    r.run_long += 1;
                } else if n + 64 <= max {
                    r.run_vector += 1;
                } else {
                    r.run_bytes_at_bound += 1;
                }
                (n, run, w) = (end, 0, 1);
            }
            if s == b.eob {
                r.ok += 1;
                r.end_at_checkpoint += (n > 0 && n % cp == 0) as u64;
                return;
            }
            if n >= max {
                r.too_long_at_byte += 1;
                return;
            }
            r.byte_at_checkpoint += (n > 0 && n % cp == 0) as u64;
            n += 1;
        }
        r.no_eob += 1;
    }

    /// `mtf_checkpoints_reg` against `mtf_checkpoints` on `b`, each kernel: the result, and
    /// the column, counts and checkpoints (on errors too: both are `mtf`'s on every input),
    /// from buffers holding another block's
    fn same_checkpoints(b: &Symbols, r: &mut Reach) {
        classify(b, r);
        for k in kernels() {
            let (mut ll, mut c, mut p) = (Vec::new(), [0; 256], Vec::new());
            let want = mtf::mtf_checkpoints(b, &mut ll, &mut c, &mut p, k);
            let (mut ll2, mut c2, mut p2) = (vec![1, 2, 3], [9; 256], vec![[3; 256]; 2]);
            let got = mtf::mtf_checkpoints_reg(b, &mut ll2, &mut c2, &mut p2, k);
            let what = format!("{} symbols, max_len {}, {k:?}", b.syms.len(), b.max_len);
            assert_eq!(got, want, "{what}: the result");
            assert!(ll == ll2, "{what}: the column");
            assert!(c == c2, "{what}: the counts");
            if want.is_ok() {
                assert!(p == p2, "{what}: the checkpoints");
            }
        }
    }

    /// A column's symbols at its length's bounds about `d` (BlockTooLong below its length,
    /// the byte store at the bound just above it)
    fn at_bounds(ll: &[u8], ds: &[isize], out: &mut Vec<Symbols>) {
        for &d in ds {
            let mut b = symbols_of(ll, 0);
            b.max_len = (ll.len() as isize + d).max(1) as usize;
            out.push(b);
        }
    }

    /// The columns whose runs and bytes land on and about the checkpoints: a run to a
    /// checkpoint, a run across one and two, a byte at one (and a block's last byte there),
    /// runs of 63, 64 and 65 at the bound
    fn checkpoint_columns() -> Vec<Vec<u8>> {
        let cp = mtf::CHECKPOINT;
        let mut v = vec![
            vec![7u8; cp],
            vec![7u8; cp + 100],
            [vec![1u8], vec![2u8; 2 * cp - 1]].concat(),
            [vec![1u8; cp - 1], vec![2u8], vec![3u8; 40]].concat(),
            [vec![1u8; cp], vec![2u8; 3]].concat(),
            [vec![7u8; cp], vec![8u8]].concat(),
        ];
        for tail in [63usize, 64, 65] {
            v.push([vec![4u8; cp - 30], vec![5u8], vec![6u8; tail]].concat());
        }
        v
    }

    /// C29's control on the inputs about its checkpoints and its raw stores, valid and
    /// not, each reaching every class (`Reach`), against `mtf_checkpoints`: seeded
    /// (xorshift), bounded (about 3,000 streams), printed
    #[test]
    fn register_checkpoints_reach_every_class() {
        let cp = mtf::CHECKPOINT;
        let mut cases: Vec<Symbols> = Vec::new();
        let bounds = [-65isize, -64, -63, -1, 0, 1, 31, 63, 64, 65];
        for ll in checkpoint_columns() {
            at_bounds(&ll, &bounds, &mut cases);
        }
        let mut r = xorshift(0xc29);
        let lens = [
            1usize,
            2,
            3,
            31,
            32,
            33,
            63,
            64,
            65,
            100,
            1000,
            cp / 2 + 3,
            cp + 5,
        ];
        for i in 0..120 {
            let len = [cp - 1, cp, cp + 1, 2 * cp, 2 * cp + 63, 3 * cp + 5][i % 6];
            let sigma = [1u64, 2, 3, 9, 200][i % 5];
            let ll = runs_column(len, sigma, &lens, r());
            at_bounds(&ll, &bounds[(i % 4)..(i % 4) + 6], &mut cases);
        }
        // malformed streams: random symbols, a random alphabet and bound, some without
        // their end-of-block symbol; and the longest runs' weights
        for i in 0..2_000 {
            let eob = 2 + (r() % 256) as u16;
            let len = (r() % (3 * cp as u64)) as usize;
            let run_heavy = i % 3 == 0;
            let mut syms: Vec<u16> = (0..len)
                .map(|_| {
                    if run_heavy && r() % 4 != 0 {
                        (r() % 2) as u16
                    } else {
                        (r() % eob as u64) as u16
                    }
                })
                .collect();
            if i % 50 != 7 {
                syms.push(eob);
            }
            let mut perm: Vec<u8> = (0..=255).collect();
            for j in (1..256).rev() {
                perm.swap(j, (r() % (j as u64 + 1)) as usize);
            }
            cases.push(Symbols {
                crc: 0,
                orig_ptr: 0,
                seq_to_unseq: perm.try_into().unwrap(),
                syms,
                eob,
                max_len: [cp, 2 * cp, 3 * cp + 7, 900_000][i % 4] - (r() % 70) as usize,
            });
        }
        let mut long = symbols_of(&[1, 2], 0);
        long.syms = [vec![0u16; 22], vec![257]].concat();
        cases.push(long);
        let mut reach = Reach::default();
        for b in &cases {
            same_checkpoints(b, &mut reach);
        }
        eprintln!(
            "C29 control: {} streams, kernels {:?}",
            cases.len(),
            kernels()
        );
        for (what, n) in reach.each() {
            eprintln!("{n:>9}  {what}");
            assert!(n > 0, "class not reached: {what}");
        }
    }

    /// The same under Miri, for its AVX2 kernel's raw stores (CI's AVX2 step, `-C
    /// target-feature=+avx2`; without AVX2, `mtf_checkpoints_reg` is `mtf_checkpoints`'
    /// scalar twin itself, so there is nothing to check and the test returns): one stream
    /// a class, on the detected kernels only
    #[test]
    fn miri_register_checkpoints() {
        if std::env::var_os("MIBZ2_EXPECT_AVX2").is_some() {
            assert!(Kernels::detect().avx2, "AVX2 not detected");
        }
        if cfg!(miri) && !Kernels::detect().avx2 {
            return;
        }
        let cp = mtf::CHECKPOINT;
        let mut cases: Vec<Symbols> = Vec::new();
        // a run to a checkpoint, ending the block there; a run across one; a block's last
        // byte at one, its bound just below (BlockTooLong at the byte); a run of 64 at the
        // bound (the byte store) and past it (BlockTooLong in the run)
        at_bounds(&vec![7u8; cp], &[0], &mut cases);
        at_bounds(&vec![7u8; cp + 100], &[0], &mut cases);
        at_bounds(&[vec![7u8; cp], vec![8u8]].concat(), &[-1, 0], &mut cases);
        let tail = [vec![4u8; cp - 30], vec![5u8], vec![6u8; 64]].concat();
        at_bounds(&tail, &[-1, 0], &mut cases);
        let mut long = symbols_of(&[1, 2], 0);
        long.syms = [vec![0u16; 22], vec![257]].concat();
        cases.push(long);
        let mut short = symbols_of(&[3, 3, 3, 4], 0);
        short.syms.pop();
        cases.push(short);
        let mut reach = Reach::default();
        for b in &cases {
            classify(b, &mut reach);
            let k = Kernels::detect();
            let (mut ll, mut c, mut p) = (Vec::new(), [0; 256], Vec::new());
            let want = mtf::mtf_checkpoints(b, &mut ll, &mut c, &mut p, k);
            // (a fresh column: its capacity exactly the bound, where Miri sees a store past it)
            let (mut ll2, mut c2, mut p2) = (Vec::new(), [9; 256], vec![[3; 256]]);
            let got = mtf::mtf_checkpoints_reg(b, &mut ll2, &mut c2, &mut p2, k);
            assert_eq!(got, want);
            assert!(ll == ll2 && c == c2);
            if want.is_ok() {
                assert!(p == p2);
            }
        }
        for (what, n) in reach.each() {
            assert!(n > 0, "class not reached: {what}");
        }
    }

    /// The run's hand-offs under Miri (its race detector): every mode, a small column
    #[test]
    fn miri_runs_are_walk() {
        let ll = runs_column(400, 4, &[8, 9, 16], 21);
        let w = want(&ll, 7);
        for k in [2, 3] {
            let p = Pool::new(k).unwrap();
            for r in each_mode(&p, &ll, 7, false, 0, false) {
                assert!(r.unwrap() == w);
            }
            for r in each_mode(&p, &ll, 7, true, 0, false) {
                assert!(r.unwrap() == w);
            }
        }
    }

    // ---- The physical write footprint (Guardian D044) ----

    /// No table entry: an entry's row is below n < 2^24 - 8, a vector lane's too
    const SENT: u32 = u32::MAX;
    /// Words of sentinel either side of the table, where a store past it would land
    const PAD: usize = 16;

    /// The footprint check: every word of `buf` (the table at PAD) inside `owned` holds
    /// the production table's entry, and every other word is still the sentinel. Returns
    /// the words written (distinct), which is then exactly `owned`'s size.
    fn footprint(buf: &[u32], want: &[u32], owned: &[bool], what: &str) -> u64 {
        let n = want.len();
        assert_eq!(buf.len(), n + 2 * PAD);
        let mut written = 0;
        for (i, &w) in buf.iter().enumerate() {
            let row = i.checked_sub(PAD).filter(|&q| q < n);
            match row {
                Some(q) if owned[q] => {
                    assert_eq!(w, want[q], "{what}: row {q}");
                    written += 1;
                }
                _ => assert_eq!(
                    w, SENT,
                    "{what}: word {i} (row {row:?}) is not this share's"
                ),
            }
        }
        written
    }

    /// The share counts the footprint tests use: every count up to 9, one-row shares
    /// (h = n), and shares of about 7, 8, 9, 15, 16 and 17 rows (the vector store's edges)
    fn share_counts(n: usize) -> Vec<usize> {
        let mut hs: Vec<usize> = (1..=9.min(n.max(1))).collect();
        if n <= 300 {
            hs.push(n.max(1));
        }
        for len in [7, 8, 9, 15, 16, 17] {
            hs.push((n / len).clamp(1, crate::coop::MAX_SHARES));
        }
        hs.sort_unstable();
        hs.dedup();
        hs
    }

    /// Each share of each fill run alone into a table of sentinels: its stores land in its
    /// own rows only (the AVX2 kernel's included, whose 8-entry stores must stop at the
    /// share's end and, in C31, at each byte's range end in the share), it writes every
    /// one of them, and the shares' rows tile [0, n)
    fn check_footprint(ll: &[u8]) {
        let n = ll.len();
        let counts = run_fed::counts_of(ll);
        let want = run_fed::production(ll, &counts);
        let recs = recs_of(ll);
        let r = recs.len() - 1;
        for h in share_counts(n) {
            let fo = sort_by_shares(&recs, &counts, n, h).expect("the shares' sort");
            for k in kernels() {
                // C30's fill: share j's rows [q0, q1)
                let mut cover = vec![0u32; n];
                for j in 0..h {
                    let (q0, q1) = share(n, h, j);
                    let mut buf = vec![SENT; n + 2 * PAD];
                    // (the share's slice of a bigger buffer: a store outside it would
                    // change a sentinel, and under Miri leave the slice's borrow)
                    let out = unsafe {
                        std::slice::from_raw_parts_mut(
                            buf.as_mut_ptr().add(PAD + q0) as *mut MaybeUninit<u32>,
                            q1 - q0,
                        )
                    };
                    let mut tally = Tally::default();
                    assert!(fill_share(
                        &recs, &fo, q0 as u32, q1 as u32, out, k, &mut tally
                    ));
                    let owned: Vec<bool> = (0..n).map(|q| q0 <= q && q < q1).collect();
                    let what = format!("n {n}, hs, {h} shares, share {j}, {k:?}");
                    let w = footprint(&buf, &want, &owned, &what);
                    assert_eq!(w, (q1 - q0) as u64, "{what}: its distinct words");
                    assert_eq!(tally.rows, w, "{what}: the tally's rows");
                    cover[q0..q1].iter_mut().for_each(|c| *c += 1);
                }
                assert!(
                    cover.iter().all(|&c| c == 1),
                    "n {n}, hs, {h} shares: a tiling"
                );
                // C31's fill, cut by records, by rows (C35), and one record a share
                let mut cuts_all = vec![cuts_of(&recs, n, h, false), cuts_of(&recs, n, h, true)];
                if h == 1 {
                    cuts_all.push((0..=r).collect());
                }
                for cuts in cuts_all {
                    let hists = direct_hists(&recs, &counts, n, &cuts).expect("C31's prefix");
                    let mut cover = vec![0u32; n];
                    for (j, hist) in hists.iter().enumerate() {
                        let mut owned = vec![false; n];
                        for (&a, &e) in hist[1].iter().zip(&hist[2]) {
                            for q in a as usize..e as usize {
                                owned[q] = true;
                                cover[q] += 1;
                            }
                        }
                        let mut buf = vec![SENT; n + 2 * PAD];
                        let mut hist = *hist;
                        let mut tally = Tally::default();
                        // SAFETY: the table is the buffer's n words after PAD, and this
                        // share's ranges are inside [0, n) (`merge_rows`)
                        let ok = unsafe {
                            fill_direct(
                                &recs,
                                cuts[j],
                                cuts[j + 1],
                                &mut hist,
                                buf.as_mut_ptr().add(PAD),
                                k,
                                &mut tally,
                            )
                        };
                        let what = format!(
                            "n {n}, dir, cuts {:?}.., share {j}, {k:?}",
                            &cuts[..cuts.len().min(4)]
                        );
                        assert!(ok, "{what}");
                        let w = footprint(&buf, &want, &owned, &what);
                        assert_eq!(tally.rows, w, "{what}: its distinct words");
                    }
                    assert!(
                        cover.iter().all(|&c| c == 1),
                        "n {n}, dir: the byte ranges tile"
                    );
                }
            }
        }
    }

    #[test]
    fn fills_write_their_own_rows_only() {
        for len in [1usize, 2, 7, 8, 9, 16, 17, 23, 64, 300, 2049] {
            for sigma in [1u64, 2, 5, 256] {
                check_footprint(&runs_column(
                    len,
                    sigma,
                    &[1, 7, 8, 9, 15, 16, 17, 40],
                    len as u64,
                ));
                let mut r = xorshift(3 * sigma + len as u64);
                let ll: Vec<u8> = (0..len).map(|_| (r() % sigma) as u8).collect();
                check_footprint(&ll);
            }
        }
    }

    /// A pointer the share threads below write through, each its own rows
    #[derive(Clone, Copy)]
    struct Rows(*mut u32);
    // SAFETY: each thread writes only its share's rows (the property under test)
    unsafe impl Send for Rows {}
    unsafe impl Sync for Rows {}

    /// The fills' shares at once, a thread each, one table (Miri's race detector sees any
    /// store of one share's into another's rows, AVX2's with `-C target-feature=+avx2`):
    /// one-row shares, one-record shares, and shares at the vector store's edges
    fn parallel_footprint(ll: &[u8]) {
        let n = ll.len();
        let counts = run_fed::counts_of(ll);
        let want = run_fed::production(ll, &counts);
        let recs = recs_of(ll);
        let r = recs.len() - 1;
        for k in kernels() {
            for h in [n, n / 8, n / 9 + 1, 3] {
                let h = h.max(1);
                let fo = sort_by_shares(&recs, &counts, n, h).expect("the shares' sort");
                let mut tt: Vec<u32> = Vec::with_capacity(n);
                let t = Rows(tt.as_mut_ptr());
                let ok = std::thread::scope(|sc| {
                    let hs: Vec<_> = (0..h)
                        .map(|j| {
                            let (fo, recs) = (&fo, &recs);
                            sc.spawn(move || {
                                let t = t;
                                let (q0, q1) = share(n, h, j);
                                // SAFETY: rows [q0, q1) of the n reserved, this share's
                                let out = unsafe {
                                    std::slice::from_raw_parts_mut(
                                        t.0.add(q0) as *mut MaybeUninit<u32>,
                                        q1 - q0,
                                    )
                                };
                                let mut tally = Tally::default();
                                fill_share(recs, fo, q0 as u32, q1 as u32, out, k, &mut tally)
                            })
                        })
                        .collect();
                    hs.into_iter().all(|x| x.join().unwrap())
                });
                assert!(ok);
                // SAFETY: every share filled its rows (asserted), and they tile [0, n)
                unsafe { tt.set_len(n) };
                assert!(tt == want, "hs, {h} shares at once, {k:?}");
            }
            for cuts in [
                (0..=r).collect::<Vec<_>>(),
                cuts_of(&recs, n, n, true),
                cuts_of(&recs, n, 3, true),
                cuts_of(&recs, n, (n / 8).max(1), false),
            ] {
                let hists = direct_hists(&recs, &counts, n, &cuts).expect("C31's prefix");
                let mut tt: Vec<u32> = Vec::with_capacity(n);
                let t = Rows(tt.as_mut_ptr());
                let ok = std::thread::scope(|sc| {
                    let hs: Vec<_> = hists
                        .iter()
                        .enumerate()
                        .map(|(j, hist)| {
                            let (recs, cuts, mut hist) = (&recs, &cuts, *hist);
                            sc.spawn(move || {
                                let t = t;
                                let mut tally = Tally::default();
                                // SAFETY: this share's byte ranges of the n reserved
                                unsafe {
                                    fill_direct(
                                        recs,
                                        cuts[j],
                                        cuts[j + 1],
                                        &mut hist,
                                        t.0,
                                        k,
                                        &mut tally,
                                    )
                                }
                            })
                        })
                        .collect();
                    hs.into_iter().all(|x| x.join().unwrap())
                });
                assert!(ok);
                // SAFETY: every share filled its ranges (asserted), which tile [0, n)
                unsafe { tt.set_len(n) };
                assert!(tt == want, "dir, {} shares at once, {k:?}", cuts.len() - 1);
            }
        }
    }

    #[test]
    fn miri_parallel_footprint() {
        if std::env::var_os("MIBZ2_EXPECT_AVX2").is_some() {
            assert!(Kernels::detect().avx2, "AVX2 not detected");
        }
        let len = if cfg!(miri) { 40 } else { 200 };
        parallel_footprint(&runs_column(len, 3, &[1, 7, 8, 9, 17], 31));
        parallel_footprint(&runs_column(len, 1, &[len], 32));
    }

    /// The reporter (bench/run_fed_leader.py) admits a run-fed line only if its phases are
    /// exactly its PHASES: this module's, in order
    #[test]
    fn phases_are_the_reporters() {
        let py = include_str!("../../bench/run_fed_leader.py");
        let start = py.find("\nPHASES = [").expect("the reporter's PHASES");
        let end = start + py[start..].find(']').unwrap();
        let listed: Vec<&str> = py[start..end].split('"').skip(1).step_by(2).collect();
        let names: Vec<&str> = run_fed_phases().into_iter().map(|p| p.0).collect();
        assert_eq!(listed, names);
    }

    // ---- C35's cuts ----

    /// The row cuts rise, start at 0 and end at r, on any records (unrisen ones included);
    /// on records that tile, each share's rows are n/h give or take one run either side
    #[test]
    fn row_cuts_partition_by_rows() {
        let mut rng = xorshift(77);
        for len in [1usize, 5, 64, 1000, 30_000] {
            for lens in [&[1usize][..], &[1, 2, 40], &[500]] {
                let ll = runs_column(len, 4, lens, len as u64);
                let recs = recs_of(&ll);
                let r = recs.len() - 1;
                let longest = recs
                    .windows(2)
                    .map(|w| (w[1] >> 8) - (w[0] >> 8))
                    .max()
                    .unwrap();
                for h in [1usize, 2, 3, 7, 64] {
                    for w in [0, RECORD_WEIGHT] {
                        let cuts = cuts_weighted(&recs, len, h, Some(w));
                        assert_eq!((cuts[0], cuts[h]), (0, r));
                        assert!(cuts.windows(2).all(|w| w[0] <= w[1]));
                        // each share's work, rows + w a record, is the total's h-th give or
                        // take one record's either side
                        let key = |i: usize| (recs[i] >> 8) as u64 + w * i as u64;
                        let total = len as u64 + w * r as u64;
                        for j in 0..h {
                            let work = key(cuts[j + 1]) - key(cuts[j]);
                            let (j0, j1) = (j as u64, j as u64 + 1);
                            let ideal = total * j1 / h as u64 - total * j0 / h as u64;
                            let slack = 2 * (longest as u64 + w);
                            assert!(
                                work.abs_diff(ideal) <= slack,
                                "len {len}, h {h}, w {w}, share {j}"
                            );
                        }
                    }
                }
            }
        }
        // records that don't rise: the cuts still a partition
        let recs: Vec<u32> = (0..50).map(|_| (rng() as u32) & 0xffff_ff00).collect();
        let mut cuts = vec![0; 6];
        row_cuts(&recs, 1 << 20, 5, RECORD_WEIGHT, &mut cuts);
        assert_eq!((cuts[0], cuts[5]), (0, recs.len() - 1));
        assert!(cuts.windows(2).all(|w| w[0] <= w[1]));
    }

    // ---- The common-origin timeline ----

    /// d = 20, L = 95, B = 100: the leader ends at 115, after the build at 100. Its
    /// duration alone (95 < 100) called it early.
    #[test]
    fn leader_late_from_one_origin() {
        let (d, l, b) = (20, 95, 100);
        let tl = Timeline {
            lead_start: d,
            lead_end: d + l,
            build_end: b,
        };
        assert!(tl.leader_late());
        let by_duration = |l: u64, b: u64| l > b;
        assert!(!by_duration(tl.lead_end - tl.lead_start, tl.build_end));
        assert!(!Timeline {
            lead_end: 100,
            ..tl
        }
        .leader_late());
        assert!(Timeline {
            lead_end: 101,
            ..tl
        }
        .leader_late());
    }

    /// A staggered start through `invert`: the leader starts LEAD_DELAY after t0 and does
    /// nothing, so its duration is far below the build's end, yet it ends after it; the
    /// run's timeline says late, from the same origin as the build's end
    #[test]
    fn a_late_started_leader_is_late() {
        let ll = runs_column(30_000, 5, &[8, 16, 20], 14);
        let b = symbols_of(&ll, 5);
        let p = Pool::new(3).unwrap();
        let k = Kernels::detect();
        for direct in [false, true] {
            let mut f = Front::default();
            if direct {
                front_stage::<DIR>(&b, &mut f, k, &mut 0, &mut 0, &mut false).unwrap();
            } else {
                front_stage::<HS>(&b, &mut f, k, &mut 0, &mut 0, &mut false).unwrap();
            }
            assert_eq!(f.path, Path::Records);
            let (mut tt, mut seg, mut out) = (TVector::new(), SegScratch::default(), Vec::new());
            let tl = invert(
                &p,
                &mut f,
                &b,
                k,
                &mut tt,
                &mut seg,
                &mut out,
                &|| {},
                direct,
                INJECT_LEAD_DELAY,
            )
            .unwrap();
            assert!(tl.lead_start >= LEAD_DELAY.as_nanos() as u64, "{tl:?}");
            assert!(
                tl.lead_start <= tl.lead_end && tl.build_end < tl.lead_start,
                "{tl:?}"
            );
            assert!(tl.leader_late(), "{tl:?}");
            // (the old predicate, its duration against B, called this run early)
            assert!(tl.lead_end - tl.lead_start < tl.build_end, "{tl:?}");
            assert!(out == want(&ll, 5).0);
        }
    }

    // ---- The census (untimed: counts, not times) ----

    /// C35's census of a corpus's team blocks by records, at the plan's helpers (h = 2, a
    /// team of three) and h = 3: each share's records r_h, rows n_h (its distinct words
    /// written), cells m_h and entries stored W_h, under C31's first cut (equal records),
    /// equal rows, C35's (rows + `RECORD_WEIGHT` a record), and C30's fill (equal rows by construction, its records the
    /// count's equal shares). Reported as the largest share over the mean (1: balanced),
    /// summed over blocks. `MIBZ2_RUN_FED_CENSUS=FILE.bz2 cargo test --release --features
    /// run-fed --lib census -- --ignored --nocapture`
    #[test]
    #[ignore = "a corpus's census: MIBZ2_RUN_FED_CENSUS=FILE.bz2"]
    fn census() {
        use crate::stream::{Event, Scanner, SliceSource};
        let Some(path) = std::env::var_os("MIBZ2_RUN_FED_CENSUS") else {
            return;
        };
        let data = std::fs::read(path).unwrap();
        let k = Kernels::detect();
        let mut sc = Scanner::new(SliceSource(&data));
        // [h][arm] -> (blocks, sum of max over shares, sum of totals) for r, n, m, W
        let arms = [
            "dir, equal records",
            "dir, equal rows",
            "dir, C35 (rows + 8 r)",
            "hs's fill, rows",
        ];
        let weights = [None, Some(0), Some(RECORD_WEIGHT)];
        let mut acc = [[[(0u64, 0u64); 4]; 4]; 2];
        let (mut blocks, mut all, mut lookups) = (0u64, 0u64, [[0u64; 4]; 2]);
        let mut sizes = (0u64, 0u64);
        while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
            all += 1;
            if !api::takes_runs(&symbols) {
                continue;
            }
            let (mut recs, mut counts) = (Vec::new(), [0; 256]);
            let n = mtf::mtf_runs(&symbols, &mut recs, &mut counts, k).unwrap();
            if n < MIN_COOP {
                continue;
            }
            blocks += 1;
            sizes.0 += n as u64;
            sizes.1 += recs.len() as u64 - 1;
            for (hi, h) in [2usize, 3].into_iter().enumerate() {
                for (ai, acc) in acc[hi].iter_mut().enumerate() {
                    let mut per: Vec<[u64; 4]> = Vec::new();
                    if ai < 3 {
                        let cuts = cuts_weighted(&recs, n, h, weights[ai]);
                        let mut hists = direct_hists(&recs, &counts, n, &cuts).unwrap();
                        let mut tt: Vec<u32> = Vec::with_capacity(n);
                        for (j, hist) in hists.iter_mut().enumerate() {
                            let mut t = Tally::default();
                            // SAFETY: merge_rows' ranges, inside the n reserved
                            assert!(unsafe {
                                fill_direct(
                                    &recs,
                                    cuts[j],
                                    cuts[j + 1],
                                    hist,
                                    tt.as_mut_ptr(),
                                    k,
                                    &mut t,
                                )
                            });
                            lookups[hi][ai] += t.lookups;
                            per.push([(cuts[j + 1] - cuts[j]) as u64, t.rows, t.cells, t.stores]);
                        }
                    } else {
                        let fo = sort_by_shares(&recs, &counts, n, h).unwrap();
                        let mut tt: Vec<u32> = Vec::with_capacity(n);
                        let r = recs.len() - 1;
                        for j in 0..h {
                            let (q0, q1) = share(n, h, j);
                            let mut t = Tally::default();
                            let out = &mut tt.spare_capacity_mut()[q0..q1];
                            assert!(fill_share(&recs, &fo, q0 as u32, q1 as u32, out, k, &mut t));
                            let (lo, hi) = share(r, h, j);
                            per.push([(hi - lo) as u64, t.rows, t.cells, t.stores]);
                        }
                    }
                    for (i, a) in acc.iter_mut().enumerate() {
                        a.0 += per.iter().map(|p| p[i]).max().unwrap();
                        a.1 += per.iter().map(|p| p[i]).sum::<u64>();
                    }
                }
            }
        }
        println!(
            "census: {blocks} team blocks by records of {all} blocks; r/n {:.4}; mean n {:.0}",
            sizes.1 as f64 / sizes.0.max(1) as f64,
            sizes.0 as f64 / blocks.max(1) as f64
        );
        for (hi, h) in [2usize, 3].into_iter().enumerate() {
            for (ai, arm) in arms.iter().enumerate() {
                let a = acc[hi][ai];
                let imb = |i: usize| a[i].0 as f64 * h as f64 / a[i].1.max(1) as f64;
                println!(
                    "h {h}, {arm:<22}: largest share / mean: r_h {:.4}, n_h {:.4}, m_h {:.4}, \
                     W_h {:.4}; W/n {:.4}, m/n {:.4}, lookups a block {:.1}",
                    imb(0),
                    imb(1),
                    imb(2),
                    imb(3),
                    a[3].1 as f64 / a[1].1.max(1) as f64,
                    a[2].1 as f64 / a[1].1.max(1) as f64,
                    lookups[hi][ai] as f64 / blocks.max(1) as f64,
                );
            }
        }
    }
}
