# 2026-11-16 · Pathfinder · Workers invert with lazy letters from 8 threads

Assignment: QUEUED 6903395401 on #145, whose adoption rule is frozen there before any data
of this build; the memory dose is QUEUED 6903426694 (2). This entry holds what changed, what
did not, the evidence, the i9 recipe, and what was tried on the way and left out.
**No timing was taken here.** The i9 run at this PR's head decides the two constants.

## Why

The memory-lever run on the i9-10900K (LOG 6903318530, `memory-lever/run-20261115T233002Z`
at `caad2bd9`) put one block a thread with lazy letters against the shipped decoder:

| threads | 4 | 6 | 8 | 9 | 10 | 12 |
|---|--:|--:|--:|--:|--:|--:|
| lazy ÷ shipped, wall | 1.086 | 1.033 | 0.958 | 0.892 | 0.949 | 1.018 |
| CPU | 1.106 | 1.059 | 0.959 | 0.885 | 1.106 | 1.546 |

Lazy letters cost a quarter more instructions in the kernel and hold 38% less memory beside
the output (`2026-11-15-memory-lever.md`). Few workers do not contend for the cache, so
they lose. Many do, and they win: at 8 and 9 threads against workers with a slot for every
letter, and at 10 against three teams of three. Louis accepted the CPU at 10 threads
(6903395401).

## What changes

- **`LAZY_FROM_THREADS = 8`** (`src/parallel.rs`, beside `WORKERS_BELOW_THREADS`). A plan
  of that many workers or more inverts with `bwt::LazyCells` (`Plan::lazy`); fewer keep
  `bwt::Cells`. Teams are untouched: a team shares one block's table.
- **The lazy kernel is no longer behind a feature.** It moves from `src/cells_lever.rs`
  (`memory-lever`) to `src/cells_lazy.rs`, always compiled. The staged kernel stays behind
  the feature, where it was.
- **`StreamDecoder::workers(threads, slots, kernels, lazy)`** gives either kernel at any
  thread count, for the bench arms `mibz2-workers-eager` and `mibz2-workers-lazy`.
  `mibz2-workers-w` takes the plan's choice, as the default does.
- **`just bench-lazy-crossover`**, the i9 recipe (below).

**What does not change:**
- **The lazy kernel's code.** The text of `slots_lazy`, `induce_lazy`, their census and
  their constants is master's, character for character, but for one `cfg` line on the
  staged arm of a `match`. It is the code #447 merged, the Guardian reviewed (D110) and the
  i9 measured. The staged kernel's text is master's too.
- **Below 8 threads:** the same kernel, the same plan. CI's bench runs at 1, 2 and 4.
- **Output and errors:** the same bytes and the same error at every thread count, whichever
  kernel a worker runs.
- **The team threshold** (`WORKERS_BELOW_THREADS = 10`). The rule in 6903395401 moves it
  only on the i9 run.

## Where it takes effect

| machine | threads | before | now |
|---|---|---|---|
| the i9-10900K (10 cores) | 8, 9 | workers, a slot for every letter | workers, lazy letters |
| the i9-10900K | 10 and more | teams | teams |
| fewer than 6 physical cores, or cores not known | 8 and more | workers, a slot for every letter | workers, lazy letters |
| 6 to 9 physical cores | more threads than cores | teams of 2 | teams of 2 |
| any | fewer than 8 | as before | as before |

The third row is not measured: 8 or more threads on a machine of fewer than 6 cores share
cores, where memory per core is scarcer still. The i9 recipe's counts above 10 are its
nearest reading.

## Safety (RULES B5)

**No unsafe code is added or changed.** The lazy kernel's unsafe is #447's, with its safety
argument in `2026-11-15-memory-lever.md` and the Guardian's review in D110. What is new is
that a default decoder reaches it, from 8 workers. So its evidence now runs in the default
build:

- **Tests** (123 in the library; 124 with `memory-lever`): the lazy kernel against the
  original induction state by state, on its boundary columns with reuse, through its own
  route against the plain walk, and its pool's bounds. They ran only with the feature
  before.
- **Miri:** the three Miri-listed lazy tests are in the default Miri step's list now
  (`miri.yml`), not the feature's. No local Miri run was taken on this branch: the code
  under them is unchanged, and CI's run is the one the gate reads.
- **Fuzz:** the Cells mode's differential check holds `LazyCells` and `ForcedLazyCells` to
  `Cells`' and `ForcedCells`' output, errors and routes in every build
  (`tests/common/cells.rs`), not only with the feature.

## Evidence for what is new

- **`lazy_letters_from_their_thread_count`** (`src/parallel.rs`): `Plan::lazy` is true
  exactly from 8 workers; the decoder a plan builds, and the default with the plan forced,
  run the lazy kernel exactly then; teams never; `workers` gives either at any count. A
  decoder's kernel is read from the type its workers were launched with, not from a flag.
- **`tests/lazy_workers.rs`:** six streams through `workers` with each kernel and through
  the plan's choice, at 1, 4, 8 and 12 threads, against workers on the Segmented kernel:
  one output and one error. The streams: one whose every block the lazy kernel admits, one
  whose every block it gives up on inside the induction (the budget), the admission
  witnesses, a pattern, a block with a wrong CRC among good ones, and a stream cut short.
- **Wikidata:** the same bytes at 1, 4, 8 and 12 threads, and from the three workers arms
  at 8.
- **Mutation** (`2026-11-16-lazy-workers/mutants.tsv`, script `mutate.py.txt`; debug
  profile): 9 mutants on the choice of kernel, in the plan, in the decoder and in the
  kernel's own dispatch; all killed. The lazy kernel's own table is
  `2026-11-15-memory-lever/mutants.tsv` (its code is unchanged).

**Instructions** are master's, counted for #447 (`2026-11-15-memory-lever/counts.txt`, per
letter): first stage 67.5 against 76.3, induction 181.0 against 115.5, writer 49.5 against
53.9, whole decoder +12.2%.

## The i9 recipe

`just bench-lazy-crossover` (`.just/bench.just`), at 4, 6, 7, 8, 9, 10, 12, 14, 16 and 20
threads, 10 paired rounds, on Wikidata:

| arm | what it is |
|---|---|
| `mibz2` | the shipped plan at this commit |
| `mibz2-workers-eager` | one block a thread, a slot for every letter, at every count |
| `mibz2-workers-lazy` | one block a thread, lazy letters, at every count |
| `mibz2-team2`, `mibz2-team3` | the plain teams |

It writes the four summaries the rule reads (`--ref mibz2`, `-workers-eager`, `-team2`,
`-team3`). Then, reported and not a gate (`bench/lazy_crossover.py`):

- **The memory dose:** each workers arm's CPU time at 10 threads over its CPU time at 1,
  round by round. Workers share nothing but the machine, so it is what a block costs among
  nine others over what it costs alone. (#229 had 1.583 for the shipped workers.)
- **Peak resident memory** of every arm at every count.

**Where an arm runs the shipped plan** (the rule's sanity case): `-eager` at 4, 6 and 7;
`-lazy` at 8 and 9.

## Tried on the way, and left out

The first build was stacked on #449 (rules and bottom slots in the output's spare
capacity), as asked. #449 then failed in CI: Wikidata 1.7%, 4.8% and 5.2% slower at 1, 2
and 4 threads, peak memory 30% and 49% up at 2 and 4, and the occupancy ledger over its
bound. So this PR is on master and has none of it. The stacked work is kept on the branch
`perf/lazy-workers-on-449`, unmerged.

- **Lazy letters with their rules in the tail too** was built there, verified (Miri, 41
  mutants) and counted. It is not a candidate while #449's premise is in doubt: the
  output buffers circulate, four a thread, so "dead" memory is also cold memory.
- **A finding about the instrument.** Eight builds of the same lazy loop, differing only in
  where a rule is written and whether the tail's offsets carry a constant, counted 176,
  180, 190, 194, 197, 204, 211 and 214 instructions a letter in the induction. A build
  with *fewer* live values counted more. So callgrind's count of this loop swings by 10%
  and more with code layout alone, and is not a gate for it. The Coordinator had already
  ruled that memory cuts are judged by the clock at 8 to 10 threads (6903426694).

## Reproduce

```
cargo test --release --lib -- cells:: lazy_letters_from_their_thread_count
cargo test --release --test lazy_workers --test cells
python3 docs/journal/2026-11-16-lazy-workers/mutate.py.txt .     # leaves the tree as it was
just bench-lazy-crossover                                         # on the workstation
```
