# 2026-11-10: The staged walk's native engagement diagnostic (C1, DRAFT, revision 2, not frozen)

**Status: DRAFT.** Not registered and not frozen; nothing built or run for it.
- **Revision 2** applies the Researcher's REVISE (#145 6759658271; report
  `state/reports/team-designs-6759589291` at `5326b6fd`). Revision 1 was at `ffb56891`.
- **It becomes a registration** only after the Researcher's PROCEED and the Guardian's
  confirmation of the resolved invalidity rule (§4). Every pin below is filled in and
  committed before any data.

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
  (the last decrement in `Relay::walk_and_leave`):
  - S, the deduplicated starts;
  - the sum of the segments' lengths;
  - the walkers counted in.
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

## 3. Validity: identity and conservation (each check independent)

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
6. **Per-block conservation,** checked against the summary, which counts differently from
   the walker records:
   - the walker records' successful claims sum to the summary's S;
   - their walked bytes sum to the summary's segment lengths;
   - the exhausting claims equal the block's walker calls.
7. **The totals** reconcile: walked bytes over all blocks = 209,697,662.

**A failure of any check makes the execution INVALID,** whatever its balance.

**The checker's own tests,** run before registration on synthetic ledgers, must each fail:
- a missing record;
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

**The pins,** to be filled in and committed before the job:
- the source commit, with the instrument and the inlining arrangement fixed;
- the toolchain (rustc, cargo) and the build command, `cargo build --release --features
  walk-stats --example ab`;
- the executable's SHA-256, recorded after the build and before either arm;
- the input's digest (`82f65ace…`);
- the checker's digest and its version.

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

**The first INVALID execution ends the diagnostic: INCONCLUSIVE.**
- If the old arm is invalid, the staged arm is not launched (after cleanup).
- **No rerun,** automatic or manual, and no selection among executions.

This is the stricter of the readings of D084's "a second invalid execution is retained and
ends this diagnostic sequence", which is ambiguous beside its no-retry terms. The Guardian
is asked to confirm it before registration.

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

## 7. Resolved from revision 1

- **The host:** the runner, chosen; its actual topology and quotas recorded.
- **The retry:** removed. The first invalid execution ends INCONCLUSIVE; the Guardian is to
  confirm.
- **(b)'s denominator:** per team, across its assigned blocks, helper-only, in integers. The
  per-block distribution is descriptive only.
