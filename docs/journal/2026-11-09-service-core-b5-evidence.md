# The service core: B5 evidence for Guardian D045

9 November 2026. This answers the HOLD in Guardian D045 (comment 6730103867; the decision is
`state/decisions.md` at 74a41357 on `team-state/guardian`). The HOLD stops service-core
timing and production admission until reached-path evidence comes in. The branch is
`safety/service-core-b5`, off 3e201246 (the frozen #258 head, whose `src/` is 9ef3c17d's).
#258's branch and its measured inputs are not touched. Every run below was local, on the
shared 4-core machine, one job at a time, with the 1-minute load at most 3 at each start.
Each test or fuzz binary ran under `ulimit -v 6000000` (Miri under 8000000), and none came
near the cap. No timing was taken.

## What changed, and where

`src/` changes, all small:

1. **`src/parallel.rs`, tests only.** `HubState` gets two `#[cfg(test)]` counters,
   `svc_waits` and `team_waits`. A service worker and a team's first thread each bump
   theirs under the hub's lock, just before `Condvar::wait`. Release builds have neither the
   fields nor the increments. There are also two new tests in `service_tests` (below).
2. **`src/read.rs`, production.** The parallel `BzReader`'s feeder thread holds a guard
   (`CancelOnUnwind`). If the source's `read` panics, the thread unwinds and the guard's
   drop then:
   - records `io::Error::other("the input's reader panicked")` as the read error;
   - cancels the decode.

   The guard does nothing on any other exit: its drop checks `thread::panicking()` once,
   per reader, not per chunk. `BzReader`'s documentation now states this scope.

None of these changes is in a measured input of #258 or of #271.

Outside `src/`:
- `tests/common/mod.rs` gains `drive`, `plain_parallel`, `kind_name`, the service
  candidates (`Serviced`, `service_candidates`, `Drive`) and `multi_block_corpus`.
- `examples/fuzz.rs` gains MODE `service` (and `mixed`, which makes every 8th case a
  service case). It also counts what each case reached.
- `tests/service.rs` gains `a_panicking_source`.

The pipeline-fuzz work for D042 (`safety/pipeline-fuzz`, 2614d832) exists only as a local
branch: it is not pushed, and it is based on master rather than on this research line. Its
pattern is reused here rather than merged:
- the `drive` helper and the plain-parallel reference;
- the multi-block corpus;
- the edits aimed at a later block's start or at the stream's end;
- the MODE argument.

If both branches reach master, `tests/common` will need a small manual merge.

## 1. Reached-path service fuzz and differential

**Candidates** (`service_candidates`, notation `teams x k + services in slots`):
- **svc2's plan at 10 threads:** `2x3 + 2 in 8`. This is `ab`'s `mibz2-svc2`, the plan
  FOR SVC2@10 was measured at. It runs both pushed and through
  `BzReader::parallel_service` (the feeder path).
- **Tight windows:** `1x3 + 1 in 1`, and on the scalar kernels `1x2 + 1 in 1`.
- **Wide window:** `3x3 + 1 in 12`.
- **Two service workers with a team of four:** `1x4 + 2 in 2`.
- **Scalar kernels:** `2x2 + 1 in 3`.

All candidates except the two scalar ones run on the detected (AVX2) kernels.

**Inputs** (`multi_block_corpus`):
- full blocks at levels 3 to 9, all at or above `MIN_COOP` rows, among them ten 300 kB
  blocks;
- small blocks (levels 1 and 2) between full ones;
- several streams, including an empty one;
- crafted streams alternating full and small blocks.

One case in eight is left valid. The rest get one or two edits (bit flips, byte sets, cuts,
inserts, deletes, splices), aimed at a later block's header and tables, at the stream's
end, or anywhere.

**How each case runs:**
- Write and read sizes are drawn from 509 B, 4093 B, 64 KiB, 100,003 B and 1 MiB.
- One case in eight is cancelled after a random number of bytes, between 0 and 3 MB. The
  push form calls `cancel`; the reader form is dropped.

**What each case is compared against:**
- **The plain parallel decoder,** exactly: the same bytes before any error, and the same
  error kind, block and bit. A cancelled case's bytes must be a prefix of its bytes.
- **libbzip2,** for the outcome class.

**Reach** is counted from the decoder's own marks. The fuzz is built with `--features
trace`, and each service case runs at trace level 1. A case counts as reaching a path when
its marks show:
- **pooled build or walk:** a `BuildStart` or `WalkStart` mark on a `mibz2-coop-*` helper
  thread, so a helper took a share;
- **service finishing:** an `RleStart` mark on a `mibz2-svc-*` worker;
- **service assembly of a walked block's segments:** an `AssembleStart` mark on a
  `mibz2-svc-*` worker;
- **back-set reuse:** more teams' `TeamGot` marks than the hub has back sets (`2 x teams`);
- **front-set reuse:** more service `JobGot` marks than the hub has front sets
  (`2 x teams + services`).

The prepared queue's longest occupancy comes from the `len` of its `Prepared` marks.
`trace` adds only these marks to the service path. A build without `trace` runs the same
cases but doesn't count reach.

### Pass A: this branch's head 7e403c8e (src = 3e201246's plus the changes above)

Build: `cargo build -j2 --profile fuzz --features trace --example fuzz`. The `fuzz` profile
is release with debug assertions and overflow checks.

| run | command | cases | failures |
|---|---|---|---|
| A1 | `fuzz 240 101 <dir> service` | 4,865 | 0 |
| A2 | `fuzz 240 102 <dir> service` | 4,814 | 0 |
| A3 | `fuzz 240 103 <dir> service` | 4,715 | 0 |
| A4 | `fuzz 60 104 <dir> mixed` | 9,104 (1,138 service) | 0 |

Reach, summed over A1–A4. Every case was counted; "longest" is the longest prepared queue
any case reached.

| candidate | cases | cancelled | pooled build | pooled walk | service finish | service assembly | back reuse | front reuse | queue > teams | longest |
|---|---|---|---|---|---|---|---|---|---|---|
| 2x3 + 2 in 8 simd push | 2217 | 282 | 1541 | 1541 | 1879 | 1774 | 688 | 393 | 220 | 3 |
| 1x3 + 1 in 1 simd push | 2220 | 310 | 1534 | 1534 | 1878 | 1787 | 1069 | 779 | 0 | 1 |
| 3x3 + 1 in 12 simd push | 2220 | 272 | 1577 | 1577 | 1917 | 1833 | 379 | 273 | 0 | 3 |
| 1x4 + 2 in 2 simd push | 2220 | 276 | 1543 | 1543 | 1863 | 1770 | 1081 | 577 | 158 | 2 |
| 2x2 + 1 in 3 scalar push | 2220 | 250 | 1549 | 1549 | 1892 | 1800 | 595 | 427 | 0 | 2 |
| 1x2 + 1 in 1 scalar push | 2218 | 264 | 1561 | 1561 | 1901 | 1821 | 1087 | 816 | 0 | 1 |
| 2x3 + 2 in 8 simd reader | 2217 | 313 | 1565 | 1565 | 1906 | 1825 | 751 | 397 | 246 | 3 |
| **all** | **15,532** | 1,967 | **10,870** | 10,870 | 13,236 | 12,610 | 5,650 | 3,662 | 624 | |

So the service path was reached in 10,870 of 15,532 service cases, with a pooled build and
walk on the helpers. The rest failed before any full block reached a team (mostly an
early-header edit or a truncation). There were no failures.

Outcome classes, A1–A3 (14,394 service cases). Every non-cancelled case matched the plain
decoder:

| outcome | cases |
|---|---|
| both fail, Truncated | 3068 |
| both fail, BlockCrc | 2821 |
| both decode, same bytes | 1723 |
| both fail, BadCodeLength | 1563 |
| both fail, BadBlockMagic | 1002 |
| both fail, BadHuffmanCode | 841 |
| cancelled, a prefix of the plain decoder's bytes | 656 |
| both fail, BadSelector | 608 |
| both fail, SelectorOverrun | 482 |
| both fail, BlockTooLong | 427 |
| both fail, StreamCrc | 390 |
| both fail, BadOrigPtr | 338 |
| both fail, BadGroups | 261 |
| both fail, RunUnfinished / Randomised / NotBz2 / RunTooLong / TrailingData / NoSymbols / Empty / BadSelectors | 68 / 58 / 35 / 30 / 9 / 8 / 3 / 3 |

No case was a panic, a timeout, different bytes, or a case where libbzip2 fails and this
decodes.

**The lookahead is an admission threshold, executed.** At svc2's plan (lookahead L = teams
= 2, K = 2 service workers), the prepared queue reached 3 = L + K − 1 blocks:
- 220 cases pushed;
- 246 cases through the reader.

At `1x4 + 2` (L = 1, K = 2) it reached 2. With one service worker it never passed L. This
is D045's bound, observed in the Rust decoder rather than the model.

### Pass B: #271's head e64469f6, admission thresholds 2, 3 and 4

#271 (`research/service-lookahead`) adds `StreamDecoder::service_admission` and
`BzReader::parallel_service_admission`. They are not on this branch's base, and this
branch does not take them in. Instead, the second pass ran in a separate worktree:
- at e64469f6, whose `src/` is #271's exactly;
- with this branch's `tests/common/mod.rs` and `examples/fuzz.rs` checked out;
- with the overlay `2026-11-09-service-core-b5-admission.patch` (beside this note) applied
  on top. The overlay adds an `admission` field to `Serviced`, the admission candidates,
  and the front-set count `teams + admission + services` for reuse.

To reproduce:

```
git checkout e64469f6
git checkout <this branch> -- tests/common/mod.rs examples/fuzz.rs
git apply docs/journal/2026-11-09-service-core-b5-admission.patch
```

The candidates, all at svc2's plan unless shown:
- `2x3 + 2 in 8` at `service` (the default threshold);
- `service_admission` at L = 2, 3 and 4, pushed;
- L = 3 and 4 through `parallel_service_admission`;
- `1x3 + 1 in 2` at L = 3, and `2x2 + 1 in 3` at L = 4 on the scalar kernels.

Build in that worktree: `cargo build -j2 --profile fuzz --features trace --example fuzz`
(its own target directory).

| run | command | cases | failures |
|---|---|---|---|
| B1 | `fuzz 180 201 <dir> service` | 3,931 | 0 |
| B2 | `fuzz 180 202 <dir> service` | 3,688 | 0 |

Reach, B1 + B2 (every case counted):

| candidate | cases | cancelled | pooled build | pooled walk | service finish | service assembly | back reuse | front reuse | queue > teams | longest |
|---|---|---|---|---|---|---|---|---|---|
| 2x3 + 2 in 8 simd push | 952 | 131 | 668 | 668 | 812 | 772 | 308 | 159 | 81 | 3 |
| 2x3 + 2 in 8 L2 simd push | 953 | 118 | 661 | 661 | 805 | 768 | 318 | 173 | 100 | 3 |
| 2x3 + 2 in 8 L3 simd push | 953 | 124 | 674 | 674 | 826 | 779 | 305 | 124 | 188 | 4 |
| 2x3 + 2 in 8 L4 simd push | 953 | 116 | 654 | 654 | 822 | 781 | 325 | 87 | 184 | 5 |
| 2x3 + 2 in 8 L3 simd reader | 952 | 112 | 686 | 686 | 832 | 784 | 316 | 121 | 185 | 4 |
| 2x3 + 2 in 8 L4 simd reader | 952 | 150 | 672 | 672 | 812 | 770 | 306 | 97 | 163 | 5 |
| 1x3 + 1 in 2 L3 simd push | 952 | 124 | 669 | 669 | 823 | 784 | 489 | 193 | 97 | 2 |
| 2x2 + 1 in 3 L4 scalar push | 952 | 108 | 656 | 656 | 812 | 770 | 277 | 109 | 1 | 3 |
| **all** | 7619 | 983 | 5340 | 5340 | 6544 | 6208 | 2644 | 1063 | 999 | |

So the service path was reached in 5,340 of 7,619 cases, with 0 failures. The outcome
classes match pass A's: Truncated 1633, BlockCrc 1514, both decode 923, BadCodeLength 765,
BadBlockMagic 515, BadHuffmanCode 474, BadSelector 340, cancelled-prefix 323, and smaller
classes. No panic, no timeout, no mismatch.

**The thresholds engaged.** At svc2's plan with K = 2 service workers, the prepared queue
reached L + K − 1:
- at L = 2 (explicit, or the default): 3;
- at L = 3: 4, in 188 cases pushed and 185 through the reader;
- at L = 4: 5, in 184 cases pushed and 163 through the reader.

So the thresholds #271 times were reached, and the occupancy bound holds there too. At
the one-service and three-slot shapes the window bounds the queue first (longest 2 and 3).

## 2. Mutation records

Each mutant is a hand-made edit, applied by `mutants.py` (`ID` → file, old text, new text;
reproduced in the table) to this branch's head on a throwaway working copy, then restored
with `git checkout -- src`.

Each mutant ran three detectors in order:
1. `cargo test -j2 --lib -- --test-threads=2 service_tests coop::`, under a 240 s timeout;
2. `cargo test -j2 --test service -- --test-threads=2`, whose own watchdog fails a call
   that takes a minute;
3. `fuzz 45 11 - service` (trace build), under a 120 s timeout.

The restored baseline passed all three: lib 25/25, `tests/service.rs` 11/11, and the fuzz
with 0 failures.

The wake mutants also ran 20 times each against each wake test (native), and once under
Miri (`cargo +nightly miri test --lib -- service_tests::miri_`). The helper mutants ran
under Miri against `coop::tests::miri_`.

| ID | invariant | mutant | killed by | survives |
|---|---|---|---|---|
| W1 | wake-up | `take_prepared`'s `notify_all` after a pop deleted | **new** `miri_a_parked_service_worker_wakes_on_the_pop`: 20/20 native, and Miri. Existing `miri_a_team_taking_a_prepared_block_wakes_the_service`: 19/20 native (the earlier reported kill reproduces; the one survivor is the schedule D045 described), and Miri | `tests/service.rs`; fuzz 45 s (other wake sources mask it end to end) |
| W2 | wake-up | that `notify_all` made `notify_one` | **new** handshake test: 20/20 native, and Miri (the one wake-up goes to the parked team, FIFO) | existing wake test 0/20; `tests/service.rs`; fuzz |
| R1 | buffer return | `retire` drops the back set it is given | new `miri_every_retirement_returns_its_buffers`; `miri_service_core_hub` hangs (timeout); 9 of 11 `tests/service.rs` tests hang (watchdog); fuzz hangs (timeout) | — |
| R2 | retirement | preparation error returns its front but never retires (`live` stays) | new retirement test; the two wake tests hang; `an_mtf_error` and `a_bad_orig_ptr` hang (watchdog); fuzz hangs | — |
| R3 | buffer return | a team's panic retires with no back set (not a fresh one) | **new retirement test only** ("a team's panic: the back sets", 1 ≠ 2) | `tests/service.rs` (one panic a decode never runs short); fuzz |
| R4 | buffer return | a preparing panic retires with no front set | **new retirement test only** ("preparing's panic: the front sets", 2 ≠ 3) | `tests/service.rs`; fuzz |
| R5 | buffer return | a cancelled finish retires with no back set | **new retirement test only** ("finishing cancelled: the back sets") | `tests/service.rs`; fuzz |
| H1 | helper completion | a pool helper reports done before running its call | `coop` tests fail or hang; Miri: data race (non-atomic read on `mibz2-coop-1` against deallocation) in `miri_build_fused`; `tests/service.rs` fails; fuzz hangs | — |
| H2 | helper completion | `Pool::run`'s wait returns with one helper still running | three `coop` tests fail; Miri data race; 6 `tests/service.rs` tests fail; fuzz aborts (heap corruption, "corrupted size vs. prev_size") | — |
| H3 | helper completion | after a team's panic, its T-vector not reset | — | **survives everything: equivalent.** Every build clears the table and proves each slot below n written once before its length is set (`build_one_pass`'s check; `FusedBuild` exposes its length only after all shares pass), so no stale entry is ever read. The reset is defence in depth. |
| H4 | helper completion / assembly | finishing publishes without assembling a walked block's segments | 10 of 11 `tests/service.rs` tests; fuzz 845 failures | lib (its hub fixture's blocks are too small to segment) |
| O1 | ordered publication | finishing publishes `Ok` (the truncated output) on an RLE/CRC failure | new retirement test, `miri_service_core_hub`; `a_block_crc_in_the_middle`; fuzz 186 failures | — |
| O2 | ordered publication | a preparation error published at place + 1 | new retirement test and both wake tests; `a_bad_orig_ptr`, `an_mtf_error`; fuzz hangs (the reader waits for the place never published) | — |
| O3 | ordered publication | finishing takes the latest walked block, not the earliest | — | **survives everything: equivalent for bytes and errors.** `ready` is keyed by place, and every walked block is finished whatever the order, so only latency changes. |
| O4 | ordered publication | a walked block's bit replaced by 0 | `miri_service_core_hub`; `a_block_crc_in_the_middle`, `injected_panics`; fuzz 174 failures | — |
| P1 | panic scope | the feeder's unwind guard deleted | **new** `a_panicking_source` (hangs: watchdog) | lib; fuzz (its sources never panic) |

Summary: of 16 mutants, 14 are killed and 2 survive (H3, O3); both survivors are argued
equivalent above. R3, R4 and R5 are killed only by the new retirement test, and W2 only by
the new handshake test. Neither the integration tests nor the fuzz catch these four: one
lost buffer, or a misdirected single wake-up, doesn't stall a decode with the buffer
surplus those runs have. So those two new tests are the only detectors for those
mutation classes.

## 3. The park handshake

`miri_a_parked_service_worker_wakes_on_the_pop` sets up:
- a hub at lookahead 1, holding one prepared block and one waiting job, with no back set;
- a team's first thread in `take_prepared`, parked waiting for a back set;
- a service worker, parked with the prepared queue at its lookahead.

The test waits until each thread's wait count is 1. A count is taken under the hub's lock
just before `Condvar::wait`, which releases that lock only once the thread waits. So a
test holding the lock and seeing the count knows the thread is parked, not merely slow.
Then, under the lock, the test asserts both counts and that nothing has moved. It adds a
back set without any wake-up and pops the block through `take_prepared`. Then it waits up
to 10 s (Miri: 120 s) for the service worker to take its job.

**Its one residual schedule** is a spurious wake-up, which leaves the worker blocked on
the lock rather than in the wait. That is the only way a missing wake-up could go unseen
here.

Results:
- baseline: 20/20 pass, and Miri passes;
- W1 (notification deleted): 20/20 fail, and Miri fails;
- W2 (`notify_one`): 20/20 fail, and Miri fails.

The finite-yield test reproduces the earlier reported kill: W1 fails it 19 times in 20 and
under Miri. It survives once in 20, which is the schedule D045 identified, and it never
kills W2.

## 4. The panic scope: a guard, with a test

The guard was small, so it was added rather than only scoping the contract in the docs.
`a_panicking_source` covers these cases:
- a source whose `read` panics at once;
- one that panics after 10 bytes;
- one that panics half way through the stream;
- one that panics at 5 MB of a 6 MB input, after the feeder's first 4 MiB chunk was
  written, so blocks are in flight.

Each case runs through `BzReader::parallel_service(2, 3, 2, 8)` and through
`BzReader::parallel(.., 3)`, all under the one-minute watchdog. Each must end with an
`io::Error` saying the input's reader panicked. Without the guard (P1) the test hangs.

**The contract now stated** (in `BzReader`'s documentation):
- A panic in the source's `read` cancels the decode, and the next read fails with that
  error. The bytes of earlier blocks may or may not already have been delivered.
- Nothing more is promised. An abort, an out-of-memory one included, ends the process. A
  `read` that blocks for ever is never interrupted: dropping the reader leaves that thread
  to finish it.
- This is a decoder-level recovery, not a watchdog over arbitrary infrastructure failure.

## 5. Miri

`cargo +nightly miri test -j2 --lib -- service_tests::miri_`, at this branch's head, passes
4 of 4 in 47 s. Those four are:
- the two new tests (`miri_a_parked_service_worker_wakes_on_the_pop` and
  `miri_every_retirement_returns_its_buffers`);
- the two existing ones (`miri_a_team_taking_a_prepared_block_wakes_the_service` and
  `miri_service_core_hub`).

Under Miri:
- W1 fails both wake tests;
- W2 fails the handshake;
- H1 and H2 are each reported as a data race in `coop::tests::miri_build_fused`.

The new tests use tiny scalar blocks, as the existing hub fixture does: Miri covers the
hub's ownership and wake-ups, not the AVX2 kernels or pooled inversion at production
block sizes (B7: that coverage remains the fuzz's, natively, above). CI's Miri job
(`cargo miri test --lib -- miri_`) picks the new tests up by their prefix.

## What this does not show

- **No interleaving proof.** The fuzz and the mutants are sampled schedules on 4 cores. The
  handshake controls one schedule.
- **Spurious wake-ups and timing-only liveness are out of scope.** No mutant tested a
  wall-clock shutdown bound.
- **The `trace` build is not the release binary.** It adds relaxed loads, and marks at
  level 1. The service code it exercises is otherwise the same source.
- **Pass B tests #271's src as it stands at e64469f6.** If #271's src changes, the pass
  must be re-run.
