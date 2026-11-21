//! `just bench-recalibrate-058`'s arms (docs/journal/2026-11-21-recalibration-058-registration.md,
//! the design of docs/journal/2026-11-18-recalibration-registration.md on master 70b18ca4): at
//! each cell, the arm the run names as the shipped trim is the decoder the default builds on
//! the measured i9 there (`StreamDecoder::new` under `with_machine`), in its worker count,
//! its window and its kernel, pinned one CPU a core (`Machine::pinned_i9`) or unpinned
//! (`Machine::service_target`); and the arm it names as the untrimmed plan is the decoder the
//! plan builds there (`Plan::for_threads`, `StreamDecoder::planned`). The neighbours are the
//! shipped arm but for one worker more or fewer, 4 blocks a worker.
use mibz2::parallel::{with_machine, Machine, Plan};
use mibz2::{Kernels, StreamDecoder};

/// This machine's kernels: the trims hold only with both AVX2 and CLMUL
fn kernels() -> Kernels {
    Kernels::detect()
}

/// An arm's decoder as `ab` builds it: `mibz2-occ-N-S` lazy, `mibz2-oce-N-S` eager
fn arm(name: &str) -> StreamDecoder {
    let (lazy, rest) = match name.strip_prefix("mibz2-occ-") {
        Some(r) => (true, r),
        None => (
            false,
            name.strip_prefix("mibz2-oce-").expect("an occupancy arm"),
        ),
    };
    let (n, s) = rest.split_once('-').expect("N-S");
    StreamDecoder::workers(n.parse().unwrap(), s.parse().unwrap(), kernels(), lazy)
}

fn shape(d: &StreamDecoder) -> (usize, u64, bool) {
    (d.threads(), d.window(), d.is_lazy())
}

/// (pinned, threads asked, the shipped arm, the untrimmed plan's arm)
const CELLS: [(bool, usize, &str, &str); 11] = [
    (true, 10, "mibz2-occ-7-28", "mibz2-occ-10-40"),
    (true, 8, "mibz2-occ-6-24", "mibz2-occ-8-32"),
    // lazy at 6 from 0.5.7 (#642), pinned and unpinned; the plan there is still eager
    (true, 6, "mibz2-occ-5-20", "mibz2-oce-6-24"),
    (true, 4, "mibz2-oce-3-12", "mibz2-oce-4-16"),
    (false, 4, "mibz2-oce-4-16", "mibz2-oce-4-16"),
    (false, 6, "mibz2-occ-6-24", "mibz2-oce-6-24"),
    (false, 8, "mibz2-occ-8-32", "mibz2-occ-8-32"),
    (false, 10, "mibz2-occ-7-28", "mibz2-occ-10-40"),
    (false, 12, "mibz2-occ-7-28", "mibz2-occ-10-48"),
    (false, 16, "mibz2-occ-7-28", "mibz2-occ-10-64"),
    (false, 20, "mibz2-occ-7-28", "mibz2-occ-10-80"),
];

#[test]
fn the_shipped_and_untrimmed_arms_are_the_defaults() {
    // the trims are the measured i9's alone, on Linux x86-64 with both kernels
    let k = kernels();
    if !(cfg!(all(target_os = "linux", target_arch = "x86_64")) && k.avx2() && k.clmul()) {
        return;
    }
    for (pinned, t, shipped, untrimmed) in CELLS {
        let m = || {
            if pinned {
                Machine::pinned_i9(t)
            } else {
                Machine::service_target()
            }
        };
        let prod = with_machine(m(), || StreamDecoder::new(t, 4 * t, k));
        let plan = with_machine(m(), || {
            StreamDecoder::planned(Plan::for_threads(t), 4 * t, k)
        });
        let (s, u) = (arm(shipped), arm(untrimmed));
        assert_eq!(
            shape(&prod),
            shape(&s),
            "{pinned} {t}: the default is {shipped}"
        );
        assert_eq!(
            shape(&plan),
            shape(&u),
            "{pinned} {t}: the plan is {untrimmed}"
        );
        for d in [prod, plan, s, u] {
            d.close();
        }
    }
}

#[test]
fn the_neighbours_differ_in_one_worker() {
    for (shipped, fewer, more) in [
        ("mibz2-occ-7-28", "mibz2-occ-6-24", "mibz2-occ-8-32"),
        ("mibz2-occ-6-24", "mibz2-occ-5-20", "mibz2-occ-7-28"),
        ("mibz2-occ-5-20", "mibz2-occ-4-16", "mibz2-occ-6-24"),
        ("mibz2-oce-3-12", "mibz2-oce-2-8", "mibz2-oce-4-16"),
        ("mibz2-oce-4-16", "mibz2-oce-3-12", "mibz2-oce-5-20"),
        ("mibz2-occ-8-32", "mibz2-occ-7-28", "mibz2-occ-9-36"),
    ] {
        let (s, f, m) = (arm(shipped), arm(fewer), arm(more));
        let (n, w, lazy) = shape(&s);
        assert_eq!(shape(&f), (n - 1, w - 4, lazy), "{fewer}");
        assert_eq!(shape(&m), (n + 1, w + 4, lazy), "{more}");
        for d in [s, f, m] {
            d.close();
        }
    }
}

/// The other kernel at a cell's shipped count (`mibz2-occ-N-S` beside `mibz2-oce-N-S`) differs
/// from the shipped arm in its kernel alone: the lazy kernel at the eager cells (pinned 4,
/// unpinned 4), and the plan's eager kernel at unpinned 6, where the shipped trim is lazy
#[test]
fn the_kernel_arms_differ_in_the_kernel() {
    for (eager, lazy) in [
        ("mibz2-oce-3-12", "mibz2-occ-3-12"),
        ("mibz2-oce-4-16", "mibz2-occ-4-16"),
        ("mibz2-oce-6-24", "mibz2-occ-6-24"),
    ] {
        let (e, l) = (arm(eager), arm(lazy));
        let ((n, w, el), (ln, lw, ll)) = (shape(&e), shape(&l));
        assert_eq!((n, w, el, ll), (ln, lw, false, true), "{eager} {lazy}");
        e.close();
        l.close();
    }
}
