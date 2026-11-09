# 2026-11-09: C29, the register-list checkpoint kernel in the shipped pipelined front, opt-in

**Status: DRAFT registration, NOT FROZEN, NOT RUN** (RULES C1). Nothing here changes the
default decoder: the switch is off in `PIPELINED_LEVERS`, and the default's entry points can't
set it. No timed run has been made, locally or on CI. Nothing here runs on the workstation, and
no recipe is in the Justfile or the workstation queue.

**The input.** #270's registered run (`docs/journal/2026-11-09-run-fed-leader-result.md`) put
`mtf_checkpoints` on `mtf_avx2`'s register-list kernel (the arm `runfed-reg`, the matched control
C29) against its own control at **reg/ctl 0.9739 [0.9682, 0.9797]** wall at 9 threads (3 teams of
3, CI's 4 vCPUs), and reg/default33 0.9745 [0.9675, 0.9816]. At one team (3 threads, reported
only) reg was 0.9615 against `pipe3-both`, with ctl at 1.0028 there. That result said C29 is a
separate lead with its own registration. The Optimiser's ranking (REVIEW 6732797905) put it first,
"in the shipped pipelined front, no records or copied worker", and noted:
- C29 changes the register list **and** the short-run stores: it is not a list-only attribution;
- bench.yml's default 1, 2 and 4-thread cells **do not execute** the pipelined path, so a separate
  exposed-path diagnostic is needed;
- B5 must cover checkpoint and error semantics, the 64-byte physical stores, and composition with
  the shipped worker.

This is that production prototype: the kernel ported to master's `src/mtf.rs`, selected in the
shipped worker's front stage by a lever, its B5 evidence, and the diagnostic's registration.

## 1. The change

- **The kernel** (`src/mtf.rs`): `mtf_checkpoints_reg` dispatches to
  `mtf_checkpoints_reg_avx2` with AVX2, else to `mtf_checkpoints_scalar`, the scalar twin itself
  (unchanged). The AVX2 kernel is #270's (research/run-fed-leader `03e52206`, `src/mtf.rs`'s
  `mtf_checkpoints_reg_avx2`) unchanged but for comments: `mtf_avx2`'s list held as bytes in two
  registers and seven aligned rows (`STEPS`, `second`, both already on master), and its run store
  (two unaligned 32-byte stores where `len <= 64 && n + 64 <= max`, else `write_bytes`), inside
  `mtf_checkpoints_avx2`'s outer loop (the inner loop's limit the nearer of the block's end and
  the next checkpoint, a run that crosses one split there). It is no longer behind the `run-fed`
  feature: it is production code, reached only through the switch.
- **The shipped kernel stays:** `mtf_checkpoints` and `mtf_checkpoints_avx2` are untouched, and
  the default decoder, the plain teams (`decode_block`) and the workers still use them.
- **The switch** (`src/parallel.rs`): `Levers::register_list`, a fourth lever beside park, small
  and spin2k. `pipelined_levers` passes it to `pipelined_worker` as a runtime `bool`, which passes
  it to `stage` and `front_stage`, which call `mtf_checkpoints_reg` when it is set and
  `mtf_checkpoints` when it isn't. One branch a block; no new monomorphisation (the other levers'
  eight const-generic workers are unchanged). `PIPELINED_LEVERS` sets it false, so
  `StreamDecoder::new`, `BzReader::parallel` and `decode_parallel` are unchanged at every plan.
- **A reach counter:** `StreamDecoder::c29_stages` (doc-hidden, as `pipeline_counts` is) counts
  the blocks a leader staged with the switch set, so the tests and the fuzz loop can show the
  switch reached the stage. One relaxed atomic add a block, on the switched path only.
- **The record** (`examples/ab.rs`): each run's line now carries its child's `exit` (0, its code,
  or -1 for a signal), as #270's research harness's does, so the reporter refuses a failed child.
- **The arms** (`examples/ab.rs`): `mibz2-pipe{2,3,4}-both-c29`, a pipeK-both arm (its plan,
  window and levers, which are `PIPELINED_LEVERS`) with `register_list` set. In one build,
  `mibz2-pipe3-both-c29` differs from `mibz2-pipe3-both` in the leader's MTF kernel alone.

**Why opt-in, default off.** The kernel's benefit was measured in #270's copied research worker,
not in the shipped one, and only on CI's oversubscribed 4 vCPUs. The Optimiser's point stands:
neutral unexposed CI cells cannot establish a changed-path benefit, and bench.yml's cells never
run the pipelined teams. A default-on change would alter the shipped decoder (`PIPELINED_PLAN`,
the i9's 10 threads) on the strength of a measurement of a different worker on a different
machine. Default off keeps the shipped decoder byte for byte and instruction for instruction what
it was on every path but one branch a block, puts both kernels in one binary so the diagnostic
compares them paired by round with nothing else different, and leaves the default-on decision to
a production gate after the diagnostic (and, for the target machine, Louis's run).

## 2. Correctness (B5)

Every check below was run on `src/` and `tests/` as at `2153cd40` (the code commit; the
commit after it changes only `examples/ab.rs`'s record, the bench files, the workflows and
this journal), rustc 1.97.0, on the shared 4-core machine, one test binary at a time. No
timing was taken: nothing here is performance evidence.

| obligation | evidence | result |
|---|---|---|
| checkpoint and error semantics: C29 = `mtf_checkpoints` (result, column, counts, checkpoints; `RunTooLong`, `BlockTooLong` in a run and at a byte, no end-of-block), each kernel | `mtf::tests::register_checkpoints_reach_every_class` (ported from #270's D049 §4 / D054): 2,811 seeded streams, 2,000 of them malformed (random symbols, alphabets and bounds, a third run-heavy, 40 with no end-of-block symbol), bounds −65 to +65 about each length; the result compared always, the column and counts on errors too, the checkpoints where Ok; every class of an independent input model (`classify`) asserted reached. And `register_checkpoints_are_checkpoints` (6 column shapes at 5 bounds, and RunTooLong) | pass. Classes reached (both kernels): run split at a checkpoint 1,575; run ending at one 211; byte at one 861; block ending at one 172; runs by the vector stores 761,427; runs ≤ 64 by bytes at the bound 627; runs > 64 12,621; Ok 1,439; BlockTooLong in a run 901, at a byte 317; RunTooLong 129; no end-of-block 25 (`2026-11-09-c29-register-checkpoints/tests/native-mtf-tests.log`). The same counts as D054's: the streams are the same seeded streams |
| the 64-byte physical stores, under Miri | `mtf::tests::miri_register_checkpoints`: one stream a class (a run to a checkpoint ending the block, a run across one, a last byte at one with its bound just below, a run of 64 at the bound and past it, RunTooLong, no end-of-block), each column fresh so its capacity is exactly the bound; asserts every class reached. In `miri.yml`'s new step built with `-C target-feature=+avx2,+bmi1`, `MIBZ2_EXPECT_AVX2=1` forwarded, so the test asserts AVX2 was detected; without AVX2 it returns (C29 is then the scalar twin) | pass under Miri with AVX2 selected (nightly `miri 0.1.0 (16d1ffac17 2026-10-26)`, 66.7 s; `mutants/logs/restored.log`); and C29c is UB there (§2.1) |
| the switch selects the kernel | `parallel::c29_tests::the_front_stage_runs_c29_exactly_when_selected`: `front_stage` with `reg` false, true, false, true on each kernel set: C29's calls counted (a `cfg(test)` thread-local in `mtf_checkpoints_reg`), the same front every time | pass |
| composition with the shipped worker | `tests/pipelined.rs`: every test now also runs C29 alone and C29 with the default's levers (`LEVERS` 6 → 8), at every plan (teams of 2, 3, 4; one team or two; tight and wide windows), against the default decoder and libbzip2: the corpora, every level and shape, small blocks among large and several streams, one block and none, a block CRC, a bad primary index, truncation, a reader dropped mid-stream, a writer cancelled. The two MTF-error tests require the failing block staged overlapped (the leader's next-block path) for C29 as well as for the shipped kernel, apart. `pipe_with` asserts `c29_stages() == 0` with the switch off and `> 0` with it on for any decode with output; `default_33_same` asserts the default at 3 × 3 stages nothing by C29 | 12 passed (`2026-11-09-c29-register-checkpoints/tests/pipelined.log`) |
| composition, fuzzed | `examples/fuzz.rs` MODE `c29` (§2.2) | 0 failures |
| the unwind tests | `parallel::unwind_tests` unchanged (the switch off: the faults are in the run, not the stage) | in CI's `--lib` and Miri runs |

### 2.1 Mutants

Five mutants, each a committed patch in `2026-11-09-c29-register-checkpoints/mutants/`, applied
to a clean tree and reversed by `mutants/mutants.sh`, which logs each run to `mutants/logs/` and
then runs every killing command on the restored tree (`summary.txt`):

| mutant | the change | killed by |
|---|---|---|
| C29a-checkpoint-before-part | a split run's checkpoint taken before its part is counted | KILLED: all three `mtf::tests` fail (the checkpoints differ) |
| C29b-final-checkpoint | the final checkpoint of a block ending on one dropped | KILLED: all three `mtf::tests` fail |
| C29c-vector-store-bound | the vector run store taken while `n < max`, not `n + 64 <= max` | KILLED: natively a heap-corruption abort (SIGABRT) in `mtf::tests`; **under Miri's AVX2 build, UB**: a 32-byte store 31 bytes from the allocation's end (`run_store!`'s `_mm256_storeu_si256`), as D054 found |
| C29d-lever-dropped | `pipelined_levers` drops `register_list` (the switch never reaches the worker) | KILLED: `tests/pipelined.rs`'s `an_mtf_error_after_a_small_block`, "C29 never staged a block" |
| C29e-front-stage-ignores-switch | `front_stage` calls `mtf_checkpoints` whatever `reg` says | KILLED: `parallel::c29_tests` (C29's calls 0 with `reg` set) |
| (restored tree) | every killing command | PASS: `mtf::tests` 3 passed; the AVX2 Miri test passed (66.7 s of Miri); the pipelined test and `parallel::c29_tests` passed |

C29a, b and c are #270's D054 mutants re-applied to master's copy of the kernel. C29d and e are
new: C29's output is the shipped kernel's by design, so no output check can tell whether the
switch reached the kernel. C29d is killed by the reach assertions (the counter), C29e by the
`cfg(test)` call count. Neither shows the kernel's speed; the diagnostic (§3) is what would show a
switch that selected nothing, as a ratio of 1.

### 2.2 The fuzz

`examples/fuzz.rs` MODE `c29`: every case a pipeline case (#273's: `pipeline_corpus`'s multi-block
inputs, one case in eight valid, else one or two edits aimed at a later block's start, the
stream's end or anywhere), each held to the plain parallel decoder exactly (bytes before any error,
the error's kind, block and bit) and to libbzip2. Its candidates (`tests/common`'s
`c29_candidates`) are the default's pipelined teams, three of three in its 12-block window with
park and small, with `register_list` set, at writes and reads of 1 MiB / 64 KiB, 997 / 509 and
100,003 / 4,093 bytes, and one team of three in 4. The default constructors can't select C29, so
each is `pipelined_levers`. A run in mode `c29` fails if any candidate that ran had no block
staged by C29; a run in any other mode fails if any block was. CI's fuzz-smoke runs it 30 s at
seed 1 beside the classic and pipeline modes.

| run | cases | failures | reached the pipeline (a job overlapped) | an overlapped job's error | cases / blocks staged by C29 |
|---|---|---|---|---|---|
| 30 s, seed 1 | 682 | 0 | 58/170, 44/171, 44/171, 91/170 | 1, 1, 0, 6 | 148/769, 143/599, 146/649, 145/568 |
| 180 s, seed 2 | 4,259 | 0 | 288/1,064, 299/1,065, 325/1,065, 578/1,065 | 3, 8, 2, 17 | 895/3,902, 906/3,895, 925/4,049, 906/3,684 |

| `pipeline` mode (the switch off), 30 s, seed 1 | 644 | 0 | the default's candidates, unchanged | | none staged by C29 (the run fails if any is) |

(Each cell lists the four candidates in the order above. The raw outputs are in `fuzz/`. Error
outcomes reached include BlockTooLong 106 and RunTooLong 6 at seed 2: the MTF stage's own errors,
through the switched kernel.)

## 3. The registration (RULES C1). DRAFT, NOT FROZEN

**The question.** In the shipped pipelined teams (the default's worker, levers and window), does
the register-list checkpoint kernel in the leader's front stage make the whole decode faster than
the shipped kernel, in the same binary, paired by round?

**The arms** (one build of `examples/ab`, no features, release; `--arms` in this order):

| arm | what | role |
|---|---|---|
| `mibz2-pipe3-both` | `BzReader::parallel_pipelined_levers`, THREADS / 3 teams of 3, `SLOTS_PER_THREAD` blocks a team, `PIPELINED_LEVERS` (park, small): the shipped default's pipelined path at `PIPELINED_PLAN` | the control, the denominator of every ratio |
| `mibz2-pipe3-both-c29` | the same, `register_list` set | C29 |

**The setting.** `sweep.yml`, `tool: c29`, `rounds: 20`, `registered_head` the frozen head; one
GitHub-hosted runner (4 vCPUs), every CPU, unpinned; `ab run --threads 9,3 --rounds 20 --settle
0.25` on the Wikidata corpus (bench/wikidata.json's windows, SHA-256 `82f65ace…c29915`, the same
corpus as #270). 9 threads is three teams of three, the shipped pipelined plan (judged); 3 threads
is one team of three (secondary, never judged). 20 rounds (df = 19) rather than #270's 15: the run
has two arms, not seven, so it costs 80 decodes, and the narrower interval keeps the threshold's
margin (below).

**The plumbing was smoked locally** (not timed evidence, and not kept): `ab run` of the two arms,
2 rounds at 9 and 3 threads on a 6 MB stand-in file named `wikidata.bz2`, read by the reporter
`--unjudged --rounds 2`: VALID, both plans seen as registered ((3, 0, 6) and (1, 0, 2)), the
outputs equal. Its ratios mean nothing and are not reported.

**VALID** (`bench/c29_checkpoints.py report`, shaped on #270's `bench/run_fed_leader.py`; its
admission is checked on synthetic runs by `bench/test_c29_checkpoints.py`, in CI's test workflow).
Every predicate must hold, else INVALID with its reasons, exit 2, and no reading:
- the identity (written by the workflow around the run, schema `c29_checkpoints/identity/1`): the
  run's head the registered head (a full 40-character commit), a clean tree over the sources the
  build and the reporter read, no features, release, rustc recorded, the harness binary's SHA-256
  the same before and after, the corpus's the registered digest before and after, the registered
  arms, threads [9, 3] and rounds;
- every record: exit 0, no error, a hash, a positive output length, finite positive wall and CPU,
  the corpus `wikidata`, and the plan seen running (ab's `observed_threads`): (3 leaders, 0 plain
  helpers, 6 pipelined helpers) at 9 threads, (1, 0, 2) at 3;
- every (arm, threads) has rounds 0..19 exactly once, each round's arms in ab's rotation;
- one output: the control's hash and length the same in every round at both thread counts, and
  C29's the control's in its round.

**The reading** (judged at 9 threads; the same rule's reading at 3 reported as secondary). The
ratio is C29 over the control, wall and CPU, each the geometric mean of the 20 paired per-round
ratios with its 95% t-interval (df 19):
- **LEAD:** wall ≤ 0.98, its interval's upper end < 1, and CPU's upper end ≤ 1.02;
- **STOP:** wall's lower end > 0.99 (no 1% gain);
- **INCONCLUSIVE:** otherwise.

**Why these thresholds.** #270's reg/ctl at 9 threads was 0.9739 [0.9682, 0.9797] over 15 rounds
(a half-width of about 0.006). If the shipped worker gains what the research worker did, 20 rounds
give a half-width near 0.005 and a point estimate near 0.974, so a LEAD needs the effect to be
about as large as #270 measured: 0.98 is #270's own interval's upper end, rounded. A gain under
2% on CI's oversubscribed runner (9 threads on 4 vCPUs, where the leader's serial MTF is most
exposed) would not by itself justify proposing a default change. CPU ≤ 1.02 is a no-harm guard:
#270's reg spent less CPU than its reference (0.9719 [0.9659, 0.9780]), and a kernel that is
faster for more CPU is not this mechanism. STOP at a lower end above 0.99 says the shipped worker
doesn't see the research worker's gain, if so. The 3-thread cell is secondary because #270's
rankings changed between 9 and 3 threads; reg's 3-thread gain was larger there (0.9615 against
pipe3-both), so a 3-thread LEAD with a 9-thread INCONCLUSIVE would be reported as such, never
promoted.

**What a reading permits (PROTOCOL §11).** A LEAD permits proposing `register_list` on in
`PIPELINED_LEVERS`, under its own production gate (the decoder A/B at the shipped plan on the
target machine, Louis's run). It is not itself a production gate change, and it changes no
registered verdict. A STOP pauses the line on CI only; it cannot close the target-machine
question. Every reading is this runner's.

**bench.yml.** This PR's own bench.yml run is expected **neutral on every default cell**: the cells
run 1, 2 and 4 threads, which plan workers, never the pipelined teams, so the switched branch and
the new kernel are never executed. Its only change on those paths is code layout (the new kernel
compiled in, `front_stage`'s branch). A non-neutral default cell would be a finding in itself, to
be explained before any freeze.

## 4. Limitations

- **CI only.** #270's measurements and this diagnostic are on 4 vCPUs with 9 threads; the i9's 10
  threads are another regime (#270's §7). This registration says nothing about the target
  machine.
- **Not list-only.** C29 changes the list (registers, no `s2u` lookup) and the short-run stores
  (two 32-byte stores) together. The diagnostic measures the pair, as #270 did; separating them
  would be another arm.
- **Rejected blocks.** C29 is in the front stage of every block the leader stages, small ones and
  failing ones included, as the shipped kernel is. #270's record arms kept the old kernel on
  rejected blocks; this prototype has no such split.
- **The reach counter** proves the switch reached the stage, and C29e's `cfg(test)` count that the
  stage called the kernel; neither is timing.

## Provenance and reproduction

- The code commit is `2153cd40` on `perf/c29-register-checkpoints`, from master `6145b3a9`.
- The kernel is from research/run-fed-leader `03e52206`, `src/mtf.rs`; the tests from its
  `src/parallel/run_team.rs` (`register_checkpoints_*`, `miri_register_checkpoints`, D049 §4 and
  D054); the mutants C29a–c from its `mutants/`.
- Reproduce the B5 evidence: `cargo test --lib -- mtf::tests parallel::c29_tests`;
  `cargo test --test pipelined`; `cargo build --profile fuzz --example fuzz` then
  `target/fuzz/examples/fuzz 30 1 - c29` (and `180 2`); the AVX2 Miri step as `miri.yml` runs it;
  `bash docs/journal/2026-11-09-c29-register-checkpoints/mutants/mutants.sh`.
