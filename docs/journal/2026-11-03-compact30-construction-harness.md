# Compact30: the construction-cost harness, and its untimed VM validation

> **Superseded in part:** reports 195–197 found validation weaknesses in this harness (A1,
> G2, G9, G7, the memory diagnostics). They are repaired at `14d9ecb`; see
> [2026-11-03-compact30-construction-harness-repair.md](2026-11-03-compact30-construction-harness-repair.md).
> This note is kept as it was for `e264eed`.

The measurement harness that report 194 authorises, for protocol draft 3
([2026-11-03-compact30-construction-protocol.md](2026-11-03-compact30-construction-protocol.md)).
This covers implementation and untimed validation on the VM only.

- **No measured benchmark was run,** and nothing here is a performance figure. The dry runs
  read clocks because the machinery does, but their numbers are not measurements. They are
  not reported, interpreted or kept, except as evidence that the machinery ran.
- **No i9 or workstation run.** The proposed i9 command (§7) is **not authorised**.
- **Unchanged:** `Ctor::build`, the walker, the record format and production `src/`.
  `examples/c30walk/ctor.rs` is byte for byte the audited file (blob `73b3179`).

## 1. The pins

- **The harness:** `e264eed` on `research/compact30-constructor`. Its code, its analysis
  and tests, and the reference listings.
- **The recipe:** `just bench-c30-construction` (`.just/bench.just`), added in the commit
  after it, with `pin=e264eed7e1a1c2f1d456d40207ff302811df2eae`. Like the traversal recipe,
  it builds the pin in its own worktree (`target/c30-construction-pin`).
- **The toolchain:** rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6) via `rustup run 1.97.0`,
  `CARGO_ENCODED_RUSTFLAGS=` (empty), `--locked`, the release profile (codegen-units 1,
  fat LTO, line tables), baseline x86-64.
- **The constructor's reviewed pin:** `e003a32` (audit PASS, report 194).

## 2. What was built

| file | lines | what it is |
|---|--:|---|
| `examples/c30walk/cbuild.rs` | 1,270 | the collector: modes `cbuild` and `cbuild-rss` |
| `examples/c30walk.rs` | +12 | the module declaration, the two modes' dispatch, usage lines; nothing removed |
| `bench/c30_construction.py` | 558 | the analysis: S0–S10, G1–G10, the decision table |
| `bench/test_c30_construction.py` | 612 | 61 synthetic regression tests |
| `bench/c30_build_listing.py` | 199 | G3: the timed path's normalised listings against reviewed references |
| `bench/test_c30_build_listing.py` | 82 | 6 tests of that comparison |
| `docs/journal/2026-11-03-compact30-construction-harness/listings/` | | the reference listings (§5) |
| `.just/bench.just` | | `bench-c30-construction` |

### The collector, `c30walk cbuild`

```
c30walk cbuild OUT.jsonl CORPUS.bz2 --rounds R --cpus A,B,C,D [--dry-run]
    [--peak-rss BYTES] [--inject KIND]
```

**The arms (§2).** Each is timed around one call to an `#[inline(never)]` wrapper, so each
timed function is a single symbol G3 can check:
- **P:** `p_build(ll, counts, &mut tt)`, which calls `mibz2::bwt::build`;
- **C:** `c_build(&mut ctor, ll, counts, &mut rec, &mut raw)`, which calls
  `Ctor::build::<false>(…, permille 0, …)`.

Nothing is done for either builder outside its call. The arenas are whatever the builder
left (warm) or empty (cold).

**The window,** in both regimes: `Instant::now`, `CLOCK_THREAD_CPUTIME_ID`,
`getrusage(RUSAGE_THREAD)`, the call, `getrusage`, the CPU clock, `Instant::now`.
- Each build record carries both clocks' readings separately, and the window's minor and
  major faults.
- `sched_getcpu` is read before the start barrier (W4) or before the call (W1), and after
  the completion barrier.

**The preamble, untimed, in this order:**
1. **A sizing pass:** the scanner and MTF only, one block at a time, for each block's rows
   and counts.
2. **The memory preflight (G9):**
   - the estimate is computed from those sizes, with every part recorded (§3 of the
     protocol: inputs, production's output, oracle images, P references, eight workers'
     arenas, the round −1 walkers' scratch, the oracle's transient, the decode's transient,
     a fixed 64 MiB);
   - the requirement is max(estimate, `--peak-rss`), and the margin max(2 GiB, 25%);
   - also read: `MemAvailable`, swap in use, `/proc/pressure/memory`, `pswpin`/`pswpout`,
     and the process's VmRSS, VmHWM and major faults.

   The `preflight` record is written. If the preflight fails, a failure record follows and
   the run stops before any large allocation.
3. **The decode:** the parent's `blocks()`, the last columns with production's
   `Segmented::invert` output.
4. **The one-pass precondition (G8):** each block's histogram recomputed and compared with
   its counts, and n < 2²⁴. One `onepass` record per block.
5. **The oracle images** (`C30::build`, permille 0) and **the untimed counter pass**
   (`build::<true>` into its own arena, also compared with the image). One `counters`
   record per block.
6. **The meta record,** then the `before` calibration: 20,000 empty windows a CPU, pinned,
   with the window's own calls.

**The rounds.** Rounds −1..R; in each, the four passes W1 warm, W4 warm, W1 cold and W4
cold, each under both arms in the round's order (P first in even rounds).
- **W1:** worker 0 on the first CPU, blocks in order.
- **W4:** four pinned workers, the traversal protocol's snake. Each step runs the
  preparation, then the parent's `SpinBarrier`, the timed build, `std::sync::Barrier`, and
  then, untimed, the check and the record into storage reserved before the pass.
- **Warm** arenas belong to the worker for the whole run (worker 0's serve W1 and its W4
  lists).
- **Cold:** the preparation drops the worker's cold arenas (`Arenas::default()`) before
  the start barrier; they are dropped again after the pass.
- The records are appended between passes, when no build runs. After round −1 its `check0`
  records follow. The `after` calibration and the `postflight` record close the file.

**The checks, after the completion barrier, outside every window:**
- **G6, every build:**
  - C must have returned true, and `rec` and `raw` must equal the block's oracle image in
    length and every byte;
  - P's table must equal the block's reference in length and every entry.
- **G7, round −1's W1 warm pass, every block:**
  - **C:** before its build, the warm record arena's spare capacity is filled with the
    0xA5A5 poison. After the G6 comparison, the arrays are moved into a `C30`, walked by the
    unchanged `walk::<false>` from the block's primary index and compared with production's
    output, moved back, and compared with the image again.
  - **P:** the table is checked against the counting-scatter definition, computed
    independently of production's code (`tt[s] == ll[s] | i << 8` for each L-position i in
    scatter order). It is then walked by production's `walk_segments_room` against
    production's output and compared with itself afterwards (restoration). Only then is it
    stored as the block's reference.

  One `check0` record per block and arm.
- **Any mismatch** calls the parent's `fail`: a failure record is appended, the line is
  printed, and the process exits with status 3, so no worker is left at a barrier.

**Fault injection** (dry runs only; refused otherwise):

| injection | what it does |
|---|---|
| `output:MODE:ARM:WORKER` | flips one entry of that worker's first round −1 warm table, after the completion barrier |
| `pin:MODE:WORKER` | pins that worker to CPU 1023 |
| `reject:MODE:WORKER` | moves one count between bytes for that worker's first round −1 warm C build |
| `preflight` | fails the preflight |

### `c30walk cbuild-rss`

```
c30walk cbuild-rss OUT.json CORPUS.bz2 --arm P|C --cpus A,B,C,D
```

This is a fresh process holding only the last columns and counts: no oracle, no references,
no production output. It runs:
1. VmRSS after loading (the baseline);
2. `clear_refs` 5, which resets VmHWM;
3. one W4 warm pass of one arm, with the same barriers;
4. VmHWM again.

It records the growth over the baseline and the workers' arena capacities. The recipe runs
three per arm, alternating P, C.

### The analysis, `bench/c30_construction.py`

It follows `bench/c30_traversal.py`'s structure, reading every record kind the collector
writes. Unknown kinds are rejected.

**Stage 1, the schedule:**

| check | what it requires |
|---|---|
| S0 | no failure record and no unknown kind |
| S1 | exactly one meta, preflight and postflight record. The meta must be consistent: rows, full blocks, the lists equal to the re-derived snake, the per-worker rows, four distinct CPUs, both regimes, the RAW counts, the memory estimate |
| S2 | 20 measured rounds and no injection, or a dry run of 2 |
| S3 | exactly the expected (round, regime, mode, arm, worker, step) set, with registered blocks and rows (so no other denominator can enter) and integer readings |
| S4 | placement |
| S5 | window order, the CPU window within the monotonic window plus 2 µs, faults non-negative |
| S6 | W1's sequence |
| S7 | both barriers and the cross-step order: each check ends before the next step's builds start |
| S8 | the registered pass order |
| S9 | the calibration's completeness and form, as the traversal analyser checks it |
| S10 | one `onepass` and one `counters` record per block, and one `check0` per block and arm |

**Stage 2, the gates:**

| gate | condition |
|---|---|
| G1 | P's W1 cost, W4 cost and W4 completion, max ÷ min ≤ 1.05, per regime |
| G2 | start skew ≤ 0.02, per arm, regime and round, in all-full steps |
| G3 | `--identity-ok`, from the recipe |
| G4 | allocation |
| G5 | the calibration-based clock share ≤ 0.5% for every arm, regime, mode and round |
| G6 | every build exact |
| G7 | every `check0` field true |
| G8 | every block's one-pass condition |
| G9 | recomputed from the recorded numbers, not the collector's verdict alone. Before: the requirement ≥ the estimate, the margin exactly max(2 GiB, 25%), `MemAvailable` ≥ requirement + margin, no swap in use, zero pressure, not injected. After: no swap in use, zero pressure, no `pswpin`/`pswpout` increase |
| G10 | no major fault in any measured build (rounds ≥ 0). Round −1's are reported |

**Stage 3, the figures:** per round, regime and arm. The six ratios (b1, b4, e4, warm and
cold) each get a 20-round t interval. `interval()` refuses any other sample size.

**The decision:** warm b4 and e4 only:
- **PASS:** b4+ ≤ 1.06714 and e4+ ≤ 1.02;
- **FAIL:** b4− > 1.06714 or e4− > 1.02;
- otherwise **INCONCLUSIVE**;
- **INVALID** on any S check, any gate, or a failure record.

**Reported, deciding nothing:**
- the cold ratios;
- the full- and short-block economics;
- running fractions;
- the clock shares, including the short blocks';
- faults;
- capacities;
- the peak RSS against the estimate, and the `cbuild-rss` processes' median and range;
- the slow-path table: per-block warm W1 ratio medians, by deciles of the one-row share,
  with fragment shares and promotions;
- the implied saving S at b4's and the traversal w4's interval ends.

**A dry run** checks stage 1, G3, G4 and G6–G10, and suppresses every timing figure, G1,
G2, G5 and the decision.

## 3. Proposed amendments to the protocol (for the guardian; none applied silently)

The implementation needed these choices. Each is in the code as described, and each is
open to the guardian's decision.

- **A1. Reading production's table for G6 and G7.** `mibz2::bwt::TVector` is a newtype,
  `TVector(pub(crate) Vec<u32>)`, with no public accessor to its contents, and `src/` may
  not change. The harness reads it through a pointer reinterpretation, `&TVector` as
  `&Vec<u32>` (`tv` in `cbuild.rs`), guarded in two ways:
  - at compile time, that the two types have equal size and alignment;
  - on every read, that the reinterpreted length equals `TVector::len()`.

  Rust does not guarantee a repr(Rust) newtype's layout equals its field's, so this is a
  relied-on implementation property, not a language guarantee. It is used only outside every
  window: P's checks, and P's capacity reading. The alternatives would be a one-line public
  read-only accessor in `src/bwt.rs` (a `src/` change, not authorised) or weakening P's G6
  to an output check (which can't show byte equality). **Proposed:** accept the guarded
  reinterpretation for this research harness, and record it as a limit.
- **A2. The timed function is a thin wrapper.** `p_build` compiles to a single
  `jmp mibz2::bwt::build`. `c_build` is a 21-byte call of `Ctor::build::<false>` that passes
  its six arguments, with a push and pop. The windows therefore hold a few more instructions
  on C's side than on P's, the price of making each arm one checkable symbol. **Proposed:**
  register the wrappers as the timed functions, with their listings in G3 (§5).
- **A3. G3's references.**
  - The constructor's four timed-path functions (`Ctor::build::<false>`,
    `W<false>::promote`, `W<false>::complete_checked`, `canonicalise`) and
    `mibz2::bwt::build` are compared with the listings of **the reviewed pin's own binary**
    (`e003a32`, rebuilt here with the same toolchain).
  - The wrappers, which only the harness has, are compared with this harness's first build,
    **recorded here for the guardian's review**.
- **A4. The dry run also runs G9 and G10.** The protocol's dry run lists the preflight, S,
  G3, G4, G6, G7 and G8. G9 and G10 read no timing, so the analysis reports them in a dry
  run too.
- **A5. The P reference's acceptance is stronger than registered.** Besides the walk (G7b),
  the reference is checked against the counting-scatter definition before it is stored.
- **A6. The capture recipe needs the dry run's peak.** The protocol has the preflight use
  "whichever is larger" of the estimate and the dry run's peak RSS. The collector takes that
  peak as `--peak-rss`, and the recipe refuses a capture without it.
- **A7. Mixed steps in the block economics.** The full- and short-block e4 sums leave out any
  step mixing the two (step 57 on the primary corpus), rather than reporting it apart.
- **A8. Wording only: two failure-path details.**
  - The parent's shared `fail` prints its line with a `grid:` prefix.
  - A preflight failure stops the run before the meta record exists, so its analysis says
    "INVALID: no meta record" (with the failure shown).

  Both are INVALID as registered.

## 4. The regression tests

`python3 -I bench/test_c30_construction.py`: **61 tests passed.** Each defect fixture
starts from records that would otherwise decide PASS, and most also from records that would
decide FAIL. Each must come out INVALID; most are rejected before any figure is computed.

| class | fixtures (each must be INVALID) |
|---|---|
| malformed schedules | a missing build; a duplicate; an extra round; lists not the snake; workers shifted; a wrong CPU; the arm order swapped; a start before a peer's preparation ends; a check inside a peer's build; a W1 overlap; a CPU window longer than its build; negative faults; an unknown record kind; no meta record |
| wrong denominators | the candidate's rows doubled; the meta's rows off by one; the block rows rewritten consistently in the meta alone; the full-block count wrong |
| missing calibration | none; one batch; a CPU missing; 19,999 windows; decreasing, NaN, short or boolean quantiles; the clock calls' share too large (G5) |
| correctness failures | a failure record; a failed run's partial records; an inexact build (G6); a round −1 check false (G7); a round −1 check missing (S10); a one-pass condition false (G8); one-pass records missing; counter records missing; identity false or unknown (G3) |
| memory-gate violations | the margin one byte short; the margin misstated; the requirement below the estimate; swap in use before or after; pressure before or after; swap activity (`pswpin`, `pswpout`); the preflight not passed or injected; the preflight or postflight record missing; a dry run with swap activity |
| major faults | one major fault in a measured cold W4 build (INVALID); one in round −1 (reported, not gated) |
| insufficient rounds | 1, 2 and 19 rounds at both bases; a dry-run label on 20 rounds; a capture carrying an injection; `interval()` refusing 1, 2, 19 and 21 |
| other gates | an unstable control (G1); a staggered start (G2) |

**The positive fixtures:**
- PASS at b4 ≈ 1.00, and at b4 ≈ 1.06 with its upper bound under 1.06714;
- FAIL on cost (b4 ≈ 1.20);
- FAIL on completion (b4 ≈ 1.00, e4 ≈ 1.10);
- INCONCLUSIVE with b4's interval straddling 1.06714 (e4 under 1.02);
- INCONCLUSIVE with e4's interval straddling 1.02;
- a cold regime twice as costly still giving PASS;
- the one-core ratio reported but not deciding;
- a dry run suppressing every figure while its other gates pass.

`python3 -I bench/test_c30_build_listing.py`: **6 tests passed.**
- A relocated listing (other addresses, other callee addresses) compares identical.
- Each of these is detected: a changed operand, a changed size, a changed internal branch
  target, another callee, a missing instruction.

The Python files pass `ruff check` and are `ruff format`ted. `cbuild.rs` is
rustfmt-clean. Clippy wasn't run (not installed for 1.97.0 here).

## 5. Identity after the harness is added (G3)

The harness changes the binary, so the reviewed addresses are no identity. Each function
of the timed path was compared after normalisation:
- addresses dropped;
- internal branches as offsets from the function's start;
- external targets by symbol;
- RIP displacements masked.

Each was compared over its own `nm -S` extent, in the harness binary built from `e264eed`
in the recipe's own worktree:

| function | size | instructions | reference | result |
|---|--:|--:|---|---|
| `<c30walk::ctor::Ctor>::build::<false>` | 0x1b5e | 1,538 | `e003a32`'s binary | identical |
| `<c30walk::ctor::W<false>>::promote` | 0x21e | 135 | `e003a32`'s binary | identical |
| `<c30walk::ctor::W<false>>::complete_checked` | 0x3be | 241 | `e003a32`'s binary | identical |
| `c30walk::ctor::canonicalise` | 0x305 | 173 | `e003a32`'s binary | identical |
| `mibz2::bwt::build` | 0xe97 | 814 | `e003a32`'s binary | identical |
| `c30walk::cbuild::c_build` | 0x15 | 8 | this harness (recorded for review) | identical |
| `c30walk::cbuild::p_build` | 0x5 | 1 | this harness (recorded for review) | identical |

**The constructor's machine code is unchanged in substance** from the reviewed listing.
`e003a32`'s `Ctor::build::<false>` also compares identical, by the same normalisation, to the
prototype journal's committed listing
(`2026-11-03-compact30-constructor-prototype/build-false.asm`).

**`mibz2::bwt::build`'s compilation:**
- It is one 0xe97-byte function. Its only out-of-line callees are
  `RawVecInner::reserve::do_reserve_and_handle` (production's `reserve(n)`),
  `panic_bounds_check` and `slice_index_fail`.
- No `build_one_pass` or `build_two_pass` symbol exists in the binary, so both are inlined
  into it.
- `p_build` is `jmp <mibz2::bwt::build>`.
- `c_build` is:

  ```
  push %rax; mov %r8,%rax; mov %r9,(%rsp); xor %r8d,%r8d; mov %rax,%r9
  call <<c30walk::ctor::Ctor>::build::<false>>; pop %rcx; ret
  ```

  The `xor %r8d,%r8d` is the `permille = 0` argument.

**The listings for review** are in `2026-11-03-compact30-construction-harness/listings/`:
the references, and `vm-dry-run/listing/` (each function with source lines,
`*-lines.asm`, and the empty diffs, from the recipe's build). The recipe ran G3 in each of
its six runs; each printed `yes`.

## 6. The untimed VM dry runs and the injected failures

**The VM:** 4 cores (one thread each, so the CPUs used were 0–3, the recipe's "first four"
case), 15 GB RAM, no swap, and `/proc/pressure/memory` present. It is a shared machine, and
no other heavy job ran during these runs.

**Memory allowed the registered dry-run scope:** the full primary corpus, rounds −1, 0 and
1, through the recipe exactly as a capture would run it, apart from the dry-run flag. Nothing
was shrunk.

**`just bench-c30-construction <corpora> 20 dry-run` (recipe exit 0):**

| what | behaviour |
|---|---|
| corpus and pin | the corpus hash OK. The pin worktree was created at `e264eed` with no tracked changes. The toolchain was verified (commit, LLVM) |
| tests | 61 and 6 passed inside the recipe |
| G3 | `yes` for all seven functions and the three structural checks |
| CPU placement | workers on CPUs 0,1,2,3 (W1 on 0). S4 passed for all 10,416 build records: each build's CPU before and after equals its worker's |
| barriers | `barrier-test` passed (12,000 steps, 3 rounds × 2 arms, uneven arrivals and lists). S7 passed: start barrier, completion barrier, cross-step order |
| both regimes | 24 passes (3 rounds × 4 passes × 2 arms), 434 records each, warm and cold. In the measured rounds, cold windows recorded minor faults and warm windows none: the cold builds did allocate, and the fault counters are live |
| correctness | G6 on every one of the 10,416 builds. G7 on all 434 blocks for both arms (868 `check0` records: C poisoned, exact, walked, restored; P definition, walked, restored). G8 on all 434 blocks |
| memory | the preflight passed (G9). The postflight showed no swap, no pressure and no swap activity. No major fault in any build (G10). The capture process's recorded peak RSS (VmHWM) was below the computed estimate, so the estimate held here |
| peak-RSS processes | six ran (P, C × 3), each with the HWM reset accepted, and their records were read by the analysis |
| analysis | schedule S0–S10 all true. G3, G4 and G6–G10 all true. Every timing figure, G1, G2, G5 and the decision suppressed, as registered |
| archive | `target/c30-construction-20261103T153959Z-dry-run.tar.gz` (sha256 `4afefc95…`), 33 files plus `MANIFEST.json`, naming commit `e264eed`, no tracked changes, the corpus and binary hashes, the command and exit status 0 |

**The injected failures,** each a recipe run on the full primary corpus. In each, G3 still
printed `yes`, the collector exited with status 3, the failure record was appended and
printed, the analysis ran and declared the run INVALID, and the recipe archived it and
exited 3.

| mode | where it stopped | the failure record | analysis |
|---|---|---|---|
| `fail-preflight` | after the sizing pass, before the decode or any large allocation | `preflight`: MemAvailable … against required … + margin …; (injected) | INVALID (no meta record yet; the failure shown) |
| `fail-output` | round −1, W4 warm, C, worker 2, step 0 | `output`: compact30's arrays differ from the oracle image | INVALID: S0, S1 (no postflight), S3 |
| `fail-output-p` | round −1, W1 warm, P, block 0 | `output`: P's table: definition false, walk false, restored true | INVALID: S0, S1, S3 |
| `fail-pin` | round −1, W4 warm, worker 1 | `pin`: could not pin to CPU 1023 | INVALID: S0, S1, S3 |
| `fail-reject` | round −1, W4 warm, C, worker 3, step 0 | `rejection`: the constructor rejected the block | INVALID: S0, S1, S3 |

The archives' sha256s:
- fail-preflight `6a9e7ec0…`;
- fail-output `4b207da0…`;
- fail-output-p `49791825…`;
- fail-pin `9aa66569…`;
- fail-reject `c7699021…`.

**Two refusals,** checked directly:
- `bench-c30-construction … capture` without a peak argument exits 2 before doing anything;
- `cbuild … --inject preflight` without `--dry-run` panics on its assertion before reading
  the corpus.

**Not exercised here:**
- a 20-round capture, deliberately (not authorised);
- the five-or-more-core CPU choice (cores 2–5), because this VM has four;
- the analysis's figure stage on real records (it ran only on synthetic fixtures, since a
  dry run suppresses it);
- the `--peak-rss` path with a nonzero value in the collector (only the recipe's refusal of
  a zero value);
- a preflight failure from real memory shortage (only the injected one).

**What was kept** (`2026-11-03-compact30-construction-harness/vm-*/`): each run's console,
analysis JSON, manifest, barrier test, toolchain identity and listing results, and the
dry run's six peak-RSS records.

**Not kept:** the record files (`c30-construction.jsonl`) and the tarballs. As in the
traversal protocol, they hold the VM's timestamps and calibration, which are not
measurements. Their checksums are in each manifest.

## 7. The proposed i9 command (NOT AUTHORISED)

From a checkout of `research/compact30-constructor` at the commit holding the recipe, with
the primary corpus in `target/walk-decoder/work/corpora`:

```
# 1. the registered dry run on the i9 (untimed), for its preflight evidence and peak RSS
just bench-c30-construction target/walk-decoder/work/corpora 20 dry-run
# 2. the capture, given that dry run's peak RSS in bytes
#    (memory.peak_rss_vm_hwm in its c30-analysis.json)
just bench-c30-construction target/walk-decoder/work/corpora 20 capture <PEAK_BYTES>
```

Each step uploads one file: `target/c30-construction-<UTC time>[-dry-run].tar.gz`.

**Neither step is authorised.** This draft asks the guardian to review the harness, the
listings and the amendments in §3 before any i9 run.

## 8. Negative and uncertain findings

- **A1's layout reliance** (P's table read through a pointer reinterpretation) is the main
  open point. It is guarded but not guaranteed by the language.
- **Cold doesn't always mean first-touch.** Cold windows recorded minor faults, but glibc
  may serve a freed arena again without new page faults. The cold regime measures the
  allocator as configured, as registered; it is not a page-fault-cold measurement.
- **G7a's poison has no effect on a block whose arena must grow** (the first block, and any
  block larger than every earlier one): growth allocates fresh, unpoisoned memory. That is
  the audit's harness finding 3, unchanged here.
- **The wrappers add a small asymmetry** of a few instructions to C's window (A2).
- **The analysis's figure stage** (ratios, intervals, gates G1, G2, G5) is tested only on
  synthetic records.
- **The VM has four cores,** so the recipe's choice of cores 2–5 on a larger machine is
  untested here.
- **The peak RSS was below the estimate on the VM.** That says the estimate isn't too small
  here. The i9's own dry run must repeat it, and the capture uses the larger of the two.
- **Clippy wasn't run.**

## 9. Files

- `listings/`: the G3 references. They are `ctor-build-false`, `ctor-promote`,
  `ctor-complete-checked`, `ctor-canonicalise` and `bwt-build` from `e003a32`'s binary, and
  `c-build` and `p-build` from this harness's build.
- `vm-dry-run/`, `vm-fail-preflight/`, `vm-fail-output/`, `vm-fail-output-p/`, `vm-fail-pin/`,
  `vm-fail-reject/`: each run's kept records (§6).
