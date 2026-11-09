# 2026-11-09: A deterministic worker-unwind test kills M12

**Result: M12 is killed.** The new tests inject a panic into a pipelined team's run,
outside its stages' own catches, so that it unwinds through the pool into
`pipelined_worker`'s outer `Err` branch. With the branch as it is, every state passes. With
the M12 mutant applied unchanged, every state fails, each at a bounded watchdog. Restored,
every state passes again. Miri runs the same composition on tiny blocks. The decoder's
behaviour is unchanged: what the PR adds to `src/` is test-only, apart from one function
that returns `MIN_COOP` outside tests.

**The input.** The Guardian's D047 (#145, comment 6730448704; `state/decisions.md` on
team-state/guardian at bab6f816) narrows D042's HOLD to this one recovery obligation. M12
(`2026-11-09-pipeline-fuzz-and-mutation/mutants/M12.patch`) drops the publication of the
run's current and staged-next blocks in that branch. Every native suite passed it, because
no input reaches the branch. D047 asks for:
- a deterministic injection into the real outer branch;
- three states (current; current + next; current + a held prior);
- checks of retirement, order, the reader and the join, under a watchdog;
- the M12 kill, then the restored baseline passing.

The same PR widens Miri's trigger paths and corrects the fuzz entry's provenance (that
entry's §7).

Labels used below:
- **checked:** by a run on this machine (the shared 4-core box), at the commit named;
- **unmeasured:** no timing was taken, and none is needed: the tests are correctness checks.

## 1. The hook (what changed in `src/`)

**The design.** `Shared` gains a `faults` field (`Faults`, behind a mutex), and three
call sites act on it. All of this is `cfg(test)`:
- **`inject`, in the leader's part of the run.** It is called at two points of the leader's
  closure in `pipelined_worker`:
  - `Fault::LeadStart`, on entry, before the leader finishes the prior block it holds;
  - `Fault::LeadEnd`, on its way out, after it has published the prior and staged the next
    job.

  If the fault is set for this run's block, it records what the team holds (the prior
  inverted block, the next staged block) and panics. Both points are outside every catch
  of the leader's (`finish_and_publish`, `stage`, `solo`), and the helpers are inverting the
  block meanwhile. So the panic takes the real route: `Pool::run` waits for the helpers and
  resumes it, `invert_pipelined_with` unwinds, and `pipelined_worker`'s `catch_unwind`
  returns `Err(p)`. A helper's panic, caught by the relay, would take the `Ok(Err(..))`
  branch instead: the wrong target, as D047 says.
- **`publish`** records every block it publishes, with its error if any. This lets a test
  check "exactly once", and the held blocks hidden behind the reader's first error.
- **The outer `Err` branch** counts its recoveries, so a test asserts that it was the branch
  reached. The count goes in before the branch's publications, and outside the lines
  `M12.patch` touches, so the patch still applies unchanged.
- **`min_coop(shared)`** replaces the `MIN_COOP` constant in `stage`'s threshold. Outside
  tests it returns `MIN_COOP`. Under test, `Faults::min_coop` can lower it, so that blocks
  small enough for Miri still run on the team.

**The size:**
- `src/parallel.rs`: +78 −1. The only change outside `cfg(test)` is `min_coop`: a function
  that returns `MIN_COOP`, called where the constant was.
- `src/parallel/unwind_tests.rs`: 506 lines, new.

**Why the tests drive `pipelined_worker` directly.** A test builds the decoder's `Shared` as
the scanner would leave it: every place issued and the end known. It queues every job in
full before the worker starts, then hands the worker to a `StreamDecoder` for reading and
joining. Then `try_job` finds the next job at every run, and nothing else takes it, so each
run's state is fixed. The jobs themselves are real: the scanner's output over real `.bz2`
streams, with their real first bits. A last test runs the default decoder (three teams of
three, forced) end to end through the scanner, where the state at the fault is up to the
threads.

## 2. What is checked, in each state

Each test decodes four blocks, each its own stream, and injects one fault.

| state | fault | the team holds when it fires | the reader must see | hidden behind the error |
|---|---|---|---|---|
| current | block 3's run, `LeadEnd` | block 3 alone (block 2 published in the run, nothing queued) | blocks 0–2, then block 3's `Internal` | — |
| current + next | block 1's run, `LeadEnd` | block 1, and block 2 staged | block 0, then block 1's `Internal` | block 2's `Internal`; block 3 decoded on the reset team |
| current + held prior | block 2's run, `LeadStart` | block 2, and block 1 inverted, its RLE1 not yet run | block 0, then block 1's `Internal` | block 2's `Internal`; block 3 decoded on the reset team |

**Each test checks, every wait under a 60-second watchdog:**
1. **The reader:** the bytes are exactly the blocks before the first held block, and then the
   error comes. It is `Internal`, with the injected message, and the place and bit of that
   block. If the reader stalls, the watchdog cancels the decoder and fails the test, naming
   the place it is stuck at and the places ready. This covers the stranding case.
2. **The state:** the fault fired, the team held what the table says, and the outer branch
   recovered exactly once.
3. **The retirements:** every block is published exactly once. Each held block is `Internal`
   with its own place and bit, and no other block has an error.
4. **The reset:** the worker ends of itself once its jobs are done, without a cancel. Every
   block left in `ready` after the reader's error is decoded, with its source's bytes.
5. **The join:** `StreamDecoder::join` ends.

**The variants:**
- **Full blocks** (`a_run_panic_retires_the_*`). Each block is 272,144 rows, above
  `MIN_COOP`, so the team's real threshold is used. They run on teams of 2, 3 and 4.
- **Tiny blocks** (`miri_run_panic_*`). Each block is 240 bytes, `TINY`, compressed by
  Python's bz2, since libbzip2 can't run under Miri. They run on a team of two, one helper
  live through the unwind, with the threshold lowered to 1. They run natively and under
  Miri.
- **The default decoder end to end** (`the_default_decoder_recovers_from_a_run_panic`). It
  decodes six full blocks, with faults at (2, `LeadStart`), (2, `LeadEnd`) and
  (4, `LeadStart`). The error expected is at the first block the run held, read from what
  `inject` recorded. Every block must be published once, the branch reached once, and the
  join must end.

## 3. Results (checked)

All of these ran at 44b0b1a3 plus this PR's `src/` change, in one script, one step at a time.
The logs are in `2026-11-09-pipeline-m12-unwind/`, each headed by its command and the
SHA-256 prefixes of the two source files. The steps were: the baseline, natively and under
Miri; `M12.patch`, natively and under Miri; and the restored file, natively.

**The baseline passes.**
- **Natively:** all 8 tests passed (`baseline.log`). That is 7 fault tests, plus the check
  that `TINY` decodes to its chunks. An earlier version of the file, differing only in Miri's
  watchdog, passed 30 repeats of the suite.
- **Under Miri:** `cargo +nightly miri test --lib -- parallel::unwind_tests::miri_` ran 3
  tests, and all passed, in 73.7 s of Miri's clock (`miri.log`).
  - **The toolchain:** rustc 1.101.0-nightly (16d1ffac1 2026-10-26), miri 0.1.0 (16d1ffac17
    2026-10-26), with `MIRIFLAGS` unset.
  - **What it found:** no data race, leak or deadlock. That covers the unwind through a live
    helper, the retirements, the team's reset (its pool dropped and a new one spawned) and
    the reader.

**M12 is killed, natively and under Miri.** `M12.patch` was applied with `git apply`,
unchanged (at an offset of 70 lines). It adds `.take(0)` to the loop over
`[Some(s), l.next.take()]`.

Natively, `RUST_BACKTRACE=0 cargo test --lib unwind_tests` gave 7 failed and 1 passed, in
120 s (`m12-mutant.log`):

| test | how it failed |
|---|---|
| current (full, k 2) | watchdog: "the reader is stuck at place 3, with places [] ready" |
| current + next (full, k 2) | watchdog: "the reader is stuck at place 1, with places [3] ready" |
| current + held prior (full, k 2) | watchdog: "3 of 4 blocks published, places [0, 1, 3]" |
| tiny, current | watchdog: reader stuck at place 3 |
| tiny, current + next | watchdog: reader stuck at place 1, place 3 ready |
| tiny, current + held prior | watchdog: 3 of 4 published, place 2 missing |
| default 3×3, (2, `LeadStart`) | watchdog: "the reader is stuck at place 2, with places [3, 4, 5] ready" |
| `the_tiny_blocks` | passed (it injects nothing) |

Under Miri, all 3 failed the same way, each at its watchdog (`m12-mutant-miri.log`). That
took 1,842 s of Miri's clock and 50 s of wall time.

**Reading the M12 failures:**
- **Current + next is D047's stranding case.** Block 3 finished on the reset team and sits
  in `ready`, and the scanner's end is known. But places 1 and 2 never come, so the reader
  waits at 1 for ever.
- **Current + held prior.** The reader does get an error at the right place: M12 leaves the
  prior's separate publication alone. Only the private check catches it: block 2, owned by
  the run, is never retired.
- **The default end-to-end case.** It caught M12 too. In this run, the team that ran block 2
  held no prior, so the reader stalled at 2. In an earlier run of the same mutant, before
  Miri's watchdog was shortened, that team also held block 1. Then the reader got block 1's
  error, and the check that failed was "5 of 6 blocks published, places [0, 3, 1, 4, 5]".
  The test passes or fails on either state.

**The restored baseline passes.** `git apply -R` restored the file, byte for byte the
pre-mutant file (`cmp`), and all 8 tests passed (`restored.log`).

## 4. The CI hook

`.github/workflows/miri.yml` changes in two ways:
- **Its trigger paths** add `src/parallel.rs`, `src/parallel/**` and `src/read.rs`, for both
  `pull_request` and the push to master. `src/bwt.rs`, `src/coop.rs` and the workflow stay.
- **A new step before the Miri step** lists the tests `--lib -- miri_` selects, and fails
  unless `parallel::unwind_tests::miri_run_panic*` is among them. The Miri step itself is
  unchanged: the new tests are library unit tests named `miri_`, so its filter already runs
  them. The check is there so that a rename can't drop them without notice.

tests/pipelined.rs stays native. It is an integration test, which `--lib` never runs, and
full-size.

**Its cost:** the three tests took about 40 seconds of wall time under Miri here. On CI's
runners that is unmeasured. The Miri job runs only on the paths above, and beside the
others.

## 5. Also in this PR

- **The "records level 4" corpus entry is restored** in `tests/common/mod.rs`. D047 left a
  choice: restore it, or document its removal. Restored, `pipeline_corpus` is again the
  nine entries the 14-minute fuzz ran at 2614d832, the same values in the same order.
- **The fuzz entry's provenance is corrected** (`2026-11-09-pipeline-fuzz-and-mutation.md`,
  its opening and §7):
  - the 14-minute results are kept at their actual pin;
  - the final harness's CI smoke run is recorded;
  - the original Miri logs for M0–M2, M4 and M5, M3's fuzz log and the kernel's OOM lines
    are preserved, with the toolchain and flags;
  - the wording that called data races harmless on x86 is gone.

## 6. What is not covered

- **Other points outside the catches.** The hook injects at two points in the leader's part.
  `invert_pipelined_with`'s set-up also runs outside the catches, before the pool's run (the
  build's `debug_assert!` on a refused checkpoint set). A panic there would leave the team
  holding the current block and the unfinished prior, the state `LeadStart` sets up. But
  with no helper started, it was not injected separately.
- **A panic outside the run.** For example, in the worker loop's own code between runs. The
  outer branch doesn't catch it, so it is a different path. Nothing here tests it, and D047
  doesn't ask for it.
- **The two non-countable shapes,** `BzReader::parallel` and `decode_parallel`. They are
  not driven with a fault. Their worker is the one tested.
