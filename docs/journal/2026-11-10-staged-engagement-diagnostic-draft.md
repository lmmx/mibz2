# 2026-11-10: The staged walk's native engagement diagnostic (C1, DRAFT, revision 3, not frozen)

**Status: DRAFT.** Not registered and not frozen; nothing built or run for it.

**The history of this draft:**

| revision | at | what it answered |
|---|---|---|
| 1 | `ffb56891` | — |
| 2 | `25b778cd` | the Researcher's REVISE (#145 6759658271) |
| 3 | this head | the Researcher's recheck (#145 6760210997; `state/reports/team-designs-recheck-6760095177` at `ddd3479c`), and it cites the Guardian's **D087** (#145 6760280679; `state/records/D087/decision.md` at `5059ce63`) |

**It becomes a registration** only after the Researcher's PROCEED, with every pin in §4
filled and committed before any data.

**Authority:** the Guardian's D084 (G2.1, item 4), #145 6759487306; record
`state/records/D084` at `f4012c4f`. This is the native-instrumentation option, framed as a
separate engagement diagnostic.

## 0. What it is not

- **It cannot PASS the counted run.**
  - The counted run (`2026-11-10-staged-traverse-count.md`) stays FAIL-MEASUREMENT, with
    its binary deviation recorded.
  - A native execution is a different execution: it is never spliced into the callgrind
    run's 2(b), or combined with its 9.966 Ir/D.
  - The pass-earned forced-team entitlement stays unearned.
- **No timing qualification and no performance inference.**
- **Not a qualifying count, and not a gate change.** Relaxing the old gate is RULES D3,
  Louis's.
- **No sweep and no repeat:** one sequence, ended by its first invalid execution (§4).
- **ENGAGED would establish only that this instrumented execution was balanced.**
  - Local increments and record writes can change register pressure, spills and
    scheduling, and matched counters do not prove matched perturbation between two
    different loops.
  - So ENGAGED says nothing about uninstrumented balance, or about the cause of the
    counted run's imbalance.

**D087 (1) resolves the sequence's terms:**
- **The first INVALID execution ends the sequence INCONCLUSIVE.** The order is old,
  cleanup, staged. An invalid old arm means the staged arm is never launched.
- **No rerun and no method switch.**
- **A VALID but unbalanced old arm** is reference evidence, not an abort.
- **A VALID staged arm that fails a predicate** is NOT ENGAGED, with no retry.
- **D084's ambiguous sentence** ("a second invalid execution … ends this diagnostic
  sequence") is superseded for this sequence; the original record is kept.

## 1. The information question

On one instrumented native execution on a GitHub-hosted runner, does the staged walk
distribute its work among each team's helpers within the counted run's engagement
predicates? The predicates are unchanged, and evaluated from this execution's own records
alone.

## 2. The instrument: the `walk-stats` feature (to be built, then pinned)

**A cargo feature, off by default** (as `wait-stats` is). Shared with the team screen's
PR-tier probes (`2026-11-10-team-screen-design-draft.md`, T2).

**Feature off, the decoder is unchanged:**
- every counter compiles away;
- a test checks that the outputs are byte-exact with the feature on and off;
- any `unsafe` it adds goes to an independent B5 review.

**The identities:**
- **team:** an index given to each pipelined team when its pool is spawned, from a
  stats-only counter. Teams are never inferred from equal totals.
- **block:** its `place` in the stream (the scanner's index, carried by `Relay`), with its
  n rows.
- **thread:** the team pool's index, 0 for the leader and 1..k for the helpers, with its
  role.

**Collection, race-free, for leaders and helpers alike:**
- **One ledger per pool thread,** owned by that thread alone: a `Vec` of fixed-size
  records, in the thread's own storage. There is no shared counter in any hot loop, and no
  false sharing between threads.
- **Each record is pushed by the thread that did the work,** on the return from its call:
  - a helper's build share, on the return from `FusedBuild::share`;
  - any thread's walker call, on the return from `RoomWalk::share`. That covers both a
    helper's call and a leader's join through `Relay::join`.
  - the leader's solo block, on the return from `parallel::solo`.
- **One summary record per cooperative block,** pushed by the thread that ends its walk
  (the last decrement in `Relay::walk_and_leave`). It counts independently of the call
  records:
  - S, the deduplicated starts;
  - the sum of the segments' lengths;
  - **the entered set:** a per-block bitmask of the pool indices that entered
    `walk_and_leave`, each thread setting its bit with a Relaxed `fetch_or` on entry,
    before its `share`. The bits are published by the walkers' AcqRel decrement, as the
    output is;
  - **the leader's join:** whether its `Relay::join` CAS succeeded, a separate bit set by
    the leader;
  - **the build's members:** the helpers counted into `built`.
- **At the decoder's end,** each pool's threads hand their ledgers to the pool at join
  (the pool's run end already orders them). The harness child writes them all to
  `AB_WALK_STATS` as JSON lines. Nothing is written from a hot path, and no record can
  interleave with another.

**Per walker-call record:** (team, block, thread, role); the walker (`traverse` or
`traverse_staged`); successful claims (takes); exhausting claims (0 or 1); walked bytes by
kind (take, full phase and tail); and the staged walker's fragments.

**What a claim is:**
- a **successful claim** is a `fetch_add` that returned an index below S. It is one take,
  and the take writes the segment's first byte;
- an **exhausting claim** is one that returned S or more. There is exactly one per walker
  call: the call that first finds the queue empty, after which the call returns;
- **bytes are counted arithmetically:** per chunk (the lanes' flushed spans), per take, and
  per tail step. There are no per-byte atomics.

## 3. Validity: identity, reconciliation and conservation (each check independent)

**An execution is VALID only if all of these hold.** Otherwise it is INVALID.

1. **The process:**
   - exit 0, no signal, an empty `err`;
   - 209,700,196 bytes, hash `a8830caee5c511a6`;
   - each stage within its bound (§4).
2. **The records:**
   - every line parses with its full schema;
   - every counter is a nonnegative integer within its bound (no overflow);
   - the file is complete: its last line is a terminal summary with the run's record count.
3. **The keys:** unique (arm, team, block, thread, record kind) keys, with no duplicates.
4. **Block membership:**
   - the cooperative blocks' places are exactly the frozen set of 233 places, from W's
     frozen class sequence;
   - each block belongs to exactly one team;
   - each has both of its team's helpers' share records;
   - each has exactly one block summary.
5. **Team rosters:** each team's members and roles are exactly one leader and two helpers,
   the same members in every block of that team.
6. **Reconciling calls with records, per block.** The completed walker-call records must be
   exactly the summary's entered set: the same threads, no more and no fewer.
   - **A lost record** is a thread in the entered set with no call record. That makes the
     execution INVALID, **even when the lost call did zero work** (no take, no bytes, its
     one exhausting claim).
   - **An absent call** is a thread not in the entered set, and so with no record. That is
     a fact about the run, judged by the predicates: a helper absent from its block fails
     (a), making the outcome NOT ENGAGED.
   - So a deleted zero-work record can no longer pass as an absent call. The terminal
     record's count is not used as a witness.
   - **The leader:** its call record must exist if and only if the summary's join bit is
     set. A lost leader record is INVALID.
   - **The build:** each helper's share record must exist if and only if the helper is
     counted in `built`.
7. **Per-block conservation,** against the summary:
   - the successful claims sum to its S;
   - the walked bytes sum to its segment lengths;
   - each call record has exactly one exhausting claim.
8. **The totals** reconcile. Walked bytes summed over the 233 cooperative blocks =
   **M = 209,697,662**.
   - **M counts** only the cooperative walkers' bytes (take, full phase and tail over the
     T blocks' segments).
   - **The 198 S blocks** are walked by the leader's solo path. They have their own solo
     records, are not walker calls, and are not in M.
   - **The S blocks' check:** their rows plus M plus the T blocks' unwalked rows (none on
     W, every T block one cycle) reconcile to N = 209,706,557.

**A failure of any check makes the execution INVALID,** whatever its balance.

**The checker's own tests,** run before registration on synthetic ledgers, must each fail:
- a missing record;
- **a missing zero-work helper record** (zero bytes, zero takes, one exhausting claim),
  with the terminal count adjusted: the Researcher's counterexample, which passed byte
  and take conservation in revision 2;
- **a missing zero-work leader record,** likewise;
- a helper truly absent from a block (not in the entered set), which must give NOT
  ENGAGED, not INVALID;
- a duplicate record;
- a record attributed to the wrong team or block;
- a missing summary;
- a byte count off by one;
- a negative or overflowed counter.

## 4. The fixed single sequence: the first invalid execution ends it

**The host:** a GitHub-hosted runner (`ubuntu-latest`), in one job dispatched once.
- **The topology is recorded, not assumed:** the actual CPU model, sockets, cores, threads,
  vCPUs, cgroup CPU and memory quotas, and the kernel, from `ab env`, `/proc/cpuinfo`,
  `lscpu`, `nproc` and `/sys/fs/cgroup`.
- **The context:** at 9 threads the team oversubscribes the runner. That is part of the
  context, not an equivalence with the counted run's serialised execution.
- **No i9 generalisation, and no C5 run,** follows.

**The pins, filled as far as they can be now:**

| pin | value |
|---|---|
| input | `wikidata.bz2`, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`; output 209,700,196 B, hash `a8830caee5c511a6`, SHA-256 `3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf` (the lbzip2 oracle, `bench/team-screen/manifest.json`) |
| cooperative block set | the 233 T places in `bench/team-screen/w-classes.txt` |
| toolchain | the runner's `dtolnay/rust-toolchain@stable`, **pinned to 1.99.1** (rustc `58cf8f9c3`, cargo `5f94df478`), as the counted run used |
| build | `cargo build --release --features walk-stats --example ab`; release profile (fat LTO, one codegen unit, line tables only) |
| harness record | `ab-child/2`, with `AB_SHA256=1` |
| run | `AB_WALK_STATS=FILE ab child mibz2-pipe3-both 9 wikidata.bz2`; `AB_STAGED_TRAVERSE=1` for the staged arm |

**Still to fill** (they do not exist yet):
- the source commit (the instrument's head, with the inlining arrangement fixed);
- the executable's SHA-256 (recorded after the build, before either arm);
- the checker (`bench/walk_stats_check.py`): its digest and version.

Bounds alone are not readiness: none of this is registrable until those are filled.

**Hard bounds,** in seconds:

| stage | bound |
|---|--:|
| the build | 900 |
| each child | 120 |
| collection: the ledger's write and the copy into the artifact | 60 |
| the judge (the checker) | 120 |
| finalisation: hashing and the artifact upload | 120 |
| the whole job | 1,800 |

A bound exceeded is an INVALID execution.

**The sequence,** in this order:
1. **The old arm:** `AB_WALK_STATS=old.jsonl ab child mibz2-pipe3-both 9 wikidata.bz2`.
2. **Cleanup:** the old arm's process has exited, and its threads and files are closed.
3. **The staged arm:** `AB_WALK_STATS=staged.jsonl AB_STAGED_TRAVERSE=1 ab child
   mibz2-pipe3-both 9 wikidata.bz2`.

**The first INVALID execution ends the diagnostic: INCONCLUSIVE** (D087 (1)).
- If the old arm is invalid, the staged arm is not launched (after cleanup).
- **A VALID but unbalanced old arm** is reference evidence, not an abort.
- **No rerun,** automatic or manual, no method switch, and no selection among executions.

**Everything is retained:** records, child JSON, stdout, stderr, exit statuses, the
topology record, the checker's output, and the build log, whether valid or not. The
child JSON's `wall` and `cpu` fields are incidental and unused.

**The order is fixed and descriptive.** A difference between the arms cannot be read as
caused by the algorithm or the scheduler.

## 5. The predicates (unchanged) and the outcomes

**Evaluated on the staged arm's VALID execution,** per team by its explicit identity:
- **(a)** in every block of its team, each helper's walker calls equal its build shares
  (1 each).
- **(b)** helper-only, per team, across its assigned blocks. With h₁ and h₂ the two
  helpers' total walked bytes over the team's blocks (the leader excluded, as in D084):
  **3·max(h₁, h₂) ≤ 2·(h₁ + h₂)**, compared in integers, with **h₁ + h₂ > 0**. A zero
  denominator is not ENGAGED. No global fraction across teams.
- **(c)** at least one leader called `traverse_staged`.
- **(d)** `traverse` has no calls in the staged arm. In each arm, every walker record names
  that arm's selected walker.

**The old arm** is evaluated as a reference, not a gate, on the same identities and
conservation. Its analogue of (c) is that at least one leader called `traverse`; it cannot
literally require `traverse_staged`. Its (d) is that `traverse_staged` has no calls.

**Reported descriptively, never gated:** for each block, each helper's and the leader's
bytes and shares, and the empty-claim calls. An aggregate-balanced team can hide
alternating monopolies, (100, 0) then (0, 100). That is a reason to show the
distribution, not a per-block gate added after the data.

**The outcomes,** with validity and the predicates kept apart in the report:

| outcome | when |
|---|---|
| **INCONCLUSIVE** | any execution INVALID (§3, §4) |
| **NOT ENGAGED** | valid, and any of (a)–(d) fails on the staged arm |
| **ENGAGED** | valid, and (a)–(d) all hold on the staged arm: this instrumented execution's balance, and nothing more (§0) |

Each outcome is reported under this diagnostic's own registration, beside the counted
run's FAIL-MEASUREMENT, never in its place. A later qualifying design would need its own
justification under C6.

## 6. Its price (model; the Optimiser prices it)

**The instrument, about 120–200 lines, all feature-gated:**
- the ledgers and records in `src/coop.rs` and `src/parallel.rs`;
- the writer in `examples/ab.rs`;
- the checker, `bench/walk_stats_check.py`.

**Its tests:**
- feature on and off give the same bytes;
- conservation holds on the crafted tables;
- the checker's negative controls (§3) all fail.

**The job:**
- one build with the feature: about 30 s warm, 5–15 min cold;
- two native decodes of `wikidata.bz2` at 9 threads, each about 1 s;
- the checker, in seconds.

No retry, so no doubling. The cost is engineering, not machine time.

## 7. Resolved

**From revision 2 (the recheck):**
- **A lost zero-work record** is now caught. The calls are reconciled against the
  summary's independently counted entered set, join bit and build members, and a lost
  record is told apart from an absent call (§3).
- **M** is defined (§3, item 8).
- **The pins** are filled as far as they can be (§4).
- **D087** is cited for the sequence's terms.

**From revision 1:**

- **The host:** the runner, chosen; its actual topology and quotas recorded.
- **The retry:** removed. The first invalid execution ends INCONCLUSIVE; the Guardian is to
  confirm.
- **(b)'s denominator:** per team, across its assigned blocks, helper-only, in integers. The
  per-block distribution is descriptive only.

## 8. The instrument, as built

**Where it lives:**
- **The instrument is #338 on master** (`bench/walk-stats`): master's own paths. That is
  `coop::traverse` through `RoomWalk::share_with`, `FusedBuild::share`, the relay's
  summary, and `parallel::solo`, with the checker, its tests and the `walk-stats`
  workflow.
- **#334 adds only `traverse_staged`'s hooks:** the `Stat::fragment` method and the
  staged walker's claim, segment, tail and fragment counts.
- **This section was first written on the stacked branch `research/walk-stats`,** which
  #338 and the merge into #334 replace.

The `walk-stats` feature now exists. **This section is the instrument's record. It does not
freeze or register anything,** and nothing in the diagnostic's sequence has run.

### Where it differs from §2's design, and why

- **One lock per record, not ledgers handed over at pool join.** Each record is pushed whole
  under one `Mutex` (`coop::walk_stats::push`) by the thread that did the work, once per
  call. That is never inside a walk's loop, and no record can interleave with another or
  vanish on a thread's return.
  - Handing ledgers over at pool join would need the pool's run to carry them out. The
    lock costs a few uncontended lock operations per block.
- **The team identity** is a per-thread index (`walk_stats::team`). A pipelined team's
  leader takes it the first time it asks (its `pipelined_worker` thread) and passes it to
  the run's helpers in the relay (`RelayStats::team`). Solo records read the same index on
  the leader.
- **Counting is arithmetic, through a `Stat` trait** whose every method is a no-op for `()`:
  - each claim is counted as a take or an exhaustion;
  - each finished segment adds its length;
  - each tail byte adds 1;
  - each non-empty staged fragment adds its length.

  Without the feature, the walkers are called with `()`.

### The records

| record | fields |
|---|---|
| `call` | team, block (`place`), thread (pool index, 0 = leader), role, walker, takes, exhausts, bytes, tail, fragments, copied |
| `share` | team, block, thread, after a completed `FusedBuild::share` |
| `summary` | team, block, rows, starts, `seg_bytes`, `entered`, `joined`, `built`, by the thread ending the walk |
| `solo` | team, block, rows |
| `end` | the record count |

- **The summary counts independently of the calls:**
  - `seg_bytes` is the segments' lengths summed from the scratch;
  - `entered` is a bitmask each thread sets on entering `walk_and_leave`, before its share;
  - `joined` is set by the leader's successful join CAS;
  - `built` is set by each helper as it counts into the build.
- **The harness** (`examples/ab.rs`, feature only) writes the records to `AB_WALK_STATS`
  after the child's own record.

### What was checked (no diagnostic run, and no engagement figure read)

- **Without the feature, the machine code is unchanged.** `coop::traverse::<32>` (1,018
  instructions), `coop::traverse_staged::<32>` (871) and `bwt::room_segments::<32>` (1,366)
  are instruction-identical, addresses aside, between `02547e7b`'s `ab` and this branch's
  featureless `ab` (`objdump`).
- **No new `unsafe`.** The two `finish` closures compute their segment's length once and
  reuse it: the same operations, split.
- **With the feature,** `cargo test --features walk-stats` passes the library's 63 tests
  (the walk, staged and pipelined oracles), `tests/pipelined.rs` (12) and
  `tests/staged_traverse.rs` (1).
- **`walk_stats_reconcile_per_block`** runs pipelined decodes at k = 2, 3 and 4:
  - **The coverage:** both walkers, with the leader early and late.
  - **The checks:** every output equals `walk`'s, and every block's records reconcile.
  - **The mutants it catches:** a lost call record, a zero-work one included (at least
    one zero-work call is required to occur); a duplicate; a byte miscounted; the wrong
    walker; a lost share.
- **Feature on against feature off** on two generated corpora (`records` and `text` at
  scale 4; not W), `mibz2-pipe3-both` at 3 threads, both walkers:
  - the outputs are identical (length and SHA-256);
  - the records are VALID by `bench/walk_stats_check.py`.
- **`bench/walk_stats_check.py`** is §3 and §5's checker. `bench/test_walk_stats_check.py`
  has 17 negative controls and positive cases:
  - a lost zero-work helper record and a lost zero-work leader record: INVALID;
  - a helper truly absent (not entered): NOT ENGAGED;
  - a duplicate; a misattributed team or block; a missing summary or solo record; a byte
    off by one; a negative, overflowed, string or boolean counter; the terminal record; an
    exhaust count other than 1; a lost share: all INVALID;
  - an unbalanced team, an exact two-thirds boundary, a zero denominator, the other
    walker: judged as §5 says.

### Still open before registration

- **The pins:** the instrument's commit (this branch's head, once reviewed), the executable's
  digest, and the checker's digest.
- **B5 review** of the instrument.
- **CI runs the feature's tests** in #338's `walk-stats` workflow, a new workflow for
  review.
