# 2026-11-09: Pipeline fuzz and invariant mutants for 0.2.6's default

**Result: no failure in 20,775 pipeline fuzz cases.** These cases ran the shipped default:
three pipelined teams of three, with park and small, forced through each default entry point.
In 4,706 of the 12,463 cases whose decoder could be counted, a leader took a job while its
helpers inverted another block. Of 12 hand-made mutants of the pipeline's invariants:
- the native suite (tests/pipelined.rs, coop's unit tests, the new fuzz) kills 8;
- Miri's existing `coop::tests::miri_*` kills 3 more: data races, which these native x86 runs
  did not detect (a data race is undefined behaviour in Rust on x86 as anywhere else);
- 1 survives: the run-panic recovery path, which no test reaches.

No decoder bug was found, and the decoder is unchanged.

**Corrections (Guardian D047, after this entry merged in #273).** Each is made in place below,
and listed in §7:
- the 14-minute fuzz ran at 2614d832, whose corpus differs from the merged head's (§2);
- the final harness's CI smoke run is now recorded (§7);
- the Miri and fuzz diagnostics behind M1–M5's verdicts are preserved, with the toolchain
  and flags (§7);
- M12 now has a deterministic worker-level test that kills it
  (`2026-11-09-pipeline-m12-unwind.md`).

**The input.** The Guardian's REVIEW (#145, comment 6729223027; decision D042 on
team-state/guardian) holds back the B5 production-safety qualification of 0.2.6's default,
for this reason: `examples/fuzz.rs`'s candidates never reach the pipeline, and there was no
pipeline-specific fuzz or mutation evidence. A search of the journals, the CHANGELOG and the
tests found no earlier pipeline fuzz or mutation evidence. The closest is tests/pipelined.rs,
which runs fixed cases (corpora, levels, crafted errors, drops). This entry adds the evidence.

Labels used below:
- **checked:** by a run on this machine (the shared 4-core box), as counts, at the commit
  named;
- **unmeasured:** no timing was taken. Local timing on the shared machine is not evidence,
  and none is reported. The cases-per-second figures are only the fuzz's own throughput.

## 1. The pipeline candidates (what changed)

There is no change to the decoder. `tests/common/mod.rs` gains the following:
- **`pipeline_candidates()`, five shapes.** All but the last are the default's own
  constructors, with the plan forced to `PIPELINED_PLAN` by `parallel::with_plan`. No 4-core
  machine selects that plan: `plan_with` needs 10 physical cores and a 20 MiB L3. The shapes:
  - `StreamDecoder::new(4, 16, ..)`, written 1 MiB and read 64 KiB at a time;
  - the same, written 997 B and read 509 B at a time (the streaming shape);
  - `BzReader::parallel(.., 10)`, read 4,093 B at a time;
  - `decode_parallel(.., 10)`;
  - `StreamDecoder::pipelined_levers(1, 3, 4, .., PIPELINED_LEVERS)`: one team of three with
    the default's levers, which overlaps on far more blocks.

  The two `StreamDecoder::new` shapes and the one-team shape assert `is_pipelined()`, and the
  `StreamDecoder::new` shapes also assert 9 threads. The `BzReader::parallel` and
  `decode_parallel` shapes keep their decoder private, so they assert neither: that they run
  the pipeline rests on the source (the same constructor under the same forced plan), not on
  an assertion of their own.
- **`pipeline_corpus()`, multi-block inputs.** Text at levels 9 and 5, random bytes at level 3,
  binary records at level 4, runs at level 9, `ab` repeated at level 6, and text at level 1.
  There are also several streams (levels 9, 1, 9, 2, 1, 9, an empty stream, then level 3), and
  a crafted stream that alternates 300,000-row and 2,000-row blocks. A team engages only on a
  block of `MIN_COOP` rows (262,144) or more, and overlaps only while another job is queued.
  The small blocks exercise the leader's solo path and lever B.
- **`plain_parallel()`, the reference.** It is production's plain workers
  (`with_inverse::<Segmented>(2, 4, ..)`), the same reference tests/pipelined.rs uses.

`examples/fuzz.rs` takes a MODE: `mixed` (the default; every 8th case is a pipeline case),
`classic` (the loop as before, with the same random sequence for a given seed) or `pipeline`.

**A pipeline case:**
1. It picks a corpus entry.
2. It leaves one case in eight valid. Otherwise it makes one or two edits, each aimed at one
   of three places:
   - a block's first 120 bytes (its header, tables and first symbols), on a block after the
     first, so that the blocks before it are in flight;
   - the last 16 bytes (the stream CRC);
   - anywhere.
3. It decodes the input with the candidate, with the plain decoder and with libbzip2.

**A pipeline case fails when any of these happens:**
- a panic;
- the candidate differs from the plain decoder: in the bytes before any error (except for
  `decode_parallel`, which returns none), or in the error's kind, block or bit;
- the candidate and libbzip2 both decode, to different bytes;
- the candidate decodes what libbzip2 refuses.

**Reached path.** `StreamDecoder::pipeline_counts()` already existed. It counts the jobs a
leader took while its helpers inverted another block, and those of them whose move-to-front
failed. A case counts as reaching the pipeline when that first count is above 0. This is
conservative: a case where a team ran a block, but no job was queued during the run, is not
counted. `BzReader` and `decode_parallel` keep their decoder private, so their cases are not
countable. They are the same constructor under the same forced plan as the counted
`StreamDecoder::new` shape. Exposing a count through them would need a change to `src/`, and
this entry makes none.

## 2. The fuzz (checked)

**Setup:**
- **The commit:** 2614d83225083ee927ef91f1c8659b2e5f8c9c76 (this branch's first commit, on
  master 6ea21e5b; `src/` is identical to master's). Every result in this section belongs
  to that commit.
- **Not the merged head's corpus.** The branch's later commit rewrote `pipeline_corpus`'s
  pushes as one `vec![]`, for clippy, and in doing so deleted the "records level 4" entry.
  So the corpus #273 merged has eight entries, not the nine these runs drew from. Since a
  case's entry is chosen from the seed, the same seed picks different inputs at the merged
  head, and these runs' case stream can't be reproduced there. This entry first called the
  change formatting-only ("the same values, built in the same order"); that was wrong.
  The runs stand at their pin: the decoder is unchanged between the two commits.
- **The entry is restored** by the PR that makes these corrections
  (`2026-11-09-pipeline-m12-unwind.md`), so the corpus is again the nine entries above, the
  same values in the same order as at 2614d832.
- **The build:** `cargo build --profile fuzz --example fuzz`, which has debug assertions and
  overflow checks on.
- **The runs:** one job at a time on the shared 4-core machine.
- **The logs:** in `2026-11-09-pipeline-fuzz-and-mutation/fuzz/`.

| run | mode | seconds | cases | pipeline cases | failures |
|---|---|---|---|---|---|
| `fuzz 180 11 - pipeline` | pipeline | 180 | 4,539 | 4,539 | 0 |
| `fuzz 180 12 - pipeline` | pipeline | 180 | 4,468 | 4,468 | 0 |
| `fuzz 180 13 - pipeline` | pipeline | 180 | 4,495 | 4,495 | 0 |
| `fuzz 180 14 - pipeline` | pipeline | 180 | 4,519 | 4,519 | 0 |
| `fuzz 120 21 - mixed` | mixed | 120 | 22,032 | 2,754 | 0 |
| **total** | | **840** | **40,053** | **20,775** | **0** |

**Reached path, by candidate (all five runs):**

| candidate | cases | reached (a job taken overlapped) | cases with an overlapped job's error | overlapped jobs |
|---|---|---|---|---|
| default 3×3, `StreamDecoder::new`, w1 MiB r64 KiB | 4,152 | 1,158 (27.9%) | 8 | 1,947 |
| default 3×3, `StreamDecoder::new`, w997 r509 | 4,156 | 1,263 (30.4%) | 27 | 2,163 |
| default 3×3, `BzReader::parallel`, r4093 | 4,156 | not countable | — | — |
| default 3×3, `decode_parallel` | 4,156 | not countable | — | — |
| 1 × 3 park+small (`pipelined_levers`), s4 | 4,155 | 2,285 (55.0%) | 66 | 6,142 |
| **countable, in all** | **12,463** | **4,706 (37.8%)** | **101** | **10,252** |

**Why most cases don't reach the pipeline.** Most cases fail at their first edited block, or
in the scanner. A team is reached only when blocks before the failure are full blocks
in flight. The 101 cases with an overlapped error are the ones that test error placement
across the overlap: a block staged by a leader during another block's run fails, and its
error must still come after every earlier block's bytes.

**Outcome classes of the 20,775 pipeline cases:**
- 2,728 decode to libbzip2's bytes;
- 18,046 fail as libbzip2 fails, each with the plain decoder's exact error. In order of
  frequency: Truncated 4,812, BlockCrc 4,195, BadCodeLength 2,328, BadBlockMagic 1,521,
  BadHuffmanCode 1,336, BadSelector 893, SelectorOverrun 714, StreamCrc 596, BlockTooLong 519,
  BadOrigPtr 457, BadGroups 383, RunUnfinished 104, Randomised 81, NotBz2 42, RunTooLong 32,
  TrailingData 13, NoSymbols 10, BadSelectors 7, Empty 3;
- 1 is decoded by libbzip2 but refused by this decoder (Randomised), as the plain decoder
  refuses it. This is the documented refusal.

**The 19,278 classic cases of the mixed run** (sequential on both kernel sets, and parallel at 3
threads) also had no failure. Their 3 "libbzip2 decodes, this refuses (Randomised)" cases are
the same documented refusal.

## 3. The invariant mutants (checked)

There are 12 mutants, each a small edit to `src/coop.rs` or `src/parallel.rs`:
- each was applied in a throwaway detached worktree at the commit above, never on this branch;
- `mutants/mutants.py` holds the edits exactly, and `mutants/M*.patch` the diffs;
- `mutants/results.json` holds the exit codes and seconds.

**What each mutant ran, each run with a timeout (a hang counts as a kill):**
- `cargo test --test pipelined` (300 s);
- `cargo test --lib coop::` (300 s);
- for the relay's mutants (M1–M5), `cargo +nightly miri test --lib -- coop::tests::miri_`, the
  CI Miri job's coop tests (900 s);
- the pipeline fuzz, `fuzz 45 7 - pipeline` (285 s), built on the fuzz profile with LTO off.

**The baseline.** The unmutated code (M0) passed all four: 12 + 20 tests, Miri's 10 coop tests,
and the fuzz with 0 failures.

| mutant | invariant broken | tests/pipelined | coop unit | Miri coop | pipeline fuzz | verdict |
|---|---|---|---|---|---|---|
| M1 | build-to-walk ordering: `walk_ready` set before the build's end check and the walk's set-up | pass | pass | **UB: data race** (`miri_split_build_and_walk`) | pass | killed by Miri only |
| M2 | `walk_ready`'s release store and acquire load weakened to relaxed | pass | pass | **UB: data race** (`miri_split_build_and_walk`) | pass | killed by Miri only (not detected natively on x86) |
| M3 | relay handoff: `walkers` starts at helpers − 1, so the last to leave the walk is early | **fail** (9 of 12 tests) | **fail** | **fail** | **fail** (panics in the walk, then SIGKILL: the machine's memory killer) | killed |
| M4 | park: a parked waiter returns on any wake-up without re-checking the flag | pass | pass | **UB: data race** (`miri_split_build_and_walk`) | pass | killed by Miri only |
| M5 | park/unpark ordering: the setter drains the sleepers before storing the flag (a yield in between widens the window) | **hang** | **hang** | **deadlock** (`miri_parked_wait_loses_no_wakeup`) | **hang** | killed |
| M6 | error placement: a move-to-front error published one place late | **hang** | pass | n/a | **fail** (not the plain decoder's error), then **hang** | killed |
| M7 | error propagation: a move-to-front error swallowed, and the block decoded anyway | **fail** (`a_bad_orig_ptr`, both `an_mtf_error_*`) | pass | n/a | **fail** (BlockCrc, or an Internal index panic, where the plain decoder gives BlockTooLong) | killed |
| M8 | the never-wait rule: the leader waits for a job (`recv`) inside a run, holding an unpublished block | **hang** | pass | n/a | **hang** | killed |
| M9 | small-block priming boundary: lever B keeps staging after a block for the team, into the same front set | **fail** (`a_bad_orig_ptr`, `small_blocks_keep_the_pipeline_primed`, `dropped_mid_stream`), then **hang** | pass | n/a | **hang** | killed |
| M10 | buffer reuse: the helpers assemble block i into the buffer that block i − 1 is finished from | **fail** (9 of 12 tests) | pass | n/a | **fail** (BlockCrc, computed 0) | killed |
| M11 | front-set ownership: the next block labelled with the front set that the helpers read | **fail** (9 of 12 tests) | pass | n/a | **fail** (BlockCrc where the plain decoder decodes) | killed |
| M12 | panic completion: a run that panics outside its caught stages publishes none of its blocks | pass | pass | n/a | pass | **survived** |

**Reading the table:**
- **M1, M2 and M4 are undefined behaviour that these native x86 runs did not detect.** A
  data race is UB in Rust whatever the hardware: the language's synchronisation is missing,
  and the compiler may reorder or remove the accesses accordingly. That the output came out
  right here is an observation of these runs, not a property of the code.
  - **M1:** a waiter that sees the stage still `Build` skips the walk and leaves. The thread
    that ended the build then walks every segment itself, and is the last to leave, so these
    runs' output was right. But it is reached through a data race on the stage.
  - **M2:** x86's ordering of plain loads and stores is why these runs didn't show it. It
    doesn't make the relaxed accesses synchronise in the language's model.
  - **M4:** the waiter's early return is the same race as M1's.

  The CI Miri job (`.github/workflows/miri.yml`, `cargo miri test --lib -- miri_`, on PRs
  and pushes touching `src/coop.rs` or `src/bwt.rs`) kills all three. So the existing evidence already
  covers these invariants. No native test could catch them on x86 short of a race detector.
- **M5, M6, M8 and M9 are killed by a hang.** tests/pipelined.rs's plan at one slot,
  `(1, 2, 1)`, makes the never-wait rule (M8) a deadlock for certain, and
  `dropped_mid_stream`'s 60-second watchdog fails it.
- **M12 is a test gap** (closed since: `2026-11-09-pipeline-m12-unwind.md`).
  - **Why nothing reaches it:** the leader's own stages each catch their panics
    (`finish_and_publish`, `stage`, `solo`), and a helper's panic is returned as the run's Err,
    not a panic. The `Err(p)` branch of `pipelined_worker`'s run is reached only by a bug
    outside those stages, and no input can cause one.
  - **What a kill would need:** a fault-injection hook inside `pipelined_worker`, a change to
    `src/`. That is not a cheap test, and it was not added. The coop-level panic tests
    (`pipelined_leader_panic`, `miri_pool_*`) cover the pool's half of panic completion. That
    is: the run waits for its helpers and resumes the panic. They do not cover the worker's
    publication of the held blocks.

## 4. The CI hook

**There was no fuzz job in CI.** `.github/workflows/test.yml` gains `fuzz-smoke`:
1. it builds the fuzz profile;
2. it runs `fuzz 30 1 fuzz-failures/classic classic`, then
   `fuzz 30 1 fuzz-failures/pipeline pipeline`;
3. it uploads any failing inputs.

**Its cost:** the 60 s of fuzz plus the build. On this machine the build took 20 s from a cold
dependency build, so the build cost on a runner is about 1–2 minutes (unmeasured there). The
job runs beside the others, not in series with them, and adds nothing to the bench. Its seeds
are fixed, but that doesn't make it deterministic: the threads' scheduling, and how many
cases fit in its 30 seconds, vary from run to run. On CI's 2-core runners the 3 × 3 default runs 9
decoding threads, oversubscribed, which also tests the pipeline under preemption.

## 5. What is not covered

- **The run-panic recovery path** (M12, above). Closed since by a deterministic worker-level
  test (`2026-11-09-pipeline-m12-unwind.md`).
- **Weak memory orderings on real hardware.** Everything here ran on x86, and only Miri
  checks the orderings, on coop's unit-level scenarios. There is no ARM run.
- **The 3 × 3 plan's own topology.** The plan was forced on a 4-core machine, so its 9
  decoding threads ran oversubscribed. The i9's 10-core timing of the interleavings was not
  reproduced. Because the threads are oversubscribed, preemption at any point is more likely
  here than there.
- **Per-case reach through `BzReader::parallel` and `decode_parallel`.** These cases are not
  countable without a `src/` accessor. Their constructor is the counted one's.
- **The fuzz's reach.** Reach is about 30% of the default 3×3's countable cases, and 55% of
  the one team's. It is bounded by how often an edit leaves several full blocks before the
  failure.
- **The real Wikidata dump windows on the pipeline.** `examples/windows.rs` still runs only
  `candidates()`. It is a bench input, and was left alone.
- **Duration.** This is a bounded 14-minute local fuzz, not a long campaign.

## 6. Also in this PR: the Guardian's wording corrections

- **CHANGELOG, 0.2.6, the pipeline alone.** "for 0.918 [0.895, 0.942] of their CPU" read the
  CPU ratio the wrong way. The ratio is the reference's CPU over the arm's
  (2026-11-07-team-pipeline-result.md: "below 1 the arm spends more"). So the pipeline spends
  about 8.9% more CPU (1/0.918 = 1.089; the interval gives 6.2–11.7%). The number is kept,
  and its meaning is corrected.
- **README and CHANGELOG, the low-level regression.** "About 23% slower" is a speed ratio of
  0.763 (level 1) and 0.776 (level 2). That is about 23% lower throughput. In time, such a
  file takes 1/0.763 = 1.31 and 1/0.776 = 1.29 times as long: about 29–31% longer. Both
  readings are now stated.

## 7. Provenance and verification (added for Guardian D047)

**Where each result belongs:**
- the fuzz runs of §2: 2614d832, with the nine-entry corpus;
- the mutants of §3: applied at the same commit, in a detached worktree;
- #273's merged head (44b0b1a3): the eight-entry corpus, checked only by CI's smoke run below.

**The final harness's CI smoke run (verified by the Guardian, D047).**
- **The job:** 128934955612, Test run 42547303818.
- **What it checked out:** the PR's merge commit 83fe1bbfb9bddbd8a575bc13086b367678ee8a45.
  That is not literally the reviewed head 898482b9, but its diff from that head touches
  only bench/run-queue.sh, two documentation pages, PROTOCOL.md and zensical.toml. The
  decoder, the fuzz's source and the Cargo inputs are the same.
- **The result:** 84,142 classic cases and 1,020 pipeline cases, with zero failures. 215 of
  the 612 countable pipeline cases reached the pipeline, 4 of them with an overlapped error.

This verifies the final harness's smoke run. It doesn't relabel the longer campaign above,
which belongs to 2614d832.

**The diagnostics behind the verdicts.** These are the original logs of the §3 run, kept in
`2026-11-09-pipeline-fuzz-and-mutation/mutants/logs/`:
- **The toolchain:** `cargo +nightly miri test --lib -- coop::tests::miri_`, on rustc
  1.101.0-nightly (16d1ffac1 2026-10-26), miri 0.1.0 (16d1ffac17 2026-10-26). That nightly
  was installed on 27 October, and was the machine's only nightly through the run.
- **The flags:** `MIRIFLAGS` was empty. The driver (`mutants.py`) passes the environment's
  value or "", and the logs' hint to rerun "with `MIRIFLAGS=-Zmiri-backtrace=full`" is
  Miri's default output. `CARGO_TARGET_DIR` was the worktree's `target-miri`.
- **The profile:** the test profile (opt-level 3, which Miri ignores; debug assertions on).

What each log shows:

| mutant | log | Miri's diagnostic |
|---|---|---|
| M0 (the baseline) | `M0-miri.log` | 10 passed, 0 failed (276.81 s) |
| M1 | `M1-miri.log` | `miri_split_build_and_walk`: "Undefined Behavior: Data race detected between (1) non-atomic write on thread `mibz2-coop-1` and (2) non-atomic read on thread `coop::tests::mi`". The read is the stage's (`src/coop.rs:1009`, `walk_and_leave`), the write the build's end replacing it (`:972`). |
| M2 | `M2-miri.log` | the same test: a data race between the write `*stage = Stage::Walk(w)` on `mibz2-coop-1` (`:999`) and the walk's read of the stage (`:1009`) |
| M4 | `M4-miri.log` | the same test: a data race between the walk's read of the stage (`:1009`) and a retag write of `coop::Stage` on `mibz2-coop-1` (`:968`, `end_build`) |
| M5 | `M5-miri.log` | `miri_parked_wait_loses_no_wakeup`: "the evaluated program deadlocked", every thread stuck in a futex wait, the waiters spawned at `src/coop.rs:1964` |

Line numbers are those of the mutated `src/coop.rs` at 2614d832.

**M3's SIGKILL.** `M3-fuzz.log` ends in panics in the walk ("attempt to add with overflow",
`src/coop.rs:1044`), and `results.json` has the fuzz's exit as −9.
- **What the kernel logged:** `M3-dmesg-oom.txt` is the kernel log's out-of-memory lines from
  that boot. It shows the memory cgroup's OOM killer ending a process named `fuzz` at uptime
  3,820 s, about 00:50 UTC (the boot was at 23:46:39). That fits M3's fuzz, whose log was
  last written at 00:49:47 (kernel timestamps drift from wall time). It is the only `fuzz`
  process killed in that boot.
- **Read as:** consistent with the memory killer, not proof that it was M3's process.

**The restored baseline.** The driver restores the source in a `finally` after each mutant.
That restores the file, but it is not itself evidence that the restored code passes. The
baseline evidence is M0's run of all four suites, made before the mutants (`baseline-M0.json`,
`M0-miri.log`). No post-restore run was made.

**The timeouts.** The driver's timeout kills the direct subprocess (cargo, or the fuzz
binary). It does not kill or reap the process group, so it doesn't show that cargo's test
binaries ended before the next mutant. No leftover process was seen, and the recorded hangs
stand as kills. The next hang-targeted run should use a watchdog that ends and reaps the
whole process tree.
