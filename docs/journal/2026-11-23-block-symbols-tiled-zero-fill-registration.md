# bench-block-symbols-tiled-zero-fill: registration

*23 November 2026. A draft for review, written before any data, on
`bench/block-symbols-tiled-zero-fill`. The recipe is `just bench-block-symbols-tiled-zero-fill`
(`.just/bench.just`). The verdict is `bench/block_symbols_tiled.py`'s, tested by
`bench/test_block_symbols_tiled.py`: a wrapper around `bench/parscan_verdict.py`, copied
unchanged from `bench/parscan-stream` (#771), with its tests. The recipe is bench-parscan-stream's
(its builds, contrast check, corpora, machine records and untimed preflight) on a six-count
grid with its own rule. It is frozen once the Coordinator accepts its review; until then nothing
here is final and no run is queued.*

**The decision it settles.** Whether perf/block-symbols-tiled-zero-fill (#785) merges: the block
reader's symbol buffer zero-filled in tiles of 8,192 symbols just ahead of the groups that write
them, in place of one fill to `max_out` before the first group. It is an adoption registration
(RULES C6): CI's 4-CPU runners did not resolve this effect on #770, so a neutral CI bench is
expected, and the merge would rest on this run, outside PERFORMANCE-POLICY's CI rule.

- **ADOPTS:** the PR merges, provided CI's condition below is met, and ships in the next
  release; DELIVERED.md gets its line.
- **HELD ON MEMORY:** not ADOPTS. The Coordinator weighs the growth under PERFORMANCE-POLICY
  rule 4, from this run's figures alone.
- **NOT ADOPTED:** the PR closes unmerged. The result gets a journal entry and a line in the
  record; any later attempt at the same mechanism is a new PR with its own registration.
- **VOID:** repeated once, with the reason logged.

**CI's condition, frozen before any i9 data** (the acting Researcher's correction, #145 REVIEW
7096174288; accepted in LOG 7096174751). #785's CI paired bench (run 45097921063, head
`39aaf55b`, 10 rounds an attempt) so far:

| attempt | wikidata, 1 thread | text, 2 threads |
|---|---|---|
| 1 | −0.5% (−1.6, +0.6) | +4.6% (+0.0, +9.4), flagged SLOWER |
| 2 (text 2t's one re-run) | +0.4% (+0.1, +0.7), flagged SLOWER | +1.2% (−1.7, +4.3), not reproduced |
| 3 (wikidata 1t's one re-run) | read alone, as below | settled by attempt 2 |

Text at 2 threads is settled: its flag did not reproduce (PERFORMANCE-POLICY §2). No other primary
cell was flagged in either attempt. Attempt 3 is read at wikidata, 1 thread, alone:
- **SLOWER beyond noise again:** the flag stands, and #785 is **NOT ADOPTED** with no i9 run.
- **Otherwise:** CI's condition is met and is never revisited. This run alone decides.

The ready tag is set only once attempt 3 has reported and its outcome is logged on #145.

## Where this comes from: a confirmatory test, not a fishing expedition

PR #770 (perf/parscan-stream, head `2618c54c`; the push-streaming parallel front end of Carbon's
final handoff) was NOT ADOPTED on the i9: from 8 threads its front end starved the workers (#145
LOG 7089082075). Its run (evidence branch at `f693e20b`, `parscan-stream/run-20261123T005713Z`,
SHA-256 `e432a2ba1fc997fe82bf91438e31dab20df9392e6a0cdb5946e3899d44340bb7`) also measured the
head against master at 1, 2, 4 and 6 threads, where the front end does not run (its thread
census: no helper scanner on either arm below 8). There, the one change that runs is the block
reader's zero-fill:

| corpus | threads | wall, head over base [95%] | step |
|---|--:|--|---|
| wikidata-x4 | 1 | 1.0166 [0.9985, 1.0351] | INCONCLUSIVE |
| wikidata-x4 | 2 | 0.9959 [0.9770, 1.0152] | INCONCLUSIVE |
| wikidata-x4 | 4 | 0.9778 [0.9712, 0.9844] | FASTER |
| wikidata-x4 | 6 | 0.9771 [0.9712, 0.9831] | FASTER |
| text | 1 | 1.0187 [0.9907, 1.0475] | INCONCLUSIVE |
| text | 2 | 1.0105 [0.9966, 1.0246] | INCONCLUSIVE |
| text | 4 | 1.0126 [0.9995, 1.0258] | INCONCLUSIVE |
| text | 6 | 1.0145 [0.9892, 1.0404] | INCONCLUSIVE |

Those cells were reported or supporting in #770's rule and decided nothing there. This
registration names the effect first seen in them and tests it once, on its own, with the
judged count fixed where it was seen: **6 threads, where #770's run measured 0.977.** (4, where
it measured 0.978, is a supporting count: one judged count, chosen before this run's data, and
not the better of two.) The split PR is not byte-for-byte #770's code (below), so this run
measures the split itself.

## The hypothesis

**What changed** (perf/block-symbols-tiled-zero-fill, from master; `src/block.rs` alone, plus
tests): `read_item_with_scratch`, the block reader of the serial scanner and of the one-thread
decoder, read master as:

```rust
let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
if syms.len() < max_out {
    syms.resize(max_out, 0);
}
let out = &mut syms[..max_out];
```

and then the group loop wrote 66 symbols a group into `out`. The split drops the fill before the
loop and fills inside it, a branch a group:

```rust
if n + 66 > syms.len() && syms.len() < max_out {
    syms.resize((n + ZERO_FILL_TILE).min(max_out), 0);   // ZERO_FILL_TILE = 8192
}
```

The upfront `reserve(MAX_SYMS)` stays. The output is byte-identical: every symbol below `n` is
written by a group before it is read, and the buffer is truncated to `n` at the end, as before.

**What #770 carried and the split leaves out.** In #770, `read_item_with_scratch` became an
`#[inline]` wrapper over `read_item_core(.., tight: false)`, beside `read_item_tight` (`tight:
true`), which grows the buffer's capacity in quantized steps for the front end's bounded-memory
scanners. `read_item_tight` exists only for the front end and runs nowhere below 8 threads, so
the split has no `tight` parameter, no `read_item_core` and no wrapper: its reader is master's
function with the tiled fill. The rest of #770 that compiles into the base paths (the worker
loop's body moved into `run_worker_job` for the front end's second worker kind, `parscan_here`'s
early return below 8 threads, and `pub(crate)` visibility) exists to support the front end, does
nothing on its own, and is left out.

**The mechanism, as a hypothesis.** Master writes zeros from the buffer's length to `max_out`
(up to 900,066 symbols, 1.8 MB) in one pass, then writes the same memory again, group by group.
The fill runs ahead of the decode by up to the whole block, so the lines it touched have left
L1 and L2 (32 KB and 256 KB a core on the i9) before the groups write them. The tiled fill
touches 16 KB, then the next 8,192 symbols' groups write it while it is in L1. How much master
fills depends on the buffer handed in:
- **The serial scanner of the parallel decoder** (2 threads and up) takes its buffer from the
  pool of symbol buffers (`free_syms`), or a new one when the pool is empty. A buffer comes back
  to the pool at its block's length (`decode_owned` hands the symbols back truncated to `n`), so
  master fills `max_out` less the previous block's `n` when that is positive. A whole fill
  happens only for a fresh buffer, at most `slots` in a run. #770 measured its gain at 4 and 6
  threads; which of these fills it came from is not known.
- **The one-thread decoder** reuses one buffer, whose length is the last block's symbol count,
  so master fills only the difference between blocks. Little changes there but the branch a
  group: consistent with #770's 1.017 at one thread, inconclusive.

No pool census or cache counter was taken; the run tests the wall-time effect, not this
account of it.

**The risks, named:**
- **The branch a group** (about 18,000 a block), at every count. One thread, where the fill
  saved is smallest, is where it would show: #770's 1.017 [0.998, 1.035] there was close to
  SLOWER, and 1 is a supporting count here, so the rule may refuse on it. CI measured it on
  #785: in attempt 2's stage table at one thread, the scanner stage (headers and Huffman) costs
  +5.8% (+5.4%, +6.2%), about +0.6% of wall at its 11% share. A rework that hoists the check out
  of the per-group loop is a new head and needs a new registration.
- **text:** #770's text cells at 1 to 6 threads were all above 1 (1.010 to 1.019), none beyond
  noise. Text is guarded at every count.
- **Layout:** the split's code is not #770's (above), so its layout differs from both #770's
  and master's.

## What is compared

- **Head:** perf/block-symbols-tiled-zero-fill (#785) at `39aaf55bd2c4ba63a4021f0d600c38423e96a7e0`, arm `mibz2` (`BzReader::new` at one
  thread, `BzReader::parallel` above).
- **Base:** `ff0fd5c783f7fbc9a1618accb0c3373d1630a98e`, master as it stood when the split was cut (its `src`, `Cargo.toml` and
  `Cargo.lock` are those of #770's base, `6f7b2ac4`), arm `mibz2@build-base`. It is an ancestor
  of HEAD; the recipe checks it.
- **The contrast, checked by the recipe:** the builds' `src`, `examples`, `tests`, `Cargo.toml`,
  `Cargo.lock`, `build.rs` and `rust-toolchain.toml` may differ only in `src/block.rs` and
  `tests/block_symbols.rs`, or the run stops.
- **Should master move before the run,** the run still measures the PR against this base; the
  Coordinator weighs the later change at the merge.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run.
- **The corpora:**
  - **Wikidata (judged):** `corpora/wikidata.bz2` (431 blocks, 209.7 MB decoded), checked by
    `bench/wikidata.json`'s SHA-256, four copies end to end (`wikidata-x4`, 838,800,784 bytes
    out).
  - **text (the loss guard):** `ab gen DIR 16`'s `text.bz2` (about 48 MB, level 9), made afresh
    by the head's harness alone, its decoded bytes counted by Python's `bz2` (libbzip2); the
    verdict checks every text output against that count.
- **The cells:** `ab run --arms mibz2@build-base,mibz2` on both corpora at 1, 2, 4, 6, 8 and 10
  threads, the corpora, counts and arms interleaved within each round. Above one thread
  unpinned on every CPU (no `--spread`, no spinner). At one thread on ab's one-CPU pin, for both
  arms alike: that is what ab does at one thread, and what #770's run measured; the verdict
  requires it.
- **Why these counts:** on master the serial scanner runs at every count (the default decoder's
  dispatch below and from 8 threads is the same on both builds), so the changed reader runs in
  every cell. 1 is the one-thread decoder's reader; 2 to 10 the parallel decoder's serial
  scanner, the decoder's bound from 8 threads on the i9 (the plateau).
- **The preflight, untimed:** one round of every cell with a 0.25 s settle, before the timed
  run (`preflight.jsonl`). `bench/block_symbols_tiled.py --preflight` checks it as the verdict
  checks a run: the machine (lscpu's model, and `ab env`'s topology by
  `two_arm.topology(env, 10, 20)`), every record present on both corpora, successful, unpinned
  above one thread, and every output of the registered size with one hash a corpus. It also
  checks **the dispatch** by `ab`'s thread census (`observed_threads`): every record above one
  thread, on either arm, has master's serial scanner (`mibz2-scan`) and no helper scanner
  (`mibz2-scan-1`). A failure stops the recipe before the timed run and is not a verdict;
  `just bench-block-symbols-tiled-zero-fill 1` (DRY) runs the preflight alone. No step is taken
  from it.
- **Rounds:** 16, each running both arms at each count on each corpus, interleaved, the arms'
  order rotated, with a **2 s settle** before each run (bench-parscan-stream's method, and #318's
  idle-gap effect).
- **Cost:** about 25 minutes of workstation time: 6 counts x 2 arms x 2 corpora x 17 rounds
  (408 runs, each its settle and a decode of 0.06 to 1.4 s, about 16 minutes), `ab gen` about a
  minute, and two release builds.

## The rule

A count's **step** is the 16 rounds' paired wall ratios, head over base: their geometric mean
and its 95% interval (Student's t on the logs, 15 degrees of freedom). FASTER if the interval
lies below 1, SLOWER if above, INCONCLUSIVE otherwise. The steps are taken per corpus and per
count, each cell on its own; none is pooled. The verdict line is named TILED ZERO-FILL.

- **ADOPTS:** all four hold:
  - on Wikidata, the step at 6 threads is FASTER;
  - on Wikidata, none of 1, 2, 4, 8 or 10 threads is SLOWER;
  - **the text loss guard:** on `text`, none of 1, 2, 4, 6, 8 or 10 threads is SLOWER;
  - at every count, the median peak RSS over the rounds, head over base, is at most 1.10, on
    Wikidata and on text.
- **HELD ON MEMORY:** as ADOPTS, but with the median peak-RSS ratio above 1.10 at any count on
  either corpus. The verdict line names every cell over. It is not ADOPTS.
- **NOT ADOPTED:** any other outcome: 6 not FASTER on Wikidata, or Wikidata SLOWER at any of 1,
  2, 4, 8 or 10, or text SLOWER at any count. A loss outranks the memory gate. A text gain
  decides nothing.
- No count is only reported, and no transfer is computed: there is no claimed figure from
  another machine to carry over.
- **The false-refusal chance:** with no effect at the eleven loss cells, each refuses with
  probability about 2.5%, so up to about 24% together (fewer, since the cells are correlated).
  A true 0.977 at 6, with #770's interval half-width of about 0.6%, is FASTER with high power.
- **NOT ADOPTED is a likely outcome,** and it would not be a false refusal: rule 2.2 is a bar.
  Resampling #770's rounds at face value puts the chance that some loss cell is SLOWER at about
  0.87 (wikidata at 1 thread alone 0.43, text at any count 0.77); shrunk toward no effect, still
  plausibly 0.4 to 0.7. The 24% above is the chance under no effect at all.
- **VOID,** with no verdict, if any of these holds: the machine is not the i9's profile; any run
  is unsuccessful or has a non-finite or non-positive wall, CPU or peak RSS; any run above one
  thread is pinned, any one-thread run is not on a single CPU, or any run is beside a spinner;
  any record is duplicated, unexpected or missing at any of the six counts on either corpus;
  any record's corpus is neither `wikidata-x4` nor `text`; any output is not its corpus's
  registered size; or a corpus's outputs' hashes are not all one.
- **One run, one verdict.** The verdict reads `main.jsonl` alone. A verdict is never re-run to
  change it.

## Why this run (PERFORMANCE-POLICY 4a)

The run costs about 25 minutes, under 4a's 30-minute line; the three answers are given anyway.
- **The decision on each outcome:** above.
- **The causal relevance of each condition:**
  - **6 (judged) and 4:** the parallel decoder's serial scanner, below the plateau, where #770
    measured 0.977 and 0.978 with nothing else of #770 running.
  - **8 and 10:** the serial scanner is the decoder's bound there on the i9, so a faster reader
    could show; #770's front end hid these cells, so they are unmeasured for this change.
  - **1:** the one-thread decoder's reader, where the branch a group is the change's main cost
    and the fill saved is least.
  - **2:** the serial scanner with one worker.
  - **text at every count:** the second primary corpus (PERFORMANCE-POLICY rule 2.2: no primary
    corpus slower beyond noise at any count), whose cells in #770 all sat above 1.
- **Why cheaper evidence won't do:** CI's runner has 4 CPUs, and on #770 its paired bench did
  not resolve this 2% effect at 1, 2 or 4 threads; the effect was seen on the i9 alone. The
  static path settles where the code runs (every count), not what it costs. #770's own cells
  cannot adopt the split: they measured different code, and were not registered to decide it.

## Limits

- **Wikidata and text alone:** no records or random corpus on the i9. CI's bench covers every
  corpus at 1, 2 and 4 threads.
- **What an ADOPTS shows:** the split's wall-time gain at 6 threads on the i9 and no loss at the
  other cells, not its cause: no cache counters, no pool census.
- **Not covered:** 12 to 20 threads, where the decoder runs the same serial scanner and seven
  workers as at 10; another machine.

## Result

*Added after the run; nothing above this section changes once it is frozen.*
