# The build's windows cut into runs, v2: the classifier's cost removed (registration)

Registered before any timing of this branch, under the Guardian's D075 (`state/decisions.md`
on `team-state/guardian` at fbbf539a). **This rule is frozen:** it is not amended after the
pull request is opened. A result that fails it is recorded here, never re-read.

## Where this comes from: #310 failed its no-harm rule

#310 (`perf/build-window-runs`, head **eacd58826cdef3c040fb268356d787ce442b0d53**, base
b681712235a3576306cb46d4818b9608a0e0b13d; registration `2026-11-09-build-window-runs.md`) cut
each 16-row window of the build into its runs. Its two CI runs (run 42704879531, attempts 1
and 2; attempt 1's table preserved in comment 6752574723, attempt 2's in 6752208345):

| figure | attempt 1 | attempt 2 |
|---|--:|--:|
| `wikidata` wall, 1 thread | **−1.7% (−2.7, −0.7)** | **−4.0% (−4.8, −3.2)** |
| `wikidata` wall, 2 threads | **−3.5% (−6.5, −0.5)** | **−4.7% (−8.2, −1.0)** |
| `wikidata` wall, 4 threads | −3.3% (−7.5, +1.0) | −0.5% (−2.7, +1.7) |
| `records` wall, 1 thread | **+1.9% (+1.3, +2.5) slower** | **+1.9% (+0.1, +3.7) slower** |
| `records` CPU, 4 threads | +2.2% (−0.8, +5.2) | **+3.8% (+0.8, +6.8) slower** |
| `random` wall, 1 thread | not flagged | **+2.6% (+1.9, +3.2) slower** |
| T-vector build stage (1 thread) | −11.7% | −22.5% |

Its benefit rule was met; its no-harm rule failed (`records` wall at 1 thread flagged slower in
both runs). D075: that head is held, recorded as failed, and never merged; **there is no
reverting to it or re-benching it.** **#310's runs do not count for this head**, neither as
its first run nor as its "other run".

## What cost #310 on high-entropy blocks: two things, not one

`records` and `random` are nearly incompressible. Their windows almost all have 16 runs
(`run_census.rs`, every block of the CI corpora, which are byte-identical to the files counted
here: `records.bz2` SHA-256 `ff8b5f6ed3dd4445…`, `random.bz2` `10d444e6204289af…`, made by
`corpora_gen.py` as `ab gen` makes them, and `bzip2 -9`):

| corpus | windows | 1 run | 2–6 runs | 7–15 runs | 16 runs |
|---|--:|--:|--:|--:|--:|
| `wikidata-independent.bz2` (473 blocks) | 14,735,731 | 83.38% | 10.06% | 5.60% | 0.96% |
| `records` | 599,992 | 0 | 0 | 4.64% | 95.36% |
| `random` | 499,993 | 0 | 0 | 11.03% | 88.97% |

So every window of those blocks takes #310's classifier and then master's one-row steps.
#310's emitted code (`asm/build-old.asm`, rustc 1.99.1, fat LTO, the bench's baseline
x86-64) charges each such window two costs over master's:
1. **The software popcount** (no POPCNT in baseline x86-64): `not`, `or`, `movzwl`, then the
   SWAR count (`shr`, `and 0x5555`, `sub`, two `and 0x33333333`, `add`, `shr 4`, `add`,
   `and 0x0f0f0f0f`, `imul 0x01010101`, `shr 24`), `cmp`, `jbe`: about 19 instructions.
2. **A stack copy of the window's bytes,** which #310's notes did not name. The window's
   16 bytes are in a vector register (`run_heads`), and the one-row steps read `ll[i]` for
   those same bytes, so the compiler took them from the vector: it stored the vector to the
   stack (`movdqa %xmm5,0x10(%rsp)`) and copied each byte to another stack slot (14 `movzbl`
   and 13 `mov`), then each step re-read its byte from there. About 28 instructions a window.
   Master reads the window as two u64s and takes each byte by a shift.

Counted (callgrind, below): #310's build is **+10.0%** instructions on every `records` and
`random` block (+1,406,205 on a 900 kB block: 25 a window), +1.47% to +1.66% of the decode.
That is the size of the 1-thread wall flags (+1.9%, +2.6%).

## The change (code commit b526fdda59a21212dd5c57fd92c0ab279d5c2c37)

Only the classification and the source of the one-row steps' bytes change; the run-by-run
path, its writes, the one-row step and the safety argument are #310's.
- **`run_joins`** returns the window's joins, bit j set when row j equals row j − 1 (j ≥ 1):
  `pmovmskb` of the window against itself shifted by a byte, `and 0xfffe`. A window of one run
  is `joins == 0xfffe`, one of 16 runs is `joins == 0`: **each one compare, with no count.**
  A 16-run window (89–95% of a high-entropy block's) goes straight to the one-row steps.
- **`few_runs`** decides the windows between (2 to 15 runs) as #310 did, `heads.count_ones()
  <= 6` with heads `joins ^ 0xffff`, i.e. at least 10 joins: `POP8[joins & 0xff] + POP8[joins
  >> 8] >= 10`, two loads from a 256-byte static table.
- **`row_byte`** reads row i's byte from `ll` with a volatile read (bounds checked as `ll[i]`),
  in the one-row step, for a run's byte and for a one-run window's byte. The compiler can no
  longer take the byte from the vector, so the stack copy is gone; each byte is one load.

**The emitted classifier** (`asm/build-head.asm`, same toolchain and flags as master's
build in the bench):

```
movdqu (%r14,%rsi,1),%xmm5      ; the window
movdqa %xmm5,%xmm6
pslldq $0x1,%xmm6
pcmpeqb %xmm5,%xmm6
pmovmskb %xmm6,%ebp
and    $0xfffe,%ebp             ; joins (sets ZF)
mov    %r9,0x10(%rsp)           ; a spill of the next window's end
je     <one-row steps>          ; 16 runs
cmp    $0xfffe,%ebp
jne    <few_runs>               ; else: one run, the 16-wide write
...
few_runs:                       ; windows of 2 to 15 runs only
mov %ebp,%eax ; shr $8,%eax ; mov %ebp,%r8d ; and $0xfe,%r8d
lea POP8(%rip),%r10 ; movzbl (%rax,%r10),%eax ; add (%r8,%r10),%al
cmp $0xa,%al ; jb <one-row steps> ; xor $0xffff,%ebp   ; else run by run
```

A 16-run window costs 8 instructions to its branch, against master's 9 (two u64 loads, `mov`,
`xor`, `mov`, `rol`, `xor`, `or`, `jne`). Its one-row step is 14 instructions against master's 15
(the byte is one `movzbl` from `ll`, where master's is `mov` + `shr` from a register): one
more load a row (the row's byte, from the line the window's load just brought in, off the
cursor chain) and one fewer ALU instruction.

**The table's cost:** `POP8` is 256 bytes, four cache lines, read only by windows of 2 to 15
runs (15.7% of Wikidata's, 4.6% and 11.0% of `records`' and `random`'s), two loads each, both
independent of the cursor chain. Four lines of L1D (32 KiB on the runners' 7763 and 9V74) are
negligible against the build's streams; a miss on them costs at most four misses a block.

## Equivalence (required before timing): met

`bwt::tests::few_runs_counts_the_runs` enumerates **all 32,768 masks** `run_joins` can return
(bit 0 clear, bits 1–15 free, none above), and asserts for each (a) `few_runs(joins) ==
(joins ^ 0xffff).count_ones() <= RUNS` and (b) the whole classifier (`== ALL_JOINED`, then
`!= 0 && few_runs`) sends the window where #310's (`heads == 1`, then `count_ones() <=
RUNS`) did: one run, run by run, or one row at a time. It passes (native, release).
`run_joins` itself is checked against a scalar count of runs on every window of Miri's columns
(`miri_inputs_reach_every_window_path`).

## Safety (B5)

- **Miri, native x86-64** (`cargo +nightly miri test --lib -- miri_build miri_inputs`, nightly
  miri 2026-10-26): `miri_build_one_pass` and the new `miri_inputs_reach_every_window_path`
  pass. The second asserts that Miri's columns (`build_inputs(3, 80)`) contain windows of every
  class: one run (the 16-wide write), 2–6 runs (run by run; the near-end fallback is reached
  by the runs that end near their bucket's end, as on #310), 7–15 runs (`few_runs`'s table, then
  one row at a time) and 16 runs (the first compare, then one row at a time). The CI Miri
  job runs both (`--lib -- miri_`).
- **The scalar fallbacks under Miri,** on a target without x86-64's paths
  (`--target aarch64-unknown-linux-gnu`): `miri_build_one_pass`,
  `miri_inputs_reach_every_window_path`, `one_pass_build_is_the_two_pass_build` (43 columns
  of up to 3,000 rows), `wrong_counts_take_the_two_pass_build` and `run_stores_equal_one_pass`
  all pass: the scalar `run_joins` and `run16` under the interpreter.
- **The test suite** (`cargo test --release`, every target): passes; clippy (all targets) and
  rustfmt clean.
- **The unsafe code:** `row_byte`'s read is of a reference (`&ll[i]`), bounds checked; the
  writes and `set_len` are #310's, unchanged, under the same argument (every slot below n last
  written with its entry; the end check `ftab == lim`).

## Whole decoder: instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`; driver
`driver_main.rs`, the method of `2026-11-05-deferred-flag-walk/`: rustc 1.99.1, fat LTO,
`codegen-units = 1`). Master is b68171223 (`base-*`), #310 eacd58826 (`old-*`), this
branch b526fdda5 (`head-*`). Profiles in `cg/`. Every output is identical across the three
(`cg/*.stdout`). The walk (`room_segments`) is identical to the instruction in every block.

| block | master | #310 | this branch | build: master → #310 → this (#310 / this vs master) |
|---|--:|--:|--:|---|
| wikidata 1 | 19,258,186 | 18,652,955 (−3.14%) | 18,471,069 (**−4.09%**) | 5,250,956 → 4,645,725 → 4,463,839 (−11.5% / −15.0%) |
| wikidata 41 | 18,608,438 | 17,945,838 (−3.56%) | 17,782,493 (**−4.44%**) | 5,103,413 → 4,440,813 → 4,277,468 (−13.0% / −16.2%) |
| wikidata 233 | 19,053,852 | 18,550,694 (−2.64%) | 18,381,004 (**−3.53%**) | 5,038,522 → 4,535,364 → 4,365,674 (−10.0% / −13.4%) |
| wikidata 400 | 18,620,577 | 17,988,122 (−3.40%) | 17,828,095 (**−4.26%**) | 5,026,554 → 4,394,099 → 4,234,072 (−12.6% / −15.8%) |
| wikidata 0 (54 B) | 7,037 | 7,117 | 7,046 (+9) | |
| wikidata 472 (70 B) | 8,252 | 8,357 | 8,262 (+10) | 3,585 → 3,690 → 3,595 |
| records 0 | 95,933,496 | 97,339,701 (+1.47%) | 95,399,732 (**−0.56%**) | 14,008,440 → 15,414,645 → 13,474,676 (+10.0% / −3.8%) |
| records 5 | 95,975,873 | 97,382,078 (+1.47%) | 95,442,373 (**−0.56%**) | 14,008,440 → 15,414,645 → 13,474,940 (+10.0% / −3.8%) |
| records 10 | 63,550,877 | 64,488,682 (+1.48%) | 63,195,465 (**−0.56%**) | 9,342,965 → 10,280,770 → 8,987,553 (+10.0% / −3.8%) |
| random 0 | 84,956,287 | 86,362,492 (+1.66%) | 84,462,266 (**−0.58%**) | 14,008,440 → 15,414,645 → 13,514,419 (+10.0% / −3.5%) |
| random 4 | 85,282,030 | 86,688,235 (+1.65%) | 84,787,030 (**−0.58%**) | 14,008,440 → 15,414,645 → 13,513,440 (+10.0% / −3.5%) |
| random 8 | 75,769,809 | 77,020,039 (+1.65%) | 75,330,225 (**−0.58%**) | 12,454,897 → 13,705,127 → 12,015,313 (+10.0% / −3.5%) |

- **High-entropy blocks: no longer worse.** `records` and `random` go from #310's +10.0% build
  instructions to −3.5% to −3.8% below master (−9.5 a window: the byte loads replacing
  master's shifts).
- **Wikidata gains more than #310:** −3.5% to −4.4% of the decode, against #310's −2.6% to
  −3.6%. The classifier's and the stack copy's instructions are gone from its mixed windows
  too.
- **Short blocks:** +9 and +10 instructions, the window loop's set-up.
- **Build, walk, rest:** the whole difference is the build; the walk and MTF are unchanged.

**Branches** (callgrind's simulated predictor, `cg/br-*`), the build:

| block | master: branches, mispredicted | #310 | this branch |
|---|--:|--:|--:|
| wikidata 1 | 564,032, 10,485 | 375,929, 15,500 | 431,439, 18,667 |
| records 0 | 1,968,991, 59 | 1,968,991, 48 | 1,917,963, **59,036** |

On `records` 56,247 of this branch's 59,036 simulated mispredictions are one instruction, the
window loop's back edge (`jbe` at the loop's end, `cg/br-head-rec-b0.out`), which is taken on
every window but the last, and the simulator scores it mispredicted on all but one of them.
A branch taken on every iteration is what hardware predictors predict best, so I read this
as an artefact of the simulator (its table is indexed by address and recent outcomes) on this
loop's layout; **the cause is not established,** and I can't check it on hardware here (no PMU
access on this machine). If it were real it would be a harm of about 15 cycles a window on a
high-entropy block (roughly +4% of its build, +0.5% of its decode), and the CI bench below
would show it on `records` and `random`. Wikidata block 1's +3.2k mispredictions over #310
are not attributed.

## Predictions, and what falsifies them

- **The T-vector build stage** (`phases`, Wikidata, 1 thread): **−12% to −28%** (#310: −11.7%,
  −22.5%; this branch has 3.4–4% fewer build instructions than #310 on Wikidata's blocks).
  Falsifier (#310's): a point estimate at or above −4%, or a slower stage.
- **`wikidata` wall:** 1 thread −1.5% to −5%; 2 threads −2% to −5%; 4 threads 0% to −4%.
- **`records` and `random`, every thread count: no wall row flagged slower.** Point estimates
  at 1 thread between −1.0% and +0.5% (their decode instructions fall 0.56–0.58%).
  **Falsifier of this correction's mechanism:** a `records` or `random` wall row flagged slower
  at 1 thread in either planned run: then the classifier was not the cost, or not all of it.
- **`text`, `pattern`, `runs`, `onebyte`:** no flag slower; `text` and `pattern` at 1 thread
  lean faster as on #310.

## The decision rule (frozen)

The rule is #310's, with its inherited no-harm policy written out in full.

- **Correctness (required):** CI green, Miri included (`miri_build_one_pass`,
  `miri_inputs_reach_every_window_path`); the same bytes on every corpus in every bench run;
  the build's tests pass, `few_runs_counts_the_runs` (all 32,768 masks) included.
- **Benefit (required):** on `wikidata` at 1 or 4 threads, the CPU change or the wall change
  excludes zero on the faster side in at least one run, and the same figure's point estimate is
  not slower in the other run (in every other run, if a third is made).
- **No harm (required), as in `2026-11-05-deferred-flag-walk.md`, each run read on its own:**
  - a wall row (any corpus × thread count, all seven corpora) flagged **slower** is a harm
    signal;
  - one more run on the same head settles it: flagged again, it is a harm, and the rule fails;
  - not flagged again, the row is reported as unresolved, with all its runs;
  - at most three runs in all.
  "Not flagged" means no slowdown detected, not equivalence: a harm smaller than a row's
  half-width (±1–5% in recent runs) can't be excluded by these runs, and this PR doesn't claim
  to exclude it.
- **The repeat policy:**
  - **Run 1** is the bench run the pull request's opening starts, on this head against its
    merge base with master (b681712235a3576306cb46d4818b9608a0e0b13d).
  - **Run 2** is one re-run of that same workflow run (the same head and base), started once
    run 1 has posted its table. Both runs are planned: run 2 is the benefit rule's "other run",
    and it settles any wall row flagged slower in run 1.
  - **Run 3** is made only to settle a wall row flagged slower in run 2 and not in run 1. No
    run is made for any other reason, and none after three.
  - A run whose job fails before the bench produces its table (a setup failure,
    `docs/team/CI-INCIDENTS.md` class B) is re-run as the same run; it is not a run.
  - Each run's whole table is preserved (the bench edits one comment in place) before the next
    run starts.
- **The whole grid is reported:** every corpus × thread count, wall, CPU and peak RSS, the
  resources table and the stage table, in every run, whatever it shows.
- **No retries to pass, no fallback:** if the rule fails, the PR is closed and this entry
  records the negative result. #310's head is not a fallback (D075).
- **Not changed after the data:** the corpora, thread counts, rounds, `RUNS` = 6, this rule.

## Result: the registered rule passes, with three wall rows unresolved

Recorded after the runs. The rule above is unchanged. Three runs on the benched head
**c40a1dc272c4fe83ae038ad165eb787543a0bd41** against b681712: run 42714182022, attempts 1, 2
and 3, bench 3.4.0, 10 paired rounds each. The bench jobs were 129491282563, 129497083192 and
129502398144. Run 1 was on an AMD EPYC 7763; runs 2 and 3 were on an Intel Xeon Platinum
8573C. The tables are preserved on #312: run 1 in comment 6753648090, run 2 in 6753848898,
and run 3 in the bench comment 6753433826 as it stands after attempt 3.

Run 3 was made under the repeat policy, only to settle the two rows that run 2 flagged slower
and run 1 had not. Every run gave the same bytes on every corpus. CI was green on the head:
24 checks passed, 3 were skipped, and Miri passed.

**The whole grid, wall change (CPU change), head over base, 95% intervals:**

| corpus | threads | run 1 | run 2 | run 3 |
|---|--:|--:|--:|--:|
| `wikidata` | 1 | **−8.1% (−9.1, −7.1)** (**−8.1%**) | **−1.4% (−2.1, −0.7)** (**−1.4%**) | +0.3% (−2.3, +3.0) (+0.3%) |
| `wikidata` | 2 | **−5.7% (−7.3, −4.0)** (**−5.7%**) | **−1.7% (−2.5, −0.8)** (**−1.4%**) | **−1.3% (−2.4, −0.3)** (**−1.4%**) |
| `wikidata` | 4 | −2.2% (−4.5, +0.2) (**−3.0% (−5.0, −1.1)**) | −0.3% (−1.5, +0.9) (**−1.6% (−3.0, −0.0)**) | −2.6% (−5.7, +0.7) (**−3.1% (−5.5, −0.7)**) |
| `onebyte` | 1 | −1.6% (−3.7, +0.5) | −3.7% (−10.6, +3.8) | +0.6% (−1.2, +2.5) |
| `onebyte` | 2 | −1.1% (−4.6, +2.6) | −0.1% (−1.8, +1.6) | −1.3% (−3.0, +0.5) |
| `onebyte` | 4 | −0.6% (−3.5, +2.4) | −0.0% (−1.5, +1.5) | +1.2% (−0.8, +3.2) |
| `pattern` | 1 | **−2.3% (−4.4, −0.1) faster** | **+0.9% (+0.1, +1.7) slower** | +0.4% (−1.4, +2.2) |
| `pattern` | 2 | **+7.7% (+0.1, +15.9) slower** | −4.3% (−12.4, +4.4) | −0.9% (−11.3, +10.7) |
| `pattern` | 4 | +0.3% (−5.0, +5.8) | +1.1% (−4.0, +6.4) | +3.2% (−2.6, +9.4) |
| `random` | 1 | −0.7% (−2.1, +0.7) | −0.8% (−1.7, +0.1) | −1.1% (−3.2, +1.1) |
| `random` | 2 | +0.6% (−3.3, +4.6) | −1.9% (−4.7, +1.1) | +1.5% (−1.5, +4.7) |
| `random` | 4 | **−3.2% (−5.6, −0.6) faster** | +1.8% (−2.2, +6.1) (CPU **+1.7% (+0.0, +3.5) slower**) | +1.2% (−2.4, +4.9) |
| `records` | 1 | +0.4% (−1.1, +1.9) | −0.0% (−6.0, +6.3) | −0.1% (−5.7, +5.8) |
| `records` | 2 | +1.4% (−2.1, +5.1) | **+2.3% (+0.8, +3.9) slower** | −1.3% (−5.8, +3.3) |
| `records` | 4 | +0.1% (−3.5, +3.8) (CPU +1.3%) | −0.8% (−5.8, +4.6) (CPU +1.6%) | +2.3% (−0.7, +5.3) (CPU +3.1% (−0.1, +6.4)) |
| `runs` | 1 | −0.0% (−1.6, +1.6) | −0.1% (−1.5, +1.4) | −1.1% (−4.2, +2.1) |
| `runs` | 2 | −1.8% (−4.8, +1.2) | −0.1% (−1.6, +1.3) | −0.8% (−2.3, +0.7) |
| `runs` | 4 | −1.3% (−3.3, +0.7) | −0.8% (−1.9, +0.4) | +0.3% (−1.3, +1.9) |
| `text` | 1 | **−6.0% (−8.6, −3.3) faster** | **−1.7% (−2.2, −1.1) faster** | **−1.8% (−2.4, −1.2) faster** |
| `text` | 2 | **−4.5% (−8.6, −0.2) faster** | −1.2% (−4.4, +2.2) (CPU **−1.4%**) | −2.7% (−8.5, +3.4) |
| `text` | 4 | **−1.8% (−3.5, −0.2) faster** | −0.5% (−6.0, +5.4) | −0.2% (−2.4, +2.0) |

**Peak RSS** moved by at most ±4% in every row, and none was flagged except `pattern` at 4
threads in run 1 (+3.7%).

**Resources (`wikidata`):**
- run 1: "a trade-off: better CPU cost; worse contention sensitivity", with amplification at
  4 threads +5.5% (+3.4, +7.5);
- run 2: "better (CPU cost), and no measure worse";
- run 3: "better (CPU cost, contention sensitivity), and no measure worse", with amplification
  at 4 threads −3.5% (−6.6, −0.2).

**Stages (`phases`, Wikidata, 1 thread), the T-vector build:**
- run 1: **−28.4%** (−29.1, −27.8);
- run 2: **−8.7%** (−8.9, −8.5);
- run 3: **−9.2%** (−12.0, −6.2).

The MTF + RLE2 stage, whose code is unchanged, read +4.8% and +1.2% slower in runs 1 and 2,
and +1.1% (not flagged) in run 3. The walk didn't move.

### Against the rule
- **Correctness:** met.
- **Benefit:** met. `wikidata` CPU at 4 threads is faster beyond noise in all three runs
  (−3.0%, −1.6%, −3.1%). `wikidata` wall at 1 thread is faster beyond noise in runs 1 and 2,
  but its point estimate in run 3 is +0.3%. So under the "every other run" clause that figure
  alone would not qualify, and the benefit rests on the 4-thread CPU figure.
- **No harm:** no harm confirmed. Each flagged wall row was settled by the next run on the
  same head, and none was flagged again:
  - `pattern` at 2 threads: flagged in run 1, not in run 2. **Unresolved.**
  - `pattern` at 1 thread: flagged in run 2, not in run 3. **Unresolved.**
  - `records` at 2 threads: flagged in run 2, not in run 3. **Unresolved.**

  All three rows are reported with every run above. The rule allowed no fourth run.
- **Verdict:** the registered rule **passes**. The merge decision is the coordinator's.

### Against the predictions
- **The build stage:** predicted −12% to −28%; it measured −28.4%, −8.7% and −9.2%. The
  falsifier (a point estimate at or above −4%) did not fire. Runs 2 and 3, on the Xeon, fell
  short of the predicted range: the build's gain there is a third of run 1's on the EPYC.
- **`wikidata` wall at 1 thread:** predicted −1.5% to −5%; it measured −8.1%, −1.4% and
  +0.3%. The prediction failed in run 3, where the interval was ±2.7%.
- **`records` and `random` at 1 thread:** no wall flag in any run, and the point estimates
  were −1.1% to +0.4%, within the predicted −1.0% to +0.5% except `random` in run 3 (−1.1%).
  **The correction's falsifier did not fire.** The 1-thread slowdowns that failed #310 (+1.9%
  twice, +2.6%) don't recur.
- **The simulated back-edge mispredictions:** no `records` or `random` 1-thread slowdown
  appeared, so I read them as a simulator artefact. That remains unverified on hardware.
- **Uncertainties:**
  - `records` at 2 threads (+2.3% in run 2) and its 4-thread CPU (+1.3%, +1.6%, +3.1%, none
    beyond noise) lean slower. A harm smaller than these rows' half-widths (±3–6%) can't be
    excluded.
  - The MTF stage's flags in unchanged code suggest a layout effect of a size similar to the
    small rows above.
