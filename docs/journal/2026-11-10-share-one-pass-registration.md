# 2026-11-10: `build_one_pass`'s whole loop in `FusedBuild::share`, registered before the kernel

**Status: registered, not built, not counted.** Committed and pushed before any line of the
kernel. Port 2 of the Pathfinder's residual (`2026-11-10-team-residual.md`, #313 at
`dae77cfe6`, §3 and §6), stacked on port 1 (#335, `efbdb53af`). A count, never a clock.

## Hypothesis

After port 1, `share` still classifies each 16-row window by two `read_u64`, a rotate and a
compare, and a window that is not one run takes 16 one-row steps. One worker's
`build_one_pass` instead classifies a window by one SSE2 compare (`run_joins`). It cuts a
window of at most `RUNS` = 6 runs into its runs, each one cursor step and one `run16` write,
with provisional entries above the cursor.

**The port:** `share`'s scatter becomes `build_one_pass`'s loop, line for line:
- `run_joins` classifies each window;
- `ALL_JOINED` takes `run16`, as port 1 does;
- `few_runs` takes one `run16` per run when `s + 16 ≤ lim[c]`, else the run's rows one at a
  time;
- every other window takes 16 one-row steps;
- the tail and the end check stay as they are.

`lim` is the **share's window** end, not the bucket's.

**No new invariant beyond `build_one_pass`'s own.** A provisional entry lies above its share's
cursor and inside its share's window of bucket c, which no other share writes. That share's
own cursor overwrites it before the end check can pass, exactly as one worker's does inside
the bucket. A refused share leaves its provisional entries in a table that `finish` never
sets.

**Twin, not shared code:** the loop is copied into `coop.rs`. `run_joins`, `few_runs`,
`row_byte`, `ALL_JOINED` and `RUNS` are made `pub(crate)` and otherwise untouched, so
`bwt::build` keeps its source.

## Base and the prediction's derivation (#330's base, with port 1)

**What is counted** (port 1's archived profiles, `2026-11-10-share-run16/cg`; Ir/D,
D = 209,700,196):
- port 1's team: `share` inclusive **6.786**, whole process **35.800**;
- one worker's `bwt::build`: 1,082,059,629 Ir over N = 209,706,557 rows, so **5.160 Ir a
  row**.

**Model:**
- **The cooperative rows** (209,697,662) at one worker's per-row cost: 5.160 Ir/D.
- **Plus the share's own costs:**
  - per-share setup, about 0.01 (466 shares × the 256-entry setup);
  - runs that take the one-row steps because they start within 15 slots of a share's window
    end inside a bucket, one more window end a bucket a block. At most 233 × 256 × 15 runs;
    expected well under 0.07 (#313 §3).

| figure (Ir/D) | port 1 (counted) | predicted |
|---|--:|--:|
| `FusedBuild::share`, inclusive | 6.786 | **about 5.2** (5.16 to 5.25) |
| team whole process | 35.800 | **about 34.2** |

- **Falsifier:** `share`'s inclusive cost **not below 5.5 Ir/D**: more than 0.34 above one
  worker's per-row cost, so the port does not reach `build_one_pass`'s cost.
- **On #334's base (model, not counted):** 30.21 − 2.15 (port 1) − about 1.6 ≈ **26.5**, the
  Pathfinder's figure.

## Count method

- **Binaries:** `cargo build --release --example ab`, rustc 1.99.1, built at:
  - `efbdb53af` (port 1, the control);
  - this branch's kernel commit (port 2).

  The SHA-256 of each is recorded.
- **Corpus:** `wikidata.bz2`, `82f65ace…c2c29915`, all 431 blocks.
- **Profiler:** `valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes`, each arm
  once:
  1. port 1 team: `ab child mibz2-pipe3-both 9`;
  2. port 2 team: the same;
  3. port 2 one worker: `ab child mibz2 1`.
- **Profiles:** written to a directory that only ever holds raw files. They are gzipped
  into the journal's archive **by moving** (`gzip` without `-k`).
- **Every figure is computed from the archive** after it is written.
- **The parser refuses doubled input:** `team_gap_count.load` refuses a prefix with both
  `PREFIX.PID-NN` and `PREFIX.PID-NN.gz`, and any thread number appearing twice.
- **The result lists every profile** with its SHA-256, thread and total.
- **The figures:**
  - the stage table, from `bench/team_gap_count.py report`;
  - `share`'s inclusive Ir: self plus arcs out, summed over threads;
  - per-instruction executions: summed over threads, by address, with objdump of the
    counted binary.
- **No retries.**

## Conditions (each must hold; read from instruction identity, as registered)

1. **Same bytes:** each arm writes 209,700,196 bytes with hash `a8830caee5c511a6`.
2. **Helpers execute `share`:** in both team arms, 466 calls, all on threads with no MTF.
3. **The classifier is `run_joins` (instruction identity):** in port 2's `share`, the
   `pmovmskb` of `run_joins` executes **exactly 13,106,081** times, summed over threads. That
   is port 1's counted window count (`bwt.rs:544`'s `read_u64` executions in `share`). The
   `rol` of the old classifier executes **0** times there. Port 1's control arm: `pmovmskb`
   0 times, `rol` 13,106,081.
4. **The one-run window takes `run16` (instruction identity):** in port 2's `share`, the
   four `movdqu` stores of the `ALL_JOINED` site execute **exactly 10,740,083** times each
   (port 1's count).
5. **The run-by-run windows take `run16` (instruction identity):** the four `movdqu` stores
   of the second inlined `run16` site in port 2's `share` (the `few_runs` site) each execute
   between **3,670,695 and 4,568,175** times. In port 1's `share` this site does not exist.
   - **The bound comes from one worker's archived count** of the same site in `bwt::build`
     (port 1's `new-one` profile: 4,568,175).
   - **The windows are the same:** `CHECKPOINT` = 16,384, so every share starts on a
     16-aligned row and its windows are one worker's.
   - **Why a team cannot exceed it:** a run that one worker writes by the one-row steps
     (near its bucket's end) is near the last share's window end too.
   - **Why it can fall short:** the short blocks' runs leave (at most 460 windows × 6), and
     each share's interior window end adds at most 15 one-row runs a bucket a cooperative
     block (233 × 256 × 15 = 894,720).
6. **Control:** the port 1 arm reproduces port 1's archived figures: `share` inclusive within
   0.01 of 6.786, and whole process within 0.1 of 35.800.
7. **One worker:** the port 2 one-worker arm is within 0.01 of 25.792.

A deviation (an inlining change, a symbol renamed by LTO, a condition's instruction not
found by its mnemonic) is disclosed as a deviation, not fixed silently. If an instruction
named above cannot be identified in the counted binary, that condition is **unmet**.

## Dispatch paths (G3, D083)

| path | changes? | why | evidence |
|---|---|---|---|
| pipelined team helpers (`invert_pipelined` → `FusedBuild::share`) | **yes** | the port | this count, the unit tests |
| ordinary teams (`Cooperative::invert_checkpointed` → `build_fused` → `share` on the pool) | **yes**, the same function | the same port | the unit tests; not counted |
| plain worker `bwt::build` / `build_one_pass` / `run16` / `run_joins` / `few_runs` | **no** (visibility only) | it is the original | condition 7; CI's A/B |
| team solo paths (`parallel::solo`: short blocks, a failed pool) | **no** | they call `bwt::build` | read |
| `build_two_pass` | no | untouched | |

## Tests before the count

- **Differential:** the split build against `bwt::build` and against a verbatim copy of port
  1's `share` (the reference), entry for entry.
  - **Crafted columns:** port 1's set, plus windows of 2 to 7 runs at every offset, with runs
    reaching each share's window end at every distance 0 to 16.
  - **Real columns:** libbzip2's, at k 1 to 4.
- **Kept passing:**
  - port 1's tests;
  - the independent B5 reviewer's three tests (`b5_guard_top_bucket_sentinel`,
    `b5_top_byte_and_largest_block`, `b5_lying_counts_fuzz`).
- **Miri:** a targeted test of the run-by-run windows and their provisional entries at a
  share's window end, under its race detector at k = 2 and 3.
- **Mutants:** at least three of the new logic, each of which the tests must catch.
- **B5:** the safety argument goes in the code and the result. An independent adversarial
  review is a separate agent's. The implementer does not discharge B5.

**Not claimed:** time, adoption, merge. CI's A/B never runs a team.
