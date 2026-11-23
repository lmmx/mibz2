# bench-open-route-adaptive: registration

*23 November 2026. A draft for review, written before any data, on `bench/open-route-adaptive`.
The recipe is `just bench-open-route-adaptive` (`.just/bench.just`). The verdict is
`bench/open_route.py`'s, tested by `bench/test_open_route.py`: a wrapper around
`bench/parscan_verdict.py`, copied unchanged from bench-parscan-stream's branch (#771 at
`584d0c2e`), with that registration's corrected rule (REVIEW 7088432559, LOG 7088432851) and a
verdict name. The recipe is bench-parscan-stream's with its pins, its contrast and its
preflight's activation census changed (below). It is frozen once the Coordinator accepts its
review; until then nothing here is final and no run is queued.*

*If CI's paired bench shows piece A (#781) or piece B (#782) alone beating this stack, the
registration is re-pinned to that piece before any i9 data (the Coordinator's instruction).*

**The decision it settles.** Whether perf/open-route-adaptive (#783) merges: the Open kernel's
route by block size, as it runs in production's lazy workers on the i9-10900K. It is an
adoption registration (RULES C6): CI's 4-CPU runner measures 1, 2 and 4 threads only, and the
change's threshold was chosen on a machine with 8 times the i9's L2 (below).

- **ADOPTS:** the PR merges and ships in the next release; DELIVERED.md gets its line.
- **HELD ON MEMORY:** the Coordinator weighs the growth under PERFORMANCE-POLICY rule 4, from
  this run's figures alone.
- **NOT ADOPTED:** #783 does not merge. #781 (A) and #782 (B) are not judged by this verdict:
  each has its own CI bench, and an i9 run of either is its own registration. A Wikidata loss
  would be read as evidence against the 98,304 line on the i9's 256 KiB L2 (B's part), not as
  a reason to move the line in a re-run.
- **VOID:** repeated once, with the reason logged.

## The hypothesis

**What changed.** The PR's head, `7f89772f`, is master's `6f7b2ac4` with two commits on the
Open kernel (`src/open.rs`, `src/open/{checks,endpoint,outq}.rs`; safe Rust, no allocator,
admission or worker change):

- **A (mark-free two-ended wiring), on blocks of more than 98,304 retained rows:** master's
  two-ended walk (`wire_xor`, `emit_bidirectional`), with the mark search (`v == mark`, a
  compare and a conditional store per rank) left out of every byte's queue but the marked
  row's: a queue holds only its own byte's ranks, so no other can hold the mark.
- **B (one-ended small blocks), on blocks of up to 98,304 retained rows** (master's line is
  65,536): the one-ended walk (`wire`, `emit`, master's own), with the induced construction's
  bands coalesced: a byte's consecutive bands with contiguous destinations and the same code
  are one band, and a repeated node word reuses its code without a dictionary lookup.

**Where the code runs** (a routing census on the bench's Wikidata corpus, `corpora/wikidata.bz2`,
431 blocks, with the head's kernel instrumented in a scratch build, not in the PR; output
identical to master's decoder):

| retained rows m | blocks | master | head |
|---|---:|---|---|
| m ≤ 65,536 | 1 | one-ended | one-ended, bands coalesced (B) |
| 65,537 to 98,304 | 83 | two-ended | **one-ended, bands coalesced (B)** |
| m > 98,304 | 149 | two-ended | two-ended, mark-free wiring (A) |
| not the Open kernel | 198 | LazyCells | LazyCells (unchanged) |

So 233 of 431 Wikidata blocks run changed code: 84 on B's path, 83 of them moved from the
two-ended to the one-ended walk, and 149 on A's. On `text` (the repository's sources repeated
to 48 MB, an approximation of `ab gen`'s text made for the census), **no block takes the Open
kernel**: the head's code is not on text's path, and text measures layout and the shared
binary alone.

**What it predicts.** The source's claims, as claims (Neon's foundational contraction programme,
`research/neon-teamwork-paradigm`, `docs/journal/2026-11-22-foundational-contraction-programme.md`
§1 Breakthrough B and §3; `bench/moonshot` scorer v1 on a 2-vCPU Xeon Platinum 8481C with 2 MiB
of L2, against shipped 0.5.12's Open kernel, which is master's): 0.966 [0.933, 1.00001] at 1
worker and 0.988 [0.923, 1.057] at 2, both inconclusive; peak RSS 1.000 / 1.013; 0.9697× on
the 431 Wikidata blocks at 1 thread; the advisory contention proxy 0.982 / 0.966 / 0.980 at 1,
8 and 10 workers. On the i9 the Open kernel's emit is about 5 ns a retained row and is not
DRAM-bound at 1–10 workers (bench-emit-cost, as the handoff quotes it), so A's saving (one
compare and one store a rank in the wiring) is a small share of a block. The prediction is a
small Wikidata gain at 10, or none.

**The risks, named:**
- **The line on the i9's L2.** 98,304 retained rows is 384 KiB of `edges`, chosen where L2 is
  2 MiB. The i9's L2 is 256 KiB, which is master's 65,536 line. The 83 blocks between the two
  lines are walked from one end past L2 on the head: one dependent chain where master runs two.
  The same programme's "Law 4" measured one-ended walks past L2 at +4% to +100% on its own
  machine, and the handoff recommends sweeping the line on the i9 (65,536, 98,304, 131,072).
  This run tests the one line, 98,304; no other is tried after the data.
- **Contention at 8–20 threads:** a one-ended walk issues fewer independent misses; with every
  core busy, the i9's shared 20 MiB L3 may hide that or amplify it.
- **Memory:** neither piece adds scratch (coalescing writes fewer bands; A changes no
  allocation). The gate is kept as bench-parscan-stream's.

## What is compared

- **Head:** perf/open-route-adaptive at `7f89772ff382e99155d22ccd030bd30d5056d7db` (#783), arm
  `mibz2` (`BzReader::new` at one thread, `BzReader::parallel` above).
- **Base:** `6f7b2ac4223f26e2252bbd30931609cb3a39a1ec`, the head's merge base, arm
  `mibz2@build-base`. It is an ancestor of the head; the recipe checks it. Master has moved
  since (to `40a19a00`) in docs and the workstation queue alone: its `src`, `Cargo.toml` and
  `Cargo.lock` are this base's.
- **The contrast, checked by the recipe:** the builds' `src`, `examples`, `tests`,
  `Cargo.toml`, `Cargo.lock`, `build.rs` and `rust-toolchain.toml` may differ only in
  `src/open.rs` and `src/open/{checks,endpoint,outq}.rs`, or the run stops.
- **Should master move before the run,** the run still measures the PR against this base; the
  Coordinator weighs the later change at the merge.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run.
- **The corpora:**
  - **Wikidata (judged):** `corpora/wikidata.bz2` (431 blocks, 209.7 MB decoded), checked by
    `bench/wikidata.json`'s SHA-256, four copies end to end (`wikidata-x4`, 838,800,784 bytes
    out).
  - **text (the loss guard):** `ab gen DIR 16`'s `text.bz2`, made afresh in each attempt by the
    head's harness alone, its SHA-256 recorded, and its decoded bytes counted by Python's `bz2`
    (libbzip2), independently of either build; the verdict checks every text output against
    that count.
- **The cells:** `ab run --arms mibz2@build-base,mibz2` on both corpora at 1, 2, 4, 6, 8, 10,
  12, 16 and 20 threads, the corpora, counts and arms interleaved within each round. Above one
  thread unpinned on every CPU (no `--spread`, no spinner); at one thread on ab's one-CPU pin,
  for both arms alike.
- **The preflight, untimed:** one round of every cell with a 0.25 s settle, before the timed
  run (`preflight.jsonl`). `bench/open_route.py --preflight` checks it as the verdict checks a
  run: the machine (lscpu's model, and `ab env`'s topology by `two_arm.topology(env, 10, 20)`:
  10 cores, 20 CPUs, no CPU quota), every record present on both corpora, successful, unpinned
  above one thread, and every output of the registered size with one hash a corpus across both
  arms. A failure stops the recipe before the timed run and is not a verdict;
  `just bench-open-route-adaptive 1` (DRY) runs the preflight alone. No step is taken from it.
  **No activation census:** bench-parscan-stream's checked its front end by thread names; this
  change has no thread of its own (the route is chosen per block inside the workers), so its
  activation is the routing census above and the PR's route tests
  (`both_routes_at_and_around_the_one_ended_line`), not the preflight.
- **Rounds:** 16, each running both arms at each count on each corpus, interleaved, the arms'
  order rotated (16 is a multiple of 2 × 2 arms), with a **2 s settle** before each run (#318's
  idle-gap effect; REVIEW 7084586885).
- **Cost:** about 30–35 minutes of workstation time with the two builds, as
  bench-parscan-stream's (the same cells, rounds and settles).

## The rule

bench-parscan-stream's, as corrected before its data. A count's **step** is the 16 rounds'
paired wall ratios, head over base: their geometric mean and its 95% interval (Student's t on
the logs, 15 degrees of freedom). FASTER if the interval lies below 1, SLOWER if above,
INCONCLUSIVE otherwise. The steps are taken per corpus and per count, each cell on its own; none
is pooled. The verdict line is named OPEN KERNEL ROUTE.

- **ADOPTS:** all four hold:
  - on Wikidata, the step at 10 threads is FASTER;
  - on Wikidata, none of 6, 8, 12, 16 or 20 threads is SLOWER;
  - **THE TEXT LOSS GUARD:** on `text`, none of 6, 8, 10, 12, 16 or 20 threads is SLOWER;
  - at each of 8, 10, 12, 16 and 20 threads, the median peak RSS over the rounds, head over
    base, is at most 1.10, on Wikidata and on text.
- **HELD ON MEMORY:** as ADOPTS, but with the median peak-RSS ratio above 1.10 at any of 8, 10,
  12, 16 or 20 threads on either corpus. The verdict line names every cell over.
- **NOT ADOPTED:** any other outcome. A loss outranks the memory gate.
- 1, 2 and 4 are **reported** on both corpora: their steps, CPU and peak RSS are printed and
  decide nothing here; CI's paired bench gates them under PERFORMANCE-POLICY rule 2.
- **The transfer**, reported and deciding nothing: at 10 threads, ln(step) / ln(0.980), the
  share of the handoff's advisory proxy for 10 workers that the i9 shows (a model's figure, not
  a measurement).
- **VOID,** with no verdict, if any of these holds: the machine is not the i9's profile; any run
  is unsuccessful or has a non-finite or non-positive wall, CPU or peak RSS; any run above one
  thread is pinned, any one-thread run is not on a single CPU, or any run is beside a spinner;
  any record is duplicated, unexpected or missing at any of the nine counts on either corpus;
  any record's corpus is neither `wikidata-x4` nor `text`; any Wikidata output is not
  838,800,784 bytes, or any text output not the recipe's count; or a corpus's outputs' hashes
  are not all one.
- **One run, one verdict.** The verdict reads `main.jsonl` alone. A verdict is never re-run to
  change it.

## Why this run (PERFORMANCE-POLICY 4a)

- **The decision on each outcome:** above.
- **The causal relevance of each condition:**
  - **Wikidata at 10 (judged), 6, 8, 12, 16, 20 (supporting):** 233 of 431 blocks run changed
    code (the census); the 83 that move to the one-ended walk are where the i9's 256 KiB L2
    and its contention can make the line cost, and only the i9 at 6–20 threads has both.
  - **1, 2, 4 (reported):** the same code at counts CI's bench measures and gates.
  - **text (the loss guard at 6–20; memory at 8–20):** no block takes the Open kernel, so a
    text loss would be layout or binary-level cost, which the policy counts as real deployed
    cost (4a's pooled-wall clause). It is kept per cell, as bench-parscan-stream's rule has it,
    rather than pooled, so that the rule is that registration's unchanged.
- **Why cheaper evidence won't do:** CI's runner has 4 CPUs (1, 2 and 4 threads) and a
  different cache hierarchy; the claims come from a 2-vCPU machine with 2 MiB of L2. The line's
  cost on a 256 KiB L2 under 6–20 busy threads is the open question, and only the i9 has it.
  The static path and the census settle where the code runs, not what it costs there.

## Limits

- **Wikidata and text alone:** CI's bench covers every corpus at 1, 2 and 4 threads.
- **What an ADOPTS shows:** the stack's gain at 10 threads on the i9 and no loss at 6–20, not
  which piece gives it; A and B are benched apart on CI (#781, #782).
- **One line:** 98,304 alone. A sweep of the line is its own experiment.

## Result

*Added after the run; nothing above this section changes once it is frozen.*
