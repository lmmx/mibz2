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
    finish_and_publish, finish_block, panic_message, publish, take_out, try_job, Inverted, Job,
    Lead, Shared, Solo, Staged, Staging, StreamDecoder,
};
use crate::block::Symbols;
use crate::bwt::{self, SegScratch, TVector, FLAGGED_LANES, FLAGGED_SEGMENTS, MIN_COOP};
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
    /// each fills its runs' F intervals directly, reading L by predecessor lookup
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
    join_wait_ns: "the leader waiting for the build after its work",
    build_ns: "a run's start to its walk's set-up done (the helpers' build)",
    run_ns: "a run's whole span",
    leader_late: "runs whose leader ended its work after the build",
    helper_sort_ns: "the helpers' count, prefix and scatter (summed, waits included)",
    helper_fill_ns: "the helpers' fills from records (summed)",
    helper_fused_ns: "the helpers' column builds (summed)",
    drained_front_ns: "the MTF or emitter and sort with no run in flight",
    records: "team blocks' run records (r, summed)",
    rows: "team blocks' rows by records (n, summed)",
    stores: "table entries the record fills stored (W, summed: n <= W)",
    lookups: "the direct fill's predecessor lookups (C31)",
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
        Ok(Ok(n)) if n >= MIN_COOP => Staging::Team(s),
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

/// One cell, rows [q, stop): `T[q + i] = base + 256·i`; with `AVX`, 8 entries a store
/// while a store ends at or before `end` (the rows this thread owns), then single entries.
/// A store past `stop` writes rows of this thread's next cell, which that cell writes
/// again (C32: never past `end`, another thread's). Returns the entries stored.
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
/// records that don't tile, which the checks before it refuse); adds to `stores` the
/// entries it stored (C32's W). A bad index panics.
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
    stores: &mut u64,
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
        *stores += unsafe { cell::<AVX>(t, q, stop, q1, base) };
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
    stores: &mut u64,
) -> bool {
    // SAFETY: the caller's contract
    unsafe { fill_rows_at::<true>(recs, fo, q0, q1, t, stores) }
}

/// `fill_rows_at` on the kernels given: rows [q0, q1) into `out`, its q1 − q0 slots
fn fill_share(
    recs: &[u32],
    fo: &[[u32; 2]],
    q0: u32,
    q1: u32,
    out: &mut [MaybeUninit<u32>],
    k: Kernels,
    stores: &mut u64,
) -> bool {
    assert_eq!(out.len(), q1.saturating_sub(q0) as usize);
    // (the rows' pointer, offset back by q0: only rows [q0, q1) are written through it)
    let t = (out.as_mut_ptr() as *mut u32).wrapping_sub(q0 as usize);
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected; rows [q0, q1) of t are
            // `out`'s, held mutably here
            return unsafe { fill_rows_avx2(recs, fo, q0, q1, t, stores) };
        }
    }
    let _ = k;
    // SAFETY: as above
    unsafe { fill_rows_at::<false>(recs, fo, q0, q1, t, stores) }
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
    let mut stores = 0;
    let ok = fill_share(
        recs,
        fo,
        0,
        n as u32,
        &mut tt.0.spare_capacity_mut()[..n],
        k,
        &mut stores,
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

/// C31's fill: share [lo, hi) of the records, each run's F interval [a, a + ℓ) written
/// directly (a from `merge_rows`' cursor, advanced by ℓ), its bytes L[q] found by one
/// predecessor lookup a byte (the first of its runs in the share: the byte's F rows in the
/// share are consecutive, so the L rows they read are too) and then by walking the L runs
/// on. Writes only inside each byte's range ([hist[1][c], hist[2][c]), the share's own);
/// returns whether every range was filled exactly, and adds its lookups and stores.
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
    lookups: &mut u64,
    stores: &mut u64,
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
            *lookups += 1;
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
            *stores += unsafe { cell::<AVX>(t, q, stop, end, base) };
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
    lookups: &mut u64,
    stores: &mut u64,
) -> bool {
    // SAFETY: the caller's contract
    unsafe { fill_direct_at::<true>(recs, lo, hi, hist, t, lookups, stores) }
}

/// `fill_direct_at` on the kernels given
///
/// SAFETY: as `fill_direct_at`'s, but for AVX2's availability, which `k` says
#[allow(clippy::too_many_arguments)]
unsafe fn fill_direct(
    recs: &[u32],
    lo: usize,
    hi: usize,
    hist: &mut [[u32; 256]; 3],
    t: *mut u32,
    k: Kernels,
    lookups: &mut u64,
    stores: &mut u64,
) -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: the caller's, and AVX2 was detected
            return unsafe { fill_direct_avx2(recs, lo, hi, hist, t, lookups, stores) };
        }
    }
    let _ = k;
    // SAFETY: the caller's
    unsafe { fill_direct_at::<false>(recs, lo, hi, hist, t, lookups, stores) }
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
        inject: u8,
    ) -> RunFill<'a> {
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
            inject,
        }
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

    /// Share j's count (the helpers' sort)
    fn count(&self, j: usize) {
        self.injected(INJECT_COUNT, j);
        let (lo, hi) = share(self.r, self.shares, j);
        // SAFETY: entry j is this share's alone until the counted gate
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

    /// Share j's scatter, if the merge passed
    fn scatter(&self, j: usize) {
        if !self.sorted.load(Ordering::Relaxed) {
            return;
        }
        self.injected(INJECT_SCATTER, j);
        let (lo, hi) = share(self.r, self.shares, j);
        // SAFETY: entry j is this share's; its ranges are merge_shares', disjoint from
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

    /// Share j's rows of the table from the F order, if it is whole and checked
    fn fill(&self, j: usize, k: Kernels) {
        if !self.sorted.load(Ordering::Relaxed) {
            return;
        }
        self.injected(INJECT_FILL, j);
        let bit = 1u64 << j;
        if self.claimed.fetch_or(bit, Ordering::Relaxed) & bit != 0 {
            return;
        }
        let mut stores = 0;
        if self.direct {
            let (lo, hi) = share(self.r, self.shares, j);
            let mut lookups = 0;
            // SAFETY: entry j is this share's, merge_rows': its ranges are disjoint from
            // every other share's and inside [0, n), the n rows reserved, the buffer held
            let ok = unsafe {
                fill_direct(
                    self.recs,
                    lo,
                    hi,
                    &mut *self.hist.add(j),
                    self.tt_p,
                    k,
                    &mut lookups,
                    &mut stores,
                )
            };
            add(&PH.lookups, lookups);
            add(&PH.stores, stores);
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
        let ok = fill_share(self.recs, fo, q0 as u32, q1 as u32, out, k, &mut stores);
        add(&PH.stores, stores);
        if ok {
            self.filled.fetch_or(bit, Ordering::Release);
        }
    }

    /// The table, its length set only if every share filled its rows; else the block
    /// rebuilt by the column's path from its symbols (a bug's path, counted)
    fn finish(self, b: &Symbols, k: Kernels) -> Result<&'a mut TVector, ErrorKind> {
        let ok = self.sorted.load(Ordering::Relaxed)
            && self.filled.load(Ordering::Acquire) == self.all();
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
    build_ns: AtomicU64,
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

    /// One helper's build: its share of the fused build, or of the run-fed sort and fill
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
        self.build_ns.store(since(self.t0), Ordering::Relaxed);
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
/// `lead`, then joins the walk if it is still going. Exactly `bwt::walk`'s output.
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
) -> Result<(), Box<dyn Any + Send>> {
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
        build_ns: AtomicU64::new(0),
    };
    let relay = &relay;
    let lead_ns = AtomicU64::new(0);
    let helper = |j: usize| {
        relay.build_share(j, helpers);
        // SAFETY: the gate's last helper is the last to count in: every share has ended
        relay.built.pass(helpers, || unsafe { relay.end_build() });
        relay.walk_and_leave();
    };
    let leader = || {
        let t = Instant::now();
        lead();
        let l = since(t);
        add(&PH.lead_ns, l);
        lead_ns.store(l, Ordering::Relaxed);
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
    add(&PH.build_ns, relay.build_ns.load(Ordering::Relaxed));
    if lead_ns.load(Ordering::Relaxed) > relay.build_ns.load(Ordering::Relaxed) {
        add(&PH.leader_late, 1);
    }
    let caught = lock(&relay.panic).take();
    caught.map_or(Ok(()), Err)
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
        let leader = || {
            let mut l = lock(&lead);
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
            Ok(Ok(())) => {
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
mod tests {
    use super::*;
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

    /// C31 in `h` shares run one after another: the table, or None if refused
    fn direct_by_shares(
        recs: &[u32],
        counts: &[u32; 256],
        n: usize,
        h: usize,
        k: Kernels,
    ) -> Option<Vec<u32>> {
        let r = recs.len().checked_sub(1)?;
        let mut hists = vec![[[0u32; 256]; 3]; h];
        let mut rise = true;
        for (j, hist) in hists.iter_mut().enumerate() {
            let (lo, hi) = share(r, h, j);
            rise &= count_rows(recs, lo, hi, hist);
        }
        if !(rise && merge_rows(recs, counts, n, &mut hists)) {
            return None;
        }
        let mut tt: Vec<u32> = Vec::with_capacity(n);
        let (mut all, mut lookups, mut stores) = (true, 0, 0);
        for (j, hist) in hists.iter_mut().enumerate() {
            let (lo, hi) = share(r, h, j);
            // SAFETY: merge_rows' ranges, inside the n reserved, one share at a time
            all &= unsafe {
                fill_direct(
                    recs,
                    lo,
                    hi,
                    hist,
                    tt.as_mut_ptr(),
                    k,
                    &mut lookups,
                    &mut stores,
                )
            };
        }
        assert!(lookups <= 256 * h as u64);
        if !all {
            return None;
        }
        // SAFETY: every row written (each range filled exactly, the ranges tiling [0, n))
        unsafe { tt.set_len(n) };
        Some(tt)
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
                let mut stores = 0;
                for j in 0..h {
                    let (q0, q1) = share(n, h, j);
                    let out = &mut tt.spare_capacity_mut()[q0..q1];
                    assert!(fill_share(
                        &recs,
                        &fo,
                        q0 as u32,
                        q1 as u32,
                        out,
                        k,
                        &mut stores
                    ));
                }
                // SAFETY: the shares tile [0, n), each filled whole (asserted)
                unsafe { tt.set_len(n) };
                assert!(tt == want, "n {n}, {h} shares, {k:?}: the table");
                assert!(stores >= n as u64 && stores <= n as u64 + 7 * 2 * recs.len() as u64);
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
        let b = symbols_of(ll, orig_ptr);
        let mut f = Front::default();
        let k = Kernels::detect();
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
}
