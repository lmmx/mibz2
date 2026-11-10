# The records' path on the teams: #324 carried into the team path

10 November 2026. The Pathfinder (Architect). The first item of the order of work agreed
with the Coordinator: #324's run-record front and backward table, which the one-core path
has had since `47149bff`, on every team path. A patch against master `47149bff`, in
[`team-runs.diff.txt`](2026-11-10-team-runs-port/team-runs.diff.txt) beside this note.
Local timing here is indicative only (RULES C5).

## 1. Why

The workstation's A/B of #324 (bundle of 10 November, `ev-324`) gave 451 against 411 MB/s
at 1 thread and 1,119 against 1,036 at 4, and no gain at 10 (1,497 against 1,514). At 10
threads the i9 runs teams, and a team staged every block by its column: `decode_block`
kept the records for kernels that are not cooperative, and the pipelined worker and the
service core never looked. So the release's headline figure had none of #324.

## 2. What changes

A block `runs::takes_runs` admits now goes by its records on all three team paths.

| stage | by the column (as before) | by the records (new on the teams) |
|---|---|---|
| front, one thread | `mtf::mtf_checkpoints`: the column, counts, a checkpoint each 16,384 rows | `runs::front` (#324's, unedited), then `runs::cuts`: one pass over the records |
| build, the team | `coop::FusedBuild`: a scatter of each range's rows | `coop::RunsBuild`: `runs::fill`'s table, written in order by shares |
| walk, the team | `coop::RoomWalk::new`, from Ψ's first row | `RoomWalk::new_back`, from the primary index: the same shares |
| join, one thread | `SegScratch::assemble` | `assemble_reversed` (#324's) |

- **`runs::cuts`** reads every record once. It is the tiling's whole check (`fill`'s:
  the counts sum to n, no record is empty, each byte's rows are its count). It also cuts
  the records into 12 pieces of about equal work and keeps, at each cut, the row and every
  byte's next LF value. 12 divides into the shares of 1, 2, 3, 4, 6 or 12 threads, so the
  front needs no knowledge of the team's size.
- **`runs::fill_share`** writes `fill`'s entries for the records between two cuts, from
  the first cut's state. It is the one new `unsafe` routine (scalar and AVX2).
- **`coop::RunsBuild`** is `FusedBuild`'s twin: checks before the run, each share claimed
  once, an end check a share, the table's length set only if every share passed.
- **`coop::Source`** names what a team inverts a block from, its column or its records.
  `invert_pipelined_from` and `invert_team_from` take either; the run, its waits, its
  panic handling and the walk's code are the same.
- **The callers:** `bwt::Cooperative` (plain teams) reads runs; `parallel::front_stage`
  stages by records where `takes_runs` admits, for the pipelined worker and the service
  core alike; the service's finisher joins a backward walk's segments in reverse.

Not edited: the one-core path, `runs::front`, `runs::fill`, the walk's `traverse`, the
plan policy, and `takes_runs` (the admission patch of this morning is separate and does not
overlap these lines).

## 3. The safety argument (RULES B5)

**The new unsafe code is `fill_share`'s stores through the shared table.** Its contract:
`out` is valid for `to.row` slots, and no other thread touches slots `from.row..to.row`
during the call.

- **Every store is inside `from.row..to.row`,** whatever the records and the cuts hold.
  Before any store the function checks that the records' range exists, that the rows rise
  from `from` to `to`, and that no byte's value falls. A record is refused before it is
  written if it is empty, or longer than the rows left before `to.row`. The scalar kernel
  then stores exactly the record's slots.
- **The AVX2 kernel's 8-slot stores** run up to 7 slots past a record's end. A record
  takes them only if 7 rows or more of the share follow its end; otherwise it is stored a
  slot at a time. So no store passes `to.row`. The slots a store wrote past its own record
  are the same share's next records', which write them again. (An empty record must be
  refused for this to hold: its one store would have no record under it.)
- **`RunsBuild` gives each share rows of its own.** `runs::cuts_tile` is checked before
  any share runs, on whatever cuts it is given: they start at row 0 and end at row n, and
  nothing falls from one cut to the next. Share j takes pieces `12j/k` to `12(j+1)/k`, so
  two shares' rows are disjoint; a share is filled only by the caller that claimed its bit.
  The table is reserved for n slots and held, unmoved, until `finish`.
- **`set_len(n)` only after every share's end check.** A share passes only if its rows
  ended exactly at its cut. The shares' rows tile [0, n), so every slot was written.
- **The table is right, not only initialised,** when every share also ended at its cut's
  values for all 256 bytes: the first share starts from the buckets' starts, so each share
  starts from the state the records before it leave. Every value is then below n, which
  the walk requires.
- **What a failed share leaves** is unspecified inside its own rows (an 8-slot store's
  values past a record among them). The length is never set, so nothing reads them.

**The backward walk on a team adds no unsafe code.** `RoomWalk::new_back` differs from
`new` in the first row alone (the primary index, checked below n; n at most 2^23, as
`bwt::room_segments` asserts). The shares, the flags and the hand-offs are `new`'s.

**Proof scope.** The argument covers `fill_share`, `RunsBuild` and their use by the three
callers. It relies on `coop::Pool::run`'s ordering (every thread's writes visible to the
caller on return), as `FusedBuild` does, and on #324's argument for `front`, `fill` and
`assemble_reversed`, which are unchanged.

## 4. The evidence

Port at local commit `4022b6b1` on master `47149bff`. Everything below ran in this
container.

| obligation | what was run | result |
|---|---|---|
| differential | `fill_share` by 1 to 13 shares against `fill`, both kernels, on synthetic blocks, and each piece alone into a guarded table; `build_runs` and `walk_room_back` on 1 to 4 threads against `runs::fill` and `bwt::walk`; the records through `invert_pipelined_from` (both waits) and `invert_team_from`; `front_stage` on both paths | pass, debug and release |
| whole suite | `cargo test`, `cargo test --release`, `cargo test --features walk-stats --lib coop::`, `cargo test --example ab` | exit 0, each |
| lint | `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`, and with `trace,wait-stats,walk-stats` | clean |
| Miri | `cargo miri test --lib -- miri_runs miri_build_runs`: 10 tests scalar; 11 with AVX2, BMI1, BMI2, LZCNT and POPCNT enabled | pass, both; complete runs, no timeout |
| fuzz | `examples/fuzz 30 1`: classic, pipeline, service | exit 0, each |
| mutation | a hand pass, one edit a mutant, 57 mutants over every new check | 56 killed, 1 equivalent (§4.1) |
| review | by an agent other than the implementer | **not done: the Coordinator's to arrange** |

- **Miri's cost to CI:** the scalar pass took 5.6 minutes here against 3.0 for master's four
  `miri_runs` tests, the two run side by side.
- **Miri's limits:** it ran the tests named, on small blocks. It does not run the decoders'
  integration tests, and its AVX2 pass is not CI's (CI runs the scalar pass alone).
- **The existing integration tests** (`tests/pipelined.rs`, `tests/service.rs`) decode
  `text`, `runs`, `pattern` and `one byte` through the teams and compare the bytes with the
  default decoder's. `takes_runs` admits those corpora, so they now run this path.

### 4.1 The mutation pass

[`mutants.tsv`](2026-11-10-team-runs-port/mutants.tsv), made by
[`mutate.py`](2026-11-10-team-runs-port/mutate.py.txt): debug builds, as CI's `cargo test`.

- **51 of 57 were killed on the first pass.**
- **Five survivors were gaps in my tests, not in the code,** and each is now killed by a
  test added or corrected for it: the cuts' bound on the records' number; `cuts_tile`'s
  bound on the rows; the rows' bound in each `fill_share` kernel; and the column path's
  `rows`.
- **One is equivalent:** the AVX2 kernel made to store every record a slot at a time
  writes the same entries. Only its speed differs.
- **Two checks were deleted rather than tested.** Listing the mutants showed each was
  implied by another: the rows' sum in `cuts` (each byte's rows being its count implies it)
  and a minimum number of cuts in `cuts_tile` (the first and last cuts' checks imply it).
- **One claim in my own contract was wrong, and a test of it failed.** I had written that
  `fill_share` never writes a value past its byte's bound. The AVX2 kernel's 8-slot stores
  do, in slots the share then rewrites or, in a share that fails, never exposes. The
  contract now says what holds (§3, last point).

## 5. Measured here (indicative only)

A 4-core cloud Xeon, no SMT, 8 rounds, arms interleaved, each arm against the same arm
built at master. Ratios are wall time, new ÷ master, with the harness's 95% interval. Full
tables: [`ab-summaries.txt`](2026-11-10-team-runs-port/ab-summaries.txt). Both patches of
this note are applied (§6 explains the second).

| corpus | plain teams of 2: one team (3 threads asked), two teams (4) | one pipelined team of 3 (park, small): 3 and 4 threads asked | service core, one team (in process, medians) |
|---|---|---|---|
| `wikidata` | 0.82 [0.80, 0.84], 0.88 [0.85, 0.91] | 0.93 [0.89, 0.98], 0.92 [0.89, 0.94] | about 0.86 |
| `text` | 0.75 [0.67, 0.83], 0.84 [0.79, 0.90] | 0.69 [0.59, 0.80], 0.73 [0.69, 0.77] | 0.74 to 0.83 |
| `records` | 0.96 [0.91, 1.02] | 1.02 [0.91, 1.14] | about 1.0 |
| `random` | 0.98 [0.93, 1.03] | 0.96 [0.87, 1.07] | not run |
| `runs` | 1.06 [1.01, 1.11] | 0.99 [0.92, 1.06] | not run |
| `pattern`, `onebyte` | 1.00, 1.08 [0.92, 1.28] | 0.98, 1.00 | not run |

- **The bytes are the same in every cell.** Peak RSS is level (within 1 MB).
- **`runs` on plain teams, +5.5%,** is a file of one block that decodes in 0.03 s.
- **The default decoder on this machine is the one-core path's workers,** which the port
  does not edit. It is level: 12 cells of `wikidata`, `text`, `records` and `random` at 1,
  2 and 4 threads, 11 intervals holding 1.0 and one at 0.94 [0.91, 0.97].
- **What this cannot show:** the i9's plans. The service core at 2 × 3 + 2 and three
  pipelined teams of three need ten cores. One team on four cores is a different regime.

## 6. Found on the way: the teams' move-to-front has a 2× cliff, by stack depth

**It nearly sank the port's measurement, and it is in master.**

- **What I saw.** With the port alone, `records` on plain teams was 1.40× master's wall
  time [1.36, 1.44] in the bench harness's binary, and level in a second binary built from
  the same source ([`ab-records-three-builds.txt`](2026-11-10-team-runs-port/ab-records-three-builds.txt)).
  The port does not touch a `records` block's path: 21 of its 22 blocks keep the column.
- **Not more work.** Callgrind counted 2.913 and 2.926 billion instructions in the two
  builds.
- **Where the time went.** The teams' lead threads: 0.20 s of CPU each against 0.12. Stack
  samples (gdb, 28 a build) put the difference in libc's `memmove`, which
  `mtf_checkpoints_avx2` calls to shift its list for every symbol past the 32nd place.
- **The mechanism.** That kernel keeps its 256-byte list, with slack 320 bytes, in its own
  stack frame, aligned to 64. Where the frame puts the list across a 4 KiB page boundary,
  the shifts' unaligned loads and stores cross it.
- **Measured directly.** The kernel timed on one block as the calling stack deepens in
  80-byte steps through 4 KiB ([`stacksweep.rs`](2026-11-10-team-runs-port/stacksweep.rs.txt),
  least of three passes):

  | block | master, outside the band | master, in the band | the band (the caller's stack, modulo 4 KiB) | with the fix, all 80 depths |
  |---|--:|--:|---|--:|
  | `records` | 9.1–10.9 ns a row | 12.4, 15.4, 18.6 | 3 of 80 depths, 0x5f0–0x690 | 8.8–10.5 |
  | `random` | 7.9–9.3 | 9.6, 10.6, 16.4, 17.3 | 4 of 80 depths, 0x630–0x690 | 7.9–9.2 |

  The one-core path's `mtf_avx2` is flat in both builds: it reads its rows with aligned
  32-byte loads, which cannot cross a page.
- **Why it is a build's property.** A worker thread's stack starts page-aligned, so the
  depth at which the kernel runs is fixed by the build's frames. A build has the cost on
  every block of a symbol a row, or on none. About one layout in 13 has it (320 of 4,096
  bytes). Any change to the binary redraws it. Mine drew it; master's happens not to.
- **My first explanation was wrong.** I took it for the 4 KiB aliasing of `counts` against
  the list, which I had analysed for #324 this morning, and a patch moving the counts into
  the frame did restore the speed. A sweep of the counts' address showed no band. That
  patch had only redrawn the layout. I dropped it.

**The fix, [`mtf-list-in-one-page.diff.txt`](2026-11-10-team-runs-port/mtf-list-in-one-page.diff.txt):**
the list's alignment goes from 64 to 512. Its 320 bytes fill one 512-byte slot and a page is
eight slots, so it lies in one page wherever the frame falls. A constant assertion holds
that property at compile time. No pointer arithmetic changes. Whole suite, debug and
release: exit 0.

**With the fix under the port,** `records` on plain teams is 1.05 [1.00, 1.11] of master's
in one build and 0.96 [0.91, 1.02] in the final one (§5), where the port alone was at 1.40.

**What it may explain elsewhere, unverified:** unexplained moves of `random` and `records`
at 2 threads and more between builds whose decoder source did not change. I have not gone
back through the bench's history to test that.

## 7. What it costs

- **One more pass over the records on the front's thread** (`runs::cuts`): a load, an add
  and a store a record, about 64,000 records in a Wikidata block and 400,000 in a text
  block. It is inside the timings of §5.
- **A front set holds both kinds of buffer** once it has staged both kinds of block: up to
  about 1 MB more a front set on a stream that mixes them. On Wikidata and text every block
  goes by records, and the column's buffers are never allocated.
- **Code:** about 850 lines of decoder, comments included, and 850 of tests.

## 8. The decision this serves, and what would falsify it

- **Decision:** merge under PERFORMANCE-POLICY.md, or close.
- **CI cannot decide it.** CI's runners have 2 cores, where the policy plans workers, so
  CI's bench never runs a team. It can only show that the default path is level.
- **The workstation can:** the standing A/B template of two commits on the release-scaling
  harness (RULES C5), master against master with both patches, at the grid's 6 to 20
  threads, on `wikidata`, `text`, `records` and `random`.
- **The rule, written before that data:** the port stands if `wikidata` or `text` is faster
  beyond noise at 10 threads and neither is slower beyond noise at any thread count of 6
  or more. It closes, with its numbers, if neither is faster at 10 threads. A loss on
  `records` or `random` is weighed under the policy's rule 3.
- **If `records` or `random` moves by tens of percent** in either direction at team thread
  counts, suspect a layout of the kind in §6 before the port, and run the stack sweep on
  that build.

## 9. To apply

Two files, both against master `47149bff85a13256f36982399d900672ab607254`. They touch
different files and apply in either order, and over the admission patch.

| file | SHA-256 | touches |
|---|---|---|
| `2026-11-10-team-runs-port/mtf-list-in-one-page.diff.txt` | `789aad9cac21be2323f6a6698370d556612a58f0e0868213b445bfe0fc721c4e` | `src/mtf.rs` |
| `2026-11-10-team-runs-port/team-runs.diff.txt` | `3ab375a7bcdee3ca3edca9813100afa90cb30b016dbb9ee6bcc278d3efa22229` | `src/bwt.rs`, `src/coop.rs`, `src/lib.rs`, `src/parallel.rs`, `src/runs.rs`, `src/runs/tests.rs` |

**Against today's master too.** Master moved to `8ed2fbdf` while this was measured (0.2.8,
and the admission patch as #353). Both files apply to it unchanged. With both applied there:
`cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` clean, and
`cargo test --release` exit 0. Everything else in this note was run on `47149bff`; with
both applied there, the whole suite (debug and release), the lint and the three fuzzers
also exit 0.

**I recommend the first on its own merits and first in order:** it removes a cliff from
master, and without it the port's measurement is at the mercy of the draw.

**Not done:** the review B5 requires; any run on the workstation; CI on either patch.
