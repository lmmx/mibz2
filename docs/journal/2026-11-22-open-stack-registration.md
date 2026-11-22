# bench-open-stack: registration

*22 November 2026. Written and frozen before any data, on `bench/open-stack`. The recipe is
`just bench-open-stack` (`.just/bench.just`); the verdict is `bench/open_stack.py`'s, tested by
`bench/test_open_stack.py`. The recipe, the rule and the verdict script are bench-open-graded's
(#715, `docs/journal/2026-11-22-open-graded-registration.md` on `bench/open-graded`), themselves
bench-open-relay8's (#707), bench-open-diet's (#693) and bench-open-r4's (#680), with the same
median peak RSS report. The builds differ, and one diagnostic is added (the nobi pass, below),
which decides nothing and runs after the judged run, outside it.*

**The decision it settles:** whether #719 (`perf/open-neon-c29-on-714`, "perf(open):
construction stack (Neon C29 + Wormhole L4/L5) on #714") reaches the decoder's default on the
i9, that is, whether its construction stack and two-ended emission, with #714 beneath them,
merge. It is an ordinary merge under PERFORMANCE-POLICY.md, measured on the one machine where
production runs `OpenCells` on the i9's cache: CI's bench never runs Open there
(`docs/journal/2026-11-20-open-stage-attribution.md`). #719 is stacked on #714
(`perf/open-wormhole-15-17`); this run measures the whole stack, #714's two commits included,
against master.

## The hypothesis

**What changed.** #719's head is master's `8ff1187a` plus eleven commits: #714's two
(GradedHandle and Prefix15, Wormhole cycles 15 and 17) and #719's nine. They change
`src/open.rs`, `src/open/b5.rs`, `src/open/checks.rs`, `src/open/dense.rs`,
`src/open/endpoint.rs`, `src/open/outq.rs` and `src/open/spirals.rs` (970 lines added, 163
removed), in safe Rust, with no allocator or routing change:
- **the construction stack:** GradedHandle and Prefix15 (#714); BidiBottom (Wormhole L4, bottom
  coordinates from both ends of side one); pass-one totals (thin and thick row counts, each
  symbol's total derived after pass one); strip and band packing (capped strip records in the
  edge arena's prefix; band coordinates and dictionary code in 64 bits); short-rule payloads
  (short rule spellings stored directly in the rule arena); long spelling ranges in the inline
  dictionary; an interned dictionary (constructed words interned in the spent arena prefix or
  tail); a 48 KiB reserve for full-block long spellings;
- **two-ended emission** (the last commit, `ccdb843a`): reversible neighbours wired, and the
  marked cycle emitted from both ends.

**The native evidence (#719's body).** Measured in a shared container (AMD EPYC 9V45, four-CPU
quota, rustc 1.99.1), against master's kernel in the same binary, 12 fresh-process rounds,
paired 95% intervals:
- **The kernel at one thread:** the six-block sum **0.872 [0.869, 0.875]** of master's (A/A
  1.005 [1.002, 1.008]).
- **Four concurrent workers:** a worker's six-block pass **0.880 [0.877, 0.883]** (A/A 1.001
  [0.998, 1.005]); two workers 0.882 [0.879, 0.886].
- **The whole Wikidata stream** (`OpenCells` forced): **0.925 [0.882, 0.971]** at one worker and
  0.912 [0.857, 0.970] at four.
- **Memory:** retained Open scratch per worker after the six blocks 1,200,839 to 968,484 B
  (**−19.3%**), after all 431 blocks 1,377,699 to 1,128,960 B (−18.1%).
- **By commit:** the two-ended emission is the largest single step, **0.941 [0.937, 0.945]** at
  one thread and 0.943 [0.940, 0.946] at four workers against its parent. The stack below it
  reaches 0.924 and 0.931 of master; S4 grows (1.721) and S5 shrinks (0.791) with the
  reversible wiring.

**Why it should hold under contention.** Construction and memory gains have held on the i9
before. #692's memory diet (native four-worker pass 0.960 [0.934, 0.987]) measured 0.952, 0.917
and 0.931 at 6, 8 and 10 threads (`open-diet/run-20261121T121714Z`, ADOPTS); #703's dense
admission held there too (peak memory about 28% lower from 8 to 20 threads, and faster;
CHANGELOG). Most of #719's stack is of that kind: work removed from construction
(S1 to S4) on the block's own arrays, and about 19% less retained scratch a worker, which is
what made the diet's gain grow under contention. The native four-worker pass, 0.880, already
holds under four workers. If it holds on the i9, production's decoder gets faster at 6, 8 and 10
threads by the Open kernel's share of the wall, by less than the kernel's 12%. The cause is a
hypothesis, not shown.

**The risk, named: two-ended emission under shared-cache contention.** It is the stack's largest
single step natively, and it is an emit change (S5), where the record on the i9 is mixed:
- a related two-ended emit, #718's Fused-XOR, lost at 4 workers;
- an 8-chain emit relay (#705, Relay8) was neutral on the i9.

A plausible mechanism, not shown: emission walks the output and the cycle's links, which may
compete for the i9's shared L3 at 7 to 8 workers in a way a four-CPU container does not show. If that step erodes or reverses there, the
stack's gain at 10 threads shrinks, and the judged count may land INCONCLUSIVE. The nobi pass
below shows that step's share on the i9; it decides nothing.

## What is compared

- **Head:** #719's head, `perf/open-neon-c29-on-714` at
  `ccdb843ac5c15f35fb2b20e8e0c4cff089ccc303`. Its src differs from the base in the seven
  `src/open*` files above and nothing else; its other changes are #719's journal
  (`docs/journal/2026-11-22-open-construction/`), which no build reads.
- **Base:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5`, #719's merge-base with master (#713's
  merge, and #714's base). Its src is v0.5.11's: the Open kernel with the memory diet (#692) and
  dense-block admission (#703). It is an ancestor of HEAD; bench-open-r4's ancestry check
  (`git merge-base --is-ancestor "$BASE" "$HEAD"`) is kept word for word.
- **Base and master agree.** When the recipe was written, master's head *was* the base,
  `8ff1187af`, so the base's build inputs (`src`, `examples`, `Cargo.toml`, `Cargo.lock`,
  `build.rs`, `rust-toolchain.toml`) equal current master's. **So this run measures #719's stack
  against master's head.** Should master move before the run, the run still measures it against
  its own base; the Coordinator weighs any later master change to `src` at the merge.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock` and `build.rs` may differ only in `src/open.rs` and files under `src/open/`, or
  the run stops.
  - At these SHAs the changed files are `src/open.rs`, `src/open/b5.rs`, `src/open/checks.rs`,
    `src/open/dense.rs`, `src/open/endpoint.rs`, `src/open/outq.rs` and `src/open/spirals.rs`.
    The bundle keeps the list as `src-changed.txt`.
  - Both builds use rustc 1.99.1 (`rust-toolchain.toml`, the same at both), and the head's
    harness drives the run. `examples/ab.rs` is the same at both.
- **Admission:** as read from the diff, unchanged. The refusal test still compares the block's
  strip count with the cap (`ns > cap.strips`, where the base had `self.strips.len()`); the
  strips now live in the edge arena's prefix. The same blocks should take Open in both builds.
  The single-hash check over every run's output guards the decoded bytes either way.
- **What it predicts:** if the stack's gain holds under contention, a whole-decoder wall gain at
  10 threads (7 workers), 8 (8 workers) and 6 (6 workers). If contention erodes it, a smaller
  or level step. For scale, on the same grid: bench-open-diet measured 0.952, 0.917 and 0.931 at
  6, 8 and 10 threads, ADOPTS; bench-open-r4 measured 0.956, 0.982 and 0.985 [0.970, 1.001], NOT
  ADOPTED by 0.001 at 10.
- **Memory, reported:** expected flat or lower. #719 removes about 230 KiB of retained scratch a
  worker (of a process peak in the hundreds of MiB), so the 1.10 gate is not expected to bind.
  The verdict prints each count's median peak RSS at the base and the head, in MiB, and their
  difference. Only the 1.10 gate decides anything.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5.
  Any other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (SHA-256 `82f65ace…`, 431
  blocks, 209.7 MB decoded), checked by its digest. Four copies end to end make the timed file
  (`wikidata-x4`), as the i9 recipes do.
- **The cells:** `ab run --arms mibz2@build-base,mibz2`, production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner), at 6, 8 and 10 threads. Production runs
  `OpenCells` on the i9 at each of these (unchanged since bench-open-diet):
  - 6 threads: 6 lazy workers;
  - 8 threads: 8 lazy workers;
  - 10 threads (production's default on the i9): 7 workers, `trimmed`, unpinned.

  Production also runs `OpenCells` at exactly 4 threads unpinned (#678). That count is not in
  this run, which keeps bench-open-graded's grid so the results compare directly.
- **Rounds:** 16. Each round runs both arms at each count, interleaved, with the arms' order
  rotated (`arm_order`) and a 2 s settle gate before each run.
- **The lazy diagnostic** (decides nothing): `mibz2-workers-lazy` at each build, the workers'
  lazy route with plan and window fixed, at 6 and 8 threads, 16 rounds, reported as steps only.
  As in bench-open-graded, it follows the judged run.

### The nobi pass (a diagnostic; decides nothing)

- **The build:** `NOBI`, `dbfa95db28f2ad1c8a019ad8a3fa22c2e490b2af`, HEAD's parent: #719's head
  without its last commit, "wire reversible neighbors and emit the marked cycle from both ends".
  It runs as `mibz2@build-nobi`. The recipe checks that NOBI is exactly `HEAD^` and that HEAD and
  NOBI's src differs only in `src/open.rs` and `src/open/**` (at these SHAs `src/open.rs`,
  `src/open/checks.rs`, `src/open/endpoint.rs` and `src/open/outq.rs`; kept as
  `src-changed-nobi.txt`).
- **Why a separate pass, not a third arm in the judged run.** A third arm would change the
  judged run's shape: three arms a round instead of two, a different `arm_order` rotation, and
  each base–head pair further apart in time and heat. That would no longer be R4's comparison.
  So the judged run stays exactly bench-open-graded's two arms, and NOBI runs in its own pass
  after the lazy diagnostic. NOBI is also built only after the judged run, so nothing that
  happens before the judged run changes, and a failed NOBI build or run leaves the verdict and
  the bundle as they are.
- **The cells:** `ab run --arms mibz2@build-base,mibz2@build-nobi,mibz2`, unpinned, at 6, 8 and 10
  threads, 16 rounds, settle 2 s, on the same `wikidata-x4`.
- **The report** (`bench/open_stack.py --diagnostic --nobi`, `nobi.txt`): at each count, the
  steps **head/base**, **head/nobi** and nobi/base, each a geometric mean with its 95% t interval
  over the pass's own paired rounds, with the CPU step and the median peak RSS ratio beside it.
  The same validity checks apply to all three arms' full roster. head/nobi is the two-ended
  emission's own step on the i9; head/base here is a second look at the judged ratio, from a
  different run shape, and never replaces it.
- **What it can show:** whether the step that gave 0.941 natively helps, does nothing or hurts
  under the i9's contention. A head/nobi SLOWER at 10 threads, say, would point at the named
  risk whatever the verdict. Any follow-up it suggests (dropping the commit, say) would be its
  own registered run.
- **Cost:** about 5–8 minutes (144 runs of a fraction of a second to a few seconds each, settle
  included), plus NOBI's build.

**The whole run's cost:** about 20–25 minutes of workstation time with the three builds (the
judged run and the lazy diagnostic alone took under 10 minutes in bench-open-relay8's bundle),
below PERFORMANCE-POLICY §4a's 30-minute line.

## The rule

bench-open-graded's (R4's), unchanged. A count's **step** is the 16 rounds' paired wall
ratios, head over base: their geometric mean and its 95% interval (Student's t on the logs, 15
degrees of freedom). A step is FASTER if the interval lies below 1, SLOWER if it lies above 1,
and INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the step at 10 threads is FASTER (its interval lies below 1);
  - neither 6 nor 8 threads is SLOWER;
  - at each of 6, 8 and 10 threads, the median peak RSS over the rounds, head over base, is
    at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at some count. The Coordinator
  weighs the growth under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** any other outcome: 10 threads not FASTER, or 6 or 8 SLOWER.
- 6 and 8 threads are **supporting**: they are reported with their intervals, and only a loss
  there counts against adoption.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile;
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run is pinned or beside a spinner;
  - any record is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one.

  A VOID run may be repeated once, with the reason logged. A verdict is never re-run to change
  it.
- **Reported beside each step, deciding nothing:** CPU time, and the median peak RSS at the
  base and the head in MiB with their difference.
- **Neither diagnostic decides anything.** The verdict is the judged run's alone (`verdict.txt`).
  A VOID lazy or nobi diagnostic leaves the verdict as it is.

**Limits:**
- **The corpus:** Wikidata alone, the decoder's primary workload and where Open admits most of
  the rows. `text` is not in this run.
- **What a result separates:** an ADOPTS shows the whole stack's whole-decoder gain, #714's two
  commits included, not its cause. Only the two-ended emission's share is looked at, by the nobi
  pass, and that decides nothing. A NOT ADOPTED does not say which commit failed to carry.
- **#714:** its own run, bench-open-graded (#715), measures #714 alone against the same base. The
  two runs share a base and a grid, so their ratios compare, but each decides only its own PR.
- **The head's branch:** the recipe fetches HEAD and NOBI from `perf/open-neon-c29-on-714` if the
  workstation lacks them. A force-push that drops ccdb843a from every ref on origin would stop
  the run at the fetch, not change what it measures.
