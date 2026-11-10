# 2026-11-10: The staged walk's native engagement diagnostic (C1, DRAFT, not frozen)

**Status: DRAFT for pricing (Optimiser) and PROCEED (Researcher). Not registered and not
frozen.** Nothing is built or run for it. It becomes a registration only after the
Researcher's PROCEED, with every pin below filled in and committed before any data.

**Authority:** the Guardian's D084 (G2.1, item 4), #145 6759487306; record
`state/records/D084` at `f4012c4f`. This is the native-instrumentation option, framed as a
separate engagement diagnostic under C1.

## 0. What it is not

- **It cannot PASS the counted run.** The counted run (`2026-11-10-staged-traverse-count.md`,
  FAIL-MEASUREMENT) stays failed, its result and the binary deviation unchanged.
  - A native balance is a different execution from the serialised callgrind one, so it is
    never spliced into that run's 2(b). It cannot be combined with that run's 9.966 Ir/D
    to claim the original PASS.
  - The pass-earned forced-team entitlement stays unearned.
- **No timing qualification and no performance inference:** the instrument counts bytes
  and calls, not time.
- **Not a qualifying count, and not a gate change.** Relaxing the old gate is RULES D3,
  Louis's.
- **No sweep:** of thread counts, schedulers, inputs or repeats.

## 1. The information question

On a native, concurrent execution of the same team arm, do the staged and unstaged walks
distribute the walk among a team's helpers within the registered engagement predicates?

The predicates are the counted run's condition 2, **unchanged**, including 2(b)'s
helper-only two-thirds bar. Every predicate is evaluated from this run's own records alone.

## 2. The instrument (to be built, then pinned)

**A cargo feature `walk-stats`,** off by default, like `wait-stats`. Without it, the
decoder is unchanged: every counter compiles away. The `examples/ab` child writes the
records when `AB_WALK_STATS=PATH` is set.

**The identities, stable and explicit:**
- **team:** an index given to each pipelined team when its pool is spawned
  (`Pool::pipelined`), from a stats-only counter. It is recorded with each block's records.
  Teams are no longer inferred by pairing equal share counts.
- **block:** the block's `place` in the stream (the scanner's index, already carried by
  `Relay`), with its row count n.
- **thread:** the team pool's thread index, 0 for the leader and 1..k for the helpers,
  with its role.

**Per (team, block, thread)** walker call, accumulated in locals and written once when
`share` returns (no shared atomic in the walk's loop):
- the walker (`traverse` or `traverse_staged`) and its calls (0 or 1);
- the build share (0 or 1);
- claims: successful, and exhausting;
- walked bytes by kind: take, full phase and tail;
- the staged walker's flush fragments and copied bytes.

**Per (team, block)**, written by `RoomWalk::finish`: S (the deduplicated starts), and
the sum of the segments' lengths (`s.bufs[..S]`).

**The perturbation, stated:**
- **The cost:** a few register increments a step at most (the take, full-phase and tail
  byte counts are derived per chunk and per tail step), and one record write a share. It
  is not zero, and it is not a timing measurement.
- **Matched:** the old and staged arms carry the same instrumentation.

## 3. Conservation and invalidity

**An execution is valid only if all of these hold:**

1. The process exits 0, with an empty `err`. Its output is 209,700,196 bytes with hash
   `a8830caee5c511a6`.
2. Every cooperative block (233) has records from exactly one team, with both helpers'
   build shares.
3. For each block, the walked bytes summed over its threads equal its segments' summed
   lengths.
4. For each block, the successful claims equal S, and the exhausting claims equal its
   walker calls.
5. Every record carries all identities, and the records parse.
6. Over the run, the walked bytes sum to 209,697,662.

**A failure of any of these makes the execution INVALID,** whatever its balance.

## 4. The fixed single sequence

**The pins,** to be filled in and committed before any execution:
- **source:** this diagnostic's branch head, with the `walk-stats` instrument and the
  inlining arrangement both fixed;
- **toolchain and build:** rustc 1.99.1, `cargo build --release --features walk-stats
  --example ab`; the binary's SHA-256;
- **input:** `wikidata.bz2` (SHA-256 `82f65ace…c2c29915`);
- **host:** one GitHub-hosted `ubuntu-latest` runner, in one job dispatched once. The CPU
  model, vCPUs and kernel are recorded from `ab env`. The 9 threads oversubscribe its 4
  vCPUs, as the counted run's 9 threads did on the shared machine. Hardware balance on the
  i9 is not this question.
- **affinity:** none, the harness's default for this arm.

**The sequence,** exactly once, in this order:
1. `AB_WALK_STATS=old.jsonl ab child mibz2-pipe3-both 9 wikidata.bz2`;
2. `AB_WALK_STATS=staged.jsonl AB_STAGED_TRAVERSE=1 ab child mibz2-pipe3-both 9 wikidata.bz2`.

**Retention and repeats:**
- Every attempt is retained: records, child JSON, stdout, stderr and exit code, valid or
  not.
- If either execution is INVALID, the sequence runs once more, in the same order.
- A second invalid execution of either arm is retained and **ends the diagnostic**:
  INCONCLUSIVE.
- **No other repeats,** and no selection among runs.

**The incidental figures:** the child JSON's `wall` and `cpu` fields are retained and
unused, as in the counted run.

## 5. The predicates (unchanged) and the outcomes

**Evaluated on the staged arm's valid execution,** per team by its explicit identity:
- **(a)** every helper's walker calls equal its build shares, in every block of its team;
- **(b)** no helper walked more than two thirds of its team's helper-walked bytes. Leaders
  are excluded from the denominator, as in D084;
- **(c)** at least one leader called `traverse_staged`;
- **(d)** `traverse` has no calls in the staged arm, and `traverse_staged` none in the old
  arm.

**The old arm** is evaluated on the same predicates, as a reference, not a gate.

**The outcomes:**
- **ENGAGED:** the staged arm meets (a)–(d).
- **NOT ENGAGED:** it fails any of them.
- **INCONCLUSIVE:** §4's end.

Each is reported under this diagnostic's own registration, beside, never in place of, the
counted run's FAIL-MEASUREMENT.

**What ENGAGED would mean:**
- **It shows** that natively, on this runner, the staged walk's work is shared within the
  bar;
- **it does not show** the callgrind run's balance, an instruction count, or any time.

**The next step after either outcome** would be a separately justified prospective design
under C6, if anyone proposes one.

## 6. Its price (model; for the Optimiser)

| part | cost |
|---|---|
| the instrument | about 80–150 lines in `src/coop.rs` and `examples/ab.rs`, all feature-gated |
| the instrument's tests | conservation on the crafted tables; feature on and off giving the same bytes |
| the job | one build with the feature (about 30 s warm, 5–10 min cold) and two native decodes of `wikidata.bz2` at 9 threads (about 1 s each) |
| at most | twice that, by §4's single retry |

**The cost is engineering, not machine time.**

## 7. Open questions for the Researcher

1. **Host:** a GitHub-hosted 4-vCPU runner (cheap; oversubscribed, as the counted run was),
   or a C5 workstation run (the i9, where `Plan::for_threads` picks this very plan, but a
   Louis-reserved run)? This draft proposes the runner and leaves the i9 out.
2. **One retry:** does it match D084's "a second invalid execution ends the sequence", or
   should the first invalid execution end it?
3. **(b)'s denominator:** it sums a team's helper bytes over all its blocks, as the counted
   run did. Should a per-block distribution also be reported, descriptively only?
